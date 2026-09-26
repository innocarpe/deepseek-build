// Per-test-case module for the `pty_e2e` integration test crate.
#[allow(unused_imports)]
use super::common::*;

/// The measured iPhone Orca pane.
const PHONE_ROWS: u16 = 41;
const PHONE_COLS: u16 = 55;

fn divider_row(screen: &str) -> (usize, String) {
    let lines: Vec<&str> = screen.lines().collect();
    lines
        .iter()
        .enumerate()
        .rev()
        .find(|(_, line)| line.contains('\u{2570}') && line.contains('\u{256f}'))
        .map(|(i, line)| (i, (*line).to_string()))
        .unwrap_or_else(|| panic!("no prompt divider in the frame\nscreen:\n{screen}"))
}

fn is_plain_rule(line: &str) -> bool {
    let chars: Vec<char> = line.trim_end().chars().collect();
    chars.len() >= 2
        && chars.first() == Some(&'\u{2570}')
        && chars.last() == Some(&'\u{256f}')
        && chars[1..chars.len() - 1].iter().all(|c| *c == '\u{2500}')
}

/// Phone frame: the box rule is plain and exactly one row follows it. Desktop
/// frames put the model back on that rule.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn bottom_band_is_one_row_on_the_phone_frame() {
    let content = ContentController::start().await.expect("start content");
    content.set_response(format!("{MOCK_RESPONSE_SENTINEL} bottom band."));

    let binary = pager_binary().expect("resolve pager binary");
    let mut harness =
        PtyHarness::spawn_with_content(&binary, PHONE_ROWS, PHONE_COLS, &content, &[])
            .expect("spawn pager at phone size");

    harness
        .wait_for_text(WELCOME_SCREEN_SENTINEL, WELCOME_TIMEOUT)
        .expect("welcome text");
    harness
        .inject_keys(format!("{PROMPT}\r").as_bytes())
        .expect("submit prompt");
    harness
        .wait_for_text(MOCK_RESPONSE_SENTINEL, Duration::from_secs(30))
        .expect("mock response rendered");
    harness.update(Duration::from_millis(900));

    let screen = harness.screen_contents();
    eprintln!("--- {PHONE_COLS}x{PHONE_ROWS} bottom band ---\n{screen}");
    let lines: Vec<&str> = screen.lines().collect();
    let (div_at, divider) = divider_row(&screen);
    assert!(
        is_plain_rule(divider.trim_start()),
        "the phone divider is a plain rule\nrow: {divider:?}"
    );
    let after: Vec<&str> = lines
        .iter()
        .skip(div_at + 1)
        .copied()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert_eq!(
        after.len(),
        1,
        "exactly one row follows the phone box\nrows: {after:?}\nscreen:\n{screen}"
    );
    let band = after.first().copied().unwrap_or("");
    assert!(
        !band.contains('\u{2570}'),
        "the band is not a second box\nrow: {band:?}"
    );
    assert!(
        !band.contains("Enter:send") && !band.contains("Shift+Tab"),
        "the phone band is not the hint row\nrow: {band:?}"
    );

    for (rows, cols) in [(40u16, 80u16), (50, 180)] {
        harness.resize(rows, cols).expect("resize to desktop");
        harness.update(Duration::from_millis(900));
        let wide = harness.screen_contents();
        eprintln!("--- {cols}x{rows} bottom band ---\n{wide}");
        let (_, divider) = divider_row(&wide);
        assert!(
            !is_plain_rule(divider.trim_start()),
            "the desktop divider keeps the model on the rule\nrow: {divider:?}\nscreen:\n{wide}"
        );
    }

    harness.quit().expect("clean quit");
}
