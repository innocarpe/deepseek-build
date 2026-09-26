//! The `Jump to bottom` chip: scrolled up, the scrollback's last row carries a clickable chip
//! that returns to the bottom and re-engages follow mode (Claude Code parity, tappable on a phone).
//!
//! Frames here draw the whole agent view (`phone_bottom_tests` style) and assert the painted row,
//! the hit rect the frame leaves behind, and the frames that must hide the chip. The label ladder
//! and the unchanged one-glyph `▲` are pinned next to their primitives in `render.rs`.

use super::test_fixtures::make_agent;
use super::{AgentView, AppRenderParams, BannerSlotParams};
use crate::actions::ActionRegistry;
use crate::scrollback::block::RenderBlock;
use crate::scrollback::render::ScratchBuffer;
use crate::scrollback::search::ScrollbackSearchState;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

/// The measured iPhone Orca pane.
const PHONE_COLS: u16 = 55;
const PHONE_ROWS: u16 = 41;
const DESKTOP_COLS: u16 = 80;
const DESKTOP_ROWS: u16 = 40;
const FULL_LABEL: &str = "Jump to bottom (click) ↓";
const SHORT_LABEL: &str = "Jump to bottom ↓";

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

/// Forty stacked messages: taller than every viewport these frames use.
fn seed_conversation(agent: &mut AgentView) {
    for i in 0..40 {
        agent
            .scrollback
            .push_block(RenderBlock::agent_message(format!("line {i}")));
    }
}

/// A settled frame with the conversation scrolled up: not following, content below the viewport.
/// The first draw settles the layout at this width; `scroll_up` then leaves follow mode.
fn scrolled_up_agent(cols: u16, rows: u16) -> AgentView {
    let mut agent = make_agent();
    seed_conversation(&mut agent);
    let _ = draw(&mut agent, cols, rows);
    agent.scrollback.scroll_up(5);
    assert!(
        !agent.scrollback.is_follow_mode() && agent.scrollback.has_content_below(),
        "fixture: scrolled up with content below"
    );
    agent
}

fn assert_inside(rect: Rect, area: Rect) {
    assert!(
        rect.x >= area.x
            && rect.x + rect.width <= area.right()
            && rect.y >= area.y
            && rect.y + rect.height <= area.bottom(),
        "chip {rect:?} must stay inside the scrollback {area:?}"
    );
}

#[test]
fn chip_sits_on_the_scrollback_last_row_over_the_gap_row() {
    let mut agent = scrolled_up_agent(DESKTOP_COLS, DESKTOP_ROWS);
    let buf = draw(&mut agent, DESKTOP_COLS, DESKTOP_ROWS);

    let sb = agent.pane_areas.scrollback;
    let rect = agent.hit_follow_indicator.rect.expect("chip visible");
    assert_eq!(
        rect.y,
        sb.bottom() - 1,
        "the chip owns the last scrollback row"
    );
    assert_eq!(rect.height, 1);
    // 24 columns: 21 chars + space + `↓` (U+2193, one terminal column).
    assert_eq!(rect.width, 24, "the hit covers the whole phrase");
    assert_inside(rect, sb);
    assert!(
        row_text(&buf, rect.y).contains(FULL_LABEL),
        "row {}: {:?}",
        rect.y,
        row_text(&buf, rect.y)
    );
    let gap = row_text(&buf, sb.bottom());
    assert!(
        !gap.contains("Jump to bottom"),
        "the chip never takes the gap row below the scrollback: {gap:?}"
    );
}

#[test]
fn chip_is_absent_at_the_bottom_and_leaves_no_hit_area() {
    let mut agent = make_agent();
    seed_conversation(&mut agent);
    let buf = draw(&mut agent, DESKTOP_COLS, DESKTOP_ROWS);

    assert!(
        agent.scrollback.is_follow_mode() && !agent.scrollback.has_content_below(),
        "fixture: pinned to the bottom"
    );
    assert!(!frame_text(&buf).contains("Jump to bottom"));
    assert!(agent.hit_follow_indicator.rect.is_none());
    assert!(!agent.hit_follow_indicator.hovered);
}

#[test]
fn a_stale_chip_hit_is_cleared_by_the_frame_that_hides_it() {
    let mut agent = scrolled_up_agent(DESKTOP_COLS, DESKTOP_ROWS);
    let _ = draw(&mut agent, DESKTOP_COLS, DESKTOP_ROWS);
    assert!(
        agent.hit_follow_indicator.rect.is_some(),
        "setup: chip was visible"
    );
    agent.hit_follow_indicator.hovered = true;

    agent.scrollback.goto_bottom();
    let buf = draw(&mut agent, DESKTOP_COLS, DESKTOP_ROWS);

    assert!(agent.scrollback.is_follow_mode());
    assert!(
        agent.hit_follow_indicator.rect.is_none(),
        "no invisible click target"
    );
    assert!(
        !agent.hit_follow_indicator.hovered,
        "hover dies with the chip"
    );
    assert!(!frame_text(&buf).contains(FULL_LABEL));
}

