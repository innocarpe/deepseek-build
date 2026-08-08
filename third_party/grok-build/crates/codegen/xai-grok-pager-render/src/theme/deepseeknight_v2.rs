//! DeepSeekNight v2 — measured C-balanced palette.
//!
//! Design invariants I1..I9 are enforced by the tests at the bottom of this
//! file. See `docs/product/THEME_V2_REPORT.md` for the measurements and
//! `docs/superpowers/plans/2026-08-07-deepseek-theme-v2.md` for the rules.

use ratatui::style::{Color, Modifier};

use super::tokyonight::Theme;

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

/// Official DeepSeek product accent — value unchanged from v1.
/// v2 restricts it to lines, borders and fills (4.46:1 is below the AA floor).
#[allow(dead_code)] // public API for product branding / future chrome
pub const DEEPSEEK_BLUE_V2: Color = rgb(77, 107, 254);

// Brand ramp — one hue family (227-231 deg), four lightness steps.
#[allow(dead_code)]
const BLUE_DIM: Color = rgb(50, 72, 190); //  #3248BE  2.58:1  line/fill
const BLUE: Color = rgb(77, 107, 254); //     #4D6BFE  4.46:1  line/fill, official
#[allow(dead_code)]
const BLUE_MID: Color = rgb(103, 128, 254); // #6780FE  5.60:1  text (reserved)
const BLUE_TEXT: Color = rgb(126, 154, 255); // #7E9AFF  7.31:1  text

// Semantic colors — mutually >= 38 deg apart.
const VIOLET: Color = rgb(185, 140, 245); //  #B98CF5  7.47:1  reasoning
const GREEN: Color = rgb(95, 211, 155); //    #5FD39B 10.37:1  success/insert
const RED: Color = rgb(242, 112, 138); //     #F2708A  6.86:1  error/delete
const AMBER: Color = rgb(232, 183, 95); //    #E8B75F 10.45:1  running/warning

// Command accent — v1's YELLOW restored. It sits adjacent to AMBER on
// purpose (v1 rendered command and warning in the same yellow), so it is
// NOT part of the four status hues above. Path now lives in the blue ramp,
// which removes the v1 yellow-orange collision that forced `command: FG`.
const YELLOW: Color = rgb(230, 190, 110); //  #E6BE6E 10.99:1  command

// Surfaces — even L* ladder, chroma held at 3-5.
const BG_TERMINAL: Color = rgb(8, 9, 11); //  #08090B  L*  2.45
#[allow(dead_code)]
const BG_PROMPT: Color = rgb(10, 11, 13); //  #0A0B0D  L*  3.01
const BG_BASE: Color = rgb(13, 14, 17); //    #0D0E11  L*  3.98
const BG_DARK: Color = rgb(19, 20, 24); //    #131418  L*  6.37
const BG_RAISED: Color = rgb(24, 25, 28); //  #18191C  L*  8.77
const BG_HOVER: Color = rgb(35, 36, 39); //   #232427  L* 14.21  lightness axis
const BG_VISUAL: Color = rgb(26, 34, 64); //  #1A2240  L* 14.02  chroma axis (38)

// Gray ramp — AA on both bg_base and bg_raised.
const FG: Color = rgb(226, 227, 230); //      #E2E3E6  15.04 / 13.70
const FG_DARK: Color = rgb(182, 183, 186); // #B6B7BA   9.62 /  8.77
const GRAY_BRIGHT: Color = rgb(147, 148, 151); // #939497  6.36 /  5.80
const GRAY: Color = rgb(128, 129, 132); //    #808184   4.95 /  4.51
const GRAY_DIM: Color = rgb(84, 85, 88); //   #545558   2.59 /  2.36  RULES ONLY

// Structure.
const BORDER: Color = rgb(46, 48, 54); //     #2E3036
const SCROLL_FG: Color = rgb(58, 59, 64); //  #3A3B40
const DIFF_ADD_BG: Color = rgb(11, 58, 34); // #0B3A22  GREEN reads 6.87:1
const DIFF_DEL_BG: Color = rgb(69, 19, 31); // #45131F  RED reads 5.46:1

