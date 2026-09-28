use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use tokio::io::AsyncReadExt;
use tokio::process::Command;

const ATTR_LOOKUP_TIMEOUT: Duration = Duration::from_secs(2);
const ATTR_REAP_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_ATTR_OUTPUT_BYTES: u64 = 16 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineEnding {
    Lf,
    Crlf,
}

impl LineEnding {
    fn restore(self, text: &str) -> String {
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        match self {
            Self::Lf => normalized,
            Self::Crlf => normalized.replace('\n', "\r\n"),
        }
    }
}

/// Normalize content for a path that did not exist before this write.
///
/// Attributes are resolved for each creation only when the filesystem backend
/// confirms that `path` names the same target on the host. Virtual/client-backed
/// filesystems use the platform fallback instead of host repository policy.
pub(super) async fn for_new_file(
    path: &Path,
    content: &str,
    path_is_on_host_filesystem: bool,
) -> String {
    if path_is_on_host_filesystem {
        for_new_file_with(
            path,
            content,
            std::ffi::OsStr::new("git"),
            ATTR_LOOKUP_TIMEOUT,
            platform_default(),
        )
        .await
    } else {
        platform_default().restore(content)
    }
}

async fn for_new_file_with(
    path: &Path,
    content: &str,
    git: &std::ffi::OsStr,
    timeout: Duration,
    fallback: LineEnding,
) -> String {
    let ending = git_attribute_line_ending(path, git, timeout)
        .await
        .unwrap_or(fallback);
    ending.restore(content)
}

fn platform_default() -> LineEnding {
    if cfg!(windows) {
        LineEnding::Crlf
    } else {
        LineEnding::Lf
    }
}

async fn git_attribute_line_ending(
    path: &Path,
    git: &std::ffi::OsStr,
    timeout: Duration,
) -> Option<LineEnding> {
    let (working_dir, relative_path) = git_query_path(path)?;
    let mut command = Command::new(git);
    command
        .arg("--literal-pathspecs")
        .arg("-C")
        .arg(&working_dir)
        .arg("check-attr")
        .arg("-z")
        .arg("text")
        .arg("eol")
        .arg("--")
        .arg(&relative_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let (mut child, process_group) = crate::util::global_process_scope().spawn(command).ok()?;
    let mut stdout = child.stdout.take()?.take(MAX_ATTR_OUTPUT_BYTES + 1);
    let mut output = Vec::new();

    let result = tokio::time::timeout(timeout, async {
        let (status, read) = tokio::join!(child.wait(), stdout.read_to_end(&mut output));
        let status = status.ok()?;
        read.ok()?;
        if !status.success() || output.len() as u64 > MAX_ATTR_OUTPUT_BYTES {
            return None;
        }
        parse_check_attr_output(&output)
    })
    .await;

    match result {
        Ok(ending) => ending,
        Err(_) => {
            let _ = process_group.kill();
            let _ = xai_tty_utils::reap_killed_bounded(&mut child, ATTR_REAP_TIMEOUT).await;
            None
        }
    }
}

/// Find a working directory Git can enter and a path relative to it. The
/// nearest existing parent lets `check-attr` evaluate paths before creation.
fn git_query_path(path: &Path) -> Option<(PathBuf, PathBuf)> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(path)
    };
    let mut working_dir = absolute.parent()?;
    while !working_dir.is_dir() {
        working_dir = working_dir.parent()?;
    }
    let relative_path = absolute.strip_prefix(working_dir).ok()?.to_path_buf();
    if relative_path.as_os_str().is_empty() {
        return None;
    }
    Some((working_dir.to_path_buf(), relative_path))
}

