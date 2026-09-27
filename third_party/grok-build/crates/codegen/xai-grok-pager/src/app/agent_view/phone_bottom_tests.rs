//! Phone-width bottom stack (measured iPhone Orca pane: 55 columns).
//!
//! The prompt box keeps a plain bottom rule. Balance, cache, model and permission
//! share the single row under it: cost and cache on the left, model and permission
//! on the right. The shortcut-hint row stays absent. Desktop widths keep the label
//! on the divider and the balance on its own row.
//!
//! `views::prompt_widget`'s tests pin the widget in isolation; these frames pin the
//! whole-view stack (the hint row is a layout row owned by `render`).

use super::{AgentView, AppRenderParams, BannerSlotParams, test_fixtures};
use crate::actions::ActionRegistry;
use crate::app::agent::AgentState;
use crate::scrollback::RenderBlock;
use crate::scrollback::render::ScratchBuffer;
use crate::theme::Theme;
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
    agent_with(MODEL_LABEL, "15.87", 3604, 4096)
}

fn agent_with(model: &str, balance: &str, cached: u64, input: u64) -> AgentView {
    let mut agent = test_fixtures::make_agent();
    let id = acp::ModelId::new(Arc::from(MODEL_ID));
    agent.session.models.available.insert(
        id.clone(),
        acp::ModelInfo::new(id.clone(), model.to_string()),
    );
    agent.session.models.current = Some(id);
    agent.session.models.reasoning_effort = Some(ReasoningEffort::Max);
    agent.session.set_yolo_mode_for_test(true);
    agent.deepseek_status = Some(
        serde_json::from_value(serde_json::json!({
            "isDeepseek": true,
            "balance": {"currency": "USD", "totalBalance": balance, "isAvailable": true},
            "usage": {"inputTokens": input, "cachedReadTokens": cached},
        }))
        .expect("deepseek status fixture"),
    );
    agent.deepseek_status_session_id = agent.session.session_id.clone();
    agent
}