impl Theme {
    /// DeepSeek Build product theme v2 — measured C-balanced palette.
    ///
    /// Blue carries 12 roles: identity / "you can go here" affordances plus
    /// the markdown hierarchy (h1·h2 hue-marked, code). Everything else is
    /// the gray ramp, four semantic status hues, and the v1 command accent.
    pub const fn deepseeknight_v2() -> Self {
        Self {
            // Backgrounds
            bg_base: BG_BASE,
            bg_light: BG_RAISED,
            bg_dark: BG_DARK,
            bg_highlight: BG_RAISED,
            bg_hover: BG_HOVER,
            bg_terminal: BG_TERMINAL,
            bg_visual: BG_VISUAL,

            // Accents — blue marks identity, violet marks reasoning
            accent_user: BLUE,
            accent_assistant: BLUE,
            accent_system: BLUE,
            accent_skill: BLUE,
            accent_thinking: VIOLET,
            accent_tool: GRAY_BRIGHT,
            accent_error: RED,
            accent_success: GREEN,
            accent_running: AMBER,

            // Text
            text_primary: FG,
            text_secondary: FG_DARK,
            gray_dim: GRAY_DIM,
            gray: GRAY,
            gray_bright: GRAY_BRIGHT,

            // Semantic
            command: YELLOW,   // v1 YELLOW restore — path is BLUE_TEXT, no collision
            path: BLUE_TEXT,
            running: AMBER,
            warning: AMBER,

            fuzzy_accent: BLUE,

            accent_plan: AMBER,
            accent_verify: VIOLET,
            accent_remember: GREEN,
            accent_model: GRAY_BRIGHT,

            // Borders
            selection_border: BLUE,
            hover_border: BORDER,
            prompt_border: GRAY_DIM,
            prompt_border_active: BLUE,

            // Scrollbar
            scrollbar_bg: BG_RAISED,
            scrollbar_fg: SCROLL_FG,

            // Diff
            diff_delete_bg: DIFF_DEL_BG,
            diff_delete_fg: RED,
            diff_insert_bg: DIFF_ADD_BG,
            diff_insert_fg: GREEN,
            diff_equal_fg: GRAY,
            diff_gutter_fg: GRAY_DIM,

            // Paste
            paste_bg: BG_RAISED,
            paste_fg: FG_DARK,
            paste_dim: GRAY_DIM,

            // Markdown — h1·h2 are hue-marked (brand blue ramp), h3..h6 dim
            // monotonically. v1 had blue h2 and blue code; the AA-miss on
            // #4D6BFE forced the neutral ladder, which read as "all white".
            // BLUE_MID (5.60:1) and BLUE_TEXT (7.31:1) pass AA on both surfaces.
            md_heading_h1: BLUE_TEXT,
            md_heading_h1_mod: Modifier::BOLD,
            md_heading_h2: BLUE_MID,
            md_heading_h2_mod: Modifier::BOLD,
            md_heading_h3: GRAY_BRIGHT,
            md_heading_h3_mod: Modifier::BOLD,
            md_heading_h4: GRAY_BRIGHT,
            md_heading_h4_mod: Modifier::empty(),
            md_heading_h5: GRAY,
            md_heading_h5_mod: Modifier::BOLD,
            md_heading_h6: GRAY,
            md_heading_h6_mod: Modifier::empty(),
            md_code: BLUE_TEXT,
            md_code_bg: BG_RAISED,
            md_text: FG_DARK,
            md_task_checked: GREEN,
            md_task_unchecked: FG_DARK,
            md_muted: GRAY,
            link_fg: BLUE_TEXT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- colorimetry helpers -------------------------------------------

    fn rgb_of(c: Color) -> (f64, f64, f64) {
        match c {
            Color::Rgb(r, g, b) => (r as f64, g as f64, b as f64),
            other => panic!("expected Color::Rgb, got {other:?}"),
        }
    }

    fn lin(v: f64) -> f64 {
        let v = v / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }

    fn luminance(c: Color) -> f64 {
        let (r, g, b) = rgb_of(c);
        0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)
    }

    fn contrast(a: Color, b: Color) -> f64 {
        let (la, lb) = (luminance(a), luminance(b));
        let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
        (hi + 0.05) / (lo + 0.05)
    }

    fn l_star(c: Color) -> f64 {
        let y = luminance(c);
        let d = 6.0 / 29.0;
        let f = if y > d * d * d {
            y.cbrt()
        } else {
            y / (3.0 * d * d) + 4.0 / 29.0
        };
        116.0 * f - 16.0
    }

    fn hue_deg(c: Color) -> f64 {
        let (r, g, b) = rgb_of(c);
        let (r, g, b) = (r / 255.0, g / 255.0, b / 255.0);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        if delta == 0.0 {
            return 0.0;
        }
        let h = if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * ((b - r) / delta + 2.0)
        } else {
            60.0 * ((r - g) / delta + 4.0)
        };
        if h < 0.0 {
            h + 360.0
        } else {
            h
        }
    }