fn parse_check_attr_output(output: &[u8]) -> Option<LineEnding> {
    let mut fields = output.split(|byte| *byte == 0);
    let path_text = fields.next()?;
    let text_attribute = fields.next()?;
    let text_value = fields.next()?;
    let path_eol = fields.next()?;
    let eol_attribute = fields.next()?;
    let eol_value = fields.next()?;
    if fields
        .next()
        .is_some_and(|trailing| !trailing.is_empty() || fields.next().is_some())
        || path_text != path_eol
        || text_attribute != b"text"
        || eol_attribute != b"eol"
        || text_value == b"unset"
    {
        return None;
    }
    match eol_value {
        b"lf" => Some(LineEnding::Lf),
        b"crlf" => Some(LineEnding::Crlf),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command as StdCommand, Stdio as StdStdio};
    use tempfile::TempDir;

    fn git_repo() -> TempDir {
        let repo = TempDir::new().unwrap();
        let status = StdCommand::new("git")
            .arg("init")
            .arg("--quiet")
            .current_dir(repo.path())
            .stdout(StdStdio::null())
            .stderr(StdStdio::null())
            .status()
            .expect("git is required for Git attribute tests");
        assert!(status.success());
        let status = StdCommand::new("git")
            .args([
                "config",
                "--local",
                "core.attributesFile",
                if cfg!(windows) { "NUL" } else { "/dev/null" },
            ])
            .current_dir(repo.path())
            .stdout(StdStdio::null())
            .stderr(StdStdio::null())
            .status()
            .unwrap();
        assert!(status.success());
        repo
    }

    #[cfg(unix)]
    fn fake_git(directory: &Path, body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let path = directory.join("fake-git");
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        let mut permissions = std::fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&path, permissions).unwrap();
        path
    }

    #[tokio::test]
    async fn eol_attributes_override_the_platform_default_and_keep_eof_choice() {
        let repo = git_repo();
        std::fs::write(
            repo.path().join(".gitattributes"),
            "lf.txt text eol=lf\r\ncrlf.txt text eol=crlf\r\n",
        )
        .unwrap();

        let lf = for_new_file_with(
            &repo.path().join("lf.txt"),
            "one\r\ntwo",
            std::ffi::OsStr::new("git"),
            ATTR_LOOKUP_TIMEOUT,
            LineEnding::Crlf,
        )
        .await;
        let crlf = for_new_file_with(
            &repo.path().join("crlf.txt"),
            "one\ntwo\r\n",
            std::ffi::OsStr::new("git"),
            ATTR_LOOKUP_TIMEOUT,
            LineEnding::Lf,
        )
        .await;

        assert_eq!(lf, "one\ntwo");
        assert_eq!(crlf, "one\r\ntwo\r\n");
    }

    #[tokio::test]
    async fn nested_attributes_handle_missing_parents_spaces_unicode_and_dash_paths() {
        let repo = git_repo();
        std::fs::write(repo.path().join(".gitattributes"), "*.txt text eol=lf\n").unwrap();
        let nested = repo.path().join("space β");
        std::fs::create_dir(&nested).unwrap();
        std::fs::write(nested.join(".gitattributes"), "*.txt text eol=crlf\n").unwrap();
        let target = nested.join("not-created").join("-new file.txt");

        let content = for_new_file_with(
            &target,
            "first\nsecond",
            std::ffi::OsStr::new("git"),
            ATTR_LOOKUP_TIMEOUT,
            LineEnding::Lf,
        )
        .await;

        assert_eq!(content, "first\r\nsecond");
    }

    #[tokio::test]
    async fn invalid_unset_text_and_unavailable_git_use_fallback() {
        let repo = git_repo();
        std::fs::write(
            repo.path().join(".gitattributes"),
            "*.bad text eol=sideways\n*.bin -text eol=crlf\n",
        )
        .unwrap();
        let fallback = LineEnding::Lf;
        for name in ["file.bad", "file.bin", "unmatched.md"] {
            let content = for_new_file_with(
                &repo.path().join(name),
                "one\r\ntwo",
                std::ffi::OsStr::new("git"),
                ATTR_LOOKUP_TIMEOUT,
                fallback,
            )
            .await;
            assert_eq!(content, "one\ntwo", "path={name}");
        }

        let missing_git = repo.path().join("missing-git-executable");
        let content = for_new_file_with(
            &repo.path().join("missing-git.txt"),
            "one\r\ntwo",
            missing_git.as_os_str(),
            ATTR_LOOKUP_TIMEOUT,
            fallback,
        )
        .await;
        assert_eq!(content, "one\ntwo");
    }

    #[tokio::test]
    async fn non_repository_uses_fallback() {
        let directory = TempDir::new().unwrap();
        let target = directory.path().join("outside.txt");
        let fallback = LineEnding::Crlf;
        let outside = for_new_file_with(
            &target,
            "one\ntwo",
            std::ffi::OsStr::new("git"),
            ATTR_LOOKUP_TIMEOUT,
            fallback,
        )
        .await;
        assert_eq!(outside, "one\r\ntwo");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn slow_git_is_terminated_within_the_lookup_bound() {
        let repo = git_repo();
        let fake = fake_git(repo.path(), "exec /bin/sleep 5");
        let started = std::time::Instant::now();
        let content = for_new_file_with(
            &repo.path().join("slow.txt"),
            "one\ntwo",
            fake.as_os_str(),
            Duration::from_millis(75),
            LineEnding::Crlf,
        )
        .await;
        assert_eq!(content, "one\r\ntwo");
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn descendant_holding_stdout_cannot_extend_the_lookup_bound() {
        let repo = git_repo();
        let fake = fake_git(repo.path(), "(/bin/sleep 0.3) &\nexit 0");
        let started = std::time::Instant::now();
        let content = for_new_file_with(
            &repo.path().join("slow-output.txt"),
            "one\ntwo",
            fake.as_os_str(),
            Duration::from_millis(75),
            LineEnding::Crlf,
        )
        .await;
        assert_eq!(content, "one\r\ntwo");
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[tokio::test]
    async fn attributes_are_read_again_for_each_creation() {
        let repo = git_repo();
        let attrs = repo.path().join(".gitattributes");
        std::fs::write(&attrs, "*.txt text eol=crlf\n").unwrap();
        let path = repo.path().join("new.txt");
        assert_eq!(
            for_new_file_with(
                &path,
                "one\ntwo",
                std::ffi::OsStr::new("git"),
                ATTR_LOOKUP_TIMEOUT,
                LineEnding::Lf,
            )
            .await,
            "one\r\ntwo"
        );

        std::fs::write(&attrs, "*.txt text eol=lf\n").unwrap();
        assert_eq!(
            for_new_file_with(
                &path,
                "one\ntwo",
                std::ffi::OsStr::new("git"),
                ATTR_LOOKUP_TIMEOUT,
                LineEnding::Crlf,
            )
            .await,
            "one\ntwo"
        );
    }

    #[test]
    fn parser_requires_the_expected_nul_delimited_records() {
        assert_eq!(
            parse_check_attr_output(b"some path\0text\0set\0some path\0eol\0crlf\0"),
            Some(LineEnding::Crlf)
        );
        assert_eq!(
            parse_check_attr_output(b"x\0text\0unset\0x\0eol\0lf\0"),
            None
        );
        assert_eq!(
            parse_check_attr_output(b"x\0text\0set\0x\0eol\0other\0"),
            None
        );
        assert_eq!(parse_check_attr_output(b"x text set\nx eol lf\n"), None);
    }
}
