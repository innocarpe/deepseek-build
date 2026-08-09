use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-env-changed=GROK_VERSION");
    println!("cargo:rerun-if-env-changed=DEEPSEEK_BUILD_VERSION");

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

    // ALSO write a generated file read via include_str! in main.rs, so sccache
    // keys on the file CONTENT (guaranteed cache miss on version change) —
    // the belt to the --cfg suspenders. env!-based injection alone shipped
    // stale versions (5.5.1 labeled 5.5.0, 5.5.2 labeled 5.5.1).
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR set by cargo");
    std::fs::write(
        std::path::Path::new(&out_dir).join("version_with_commit.txt"),
        format!("{version} ({commit})"),
    )
    .expect("write version_with_commit.txt");
}
