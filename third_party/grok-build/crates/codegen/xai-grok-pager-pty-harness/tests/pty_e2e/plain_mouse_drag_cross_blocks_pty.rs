// Per-test-case module for the scroll-selection PTY family.
#[allow(unused_imports)]
use super::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "PTY e2e; run with --ignored after building the pager binary"]
async fn plain_mouse_drag_cross_blocks_copies_exact_selected_text() {
    const PROMPT_TEXT: &str = "PLAIN_DRAG_PROMPT";
    const REPLY_TEXT: &str = "PLAIN_DRAG_REPLY";
    let content = ContentController::start().await.expect("start content");
    content.set_response(REPLY_TEXT.to_string());
    let binary = pager_binary().expect("resolve pager binary");
    let mut harness = PtyHarness::spawn_with_content_env_in_dir(
        &binary,
        DEFAULT_ROWS,
        DEFAULT_COLS,
        &content,
        &[],
        &[("SSH_CONNECTION", "scripted-test 1 127.0.0.1 2")],
        Some(content.home()),
    )
    .expect("spawn pager");
    harness
        .wait_for_text(WELCOME_SCREEN_SENTINEL, WELCOME_TIMEOUT)
        .expect("welcome");
    harness
        .inject_keys(format!("{PROMPT_TEXT}\r").as_bytes())
        .expect("submit prompt");
    harness
        .wait_for_text(REPLY_TEXT, Duration::from_secs(45))
        .expect("response");
    harness
        .wait_for_text("Worked for", Duration::from_secs(20))
        .expect("completed turn");
    harness.inject_keys(b"\t").expect("focus scrollback");
    harness.update(Duration::from_millis(500));

    let screen = harness.screen_contents();
    let (prompt_row, prompt_col) = locate_screen_text(&screen, PROMPT_TEXT)
        .unwrap_or_else(|| panic!("prompt missing from screen:\n{screen}"));
    let (reply_row, reply_col) = locate_screen_text(&screen, REPLY_TEXT)
        .unwrap_or_else(|| panic!("reply missing from screen:\n{screen}"));
    assert!(
        prompt_row < reply_row,
        "expected two distinct visible blocks:\n{screen}"
    );

    let mut drag = String::new();
    drag.push_str(&sgr_mouse(0, prompt_row, prompt_col + 6, 'M'));
    drag.push_str(&sgr_mouse(32, reply_row, reply_col + 9, 'M'));
    drag.push_str(&sgr_mouse(0, reply_row, reply_col + 9, 'm'));
    harness
        .inject_keys(drag.as_bytes())
        .expect("drag and release");

    let payloads = wait_for_osc52_payloads(&mut harness, Duration::from_secs(10));
    assert_eq!(
        payloads.last().map(String::as_str),
        Some("DRAG_PROMPT\n\nPLAIN_DRAG"),
        "clipboard must contain exactly the cross-block selection; screen:\n{}",
        harness.screen_contents()
    );
    let initial_count = payloads.len();

    // The explicit scrollback copy action must preserve the held selection,
    // rather than replacing the clipboard with the selected prompt block.
    harness.inject_keys(b"y").expect("copy held selection");
    let deadline = Instant::now() + Duration::from_secs(10);
    let explicit_payloads = loop {
        harness.update(Duration::from_millis(200));
        let payloads = decode_osc52_payloads(harness.raw_output());
        if payloads.len() > initial_count || Instant::now() >= deadline {
            break payloads;
        }
    };
    assert_eq!(
        explicit_payloads.get(initial_count).map(String::as_str),
        Some("DRAG_PROMPT\n\nPLAIN_DRAG"),
        "y must copy the held selection again, not the whole selected block"
    );
    harness.quit().expect("clean quit");
}
