use std::ffi::OsStr;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::snippets::LineEnding;

const ATTR_LOOKUP_TIMEOUT: Duration = Duration::from_secs(2);
const ATTR_POLL_INTERVAL: Duration = Duration::from_millis(10);
const MAX_ATTR_OUTPUT_BYTES: u64 = 16 * 1024;

/// Resolve Git's current EOL policy for one new file, with a platform fallback.
/// The lookup is deliberately not cached so edits to `.gitattributes` apply to
/// the next creation.
pub(crate) fn normalize_new_file(path: &Path, content: &str) -> String {
    normalize_new_file_with(
        path,
        content,
        OsStr::new("git"),
        ATTR_LOOKUP_TIMEOUT,
        platform_default(),
    )
}

fn normalize_new_file_with(
    path: &Path,
    content: &str,
    git: &OsStr,
    timeout: Duration,
    fallback: LineEnding,
) -> String {
    git_attribute_line_ending(path, git, timeout)
        .unwrap_or(fallback)
        .restore(content)
}

pub(crate) fn platform_default() -> LineEnding {
    if cfg!(windows) {
        LineEnding::Crlf
    } else {
        LineEnding::Lf
    }
}

fn git_attribute_line_ending(path: &Path, git: &OsStr, timeout: Duration) -> Option<LineEnding> {
    let (working_dir, relative_path) = git_query_path(path)?;
    let deadline = Instant::now() + timeout;
    let mut child = Command::new(git)
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
        .spawn()
        .ok()?;
    let stdout = child.stdout.take()?;
    let (sender, receiver) = mpsc::sync_channel(1);
    let _reader = match thread::Builder::new().spawn(move || {
        let _ = sender.send(read_limited(stdout));
    }) {
        Ok(reader) => reader,
        Err(_) => {
            kill_and_reap(child);
            return None;
        }
    };

    let status = loop {
        if Instant::now() >= deadline {
            kill_and_reap(child);
            return None;
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                let remaining = deadline.saturating_duration_since(Instant::now());
                thread::sleep(ATTR_POLL_INTERVAL.min(remaining));
            }
            Err(_) => {
                kill_and_reap(child);
                return None;
            }
        }
    };
    if !status.success() {
        return None;
    }
    let remaining = deadline.saturating_duration_since(Instant::now());
    let output = receiver.recv_timeout(remaining).ok()?.ok()?;
    if output.len() as u64 > MAX_ATTR_OUTPUT_BYTES {
        return None;
    }
    parse_check_attr_output(&output)
}

/// Kill the owned process immediately and transfer any remaining reap work to
/// a native waiter thread so cleanup cannot extend the attribute lookup bound.
fn kill_and_reap(mut child: Child) {
    let _ = child.kill();
    if !matches!(child.try_wait(), Ok(Some(_))) {
        let _ = thread::Builder::new()
            .name("dsb-git-attr-reaper".into())
            .spawn(move || {
                let _ = child.wait();
            });
    }
}

fn read_limited(mut stdout: impl Read) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();
    stdout
        .by_ref()
        .take(MAX_ATTR_OUTPUT_BYTES + 1)
        .read_to_end(&mut output)?;
    Ok(output)
}