    fn hue_gap(a: Color, b: Color) -> f64 {
        let d = (hue_deg(a) - hue_deg(b)).abs();
        d.min(360.0 - d)
    }

    fn chroma(c: Color) -> f64 {
        let (r, g, b) = rgb_of(c);
        r.max(g).max(b) - r.min(g).min(b)
    }

    /// Every role that renders as **glyphs**. Lines, borders and fills are
    /// deliberately excluded — they are allowed to sit below the AA floor.
    fn text_roles(t: &Theme) -> Vec<(&'static str, Color)> {
        vec![
            ("text_primary", t.text_primary),
            ("text_secondary", t.text_secondary),
            ("gray_bright", t.gray_bright),
            ("gray", t.gray),
            ("command", t.command),
            ("path", t.path),
            ("running", t.running),
            ("warning", t.warning),
            ("accent_thinking", t.accent_thinking),
            ("accent_tool", t.accent_tool),
            ("accent_error", t.accent_error),
            ("accent_success", t.accent_success),
            ("accent_running", t.accent_running),
            ("accent_model", t.accent_model),
            ("accent_plan", t.accent_plan),
            ("accent_verify", t.accent_verify),
            ("accent_remember", t.accent_remember),
            ("md_heading_h1", t.md_heading_h1),
            ("md_heading_h2", t.md_heading_h2),
            ("md_heading_h3", t.md_heading_h3),
            ("md_heading_h4", t.md_heading_h4),
            ("md_heading_h5", t.md_heading_h5),
            ("md_heading_h6", t.md_heading_h6),
            ("md_code", t.md_code),
            ("md_text", t.md_text),
            ("md_muted", t.md_muted),
            ("md_task_checked", t.md_task_checked),
            ("md_task_unchecked", t.md_task_unchecked),
            ("link_fg", t.link_fg),
            ("paste_fg", t.paste_fg),
            ("diff_equal_fg", t.diff_equal_fg),
        ]
    }

    // ---- I4 -------------------------------------------------------------

    #[test]
    fn text_roles_pass_aa_on_base_and_raised() {
        let t = Theme::deepseeknight_v2();
        for (sname, surface) in [("bg_base", BG_BASE), ("bg_raised", BG_RAISED)] {
            for (rname, c) in text_roles(&t) {
                let cr = contrast(c, surface);
                assert!(cr >= 4.5, "{rname} on {sname}: {cr:.2}:1 < 4.5 (AA)");
            }
        }
    }

    // ---- I9 -------------------------------------------------------------

    #[test]
    fn official_blue_is_never_a_text_role() {
        // #4D6BFE measures 4.46:1 on bg_base — below the AA floor. It is a
        // line/border/fill color only; glyphs use BLUE_TEXT.
        let t = Theme::deepseeknight_v2();
        for (rname, c) in text_roles(&t) {
            assert_ne!(c, BLUE, "{rname} must not use the line-only official blue");
        }
    }

    // ---- I5 -------------------------------------------------------------

    #[test]
    fn gray_dim_is_never_a_text_role() {
        let t = Theme::deepseeknight_v2();
        for (rname, c) in text_roles(&t) {
            assert_ne!(c, GRAY_DIM, "{rname} must not use rule-only GRAY_DIM");
        }
    }

    // ---- I1 -------------------------------------------------------------

    #[test]
    fn semantic_roles_only_use_declared_palette() {
        let t = Theme::deepseeknight_v2();
        let allowed = [
            BLUE_DIM, BLUE, BLUE_MID, BLUE_TEXT, VIOLET, GREEN, RED, AMBER, YELLOW, FG, FG_DARK,
            GRAY_BRIGHT, GRAY, GRAY_DIM, BORDER,
        ];
        let mut roles = text_roles(&t);
        roles.extend([
            ("accent_user", t.accent_user),
            ("accent_assistant", t.accent_assistant),
            ("accent_system", t.accent_system),
            ("accent_skill", t.accent_skill),
            ("fuzzy_accent", t.fuzzy_accent),
            ("selection_border", t.selection_border),
            ("hover_border", t.hover_border),
            ("prompt_border", t.prompt_border),
            ("prompt_border_active", t.prompt_border_active),
            ("diff_insert_fg", t.diff_insert_fg),
            ("diff_delete_fg", t.diff_delete_fg),
            ("diff_gutter_fg", t.diff_gutter_fg),
            ("paste_dim", t.paste_dim),
        ]);
        for (rname, c) in roles {
            assert!(
                allowed.contains(&c),
                "{rname} uses an undeclared color {c:?}; add it to the palette \
                 or reuse an existing token"
            );
        }
    }

