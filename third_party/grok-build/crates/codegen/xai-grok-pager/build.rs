use std::path::PathBuf;
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// The files whose change moves `git rev-parse HEAD`: this checkout's HEAD and
/// the ref it points at (loose, else packed-refs). `.git/HEAD` relative to this
/// crate never exists — the crate is not the repository root, and a linked
/// worktree's `.git` is a file — so cargo reran this script, and recompiled the
/// pager, on every build.
fn git_commit_inputs() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(git_dir) = git(&["rev-parse", "--absolute-git-dir"]) {
        paths.push(PathBuf::from(git_dir).join("HEAD"));
    }
    if let (Some(common), Some(head_ref)) = (
        git(&["rev-parse", "--path-format=absolute", "--git-common-dir"]),
        git(&["rev-parse", "--symbolic-full-name", "HEAD"]),
    ) {
        let common = PathBuf::from(common);
        let loose = common.join(&head_ref);
        if head_ref.starts_with("refs/") && loose.is_file() {
            paths.push(loose);
        } else {
            paths.push(common.join("packed-refs"));
        }
    }
    paths.retain(|p| p.is_file());
    paths
}

fn main() {
    for path in git_commit_inputs() {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!("cargo:rerun-if-env-changed=DEEPSEEK_BUILD_VERSION");
    println!("cargo:rerun-if-env-changed=GROK_VERSION");

    let commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Product SemVer first (DeepSeek Build), then Grok override, then crate version.
    let version = std::env::var("DEEPSEEK_BUILD_VERSION")
        .or_else(|_| std::env::var("GROK_VERSION"))
        .or_else(|_| std::env::var("CARGO_PKG_VERSION"))
        .unwrap_or_else(|_| "0.0.0".to_string());

    // Version-derived cfg: sccache hashes the rustc command line (incl. --cfg),
    // so a product version change forces a cache miss. env!-based injection
    // alone is not sccache-keyed and shipped a stale version (5.5.1 labeled
    // 5.5.0) across warm-cache release builds.
    println!("cargo:rustc-check-cfg=cfg(dsb_build_marker)");
    println!("cargo:rustc-cfg=dsb_build_marker=\"{}\"", version);

    println!(
        "cargo:rustc-env=VERSION_WITH_COMMIT={} ({})",
        version, commit
    );

    // ALSO write a generated file read via include_str! in lib.rs, so sccache
    // keys on the file CONTENT (guaranteed cache miss on version change).
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR set by cargo");
    std::fs::write(
        std::path::Path::new(&out_dir).join("version_with_commit.txt"),
        format!("{version} ({commit})"),
    )
    .expect("write version_with_commit.txt");
}
