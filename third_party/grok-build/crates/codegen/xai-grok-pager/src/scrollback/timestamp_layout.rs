//! Where a turn clock sits.
//!
//! Narrow panes (`effective_narrow`, `layout.narrow`) never reserve columns:
//! the clock uses leftover space on the first content line, or one right-aligned
//! meta row above the body after the turn finishes. Wide panes keep the
//! right-edge clock and subtract its columns from the first content line only.

use chrono::{DateTime, Local, Timelike};
use unicode_width::UnicodeWidthStr;

use super::block::RenderBlock;
use super::entry::ScrollbackEntry;
use crate::appearance::AppearanceConfig;

/// Widest short clock (`"12:59 PM"`). Wide panes subtract this from the first content line only.
pub(crate) const TIMESTAMP_SHORT_MAX_COLS: u16 = 8;

/// Columns a wide pane takes from the first content line so the short clock and one blank column fit.
/// Narrow panes, and `show_timestamps == false`, reserve nothing.
pub(crate) fn wide_first_line_reserve(appearance: &AppearanceConfig) -> u16 {
    if appearance.show_timestamps && !appearance.scrollback.layout.narrow {
        TIMESTAMP_SHORT_MAX_COLS
    } else {
        0
    }
}

/// User prompts, agent messages, and `/btw` replies carry a clock. Tools, thoughts, and system rows do not.
pub(crate) fn timestamp_gutter_applies(block: &RenderBlock) -> bool {
    matches!(
        block,
        RenderBlock::UserPrompt(_) | RenderBlock::AgentMessage(_) | RenderBlock::Btw(_)
    )
}

pub(crate) fn short_clock(ts: DateTime<Local>) -> String {
    ts.format("%-I:%M %p").to_string()
}

pub(crate) fn long_clock(ts: DateTime<Local>) -> String {
    ts.format("%H:%M:%S | %b %d").to_string()
}

pub(crate) fn clock_cols(text: &str) -> u16 {
    UnicodeWidthStr::width(text).min(u16::MAX as usize) as u16
}

pub(crate) fn line_cols(line: &ratatui::text::Line<'_>) -> u16 {
    line.spans
        .iter()
        .map(|span| UnicodeWidthStr::width(span.content.as_ref()))
        .sum::<usize>()
        .min(u16::MAX as usize) as u16
}

pub(crate) fn same_clock_minute(a: DateTime<Local>, b: DateTime<Local>) -> bool {
    a.date_naive() == b.date_naive()
        && a.time().hour() == b.time().hour()
        && a.time().minute() == b.time().minute()
}

/// The clock this entry would paint, if it paints one.
///
/// A narrow entry that is still running paints nothing and does not consume the minute.
/// A finished entry that suppresses its meta row still carries the same minute as the
/// clock it would have repeated, so later rows can use this value as the anchor.
pub(crate) fn timestamp_anchor(
    entry: &ScrollbackEntry,
    appearance: &AppearanceConfig,
) -> Option<DateTime<Local>> {
    if !appearance.show_timestamps || !timestamp_gutter_applies(&entry.block) {
        return None;
    }
    if appearance.scrollback.layout.narrow && entry.is_running {
        return None;
    }
    entry.created_at
}

/// One blank column between the last body glyph and the clock.
fn fits(text_cols: u16, clock: &str, row_span: u16) -> bool {
    text_cols
        .saturating_add(1)
        .saturating_add(clock_cols(clock))
        <= row_span
}

/// What to paint for one entry. `meta` is a layout row; hover must not flip it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClockPlan {
    pub meta: bool,
    pub text: Option<String>,
}

pub(crate) struct ClockQuery<'a> {
    pub entry: &'a ScrollbackEntry,
    pub appearance: &'a AppearanceConfig,
    /// Display columns of the first content line. Ignored when `has_content` is false.
    pub first_line_cols: u16,
    /// Columns from the text origin to the entry's right edge.
    pub row_span: u16,
    pub prev_clock: Option<DateTime<Local>>,
    /// Mouse is over the clock cells. Height passes `false` so a hover cannot add a row.
    pub hovered: bool,
    /// Clipped sticky fragments may keep the short format.
    pub allow_long: bool,
    pub has_content: bool,
}

impl ClockPlan {
    pub(crate) fn decide(q: &ClockQuery<'_>) -> Self {
        let Some(ts) = q.entry.created_at else {
            return Self::none();
        };
        if !q.has_content
            || !q.appearance.show_timestamps
            || !timestamp_gutter_applies(&q.entry.block)
        {
            return Self::none();
        }
        let narrow = q.appearance.scrollback.layout.narrow;
        if narrow && q.entry.is_running {
            return Self::none();
        }

        let short = short_clock(ts);
        let long = long_clock(ts);
        let short_fits = fits(q.first_line_cols, &short, q.row_span);
        let long_fits = fits(q.first_line_cols, &long, q.row_span);
        let expanded = q.allow_long && q.entry.timestamp_expanded;
        let want_long = expanded || (q.allow_long && q.hovered);

        if want_long && long_fits {
            return Self {
                meta: false,
                text: Some(long),
            };
        }
        // A tap that opens the long form, when that form does not fit, takes the meta
        // row. The body keeps the wrap it already had.
        if expanded && !long_fits {
            return Self {
                meta: true,
                text: Some(long),
            };
        }
        if short_fits {
            return Self {
                meta: false,
                text: Some(short),
            };
        }
        if !narrow {
            // A wide first line is wrapped to leave the short clock room. A row that
            // still fills the span (a table that does not wrap) is left alone.
            return Self::none();
        }
        let same = q.prev_clock.is_some_and(|prev| same_clock_minute(prev, ts));
        if same {
            return Self::none();
        }
        let text = if want_long { long } else { short };
        Self {
            meta: true,
            text: Some(text),
        }
    }

    fn none() -> Self {
        Self {
            meta: false,
            text: None,
        }
    }
}

/// Right edge of a painted clock inside an entry area whose right edge is `entry_right`.
pub(crate) fn clock_origin(entry_right: u16, text: &str) -> u16 {
    entry_right.saturating_sub(clock_cols(text))
}

pub(crate) fn point_in_clock(col: u16, row: u16, clock_x: u16, clock_y: u16, clock_w: u16) -> bool {
    row == clock_y && col >= clock_x && col < clock_x.saturating_add(clock_w)
}