    // ---- I2 -------------------------------------------------------------

    #[test]
    fn semantic_hues_are_at_least_35_degrees_apart() {
        let sem = [
            ("BLUE_TEXT", BLUE_TEXT),
            ("VIOLET", VIOLET),
            ("GREEN", GREEN),
            ("RED", RED),
            ("AMBER", AMBER),
        ];
        for i in 0..sem.len() {
            for j in (i + 1)..sem.len() {
                let gap = hue_gap(sem[i].1, sem[j].1);
                assert!(
                    gap >= 35.0,
                    "{} vs {}: hue gap {gap:.1} deg < 35",
                    sem[i].0,
                    sem[j].0
                );
            }
        }
    }

    // ---- I3 -------------------------------------------------------------

    #[test]
    fn brand_ramp_is_a_single_hue_family() {
        let ramp = [BLUE_DIM, BLUE, BLUE_MID, BLUE_TEXT];
        let hues: Vec<f64> = ramp.iter().map(|c| hue_deg(*c)).collect();
        let max = hues.iter().cloned().fold(f64::MIN, f64::max);
        let min = hues.iter().cloned().fold(f64::MAX, f64::min);
        assert!(
            max - min <= 6.0,
            "brand ramp hue spread {:.1} deg > 6 — it is no longer one family",
            max - min
        );
    }

    // ---- I6 -------------------------------------------------------------

    #[test]
    fn surface_ladder_steps_are_visible() {
        let ladder = [BG_BASE, BG_DARK, BG_RAISED, BG_HOVER];
        for pair in ladder.windows(2) {
            let step = l_star(pair[1]) - l_star(pair[0]);
            assert!(step >= 2.3, "surface step dL* {step:.2} < 2.3");
        }
    }

    // ---- I7 -------------------------------------------------------------

    #[test]
    fn hover_and_visual_separate_on_different_axes() {
        let t = Theme::deepseeknight_v2();
        let dc = chroma(t.bg_visual) - chroma(t.bg_hover);
        assert!(dc >= 30.0, "bg_visual vs bg_hover chroma delta {dc:.0} < 30");
        assert!(
            hue_gap(t.bg_visual, BLUE) <= 6.0,
            "the selection tint must sit in the brand hue family"
        );
    }

    // ---- I8 -------------------------------------------------------------

    #[test]
    fn heading_ladder_is_non_increasing_below_h2() {
        let t = Theme::deepseeknight_v2();
        // h1 and h2 are hue-marked (brand blue ramp); they are NOT part of
        // the luminance ladder (BLUE_TEXT 7.31 / BLUE_MID 5.60 sit below
        // FG 15.04 on purpose — hue, not brightness, marks the hierarchy).
        assert_eq!(t.md_heading_h1, BLUE_TEXT, "h1 must be the brand text blue");
        assert_eq!(t.md_heading_h2, BLUE_MID, "h2 must be the brand mid blue");
        let tail = [
            ("h3", t.md_heading_h3),
            ("h4", t.md_heading_h4),
            ("h5", t.md_heading_h5),
            ("h6", t.md_heading_h6),
        ];
        for pair in tail.windows(2) {
            let (an, a) = pair[0];
            let (bn, b) = pair[1];
            assert!(
                contrast(a, BG_BASE) >= contrast(b, BG_BASE) - 1e-9,
                "{an} ({:.2}) must not be dimmer than {bn} ({:.2})",
                contrast(a, BG_BASE),
                contrast(b, BG_BASE)
            );
        }
    }

    // ---- composite surfaces --------------------------------------------

    #[test]
    fn text_reads_on_selection_and_diff_surfaces() {
        let t = Theme::deepseeknight_v2();
        for (label, fg, bg) in [
            ("primary on selection", t.text_primary, t.bg_visual),
            ("primary on hover", t.text_primary, t.bg_hover),
            ("diff insert", t.diff_insert_fg, t.diff_insert_bg),
            ("diff delete", t.diff_delete_fg, t.diff_delete_bg),
        ] {
            let cr = contrast(fg, bg);
            assert!(cr >= 4.5, "{label}: {cr:.2}:1 < 4.5");
        }
    }

    #[test]
    fn official_blue_value_is_unchanged() {
        assert!(matches!(BLUE, Color::Rgb(77, 107, 254)));
        assert!(matches!(DEEPSEEK_BLUE_V2, Color::Rgb(77, 107, 254)));
    }
}
