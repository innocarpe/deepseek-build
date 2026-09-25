//! Test harness for the vendored release-channel overlay.
//!
//! The function lives next to `xai-grok-update`'s `channel_label` so the
//! binary and this crate compile the same file. Building `xai-grok-update`
//! pulls in the vendored pager; this crate does not.

#[path = "../../../third_party/grok-build/crates/codegen/xai-grok-update/src/product_channel.rs"]
mod product_channel;

pub use product_channel::{
    channel_label_for, channel_name_for, compare_channel, release_semver_omits_channel_label,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_with_a_stale_pointer_has_no_alpha_suffix() {
        // Measured on a 6.0.0 release binary: version.json still said
        // stable_version 5.7.0, and --version printed "[alpha]".
        assert_eq!(channel_label_for("6.0.0", Some("5.7.0")), "");
        assert_eq!(channel_name_for("6.0.0", Some("5.7.0")), Some("stable"));
    }

    #[test]
    fn release_suffix_does_not_depend_on_the_pointer() {
        let releases = ["6.0.0", "6.0.0\n", "  1.2.3", "6.0.0+build.5"];
        let pointers = [
            None,
            Some("5.7.0"),
            Some("6.0.0"),
            Some("9.9.9"),
            Some("garbage"),
        ];
        for current in releases {
            for pointer in pointers {
                assert_eq!(
                    channel_label_for(current, pointer),
                    "",
                    "label for {current:?} pointer {pointer:?}"
                );
                assert_eq!(channel_name_for(current, pointer), Some("stable"));
            }
        }
    }

    #[test]
    fn prerelease_still_uses_the_upstream_comparison() {
        assert_eq!(channel_label_for("6.1.0-alpha.1", None), "");
        assert_eq!(
            channel_label_for("6.1.0-alpha.1", Some("6.0.0")),
            " [alpha]"
        );
        assert_eq!(
            channel_label_for("6.1.0-alpha.1", Some("6.1.0-alpha.1")),
            " [stable]"
        );
        // semver: a release is greater than its pre-release, so this build
        // is not ahead of the pointer.
        assert_eq!(
            channel_label_for("6.1.0-alpha.1", Some("6.1.0")),
            " [stable]"
        );
        assert_eq!(channel_name_for("6.1.0-alpha.1", None), None);
        assert_eq!(
            channel_name_for("6.1.0-alpha.1", Some("6.0.0")),
            Some("alpha")
        );
    }

    #[test]
    fn compare_channel_keeps_the_upstream_matrix() {
        // Copied from xai-grok-update version.rs::test_derive_channel_matrix.
        // A release that is merely ahead of the pointer is still "alpha"
        // HERE. channel_label_for is the overlay that stops printing it.
        let cases = [
            ("0.1.220-alpha.2", "0.1.219", Some("alpha")),
            ("0.1.219", "0.1.219", Some("stable")),
            ("0.1.218", "0.1.219", Some("stable")),
            ("0.1.220-alpha.2", "0.1.220-alpha.2", Some("stable")),
            ("0.1.220-alpha.2", "0.1.220", Some("stable")),
            ("0.2.5", "0.2.3", Some("alpha")),
            ("0.2.5", "0.2.5", Some("stable")),
            ("0.2.3", "0.2.5", Some("stable")),
            ("0.2.0", "0.2.0", Some("stable")),
            ("0.2.0", "0.1.219", Some("alpha")),
            ("0.1.220-alpha.2", "0.2.0", Some("stable")),
            ("garbage", "0.1.219", None),
            ("0.1.219", "garbage", None),
            ("", "0.1.219", None),
            ("0.1.219", "", None),
            ("6.0.0", "5.7.0", Some("alpha")),
        ];
        for (current, stable, expected) in cases {
            assert_eq!(
                compare_channel(current, stable),
                expected,
                "compare_channel({current:?}, {stable:?})"
            );
        }
        // The display overlay and the raw comparison disagree on purpose
        // for a release SemVer. Losing either side is a regression.
        assert_eq!(compare_channel("0.2.5", "0.2.3"), Some("alpha"));
        assert_eq!(channel_label_for("0.2.5", Some("0.2.3")), "");
        assert_eq!(channel_label_for("6.0.0", Some("5.7.0")), "");
    }

    #[test]
    fn unparseable_current_has_no_channel() {
        assert!(!release_semver_omits_channel_label("not-a-version"));
        assert_eq!(channel_label_for("not-a-version", Some("5.7.0")), "");
        assert_eq!(channel_name_for("not-a-version", Some("5.7.0")), None);
        assert_eq!(channel_name_for("not-a-version", None), None);
    }

    #[test]
    fn display_paths_call_the_overlay() {
        let updater = include_str!(
            "../../../third_party/grok-build/crates/codegen/xai-grok-update/src/version.rs"
        );
        assert!(
            updater.contains("product_channel::channel_label_for("),
            "channel_label must delegate to channel_label_for"
        );
        assert!(
            updater.contains("product_channel::channel_name_for("),
            "channel_name must delegate to channel_name_for"
        );
        let shell = include_str!(
            "../../../third_party/grok-build/crates/codegen/xai-grok-shell/src/util/config/resolve/version.rs"
        );
        assert!(
            shell.contains("product_channel::channel_name_for("),
            "inspect's channel must delegate to channel_name_for"
        );
    }
}
