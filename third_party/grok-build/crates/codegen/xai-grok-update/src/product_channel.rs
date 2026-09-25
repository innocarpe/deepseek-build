//! Channel suffix for DeepSeek Build's `--version` line.
//!
//! Upstream Grok (`derive_channel` in `version.rs`) returns `"alpha"` when the
//! compiled version is strictly newer than the cached stable pointer. That is
//! a real signal there: Grok ships builds ahead of the pointer it calls
//! stable. The comparison itself stays in `derive_channel` and is duplicated
//! here as [`compare_channel`] so a pre-release still follows it. Do not
//! change one without the other — `test_derive_channel_matrix` is the
//! upstream contract, and the tests in `dsb-version-label` repeat that matrix
//! for [`compare_channel`].
//!
//! DeepSeek Build does not publish that kind of build. A release
//! `MAJOR.MINOR.PATCH` is the npm `latest` / GitHub Release line. The pointer
//! in `version.json` is written by the updater on a 30-minute TTL. Install
//! does not write it, and replacing the binary (a source install, a release
//! script) does not write it either. `6.0.0` with `stable_version` `5.7.0`
//! is a stale cache, not an alpha channel.
//!
//! [`channel_label_for`] therefore ignores the pointer for a version whose
//! pre-release list is empty, and the suffix is `""`. The same binary must
//! not flicker between `` and ` [stable]` as the file appears. A pre-release
//! (`6.1.0-alpha.1`) still uses the upstream comparison. [`channel_name_for`]
//! reports `"stable"` for a release version so telemetry does not record the
//! stale-pointer false `"alpha"`.

use semver::Version;

/// `true` when `version` is a release SemVer: it parses, and it has no
/// pre-release identifiers. Build metadata is ignored, matching semver's
/// ordering rules.
pub fn release_semver_omits_channel_label(version: &str) -> bool {
    match Version::parse(version.trim()) {
        Ok(parsed) => parsed.pre.is_empty(),
        Err(_) => false,
    }
}

/// Same comparison as `version.rs::derive_channel`.
///
/// `Some("alpha")` when `current > stable`, `Some("stable")` when
/// `current <= stable`, `None` when either side fails to parse.
pub fn compare_channel(current: &str, stable: &str) -> Option<&'static str> {
    let current_v = Version::parse(current).ok()?;
    let stable_v = Version::parse(stable).ok()?;
    if current_v > stable_v {
        Some("alpha")
    } else {
        Some("stable")
    }
}

/// User-facing suffix: `" [alpha]"`, `" [stable]"`, or `""`.
///
/// A release SemVer always returns `""`, including when `cached_stable` is
/// older (`6.0.0` vs `5.7.0`), newer, equal, or absent. A pre-release with
/// no cached pointer returns `""`, matching the historical
/// `channel_label` behavior. A pre-release with a pointer uses
/// [`compare_channel`].
pub fn channel_label_for(current: &str, cached_stable: Option<&str>) -> &'static str {
    if release_semver_omits_channel_label(current) {
        return "";
    }
    let Some(stable) = cached_stable else {
        return "";
    };
    match compare_channel(current, stable) {
        Some("alpha") => " [alpha]",
        Some(_) => " [stable]",
        None => "",
    }
}

/// Machine-readable channel. A release SemVer is `"stable"` whether or not
/// a pointer is cached. A pre-release is `None` without a pointer, otherwise
/// the upstream comparison.
pub fn channel_name_for(current: &str, cached_stable: Option<&str>) -> Option<&'static str> {
    if release_semver_omits_channel_label(current) {
        return Some("stable");
    }
    let stable = cached_stable?;
    compare_channel(current, stable)
}
