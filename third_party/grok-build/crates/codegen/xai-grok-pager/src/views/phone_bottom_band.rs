//! One row under the prompt on a phone-width pane.
//!
//! Cost and cache stay on the left, in full. Model and permission stay on the
//! right. The model is the only field that shrinks, and only after a leading
//! `DeepSeek ` prefix and a trailing reasoning-effort parenthesis are gone.
//! A 55-column pane has 53 columns inside the flush frame. `$10.77` +
//! `cache 94%` costs 16 columns and `V4.1 Flash (max) · always-approve` 33, so
//! the measured pair fits with a four-column gap. The widest money string,
//! `$1234.56` + `cache 100%`, costs 19; the model then drops its effort suffix
//! and `V4.1 Flash · always-approve` (27) leaves a seven-column gap.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Span;
use unicode_width::UnicodeWidthStr;

use crate::render::SafeBuf;
use crate::theme::Theme;
use crate::util::truncate_to_width;

/// Columns kept clear between the cost cluster and the model cluster.
const GAP: usize = 2;

/// Dropped on this row only. The balance chip is already the DeepSeek account,
/// and nine columns of a 53-column row are the model name.
const MODEL_PREFIX: &str = "DeepSeek ";

/// Trailing effort the desktop info line appends as ` (max)`. Dropped only
/// when keeping it would clip the permission mode.
const EFFORT_SUFFIXES: &[&str] = &[
    " (none)",
    " (minimal)",
    " (low)",
    " (medium)",
    " (high)",
    " (xhigh)",
    " (max)",
];

/// The phone row, split so the balance can stay brighter than the cache marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PhoneBottomBand {
    pub balance: Option<String>,
    pub cache: Option<String>,
    pub right: String,
}

impl PhoneBottomBand {
    pub(crate) fn left_width(&self) -> usize {
        left_width(self.balance.as_deref(), self.cache.as_deref())
    }
}

/// `balance` is the balance chip (`$10.77`). `cache_marker` is the same
/// `cache 94%` label the desktop info line shows. `flags` are the info-line flag
/// texts already filtered (Ask contributes none).
pub(crate) fn compose_phone_bottom_band(
    width: usize,
    balance: Option<&str>,
    cache_marker: Option<&str>,
    model: &str,
    flags: &[&str],
    usage_warning: Option<&str>,
    multiline: bool,
) -> PhoneBottomBand {
    let (balance, cache) = fit_left(width, balance, cache_marker);
    let right = right_cluster(
        width,
        left_width(balance.as_deref(), cache.as_deref()),
        model,
        flags,
        usage_warning,
        multiline,
    );
    PhoneBottomBand {
        balance,
        cache,
        right,
    }
}

/// Blank the row, then pin the cost cluster to the left and the model cluster
/// to the right. The composer guarantees they do not meet.
pub(crate) fn paint_phone_bottom_band(
    buf: &mut Buffer,
    area: Rect,
    band: &PhoneBottomBand,
    theme: &Theme,
) {
    if area.height == 0 || area.width == 0 {
        return;
    }
    let bg = theme.bg_base;
    let y = area.y;
    let blank = Style::default().bg(bg);
    for x in area.x..area.x.saturating_add(area.width) {
        if let Some(cell) = buf.cell_mut((x, y)) {
            cell.set_char(' ');
            cell.set_style(blank);
        }
    }
    let mut x = area.x;
    if let Some(balance) = &band.balance {
        let w = balance.width() as u16;
        let span = Span::styled(
            balance.clone(),
            Style::default().fg(theme.text_secondary).bg(bg),
        );
        buf.set_span_safe(x, y, &span, w);
        x = x.saturating_add(w);
    }
    if let Some(cache) = &band.cache {
        if band.balance.is_some() {
            let sep = Span::styled(" ", blank);
            buf.set_span_safe(x, y, &sep, 1);
            x = x.saturating_add(1);
        }
        let w = cache.width() as u16;
        let span = Span::styled(cache.clone(), Style::default().fg(theme.gray_dim).bg(bg));
        buf.set_span_safe(x, y, &span, w);
    }
    let right_w = band.right.width() as u16;
    if right_w == 0 {
        return;
    }
    let right_x = area.x.saturating_add(area.width.saturating_sub(right_w));
    let span = Span::styled(
        band.right.clone(),
        Style::default().fg(theme.gray_dim).bg(bg),
    );
    buf.set_span_safe(right_x, y, &span, right_w);
}

fn left_width(balance: Option<&str>, cache: Option<&str>) -> usize {
    match (balance, cache) {
        (None, None) => 0,
        (Some(b), None) => b.width(),
        (None, Some(c)) => c.width(),
        (Some(b), Some(c)) => b.width() + 1 + c.width(),
    }
}

fn nonempty(text: Option<&str>) -> Option<String> {
    text.filter(|s| !s.is_empty()).map(str::to_owned)
}