/// Resolve a path relative to the nearest existing parent so Git can apply
/// repository and nested `.gitattributes` rules before missing directories are
/// created.
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
    let mut fields: Vec<&[u8]> = output.split(|byte| *byte == 0).collect();
    if fields.last().is_some_and(|field| field.is_empty()) {
        fields.pop();
    }
    if fields.len() != 6
        || fields[0] != fields[3]
        || fields[1] != b"text"
        || fields[4] != b"eol"
        || fields[2] == b"unset"
    {
        return None;
    }
    match fields[5] {
        b"lf" => Some(LineEnding::Lf),
        b"crlf" => Some(LineEnding::Crlf),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
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
            .expect("git is required for attribute tests");
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
        let path = directory.join("fake-git");
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        let mut permissions = std::fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&path, permissions).unwrap();
        path
    }

    #[test]
    fn attributes_override_fallback_and_preserve_eof_choice() {
        let repo = git_repo();
        std::fs::write(
            repo.path().join(".gitattributes"),
            "lf.txt text eol=lf\r\ncrlf.txt text eol=crlf\r\n",
        )
        .unwrap();
        assert_eq!(
            normalize_new_file_with(
                &repo.path().join("lf.txt"),
                "one\r\ntwo",
                OsStr::new("git"),
                ATTR_LOOKUP_TIMEOUT,
                LineEnding::Crlf,
            ),
            "one\ntwo"
        );
        assert_eq!(
            normalize_new_file_with(
                &repo.path().join("crlf.txt"),
                "one\ntwo\r\n",
                OsStr::new("git"),
                ATTR_LOOKUP_TIMEOUT,
                LineEnding::Lf,
            ),
            "one\r\ntwo\r\n"
        );
    }

    #[test]
    fn nested_attribute_and_quoted_unicode_path_apply_before_parent_exists() {
        let repo = git_repo();
        std::fs::write(repo.path().join(".gitattributes"), "*.txt text eol=lf\n").unwrap();
        let nested = repo.path().join("space β");
        std::fs::create_dir(&nested).unwrap();
        std::fs::write(nested.join(".gitattributes"), "*.txt text eol=crlf\n").unwrap();
        let path = nested.join("missing").join("-new file.txt");
        assert!(!path.parent().unwrap().exists());

        assert_eq!(
            normalize_new_file_with(
                &path,
                "one\ntwo",
                OsStr::new("git"),
                ATTR_LOOKUP_TIMEOUT,
                LineEnding::Lf,
            ),
            "one\r\ntwo"
        );
    }

    #[test]
    fn unset_invalid_missing_and_failed_attributes_use_fallback() {
        let repo = git_repo();
        std::fs::write(
            repo.path().join(".gitattributes"),
            "*.bad text eol=sideways\n*.bin -text eol=crlf\n",
        )
        .unwrap();
        let fallback = LineEnding::Lf;
        for name in ["file.bad", "file.bin", "unmatched.md"] {
            assert_eq!(
                normalize_new_file_with(
                    &repo.path().join(name),
                    "one\r\ntwo",
                    OsStr::new("git"),
                    ATTR_LOOKUP_TIMEOUT,
                    fallback,
                ),
                "one\ntwo",
                "path={name}"
            );
        }

        let missing_git = repo.path().join("missing-git-executable");
        assert_eq!(
            normalize_new_file_with(
                &repo.path().join("missing-git.txt"),
                "one\r\ntwo",
                missing_git.as_os_str(),
                ATTR_LOOKUP_TIMEOUT,
                fallback,
            ),
            "one\ntwo"
        );
        let outside = TempDir::new().unwrap();
        assert_eq!(
            normalize_new_file_with(
                &outside.path().join("outside.txt"),
                "one\ntwo",
                OsStr::new("git"),
                ATTR_LOOKUP_TIMEOUT,
                LineEnding::Crlf,
            ),
            "one\r\ntwo"
        );
    }

    #[test]
    fn non_repository_creation_writes_the_platform_fallback() {
        let outside = TempDir::new().unwrap();
        let path = outside.path().join("outside.txt");
        let git_status = StdCommand::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(outside.path())
            .stdout(StdStdio::null())
            .stderr(StdStdio::null())
            .status()
            .expect("git is required for the non-repository fallback test");
        assert!(!git_status.success(), "fixture must be outside Git");

        let supplied = "one\r\ntwo\n";
        let expected = platform_default().restore(supplied);
        let mut store = crate::snippets::SnippetStore::new();
        let returned = store.write_new(&path, supplied).unwrap();

        assert_eq!(returned, expected);
        assert_eq!(std::fs::read(&path).unwrap(), expected.as_bytes());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), expected);
    }

    #[cfg(unix)]
    #[test]
    fn slow_git_is_terminated_within_the_lookup_bound() {
        let repo = git_repo();
        let start = Instant::now();
        let fake = fake_git(repo.path(), "exec /bin/sleep 5");
        let output = normalize_new_file_with(
            &repo.path().join("new.txt"),
            "one\ntwo",
            fake.as_os_str(),
            Duration::from_millis(75),
            LineEnding::Crlf,
        );
        assert_eq!(output, "one\r\ntwo");
        assert!(start.elapsed() < Duration::from_secs(2));
    }

    #[cfg(unix)]
    #[test]
    fn descendant_holding_stdout_cannot_extend_the_lookup_bound() {
        let repo = git_repo();
        let fake = fake_git(repo.path(), "(/bin/sleep 0.3) &\nexit 0");
        let start = Instant::now();
        let output = normalize_new_file_with(
            &repo.path().join("new.txt"),
            "one\ntwo",
            fake.as_os_str(),
            Duration::from_millis(75),
            LineEnding::Crlf,
        );
        assert_eq!(output, "one\r\ntwo");
        assert!(start.elapsed() < Duration::from_millis(500));
    }

    #[test]
    fn attributes_are_not_cached_between_creations() {
        let repo = git_repo();
        let attrs = repo.path().join(".gitattributes");
        let path = repo.path().join("new.txt");
        std::fs::write(&attrs, "*.txt text eol=crlf\n").unwrap();
        assert_eq!(
            normalize_new_file_with(
                &path,
                "one\ntwo",
                OsStr::new("git"),
                ATTR_LOOKUP_TIMEOUT,
                LineEnding::Lf,
            ),
            "one\r\ntwo"
        );
        std::fs::write(&attrs, "*.txt text eol=lf\n").unwrap();
        assert_eq!(
            normalize_new_file_with(
                &path,
                "one\ntwo",
                OsStr::new("git"),
                ATTR_LOOKUP_TIMEOUT,
                LineEnding::Crlf,
            ),
            "one\ntwo"
        );
    }

    #[test]
    fn parser_rejects_unexpected_nul_delimited_output() {
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
