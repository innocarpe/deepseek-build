//! Phone-width bottom stack (measured iPhone Orca pane: 55 columns).
//!
//! The model/mode label leaves the prompt box's bottom divider and takes the row below
//! the box; the shortcut-hint row is not rendered in any state there, so the row it
//! used to occupy goes to the label. The DeepSeek balance row stays.
//!
//! `views::prompt_widget`'s tests pin the widget in isolation; these frames pin the
//! whole-view stack (the hint row is a layout row owned by `render`).

use super::{AgentView, AppRenderParams, BannerSlotParams, test_fixtures};
use crate::actions::ActionRegistry;
use crate::app::agent::AgentState;
use crate::scrollback::render::ScratchBuffer;
use agent_client_protocol as acp;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::sync::Arc;
use xai_grok_shell::sampling::types::ReasoningEffort;

/// The measured iPhone Orca pane.
const PHONE_COLS: u16 = 55;
const PHONE_ROWS: u16 = 41;
/// The narrowest desktop pane this repo pins elsewhere.
const DESKTOP_COLS: u16 = 120;
const DESKTOP_ROWS: u16 = 40;
const MODEL_LABEL: &str = "DeepSeek V4.1 Flash";
const MODEL_ID: &str = "deepseek-v4.1-flash";

/// An idle agent whose bottom stack matches the iPhone report: DeepSeek model label
/// with `(max)` effort, always-approve mode, and a known balance + cache-hit rate.
fn phone_agent() -> AgentView {
    let mut agent = test_fixtures::make_agent();
    let id = acp::ModelId::new(Arc::from(MODEL_ID));
    agent.session.models.available.insert(
        id.clone(),
        acp::ModelInfo::new(id.clone(), MODEL_LABEL.to_string()),
    );
    agent.session.models.current = Some(id);
    agent.session.models.reasoning_effort = Some(ReasoningEffort::Max);
    agent.session.set_yolo_mode_for_test(true);
    agent.deepseek_status = Some(
        serde_json::from_value(serde_json::json!({
            "isDeepseek": true,
            "balance": {"currency": "USD", "totalBalance": "15.87", "isAvailable": true},
            "usage": {"inputTokens": 4096, "cachedReadTokens": 3604},
        }))
        .expect("deepseek status fixture"),
    );
    agent.deepseek_status_session_id = agent.session.session_id.clone();
    agent
}

fn draw(agent: &mut AgentView, cols: u16, rows: u16) -> Buffer {
    agent.last_terminal_size = (cols, rows);
    let area = Rect::new(0, 0, cols, rows);
    let mut buf = Buffer::empty(area);
    let mut scratch = ScratchBuffer::new();
    let registry = ActionRegistry::defaults();
    agent.draw(
        area,
        &mut buf,
        &registry,
        &mut scratch,
        None,
        false,
        BannerSlotParams::none(),
        false,
        &mut Vec::new(),
        AppRenderParams::default(),
    );
    buf
}

fn row_text(buf: &Buffer, y: u16) -> String {
    (0..buf.area.width)
        .filter_map(|x| buf.cell((x, y)).map(|c| c.symbol().to_string()))
        .collect()
}