/// The balance survives a tight row; the cache marker goes before the balance
/// is cut. A 53-column row never reaches either cut for the balances this
/// product shows.
fn fit_left(
    width: usize,
    balance: Option<&str>,
    cache: Option<&str>,
) -> (Option<String>, Option<String>) {
    let balance = nonempty(balance);
    let cache = nonempty(cache);
    if left_width(balance.as_deref(), cache.as_deref()) <= width {
        return (balance, cache);
    }
    if left_width(balance.as_deref(), None) <= width {
        return (balance, None);
    }
    let Some(balance) = balance else {
        let cache = cache.map(|c| truncate_to_width(&c, width).into_owned());
        return (None, cache.filter(|s| !s.is_empty()));
    };
    let shown = truncate_to_width(&balance, width).into_owned();
    let balance = if shown.is_empty() { None } else { Some(shown) };
    (balance, None)
}

fn right_room(width: usize, left_w: usize) -> usize {
    if left_w == 0 {
        return width;
    }
    let gap = GAP.min(width.saturating_sub(left_w));
    if gap == 0 {
        return 0;
    }
    width - left_w - gap
}

fn strip_model_prefix(model: &str) -> &str {
    model.strip_prefix(MODEL_PREFIX).unwrap_or(model)
}

fn strip_effort(model: &str) -> &str {
    EFFORT_SUFFIXES
        .iter()
        .find_map(|suffix| model.strip_suffix(suffix))
        .unwrap_or(model)
}

fn assemble(warning: Option<&str>, model: &str, flags: &str, multiline: bool) -> Option<String> {
    let mut parts: Vec<&str> = Vec::new();
    if let Some(warning) = warning {
        parts.push(warning);
    }
    if !model.is_empty() {
        parts.push(model);
    }
    if !flags.is_empty() {
        parts.push(flags);
    }
    if multiline {
        parts.push("ml");
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" · "))
    }
}

fn right_cluster(
    width: usize,
    left_w: usize,
    model: &str,
    flags: &[&str],
    usage_warning: Option<&str>,
    multiline: bool,
) -> String {
    let room = right_room(width, left_w);
    if room == 0 {
        return String::new();
    }
    let model_full = strip_model_prefix(model);
    let model_short = strip_effort(model_full);
    let flags_body = flags.join(" · ");
    let warning = usage_warning.filter(|s| !s.is_empty());
    // Warning first, then without it. Effort stays until the row cannot hold
    // the mode beside it. Multiline is the first thing dropped inside a pass.
    let mut passes = Vec::with_capacity(2);
    if let Some(warning) = warning {
        passes.push(Some(warning));
    }
    passes.push(None);
    for warn in passes {
        for model_try in [model_full, model_short] {
            if multiline
                && let Some(line) = assemble(warn, model_try, &flags_body, true)
                && line.width() <= room
            {
                return line;
            }
            if let Some(line) = assemble(warn, model_try, &flags_body, false)
                && line.width() <= room
            {
                return line;
            }
        }
    }
    ellipsize_right(room, model_short, &flags_body)
}

