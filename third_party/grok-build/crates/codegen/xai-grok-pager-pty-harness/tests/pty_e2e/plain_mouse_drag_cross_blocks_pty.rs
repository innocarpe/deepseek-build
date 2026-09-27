// Per-test-case module for the scroll-selection PTY family.
#[allow(unused_imports)]
use super::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "PTY e2e; run with --ignored after building the pager binary"]
async fn plain_mouse_drag_cross_blocks_copies_exact_selected_text() {
    const PROMPT_TEXT: &str = "PLAIN_DRAG_PROMPT";
    const REPLY_TEXT: &str = "PLAIN_DRAG_REPLY";
    let content = ContentController::start().await.expect("start content");
    // Bare `y` is a scrollback action only in vim mode. Keep the held
    // highlight alive while checking the second, explicit copy delivery.
    seed_ui_config(
        &content,
        "vim_mode = true\nsimple_mode = false\nkeep_text_selection = \"hold\"",
    );
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
    let expected = format!(
        "DRAG_PROMPT{}PLAIN_DRAG",
        "\n".repeat((reply_row - prompt_row) as usize)
    );

    harness
        .inject_keys(sgr_mouse(0, prompt_row, prompt_col + 6, 'M').as_bytes())
        .expect("mouse down");
    harness.update(Duration::from_millis(300));
    harness
        .inject_keys(sgr_mouse(32, reply_row, reply_col + 9, 'M').as_bytes())
        .expect("mouse drag");
    harness.update(Duration::from_millis(300));
    harness
        .inject_keys(sgr_mouse(0, reply_row, reply_col + 9, 'm').as_bytes())
        .expect("mouse up");

    let payloads = wait_for_osc52_payloads(&mut harness, Duration::from_secs(10));
    assert_eq!(
        payloads.last().map(String::as_str),
        Some(expected.as_str()),
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
        Some(expected.as_str()),
        "y must copy the held selection again, not the whole selected block"
    );
    harness.quit().expect("clean quit");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "PTY e2e; run with --ignored after building the pager binary"]
async fn plain_mouse_drag_copy_button_reuses_exact_selection_in_default_key_mode() {
    const PROMPT_TEXT: &str = "BUTTON_DRAG_PROMPT";
    const REPLY_TEXT: &str = "BUTTON_DRAG_REPLY";
    let content = ContentController::start().await.expect("start content");
    // No selection settings are injected: a fresh default install must keep
    // the dragged span and show ⧉ with vim_mode=false.
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
        "expected two visible blocks:\n{screen}"
    );
    let expected = format!(
        "DRAG_PROMPT{}BUTTON_DRAG",
        "\n".repeat((reply_row - prompt_row) as usize)
    );
    harness
        .inject_keys(sgr_mouse(0, prompt_row, prompt_col + 7, 'M').as_bytes())
        .expect("mouse down");
    harness.update(Duration::from_millis(300));
    harness
        .inject_keys(sgr_mouse(32, reply_row, reply_col + 10, 'M').as_bytes())
        .expect("mouse drag");
    harness.update(Duration::from_millis(300));
    harness
        .inject_keys(sgr_mouse(0, reply_row, reply_col + 10, 'm').as_bytes())
        .expect("mouse up");
    let payloads = wait_for_osc52_payloads(&mut harness, Duration::from_secs(10));
    assert_eq!(payloads.last().map(String::as_str), Some(expected.as_str()));
    let initial_count = payloads.len();

    // Wait beyond the former 150ms flash timeout before the explicit click.
    harness.update(Duration::from_millis(500));
    let screen = harness.screen_contents();
    let (button_row, button_col) = locate_screen_text(&screen, "⧉")
        .unwrap_or_else(|| panic!("selected scrollback copy button missing:\n{screen}"));
    assert!(
        button_row > 0,
        "copy button overlaps the status row:\n{screen}"
    );
    harness
        .inject_keys(sgr_mouse(0, button_row, button_col, 'M').as_bytes())
        .expect("press copy button");
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
        Some(expected.as_str()),
        "copy button must preserve the exact dragged payload; screen:\n{}",
        harness.screen_contents()
    );
    harness
        .inject_keys(sgr_mouse(0, button_row, button_col, 'm').as_bytes())
        .expect("release copy button");
    harness.update(Duration::from_millis(200));
    assert!(
        harness.screen_contents().contains(REPLY_TEXT),
        "button release must not switch away from the transcript"
    );
    harness.quit().expect("clean quit");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "PTY e2e; run with --ignored after building the pager binary"]