fn frame_text(buf: &Buffer) -> String {
    (0..buf.area.height)
        .map(|y| row_text(buf, y))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The bottom `n` rows with their row index, for the PR captures.
fn bottom_rows(buf: &Buffer, n: u16) -> String {
    let first = buf.area.height.saturating_sub(n);
    (first..buf.area.height)
        .map(|y| format!("{y:>3} │{}", row_text(buf, y)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The prompt box's bottom divider row: the last row whose first chrome column holds `╰`.
///
/// `LayoutConfig::default()` pads the outer viewport by two columns, so the box's left
/// edge sits at column 2.
fn border_row(buf: &Buffer) -> u16 {
    (0..buf.area.height)
        .rev()
        .find(|&y| {
            row_text(buf, y)
                .chars()
                .nth(2)
                .is_some_and(|c| c == '\u{2570}')
        })
        .unwrap_or_else(|| panic!("prompt box divider not found in\n{}", frame_text(buf)))
}

/// Every hint the bar would paint right now, as `Key:label`.
fn hint_texts(agent: &AgentView, registry: &ActionRegistry) -> Vec<String> {
    agent
        .current_shortcut_hints(registry)
        .iter()
        .map(|hint| {
            let key = hint
                .custom_display
                .map(str::to_owned)
                .or_else(|| hint.keys.first().map(|k| k.display()))
                .unwrap_or_default();
            format!("{key}:{}", hint.label)
        })
        .collect()
}

/// The bar would paint these hints at the phone width; none of them may reach the frame.
fn assert_no_hint_row(agent: &AgentView, registry: &ActionRegistry, buf: &Buffer) {
    let hints = hint_texts(agent, registry);
    assert!(
        !hints.is_empty(),
        "the fixture must have hints for this check to mean anything"
    );
    let frame = frame_text(buf);
    for hint in &hints {
        assert!(
            !frame.contains(hint.as_str()),
            "hint {hint:?} must not render on a phone-width pane:\n{frame}"
        );
    }
}

#[test]
fn phone_pane_puts_the_label_below_the_box_and_drops_the_hint_row() {
    let mut agent = phone_agent();
    let registry = ActionRegistry::defaults();
    let buf = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
    eprintln!("phone 55x41 — bottom band:\n{}", bottom_rows(&buf, 6));

    let border_y = border_row(&buf);
    let border = row_text(&buf, border_y);
    let label_row = row_text(&buf, border_y + 1);
    let balance_row = row_text(&buf, PHONE_ROWS - 2);

    // The divider is a plain rule: no glyph of the label sits between ╰ and ╯.
    assert!(
        !border.contains(MODEL_LABEL) && !border.contains("always-approve"),
        "the divider row carries no label text: {border:?}"
    );
    assert!(
        border.trim_end().ends_with('\u{256f}'),
        "the divider row still closes the box: {border:?}"
    );

    // The label sits on the row directly below the box; that row is the slot the
    // shortcut hints used to take.
    assert!(
        label_row.contains(&format!("{MODEL_LABEL} (max)")),
        "the model label is on the row below the box: {label_row:?}"
    );
    assert!(
        label_row.contains("always-approve"),
        "the mode flag moved with it: {label_row:?}"
    );
    // The label keeps the divider row's one-cell inset, so it never paints the
    // columns the box's `╰` / `╯` corners occupy.
    let label_chars: Vec<char> = label_row.chars().collect();
    assert!(
        label_chars[..3].iter().all(|c| *c == ' '),
        "the label stays clear of the box's left edge: {label_row:?}"
    );
    assert!(
        label_chars[(PHONE_COLS - 3) as usize..]
            .iter()
            .all(|c| *c == ' '),
        "and of its right edge: {label_row:?}"
    );

    // The DeepSeek status row is exactly where it was before the label moved —
    // the bottom stack kept its height while the hint row went away.
    assert!(
        balance_row.contains("$15.87"),
        "the balance chip stays on the bottom row: {balance_row:?}"
    );
    assert!(
        balance_row.contains("cache"),
        "the cache-hit chip stays too: {balance_row:?}"
    );
    for y in (PHONE_ROWS - 1)..buf.area.height {
        assert!(
            row_text(&buf, y).trim().is_empty(),
            "nothing renders below the balance row (row {y}): {:?}",
            row_text(&buf, y)
        );
    }

    assert_no_hint_row(&agent, &registry, &buf);
}

#[test]
fn phone_pane_drops_the_hint_row_mid_turn_too() {
    let mut agent = phone_agent();
    agent.session.state = AgentState::TurnRunning;
    let registry = ActionRegistry::defaults();
    let buf = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
    eprintln!(
        "phone 55x41 mid-turn — bottom band:\n{}",
        bottom_rows(&buf, 7)
    );

    let border_y = border_row(&buf);
    let label_row = row_text(&buf, border_y + 1);
    assert!(
        label_row.contains(&format!("{MODEL_LABEL} (max)")),
        "the model label is on the row below the box mid-turn: {label_row:?}"
    );
    assert_no_hint_row(&agent, &registry, &buf);
}

#[test]
fn desktop_pane_keeps_the_label_on_the_divider_and_the_hint_row() {
    let mut agent = phone_agent();
    let registry = ActionRegistry::defaults();
    let buf = draw(&mut agent, DESKTOP_COLS, DESKTOP_ROWS);
    eprintln!("desktop 120x40 — bottom band:\n{}", bottom_rows(&buf, 6));

    let border_y = border_row(&buf);
    let border = row_text(&buf, border_y);
    assert!(
        border.contains(&format!("{MODEL_LABEL} (max)")) && border.contains("always-approve"),
        "the desktop box keeps the label on its divider row: {border:?}"
    );

    let balance_y = (0..buf.area.height)
        .rev()
        .find(|&y| row_text(&buf, y).contains("$15.87"))
        .expect("the balance chip row");
    assert!(
        balance_y > border_y + 1,
        "the desktop stack keeps a row between the box and the chips (hints): {border_y} -> {balance_y}"
    );
    let between: Vec<String> = ((border_y + 1)..balance_y)
        .map(|y| row_text(&buf, y))
        .collect();
    assert!(
        between.iter().any(|row| !row.trim().is_empty()),
        "the desktop hint row still paints: {between:?}"
    );
}