/// Flags stay whole. The model takes the columns that remain, with an
/// ellipsis. Only a row too narrow for the mode itself cuts the mode.
fn ellipsize_right(room: usize, model: &str, flags_body: &str) -> String {
    let tail = if flags_body.is_empty() {
        String::new()
    } else {
        format!(" · {flags_body}")
    };
    if tail.width() <= room {
        let budget = room - tail.width();
        let shown = if budget == 0 {
            String::new()
        } else {
            truncate_to_width(model, budget).into_owned()
        };
        if shown.is_empty() {
            return if flags_body.width() <= room {
                flags_body.to_string()
            } else {
                truncate_to_width(flags_body, room).into_owned()
            };
        }
        return if tail.is_empty() {
            shown
        } else {
            format!("{shown}{tail}")
        };
    }
    if flags_body.width() <= room {
        flags_body.to_string()
    } else {
        truncate_to_width(flags_body, room).into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Inner width of the measured 55-column iPhone pane (one column each side).
    const PHONE_INNER: usize = 53;

    fn band(
        width: usize,
        balance: Option<&str>,
        cache: Option<&str>,
        model: &str,
        flags: &[&str],
    ) -> PhoneBottomBand {
        compose_phone_bottom_band(width, balance, cache, model, flags, None, false)
    }

    fn assert_separated(width: usize, band: &PhoneBottomBand) {
        let left = band.left_width();
        let right = band.right.width();
        assert!(
            left + right <= width,
            "clusters exceed {width}: left {left} right {right} {band:?}"
        );
        if left > 0 && right > 0 {
            let right_start = width - right;
            assert!(
                right_start >= left + GAP,
                "gap under {GAP} at {width}: left {left} right starts {right_start} {band:?}"
            );
        }
    }

    #[test]
    fn measured_pane_keeps_effort_and_the_full_mode() {
        let band = band(
            PHONE_INNER,
            Some("$10.77"),
            Some("cache 94%"),
            "DeepSeek V4.1 Flash (max)",
            &["always-approve"],
        );
        assert_eq!(band.balance.as_deref(), Some("$10.77"));
        assert_eq!(band.cache.as_deref(), Some("cache 94%"));
        assert_eq!(band.right, "V4.1 Flash (max) · always-approve");
        assert_separated(PHONE_INNER, &band);
    }

    #[test]
    fn worst_money_and_full_cache_still_keep_the_mode() {
        let band = band(
            PHONE_INNER,
            Some("$1234.56"),
            Some("cache 100%"),
            "DeepSeek V4.1 Flash (max)",
            &["always-approve"],
        );
        assert_eq!(band.balance.as_deref(), Some("$1234.56"));
        assert_eq!(band.cache.as_deref(), Some("cache 100%"));
        // The full label costs five columns more than the old `c100%`, so the
        // widest money string drops the effort suffix to keep the mode whole.
        assert_eq!(band.right, "V4.1 Flash · always-approve");
        assert!(!band.right.contains('…'));
        assert_separated(PHONE_INNER, &band);
    }

    #[test]
    fn a_long_model_ellipsizes_rather_than_clipping_the_mode() {
        let band = band(
            PHONE_INNER,
            Some("$1234.56"),
            Some("cache 100%"),
            "DeepSeek V4.1 Flash Thinking (max)",
            &["always-approve"],
        );
        assert_eq!(band.right, "V4.1 Flash Thi… · always-approve");
        assert!(band.right.ends_with(" · always-approve"));
        assert!(!band.right.contains("(max)"));
        assert_separated(PHONE_INNER, &band);
    }

    #[test]
    fn a_model_that_still_does_not_fit_ellipsizes_only_itself() {
        let model = format!("DeepSeek {} (max)", "M".repeat(80));
        let band = band(
            PHONE_INNER,
            Some("$1234.56"),
            Some("cache 100%"),
            &model,
            &["always-approve"],
        );
        assert_eq!(band.balance.as_deref(), Some("$1234.56"));
        assert_eq!(band.cache.as_deref(), Some("cache 100%"));
        assert!(band.right.ends_with(" · always-approve"), "{band:?}");
        assert!(band.right.contains('…'), "{band:?}");
        assert!(!band.right.contains("always-appro…"), "{band:?}");
        assert_separated(PHONE_INNER, &band);
    }

    #[test]
    fn other_permission_modes_stay_whole() {
        let auto = band(
            PHONE_INNER,
            Some("$10.77"),
            Some("cache 94%"),
            "DeepSeek V4.1 Flash (max)",
            &["auto"],
        );
        assert_eq!(auto.right, "V4.1 Flash (max) · auto");
        let ask = band(
            PHONE_INNER,
            Some("$10.77"),
            Some("cache 94%"),
            "DeepSeek V4.1 Flash (max)",
            &[],
        );
        assert_eq!(ask.right, "V4.1 Flash (max)");
        let plan = band(
            PHONE_INNER,
            Some("$1234.56"),
            Some("cache 100%"),
            "DeepSeek V4.1 Flash (max)",
            &["plan", "always-approve"],
        );
        assert_eq!(plan.right, "V4.1 Fl… · plan · always-approve");
        assert_separated(PHONE_INNER, &plan);
    }

    #[test]
    fn multiline_fits_the_open_row_and_yields_on_the_tight_one() {
        let open = compose_phone_bottom_band(
            PHONE_INNER,
            Some("$10.77"),
            Some("cache 94%"),
            "DeepSeek V4.1 Flash (max)",
            &[],
            None,
            true,
        );
        assert_eq!(open.right, "V4.1 Flash (max) · ml");
        let tight = compose_phone_bottom_band(
            PHONE_INNER,
            Some("$1234.56"),
            Some("cache 100%"),
            "DeepSeek V4.1 Flash (max)",
            &["always-approve"],
            None,
            true,
        );
        // The five columns the full label added come out of the effort here;
        // the `ml` suffix still fits beside the mode.
        assert_eq!(tight.right, "V4.1 Flash · always-approve · ml");
    }

    #[test]
    fn clusters_do_not_meet_at_any_width() {
        let model = format!("DeepSeek {} (max)", "M".repeat(40));
        for width in [0usize, 1, 8, 14, 20, 28, 40, 53, 58, 80] {
            for flags in [&[][..], &["auto"][..], &["always-approve"][..]] {
                let band = band(width, Some("$1234.56"), Some("cache 100%"), &model, flags);
                assert_separated(width, &band);
            }
        }
    }

    #[test]
    fn desktop_width_is_not_this_functions_job_but_it_does_not_clip() {
        let band = band(
            78,
            Some("$10.77"),
            Some("cache 94%"),
            "DeepSeek V4.1 Flash (max)",
            &["always-approve"],
        );
        assert_eq!(band.right, "V4.1 Flash (max) · always-approve");
        assert_separated(78, &band);
    }
}
