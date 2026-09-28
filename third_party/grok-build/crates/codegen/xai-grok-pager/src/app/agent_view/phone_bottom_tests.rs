//! Phone-width bottom stack (measured iPhone Orca pane: 55 columns).
//!
//! The composer is a band across the frame, like the prompt echo's, and the
//! frame ends on the two-row footer under it: balance and cache against the
//! model, then this session's tokens against the permission mode. The
//! shortcut-hint row stays absent. Desktop widths keep the boxed composer with
//! the label on its divider and the balance on its own row.
//!
//! `views::prompt_widget`'s tests pin the widget in isolation; these frames pin the
//! whole-view stack (the hint row is a layout row owned by `render`).

use super::{
    AgentView, AppRenderParams, BannerSlotParams, PromptInputMode, PromptMode, test_fixtures,
};
use crate::actions::ActionRegistry;
use crate::app::agent::AgentState;
use crate::scrollback::RenderBlock;
use crate::scrollback::render::ScratchBuffer;
use crate::theme::Theme;
use crate::views::credit_bar::CreditBalance;
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
    // The app derives the pane's density flag from the terminal grid
    // (`AppView::apply_effective_density`); the fixture sets the same value, so
    // a frame test draws the frame the pane draws.
    let mut appearance = agent.scrollback.appearance().clone();
    appearance.scrollback.layout.narrow = crate::views::agent::effective_narrow(cols, rows);
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

/// The composer band's pad rows on a phone: the last row made only of `▂`
/// (the band's bottom pad) and the last row above it made only of `▆` (its top
/// pad). The band spans the frame, so every cell of a pad row is the glyph.
fn composer_band_rows(buf: &Buffer) -> (u16, u16) {
    let all = |y: u16, glyph: &str| {
        (0..buf.area.width).all(|x| buf.cell((x, y)).is_some_and(|c| c.symbol() == glyph))
    };
    let bottom = (0..buf.area.height)
        .rev()
        .find(|&y| all(y, "\u{2582}"))
        .unwrap_or_else(|| panic!("composer band bottom pad not found in\n{}", frame_text(buf)));
    let top = (0..bottom)
        .rev()
        .find(|&y| all(y, "\u{2586}"))
        .unwrap_or_else(|| panic!("composer band top pad not found in\n{}", frame_text(buf)));
    (top, bottom)
}

/// The two rows that end the phone frame, right under the composer band. Row
/// one: balance and cache from the first inner column, the model flush to the
/// inner right edge. Row two: this session's tokens, then the mode flush right.
/// The outer column on each side stays blank, and two columns stay clear
/// between the sides of a row.
fn assert_footer(
    buf: &Buffer,
    balance: &str,
    cache: &str,
    model: &str,
    tokens: Option<&str>,
    mode: Option<&str>,
) {
    let frame = frame_text(buf);
    let (_, band_bottom) = composer_band_rows(buf);
    let (row1_y, row2_y) = (buf.area.height - 2, buf.area.height - 1);
    assert_eq!(
        band_bottom + 1,
        row1_y,
        "the footer sits right under the composer band and ends the frame:\n{frame}"
    );
    let inner_right = buf.area.width as usize - 1;
    let (row1, row2) = (row_text(buf, row1_y), row_text(buf, row2_y));
    for row in [&row1, &row2] {
        assert!(row.starts_with(' ') && row.ends_with(' '), "{row:?}");
    }
    let left1 = format!(" {balance} {cache}");
    assert!(
        row1.starts_with(&left1),
        "row one opens on {left1:?}: {row1:?}"
    );
    assert!(
        row1.trim_end().ends_with(model) && row1.trim_end().chars().count() == inner_right,
        "row one closes on {model:?} at the inner edge: {row1:?}"
    );
    let model_at = row1.chars().count() - 1 - model.chars().count();
    assert!(
        model_at >= left1.chars().count() + 2,
        "two columns stay clear on row one: {row1:?}"
    );
    if let Some(tokens) = tokens {
        assert!(
            row2.starts_with(&format!(" {tokens}")),
            "row two opens on {tokens:?}: {row2:?}"
        );
    }
    match mode {
        Some(mode) => assert!(
            row2.trim_end().ends_with(mode) && row2.trim_end().chars().count() == inner_right,
            "row two closes on {mode:?} at the inner edge: {row2:?}"
        ),
        None => assert!(
            tokens.is_none_or(|t| row2.trim() == t),
            "row two carries no mode: {row2:?}"
        ),
    }
}

#[test]
fn phone_pane_ends_on_a_two_row_footer_and_drops_the_hint_row() {
    let mut agent = phone_agent();
    let registry = ActionRegistry::defaults();
    let buf = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
    eprintln!("phone 55x41 — bottom rows:\n{}", bottom_rows(&buf, 6));

    assert_footer(
        &buf,
        "$15.87",
        "cache 88%",
        "DeepSeek V4.1 Flash (max)",
        Some("4.1k in \u{b7} 0 out"),
        Some("always-approve"),
    );
    assert_no_hint_row(&agent, &registry, &buf);
}

