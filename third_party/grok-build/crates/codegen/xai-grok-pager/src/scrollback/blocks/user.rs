use std::ops::Range;

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

use crate::render::wrapping::{RtOptions, word_wrap_line_with_joiners};
use crate::scrollback::block::BlockContent;
use crate::scrollback::types::{
    AccentStyle, BlockBackground, BlockContext, BlockLine, BlockOutput, DisplayMode, Selectable,
};

const USER_PROMPT_BODY_RANGE: u16 = 0;
/// Max visible lines when a user prompt is collapsed on a roomy terminal.
const COLLAPSED_MAX_LINES: usize = 3;

/// Width (in columns) at or below which a collapsed prompt drops to
/// [`COLLAPSED_NARROW_MAX_LINES`]. The measured iPhone Orca pane is 55 columns
/// (the narrowest desktop pane on the same machine is 80). One row names the
/// turn and cuts the rest of the prompt; two rows keep enough of it to read,
/// and the echo band has no vertical padding, so the extra row is one line of
/// text. Anything wider keeps [`COLLAPSED_MAX_LINES`], so the desktop layout
/// is untouched.
///
/// [`NARROW_TERMINAL_COLS`](crate::appearance::NARROW_TERMINAL_COLS) is the same
/// number: the whole frame's phone-width density keys off it, and this alias
/// keeps the echo fold and the composer prefix on that one threshold.
///
/// `pub(crate)` so the input box can apply the same narrow-pane rule to its decorative `❯` without a second
/// threshold. The echo and the composer agree about what "phone width" means.
pub(crate) const COLLAPSED_NARROW_TERMINAL_COLS: u16 = crate::appearance::NARROW_TERMINAL_COLS;

/// Max visible lines when a user prompt is collapsed on a narrow terminal.
///
/// Two, not one: a single row on a 55-column pane hides most of a submitted
/// prompt, and the band is already tight, so a second row is the context.
const COLLAPSED_NARROW_MAX_LINES: usize = 2;

/// The collapse budget for a prompt rendered at `width` into `mode`.
///
/// `Expanded` never folds; otherwise the budget is [`COLLAPSED_MAX_LINES`],
/// tightened to [`COLLAPSED_NARROW_MAX_LINES`] when `width` is at or below
/// [`COLLAPSED_NARROW_TERMINAL_COLS`]. Deriving this from the render width
/// (rather than reading a single constant) is what lets the same block fold
/// harder in a phone-width pane than in a desktop one.
///
/// A budget only decides how many rows are *shown*. Whether the block folds at all is
/// [`UserPromptBlock::is_foldable_at`]'s call, and that decision has to be width-aware too: a width-blind
/// estimate left this budget unreached in a phone-width pane.
fn collapsed_max_lines(width: u16, mode: DisplayMode) -> Option<usize> {
    match mode {
        DisplayMode::Expanded => None,
        DisplayMode::Collapsed | DisplayMode::Truncated => {
            if width <= COLLAPSED_NARROW_TERMINAL_COLS {
                Some(COLLAPSED_NARROW_MAX_LINES)
            } else {
                Some(COLLAPSED_MAX_LINES)
            }
        }
    }
}

use crate::appearance::AppearanceConfig;
use crate::theme::Theme;

/// Drop invalid token ranges (replayed session metadata is untrusted): out of bounds, not on char boundaries, or empty.
/// Survivors are sorted; overlaps with an earlier kept range are dropped so span slicing never goes backwards.
fn sanitize_token_ranges(text: &str, mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    ranges.retain(|r| {
        r.start < r.end
            && r.end <= text.len()
            && text.is_char_boundary(r.start)
            && text.is_char_boundary(r.end)
    });
    ranges.sort_by_key(|r| (r.start, r.end));
    let mut out: Vec<Range<usize>> = Vec::new();
    for r in ranges {
        if out.last().is_some_and(|prev| r.start < prev.end) {
            continue;
        }
        out.push(r);
    }
    out
}

/// Split one logical line into token/body spans by intersecting the line's byte span in the block text with the (sanitized) token ranges.
fn token_styled_line(
    line_text: &str,
    line_start: usize,
    ranges: &[Range<usize>],
    token_style: Style,
    body_style: Style,
) -> Line<'static> {
    let line_end = line_start + line_text.len();
    let mut spans = Vec::new();
    let mut pos = line_start;
    for r in ranges {
        let start = r.start.clamp(line_start, line_end);
        let end = r.end.clamp(line_start, line_end);
        if start >= end {
            continue;
        }
        if start > pos {
            let Some(body) = pos.checked_sub(line_start).and_then(|a| {
                start
                    .checked_sub(line_start)
                    .and_then(|b| line_text.get(a..b))
            }) else {
                continue;
            };
            spans.push(Span::styled(body.to_string(), body_style));
        }
        let Some(token) = start.checked_sub(line_start).and_then(|a| {
            end.checked_sub(line_start)
                .and_then(|b| line_text.get(a..b))
        }) else {
            continue;
        };
        spans.push(Span::styled(token.to_string(), token_style));
        pos = end;
    }
    if pos < line_end
        && let Some(body) = pos.checked_sub(line_start).and_then(|a| line_text.get(a..))
    {
        spans.push(Span::styled(body.to_string(), body_style));
    }
    Line::from(spans)
}

#[derive(Debug, Clone)]
pub struct UserPromptBlock {
    pub text: String,
    /// Whether this was a bash command (! prefix).
    pub is_bash: bool,
    /// Whether this prompt was injected by the scheduler (cron/loop).
    pub is_cron: bool,
    /// Mid-turn interjection.
    /// Renders identically to a typed prompt but is excluded from shell prompt-index bookkeeping.
    /// The shell numbers only turn-starting prompts, so counting interjections would skew the positional prompt-to-entry mapping rewind uses.
    pub is_interjection: bool,
    pub prompt_index: Option<usize>,
    /// Sanitized byte ranges into `text` rendered in the skill accent color (recognized `/command` tokens).
    /// Empty means plain prompt styling.
    /// This is the sole skill signal; a leading skill invocation is `[0..token_end]`.
    pub skill_token_ranges: Vec<Range<usize>>,
}