async fn plain_mouse_drag_long_answer_has_visible_held_copy_chip() {
    const TARGET: &str = "LONG_LINE_075 MIDDLE_TARGET";
    let content = ContentController::start().await.expect("start content");
    let response = (0..80)
        .map(|line| {
            if line == 75 {
                TARGET.to_string()
            } else {
                format!("LONG_LINE_{line:03} filler")
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    content.set_response(response);
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
        .inject_keys(b"LONG_PROMPT\r")
        .expect("submit prompt");
    harness
        .wait_for_text(TARGET, Duration::from_secs(45))
        .expect("long response target");
    harness
        .wait_for_text("Worked for", Duration::from_secs(20))
        .expect("completed turn");
    harness.inject_keys(b"\t").expect("focus scrollback");
    harness.update(Duration::from_millis(500));
    let screen = harness.screen_contents();
    assert!(
        !screen.contains("LONG_LINE_000"),
        "answer top must be offscreen for this regression:\n{screen}"
    );
    let (target_row, target_col) = locate_screen_text(&screen, TARGET)
        .unwrap_or_else(|| panic!("middle line missing from screen:\n{screen}"));
    let start = TARGET.find("MIDDLE").unwrap() as u16;
    harness
        .inject_keys(sgr_mouse(0, target_row, target_col + start, 'M').as_bytes())
        .expect("mouse down");
    harness.update(Duration::from_millis(300));
    harness
        .inject_keys(sgr_mouse(32, target_row, target_col + start + 5, 'M').as_bytes())
        .expect("mouse drag");
    harness.update(Duration::from_millis(300));
    harness
        .inject_keys(sgr_mouse(0, target_row, target_col + start + 5, 'm').as_bytes())
        .expect("mouse up");
    let payloads = wait_for_osc52_payloads(&mut harness, Duration::from_secs(10));
    assert_eq!(payloads.last().map(String::as_str), Some("MIDDLE"));
    let initial_count = payloads.len();

    harness.update(Duration::from_millis(500));
    let screen = harness.screen_contents();
    let (chip_row, chip_col) = locate_screen_text(&screen, "⧉")
        .unwrap_or_else(|| panic!("held copy chip missing from long answer:\n{screen}"));
    assert_eq!(
        chip_row, target_row,
        "chip must follow the visible highlight"
    );
    harness
        .inject_keys(sgr_mouse(0, chip_row, chip_col, 'M').as_bytes())
        .expect("press held copy chip");
    let deadline = Instant::now() + Duration::from_secs(10);
    let copied = loop {
        harness.update(Duration::from_millis(200));
        let payloads = decode_osc52_payloads(harness.raw_output());
        if payloads.len() > initial_count || Instant::now() >= deadline {
            break payloads;
        }
    };
    assert_eq!(
        copied.get(initial_count).map(String::as_str),
        Some("MIDDLE")
    );
    harness
        .inject_keys(sgr_mouse(0, chip_row, chip_col, 'm').as_bytes())
        .expect("release held copy chip");
    harness.quit().expect("clean quit");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "PTY e2e; run with --ignored after building the pager binary"]
async fn plain_mouse_drag_next_turn_copies_new_block_with_button_and_y() {
    let content = ContentController::start().await.expect("start content");
    seed_ui_config(&content, "vim_mode = true\nsimple_mode = false");
    content.set_response("FIRST_HELD_TEXT".to_string());
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
        .inject_keys(b"FIRST_PROMPT\r")
        .expect("first prompt");
    harness
        .wait_for_text("FIRST_HELD_TEXT", Duration::from_secs(45))
        .expect("first reply");
    harness
        .wait_for_text("Worked for", Duration::from_secs(20))
        .expect("first turn complete");
    harness.inject_keys(b"\t").expect("focus scrollback");
    harness.update(Duration::from_millis(300));
    let screen = harness.screen_contents();
    let (row, col) = locate_screen_text(&screen, "FIRST_HELD_TEXT")
        .unwrap_or_else(|| panic!("first reply missing:\n{screen}"));
    harness
        .inject_keys(sgr_mouse(0, row, col + 6, 'M').as_bytes())
        .expect("mouse down");
    harness.update(Duration::from_millis(300));
    harness
        .inject_keys(sgr_mouse(32, row, col + 9, 'M').as_bytes())
        .expect("mouse drag");
    harness.update(Duration::from_millis(300));
    harness
        .inject_keys(sgr_mouse(0, row, col + 9, 'm').as_bytes())
        .expect("mouse up");
    let payloads = wait_for_osc52_payloads(&mut harness, Duration::from_secs(10));
    assert_eq!(payloads.last().map(String::as_str), Some("HELD"));
    let initial_count = payloads.len();

    content.set_response("SECOND_WHOLE_BLOCK".to_string());
    harness.inject_keys(b"\t").expect("focus prompt");
    harness
        .inject_keys(b"SECOND_PROMPT\r")
        .expect("send second prompt");
    harness
        .wait_for_text("SECOND_WHOLE_BLOCK", Duration::from_secs(45))
        .expect("second reply");
    harness
        .wait_for_text("Worked for", Duration::from_secs(20))
        .expect("second turn complete");
    harness.inject_keys(b"\t").expect("focus scrollback");
    harness.update(Duration::from_millis(500));
    let screen = harness.screen_contents();
    let (reply_row, reply_col) = locate_screen_text(&screen, "SECOND_WHOLE_BLOCK")
        .unwrap_or_else(|| panic!("new answer missing:\n{screen}"));
    harness
        .inject_keys(sgr_mouse(0, reply_row, reply_col + 3, 'M').as_bytes())
        .expect("select new answer down");
    harness.update(Duration::from_millis(200));
    harness
        .inject_keys(sgr_mouse(0, reply_row, reply_col + 3, 'm').as_bytes())
        .expect("select new answer up");
    harness.update(Duration::from_millis(300));
    let screen = harness.screen_contents();
    let (button_row, button_col) = locate_screen_text(&screen, "⧉")
        .unwrap_or_else(|| panic!("new block copy button missing:\n{screen}"));
    harness
        .inject_keys(sgr_mouse(0, button_row, button_col, 'M').as_bytes())
        .expect("press new block copy button");
    let deadline = Instant::now() + Duration::from_secs(10);
    let copied = loop {
        harness.update(Duration::from_millis(200));
        let payloads = decode_osc52_payloads(harness.raw_output());
        if payloads.len() > initial_count || Instant::now() >= deadline {
            break payloads;
        }
    };
    assert_eq!(
        copied.get(initial_count).map(String::as_str),
        Some("SECOND_WHOLE_BLOCK"),
        "new block button copied an old held fragment; screen:\n{}",
        harness.screen_contents()
    );
    harness
        .inject_keys(sgr_mouse(0, button_row, button_col, 'm').as_bytes())
        .expect("release block copy button");
    harness.update(Duration::from_millis(200));
    harness.inject_keys(b"y").expect("copy new block with y");
    let deadline = Instant::now() + Duration::from_secs(10);
    let copied = loop {
        harness.update(Duration::from_millis(200));
        let payloads = decode_osc52_payloads(harness.raw_output());
        if payloads.len() > initial_count + 1 || Instant::now() >= deadline {
            break payloads;
        }
    };
    assert_eq!(
        copied.get(initial_count + 1).map(String::as_str),
        Some("SECOND_WHOLE_BLOCK")
    );
    harness.quit().expect("clean quit");
}
