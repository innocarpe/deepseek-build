// Per-test-case module for the `pty_e2e` integration test crate.
#[allow(unused_imports)]
use super::common::*;

/// The measured iPhone Orca pane, and a desktop pane above the 60-column gate.
const PHONE_ROWS: u16 = 41;
const PHONE_COLS: u16 = 55;
const DESKTOP_ROWS: u16 = 40;
const DESKTOP_COLS: u16 = 120;

const PROBE: &str = "ECHOPROBE";

/// One logical line, long enough to pass both the two-row phone budget and the
/// three-row desktop budget once it wraps.
fn long_prompt() -> String {
    format!("{PROBE} ").repeat(40)
}

fn tall_response() -> String {
    let mut body = String::from("MOCKRESPONSE\n");
    for i in 0..40 {
        body.push_str(&format!("row {i:02} filler line for the frame\n"));
    }
    body
}

/// Committed echo rows. The composer draws `│`, so a draft still in the box is not the echo.
fn echo_rows(screen: &str) -> Vec<String> {
    screen
        .lines()
        .filter(|line| line.contains(PROBE) && !line.contains('│'))
        .map(str::to_owned)
        .collect()
}

fn assert_collapsed_echo(screen: &str, rows: usize, label: &str) {
    let echo = echo_rows(screen);
    assert_eq!(
        echo.len(),
        rows,
        "{label}: collapsed echo should be {rows} rows\nrows: {echo:?}\nscreen:\n{screen}"
    );
    assert!(
        echo[..echo.len() - 1]
            .iter()
            .all(|row| !row.contains('\u{2026}')),
        "{label}: the ellipsis belongs on the last row\nrows: {echo:?}"
    );
    assert!(
        echo.last().is_some_and(|row| row.contains('\u{2026}')),
        "{label}: the last row keeps the ellipsis\nrows: {echo:?}"
    );
}

/// Phone width folds a long submitted prompt to two rows plus an ellipsis.
/// Widening the same session to 120 columns restores the three-row budget.
/// One binary serves both frames.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn phone_echo_folds_to_two_lines_and_desktop_keeps_three() {
    let content = ContentController::start().await.expect("start content");
    content.set_response(tall_response());

    let binary = pager_binary().expect("resolve pager binary");
    let mut harness =
        PtyHarness::spawn_with_content(&binary, PHONE_ROWS, PHONE_COLS, &content, &[])
            .expect("spawn pager at phone size");

    harness
        .wait_for_text(WELCOME_SCREEN_SENTINEL, WELCOME_TIMEOUT)
        .expect("welcome text");

    harness
        .inject_keys(format!("{}\r", long_prompt()).as_bytes())
        .expect("submit long prompt");
    harness
        .wait_for_text("MOCKRESPONSE", Duration::from_secs(30))
        .expect("mock response rendered");
    harness.update(Duration::from_millis(900));

    let phone = harness.screen_contents();
    eprintln!("--- {PHONE_COLS}x{PHONE_ROWS} echo ---\n{phone}");
    assert!(
        !phone.contains("panicked"),
        "pager panicked\nscreen:\n{phone}"
    );
    assert!(
        !phone.contains("Approve in your browser"),
        "seeded session stopped on the sign-in screen\nscreen:\n{phone}"
    );
    assert_collapsed_echo(&phone, 2, "55x41");

    harness
        .resize(DESKTOP_ROWS, DESKTOP_COLS)
        .expect("resize to desktop");
    harness.update(Duration::from_millis(900));
    let desktop = harness.screen_contents();
    eprintln!("--- {DESKTOP_COLS}x{DESKTOP_ROWS} echo ---\n{desktop}");
    assert_collapsed_echo(&desktop, 3, "120x40");

    harness.quit().expect("clean quit");
}
