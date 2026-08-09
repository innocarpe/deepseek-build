fn main() {
    println!("cargo:rerun-if-env-changed=DEEPSEEK_BUILD_VERSION");
    println!("cargo:rerun-if-env-changed=GROK_VERSION");

    // Resolve the product SemVer: DEEPSEEK_BUILD_VERSION -> GROK_VERSION ->
    // crate version. Emit BOTH a version-derived cfg (belt) AND a generated
    // file read via include_str! (suspenders). The cfg is sccache-keyed via
    // the rustc command line; the generated file is sccache-keyed via its
    // CONTENT (a file hash), which the cfg alone cannot guarantee across
    // cache providers. env!/option_env! values are not sccache-keyed at all —
    // that gap shipped stale versions (5.5.1 labeled 5.5.0, 5.5.2 labeled
    // 5.5.1) across warm-cache release builds.
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

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR set by cargo");
    std::fs::write(
        std::path::Path::new(&out_dir).join("product_version.txt"),
        version.trim(),
    )
    .expect("write product_version.txt");
}