#[test]
fn prompt_never_renders_grok_quota_across_widths_providers_and_modes() {
    let _guard = crate::theme::cache::pin_theme();
    for (cols, rows) in [(55, 41), (61, 41), (62, 41), (80, 40), (120, 40), (180, 50)] {
        for usage_pct in [92.0, 100.0] {
            for status in [
                "missing",
                "deepseek",
                "grok",
                "stale-deepseek",
                "stale-grok",
                "unbound-grok",
                "no-session",
            ] {
                for mode in ["normal", "queued", "bash", "remember"] {
                    let mut agent = phone_agent();
                    agent.session.session_id = Some(acp::SessionId::new("active"));
                    agent.deepseek_status_session_id = agent.session.session_id.clone();
                    match status {
                        "missing" => agent.deepseek_status = None,
                        "no-session" => agent.session.session_id = None,
                        "unbound-grok" => agent.deepseek_status_session_id = None,
                        "stale-deepseek" | "stale-grok" => {
                            agent.deepseek_status_session_id =
                                Some(acp::SessionId::new("previous"));
                        }
                        _ => {}
                    }
                    if let Some(provider) = agent.deepseek_status.as_mut() {
                        provider.is_deepseek = matches!(status, "deepseek" | "stale-deepseek");
                    }
                    agent.billing_surface_visible = true;
                    agent.credit_balance = Some(CreditBalance {
                        usage_pct,
                        effective_usage_pct: usage_pct,
                        period_end_display: None,
                        pay_as_you_go: false,
                        on_demand_cap_cents: None,
                        on_demand_used_cents: None,
                        prepaid_balance_cents: None,
                        period_type: Some("USAGE_PERIOD_TYPE_WEEKLY".into()),
                        is_unified_billing_user: None,
                    });
                    let label = match mode {
                        "queued" => {
                            agent.prompt_mode = PromptMode::EditingQueued {
                                id: 0,
                                original: String::new(),
                                server_id: None,
                                kind: crate::app::agent::QueueEntryKind::Prompt,
                            };
                            "editing queued #1"
                        }
                        "bash" => {
                            agent.prompt_input_mode = PromptInputMode::Bash;
                            "Run shell command"
                        }
                        "remember" => {
                            agent.prompt_input_mode = PromptInputMode::Remember;
                            "Save memory note"
                        }
                        _ => "Flash",
                    };
                    let buf = draw(&mut agent, cols, rows);
                    let frame = frame_text(&buf);
                    assert!(
                        !frame.contains("Weekly limit") && !frame.contains("limit left"),
                        "quota at {cols}x{rows}, usage={usage_pct}, status={status}, mode={mode}:\n{frame}"
                    );
                    assert!(
                        frame.contains(label),
                        "prompt label {label:?} missing:\n{frame}"
                    );
                }
            }
        }
    }
}

#[test]
fn prompt_stays_quota_free_when_the_model_and_provider_change() {
    let _guard = crate::theme::cache::pin_theme();
    let mut agent = phone_agent();
    agent.session.session_id = Some(acp::SessionId::new("active"));
    agent.deepseek_status_session_id = agent.session.session_id.clone();
    agent.billing_surface_visible = true;
    agent.credit_balance = Some(CreditBalance {
        usage_pct: 100.0,
        effective_usage_pct: 100.0,
        period_end_display: None,
        pay_as_you_go: false,
        on_demand_cap_cents: None,
        on_demand_used_cents: None,
        prepaid_balance_cents: None,
        period_type: Some("USAGE_PERIOD_TYPE_WEEKLY".into()),
        is_unified_billing_user: None,
    });
    for (id, label, is_deepseek) in [
        (MODEL_ID, MODEL_LABEL, true),
        ("grok-code", "Grok Code", false),
        (MODEL_ID, MODEL_LABEL, true),
    ] {
        let id = acp::ModelId::new(id);
        agent.session.models.available.insert(
            id.clone(),
            acp::ModelInfo::new(id.clone(), label.to_string()),
        );
        agent.session.models.current = Some(id);
        agent.deepseek_status.as_mut().unwrap().is_deepseek = is_deepseek;
        for (cols, rows) in [(PHONE_COLS, PHONE_ROWS), (DESKTOP_COLS, DESKTOP_ROWS)] {
            let buf = draw(&mut agent, cols, rows);
            let frame = frame_text(&buf);
            assert!(
                !frame.contains("Weekly limit"),
                "after switch to {label}:\n{frame}"
            );
            assert!(frame.contains(label), "current model missing:\n{frame}");
            if is_deepseek {
                assert!(
                    frame.contains("$15.87") && frame.contains("cache 88%"),
                    "cost chips missing:\n{frame}"
                );
            }
            assert!(
                frame.contains("always-approve"),
                "permission missing:\n{frame}"
            );
        }
    }
}

#[test]
fn phone_pane_fits_a_large_balance_and_a_full_cache() {
    let mut agent = agent_with(MODEL_LABEL, "1234.56", 100, 100);
    let buf = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
    eprintln!(
        "phone 55x41 large balance — bottom rows:\n{}",
        bottom_rows(&buf, 4)
    );
    // Row one holds only money and the model, so even the widest balance
    // leaves the model its full name.
    assert_footer(
        &buf,
        "$1234.56",
        "cache 100%",
        "DeepSeek V4.1 Flash (max)",
        Some("100 in \u{b7} 0 out"),
        Some("always-approve"),
    );
}

#[test]
fn phone_pane_drops_only_the_prefix_a_long_model_needs() {
    let mut agent = agent_with("DeepSeek V4.1 Flash Thinking", "1234.56", 100, 100);
    let buf = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
    eprintln!(
        "phone 55x41 long model — bottom rows:\n{}",
        bottom_rows(&buf, 4)
    );
    // `DeepSeek V4.1 Flash Thinking (max)` is 34 columns beside 19 of money;
    // the prefix goes, the effort stays.
    assert_footer(
        &buf,
        "$1234.56",
        "cache 100%",
        "V4.1 Flash Thinking (max)",
        None,
        Some("always-approve"),
    );
}

#[test]
fn phone_pane_ellipsizes_only_a_model_that_cannot_fit() {
    let model = format!("DeepSeek {} ", "M".repeat(80));
    let mut agent = agent_with(&model, "1234.56", 100, 100);
    let buf = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
    eprintln!(
        "phone 55x41 pathological model — bottom rows:\n{}",
        bottom_rows(&buf, 4)
    );
    let (row1, row2) = (
        row_text(&buf, PHONE_ROWS - 2),
        row_text(&buf, PHONE_ROWS - 1),
    );
    assert!(row1.starts_with(" $1234.56 cache 100%"), "{row1:?}");
    assert!(row1.contains('\u{2026}'), "only the model is cut: {row1:?}");
    assert!(row2.trim_end().ends_with("always-approve"), "{row2:?}");
}

