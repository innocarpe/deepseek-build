//! Installed CLI version, lockstepped with shipping binaries.
//!
//! DeepSeek Build product: prefer `DEEPSEEK_BUILD_VERSION` (runtime, set by
//! `dsb` / npm wrapper) so the TUI shows product SemVer (`5.0.0`) instead of
//! the vendored pager crate version (`0.2.x`).

#![deny(clippy::indexing_slicing)]

use std::sync::OnceLock;

use semver::Version;

pub const TEST_VERSION_ENV: &str = "GROK_TEST_VERSION";
/// Product SemVer override (DeepSeek Build). Preferred over Grok-internal env.
pub const PRODUCT_VERSION_ENV: &str = "DEEPSEEK_BUILD_VERSION";

/// Build-time product SemVer, baked from a generated file: `build.rs` writes
/// `$OUT_DIR/product_version.txt`. Reading it via `include_str!` makes sccache
/// key on the FILE CONTENT, so a version change always forces a recompile.
/// `env!`/`option_env!` values are not sccache-keyed, which shipped stale
/// versions (5.5.1 labeled 5.5.0, 5.5.2 labeled 5.5.1) across warm-cache
/// release builds.
pub const VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/product_version.txt"));

/// The release pipeline always injects `GROK_VERSION`; without it the build is from source.
pub const IS_DEV_BUILD: bool = option_env!("GROK_VERSION").is_none();

/// Runtime-injected `"<version> (<shortcommit>)"` string.
/// Only the release binary stamps the commit in its own build.rs and injects it here at startup, so the lib crates don't recompile on every commit.
static FULL_VERSION: OnceLock<&'static str> = OnceLock::new();

/// Inject the binary's stamped `"<version> (<shortcommit>)"` string.
/// Idempotent: the first set wins, repeats are ignored.
pub fn set_full_version(v: &'static str) {
    let _ = FULL_VERSION.set(v);
}

/// The injected version-with-commit string, or plain [`VERSION`] when no binary has called [`set_full_version`] (e.g. lib tests, dev harnesses).
pub fn full_version() -> &'static str {
    FULL_VERSION.get().copied().unwrap_or(VERSION)
}

/// Runtime product version, then test override, then compiled [`VERSION`].
///
/// Trimmed so non-semver-aware callers can pass the result straight into parsing.
pub fn installed() -> String {
    if let Ok(v) = std::env::var(PRODUCT_VERSION_ENV) {
        let t = v.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    std::env::var(TEST_VERSION_ENV)
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|_| VERSION.to_string())
}

pub fn installed_semver() -> Result<Version, semver::Error> {
    Version::parse(&installed())
}

/// Formats the compiled version with a channel label for user-facing display, e.g. `"0.2.5 [stable]"`.
/// `channel_label` is pre-formatted by `xai_grok_update::channel_label()`: `" [alpha]"`, `" [stable]"`, or `""` when no pointer is cached.
pub fn display_version(channel_label: &str) -> String {
    format!("{}{}", VERSION, channel_label)
}

/// Like [`display_version`], but for the full `"0.2.5 (abc1234)"` string.
pub fn display_version_with_commit(version_with_commit: &str, channel_label: &str) -> String {
    format!("{}{}", version_with_commit, channel_label)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Checks that the channel label is appended for alpha, stable, and empty labels.
    #[test]
    fn test_display_version_formatting_matrix() {
        let cases: &[(&str, &str, &str)] = &[
            // (version_with_commit,    label,        expected_suffix)
            ("0.2.5 (abc1234)", " [alpha]", "0.2.5 (abc1234) [alpha]"),
            ("0.2.5 (abc1234)", " [stable]", "0.2.5 (abc1234) [stable]"),
            ("0.2.5 (abc1234)", "", "0.2.5 (abc1234)"),
            (
                "0.1.220-alpha.2 (def0)",
                " [alpha]",
                "0.1.220-alpha.2 (def0) [alpha]",
            ),
        ];
        for (vwc, label, expected) in cases {
            assert_eq!(
                display_version_with_commit(vwc, label),
                *expected,
                "display_version_with_commit({:?}, {:?})",
                vwc,
                label,
            );
        }
        // display_version uses compiled VERSION, so verify only that the label appends
        assert_eq!(display_version(""), VERSION);
        assert!(display_version(" [stable]").ends_with("[stable]"));
    }

    /// DeepSeek Build: product env must win over compiled vendor crate version.
    #[test]
    fn product_version_env_overrides_compiled() {
        // SAFETY: test-only env mutation in serial unit test.
        unsafe {
            std::env::set_var(PRODUCT_VERSION_ENV, "5.0.0");
            std::env::remove_var(TEST_VERSION_ENV);
        }
        assert_eq!(installed(), "5.0.0");
        unsafe {
            std::env::remove_var(PRODUCT_VERSION_ENV);
        }
    }

    #[test]
    fn full_version_falls_back_then_first_set_wins() {
        assert_eq!(full_version(), VERSION);
        set_full_version("first (aaaaaaa)");
        assert_eq!(full_version(), "first (aaaaaaa)");
        set_full_version("second (bbbbbbb)");
        assert_eq!(full_version(), "first (aaaaaaa)");
    }
}
