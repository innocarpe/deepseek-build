//! The `Jump to bottom` chip: scrolled up, the scrollback's last row carries a clickable chip
//! that returns to the bottom and re-engages follow mode (Claude Code parity, tappable on a phone).
//!
//! Frames here draw the whole agent view (`phone_bottom_tests` style) and assert the painted row,
//! the hit rect the frame leaves behind, and the frames that must hide the chip. The label ladder
//! and the unchanged one-glyph `▲` are pinned next to their primitives in `render.rs`.

use super::test_fixtures::make_agent;
use super::{AgentView, AppRenderParams, BannerSlotParams};
use crate::actions::ActionRegistry;
use crate::app::app_view::InputOutcome;
use crate::scrollback::block::RenderBlock;
use crate::scrollback::render::ScratchBuffer;
use crate::scrollback::search::ScrollbackSearchState;
use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::backend::{Backend, TestBackend};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

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
    // Half-block padding needs concrete colours; hold the theme lock so no other test can put
    // the process on the terminal-native palette mid-frame.
    let _theme = crate::theme::cache::pin_theme();
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
    assert_eq!(
        rect.width, 26,
        "the hit covers the phrase and both padding cells"
    );
    assert_inside(rect, sb);
    assert!(
        row_text(&buf, rect.y).contains(FULL_LABEL),
        "row {}: {:?}",
        rect.y,
        row_text(&buf, rect.y)
    );
    let theme = crate::theme::Theme::current();
    assert_ne!(
        theme.bg_light,
        Color::Reset,
        "fixture: concrete chip colours"
    );
    for (x, glyph) in [(rect.x, "▐"), (rect.right() - 1, "▌")] {
        let pad = &buf[(x, rect.y)];
        assert_eq!(pad.symbol(), glyph, "half-cell padding at column {x}");
        assert_eq!(pad.fg, theme.bg_light, "the pad inks the chip's colour");
        assert_eq!(pad.bg, theme.bg_base, "the outer half shows the canvas");
    }
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
    assert_eq!(rect.width, 26, "55 columns hold the padded full phrase");
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
    assert_eq!(rect.width, 18, "the padded short form's width");
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
    assert_eq!(rect.width, 3, "the arrow keeps a padding cell on each side");
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
    // The assertion samples `Theme::current()` before and after the draw; hold the lock so the
    // palette cannot change between them.
    let _theme = crate::theme::cache::pin_theme();
    let theme = crate::theme::Theme::current();

    let mut agent = scrolled_up_agent(DESKTOP_COLS, DESKTOP_ROWS);
    let buf = draw(&mut agent, DESKTOP_COLS, DESKTOP_ROWS);
    let rect = agent.hit_follow_indicator.rect.expect("chip visible");
    let idle = buf.cell((rect.x + 1, rect.y)).expect("label cell");
    assert_eq!(idle.fg, theme.gray);
    assert_eq!(idle.bg, theme.bg_light);

    agent.hit_follow_indicator.hovered = true;
    let buf = draw(&mut agent, DESKTOP_COLS, DESKTOP_ROWS);
    let rect = agent.hit_follow_indicator.rect.expect("chip visible");
    let hovered = buf.cell((rect.x + 1, rect.y)).expect("label cell");
    assert_eq!(hovered.fg, theme.gray_bright);
    assert_eq!(hovered.bg, theme.bg_light);
}

#[test]
fn both_padding_cells_hover_and_click_in_full_frames_at_every_label_width() {
    let _theme = crate::theme::cache::pin_theme();
    for cols in [PHONE_COLS, 26, 19] {
        for edge in [false, true] {
            let mut agent = scrolled_up_agent(cols, PHONE_ROWS);
            let buf = draw(&mut agent, cols, PHONE_ROWS);
            let rect = agent.hit_follow_indicator.rect.expect("chip visible");
            let x = if edge { rect.right() - 1 } else { rect.x };
            let theme = crate::theme::Theme::current();
            let pad = &buf[(x, rect.y)];
            assert_eq!(
                pad.symbol(),
                if edge { "▌" } else { "▐" },
                "half-cell padding is drawn on both ends"
            );
            assert_eq!(pad.fg, theme.bg_light, "the pad inks the chip's colour");
            assert_eq!(pad.bg, theme.bg_base, "the outer half shows the canvas");
            assert!(!agent.hit_follow_indicator.contains(rect.x - 1, rect.y));
            assert!(!agent.hit_follow_indicator.contains(rect.right(), rect.y));
            let mouse = |kind| MouseEvent {
                kind,
                column: x,
                row: rect.y,
                modifiers: KeyModifiers::NONE,
            };
            agent.handle_mouse(&mouse(MouseEventKind::Moved));
            assert!(agent.hit_follow_indicator.hovered);
            let hovered = draw(&mut agent, cols, PHONE_ROWS);
            assert_eq!(
                hovered[(rect.x + 1, rect.y)].fg,
                crate::theme::Theme::current().gray_bright
            );
            assert!(matches!(
                agent.handle_mouse(&mouse(MouseEventKind::Down(MouseButton::Left))),
                InputOutcome::Changed
            ));
            assert!(
                agent.scrollback.is_follow_mode(),
                "padding clicks return to bottom"
            );
            let hidden = draw(&mut agent, cols, PHONE_ROWS);
            assert!(agent.hit_follow_indicator.rect.is_none());
            assert!(!agent.hit_follow_indicator.hovered);
            assert!(!frame_text(&hidden).contains("Jump to bottom"));
        }
    }
}