#[test]
fn phone_width_keeps_the_full_phrase_on_one_line_inside_the_scrollback() {
    let mut agent = scrolled_up_agent(PHONE_COLS, PHONE_ROWS);
    let buf = draw(&mut agent, PHONE_COLS, PHONE_ROWS);

    let sb = agent.pane_areas.scrollback;
    let rect = agent.hit_follow_indicator.rect.expect("chip visible");
    assert_eq!(rect.width, 24, "55 columns hold the full phrase");
    assert_eq!(rect.y, sb.bottom() - 1);
    assert_inside(rect, sb);
    eprintln!(
        "phone 55x41 scrolled up — around the last scrollback row:\n{}",
        ((sb.bottom() - 2)..=sb.bottom())
            .map(|y| format!("{y:>3} │{}", row_text(&buf, y)))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(
        frame_text(&buf).matches(FULL_LABEL).count(),
        1,
        "the phrase paints once, unwrapped"
    );
}

#[test]
fn a_scrollback_below_the_full_phrase_widens_to_the_short_form() {
    // 26 frame columns: inner width 24, one column of air either side leaves 22 — below the full
    // phrase's 26-column floor, above the short form's 18.
    let mut agent = scrolled_up_agent(26, PHONE_ROWS);
    let buf = draw(&mut agent, 26, PHONE_ROWS);

    let sb = agent.pane_areas.scrollback;
    assert_eq!(
        sb.width, 24,
        "fixture: one reserved outer column either side"
    );
    let rect = agent.hit_follow_indicator.rect.expect("chip visible");
    assert_eq!(rect.width, 16, "the short form's width");
    assert_eq!(rect.y, sb.bottom() - 1);
    assert_inside(rect, sb);
    let frame = frame_text(&buf);
    assert_eq!(frame.matches(SHORT_LABEL).count(), 1, "one line, no wrap");
    assert!(
        !frame.contains(FULL_LABEL),
        "the full phrase does not fit here"
    );
}

#[test]
fn a_scrollback_below_the_short_form_falls_back_to_the_bare_arrow() {
    // 19 frame columns: inner width 17, below the short form's 18-column floor.
    let mut agent = scrolled_up_agent(19, PHONE_ROWS);
    let buf = draw(&mut agent, 19, PHONE_ROWS);

    let sb = agent.pane_areas.scrollback;
    let rect = agent.hit_follow_indicator.rect.expect("chip visible");
    assert_eq!(rect.width, 1, "the arrow is one column");
    assert_eq!(rect.y, sb.bottom() - 1);
    assert_inside(rect, sb);
    let row = row_text(&buf, rect.y);
    assert!(row.contains('▼'), "row: {row:?}");
    assert!(!row.contains("Jump"), "row: {row:?}");
}

#[test]
fn the_indicator_setting_none_hides_the_chip() {
    let mut agent = scrolled_up_agent(DESKTOP_COLS, DESKTOP_ROWS);
    let mut appearance = agent.scrollback.appearance().clone();
    appearance.scrollback.scroll.follow_indicator = crate::appearance::FollowIndicator::None;
    agent.scrollback.set_appearance(appearance);

    let buf = draw(&mut agent, DESKTOP_COLS, DESKTOP_ROWS);

    assert!(
        !agent.scrollback.is_follow_mode() && agent.scrollback.has_content_below(),
        "the config gate is the only reason the chip is gone"
    );
    assert!(!frame_text(&buf).contains("Jump to bottom"));
    assert!(agent.hit_follow_indicator.rect.is_none());
}

#[test]
fn an_open_block_viewer_or_scrollback_search_hides_the_chip() {
    let mut viewer_agent = scrolled_up_agent(DESKTOP_COLS, DESKTOP_ROWS);
    viewer_agent.block_viewer = Some(crate::views::block_viewer::BlockViewerPane::for_plain_text(
        "t", "content",
    ));
    let buf = draw(&mut viewer_agent, DESKTOP_COLS, DESKTOP_ROWS);
    assert!(!frame_text(&buf).contains("Jump to bottom"));
    assert!(viewer_agent.hit_follow_indicator.rect.is_none());

    let mut search_agent = scrolled_up_agent(DESKTOP_COLS, DESKTOP_ROWS);
    search_agent.active_pane = crate::app::agent_view::AgentPane::Scrollback;
    search_agent.scrollback_search = Some(ScrollbackSearchState::open());
    let buf = draw(&mut search_agent, DESKTOP_COLS, DESKTOP_ROWS);
    assert!(!frame_text(&buf).contains("Jump to bottom"));
    assert!(search_agent.hit_follow_indicator.rect.is_none());
}

#[test]
fn hovered_chip_brightens_its_text_over_the_gray_background() {
    let theme = crate::theme::Theme::current();

    let mut agent = scrolled_up_agent(DESKTOP_COLS, DESKTOP_ROWS);
    let buf = draw(&mut agent, DESKTOP_COLS, DESKTOP_ROWS);
    let rect = agent.hit_follow_indicator.rect.expect("chip visible");
    let idle = buf.cell((rect.x, rect.y)).expect("chip cell");
    assert_eq!(idle.fg, theme.gray);
    assert_eq!(idle.bg, theme.bg_light);

    agent.hit_follow_indicator.hovered = true;
    let buf = draw(&mut agent, DESKTOP_COLS, DESKTOP_ROWS);
    let rect = agent.hit_follow_indicator.rect.expect("chip visible");
    let hovered = buf.cell((rect.x, rect.y)).expect("chip cell");
    assert_eq!(hovered.fg, theme.gray_bright);
    assert_eq!(hovered.bg, theme.bg_light);
}