impl UserPromptBlock {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_bash: false,
            is_cron: false,
            is_interjection: false,
            prompt_index: None,
            skill_token_ranges: Vec::new(),
        }
    }

    pub fn copy_text(&self) -> String {
        self.text.clone()
    }

    pub fn bash(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_bash: true,
            is_cron: false,
            is_interjection: false,
            prompt_index: None,
            skill_token_ranges: Vec::new(),
        }
    }

    /// The leading token on the first line (up to whitespace, or the whole line) gets the skill accent.
    pub fn skill(text: impl Into<String>) -> Self {
        let text = text.into();
        let first_line = text.lines().next().unwrap_or("");
        let token_end = first_line
            .find(char::is_whitespace)
            .unwrap_or(first_line.len());
        #[allow(clippy::single_range_in_vec_init)] // field is multi-range capable
        let skill_token_ranges = if token_end > 0 {
            vec![0..token_end]
        } else {
            Vec::new()
        };
        Self {
            text,
            is_bash: false,
            is_cron: false,
            is_interjection: false,
            prompt_index: None,
            skill_token_ranges,
        }
    }

    /// Create a plain user prompt with recognized mid-text slash tokens styled in the skill accent.
    /// Ranges are sanitized here: replayed session metadata is untrusted, so invalid ranges are dropped rather than panicking.
    pub fn with_skill_tokens(text: impl Into<String>, ranges: Vec<Range<usize>>) -> Self {
        let text = text.into();
        let skill_token_ranges = sanitize_token_ranges(&text, ranges);
        Self {
            text,
            is_bash: false,
            is_cron: false,
            is_interjection: false,
            prompt_index: None,
            skill_token_ranges,
        }
    }

    pub fn cron(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_bash: false,
            is_cron: true,
            is_interjection: false,
            prompt_index: None,
            skill_token_ranges: Vec::new(),
        }
    }

    pub fn interjection(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_bash: false,
            is_cron: false,
            is_interjection: true,
            prompt_index: None,
            skill_token_ranges: Vec::new(),
        }
    }

    /// The elevated band behind user-prompt rows in the full TUI.
    /// Minimal / terminal-native keeps bold only — no fill.
    ///
    /// RGB themes: `bg_light` (matches the fullscreen prompt band).
    ///
    /// Takes `theme` so unit tests can call it without toggling `terminal_native_lock`, which races `Theme::current()`
    /// tests that do not hold the theme test mutex.
    fn prompt_band_color_for(
        theme: &Theme,
        _is_selected: bool,
        terminal_native: bool,
    ) -> Option<ratatui::style::Color> {
        use ratatui::style::Color;
        if terminal_native {
            None
        } else {
            match theme.bg_light {
                Color::Reset => None,
                c => Some(c),
            }
        }
    }

    /// Prefix/body/skill styles. Bold only when `terminal_native` (minimal mode). Minimal keeps a Cyan pointer when
    /// `accent_user` is Reset (its prompt rows have no band, so the pointer is the turn cue).
    fn prompt_styles(theme: &Theme, terminal_native: bool) -> (Style, Style, Style) {
        let prefix_color = match theme.accent_user {
            ratatui::style::Color::Reset if terminal_native => ratatui::style::Color::Cyan,
            c => c,
        };
        let mut prefix_style = theme.fg(prefix_color);
        let mut text_style = theme.fg(theme.text_primary);
        let mut skill_style = theme.fg(theme.accent_skill);
        if terminal_native {
            prefix_style = prefix_style.add_modifier(Modifier::BOLD);
            text_style = text_style.add_modifier(Modifier::BOLD);
            skill_style = skill_style.add_modifier(Modifier::BOLD);
        }
        (prefix_style, text_style, skill_style)
    }

    /// The prefix this prompt draws at `width`, or `""` for none.
    ///
    /// `$ ` (bash) and `↻  ` (cron) say what kind of turn this was, so they stay whenever the caller asked for a
    /// prefix. `❯ ` is decoration: above the narrow threshold it marks the prompt, and at or below it the band
    /// background already does that, so the two columns go back to the text. `show_prefix = false` still means no
    /// prefix at all.
    ///
    /// Both `wrap_prompt_lines` and [`Self::is_foldable_at`] call this, so the row count cannot assume a prefix the
    /// renderer did not draw.
    fn prefix_for(&self, show_prefix: bool, width: u16) -> &'static str {
        if !show_prefix {
            ""
        } else if self.is_bash {
            "$ "
        } else if self.is_cron {
            "\u{21BB}  "
        } else if width <= COLLAPSED_NARROW_TERMINAL_COLS {
            ""
        } else {
            crate::glyphs::prompt_arrow()
        }
    }

    /// Wrap and style the prompt text, returning visual lines.
    /// When `max_lines` is set and content exceeds it, the last line is truncated with a " …" ellipsis.
    fn wrap_prompt_lines(
        &self,
        width: u16,
        max_lines: Option<usize>,
        show_prefix: bool,
        is_selected: bool,
    ) -> Vec<BlockLine> {
        let theme = Theme::current();
        // Minimal mode engages this lock; read it here instead of app state.
        let terminal_native = crate::theme::cache::terminal_native_locked();
        // The terminal theme (fullscreen) renders prompts bandless: bold
        // primary text instead of a bright-black band, which can sit too
        // close to the default fg on some profiles. Minimal keeps its band.
        let attribute_emphasis = !terminal_native && crate::theme::cache::terminal_native_active();
        let (mut prefix_style, mut text_style, mut skill_style) =
            Self::prompt_styles(&theme, terminal_native);
        if attribute_emphasis {
            prefix_style = prefix_style.add_modifier(Modifier::BOLD);
            text_style = text_style.add_modifier(Modifier::BOLD);
            skill_style = skill_style.add_modifier(Modifier::BOLD);
        }
        let band = Self::prompt_band_color_for(&theme, is_selected, terminal_native);
        // Semantic line bg (not a "panel") so it survives minimal's flat_background.
        // Bandless prompts (terminal theme) carry no extra selected cue: the
        // rewind picker and the dimmed tail already mark the target.
        let with_band = |line: BlockLine| -> BlockLine {
            match band {
                Some(c) => line.with_background(c),
                None => line,
            }
        };

        let prefix = self.prefix_for(show_prefix, width);
        let prefix_width = prefix.width();
        let has_visible_prefix = prefix_width > 0;
        let ellipsis = " \u{2026}";
        let ellipsis_width = 2;

        let base_content_width = (width as usize).saturating_sub(prefix_width).max(1);

        let mut all_lines: Vec<BlockLine> = Vec::new();
        let logical_lines: Vec<&str> = self.text.lines().collect();
        let total_logical = logical_lines.len();

        for (logical_idx, line_text) in logical_lines.iter().enumerate() {
            if line_text.is_empty() {
                // Empty line: just show prefix/indent
                let indent = " ".repeat(prefix_width);
                let line = if logical_idx == 0 {
                    Line::from(vec![Span::styled(prefix.to_string(), prefix_style)])
                } else {
                    Line::from(vec![Span::styled(indent, prefix_style)])
                };
                let block_line = with_band(BlockLine {
                    selectable: if has_visible_prefix {
                        Selectable::Spans(1..1) // empty content range past prefix
                    } else {
                        Selectable::All
                    },
                    selection_range: Some(USER_PROMPT_BODY_RANGE),
                    content: line,
                    ..Default::default()
                });
                all_lines.push(block_line);

                // Check if we hit max_lines
                if let Some(max) = max_lines
                    && all_lines.len() >= max
                {
                    let has_more = logical_idx + 1 < total_logical;
                    if has_more {
                        // Add ellipsis to last line
                        if let Some(last) = all_lines.last_mut() {
                            last.content
                                .spans
                                .push(Span::styled(ellipsis.to_string(), text_style));
                        }
                    }
                    return all_lines;
                }
                continue;
            }

            // Wrap this line's content.
            let content_line = if self.skill_token_ranges.is_empty() {
                Line::from(Span::styled(line_text.to_string(), text_style))
            } else {
                // `lines()` strips terminators, so recover this line's byte offset from the subslice pointer into `self.text`
                debug_assert!(
                    self.text
                        .as_bytes()
                        .as_ptr_range()
                        .contains(&line_text.as_ptr()),
                    "line_text must be a subslice of self.text"
                );
                let line_start = line_text.as_ptr() as usize - self.text.as_ptr() as usize;
                token_styled_line(
                    line_text,
                    line_start,
                    &self.skill_token_ranges,
                    skill_style,
                    text_style,
                )
            };
            let (wrapped, wrap_joiners) =
                word_wrap_line_with_joiners(&content_line, RtOptions::new(base_content_width));
            let wrapped_count = wrapped.len();

            for (wrap_idx, (wrapped_line, wrap_joiner)) in
                wrapped.into_iter().zip(wrap_joiners).enumerate()
            {
                let is_first_line = logical_idx == 0 && wrap_idx == 0;
                let indent: String = " ".repeat(prefix_width);
                let line_prefix = if is_first_line { prefix } else { &indent };

                let will_be_last = max_lines.is_some_and(|max| all_lines.len() + 1 == max);

                // Check if there's more content after this line
                let has_more_wrapped = wrap_idx + 1 < wrapped_count;
                let has_more_logical = logical_idx + 1 < total_logical;
                let has_more = has_more_wrapped || has_more_logical;

                // First line of block or new logical line: hard break (None).
                // Continuation within same logical line: use wrap joiner.
                let joiner = if is_first_line || wrap_idx == 0 {
                    None
                } else {
                    wrap_joiner
                };

                if will_be_last && has_more {
                    // Re-wrap the current line's content with reduced width to make room for the ellipsis
                    // Re-wrapping the styled line (not flattened text) keeps token spans teal here
                    let reduced_width = base_content_width.saturating_sub(ellipsis_width);
                    let (re_wrapped_lines, _) =
                        word_wrap_line_with_joiners(&wrapped_line, RtOptions::new(reduced_width));

                    let final_content = re_wrapped_lines
                        .into_iter()
                        .next()
                        .unwrap_or_else(Line::default);

                    // Build final line with prefix, content, and ellipsis
                    let mut spans = vec![Span::styled(line_prefix.to_string(), prefix_style)];
                    spans.extend(final_content.spans.into_iter().map(|s| Span {
                        content: s.content.to_string().into(),
                        style: s.style,
                    }));
                    spans.push(Span::styled(ellipsis.to_string(), text_style));

                    let content_start = if has_visible_prefix { 1 } else { 0 };
                    let content_end = spans.len() - 1; // exclude ellipsis
                    let block_line = with_band(BlockLine {
                        content: Line::from(spans),
                        selectable: if content_start < content_end {
                            Selectable::Spans(content_start..content_end)
                        } else {
                            Selectable::Spans(content_start..content_start)
                        },
                        selection_range: Some(USER_PROMPT_BODY_RANGE),
                        joiner,
                        ..Default::default()
                    });
                    all_lines.push(block_line);
                    return all_lines;
                }

                // Normal line (not truncated)
                let mut spans = vec![Span::styled(line_prefix.to_string(), prefix_style)];
                spans.extend(wrapped_line.spans.into_iter().map(|s| Span {
                    content: s.content.to_string().into(),
                    style: s.style,
                }));
                let content_end = spans.len();
                let block_line = with_band(BlockLine {
                    content: Line::from(spans),
                    selectable: if has_visible_prefix {
                        Selectable::Spans(1..content_end)
                    } else {
                        Selectable::All
                    },
                    selection_range: Some(USER_PROMPT_BODY_RANGE),
                    joiner,
                    ..Default::default()
                });
                all_lines.push(block_line);

                // Stop at max_lines (the case where no more content follows)
                if let Some(max) = max_lines
                    && all_lines.len() >= max
                {
                    return all_lines;
                }
            }
        }

        // Handle empty text
        if all_lines.is_empty() {
            all_lines.push(with_band(BlockLine {
                content: Line::from(Span::styled(prefix.trim().to_string(), prefix_style)),
                selectable: Selectable::All,
                selection_range: Some(USER_PROMPT_BODY_RANGE),
                ..Default::default()
            }));
        }

        all_lines
    }
}

