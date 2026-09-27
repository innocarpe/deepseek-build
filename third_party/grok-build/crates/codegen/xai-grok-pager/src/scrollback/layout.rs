use ratatui::layout::{Constraint, Layout, Rect};

use crate::appearance::LayoutConfig;

/// Horizontal layout columns for scrollback entries. Selection borders are drawn INTO the outer viewport padding,
/// not as part of this layout. Scrollbar is handled separately.
#[derive(Debug, Clone)]
pub struct HorizontalLayout {
    pub accent: Rect,
    pub left_padding: Rect,
    pub content: Rect,
    pub right_padding: Rect,
}

/// The columns an entry spends left and right of its own text.
///
/// Left: the accent column when the block paints a rail there or fills it with
/// its own band (the prompt echo's left gutter), plus the one column of air a
/// rail keeps before its body — and nothing at all for a block with neither, so
/// its text starts on the accent column: the minimum left unit. Right: the
/// band's own gutter where a block paints a band, nothing elsewhere, so text
/// runs to the entry area's last column, one column inside the frame.
///
/// Computed per entry (see `entry_chrome`) because one pane holds blocks with
/// different left edges: the echo's band, a thinking rail, and plain text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryChrome {
    pub accent: u16,
    pub right_pad: u16,
}

impl EntryChrome {
    /// The chrome a pane lays out with before it knows its entries: one accent
    /// column and the configured right pad.
    pub fn of_config(config: &LayoutConfig) -> Self {
        Self {
            accent: HorizontalLayout::ACCENT,
            right_pad: config.block_pad_right,
        }
    }

    /// Total columns between the entry area's edge and its text.
    pub fn chrome_width(&self, config: &LayoutConfig) -> u16 {
        self.accent + config.block_pad_left + self.right_pad
    }
}

impl HorizontalLayout {
    /// Accent width is always 1.
    pub const ACCENT: u16 = 1;

    pub fn new(area: Rect, config: &LayoutConfig) -> Self {
        Self::new_with_chrome(area, config, EntryChrome::of_config(config))
    }

    /// `area` laid out with the entry's own chrome (see [`EntryChrome`]): the
    /// accent column, the configured left pad, the text, and the right pad.
    pub fn new_with_chrome(area: Rect, config: &LayoutConfig, chrome: EntryChrome) -> Self {
        let [accent, left_padding, content, right_padding] = Layout::horizontal([
            Constraint::Length(chrome.accent),
            Constraint::Length(config.block_pad_left),
            Constraint::Min(1), // Content takes remaining space
            Constraint::Length(chrome.right_pad),
        ])
        .areas(area);

        Self {
            accent,
            left_padding,
            content,
            right_padding,
        }
    }

    /// `new` with the default config, kept for backwards compatibility.
    pub fn new_default(area: Rect) -> Self {
        Self::new(area, &LayoutConfig::default())
    }

    pub fn chrome_width(config: &LayoutConfig) -> u16 {
        Self::ACCENT + config.block_pad_left + config.block_pad_right
    }

    /// Get the area for rendering entry content (accent through right padding).
    /// This is the area passed to `EntryRenderer`.
    /// Layout: `│A│PL│Content│PR│`
    pub fn entry_content_area(&self) -> Rect {
        Rect {
            x: self.accent.x,
            y: self.accent.y,
            width: self.accent.width
                + self.left_padding.width
                + self.content.width
                + self.right_padding.width,
            height: self.accent.height,
        }
    }

    pub fn accent_area(&self) -> Rect {
        self.accent
    }

    /// Get the content width (for BlockContext).
    pub fn content_width(&self) -> u16 {
        self.content.width
    }

    pub fn entry_area(&self) -> Rect {
        self.entry_content_area()
    }

    /// Get the selection area (extends 1 column into outer padding on both sides). The selection border is drawn INTO
    /// the padding areas.
    pub fn selection_area(&self) -> Rect {
        // Selection extends 1 column left of accent into outer padding and 1 column right of entry into gap_left area
        let x = self.accent.x.saturating_sub(1);
        let width = self.entry_content_area().width + 2; // One column on each side

        Rect {
            x,
            y: self.accent.y,
            width,
            height: self.accent.height,
        }
    }

