// Per-test-case module for the `pty_e2e` integration test crate.
#[allow(unused_imports)]
use super::common::*;

/// The measured iPhone Orca pane (`stty` on the Orca-managed dsb panes): 55
/// columns x 41 rows. It is the frame the flush pads and the phone-only gates
/// were measured against.
const PHONE_ROWS: u16 = 41;
const PHONE_COLS: u16 = 55;

/// Rows the mock response renders, one per blank-line-separated paragraph, so the
/// transcript overflows the phone pane and the frame is bottom-anchored.
const RESPONSE_PARAGRAPHS: usize = 40;

/// Long unbroken token that wraps at the content width; its columns tell the
/// right edge of the text area straight off the captured frame. A run of dashes
/// would render as a markdown rule instead of wrapping.
const LONG_TOKEN_LEN: usize = 200;

/// Columns an agent message reserves on the right for its timestamp
/// (`timestamp_reserved_for`): the block's text wraps this much narrower than
/// the content area.
const TIMESTAMP_GUTTER: u16 = 10;

/// Where the scrollback text band starts inside the pane: the outer margin, the
/// accent rail, and the rail's gutter (the default `block_pad_left`). The value
/// is width-independent, so the desktop frame honours it too.
const TEXT_BAND_LEFT_INSET: usize = 3;

/// Where the band ends, counted from the pane's last column: the right pad
/// (which mirrors the rail's column) plus the outer-margin column the scrollbar
/// shares. Equal to [`TEXT_BAND_LEFT_INSET`] by design.
const TEXT_BAND_RIGHT_INSET: u16 = 3;

fn phone_response() -> String {
    let mut body = String::new();
    for i in 1..=RESPONSE_PARAGRAPHS {
        body.push_str(&format!("P{i:02} density probe paragraph\n\n"));
    }
    body.push_str(&"x".repeat(LONG_TOKEN_LEN));
    body
}

/// Index of the first screen row with any non-whitespace content, panicking with the
/// screen when the whole screen is blank.
fn first_content_row(harness: &PtyHarness) -> u16 {
    let screen = harness.screen_contents();
    screen
        .lines()
        .position(|line| !line.trim().is_empty())
        .unwrap_or_else(|| panic!("screen is entirely blank\nscreen:\n{screen}")) as u16
}

fn leading_spaces(row: &str) -> usize {
    row.len() - row.trim_start().len()
}

/// The committed echo row (outside the bordered composer, which carries `│`).
fn echo_row(screen: &str, text: &str) -> String {
    screen
        .lines()
        .find(|line| line.contains(text) && !line.contains('│'))
        .map(str::to_owned)
        .unwrap_or_else(|| panic!("no committed echo row containing {text:?}\nscreen:\n{screen}"))
}

/// The frame spends no rows on margins at either width: status bar on row 0,
/// echo text at column 3 (outer margin + accent column + the rail's gutter),
/// content reaching the right pad's left neighbour. Widening the same session
/// keeps the flush frame while the phone-only gates lift (the echo draws its
/// arrow again).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn tight_frame_and_phone_only_gates() {
    let content = ContentController::start().await.expect("start content");
    content.set_response(phone_response());

    let binary = pager_binary().expect("resolve pager binary");
    let mut harness =
        PtyHarness::spawn_with_content(&binary, PHONE_ROWS, PHONE_COLS, &content, &[])
            .expect("spawn pager at phone size");

    harness
        .wait_for_text(WELCOME_SCREEN_SENTINEL, WELCOME_TIMEOUT)
        .expect("welcome text");

    const PROBE: &str = "phone-density probe";
    harness
        .inject_keys(format!("{PROBE}\r").as_bytes())
        .expect("submit prompt");
    harness
        .wait_for_text("P40", Duration::from_secs(30))
        .expect("mock response rendered");
    harness.update(Duration::from_millis(900));

    let screen = harness.screen_contents();
    eprintln!("--- {PHONE_COLS}x{PHONE_ROWS} frame ---\n{screen}");

    assert!(
        !screen.contains("panicked"),
        "pager rendered 'panicked'\nscreen:\n{screen}"
    );
    assert_eq!(
        first_content_row(&harness),
        0,
        "the phone frame keeps no blank margin row above the status bar\nscreen:\n{screen}"
    );

    let echo = echo_row(&screen, PROBE);
    assert_eq!(
        leading_spaces(&echo),
        TEXT_BAND_LEFT_INSET,
        "the echo band's text starts past the outer margin, the accent column, and the rail's gutter\nrow: {echo:?}"
    );
    assert!(
        !echo.contains('\u{276f}'),
        "the phone echo folds its decorative arrow away\nrow: {echo:?}"
    );

    let token_rows: Vec<String> = screen
        .lines()
        .filter(|line| line.trim_start().starts_with("xxx"))
        .map(str::to_owned)
        .collect();
    assert!(
        !token_rows.is_empty(),
        "no wrapped token row in the frame\nscreen:\n{screen}"
    );
    for row in &token_rows {
        assert_eq!(
            leading_spaces(row),
            TEXT_BAND_LEFT_INSET,
            "wrapped content keeps the same left edge as the echo\nrow: {row:?}"
        );
    }
    // The long token breaks at the text band's width: the pane's outer margins,
    // the block's left gutter (accent column + pad) and right pad, and the
    // scrollbar column all sit outside it.
    let rightmost = token_rows
        .iter()
        .map(|row| row.rfind('x').unwrap_or(0))
        .max()
        .unwrap();
    assert_eq!(
        rightmost,
        (PHONE_COLS - 1 - TEXT_BAND_RIGHT_INSET - TIMESTAMP_GUTTER) as usize,
        "wrapped content runs to the text band's right edge\nrows: {:?}",
        token_rows
            .iter()
            .map(|row| (row.len(), row.rfind('x')))
            .collect::<Vec<_>>(),
    );

    // Widen to a desktop pane: the frame stays flush (the pads are tight at every
    // width) while the phone-only render gates lift — the echo draws its `❯`
    // again instead of folding the arrow away.
    harness.resize(40, 120).expect("resize to desktop");
    harness.update(Duration::from_millis(900));
    let wide = harness.screen_contents();
    eprintln!("--- 120x40 frame ---\n{wide}");
    assert_eq!(
        first_content_row(&harness),
        0,
        "the desktop frame is flush too, got a blank top row\nscreen:\n{wide}"
    );
    let echo = echo_row(&wide, PROBE);
    assert_eq!(
        leading_spaces(&echo),
        TEXT_BAND_LEFT_INSET,
        "the desktop echo keeps the same text-band inset (the pads are width-independent)\nrow: {echo:?}"
    );
    assert!(
        echo.contains('\u{276f}'),
        "the desktop echo keeps its decorative arrow (a phone-only gate)\nrow: {echo:?}"
    );

    harness.quit().expect("clean quit");
}
