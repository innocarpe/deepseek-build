fn main() {
    println!("cargo:rerun-if-env-changed=DEEPSEEK_BUILD_VERSION");
    println!("cargo:rerun-if-env-changed=GROK_VERSION");

    // Emit a version-derived cfg so sccache keys on it: sccache hashes the
    // rustc command line (including --cfg), so a product version change
    // forces a cache miss. env!/option_env! values alone are NOT part of
    // sccache's key — that gap shipped a stale version (the 5.5.1 binary
    // labeled 5.5.0) across warm-cache release builds.
    println!("cargo:rustc-check-cfg=cfg(dsb_build_marker)");
    let version = std::env::var("DEEPSEEK_BUILD_VERSION")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| {
            std::env::var("GROK_VERSION")
                .ok()
                .filter(|v| !v.trim().is_empty())
        })
        .or_else(|| std::env::var("CARGO_PKG_VERSION").ok())
        .unwrap_or_else(|| "0.0.0".to_string());
    println!("cargo:rustc-cfg=dsb_build_marker=\"{}\"", version);
}