    /// Create a row-specific layout (same columns, different y/height).
    pub fn for_row(&self, y: u16, height: u16) -> Self {
        Self {
            accent: Rect {
                y,
                height,
                ..self.accent
            },
            left_padding: Rect {
                y,
                height,
                ..self.left_padding
            },
            content: Rect {
                y,
                height,
                ..self.content
            },
            right_padding: Rect {
                y,
                height,
                ..self.right_padding
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_horizontal_layout() {
        let config = LayoutConfig::default();
        let area = Rect::new(0, 0, 80, 10);
        let layout = HorizontalLayout::new(area, &config);

        // Check widths
        assert_eq!(layout.accent.width, 1);
        assert_eq!(layout.left_padding.width, config.block_pad_left);
        assert_eq!(layout.right_padding.width, config.block_pad_right);

        let chrome = HorizontalLayout::chrome_width(&config);
        assert_eq!(layout.content.width, 80 - chrome);

        // All have same height
        assert_eq!(layout.accent.height, 10);
        assert_eq!(layout.content.height, 10);
    }

    /// The default frame makes the accent rail's own column the whole left
    /// gutter: the text starts one column inside the band and two from the pane
    /// edge. The right edge keeps two columns — the rail column's mirror plus
    /// the column the scrollbar shares with the outer margin.
    #[test]
    fn default_layout_spends_one_column_on_the_left_gutter() {
        let area = Rect::new(0, 0, 53, 10);
        let config = LayoutConfig::default();
        let layout = HorizontalLayout::new(area, &config);

        assert_eq!(layout.accent.width, HorizontalLayout::ACCENT);
        assert_eq!(
            layout.left_padding.width, 0,
            "the accent rail's column is the whole left gutter"
        );
        assert_eq!(
            layout.right_padding.width, 2,
            "two at the right: the rail column's mirror plus the scrollbar's column"
        );
        assert_eq!(
            layout.content.x,
            layout.accent.right() + config.block_pad_left,
            "the text starts at the rail's right edge plus the (zero) left pad, got {:?} vs accent {:?}",
            layout.content,
            layout.accent,
        );
        assert_eq!(
            layout.content.width,
            area.width - HorizontalLayout::chrome_width(&config),
            "everything but the chrome is text"
        );
        assert_eq!(
            HorizontalLayout::chrome_width(&config),
            HorizontalLayout::ACCENT + 0 + 2,
            "the accent column plus the two gutter pads"
        );
        // Left gutter: outer margin + rail column. Right gutter: right pad +
        // the scrollbar/outer column. The right side stays one column wider by
        // design, and a later pad edit should keep that relation.
        assert_eq!(
            config.block_pad_right,
            config.block_pad_left + HorizontalLayout::ACCENT + 1,
            "the right side keeps the rail's mirror plus the scrollbar column"
        );
    }

    /// The one reserved outer column is the selection border's: the box's left
    /// border draws into the left margin column and its right border into the
    /// right one, so the whole box stays inside the pane at the resolved default.
    #[test]
    fn selection_border_fits_the_reserved_outer_column() {
        let config = LayoutConfig::default();
        let pane = Rect::new(0, 0, 55, 10);
        let inner = Rect::new(
            pane.x + config.eff_hpad_left(false),
            pane.y,
            pane.width - config.eff_hpad_left(false) - config.eff_hpad_right(false),
            pane.height,
        );
        let layout = HorizontalLayout::new(inner, &config);
        let selection = layout.selection_area();

        assert_eq!(
            selection.x, pane.x,
            "the left border draws into the reserved column"
        );
        assert_eq!(
            selection.right(),
            pane.right(),
            "and the right border into the opposite one, got {:?} in pane {:?}",
            selection,
            pane,
        );
    }

    #[test]
    fn test_entry_content_area() {
        let config = LayoutConfig::default();
        let area = Rect::new(5, 10, 80, 20);
        let layout = HorizontalLayout::new(area, &config);

        let entry_area = layout.entry_content_area();

        assert_eq!(entry_area.x, layout.accent.x);
        assert_eq!(
            entry_area.width,
            1 + config.block_pad_left + layout.content.width + config.block_pad_right
        );
    }

    #[test]
    fn test_for_row() {
        let config = LayoutConfig::default();
        let area = Rect::new(0, 0, 80, 10);
        let layout = HorizontalLayout::new(area, &config);

        let row_layout = layout.for_row(5, 3);

        assert_eq!(row_layout.accent.y, 5);
        assert_eq!(row_layout.accent.height, 3);
        assert_eq!(row_layout.content.y, 5);
        assert_eq!(row_layout.content.height, 3);
        // X positions should be unchanged
        assert_eq!(row_layout.accent.x, layout.accent.x);
        assert_eq!(row_layout.content.x, layout.content.x);
    }

    #[test]
    fn test_selection_area() {
        let config = LayoutConfig::default();
        // Area starts at x=5 (simulating outer padding already applied)
        let area = Rect::new(5, 10, 80, 20);
        let layout = HorizontalLayout::new(area, &config);

        let selection = layout.selection_area();

        // The selection extends 1 column left of the accent into the outer padding
        assert_eq!(selection.x, layout.accent.x - 1);
        assert_eq!(selection.width, layout.entry_content_area().width + 2);
        // Y and height same as accent
        assert_eq!(selection.y, layout.accent.y);
        assert_eq!(selection.height, layout.accent.height);
    }

    #[test]
    fn test_selection_area_at_edge() {
        let config = LayoutConfig::default();
        // Area starts at x=0 (no outer padding)
        let area = Rect::new(0, 0, 80, 10);
        let layout = HorizontalLayout::new(area, &config);

        let selection = layout.selection_area();

        // Selection at the edge saturates at x=0 (no underflow)
        assert_eq!(selection.x, 0);
        assert_eq!(selection.width, layout.entry_content_area().width + 2);
    }
}