#[test]
fn phone_pane_keeps_other_permission_modes_whole() {
    let tokens = Some("4.1k in \u{b7} 0 out");
    let model = "DeepSeek V4.1 Flash (max)";
    let mut auto = phone_agent();
    auto.session.set_yolo_mode_for_test(false);
    auto.session.set_auto_mode_for_test(true);
    let buf = draw(&mut auto, PHONE_COLS, PHONE_ROWS);
    eprintln!("phone 55x41 auto — bottom rows:\n{}", bottom_rows(&buf, 4));
    assert_footer(&buf, "$15.87", "cache 88%", model, tokens, Some("auto"));

    let mut ask = phone_agent();
    ask.session.set_yolo_mode_for_test(false);
    let buf = draw(&mut ask, PHONE_COLS, PHONE_ROWS);
    eprintln!("phone 55x41 ask — bottom rows:\n{}", bottom_rows(&buf, 4));
    assert_footer(&buf, "$15.87", "cache 88%", model, tokens, None);

    let mut plan = phone_agent();
    plan.plan_mode_active = true;
    let buf = draw(&mut plan, PHONE_COLS, PHONE_ROWS);
    eprintln!("phone 55x41 plan — bottom rows:\n{}", bottom_rows(&buf, 4));
    // The plan flag rides row two with the mode; row one keeps the model whole.
    assert_footer(
        &buf,
        "$15.87",
        "cache 88%",
        model,
        tokens,
        Some("plan \u{b7} always-approve"),
    );
}