impl BlockContent for UserPromptBlock {
    fn output(&self, ctx: &BlockContext) -> BlockOutput {
        let max_lines = collapsed_max_lines(ctx.width, ctx.mode);

        let prompt_cfg = &ctx.appearance.scrollback.blocks.prompt;
        let compact = ctx.appearance.prompt.compact;
        let lines = self.wrap_prompt_lines(
            ctx.width,
            max_lines,
            prompt_cfg.show_prefix && !compact,
            ctx.is_selected,
        );

        BlockOutput { lines }
    }

    fn accent(&self, _ctx: &BlockContext) -> Option<AccentStyle> {
        None
    }

    fn accent_background(&self, _ctx: &BlockContext) -> bool {
        true // fill accent column with block bg so it matches content
    }

    fn background(&self, ctx: &BlockContext) -> BlockBackground {
        ctx.appearance.scrollback.blocks.prompt.bg
    }

    fn has_vpad_for(&self, appearance: &AppearanceConfig) -> bool {
        appearance.scrollback.blocks.prompt.vpad && !appearance.prompt.compact
    }

    /// On a phone-width pane the prompt echo is a two-row band, and the two blank pad rows around it cost as much
    /// vertical space as the band itself. Drop the pad there. Wider panes keep the configured pad, so the desktop
    /// rhythm is untouched. The threshold is the same [`COLLAPSED_NARROW_TERMINAL_COLS`] the narrow fold already uses.
    ///
    /// This rule survives the trait default's narrow-pane drop (`layout.narrow`) because it is the stricter one on a
    /// mid-width pane: there the pane is wide but the echo's own content column is not.
    fn has_vpad_for_width(&self, appearance: &AppearanceConfig, content_width: u16) -> bool {
        content_width > COLLAPSED_NARROW_TERMINAL_COLS && self.has_vpad_for(appearance)
    }

    fn has_raw_mode(&self) -> bool {
        false
    }

    /// Width-blind foldability, kept for callers that have no render width. Estimates the visual row count with a
    /// conservative content width and so under-reports for panes narrower than the estimate. Callers that know the
    /// real content width must use [`Self::is_foldable_at`].
    fn is_foldable(&self) -> bool {
        // Estimate visual line count to catch long single-line prompts that wrap past the limit. Uses a conservative
        // content width (terminal width minus prefix/padding). at wider terminals we may slightly over-report foldability,
        // which is harmless.
        const MIN_CONTENT_WIDTH: usize = 60;
        let mut visual_lines = 0usize;
        for line in self.text.lines() {
            let w = line.width();
            visual_lines += if w == 0 {
                1
            } else {
                w.div_ceil(MIN_CONTENT_WIDTH)
            };
            if visual_lines > COLLAPSED_MAX_LINES {
                return true;
            }
        }
        false
    }

    /// Width-aware foldability: does this prompt exceed the collapse budget it would actually be rendered under at
    /// `content_width`?
    ///
    /// The width-blind [`Self::is_foldable`] divides by a 60-column constant, which is wider than a phone pane's real
    /// content width. A ~100-column one-liner therefore scored `ceil(100/60) = 2` rows — under the three-row roomy
    /// threshold — so the block reported "not foldable" and `default_display_mode()` handed it `Expanded`. `Expanded`
    /// maps to `max_lines = None`, so the narrow budget [`collapsed_max_lines`] computes for that same width never got
    /// applied and the echo stayed at full body height.
    ///
    /// Wrapping mirrors `wrap_prompt_lines` through [`Self::prefix_for`]: the prefix it reports (the `❯ ` on a roomy
    /// pane, nothing on a narrow one, `$ ` / `↻  ` for bash and cron either way) is subtracted once, because every
    /// row is indented by it. Row counts are `ceil(line_width / wrap_width)`, a lower bound on word-boundary wrapping.
    /// A `None` budget means the mode never folds.
    fn is_foldable_at(&self, content_width: u16) -> bool {
        let Some(budget) = collapsed_max_lines(content_width, DisplayMode::Collapsed) else {
            return false;
        };
        // Same helper the renderer uses, including the columns the dropped arrow hands back on a narrow pane.
        let prefix_width = self.prefix_for(true, content_width).width();
        let wrap_width = usize::from(content_width)
            .saturating_sub(prefix_width)
            .max(1);
        let mut visual_lines = 0usize;
        for line in self.text.lines() {
            let w = line.width();
            visual_lines += if w == 0 { 1 } else { w.div_ceil(wrap_width) };
            if visual_lines > budget {
                return true;
            }
        }
        false
    }

    /// The off-screen height estimate asks this instead of assuming one row.
    /// A collapsed phone echo paints [`COLLAPSED_NARROW_MAX_LINES`]; a wider pane paints [`COLLAPSED_MAX_LINES`].
    fn collapsed_row_budget(&self, content_width: u16) -> u16 {
        let rows = collapsed_max_lines(content_width, DisplayMode::Collapsed)
            .unwrap_or(COLLAPSED_MAX_LINES);
        u16::try_from(rows).unwrap_or(u16::MAX)
    }

    fn default_display_mode(&self) -> DisplayMode {
        if self.is_foldable() {
            DisplayMode::Collapsed
        } else {
            DisplayMode::Expanded
        }
    }