fn draw(agent: &mut AgentView, cols: u16, rows: u16) -> Buffer {
    agent.last_terminal_size = (cols, rows);
    // The app derives the pane's density flag from the terminal width
    // (`AppView::apply_effective_density`); the fixture sets the same value, so
    // a frame test draws the frame the pane draws.
    let mut appearance = agent.scrollback.appearance().clone();
    appearance.scrollback.layout.narrow = crate::views::agent::effective_narrow(cols);
    agent.scrollback.set_appearance(appearance);
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
/// The box fills the inner area, so its left edge is the reserved outer column
/// (`LayoutConfig::eff_hpad_left`).
fn border_row(buf: &Buffer) -> u16 {
    let left = crate::appearance::LayoutConfig::default().eff_hpad_left(false) as usize;
    (0..buf.area.height)
        .rev()
        .find(|&y| {
            row_text(buf, y)
                .chars()
                .nth(left)
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

/// The one row under the box. Cost cluster, then a gap, then the model cluster
/// flush to the inner right edge (the outer pad column stays blank).
fn assert_one_band(buf: &Buffer, balance: &str, cache: &str, model: &str, mode: Option<&str>) {
    let border_y = border_row(buf);
    let border = row_text(buf, border_y);
    assert!(
        !border.contains(model) && mode.is_none_or(|m| !border.contains(m)),
        "the divider row carries no label text: {border:?}"
    );
    let border_chars: Vec<char> = border.chars().collect();
    let left = crate::appearance::LayoutConfig::default().eff_hpad_left(false) as usize;
    assert!(
        border_chars[left + 1..border_chars.len() - left - 1]
            .iter()
            .all(|c| *c == '\u{2500}'),
        "the phone divider is a plain rule: {border:?}"
    );

    let band_y = border_y + 1;
    let band = row_text(buf, band_y);
    let balance_at = band
        .find(balance)
        .unwrap_or_else(|| panic!("balance {balance:?} missing: {band:?}"));
    let cache_at = band
        .find(cache)
        .unwrap_or_else(|| panic!("cache {cache:?} missing: {band:?}"));
    let model_at = band
        .find(model)
        .unwrap_or_else(|| panic!("model {model:?} missing: {band:?}"));
    assert!(balance_at < cache_at, "cost sits left of cache: {band:?}");
    assert!(
        cache_at >= balance_at + balance.len(),
        "cache does not overlap the balance: {band:?}"
    );
    let left_end = cache_at + cache.len();
    assert!(
        model_at >= left_end + 2,
        "two columns stay clear between the clusters: {band:?}"
    );
    if let Some(mode) = mode {
        let mode_at = band
            .find(mode)
            .unwrap_or_else(|| panic!("mode {mode:?} missing: {band:?}"));
        assert!(mode_at >= model_at, "the mode follows the model: {band:?}");
        assert!(
            band.trim_end().ends_with(mode),
            "the mode is the right edge of the row: {band:?}"
        );
    }
    assert_eq!(
        band_y + 1 + crate::views::agent::PHONE_BOTTOM_MARGIN_ROWS,
        buf.area.height,
        "the band reaches the measured PTY bottom, got {band_y} in {}",
        buf.area.height
    );
    assert!(
        !band.contains("DeepSeek "),
        "the phone row drops the product prefix: {band:?}"
    );
}

#[test]
fn phone_pane_puts_cost_and_model_on_one_row_and_drops_the_hint_row() {
    let mut agent = phone_agent();
    let registry = ActionRegistry::defaults();
    let buf = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
    eprintln!("phone 55x41 — bottom band:\n{}", bottom_rows(&buf, 6));

    assert_one_band(
        &buf,
        "$15.87",
        "cache 88%",
        "V4.1 Flash (max)",
        Some("always-approve"),
    );
    assert_no_hint_row(&agent, &registry, &buf);
}

#[test]
fn phone_pane_fits_a_large_balance_and_a_full_cache() {
    let mut agent = agent_with(MODEL_LABEL, "1234.56", 100, 100);
    let buf = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
    eprintln!(
        "phone 55x41 large balance — bottom band:\n{}",
        bottom_rows(&buf, 4)
    );
    assert_one_band(
        &buf,
        "$1234.56",
        "cache 100%",
        // The full cache label costs the five columns the old `c100%` saved,
        // so the widest money string drops the effort suffix; the mode holds.
        "V4.1 Flash",
        Some("always-approve"),
    );
}

#[test]
fn phone_pane_ellipsizes_the_long_model_after_effort_is_gone() {
    let mut agent = agent_with("DeepSeek V4.1 Flash Thinking", "1234.56", 100, 100);
    let buf = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
    eprintln!(
        "phone 55x41 long model — bottom band:\n{}",
        bottom_rows(&buf, 4)
    );
    let band_y = border_row(&buf) + 1;
    let band = row_text(&buf, band_y);
    assert_one_band(
        &buf,
        "$1234.56",
        "cache 100%",
        "V4.1 Flash Thi",
        Some("always-approve"),
    );
    assert!(!band.contains("(max)"), "effort yields: {band:?}");
    assert!(
        band.contains('…'),
        "the long model takes the ellipsis the label left it: {band:?}"
    );
}

#[test]
fn phone_pane_ellipsizes_only_a_model_that_cannot_fit() {
    let model = format!("DeepSeek {} ", "M".repeat(80));
    let mut agent = agent_with(&model, "1234.56", 100, 100);
    let buf = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
    eprintln!(
        "phone 55x41 pathological model — bottom band:\n{}",
        bottom_rows(&buf, 4)
    );
    let band = row_text(&buf, border_row(&buf) + 1);
    assert!(band.contains("$1234.56"), "{band:?}");
    assert!(band.contains("cache 100%"), "{band:?}");
    assert!(band.trim_end().ends_with("always-approve"), "{band:?}");
    assert!(band.contains('…'), "{band:?}");
    let money = band.find("$1234.56").unwrap();
    let mode = band.find("always-approve").unwrap();
    assert!(money + "$1234.56".len() < mode, "{band:?}");
}

#[test]
fn phone_pane_keeps_other_permission_modes_whole() {
    let mut auto = phone_agent();
    auto.session.set_yolo_mode_for_test(false);
    auto.session.set_auto_mode_for_test(true);
    let buf = draw(&mut auto, PHONE_COLS, PHONE_ROWS);
    eprintln!("phone 55x41 auto — bottom band:\n{}", bottom_rows(&buf, 4));
    assert_one_band(
        &buf,
        "$15.87",
        "cache 88%",
        "V4.1 Flash (max)",
        Some("auto"),
    );

    let mut ask = phone_agent();
    ask.session.set_yolo_mode_for_test(false);
    let buf = draw(&mut ask, PHONE_COLS, PHONE_ROWS);
    eprintln!("phone 55x41 ask — bottom band:\n{}", bottom_rows(&buf, 4));
    let band = row_text(&buf, border_row(&buf) + 1);
    assert_one_band(&buf, "$15.87", "cache 88%", "V4.1 Flash (max)", None);
    assert!(
        !band.contains("always-approve") && !band.contains("auto"),
        "{band:?}"
    );
    assert!(band.trim_end().ends_with("(max)"), "{band:?}");

    let mut plan = phone_agent();
    plan.plan_mode_active = true;
    let buf = draw(&mut plan, PHONE_COLS, PHONE_ROWS);
    eprintln!("phone 55x41 plan — bottom band:\n{}", bottom_rows(&buf, 4));
    // The plan flag plus the full cache label spend the columns the effort
    // suffix used to take; the mode itself stays whole.
    assert_one_band(
        &buf,
        "$15.87",
        "cache 88%",
        "V4.1 Flash",
        Some("always-approve"),
    );
    let band = row_text(&buf, border_row(&buf) + 1);
    assert!(band.contains("plan"), "plan stays on the row: {band:?}");
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

    assert_one_band(
        &buf,
        "$15.87",
        "cache 88%",
        "V4.1 Flash (max)",
        Some("always-approve"),
    );
    assert_no_hint_row(&agent, &registry, &buf);
}

#[test]
fn desktop_pane_keeps_the_label_on_the_divider_and_the_hint_row() {
    for (cols, rows) in [(80u16, 40u16), (DESKTOP_COLS, DESKTOP_ROWS), (180, 50)] {
        let mut agent = phone_agent();
        let buf = draw(&mut agent, cols, rows);
        eprintln!(
            "desktop {cols}x{rows} — bottom band:\n{}",
            bottom_rows(&buf, 6)
        );

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
        let balance_row = row_text(&buf, balance_y);
        assert!(
            balance_row.contains("cache 88%"),
            "the desktop cache chip keeps the word: {balance_row:?}"
        );
        assert!(
            !balance_row.contains("V4.1"),
            "the desktop balance row does not take the model: {balance_row:?}"
        );
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
}

// ── Prompt-area padding: the echo band, the composer box, the PTY bottom ──

/// Seed the scrollback with the echo a submitted prompt leaves in the frame.
fn seed_prompt_echo(agent: &mut AgentView, text: &str) {
    agent.scrollback.push_block(RenderBlock::user_prompt(text));
}

/// Rows the prompt echo's band touches, read at the echo's text column: the
/// text rows carry the band as their background, the pad rows carry it as the
/// ink of their fractional block glyph (`▂` / `▆`), and the meta clock row
/// carries it as the band fill, so every row of the band shows up here.
fn echo_band_rows(buf: &Buffer, band_bg: ratatui::style::Color) -> Vec<u16> {
    (0..buf.area.height)
        .filter(|&y| {
            buf.cell((1, y))
                .is_some_and(|c| c.bg == band_bg || c.fg == band_bg)
        })
        .collect()
}

/// The composer box's top border row: the last row whose outer left column
/// holds `╭` (its `╰` is [`border_row`]).
fn top_border_row(buf: &Buffer) -> u16 {
    let left = crate::appearance::LayoutConfig::default().eff_hpad_left(false) as usize;
    (0..buf.area.height)
        .rev()
        .find(|&y| {
            row_text(buf, y)
                .chars()
                .nth(left)
                .is_some_and(|c| c == '\u{256d}')
        })
        .unwrap_or_else(|| panic!("prompt box top border not found in\n{}", frame_text(buf)))
}

/// The frame the report's screenshots show, at the measured iPhone size: the
/// echo's band is its meta clock row, its text rows and one pad row each side,
/// the composer box is the top border, one text row and the divider, the turn
/// time stops one column inside the echo's band, and the status row closes the
/// phone frame flush to the measured PTY edge (the iOS host adds a pixel inset).
#[test]
fn phone_frame_pads_the_prompt_areas_by_one_cell() {
    let _guard = crate::theme::cache::pin_theme();
    let theme = Theme::current();
    let mut agent = phone_agent();
    // Two wrapped rows at the echo's content width, so the collapsed phone
    // budget shows the whole prompt and paints no fold affordance row. The
    // first row is full, so the clock takes the meta row above the top pad.
    seed_prompt_echo(&mut agent, &"M".repeat(60));
    agent.prompt.set_text("phone draft");
    let buf = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
    let frame = frame_text(&buf);
    eprintln!("phone 55x41 — full frame:\n{frame}");

    // (a) The echo band is the meta clock row, two text rows, and one pad row
    // each side, and each pad row spends two eighths of its height on the band
    // (one column of air at the phone's font, where a whole row is 2.15 columns).
    let band = echo_band_rows(&buf, theme.bg_light);
    assert_eq!(
        band.len(),
        5,
        "the echo band is the meta clock row, two text rows plus a pad row each side: {band:?}\n{frame}"
    );
    let meta = band[0];
    assert!(
        row_text(&buf, meta).contains("AM") || row_text(&buf, meta).contains("PM"),
        "the meta row carries the right-aligned turn clock:\n{frame}"
    );
    let top_pad = buf.cell((1, band[1])).unwrap();
    assert_eq!(
        top_pad.symbol(),
        "\u{2582}",
        "the top pad row keeps the band on its lower two eighths:\n{frame}"
    );
    assert_eq!(top_pad.fg, theme.bg_light, "{frame}");
    assert_eq!(top_pad.bg, theme.bg_base, "{frame}");
    let bottom_pad = buf.cell((1, band[4])).unwrap();
    assert_eq!(
        bottom_pad.symbol(),
        "\u{2586}",
        "the bottom pad row keeps the band on its upper two eighths:\n{frame}"
    );
    assert_eq!(bottom_pad.bg, theme.bg_light, "{frame}");
    for y in [band[2], band[3]] {
        assert!(
            row_text(&buf, y).contains('M'),
            "text row {y} carries the prompt:\n{frame}"
        );
        assert_eq!(buf.cell((0, y)).unwrap().bg, theme.bg_light, "{frame}");
        assert_eq!(
            buf.cell((PHONE_COLS - 1, y)).unwrap().bg,
            theme.bg_light,
            "{frame}"
        );
    }

    // (b) The composer box is exactly the border, text and divider rows.
    let top = top_border_row(&buf);
    let divider = border_row(&buf);
    assert_eq!(
        divider - top,
        2,
        "the composer box is border + text + divider, got rows {top}..={divider}:\n{frame}"
    );
    assert!(
        row_text(&buf, top + 1).contains("phone draft"),
        "the box's middle row is the text row:\n{frame}"
    );

    // (c) The turn time stops one column inside the echo's band. The echo's
    // first content row is full, so the clock sits on the meta row; that row is
    // still part of the band.
    let echo_y = meta;
    let band_right = (0..PHONE_COLS)
        .rev()
        .find(|&x| {
            buf.cell((x, echo_y))
                .is_some_and(|c| c.bg == theme.bg_light)
        })
        .expect("the echo's band must have a right edge");
    let last_ink = (0..PHONE_COLS - 1)
        .rev()
        .find(|&x| buf.cell((x, echo_y)).is_some_and(|c| c.symbol() != " "))
        .expect("the echo's meta row must carry the clock");
    assert_eq!(
        band_right - last_ink,
        2,
        "the time keeps its original inset inside the full-width band: ink {last_ink}, band {band_right}\n{frame}"
    );

    // (d) The status band reaches the PTY bottom; the host adds pixel space below.
    let status_y = divider + 1;
    assert_eq!(
        status_y + 1 + crate::views::agent::PHONE_BOTTOM_MARGIN_ROWS,
        PHONE_ROWS,
        "the status band reaches the measured PTY bottom:\n{frame}"
    );
}

/// The status band reaches the measured PTY bottom at every phone height.
#[test]
fn phone_status_band_reaches_pty_bottom_at_every_phone_height() {
    assert_eq!(
        crate::views::agent::PHONE_BOTTOM_MARGIN_ROWS,
        0,
        "the phone layout leaves pixel spacing to the host"
    );
    for rows in [PHONE_ROWS, 36, 33, 30, 26, 24, 20] {
        let mut agent = phone_agent();
        let buf = draw(&mut agent, PHONE_COLS, rows);
        let frame = frame_text(&buf);
        let status_y = border_row(&buf) + 1;
        assert_eq!(
            status_y + 1 + crate::views::agent::PHONE_BOTTOM_MARGIN_ROWS,
            rows,
            "{PHONE_COLS}x{rows}: the status band reaches the PTY bottom\n{frame}"
        );
    }
}
