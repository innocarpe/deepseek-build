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

/// The composer band's bottom pad: the last row made only of `▂`.
fn band_bottom_row(screen: &str) -> usize {
    let lines: Vec<&str> = screen.lines().collect();
    lines
        .iter()
        .rposition(|line| {
            let line = line.trim_end();
            !line.is_empty() && line.chars().all(|c| c == '\u{2582}')
        })
        .unwrap_or_else(|| panic!("no composer band in the frame\nscreen:\n{screen}"))
}

/// Phone frame: the composer is a band with no box rule, and exactly two rows
/// (the footer) follow it — the frame's last two, one of which may be blank on
/// a session with no DeepSeek status and no mode chip. Desktop frames keep the
/// box with the model on its rule.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn phone_frame_ends_on_a_two_row_footer_under_the_composer_band() {
    let content = ContentController::start().await.expect("start content");
    content.set_response(format!("{MOCK_RESPONSE_SENTINEL} phone footer."));

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
    eprintln!("--- {PHONE_COLS}x{PHONE_ROWS} phone footer ---\n{screen}");
    assert!(
        !screen.contains('\u{2570}') && !screen.contains('\u{256d}'),
        "the phone composer draws no box\nscreen:\n{screen}"
    );
    let lines: Vec<&str> = screen.lines().collect();
    let band_at = band_bottom_row(&screen);
    // The frame reserves both footer rows; a blank trailing row never reaches
    // `lines()` because `screen_contents()` joins rows with `\n`. The composer
    // band's own index still pins the pair: two rows stay under it.
    assert_eq!(
        PHONE_ROWS as usize - 1 - band_at,
        2,
        "exactly the two footer rows follow the phone composer\nscreen:\n{screen}"
    );
    let after: Vec<&str> = lines.iter().skip(band_at + 1).copied().collect();
    for row in &after {
        assert!(
            !row.contains("Enter:send") && !row.contains("Shift+Tab"),
            "the phone footer is not the hint row\nrow: {row:?}"
        );
    }

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
