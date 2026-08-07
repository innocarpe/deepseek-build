//! Installed CLI version, lockstepped with shipping binaries.
//!
//! DeepSeek Build product: prefer `DEEPSEEK_BUILD_VERSION` (runtime, set by
//! `dsb` / npm wrapper) so the TUI shows product SemVer (`5.0.0`) instead of
//! the vendored pager crate version (`0.2.x`).

use semver::Version;

pub const TEST_VERSION_ENV: &str = "GROK_TEST_VERSION";
/// Product SemVer override (DeepSeek Build). Preferred over Grok-internal env.
pub const PRODUCT_VERSION_ENV: &str = "DEEPSEEK_BUILD_VERSION";

pub const VERSION: &str = match option_env!("DEEPSEEK_BUILD_VERSION") {
    Some(v) => v,
    None => match option_env!("GROK_VERSION") {
        Some(v) => v,
        None => env!("CARGO_PKG_VERSION"),
    },
};

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

/// Format the compiled version with a channel label for user-facing display.
///
/// `channel_label` is a pre-formatted suffix such as `" [alpha]"`, `" [stable]"`,
/// or `""` (empty when no cached pointer is available). Obtain it from
/// `xai_grok_update::channel_label()`.
///
/// Example: `"0.2.5 [stable]"` or `"0.2.5 [alpha]"`.
pub fn display_version(channel_label: &str) -> String {
    format!("{}{}", VERSION, channel_label)
}

/// Format a version-with-commit string with a channel label.
///
/// Same semantics as [`display_version`] but for the full
/// `"0.2.5 (abc1234)"` string.
pub fn display_version_with_commit(version_with_commit: &str, channel_label: &str) -> String {
    format!("{}{}", version_with_commit, channel_label)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Display formatting invariant matrix — verifies label appending
    /// works correctly across all label states (alpha, stable, empty).
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
        // display_version uses compiled VERSION — just verify the label appends
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
}