#[test]
fn scrolling_styled_wide_text_keeps_the_chip_opaque_in_buffer_diff_output() {
    // The frame's colours (chip, canvas, the exposed trailing cell's real background) are concrete
    // only off the terminal-native palette; hold the theme lock for the whole scroll.
    let _theme = crate::theme::cache::pin_theme();
    let mut agent = make_agent();
    for i in 0..40 {
        let prefix = if i % 2 == 0 { "" } else { "x" };
        let text = match i % 3 {
            0 => format!("**{prefix}{}**", "가".repeat(45)),
            1 => format!("`{prefix}{}`", "🍊".repeat(45)),
            _ => format!("{prefix}{}", "한".repeat(45)),
        };
        agent
            .scrollback
            .push_block(RenderBlock::agent_message(text));
    }
    let mut previous = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
    let mut backend = TestBackend::new(PHONE_COLS, PHONE_ROWS);
    backend
        .draw(Buffer::empty(previous.area).diff(&previous).into_iter())
        .unwrap();
    agent.scrollback.scroll_up(5);
    let mut saw_crossing_glyph = false;
    let mut saw_right_edge_glyph = false;
    let mut saw_styled_text = false;
    for _ in 0..12 {
        // Paint the same frame without the chip to measure the content it covers.
        let mut appearance = agent.scrollback.appearance().clone();
        let indicator = appearance.scrollback.scroll.follow_indicator;
        appearance.scrollback.scroll.follow_indicator = crate::appearance::FollowIndicator::None;
        agent.scrollback.set_appearance(appearance.clone());
        let underneath = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
        appearance.scrollback.scroll.follow_indicator = indicator;
        agent.scrollback.set_appearance(appearance);
        let next = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
        let rect = agent
            .hit_follow_indicator
            .rect
            .expect("scrolled-up chip visible");
        saw_crossing_glyph |=
            unicode_width::UnicodeWidthStr::width(underneath[(rect.x - 1, rect.y)].symbol()) > 1;
        saw_styled_text |=
            (rect.x..rect.right()).any(|x| !underneath[(x, rect.y)].modifier.is_empty());
        // A wide glyph starting on the chip's last column loses its hidden trailing blank to the
        // chip; that cell must not reach the terminal as a default-coloured hole.
        if unicode_width::UnicodeWidthStr::width(underneath[(rect.right() - 1, rect.y)].symbol())
            > 1
        {
            saw_right_edge_glyph = true;
            let trail = &next[(rect.right(), rect.y)];
            assert_eq!(trail.symbol(), " ");
            assert_ne!(
                trail.bg,
                Color::Reset,
                "the trailing cell the chip exposed must carry a real background"
            );
        }
        let updates = previous.diff(&next);
        assert!(
            updates
                .iter()
                .filter(|(_, y, _)| *y == rect.y)
                .all(|(_, _, cell)| cell.bg != Color::Reset),
            "no emitted cell on the chip's row may fall back to the terminal default"
        );
        backend.draw(updates.into_iter()).unwrap();
        let theme = crate::theme::Theme::current();
        let expected: Vec<String> = std::iter::once("▐".to_string())
            .chain(FULL_LABEL.chars().map(|c| c.to_string()))
            .chain(std::iter::once("▌".to_string()))
            .collect();
        for (i, symbol) in expected.iter().enumerate() {
            let x = rect.x + i as u16;
            let padding = i == 0 || i == expected.len() - 1;
            let cell = &backend.buffer()[(x, rect.y)];
            assert_eq!(cell.symbol(), symbol, "scrolling lost chip column {x}");
            assert_eq!(cell.fg, if padding { theme.bg_light } else { theme.gray });
            assert_eq!(
                cell.bg,
                if padding {
                    theme.bg_base
                } else {
                    theme.bg_light
                }
            );
            assert_eq!(cell.modifier, Modifier::empty());
            assert!(!cell.skip);
        }
        // The frame's last column is the scrollbar's. The chip leaves it alone, so nothing in
        // this row's right edge is chip residue; the only cells the chip owns end here.
        assert_eq!(
            next[(PHONE_COLS - 1, rect.y)],
            underneath[(PHONE_COLS - 1, rect.y)],
            "the chip must not disturb the scrollbar column"
        );
        previous = next;
        agent.scrollback.scroll_up(1);
    }
    assert!(
        saw_crossing_glyph,
        "fixture must cross the chip's left boundary with a wide glyph"
    );
    assert!(
        saw_right_edge_glyph,
        "fixture must start a wide glyph on the chip's right edge"
    );
    assert!(
        saw_styled_text,
        "fixture must scroll formatted text under the chip"
    );
}
