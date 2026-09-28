//! Regression for the overlay strip's background (user report 2026-09-29): on a phone-sized
//! frame, the elapsed time and the `[↗]` / `[✗]` buttons painted a gray box behind every glyph
//! that landed on a cell ratatui had reset for a wide (CJK) grapheme of the label underneath.
//!
//! Mechanism: `Buffer::set_stringn` writes the cell after a width-2 grapheme as `Cell::EMPTY`
//! (`bg = Color::Reset`). The overlay's blanking span ([`clear_overlay_area`]) and its fg-only
//! time/button spans patch only the fields they set, so those slots kept `Color::Reset` and the
//! terminal drew its own default background there instead of the row's theme background.
//!
//! The frame background is pinned to a fixed color: a headless test process quantizes the theme
//! to `Color::Reset`, which would make a `Color::Reset` assertion pass without the mechanism.

use super::*;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use std::collections::{BTreeMap, HashMap};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Generic Korean description: the label runs under the overlay strip and its wide graphemes
/// leave continuation slots inside the strip.
const DESCRIPTION: &str = "테스트 작업 실행 상태 (백그라운드 작업 확인)";
/// Frame background the test paints before the pane draws (fixed, never `Color::Reset`).
const FRAME_BG: Color = Color::Rgb(14, 19, 40);
/// Selection background pinned on the pane, same reason as [`FRAME_BG`].
const SELECTION_BG: Color = Color::Rgb(36, 48, 84);

fn running_task() -> BgTaskState {
    BgTaskState {
        task_id: "bg-1".into(),
        tool_call_id: "call-1".into(),
        command: "sleep 5".into(),
        description: Some(DESCRIPTION.to_owned()),
        cwd: "/tmp".into(),
        output_file: "/tmp/out".into(),
        status: BgTaskStatus::Running,
        start_time: std::time::SystemTime::now(),
        end_time: None,
        exit_code: None,
        signal: None,
        stdout: String::new(),
        stdout_line_count: 0,
        truncated: false,
        pending_kill: false,
        kill_requested_at: None,
        scrollback_entry_id: None,
        is_monitor: false,
        restored_from_replay: false,
    }
}

/// A pane with one running task, synced so the overlay is visible and entries are built.
fn fixture() -> (TasksPane, BTreeMap<String, BgTaskState>) {
    let tasks = BTreeMap::from([("bg-1".to_owned(), running_task())]);
    let mut pane = TasksPane::new();
    pane.sync(&tasks, &HashMap::new(), &HashMap::new(), &[]);
    assert!(
        pane.is_visible(),
        "a fresh running task auto-opens the pane"
    );
    (pane, tasks)
}

/// A buffer whose frame background is [`FRAME_BG`], as the app fills it before the pane draws.
fn frame(area: Rect) -> Buffer {
    let mut buf = Buffer::empty(area);
    buf.set_style(area, Style::default().bg(FRAME_BG));
    buf
}

/// Absolute columns whose cell is the continuation slot of a width-2 grapheme in the painted
/// label (prefix plus description), i.e. exactly the cells `Buffer::set_stringn` resets.
fn continuation_columns(inner: Rect) -> Vec<u16> {
    let mut x = inner.x + 2; // item prefix ("  ")
    let mut slots = Vec::new();
    for ch in format!("Task {DESCRIPTION}").chars() {
        let width = ch.width().unwrap_or(0) as u16;
        if width > 1 && x + 1 < inner.right() {
            slots.push(x + 1);
        }
        x = x.saturating_add(width);
    }
    slots
}

/// Absolute columns of the overlay's cleared strip on the task row: kill (3) + view (3) +
/// elapsed text + one separator, with no stdout line badge on this task.
fn overlay_strip(pane: &TasksPane, tasks: &BTreeMap<String, BgTaskState>, inner: Rect) -> Vec<u16> {
    let view_x = pane
        .view_button_rects
        .iter()
        .find(|(id, _)| matches!(id, TaskEntryId::BgTask(t) if t == "bg-1"))
        .map(|(_, rect)| rect.x)
        .expect("view button rect for the running task");
    let elapsed = format!("{} ", crate::util::format_duration(tasks["bg-1"].elapsed()));
    let start = view_x - elapsed.width() as u16 - 1;
    (start..inner.right()).collect()
}

