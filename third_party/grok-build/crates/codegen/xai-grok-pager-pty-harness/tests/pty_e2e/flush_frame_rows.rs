// Per-test-case module for the `pty_e2e` integration test crate.
#[allow(unused_imports)]
use super::common::*;

// Flush frame: the outer margin carries no vertical padding at any terminal size,
// so the status bar starts on row 0 whether the terminal is tall or short.
//
// These cases used to pin the opposite — a blank top padding row at tall sizes
// that auto-compact removed. That row is gone at every size now, so the frame is
// flush everywhere; auto-compact's remaining layout effect is the prompt gap, and
// the derivation itself is pinned by the dispatch unit tests
// (`resize_below_threshold_derives_compact_without_touching_user_setting`).

/// A height short enough to engage auto-compact.
/// It is above `SHORT_TERMINAL_ROWS` (16) but at most `AUTO_COMPACT_MAX_ROWS` (20).
const SHORT_ROWS: u16 = 18;

/// Index of the first screen row with any non-whitespace content, panicking with the screen when the whole screen is blank.
fn first_content_row(harness: &PtyHarness, when: &str) -> u16 {
    let screen = harness.screen_contents();
    screen
        .lines()
        .position(|line| !line.trim().is_empty())
        .unwrap_or_else(|| panic!("{when}: screen is entirely blank\nscreen:\n{screen}")) as u16
}

/// The flush frame holds at a tall size, after shrinking into the auto-compact
/// zone, and after growing back: the status bar owns row 0 and no blank margin
/// row appears, so the scrollback keeps the row a padded frame would spend on it.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn flush_frame_holds_status_bar_on_row_zero() {
    let content = ContentController::start().await.expect("start content");
    content.set_response(format!("{MOCK_RESPONSE_SENTINEL} flush probe."));

    let binary = pager_binary().expect("resolve pager binary");
    let mut harness =
        PtyHarness::spawn_with_content(&binary, DEFAULT_ROWS, DEFAULT_COLS, &content, &[])
            .expect("spawn pager with content");

    harness
        .wait_for_text(WELCOME_SCREEN_SENTINEL, WELCOME_TIMEOUT)
        .expect("welcome text");

    // Enter the agent view (the layout under test) and let the turn end.
    harness
        .inject_keys(format!("{PROMPT}\r").as_bytes())
        .expect("submit prompt");
    harness
        .wait_for_text(MOCK_RESPONSE_SENTINEL, Duration::from_secs(30))
        .expect("mock response rendered");
    harness.update(Duration::from_millis(500));

    let tall_row = first_content_row(&harness, "tall spawn");
    assert_eq!(
        tall_row,
        0,
        "a tall terminal starts at the status bar row, got first content on \
         row {tall_row}\nscreen:\n{}",
        harness.screen_contents()
    );

    // Shrink into the auto-compact zone: the frame stays flush.
    harness
        .resize(SHORT_ROWS, DEFAULT_COLS)
        .expect("resize short");
    harness.update(Duration::from_millis(900));
    assert!(
        harness.is_running().expect("poll pager liveness"),
        "pager exited during resize\nscreen:\n{}",
        harness.screen_contents()
    );
    let short_row = first_content_row(&harness, "after shrink");
    assert_eq!(
        short_row,
        0,
        "the flush frame stays flush at {SHORT_ROWS} rows (status bar on row 0)\
         \nscreen:\n{}",
        harness.screen_contents()
    );

    // Grow back: still flush.
    harness
        .resize(DEFAULT_ROWS, DEFAULT_COLS)
        .expect("resize tall");
    harness.update(Duration::from_millis(900));
    let restored_row = first_content_row(&harness, "after grow");
    assert_eq!(
        restored_row,
        0,
        "growing back keeps the status bar on row 0, got first content on \
         row {restored_row}\nscreen:\n{}",
        harness.screen_contents()
    );
    assert!(
        !harness.screen_contents().contains("panicked"),
        "pager rendered 'panicked'\nscreen:\n{}",
        harness.screen_contents()
    );

    harness.quit().expect("clean quit");
}

/// **A tiny spawn is flush from the first frame.**
/// No resize event ever fires here, so the startup size read is the only thing
/// that can have laid the frame out; the status bar must still land on row 0.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn flush_frame_at_startup() {
    let content = ContentController::start().await.expect("start content");
    content.set_response(format!("{MOCK_RESPONSE_SENTINEL} startup probe."));

    let binary = pager_binary().expect("resolve pager binary");
    let mut harness =
        PtyHarness::spawn_with_content(&binary, SHORT_ROWS, DEFAULT_COLS, &content, &[])
            .expect("spawn pager with content");

    // The welcome menu may be trimmed on a short spawn; the prompt marker always paints
    // The first char promotes to the agent view under test
    harness
        .wait_for_text("\u{276f}", WELCOME_TIMEOUT)
        .expect("prompt marker");
    harness
        .inject_keys(format!("{PROMPT}\r").as_bytes())
        .expect("submit prompt");
    harness
        .wait_for_text(MOCK_RESPONSE_SENTINEL, Duration::from_secs(30))
        .expect("mock response rendered");
    harness.update(Duration::from_millis(500));

    let row = first_content_row(&harness, "tiny spawn");
    assert_eq!(
        row,
        0,
        "a {SHORT_ROWS}-row spawn starts at the status bar row (no top padding \
         row) without any resize\nscreen:\n{}",
        harness.screen_contents()
    );
    assert!(
        !harness.screen_contents().contains("panicked"),
        "pager rendered 'panicked'\nscreen:\n{}",
        harness.screen_contents()
    );

    harness.quit().expect("clean quit");
}
