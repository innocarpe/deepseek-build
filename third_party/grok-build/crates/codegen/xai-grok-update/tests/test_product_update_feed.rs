//! Product regression tests: DeepSeek Build update checks must consult the
//! product npm feed and must NEVER advertise (or install) a lower version —
//! in particular the upstream Grok Build channel pointer (`x.ai/cli/stable`),
//! which reports versions like `1.0.0` while the product is at `5.5.0`.
//!
//! User-visible regression being locked in (reported twice):
//!
//! ```text
//! Update: v1.0.0 available — press ctrl+u to restart
//! ```
//!
//! Two compounding causes:
//!   1. Installs with no `GROK_INSTALLER` / config installer were classified
//!      as `"internal"`, whose version source was the Grok x.ai channel
//!      pointer (Grok Build's version, e.g. `1.0.0`).
//!   2. `"internal"` allowed downgrades, so that *lower* Grok version was
//!      advertised as an update — and pressing ctrl+u would download the
//!      Grok Build binary over the product.
//!
//! After the fix: the product default installer is `"npm"`, the internal path
//! reads the product npm feed, and no installer classification may downgrade.

#![cfg(unix)]

mod common;

use serial_test::serial;

use common::{FakeBinGuard, reset_home, set_test_version, test_home};
use xai_grok_update::UpdateConfig;
use xai_grok_update::auto_update::check_update_status;

/// Value the upstream Grok Build channel pointer (`https://x.ai/cli/stable`)
/// returns today. Must never be advertised to DeepSeek Build users.
const GROK_FEED_VERSION: &str = "1.0.0";
/// Realistic installed product version (matches `Cargo.toml` SemVer).
const PRODUCT_VERSION: &str = "5.5.0";

fn make_update_config() -> UpdateConfig {
    UpdateConfig {
        proxy_base_url: "http://test.invalid/v1".to_string(),
        auth_scope: "test".to_string(),
        deployment_key: None,
        alpha_test_key: None,
        channel: "stable".to_string(),
        npm_registry: None,
    }
}

/// Isolate a test home, pin the installed product version, and force the given
/// installer classification via `GROK_INSTALLER` with a fake `npm` on PATH.
fn setup(installer: &str) -> FakeBinGuard {
    let _ = test_home();
    reset_home();
    set_test_version(PRODUCT_VERSION);
    // SAFETY: serial_test ensures no race; reset_home clears this between tests.
    unsafe { std::env::set_var("GROK_INSTALLER", installer) };
    FakeBinGuard::install_npm()
}

// ─────────────────────────────────────────────────────────────────────────────
// Installer classification
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn default_installer_is_npm_not_internal() {
    // Regression: before the fix, an install with no GROK_INSTALLER and no
    // config installer fell through to "internal", which read the Grok x.ai
    // channel pointer. The product must default to npm (its only release feed).
    let _ = test_home();
    reset_home();

    assert_eq!(
        xai_grok_update::auto_update::get_installer().await,
        Some("npm"),
        "product default installer must be npm so update checks use the \
         product feed, never the Grok x.ai channel pointers"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// The user report: installed 5.5.0, feed returns the Grok version 1.0.0
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn internal_install_with_grok_feed_version_reports_no_update() {
    // Exact user scenario: the install is (mis)classified "internal" and the
    // version source returns the Grok Build version. The product must not
    // advertise a downgrade — this is what produced the bogus
    // "Update: v1.0.0 available" banner.
    let g = setup("internal");
    g.set_stdout(&format!("\"{GROK_FEED_VERSION}\""));

    let status = check_update_status(&make_update_config()).await;

    assert_eq!(status.current_version, PRODUCT_VERSION);
    assert_eq!(status.installer.as_deref(), Some("internal"));
    assert_eq!(status.latest_version.as_deref(), Some(GROK_FEED_VERSION));
    assert!(
        !status.update_available,
        "a lower Grok Build version must never be advertised as an update"
    );
    assert!(status.error.is_none(), "fetch succeeded, so no error");
}

#[tokio::test]
#[serial]
async fn npm_install_with_grok_feed_version_reports_no_update() {
    // Same feed value through the npm classification (the product default):
    // still no update, and the product version is never downgraded.
    let g = setup("npm");
    g.set_stdout(&format!("\"{GROK_FEED_VERSION}\""));

    let status = check_update_status(&make_update_config()).await;

    assert_eq!(status.current_version, PRODUCT_VERSION);
    assert_eq!(status.installer.as_deref(), Some("npm"));
    assert!(!status.update_available);
    assert!(status.error.is_none());
}

// ─────────────────────────────────────────────────────────────────────────────
// Genuine product upgrades must still be detected (anti-regression)
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn internal_install_with_newer_product_version_reports_update() {
    // A real product upgrade (5.5.0 -> 5.5.1) must still be advertised,
    // even through the legacy "internal" classification.
    let g = setup("internal");
    g.set_stdout("\"5.5.1\"");

    let status = check_update_status(&make_update_config()).await;

    assert_eq!(status.current_version, PRODUCT_VERSION);
    assert_eq!(status.latest_version.as_deref(), Some("5.5.1"));
    assert!(
        status.update_available,
        "genuine product upgrade must still be detected for internal installs"
    );
    assert!(status.error.is_none());
}

#[tokio::test]
#[serial]
async fn npm_install_with_newer_product_version_reports_update() {
    let g = setup("npm");
    g.set_stdout("\"5.5.1\"");

    let status = check_update_status(&make_update_config()).await;

    assert!(status.update_available);
    assert_eq!(status.latest_version.as_deref(), Some("5.5.1"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Same-version: the common steady-state (installed == npm latest)
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn npm_install_at_latest_reports_no_update() {
    // The daily case for an up-to-date product install: npm returns exactly
    // the installed version. Must be silent.
    let g = setup("npm");
    g.set_stdout(&format!("\"{PRODUCT_VERSION}\""));

    let status = check_update_status(&make_update_config()).await;

    assert_eq!(status.current_version, PRODUCT_VERSION);
    assert_eq!(status.latest_version.as_deref(), Some(PRODUCT_VERSION));
    assert!(!status.update_available);
    assert!(status.error.is_none());
}