fn button_rect(rects: &[(TaskEntryId, Rect)]) -> Rect {
    rects
        .iter()
        .find(|(id, _)| matches!(id, TaskEntryId::BgTask(t) if t == "bg-1"))
        .map(|(_, rect)| *rect)
        .expect("button rect for the running task")
}

fn painted(buf: &Buffer, rect: Rect, y: u16) -> String {
    (rect.x..rect.x + rect.width)
        .map(|x| buf.cell((x, y)).expect("cell").symbol().to_string())
        .collect()
}

#[test]
fn overlay_strip_keeps_the_row_background_over_cjk_continuation_slots() {
    let cfg = LayoutConfig::default();
    let area = Rect::new(0, 0, 44, 3);
    let inner = TasksPane::content_area(area, &cfg);
    let (mut pane, tasks) = fixture();

    let mut buf = frame(area);
    pane.render(
        area,
        &mut buf,
        false,
        &cfg,
        &tasks,
        &HashMap::new(),
        &HashMap::new(),
    );

    let row_y = inner.y + 1; // row 0 is the `Tasks` group header
    let strip = overlay_strip(&pane, &tasks, inner);
    let covered: Vec<u16> = strip
        .iter()
        .copied()
        .filter(|x| continuation_columns(inner).contains(x))
        .collect();
    assert!(
        !covered.is_empty(),
        "fixture must place a wide-grapheme continuation slot under the strip; strip={strip:?}"
    );

    for &x in &strip {
        let bg = buf.cell((x, row_y)).expect("cell").bg;
        assert_ne!(
            bg,
            Color::Reset,
            "cell {x} kept the wide-grapheme reset background; continuation slots in the strip: {covered:?}"
        );
        assert_eq!(
            bg, FRAME_BG,
            "cell {x} must carry the row background (continuation slots in the strip: {covered:?})"
        );
    }
}

#[test]
fn overlay_buttons_paint_where_their_hitboxes_are() {
    let cfg = LayoutConfig::default();
    let area = Rect::new(0, 0, 44, 3);
    let inner = TasksPane::content_area(area, &cfg);
    let (mut pane, tasks) = fixture();

    let mut buf = frame(area);
    pane.render(
        area,
        &mut buf,
        false,
        &cfg,
        &tasks,
        &HashMap::new(),
        &HashMap::new(),
    );

    let row_y = inner.y + 1;
    let kill = button_rect(&pane.kill_button_rects);
    let view = button_rect(&pane.view_button_rects);
    assert_eq!(kill, Rect::new(inner.right() - 3, row_y, 3, 1));
    assert_eq!(view, Rect::new(inner.right() - 6, row_y, 3, 1));
    assert_eq!(painted(&buf, kill, row_y), crate::glyphs::ballot_x_button());
    assert_eq!(painted(&buf, view, row_y), crate::glyphs::enlarge_button());
}

#[test]
fn focused_selection_background_survives_the_overlay_strip() {
    let cfg = LayoutConfig::default();
    let area = Rect::new(0, 0, 44, 3);
    let inner = TasksPane::content_area(area, &cfg);
    let (mut pane, tasks) = fixture();

    // Pin the selection color instead of reading the quantized ambient theme.
    pane.list_style = ListPaneStyle {
        selection_bg: SELECTION_BG,
        show_corner_indicators: false,
        ..ListPaneStyle::default()
    };
    let task_id = pane
        .entries
        .iter()
        .find(|e| matches!(e, TaskEntry::BgTask { task_id, .. } if task_id == "bg-1"))
        .expect("task entry")
        .stable_id();
    pane.list_state.select_by_id(task_id);

    let mut buf = frame(area);
    pane.render(
        area,
        &mut buf,
        true,
        &cfg,
        &tasks,
        &HashMap::new(),
        &HashMap::new(),
    );

    let row_y = inner.y + 1;
    // The selection band reached the row (sanity: `ListPane` paints it, not the overlay).
    assert_eq!(
        buf.cell((inner.x + 4, row_y)).expect("cell").bg,
        SELECTION_BG,
        "focused selection must paint the row"
    );
    // The overlay strip keeps that selection background instead of the reset slot or the row base.
    for x in overlay_strip(&pane, &tasks, inner) {
        assert_eq!(
            buf.cell((x, row_y)).expect("cell").bg,
            SELECTION_BG,
            "cell {x} must keep the selection background"
        );
    }
}
