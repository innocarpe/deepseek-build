// Per-test-case module for the `pty_e2e` integration test crate.
#[allow(unused_imports)]
use super::common::*;

/// The measured iPhone Orca pane.
const PHONE_ROWS: u16 = 41;
const PHONE_COLS: u16 = 55;
/// The composer band keeps the box's right inset (`LayoutConfig::BOX_PAD`):
/// the text row ends this many columns before the frame's right edge.
const RIGHT_PAD: u16 = 2;
const BACKSPACE: u8 = 0x7f;

/// The composer band's text rows: between the last row made only of `▆` (its
/// top pad) and the last row made only of `▂` (its bottom pad).
fn band_text_rows(screen: &str) -> std::ops::Range<usize> {
    let lines: Vec<&str> = screen.lines().collect();
    let only = |line: &str, glyph: char| {
        let line = line.trim_end();
        !line.is_empty() && line.chars().all(|c| c == glyph)
    };
    let bottom = lines
        .iter()
        .rposition(|line| only(line, '\u{2582}'))
        .unwrap_or_else(|| panic!("no composer band bottom\nscreen:\n{screen}"));
    let top = lines[..bottom]
        .iter()
        .rposition(|line| only(line, '\u{2586}'))
        .unwrap_or_else(|| panic!("no composer band top\nscreen:\n{screen}"));
    top + 1..bottom
}

/// A draft of exactly `cols` columns: words of four `unit` glyphs with a space
/// between them, and a one-column `.` closing an odd count of two-column
/// glyphs.
fn full_row_draft(unit: char, unit_cols: usize, cols: usize) -> String {
    let (mut draft, mut used, mut in_word) = (String::new(), 0, 0);
    while used + unit_cols <= cols {
        if in_word == 4 && used + 1 + unit_cols <= cols {
            draft.push(' ');
            used += 1;
            in_word = 0;
        }
        draft.push(unit);
        used += unit_cols;
        in_word += 1;
    }
    if used < cols {
        draft.push('.');
    }
    draft
}

fn type_text(harness: &mut PtyHarness, text: &str) {
    harness.inject_keys(text.as_bytes()).expect("type draft");
    harness.update(Duration::from_millis(500));
}

/// The iPhone report on the real pager: typing to the end of the phone
/// composer's text row opened an empty second row and dropped the caret onto
/// it while the row's right pad was still free. A full row keeps one text row
/// and the caret right after its last glyph; the next character opens the
/// second row and takes the caret down. One-column English and two-column
/// Hangul alike.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn phone_composer_opens_a_row_only_for_the_character_after_a_full_row() {
    let content = ContentController::start().await.expect("start content");
    content.set_response(format!("{MOCK_RESPONSE_SENTINEL} full row."));

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

    // One glyph puts the caret one column past the text row's first column.
    type_text(&mut harness, "x");
    let (_, after_one) = harness.cursor_position();
    let text_start = after_one - 1;
    let text_cols = PHONE_COLS - RIGHT_PAD - text_start;
    harness.inject_keys(&[BACKSPACE]).expect("clear probe");

    for (label, unit, unit_cols, next) in [("English", 'x', 1, "s"), ("Hangul", '가', 2, "요")] {
        let draft = full_row_draft(unit, unit_cols, usize::from(text_cols));
        type_text(&mut harness, &draft);
        let screen = harness.screen_contents();
        eprintln!("--- {label}: full row of {text_cols} columns ---\n{screen}");
        let rows = band_text_rows(&screen);
        assert_eq!(
            rows.len(),
            1,
            "{label}: a full row is one text row\nscreen:\n{screen}"
        );
        assert_eq!(
            harness.cursor_position(),
            (rows.start as u16, text_start + text_cols),
            "{label}: the caret stays after the last glyph of the full row\nscreen:\n{screen}"
        );

        type_text(&mut harness, next);
        let screen = harness.screen_contents();
        eprintln!("--- {label}: one more character ---\n{screen}");
        let rows = band_text_rows(&screen);
        assert_eq!(
            rows.len(),
            2,
            "{label}: the next character opens the second row\nscreen:\n{screen}"
        );
        let (caret_row, _) = harness.cursor_position();
        assert_eq!(
            caret_row,
            rows.start as u16 + 1,
            "{label}: the caret follows the character down\nscreen:\n{screen}"
        );

        let typed = draft.chars().count() + next.chars().count();
        harness
            .inject_keys(&vec![BACKSPACE; typed])
            .expect("clear draft");
        harness.update(Duration::from_millis(500));
    }

    harness.quit().expect("clean quit");
}