    fn next_fold_mode(&self, current: DisplayMode, _is_running: bool) -> DisplayMode {
        match current {
            DisplayMode::Collapsed | DisplayMode::Truncated => DisplayMode::Expanded,
            DisplayMode::Expanded => DisplayMode::Collapsed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Concatenated text content of a line (styles excluded)
    fn line_text(line: &Line) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    fn line_at(lines: &[BlockLine], i: usize) -> &BlockLine {
        lines
            .get(i)
            .unwrap_or_else(|| panic!("expected line {i}, len={}", lines.len()))
    }

    fn span_at<'s, 'c>(spans: &'s [Span<'c>], i: usize) -> &'s Span<'c> {
        spans
            .get(i)
            .unwrap_or_else(|| panic!("expected span {i}, len={}", spans.len()))
    }

    #[test]
    fn test_short_prompt_no_truncation() {
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::new("hello");
        let lines = block.wrap_prompt_lines(80, None, true, false);
        let expected = format!("{}hello", crate::glyphs::prompt_arrow());

        assert_eq!(lines.len(), 1);
        assert_eq!(line_text(&line_at(&lines, 0).content), expected);
    }

    #[test]
    fn test_short_prompt_with_max_lines() {
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::new("hello");
        let lines = block.wrap_prompt_lines(80, Some(2), true, false);
        let expected = format!("{}hello", crate::glyphs::prompt_arrow());

        assert_eq!(lines.len(), 1);
        assert_eq!(line_text(&line_at(&lines, 0).content), expected);
        // No ellipsis because content fits
        assert!(!line_text(&line_at(&lines, 0).content).contains('\u{2026}'));
    }

    #[test]
    fn test_long_prompt_wraps() {
        let _guard = crate::theme::cache::pin_theme();
        // Roomy width, so the `❯ ` prefix is drawn and its two columns come out of the text's room.
        let block = UserPromptBlock::new(
            "this is a very long prompt that should wrap over several rows at this width",
        );
        let lines = block.wrap_prompt_lines(65, None, true, false);

        assert!(lines.len() > 1, "Should wrap to multiple lines");
        assert!(line_text(&line_at(&lines, 0).content).starts_with(crate::glyphs::prompt_arrow()));
        // Continuation lines have 2-space indent
        assert!(line_text(&line_at(&lines, 1).content).starts_with("  "));
    }

    /// The same prompt on a narrow pane still wraps, but the dropped `❯ ` means the text starts at column 0.
    #[test]
    fn test_long_prompt_wraps_without_the_arrow_on_a_narrow_pane() {
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::new("this is a very long prompt that should wrap");
        let lines = block.wrap_prompt_lines(20, None, true, false);

        assert!(lines.len() > 1, "Should wrap to multiple lines");
        assert!(
            !line_text(&line_at(&lines, 0).content).starts_with(crate::glyphs::prompt_arrow()),
            "a narrow pane drops the arrow: {:?}",
            line_text(&line_at(&lines, 0).content)
        );
        assert!(
            line_text(&line_at(&lines, 0).content).starts_with("this"),
            "the band starts at the text: {:?}",
            line_text(&line_at(&lines, 0).content)
        );
    }

    #[test]
    fn test_truncation_adds_ellipsis() {
        let _guard = crate::theme::cache::pin_theme();
        let block =
            UserPromptBlock::new("this is a very long prompt that should wrap to many lines");
        let lines = block.wrap_prompt_lines(20, Some(2), true, false);

        assert_eq!(lines.len(), 2);
        let last = line_text(&line_at(&lines, 1).content);
        assert!(
            last.ends_with(" \u{2026}"),
            "Last line should end with ellipsis: {:?}",
            last
        );
    }

    #[test]
    fn test_ellipsis_fits_within_width() {
        let _guard = crate::theme::cache::pin_theme();
        // 15 columns is narrow, so the arrow is dropped and the text gets the full width. The extra word keeps the
        // prompt past the two-row budget so the last visible row is still truncated.
        let block = UserPromptBlock::new("aaaa bbbb cccc dddd eeee ffff gggg");
        let width = 15; // Narrow width to force wrapping
        let lines = block.wrap_prompt_lines(width, Some(2), true, false);

        assert_eq!(lines.len(), 2);

        // Each line (text and ellipsis) should fit within width
        for line in &lines {
            let text = line_text(&line.content);
            let char_count = text.chars().count();
            assert!(
                char_count <= width as usize,
                "Line exceeds width {}: {:?} ({})",
                width,
                text,
                char_count
            );
        }

        assert!(line_text(&line_at(&lines, 1).content).ends_with(" \u{2026}"));
    }

    #[test]
    fn test_bash_prompt_prefix() {
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::bash("ls -la");
        let lines = block.wrap_prompt_lines(80, None, true, false);

        assert_eq!(lines.len(), 1);
        assert!(line_text(&line_at(&lines, 0).content).starts_with("$ "));
    }

    /// Terminal theme (fullscreen): prompts render bandless — bold primary
    /// text instead of a bright-black band — including when selected (the
    /// rewind picker and dimmed tail carry the selection cue).
    #[test]
    fn terminal_theme_prompt_is_bold_and_bandless() {
        let _guard = crate::theme::cache::pin_theme();
        crate::theme::cache::set(crate::theme::ThemeKind::Terminal);

        let block = UserPromptBlock::new("hello");
        let lines = block.wrap_prompt_lines(80, None, true, false);
        assert!(line_at(&lines, 0).background.is_none(), "no band");
        let text_span = line_at(&lines, 0).content.spans.last().unwrap();
        assert!(
            text_span.style.add_modifier.contains(Modifier::BOLD),
            "prompt text is bold, got {:?}",
            text_span.style
        );
        assert!(!text_span.style.add_modifier.contains(Modifier::REVERSED));

        let selected = block.wrap_prompt_lines(80, None, true, true);
        assert!(
            line_at(&selected, 0).background.is_none(),
            "selected: still no band"
        );
        assert!(
            line_at(&selected, 0)
                .content
                .spans
                .iter()
                .all(|s| !s.style.add_modifier.contains(Modifier::REVERSED)),
            "selected prompt carries no reverse video (picker + dim tail cue it)"
        );
    }

    #[test]
    fn skill_with_args_only_command_is_teal() {
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::skill("/pr-workflow create a ticket for this");
        let lines = block.wrap_prompt_lines(80, None, true, false);
        assert_eq!(lines.len(), 1);

        let theme = Theme::current();
        let spans = &line_at(&lines, 0).content.spans;
        assert_eq!(spans.len(), 3);
        assert_eq!(span_at(spans, 1).content.as_ref(), "/pr-workflow");
        assert_eq!(span_at(spans, 1).style.fg, Some(theme.accent_skill));
        assert_eq!(
            span_at(spans, 2).content.as_ref(),
            " create a ticket for this"
        );
        assert_eq!(span_at(spans, 2).style.fg, Some(theme.text_primary));
    }

    #[test]
    fn skill_without_args_all_teal() {
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::skill("/pr-workflow");
        let lines = block.wrap_prompt_lines(80, None, true, false);
        assert_eq!(lines.len(), 1);

        let theme = Theme::current();
        let spans = &line_at(&lines, 0).content.spans;
        assert_eq!(spans.len(), 2);
        assert_eq!(span_at(spans, 1).content.as_ref(), "/pr-workflow");
        assert_eq!(span_at(spans, 1).style.fg, Some(theme.accent_skill));
    }

    #[test]
    fn skill_multiline_only_first_token_teal() {
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::skill("/foo bar\nbaz");
        let lines = block.wrap_prompt_lines(80, None, true, false);
        assert_eq!(lines.len(), 2);

        let theme = Theme::current();
        let line0 = &line_at(&lines, 0).content.spans;
        assert_eq!(span_at(line0, 1).content.as_ref(), "/foo");
        assert_eq!(span_at(line0, 1).style.fg, Some(theme.accent_skill));
        assert_eq!(span_at(line0, 2).content.as_ref(), " bar");
        assert_eq!(span_at(line0, 2).style.fg, Some(theme.text_primary));

        let line1 = &line_at(&lines, 1).content.spans;
        assert_eq!(span_at(line1, 1).content.as_ref(), "baz");
        assert_eq!(span_at(line1, 1).style.fg, Some(theme.text_primary));
    }

    #[test]
    fn mid_text_token_only_token_is_teal() {
        let _guard = crate::theme::cache::pin_theme();
        let text = "great /pr-workflow all good now";
        let block = UserPromptBlock::with_skill_tokens(text, vec![6..18]);
        let lines = block.wrap_prompt_lines(80, None, true, false);
        assert_eq!(lines.len(), 1);

        let theme = Theme::current();
        let spans = &line_at(&lines, 0).content.spans;
        assert_eq!(spans.len(), 4);
        assert_eq!(span_at(spans, 1).content.as_ref(), "great ");
        assert_eq!(span_at(spans, 1).style.fg, Some(theme.text_primary));
        assert_eq!(span_at(spans, 2).content.as_ref(), "/pr-workflow");
        assert_eq!(span_at(spans, 2).style.fg, Some(theme.accent_skill));
        assert_eq!(span_at(spans, 3).content.as_ref(), " all good now");
        assert_eq!(span_at(spans, 3).style.fg, Some(theme.text_primary));
    }

    #[test]
    fn mid_text_multiple_tokens_each_teal() {
        let _guard = crate::theme::cache::pin_theme();
        let text = "run /commit then /review please";
        let block = UserPromptBlock::with_skill_tokens(text, vec![4..11, 17..24]);
        let lines = block.wrap_prompt_lines(80, None, true, false);
        assert_eq!(lines.len(), 1);

        let theme = Theme::current();
        let teal: Vec<&str> = line_at(&lines, 0)
            .content
            .spans
            .iter()
            .filter(|s| s.style.fg == Some(theme.accent_skill))
            .map(|s| s.content.as_ref())
            .collect();
        assert_eq!(teal, vec!["/commit", "/review"]);
    }

    #[test]
    fn mid_text_token_on_second_logical_line() {
        let _guard = crate::theme::cache::pin_theme();
        let text = "first line\nthen /model here";
        // "/model" starts after "first line\nthen " = 16 bytes.
        let block = UserPromptBlock::with_skill_tokens(text, vec![16..22]);
        let lines = block.wrap_prompt_lines(80, None, true, false);
        assert_eq!(lines.len(), 2);

        let theme = Theme::current();
        let line0 = &line_at(&lines, 0).content.spans;
        assert!(
            line0.iter().all(|s| s.style.fg != Some(theme.accent_skill)),
            "line 0 has no token"
        );
        let line1 = &line_at(&lines, 1).content.spans;
        assert_eq!(span_at(line1, 1).content.as_ref(), "then ");
        assert_eq!(span_at(line1, 1).style.fg, Some(theme.text_primary));
        assert_eq!(span_at(line1, 2).content.as_ref(), "/model");
        assert_eq!(span_at(line1, 2).style.fg, Some(theme.accent_skill));
        assert_eq!(span_at(line1, 3).content.as_ref(), " here");
    }

    #[test]
    fn invalid_token_ranges_are_dropped() {
        let _guard = crate::theme::cache::pin_theme();
        let text = "héllo /model now"; // 'é' is 2 bytes: "/model" = 7..13
        let block = UserPromptBlock::with_skill_tokens(
            text,
            vec![
                2..3,   // not a char boundary (inside 'é')
                40..50, // out of bounds
                9..9,   // empty
                7..13,  // valid token
                10..15, // overlaps the kept 7..13
            ],
        );
        assert_eq!(block.skill_token_ranges, vec![7..13]);

        let lines = block.wrap_prompt_lines(80, None, true, false);
        let theme = Theme::current();
        let teal: Vec<&str> = line_at(&lines, 0)
            .content
            .spans
            .iter()
            .filter(|s| s.style.fg == Some(theme.accent_skill))
            .map(|s| s.content.as_ref())
            .collect();
        assert_eq!(teal, vec!["/model"]);
    }

    #[test]
    fn all_token_ranges_invalid_renders_plain() {
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::with_skill_tokens("plain text", vec![100..200]);
        assert!(block.skill_token_ranges.is_empty());
        let lines = block.wrap_prompt_lines(80, None, true, false);
        let theme = Theme::current();
        assert_eq!(
            span_at(&line_at(&lines, 0).content.spans, 1).style.fg,
            Some(theme.text_primary)
        );
    }

    /// Concatenated content of a line's skill-accent spans.
    fn teal_text(line: &Line, theme: &Theme) -> String {
        line.spans
            .iter()
            .filter(|s| s.style.fg == Some(theme.accent_skill))
            .map(|s| s.content.as_ref())
            .collect()
    }

    #[test]
    fn collapsed_truncation_keeps_teal_on_straddling_token() {
        let _guard = crate::theme::cache::pin_theme();
        // "/pr-workflow" (bytes 8..20) is wider than the content width, so it straddles the last visible row and the hidden continuation
        // The truncating re-wrap must keep the visible head teal
        let text = "one\ntwo\n/pr-workflow tail";
        let block = UserPromptBlock::with_skill_tokens(text, vec![8..20]);
        let lines = block.wrap_prompt_lines(8, Some(3), false, false);
        assert_eq!(lines.len(), 3);

        let theme = Theme::current();
        let last = &line_at(&lines, 2).content;
        assert!(line_text(last).ends_with(" \u{2026}"));
        let teal = teal_text(last, &theme);
        assert!(
            !teal.is_empty() && "/pr-workflow".starts_with(&teal),
            "visible head of the straddling token must stay teal, got {teal:?}"
        );
    }

    #[test]
    fn collapsed_truncation_keeps_teal_on_token_within_last_line() {
        let _guard = crate::theme::cache::pin_theme();
        // "/do-it" (bytes 8..14) fits fully on the truncated last line even at the ellipsis-reduced width, so it must survive whole and teal
        let text = "one\ntwo\n/do-it more words here";
        let block = UserPromptBlock::with_skill_tokens(text, vec![8..14]);
        let lines = block.wrap_prompt_lines(20, Some(3), false, false);
        assert_eq!(lines.len(), 3);

        let theme = Theme::current();
        let last = &line_at(&lines, 2).content;
        assert!(line_text(last).ends_with(" \u{2026}"));
        assert_eq!(teal_text(last, &theme), "/do-it");
        let body: String = last
            .spans
            .iter()
            .filter(|s| s.style.fg == Some(theme.text_primary))
            .map(|s| s.content.as_ref())
            .collect();
        assert!(body.contains("more"), "args stay body-styled, got {body:?}");
    }

    #[test]
    fn narrow_wrap_keeps_teal_on_both_rows_of_split_token() {
        let _guard = crate::theme::cache::pin_theme();
        // Expanded (no max_lines): the 12-wide token cannot fit at width 8, so the wrapper splits it mid-token; every piece must stay teal
        let text = "aa /pr-workflow zz";
        let block = UserPromptBlock::with_skill_tokens(text, vec![3..15]);
        let lines = block.wrap_prompt_lines(8, None, false, false);
        assert!(lines.len() >= 2);

        let theme = Theme::current();
        let teal_by_line: Vec<String> = lines
            .iter()
            .map(|l| teal_text(&l.content, &theme))
            .collect();
        let lines_with_teal = teal_by_line.iter().filter(|t| !t.is_empty()).count();
        assert!(
            lines_with_teal >= 2,
            "split token must stay teal on every row: {teal_by_line:?}"
        );
        assert_eq!(teal_by_line.concat(), "/pr-workflow");
    }

    #[test]
    fn test_multiline_input() {
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::new("line one\nline two\nline three");
        let lines = block.wrap_prompt_lines(80, None, true, false);

        assert_eq!(lines.len(), 3);
        assert!(line_text(&line_at(&lines, 0).content).starts_with(crate::glyphs::prompt_arrow()));
        assert!(line_text(&line_at(&lines, 1).content).starts_with("  ")); // continuation indent
        assert!(line_text(&line_at(&lines, 2).content).starts_with("  "));
    }

    #[test]
    fn test_multiline_truncated() {
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::new("line one\nline two\nline three");
        let lines = block.wrap_prompt_lines(80, Some(2), true, false);

        assert_eq!(lines.len(), 2);
        // Last line should have ellipsis since there's more content
        assert!(line_text(&line_at(&lines, 1).content).ends_with(" \u{2026}"));
    }

    #[test]
    fn test_exact_fit_no_ellipsis() {
        let _guard = crate::theme::cache::pin_theme();
        // If content fits exactly in max_lines, no ellipsis needed
        let block = UserPromptBlock::new("short");
        let lines = block.wrap_prompt_lines(80, Some(1), true, false);

        assert_eq!(lines.len(), 1);
        assert!(!line_text(&line_at(&lines, 0).content).contains('\u{2026}'));
    }

    #[test]
    fn test_selected_prompt_uses_accent_color() {
        // Pinned: asserts non-bold prompts, which the ambient terminal
        // theme (bold fullscreen prompts) legitimately fails.
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::new("hello");
        let lines = block.wrap_prompt_lines(80, None, true, true);
        let expected = format!("{}hello", crate::glyphs::prompt_arrow());

        assert_eq!(lines.len(), 1);
        assert_eq!(line_text(&line_at(&lines, 0).content), expected);

        // Prefix always uses accent, never dim gray
        // A Reset accent passes through so it matches the composer's marker (Cyan is minimal-only, where prompt rows have no band)
        // Bold is minimal-only
        let theme = Theme::current();
        let prefix_span = span_at(&line_at(&lines, 0).content.spans, 0);
        let expected_fg = Some(theme.accent_user);
        assert_eq!(prefix_span.style.fg, expected_fg);
        assert!(!prefix_span.style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_unselected_prompt_still_uses_accent_pointer() {
        // Pinned: see test_selected_prompt_uses_accent_color.
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::new("hello");
        let lines = block.wrap_prompt_lines(80, None, true, false);

        assert_eq!(lines.len(), 1);

        // Unselected no longer collapses onto gray_dim: the same accent pointer keeps user turns scannable in a long transcript
        let theme = Theme::current();
        let prefix_span = span_at(&line_at(&lines, 0).content.spans, 0);
        let expected_fg = Some(theme.accent_user);
        assert_eq!(prefix_span.style.fg, expected_fg);
        // Fullscreen (default test env): accent pointer, not bold.
        assert!(!prefix_span.style.add_modifier.contains(Modifier::BOLD));
        // Not the old unselected path (dim gray), unless the whole palette is Reset (NO_COLOR / native grays), in which case Cyan still wins above
        if !matches!(theme.gray_dim, ratatui::style::Color::Reset) {
            assert_ne!(prefix_span.style.fg, Some(theme.gray_dim));
        }
    }

    #[test]
    fn test_prompt_lines_have_selection_range() {
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::new("hello");
        let lines = block.wrap_prompt_lines(80, None, true, false);
        assert!(
            lines
                .iter()
                .all(|l| l.selection_range == Some(USER_PROMPT_BODY_RANGE))
        );
    }

    #[test]
    fn test_prompt_prefix_excluded_from_selection() {
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::new("hello");
        let lines = block.wrap_prompt_lines(80, None, true, false);
        assert_eq!(lines.len(), 1);
        // Prefix is span 0, content starts at span 1
        match &line_at(&lines, 0).selectable {
            Selectable::Spans(range) => {
                assert_eq!(range.start, 1);
            }
            _ => panic!("Expected Selectable::Spans"),
        }
    }

    #[test]
    fn test_prompt_no_prefix_all_selectable() {
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::new("hello");
        let lines = block.wrap_prompt_lines(80, None, false, false);
        assert_eq!(lines.len(), 1);
        assert!(matches!(line_at(&lines, 0).selectable, Selectable::All));
    }

    #[test]
    fn test_prompt_wrapped_lines_have_joiners() {
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::new("this is a long prompt that should wrap");
        let lines = block.wrap_prompt_lines(15, None, true, false);
        assert!(lines.len() > 1);
        assert!(line_at(&lines, 0).joiner.is_none());
        assert!(lines.iter().skip(1).any(|l| l.joiner.is_some()));
    }

    #[test]
    fn test_prompt_multiline_joiners() {
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::new("line one\nline two");
        let lines = block.wrap_prompt_lines(80, None, true, false);
        assert_eq!(lines.len(), 2);
        assert!(line_at(&lines, 0).joiner.is_none());
        // No joiner on the second line either (hard break between logical lines)
        assert!(line_at(&lines, 1).joiner.is_none());
    }

    #[test]
    fn test_cron_prompt_prefix() {
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::cron("/pr-babysit check");
        let lines = block.wrap_prompt_lines(80, None, true, false);
        assert_eq!(lines.len(), 1);
        let text = line_text(&line_at(&lines, 0).content);
        assert!(
            text.starts_with("\u{21BB}  "),
            "Cron prompt should start with \u{21BB}, got: {text:?}"
        );
        assert!(text.contains("/pr-babysit check"));
    }

    #[test]
    fn test_bash_prefix_excluded_from_selection() {
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::bash("ls -la");
        let lines = block.wrap_prompt_lines(80, None, true, false);
        assert_eq!(lines.len(), 1);
        match &line_at(&lines, 0).selectable {
            Selectable::Spans(range) => {
                assert_eq!(range.start, 1);
            }
            _ => panic!("Expected Selectable::Spans"),
        }
    }

    #[test]
    fn test_continuation_indent_excluded_from_selection() {
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::new("line one\nline two");
        let lines = block.wrap_prompt_lines(80, None, true, false);
        assert_eq!(lines.len(), 2);
        // Both lines should exclude their prefix/indent
        for line in &lines {
            match &line.selectable {
                Selectable::Spans(range) => {
                    assert_eq!(range.start, 1);
                }
                _ => panic!("Expected Selectable::Spans"),
            }
        }
    }

    #[test]
    fn test_short_prompt_not_foldable() {
        let block = UserPromptBlock::new("hello");
        assert!(!block.is_foldable());
        assert_eq!(block.default_display_mode(), DisplayMode::Expanded);
    }

    #[test]
    fn test_three_line_prompt_not_foldable() {
        let block = UserPromptBlock::new("one\ntwo\nthree");
        assert!(!block.is_foldable());
        assert_eq!(block.default_display_mode(), DisplayMode::Expanded);
    }

    #[test]
    fn test_four_line_prompt_is_foldable() {
        let block = UserPromptBlock::new("one\ntwo\nthree\nfour");
        assert!(block.is_foldable());
        assert_eq!(block.default_display_mode(), DisplayMode::Collapsed);
    }

    #[test]
    fn test_long_single_line_is_foldable() {
        // A single line long enough to wrap past 3 visual lines at 60-char width
        let long_line = "a ".repeat(120); // 240 chars wraps to 4 visual lines at 60
        let block = UserPromptBlock::new(long_line);
        assert!(block.is_foldable());
        assert_eq!(block.default_display_mode(), DisplayMode::Collapsed);
    }

    #[test]
    fn test_short_single_line_not_foldable() {
        // A single line that fits in 3 visual lines at 60-char width
        let short_line = "a ".repeat(60); // 120 chars wraps to 2 visual lines at 60
        let block = UserPromptBlock::new(short_line);
        assert!(!block.is_foldable());
    }

    #[test]
    fn test_fold_toggle_collapsed_to_expanded() {
        let block = UserPromptBlock::new("one\ntwo\nthree\nfour");
        assert_eq!(
            block.next_fold_mode(DisplayMode::Collapsed, false),
            DisplayMode::Expanded
        );
    }

    #[test]
    fn test_fold_toggle_expanded_to_collapsed() {
        let block = UserPromptBlock::new("one\ntwo\nthree\nfour");
        assert_eq!(
            block.next_fold_mode(DisplayMode::Expanded, false),
            DisplayMode::Collapsed
        );
    }

    #[test]
    fn test_fold_toggle_truncated_to_expanded() {
        let block = UserPromptBlock::new("one\ntwo\nthree\nfour");
        assert_eq!(
            block.next_fold_mode(DisplayMode::Truncated, false),
            DisplayMode::Expanded
        );
    }

    #[test]
    fn user_prompt_bold_only_in_minimal() {
        // Pinned: on the terminal theme fullscreen prompts are bold too
        // (covered by terminal_theme_prompt_is_bold_and_bandless).
        let _guard = crate::theme::cache::pin_theme();
        let theme = Theme::current();
        let (prefix, body, skill) = UserPromptBlock::prompt_styles(&theme, true);
        assert!(prefix.add_modifier.contains(Modifier::BOLD));
        assert!(body.add_modifier.contains(Modifier::BOLD));
        assert!(skill.add_modifier.contains(Modifier::BOLD));

        let (prefix, body, skill) = UserPromptBlock::prompt_styles(&theme, false);
        assert!(!prefix.add_modifier.contains(Modifier::BOLD));
        assert!(!body.add_modifier.contains(Modifier::BOLD));
        assert!(!skill.add_modifier.contains(Modifier::BOLD));

        // Default unit-test env is fullscreen (lock off).
        let block = UserPromptBlock::new("hello");
        let lines = block.wrap_prompt_lines(80, None, true, false);
        let spans = &line_at(&lines, 0).content.spans;
        assert!(
            !span_at(spans, 0)
                .style
                .add_modifier
                .contains(Modifier::BOLD)
        );
        assert!(
            !span_at(spans, 1)
                .style
                .add_modifier
                .contains(Modifier::BOLD)
        );
    }

    /// Pure band logic (no global `terminal_native_lock`; that races other tests that call `Theme::current()` without the theme test mutex).
    #[test]
    fn prompt_band_color_native_vs_rgb() {
        let theme = Theme::groknight();
        assert_eq!(
            UserPromptBlock::prompt_band_color_for(&theme, false, true),
            None
        );
        assert_eq!(
            UserPromptBlock::prompt_band_color_for(&theme, true, true),
            None
        );
        // RGB theme: band follows bg_light.
        assert_eq!(
            UserPromptBlock::prompt_band_color_for(&theme, false, false),
            Some(theme.bg_light)
        );
        // Terminal-native palette: bg_light is Reset, so there is no RGB band outside native mode; native mode still uses the ANSI elevated slots
        let native = Theme::terminal_default();
        assert_eq!(
            UserPromptBlock::prompt_band_color_for(&native, false, false),
            None
        );
        assert_eq!(
            UserPromptBlock::prompt_band_color_for(&native, false, true),
            None
        );
    }

    /// Applied band is semantic (not a panel). Minimal leaves it unset.
    /// Does not toggle the process-global native lock.
    #[test]
    fn user_prompt_band_is_semantic_not_panel() {
        let _guard = crate::theme::cache::pin_theme();
        let block = UserPromptBlock::new("scan me");
        let lines = block.wrap_prompt_lines(80, None, true, false);
        assert!(
            !line_at(&lines, 0).background_is_panel,
            "band must be semantic so flat_background keeps it"
        );
        // Whatever Theme::current() is this moment, wrap used the same source for band selection (same process, no lock toggle in this test)
        let theme = Theme::current();
        assert_eq!(
            line_at(&lines, 0).background,
            UserPromptBlock::prompt_band_color_for(
                &theme,
                false,
                crate::theme::cache::terminal_native_locked(),
            )
        );
    }

    // ── Narrow-terminal collapse budget ─────────────────────────────

    /// Build a collapsed `BlockContext` at `width`.
    fn collapsed_ctx(width: u16) -> BlockContext {
        BlockContext {
            mode: DisplayMode::Collapsed,
            is_running: false,
            width,
            raw: false,
            max_lines: None,
            appearance: AppearanceConfig::default(),
            is_selected: false,
            cwd: None,
        }
    }

    /// Text of each rendered line, styles dropped.
    fn rendered_lines(block: &UserPromptBlock, ctx: &BlockContext) -> Vec<String> {
        block
            .output(ctx)
            .lines
            .iter()
            .map(|l| {
                l.content
                    .spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect()
    }

    /// A prompt long enough to exceed both collapse budgets.
    const LONG_PROMPT: &str = "This is a long prompt that should be collapsed and truncated at \
                                narrow widths because it has plenty of words to wrap over many \
                                lines indeed.";

    /// The measured iPhone pane is 55 columns (`stty -f /dev/<tty> size` on the
    /// Orca-managed dsb panes; see the layout session's measurement record).
    /// There, a collapsed prompt shows two lines and the ellipsis on the second,
    /// which keeps the turn readable without eating the viewport.
    #[test]
    fn collapsed_prompt_folds_to_two_lines_at_phone_width() {
        for width in [40u16, 50, 53, 55, 60] {
            let block = UserPromptBlock::new(LONG_PROMPT);
            let lines = rendered_lines(&block, &collapsed_ctx(width));
            assert_eq!(
                lines.len(),
                COLLAPSED_NARROW_MAX_LINES,
                "at {width} cols a collapsed prompt must be two lines, got {lines:?}"
            );
            assert!(
                !lines[0].ends_with(" \u{2026}"),
                "the ellipsis belongs on the last visible line at {width} cols: {lines:?}"
            );
            assert!(
                lines[1].ends_with(" \u{2026}"),
                "the second line must carry the ellipsis at {width} cols: {lines:?}"
            );
        }
    }

    /// The other half of the contract: widths above the threshold keep the
    /// three-line budget, so the desktop layout is unchanged.
    #[test]
    fn collapsed_prompt_keeps_three_line_budget_above_threshold() {
        let block = UserPromptBlock::new(LONG_PROMPT);
        let lines = rendered_lines(&block, &collapsed_ctx(COLLAPSED_NARROW_TERMINAL_COLS + 1));
        assert_eq!(
            lines.len(),
            COLLAPSED_MAX_LINES,
            "one column past the threshold must keep the roomy budget: {lines:?}"
        );
    }

    /// A 120-column pane is a desktop width. A prompt longer than three rows there
    /// still stops at three, with the ellipsis on the last of them.
    #[test]
    fn collapsed_prompt_keeps_three_lines_at_desktop_width() {
        let block = UserPromptBlock::new(LONG_PROMPT.repeat(8));
        let lines = rendered_lines(&block, &collapsed_ctx(120));
        assert_eq!(
            lines.len(),
            COLLAPSED_MAX_LINES,
            "a 120-column pane keeps the three-line budget: {lines:?}"
        );
        assert!(
            lines[2].ends_with(" \u{2026}"),
            "the third line must carry the ellipsis: {lines:?}"
        );
    }

    /// The threshold itself is the boundary: `<=` is narrow, `>` is roomy.
    #[test]
    fn collapsed_budget_switches_at_the_threshold() {
        assert_eq!(
            collapsed_max_lines(COLLAPSED_NARROW_TERMINAL_COLS, DisplayMode::Collapsed),
            Some(COLLAPSED_NARROW_MAX_LINES),
        );
        assert_eq!(
            collapsed_max_lines(COLLAPSED_NARROW_TERMINAL_COLS + 1, DisplayMode::Collapsed),
            Some(COLLAPSED_MAX_LINES),
        );
    }

    /// Expanding is not affected by width: a user who asks for the full prompt
    /// gets it at any pane size.
    #[test]
    fn expanded_prompt_is_unbounded_at_phone_width() {
        assert_eq!(
            collapsed_max_lines(40, DisplayMode::Expanded),
            None,
            "Expanded must never fold, however narrow the pane"
        );
        let block = UserPromptBlock::new(LONG_PROMPT);
        let mut ctx = collapsed_ctx(40);
        ctx.mode = DisplayMode::Expanded;
        assert!(
            rendered_lines(&block, &ctx).len() > COLLAPSED_NARROW_MAX_LINES,
            "the expanded prompt must render past the narrow budget"
        );
    }

    // ── Width-derived fold, pad, and the decorative arrow ──────────

    /// The measured iPhone pane is 55 columns (`stty -f /dev/<tty> size` on the Orca-managed dsb panes — the same
    /// measurement PR #199 recorded). With the default chrome (accent + 2+2 pads) and the 10-column timestamp gutter,
    /// the prompt's text wraps at 55 - 5 - 10 = 40 columns. 34 stands for that narrow band with a little margin, so
    /// these tests stay valid if the pad widths move by a column.
    ///
    /// This is the reported case: a ~100-column one-liner that the width-blind `is_foldable()` scores as two visual
    /// rows (under the three-row roomy budget), so the block called itself unfolded and the narrow budget never
    /// applied. At phone width the same line needs a third row, which is past the two-row budget.
    const PHONE_CONTENT_WIDTH: u16 = 34;

    fn hundred_column_prompt() -> UserPromptBlock {
        UserPromptBlock::new("x".repeat(100))
    }

    #[test]
    fn narrow_single_line_prompt_is_foldable_at_phone_width() {
        let block = hundred_column_prompt();
        assert!(
            block.is_foldable_at(PHONE_CONTENT_WIDTH),
            "a ~100-column prompt needs more than two rows at phone width"
        );
        assert!(
            !block.is_foldable(),
            "the width-blind estimate divides by 60 and must still read this as two rows"
        );
    }

    #[test]
    fn prompt_vpad_drops_at_phone_width() {
        let block = UserPromptBlock::new("hello");
        let appearance = AppearanceConfig::default();
        assert!(
            appearance.scrollback.blocks.prompt.vpad,
            "the default config must ask for vpad, or this test proves nothing"
        );
        assert!(
            !block.has_vpad_for_width(&appearance, PHONE_CONTENT_WIDTH),
            "a phone-width pane drops the prompt's blank pad rows"
        );
    }

    #[test]
    fn prompt_vpad_keeps_at_desktop_width() {
        let block = UserPromptBlock::new("hello");
        let appearance = AppearanceConfig::default();
        assert!(
            block.has_vpad_for_width(&appearance, 80),
            "desktop width keeps the configured prompt pad"
        );
    }

    #[test]
    fn wide_prompt_folds_at_phone_width_but_not_at_desktop_width() {
        let block = UserPromptBlock::new("x".repeat(200));
        assert!(
            !block.is_foldable_at(80),
            "200 columns fit inside the three-row desktop budget"
        );
        assert!(
            block.is_foldable_at(PHONE_CONTENT_WIDTH),
            "the same 200 columns exceed the two-row phone budget"
        );
    }

    #[test]
    fn short_prompt_is_unfoldable_at_both_widths() {
        let block = UserPromptBlock::new("hello");
        assert!(!block.is_foldable_at(PHONE_CONTENT_WIDTH));
        assert!(!block.is_foldable_at(80));
    }

    #[test]
    fn exactly_two_rows_fit_the_narrow_budget_and_a_third_folds() {
        let band = usize::from(PHONE_CONTENT_WIDTH);
        let two = UserPromptBlock::new(format!("{}\n{}", "y".repeat(band), "z".repeat(band)));
        assert!(
            !two.is_foldable_at(PHONE_CONTENT_WIDTH),
            "two rows that each fit the band stay within the narrow budget"
        );
        assert!(
            !two.is_foldable_at(80),
            "the same two rows fit the three-row desktop budget"
        );

        let three = UserPromptBlock::new(format!(
            "{}\n{}\n{}",
            "y".repeat(band),
            "z".repeat(band),
            "w".repeat(band),
        ));
        assert!(
            three.is_foldable_at(PHONE_CONTENT_WIDTH),
            "a third row exceeds the two-row narrow budget"
        );
        assert!(
            !three.is_foldable_at(80),
            "three short rows still fit the three-row desktop budget"
        );

        let lines = rendered_lines(&three, &collapsed_ctx(PHONE_CONTENT_WIDTH));
        assert_eq!(lines.len(), COLLAPSED_NARROW_MAX_LINES, "{lines:?}");
        assert!(
            lines[1].ends_with(" \u{2026}"),
            "the folded third row leaves an ellipsis on the second line: {lines:?}"
        );
    }

    #[test]
    fn phone_width_echo_drops_the_arrow_prefix() {
        let block = UserPromptBlock::new("hello");
        assert!(
            AppearanceConfig::default()
                .scrollback
                .blocks
                .prompt
                .show_prefix,
            "the default config must ask for the arrow, or this test proves nothing"
        );

        let narrow = rendered_lines(&block, &collapsed_ctx(PHONE_CONTENT_WIDTH));
        assert_eq!(
            narrow,
            vec!["hello".to_string()],
            "a phone-width echo starts at the text, with no arrow column"
        );

        let desktop = rendered_lines(&block, &collapsed_ctx(80));
        assert_eq!(
            desktop,
            vec![format!("{}hello", crate::glyphs::prompt_arrow())],
            "desktop keeps the arrow it has always drawn"
        );
    }

    /// The dropped arrow's two columns go back to the text. The narrow budget is two rows of that band, so text
    /// exactly two bands wide fits, and one more column needs a third row and folds. Subtracting the dropped arrow
    /// would fold two columns early.
    #[test]
    fn phone_width_fold_budget_counts_the_reclaimed_arrow_columns() {
        let band = usize::from(PHONE_CONTENT_WIDTH);
        let one_row = UserPromptBlock::new("x".repeat(band));
        assert!(
            !one_row.is_foldable_at(PHONE_CONTENT_WIDTH),
            "text exactly as wide as the band fits its first row"
        );

        let two_rows = UserPromptBlock::new("x".repeat(band * 2));
        assert!(
            !two_rows.is_foldable_at(PHONE_CONTENT_WIDTH),
            "two rows exactly as wide as the band fit the narrow budget"
        );

        let over = UserPromptBlock::new("x".repeat(band * 2 + 1));
        assert!(
            over.is_foldable_at(PHONE_CONTENT_WIDTH),
            "one column past two rows needs a third row and must fold"
        );
    }

    #[test]
    fn narrow_band_keeps_meaning_bearing_prefixes() {
        let bash = UserPromptBlock::bash("ls");
        let lines = rendered_lines(&bash, &collapsed_ctx(PHONE_CONTENT_WIDTH));
        assert_eq!(lines, vec!["$ ls".to_string()], "bash prefix survives");

        let cron = UserPromptBlock::cron("wake up");
        let lines = rendered_lines(&cron, &collapsed_ctx(PHONE_CONTENT_WIDTH));
        assert_eq!(
            lines,
            vec!["\u{21BB}  wake up".to_string()],
            "cron prefix survives"
        );
    }

    /// The measured width is inside the threshold with margin, and the widest
    /// desktop pane observed on the same machine (80) is outside it. Pins the
    /// relationship the two constants were chosen for.
    #[test]
    fn measured_phone_width_is_inside_the_threshold_and_desktop_outside() {
        // The measured iPhone pane is 55 columns; the narrowest desktop pane
        // observed on the same machine is 80.
        const MEASURED_PHONE_COLS: u16 = 55;
        const NARROWEST_DESKTOP_COLS: u16 = 80;
        // `const` blocks keep this a compile-time check, which is what makes
        // the two constants' relationship part of the build rather than a
        // runtime assertion clippy reads as constant.
        const {
            assert!(MEASURED_PHONE_COLS <= COLLAPSED_NARROW_TERMINAL_COLS);
        }
        const {
            assert!(NARROWEST_DESKTOP_COLS > COLLAPSED_NARROW_TERMINAL_COLS);
        }
    }
}