#[test]
fn phone_pane_drops_the_hint_row_mid_turn_too() {
    let mut agent = phone_agent();
    agent.session.state = AgentState::TurnRunning;
    let registry = ActionRegistry::defaults();
    let buf = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
    eprintln!(
        "phone 55x41 mid-turn — bottom rows:\n{}",
        bottom_rows(&buf, 7)
    );

    assert_footer(
        &buf,
        "$15.87",
        "cache 88%",
        "DeepSeek V4.1 Flash (max)",
        Some("4.1k in \u{b7} 0 out"),
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

/// The frame the report's screenshots show, at the measured iPhone size: the
/// echo's band is its text rows (the last one closing on the turn clock) and
/// one pad row each side, the composer is a band across the frame with its text
/// row between a `▆` and a `▂` pad row, the turn time stops one column inside
/// the echo's band, and the two-row footer closes the phone frame at the
/// measured PTY edge.
#[test]
fn phone_frame_pads_the_prompt_areas_by_one_cell() {
    let _guard = crate::theme::cache::pin_theme();
    let theme = Theme::current();
    let mut agent = phone_agent();
    // Two wrapped rows at the echo's 52-column text width, so the collapsed
    // phone budget shows the whole prompt and paints no fold affordance row.
    // The last row leaves room, so the clock closes it.
    seed_prompt_echo(&mut agent, &"M".repeat(60));
    agent.prompt.set_text("phone draft");
    let buf = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
    let frame = frame_text(&buf);
    eprintln!("phone 55x41 — full frame:\n{frame}");

    // (a) The echo band is the top pad, two text rows and the bottom pad: the
    // clock closes the last text row at the band's bottom-right, like a chat
    // bubble, and takes no row of its own. Each pad row spends two eighths of
    // its height on the band (one column of air at the phone's font, where a
    // whole row is 2.15 columns).
    // The composer's band shares the echo's colour; the echo is the band above it.
    let (composer_top, _) = composer_band_rows(&buf);
    let band: Vec<u16> = echo_band_rows(&buf, theme.bg_light)
        .into_iter()
        .filter(|&y| y < composer_top)
        .collect();
    assert_eq!(
        band.len(),
        4,
        "the echo band is a pad row each side and two text rows: {band:?}\n{frame}"
    );
    assert!(
        band.windows(2).all(|w| w[1] == w[0] + 1),
        "the band's rows are contiguous: {band:?}\n{frame}"
    );
    let clock_row = band[2];
    assert!(
        row_text(&buf, clock_row).contains("AM") || row_text(&buf, clock_row).contains("PM"),
        "the last text row, above the bottom pad, carries the right-aligned turn clock:\n{frame}"
    );
    assert!(
        !(row_text(&buf, band[1]).contains("AM") || row_text(&buf, band[1]).contains("PM")),
        "the first text row carries no clock:\n{frame}"
    );
    let top_pad = buf.cell((1, band[0])).unwrap();
    assert_eq!(
        top_pad.symbol(),
        "\u{2582}",
        "the top pad row keeps the band on its lower two eighths:\n{frame}"
    );
    assert_eq!(top_pad.fg, theme.bg_light, "{frame}");
    assert_eq!(top_pad.bg, theme.bg_base, "{frame}");
    let bottom_pad = buf.cell((1, band[3])).unwrap();
    assert_eq!(
        bottom_pad.symbol(),
        "\u{2586}",
        "the bottom pad row keeps the band on its upper two eighths:\n{frame}"
    );
    assert_eq!(bottom_pad.bg, theme.bg_light, "{frame}");
    for y in [band[1], band[2]] {
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
    // The echo's text keeps the minimum unit of air inside its band: one
    // column at the left edge, and on the right the held-copy gutter (then the
    // scrollbar's column, band while no bar is drawn).
    let first_text = band[1];
    assert_eq!(buf.cell((0, first_text)).unwrap().symbol(), " ", "{frame}");
    assert_eq!(buf.cell((1, first_text)).unwrap().symbol(), "M", "{frame}");
    assert_eq!(
        buf.cell((PHONE_COLS - 3, first_text)).unwrap().symbol(),
        "M",
        "the full first row runs to the column before the gutter:\n{frame}"
    );
    assert_eq!(
        buf.cell((PHONE_COLS - 2, first_text)).unwrap().symbol(),
        " ",
        "{frame}"
    );

    // (b) The composer is a band: a top pad row whose lower three quarters are
    // the band, one text row, and a bottom pad row whose upper three quarters
    // are the band. It spans the frame in the echo's colour and draws no rule.
    let (top, bottom) = composer_band_rows(&buf);
    assert_eq!(
        bottom - top,
        2,
        "the composer band is pad + text + pad, got rows {top}..={bottom}:\n{frame}"
    );
    let text_row = top + 1;
    assert!(
        row_text(&buf, text_row).contains("phone draft"),
        "the band's middle row is the text row:\n{frame}"
    );
    let top_pad = buf.cell((0, top)).unwrap();
    assert_eq!(
        (top_pad.fg, top_pad.bg),
        (theme.bg_light, theme.bg_base),
        "{frame}"
    );
    let bottom_pad = buf.cell((0, bottom)).unwrap();
    assert_eq!(
        (bottom_pad.fg, bottom_pad.bg),
        (theme.bg_base, theme.bg_light),
        "{frame}"
    );
    for x in [0, PHONE_COLS - 1] {
        let cell = buf.cell((x, text_row)).unwrap();
        assert_eq!(
            (cell.symbol(), cell.bg),
            (" ", theme.bg_light),
            "the band's edge column {x} is fill, not a rule:\n{frame}"
        );
    }
    assert!(
        !frame.contains('\u{2570}') && !frame.contains('\u{256d}'),
        "a phone draws no box:\n{frame}"
    );

    // (c) The turn time closes on the transcript's last column: the held-copy
    // gutter the transcript leaves blank is its column of air, as tight as the
    // text's one column on the left, and the scrollbar's column follows (band
    // while no bar is drawn). The clock shares the echo's last text row.
    let echo_y = clock_row;
    let last_ink = (0..PHONE_COLS)
        .rev()
        .find(|&x| buf.cell((x, echo_y)).is_some_and(|c| c.symbol() != " "))
        .expect("the echo's last text row must carry the clock");
    for x in last_ink + 1..PHONE_COLS {
        let cell = buf.cell((x, echo_y)).unwrap();
        assert!(
            cell.symbol() == " " && cell.bg == theme.bg_light,
            "column {x} right of the time is blank band\n{frame}"
        );
    }
    assert_eq!(
        PHONE_COLS - 1 - last_ink,
        2,
        "the copy gutter and the bar's column follow the ink\n{frame}"
    );

    // (d) The two-row footer follows the band and ends the frame.
    assert_eq!(
        bottom + 1 + crate::views::agent::PHONE_FOOTER_ROWS,
        PHONE_ROWS,
        "the footer's two rows end the frame at the measured PTY bottom:\n{frame}"
    );
}

/// The footer ends the frame at every phone height, short terminals included,
/// and still paints its content there: a geometry-only check would pass on
/// blank rows, so each height runs the same content assertions as the 55x41
/// frame test.
#[test]
fn phone_frame_ends_on_its_footer_at_every_phone_height() {
    let short = crate::views::agent::SHORT_TERMINAL_ROWS;
    for rows in [PHONE_ROWS, 36, 33, 30, 26, 24, 20, short] {
        let mut agent = phone_agent();
        let buf = draw(&mut agent, PHONE_COLS, rows);
        assert_footer(
            &buf,
            "$15.87",
            "cache 88%",
            "DeepSeek V4.1 Flash (max)",
            Some("4.1k in \u{b7} 0 out"),
            Some("always-approve"),
        );
    }
}

// ── Scrolled transcript: the pinned echo, the band's right edge, the scrollbar column ──

/// Three turns, each a prompt with a unique marker and an answer long enough to
/// scroll the phone pane, with a fenced code block whose shading reaches the
/// transcript's right edge.
fn seed_scrolling_turns(agent: &mut AgentView) {
    for turn in 0..3 {
        seed_prompt_echo(
            agent,
            &format!("prompt-{turn} 지금 얼마나 완벽하게 다 개선되었나 테스트 좀 해보자."),
        );
        let mut body: String = (0..8)
            .map(|i| format!("answer {turn} line {i}\n\n"))
            .collect();
        body.push_str("```rust\nfn main() {\n    println!(\"a line long enough to reach the pane edge\");\n}\n```\n\n");
        body.extend((8..30).map(|i| format!("answer {turn} line {i}\n\n")));
        agent
            .scrollback
            .push_block(RenderBlock::agent_message(body));
    }
}

fn set_compact(agent: &mut AgentView, compact: bool) {
    let mut appearance = agent.scrollback.appearance().clone();
    appearance.prompt.compact = compact;
    agent.scrollback.set_appearance(appearance);
}

/// Scroll so the middle turn's prompt sits wholly above the viewport (its answer
/// fills the pane) and redraw. Returns the frame.
fn scroll_into_middle_answer(agent: &mut AgentView, cols: u16, rows: u16) -> Buffer {
    let _ = draw(agent, cols, rows);
    let prompt = agent
        .scrollback
        .get_cached_prompt_descriptors()
        .and_then(|d| d.get(1).copied())
        .expect("the middle prompt's descriptor");
    let past = prompt.y_virtual + usize::from(prompt.full_height) + 6;
    agent.scrollback.set_scroll_offset(past);
    let buf = draw(agent, cols, rows);
    assert_eq!(
        agent.scrollback.scroll_offset(),
        past,
        "the middle answer is tall enough to scroll into"
    );
    buf
}

fn rows_with(buf: &Buffer, needle: &str) -> Vec<u16> {
    (0..buf.area.height)
        .filter(|&y| row_text(buf, y).contains(needle))
        .collect()
}

/// A phone pane pins the prompt echo a scrolled-up reader is inside, in compact
/// mode too: a `/compact-mode` taken from the small-screen tip, or auto-compact
/// while the keyboard shrinks the pane, used to drop the pinned echo, so the
/// echo scrolled away. A desktop pane keeps compact's own rule.
#[test]
fn phone_pins_the_echo_in_compact_mode_too() {
    let _guard = crate::theme::cache::pin_theme();
    let theme = Theme::current();
    for (rows, compact) in [(PHONE_ROWS, false), (PHONE_ROWS, true), (20, true)] {
        let mut agent = phone_agent();
        seed_scrolling_turns(&mut agent);
        set_compact(&mut agent, compact);
        let buf = scroll_into_middle_answer(&mut agent, PHONE_COLS, rows);
        let frame = frame_text(&buf);
        let pinned = agent
            .scrollback
            .sticky_layout()
            .and_then(|sticky| sticky.pinned)
            .unwrap_or_else(|| {
                panic!("{PHONE_COLS}x{rows} compact={compact}: no pinned echo\n{frame}")
            });
        assert_eq!(pinned.entry_idx, 2, "the middle prompt is pinned\n{frame}");
        let marker = rows_with(&buf, "prompt-1");
        assert_eq!(
            marker.len(),
            1,
            "{PHONE_COLS}x{rows} compact={compact}: the pinned echo shows its prompt once\n{frame}"
        );
        assert!(
            marker[0] <= 3,
            "{PHONE_COLS}x{rows} compact={compact}: the echo is pinned under the status bar, got row {}\n{frame}",
            marker[0]
        );
        // The band's own right pad column never holds text.
        assert_eq!(
            buf.cell((PHONE_COLS - 3, marker[0])).unwrap().bg,
            theme.bg_light,
            "the pinned echo paints its band\n{frame}"
        );
    }

    let mut desktop = phone_agent();
    seed_scrolling_turns(&mut desktop);
    set_compact(&mut desktop, true);
    let buf = scroll_into_middle_answer(&mut desktop, DESKTOP_COLS, DESKTOP_ROWS);
    assert!(
        rows_with(&buf, "prompt-1").is_empty(),
        "a desktop pane keeps compact mode's scrolling echo\n{}",
        frame_text(&buf)
    );
}

/// A pinned phone echo keeps every row it paints, and its clock only closes a
/// last text row that leaves room: it never takes a row of its own. A pinned
/// header shrinks to its floor as the reader scrolls on; that floor was the
/// prompt's Truncated height, which left out a clock row and, for a prompt the
/// width-blind fold check keeps expanded, the rows it wraps past the fold
/// budget. The header then painted a row more than it had: its bottom pad
/// landed on the last text row, and the band showed only under its glyphs.
#[test]
fn phone_pinned_echo_keeps_its_band_and_closes_on_its_clock() {
    let _guard = crate::theme::cache::pin_theme();
    let theme = Theme::current();
    let band = theme.bg_light;
    let full_rows = "M".repeat(104);
    // Three words that wrap one to a row at the echo's 52-column text width.
    // `ceil(96 / 52) = 2` rows once kept this prompt expanded at three rows; the
    // fold check counts the wrapped rows, so it folds to two and the ellipsis.
    let three_rows = format!("{} {} {}", "A".repeat(27), "B".repeat(27), "C".repeat(40));
    // Folded, the second row fills with Arabic lam-alef pairs. `UnicodeWidthStr`
    // reads each pair as one column; the buffer paints a cell per letter.
    let ligature_rows = format!(
        "head\n{} {}\ntail",
        "A".repeat(20),
        "\u{644}\u{627}".repeat(16)
    );
    for (label, prompt, head, tail, text_rows, clock) in [
        // The prompt from the report: its second row leaves room.
        (
            "short last row",
            "지금 6.1.6 배포중인데. 관련해서 모든 이슈들 다 고쳤어??",
            "6.1.6",
            "??",
            2u16,
            true,
        ),
        // Both rows full: no room, so no clock.
        (
            "full last row",
            full_rows.as_str(),
            "MMMM",
            "MMMM",
            2,
            false,
        ),
        // Folded: the second row runs on into the third word up to the
        // ellipsis, so it is full and the clock has no room.
        (
            "word wrap past the budget folds",
            three_rows.as_str(),
            "AAAA",
            "C \u{2026}",
            2,
            false,
        ),
        // The same full row, measured by the cells it paints: no clock over it.
        (
            "full ligature row",
            ligature_rows.as_str(),
            "head",
            "\u{644} \u{2026}",
            2,
            false,
        ),
    ] {
        let mut agent = phone_agent();
        for turn in 0..3 {
            let text = if turn == 1 {
                prompt.to_owned()
            } else {
                format!("prompt-{turn} 지금 얼마나 완벽하게 다 개선되었나 테스트 좀 해보자.")
            };
            seed_prompt_echo(&mut agent, &text);
            let body: String = (0..30)
                .map(|i| format!("answer {turn} line {i}\n\n"))
                .collect();
            agent
                .scrollback
                .push_block(RenderBlock::agent_message(body));
        }
        // The echo sits above the viewport from the first frame, so its height
        // is the off-screen estimate, as after a resume or a layout rebuild.
        let buf = scroll_into_middle_answer(&mut agent, PHONE_COLS, PHONE_ROWS);
        let frame = frame_text(&buf);
        let full_height = agent
            .scrollback
            .get_cached_prompt_descriptors()
            .and_then(|d| d.get(1).copied())
            .expect("the middle prompt's descriptor")
            .full_height;
        let pinned = agent
            .scrollback
            .sticky_layout()
            .and_then(|sticky| sticky.pinned)
            .unwrap_or_else(|| panic!("{label}: no pinned echo\n{frame}"));
        assert_eq!(
            pinned.entry_idx, 2,
            "{label}: the middle prompt is pinned\n{frame}"
        );
        assert_eq!(
            full_height,
            text_rows + 2,
            "{label}: the echo is its text rows and a pad row each side, no clock row\n{frame}"
        );
        assert_eq!(
            pinned.render_height, full_height,
            "{label}: the pinned echo keeps every row it paints\n{frame}"
        );

        let first_text = rows_with(&buf, head)[0];
        let last_text = *rows_with(&buf, tail)
            .last()
            .unwrap_or_else(|| panic!("{label}: no row shows {tail:?}\n{frame}"));
        let top_pad = first_text - 1;
        let bottom_pad = top_pad + full_height - 1;
        assert_eq!(
            last_text,
            bottom_pad - 1,
            "{label}: the last text row sits right above the bottom pad\n{frame}"
        );
        assert_eq!(
            buf.cell((1, top_pad)).unwrap().symbol(),
            "\u{2582}",
            "{label}\n{frame}"
        );
        assert_eq!(
            buf.cell((1, bottom_pad)).unwrap().symbol(),
            "\u{2586}",
            "{label}: the bottom pad closes the band\n{frame}"
        );
        let bar = PHONE_COLS - 1;
        for y in first_text..bottom_pad {
            for x in 0..bar {
                let cell = buf.cell((x, y)).unwrap();
                assert!(
                    cell.symbol() != "\u{2582}" && cell.symbol() != "\u{2586}",
                    "{label}: no pad glyph on the band's row {y} (column {x})\n{frame}"
                );
            }
        }
        // Right of the last text row's glyphs the band reaches the scrollbar:
        // no cell of that row shows the pane's background.
        let tail_glyph = tail.chars().last().unwrap().to_string();
        let tail_end = (0..bar)
            .rev()
            .find(|&x| buf.cell((x, last_text)).unwrap().symbol() == tail_glyph)
            .unwrap()
            + 1;
        for x in tail_end..bar {
            assert_eq!(
                buf.cell((x, last_text)).unwrap().bg,
                band,
                "{label}: band column {x} of the last text row {last_text}\n{frame}"
            );
        }

        let clock_rows: Vec<u16> = (top_pad..=bottom_pad)
            .filter(|&y| {
                let text = row_text(&buf, y);
                text.contains(" AM") || text.contains(" PM")
            })
            .collect();
        let expected = if clock { vec![last_text] } else { Vec::new() };
        assert_eq!(
            clock_rows, expected,
            "{label}: the clock closes the last text row when it fits, and is absent otherwise\n{frame}"
        );
        if clock {
            let row = row_text(&buf, last_text);
            assert!(
                row.trim_end().ends_with('M'),
                "{label}: the clock is right-aligned at the band's bottom-right: {row:?}\n{frame}"
            );
        }
    }
}

/// A pinned prompt's floor is its own height on a wide pane too. A prompt whose
/// words wrap to four rows, with its long clock tapped open on the row above
/// its text, once stayed expanded (the fold check's `ceil` bound said three
/// rows) and painted seven rows under a floor capped at six. The fold check now
/// counts the wrapped rows, so the prompt folds to its three-row budget and
/// the pinned header keeps all six rows it paints.
#[test]
fn a_wide_pinned_prompt_keeps_every_row_it_paints() {
    let _guard = crate::theme::cache::pin_theme();
    const COLS: u16 = 72;
    const ROWS: u16 = 24;
    let mut agent = phone_agent();
    // One word to a row; the first fills its row, so the long clock does not fit
    // beside it.
    let prompt = format!(
        "{} {} {} {}",
        "A".repeat(50),
        "B".repeat(37),
        "C".repeat(37),
        "D".repeat(37)
    );
    for turn in 0..3 {
        let text = if turn == 1 {
            prompt.clone()
        } else {
            format!("prompt-{turn}")
        };
        seed_prompt_echo(&mut agent, &text);
        let body: String = (0..30)
            .map(|i| format!("answer {turn} line {i}\n\n"))
            .collect();
        agent
            .scrollback
            .push_block(RenderBlock::agent_message(body));
    }
    agent
        .scrollback
        .entry_mut(2)
        .expect("the middle echo")
        .timestamp_expanded = true;
    let buf = scroll_into_middle_answer(&mut agent, COLS, ROWS);
    let frame = frame_text(&buf);
    assert!(
        !agent.scrollback.appearance().scrollback.layout.narrow,
        "a {COLS}x{ROWS} pane is wide\n{frame}"
    );
    let full_height = agent
        .scrollback
        .get_cached_prompt_descriptors()
        .and_then(|d| d.get(1).copied())
        .expect("the middle prompt's descriptor")
        .full_height;
    assert_eq!(
        full_height, 6,
        "the long clock's row, a pad row each side and the three folded text rows\n{frame}"
    );
    let pinned = agent
        .scrollback
        .sticky_layout()
        .and_then(|sticky| sticky.pinned)
        .unwrap_or_else(|| panic!("no pinned echo\n{frame}"));
    assert_eq!(pinned.entry_idx, 2, "the middle prompt is pinned\n{frame}");
    assert_eq!(
        pinned.render_height, full_height,
        "the pinned prompt keeps every row it paints\n{frame}"
    );
    for word in ["AAAA", "BBBB", "CCCC"] {
        assert_eq!(
            rows_with(&buf, word).len(),
            1,
            "the pinned header shows the {word} row\n{frame}"
        );
    }
    // The fourth wrapped row is folded away: its word shows only where the
    // third row runs on into it up to the ellipsis.
    let third = rows_with(&buf, "CCCC");
    assert_eq!(
        rows_with(&buf, "D \u{2026}"),
        third,
        "the third row fills with the fourth word and ends in the ellipsis\n{frame}"
    );
    assert_eq!(
        rows_with(&buf, "DDDD"),
        third,
        "no row of its own for the fourth word\n{frame}"
    );
}

/// A phone echo row that ends on a wide glyph carries the band into the copy
/// gutter too. The glyph's trailing half in the transcript's last column has
/// the buffer's default style, and the gutter used to copy it: a grey dot of
/// the terminal's own background between the band and the scrollbar.
#[test]
fn phone_echo_band_reaches_the_gutter_after_a_wide_glyph() {
    let _guard = crate::theme::cache::pin_theme();
    let theme = Theme::current();
    let mut agent = phone_agent();
    // 26 two-column glyphs fill the first 52-column text row exactly.
    seed_prompt_echo(&mut agent, &format!("{}나다라", "가".repeat(26)));
    let buf = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
    let frame = frame_text(&buf);
    let y = rows_with(&buf, "가")[0];
    assert_eq!(
        buf.cell((PHONE_COLS - 4, y)).unwrap().symbol(),
        "가",
        "the row's last glyph starts two columns before the gutter\n{frame}"
    );
    for x in [PHONE_COLS - 2, PHONE_COLS - 1] {
        assert_eq!(
            buf.cell((x, y)).unwrap().bg,
            theme.bg_light,
            "column {x} past the wide glyph carries the band\n{frame}"
        );
    }
}

/// Compact prompt mode keeps the echo's fractional pad rows on a phone pane:
/// without them the text sits on the band's top and bottom edges.
#[test]
fn phone_compact_echo_keeps_its_fractional_pads() {
    let _guard = crate::theme::cache::pin_theme();
    let theme = Theme::current();
    let mut agent = phone_agent();
    seed_prompt_echo(&mut agent, "prompt-c 지금 전반적으로 구현 다 잘 됐어???");
    set_compact(&mut agent, true);
    let buf = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
    let frame = frame_text(&buf);
    let text_y = rows_with(&buf, "prompt-c")
        .first()
        .copied()
        .unwrap_or_else(|| panic!("the echo is on screen\n{frame}"));
    let band = echo_band_rows(&buf, theme.bg_light);
    let pads: Vec<&str> = band
        .iter()
        .filter(|&&y| y != text_y)
        .map(|&y| buf.cell((1, y)).unwrap().symbol())
        .collect();
    assert!(
        pads.contains(&"\u{2582}") && pads.contains(&"\u{2586}"),
        "the echo keeps a lower-eighths pad above and an upper pad below, got {pads:?} on rows {band:?}\n{frame}"
    );
}

/// Selecting the pinned echo in compact mode keeps its box inside the pane. A
/// compact echo has no pad row to pull the box onto, so an unclipped top would
/// draw its corners on the status bar's row.
#[test]
fn phone_compact_pinned_echo_selection_leaves_the_status_bar_alone() {
    let _guard = crate::theme::cache::pin_theme();
    let mut agent = phone_agent();
    seed_scrolling_turns(&mut agent);
    set_compact(&mut agent, true);
    let plain = scroll_into_middle_answer(&mut agent, PHONE_COLS, PHONE_ROWS);
    agent.active_pane = super::ActivePane::Scrollback;
    agent.scrollback.set_selected(Some(2));
    let selected = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
    assert_eq!(
        agent
            .scrollback
            .sticky_layout()
            .and_then(|sticky| sticky.pinned)
            .map(|pinned| pinned.entry_idx),
        Some(2),
        "the selected echo is the pinned one\n{}",
        frame_text(&selected)
    );
    assert_eq!(
        row_text(&selected, 0),
        row_text(&plain, 0),
        "the status bar row keeps its text\n{}",
        frame_text(&selected)
    );
}

/// With the scrollbar drawn, the echo's band fills every column up to the bar —
/// the transcript's own width plus the held-copy gutter the transcript leaves
/// blank — in the flow and pinned alike, and the bar keeps its own column.
#[test]
fn phone_echo_band_fills_up_to_the_scrollbar() {
    let _guard = crate::theme::cache::pin_theme();
    let theme = Theme::current();
    let mut agent = phone_agent();
    seed_scrolling_turns(&mut agent);
    let _ = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
    let descriptors = agent
        .scrollback
        .get_cached_prompt_descriptors()
        .expect("prompt descriptors")
        .to_vec();
    // Pinned (inside the middle answer), then flowing (the last prompt a few
    // rows under the viewport top).
    let pinned_frame = scroll_into_middle_answer(&mut agent, PHONE_COLS, PHONE_ROWS);
    agent
        .scrollback
        .set_scroll_offset(descriptors[2].y_virtual.saturating_sub(8));
    let flow_frame = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
    for (label, buf, marker) in [
        ("pinned", &pinned_frame, "prompt-1"),
        ("flow", &flow_frame, "prompt-2"),
    ] {
        let frame = frame_text(buf);
        let text_rows = rows_with(buf, marker);
        assert_eq!(
            text_rows.len(),
            1,
            "{label}: the echo is on screen\n{frame}"
        );
        let y = text_rows[0];
        let bar = PHONE_COLS - 1;
        // The left edge, and the right edge up to the bar: the band's own right
        // pad, then the held-copy gutter the transcript leaves blank. A wide
        // glyph's second cell inside the text keeps the buffer's default style.
        for x in [0, bar - 3, bar - 2, bar - 1] {
            assert_eq!(
                buf.cell((x, y)).unwrap().bg,
                theme.bg_light,
                "{label}: band column {x} of row {y} reaches the scrollbar\n{frame}"
            );
        }
        // The thumb's colour can match the band's, so only its glyph is checked.
        let bar_cell = buf.cell((bar, y)).unwrap();
        let track = bar_cell.symbol() == " " && bar_cell.bg == theme.scrollbar_bg;
        assert!(
            track || bar_cell.symbol() == "\u{2588}",
            "{label}: the scrollbar keeps its column, got {:?}\n{frame}",
            (bar_cell.symbol(), bar_cell.bg)
        );
    }
}

/// The scrollbar column only ever holds the bar: no transcript background (a
/// code block's shading, a text row's fill) and no pad glyph is copied into it
/// while the transcript scrolls under it.
#[test]
fn phone_scrollbar_column_holds_only_the_bar_while_scrolling() {
    let _guard = crate::theme::cache::pin_theme();
    let theme = Theme::current();
    let mut agent = phone_agent();
    seed_scrolling_turns(&mut agent);
    let _ = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
    let (_, viewport, total) = agent.scrollback.scroll_info();
    let bar = PHONE_COLS - 1;
    for offset in (0..total.saturating_sub(usize::from(viewport))).step_by(3) {
        agent.scrollback.set_scroll_offset(offset);
        let buf = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
        // The scrollback's rows: under the status bar, as tall as its viewport.
        for y in 1..1 + viewport {
            let cell = buf.cell((bar, y)).unwrap();
            let track = cell.symbol() == " " && cell.bg == theme.scrollbar_bg;
            let thumb = cell.symbol() == "\u{2588}";
            assert!(
                track || thumb,
                "offset {offset}: scrollbar row {y} holds {:?}\n{}",
                (cell.symbol(), cell.fg, cell.bg),
                frame_text(&buf)
            );
        }
    }
}

/// [`draw`], keeping the caret the frame puts on screen.
fn draw_with_caret(agent: &mut AgentView, cols: u16, rows: u16) -> (Buffer, Option<(u16, u16)>) {
    agent.last_terminal_size = (cols, rows);
    let mut appearance = agent.scrollback.appearance().clone();
    appearance.scrollback.layout.narrow = crate::views::agent::effective_narrow(cols, rows);
    agent.scrollback.set_appearance(appearance);
    let area = Rect::new(0, 0, cols, rows);
    let mut buf = Buffer::empty(area);
    let mut scratch = ScratchBuffer::new();
    let registry = ActionRegistry::defaults();
    let (caret, _) = agent.draw(
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
    (buf, caret)
}

/// Type `text` into the composer one key at a time, the caret following it.
fn type_keys(agent: &mut AgentView, text: &str) {
    for ch in text.chars() {
        agent.prompt.handle_key(&crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(ch),
            crossterm::event::KeyModifiers::NONE,
        ));
    }
}

/// A draft of exactly `cols` columns: words of four `unit` glyphs with a space
/// between them, and a one-column `.` closing an odd count of two-column
/// glyphs.
fn full_row_draft(unit: char, cols: usize) -> String {
    let w = unicode_width::UnicodeWidthChar::width(unit).unwrap_or(1);
    let (mut draft, mut used, mut in_word) = (String::new(), 0, 0);
    while used + w <= cols {
        if in_word == 4 && used + 1 + w <= cols {
            draft.push(' ');
            used += 1;
            in_word = 0;
        }
        draft.push(unit);
        used += w;
        in_word += 1;
    }
    if used < cols {
        draft.push('.');
    }
    draft
}

/// The iPhone report: typing to the end of the composer's text row opened an
/// empty second row, the caret dropped onto it, and the row it left still had
/// its right pad free. The phone draws the composer across the frame but
/// measured it two columns narrower, so the box asked for the second row before
/// the drawn row was full; and a full row sent the caret down before any text
/// needed the room. A full row keeps one text row and the caret after its last
/// glyph; the next character opens the second row — in English and Hangul.
#[test]
fn phone_composer_opens_a_row_only_for_the_character_after_a_full_row() {
    let _guard = crate::theme::cache::pin_theme();
    for (label, unit, next) in [("English", 'x', 's'), ("Hangul", '가', '요')] {
        let mut agent = phone_agent();
        let _ = draw(&mut agent, PHONE_COLS, PHONE_ROWS);
        let text_cols = usize::from(agent.prompt.textarea_area().width);
        let draft = full_row_draft(unit, text_cols);
        assert_eq!(
            unicode_width::UnicodeWidthStr::width(draft.as_str()),
            text_cols,
            "{label}: the draft fills the drawn text row: {draft:?}"
        );
        type_keys(&mut agent, &draft);

        let (buf, caret) = draw_with_caret(&mut agent, PHONE_COLS, PHONE_ROWS);
        let frame = frame_text(&buf);
        let (top, bottom) = composer_band_rows(&buf);
        let text = agent.prompt.textarea_area();
        assert_eq!(
            bottom - top - 1,
            1,
            "{label}: a full row is still one text row:\n{frame}"
        );
        assert_eq!(
            caret,
            Some((text.right(), top + 1)),
            "{label}: the caret stays after the last glyph of the full row:\n{frame}"
        );
        assert!(
            text.right() < PHONE_COLS,
            "{label}: that column is the band's right pad:\n{frame}"
        );

        type_keys(&mut agent, &next.to_string());
        let (buf, caret) = draw_with_caret(&mut agent, PHONE_COLS, PHONE_ROWS);
        let frame = frame_text(&buf);
        let (top, bottom) = composer_band_rows(&buf);
        assert_eq!(
            bottom - top - 1,
            2,
            "{label}: the next character opens the second row:\n{frame}"
        );
        assert_eq!(
            caret.map(|(_, y)| y),
            Some(top + 2),
            "{label}: the caret follows it down:\n{frame}"
        );
    }
}
