//! DeepSeekNight themes — product and compatibility skins for DeepSeek Build.
//!
//! Two legacy-compatible dark themes share the **DeepSeek blue `#4D6BFE`**
//! accent (user prompt, system, skill, fuzzy, selection) and differ only in
//! the background/gray ramp:
//!
//! - [`Theme::deepseeknight`] — **DeepSeek Night**: blue-tinted dark chrome,
//!   the original dsb signature look.
//! - [`Theme::deepseeknight_neutral`] — **DeepSeek Night Neutral**:
//!   hue-neutral ramp (r≈g≈b) at the same luminance, so the blue accent
//!   reads as a deliberate highlight instead of a wash. Retained for
//!   legacy/config compatibility; the current product default is the classic
//!   blue-tinted skin.
//!
//! Blue-on-dark is the worst hue for small-text legibility (chromatic
//! aberration, low blue luminance), so the neutral variant keeps the gray
//! ramp hue-free while the blue stays reserved for accents and borders.

use ratatui::style::{Color, Modifier};

use super::tokyonight::Theme;

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

/// Official DeepSeek product accent `#4D6BFE`.
pub const DEEPSEEK_BLUE: Color = rgb(77, 107, 254);
const DEEPSEEK_BLUE_BRIGHT: Color = rgb(110, 140, 255);
const DEEPSEEK_BLUE_DIM: Color = rgb(50, 72, 190);

#[allow(dead_code)]
mod palette {
    use super::*;

    // Blue-tinted ramp (DeepSeek Night). The cool cast makes the whole
    // surface read "DeepSeek". The dark-navy ground stays and the glyph
    // ramp is lifted instead: the emphasis/body L* gap holds its original
    // size, and the tool-row gray brightens while keeping its blue cast.
    pub const BG: Color = rgb(8, 10, 21); // #080A15
    pub const BG_DARK: Color = rgb(9, 12, 24); // #090C18
    pub const BG_STORM_DARK: Color = rgb(14, 17, 29); // #0E111D
    pub const BG_STORM: Color = rgb(14, 20, 37); // #0E1425
    pub const BG_HIGHLIGHT: Color = rgb(26, 36, 62); // #1A243E
    pub const FG: Color = rgb(249, 249, 252); // #F9F9FC
    pub const FG_DARK: Color = rgb(215, 217, 231); // #D7D9E7
    pub const FG_GUTTER: Color = rgb(71, 77, 100); // #474D64
    pub const COMMENT: Color = rgb(134, 141, 172); // #868DAC
    pub const DARK3: Color = rgb(104, 110, 137); // #686E89
    pub const DARK5: Color = rgb(164, 170, 199); // #A4AAC7

    // Per-field blue-ramp values that were previously inlined.
    pub const BG_SURFACE: Color = rgb(19, 26, 45); // #131A2D — bg_dark
    pub const BG_HOVER: Color = rgb(34, 44, 72); // #222C48
    pub const BG_VISUAL: Color = rgb(32, 46, 80); // #202E50
    pub const MD_CODE_BG: Color = rgb(22, 30, 52); // #161E34
    pub const HOVER_BORDER: Color = rgb(25, 34, 58); // #19223A
    pub const PROMPT_BORDER: Color = rgb(41, 54, 89); // #293659

    // Hue-neutral ramp (DeepSeek Night Neutral) — same luminance as the
    // blue ramp but r≈g≈b (blue channel at most a couple of levels for a
    // barely-cool canvas). A neutral ground makes the blue accent pop as a
    // deliberate highlight and maximizes gray-ramp contrast.
    pub const BG_N: Color = rgb(12, 12, 12);
    pub const BG_DARK_N: Color = rgb(14, 14, 14);
    pub const BG_STORM_DARK_N: Color = rgb(18, 18, 18);
    pub const BG_STORM_N: Color = rgb(22, 22, 24);
    pub const BG_HIGHLIGHT_N: Color = rgb(36, 36, 38);
    pub const FG_N: Color = rgb(232, 232, 234);
    pub const FG_DARK_N: Color = rgb(198, 198, 200);
    pub const FG_GUTTER_N: Color = rgb(72, 72, 76);
    pub const COMMENT_N: Color = rgb(112, 112, 116);
    pub const DARK3_N: Color = rgb(92, 92, 96);
    pub const DARK5_N: Color = rgb(132, 132, 136);

    // Per-field neutral-ramp values that were previously inlined.
    pub const BG_SURFACE_N: Color = rgb(26, 26, 28); // bg_dark
    pub const BG_HOVER_N: Color = rgb(44, 44, 46);
    pub const BG_VISUAL_N: Color = rgb(46, 46, 48);
    pub const MD_CODE_BG_N: Color = rgb(30, 30, 32);
    pub const HOVER_BORDER_N: Color = rgb(33, 33, 35);
    pub const PROMPT_BORDER_N: Color = rgb(54, 54, 56);

    // Shared accents — identical across both DeepSeek themes.
    pub const BLUE1: Color = rgb(90, 160, 220);
    pub const GREEN: Color = rgb(120, 210, 160);
    pub const GREEN1: Color = rgb(100, 200, 180);
    pub const RED: Color = rgb(250, 120, 140);
    pub const YELLOW: Color = rgb(230, 190, 110);
    pub const ORANGE: Color = rgb(255, 170, 110);
    pub const CYAN: Color = rgb(120, 210, 240);
    pub const MAGENTA: Color = rgb(180, 160, 250);
    pub const TEAL: Color = rgb(60, 190, 180);
    pub const PURPLE: Color = rgb(160, 140, 230);

    pub const RED_DARK: Color = rgb(66, 14, 20);
    pub const GREEN_DARK: Color = rgb(6, 56, 20);
}
use palette::*;

impl Theme {
    /// Legacy-compatible DeepSeek Night theme (`#4D6BFE` accents, blue-tinted ramp).
    pub const fn deepseeknight() -> Self {
        Self::deepseeknight_inner(false)
    }

    /// Legacy-compatible DeepSeek Night theme with a neutral ramp (`#4D6BFE` accents).
    ///
    /// Retained for direct config and alias compatibility; the current
    /// product default is [`super::Theme::deepseeknight`].
    pub const fn deepseeknight_neutral() -> Self {
        Self::deepseeknight_inner(true)
    }

    /// Shared constructor for the two DeepSeek skins.
    ///
    /// `neutral: true` selects the hue-neutral gray ramp, `false` the
    /// blue-tinted one. Accents are identical either way.
    const fn deepseeknight_inner(neutral: bool) -> Self {
        Self {
            bg_base: if neutral { BG_STORM_N } else { BG_STORM },
            bg_light: if neutral {
                BG_HIGHLIGHT_N
            } else {
                BG_HIGHLIGHT
            },
            bg_dark: if neutral { BG_SURFACE_N } else { BG_SURFACE },
            bg_highlight: if neutral {
                BG_HIGHLIGHT_N
            } else {
                BG_HIGHLIGHT
            },
            bg_hover: if neutral { BG_HOVER_N } else { BG_HOVER },
            bg_terminal: if neutral { BG_N } else { BG },

            accent_user: DEEPSEEK_BLUE,
            accent_assistant: DEEPSEEK_BLUE_BRIGHT,
            accent_thinking: MAGENTA,
            accent_tool: DARK5,
            accent_system: DEEPSEEK_BLUE,
            accent_error: RED,
            accent_success: GREEN,
            accent_running: DEEPSEEK_BLUE_BRIGHT,
            accent_skill: DEEPSEEK_BLUE,

            text_primary: if neutral { FG_N } else { FG },
            text_secondary: if neutral { FG_DARK_N } else { FG_DARK },

            gray_dim: if neutral { FG_GUTTER_N } else { FG_GUTTER },
            gray: if neutral { COMMENT_N } else { COMMENT },
            gray_bright: if neutral { DARK5_N } else { DARK5 },

            command: YELLOW,
            path: ORANGE,
            running: CYAN,
            warning: YELLOW,

            fuzzy_accent: DEEPSEEK_BLUE,

            accent_plan: rgb(255, 219, 141),
            accent_verify: MAGENTA,
            accent_remember: Color::Rgb(139, 195, 74),

            selection_border: DEEPSEEK_BLUE_DIM,
            hover_border: if neutral {
                HOVER_BORDER_N
            } else {
                HOVER_BORDER
            },
            prompt_border: if neutral {
                PROMPT_BORDER_N
            } else {
                PROMPT_BORDER
            },
            // Focused composer border: one step below the official blue so the
            // box sits behind the content (2026-09-27 tone decision, see
            // docs/product/THEME_CLASSIC_READABILITY_2026-09-27.md).
            prompt_border_active: DEEPSEEK_BLUE_DIM,

            accent_model: TEAL,

            scrollbar_bg: if neutral {
                BG_STORM_DARK_N
            } else {
                BG_STORM_DARK
            },
            scrollbar_fg: if neutral {
                BG_HIGHLIGHT_N
            } else {
                BG_HIGHLIGHT
            },

            diff_delete_bg: RED_DARK,
            diff_delete_fg: RED,
            diff_insert_bg: GREEN_DARK,
            diff_insert_fg: GREEN,
            diff_equal_fg: COMMENT,
            diff_gutter_fg: COMMENT,

            bg_visual: if neutral { BG_VISUAL_N } else { BG_VISUAL },

            paste_bg: if neutral {
                BG_STORM_DARK_N
            } else {
                BG_STORM_DARK
            },
            paste_fg: if neutral { FG_DARK_N } else { FG_DARK },
            paste_dim: if neutral { FG_GUTTER_N } else { FG_GUTTER },

            md_heading_h1: DEEPSEEK_BLUE_BRIGHT,
            md_heading_h1_mod: Modifier::BOLD,
            md_heading_h2: DEEPSEEK_BLUE,
            md_heading_h2_mod: Modifier::BOLD,
            md_heading_h3: PURPLE,
            md_heading_h3_mod: Modifier::BOLD,
            md_heading_h4: DARK5,
            md_heading_h4_mod: Modifier::BOLD,
            md_heading_h5: COMMENT,
            md_heading_h5_mod: Modifier::BOLD,
            md_heading_h6: DARK3,
            md_heading_h6_mod: Modifier::empty(),
            md_code: BLUE1,
            md_task_checked: GREEN,
            md_task_unchecked: FG_DARK,
            md_muted: COMMENT,
            md_code_bg: if neutral { MD_CODE_BG_N } else { MD_CODE_BG },
            md_text: FG_DARK,
            link_fg: DEEPSEEK_BLUE_BRIGHT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deepseek_blue_is_official() {
        assert!(matches!(DEEPSEEK_BLUE, Color::Rgb(77, 107, 254)));
        let t = Theme::deepseeknight();
        assert!(matches!(t.accent_user, Color::Rgb(77, 107, 254)));
        assert!(matches!(t.accent_system, Color::Rgb(77, 107, 254)));
        // The focused composer border reuses the dim token (tone decision
        // 2026-09-27); the official blue stays on the accents above.
        assert!(matches!(t.prompt_border_active, Color::Rgb(50, 72, 190)));
    }

    #[test]
    fn neutral_theme_keeps_official_blue_accent() {
        let t = Theme::deepseeknight_neutral();
        assert!(matches!(t.accent_user, Color::Rgb(77, 107, 254)));
        assert!(matches!(t.accent_system, Color::Rgb(77, 107, 254)));
        assert!(matches!(t.prompt_border_active, Color::Rgb(50, 72, 190)));
        assert!(matches!(t.selection_border, Color::Rgb(50, 72, 190)));
    }

    #[test]
    fn neutral_theme_ramp_is_hue_neutral() {
        // Blue channel sits at most a couple of levels above red/green — a
        // barely-cool canvas, not a tinted one.
        let t = Theme::deepseeknight_neutral();
        for (name, c) in [
            ("bg_base", t.bg_base),
            ("bg_light", t.bg_light),
            ("bg_dark", t.bg_dark),
            ("bg_hover", t.bg_hover),
            ("bg_visual", t.bg_visual),
            ("md_code_bg", t.md_code_bg),
            ("text_primary", t.text_primary),
            ("gray", t.gray),
        ] {
            let Color::Rgb(r, g, b) = c else {
                panic!("{name} must be Color::Rgb, got {c:?}");
            };
            assert!(
                r.abs_diff(g) <= 1 && b.abs_diff(r) <= 4,
                "{name} not hue-neutral: {r},{g},{b}"
            );
        }
    }

    #[test]
    fn blue_theme_ramp_is_blue_tinted() {
        // The blue variant keeps its cool cast: backgrounds carry blue
        // channel well above red/green.
        let t = Theme::deepseeknight();
        let Color::Rgb(r, g, b) = t.bg_base else {
            panic!("bg_base must be Color::Rgb, got {:?}", t.bg_base);
        };
        assert!(
            b > r + 5 && b > g + 5,
            "blue theme bg_base should be blue-tinted: {r},{g},{b}"
        );
    }

    #[test]
    fn blue_and_neutral_themes_differ_only_in_ramp() {
        let blue = Theme::deepseeknight();
        let neutral = Theme::deepseeknight_neutral();
        assert_ne!(blue.bg_base, neutral.bg_base);
        // Accents are identical — the ramp is the only difference.
        assert_eq!(blue.accent_user, neutral.accent_user);
        assert_eq!(blue.accent_assistant, neutral.accent_assistant);
        assert_eq!(blue.prompt_border_active, neutral.prompt_border_active);
        assert_eq!(blue.accent_thinking, neutral.accent_thinking);
        assert_eq!(blue.command, neutral.command);
    }

    // ---- colorimetry helpers (mirrors `deepseeknight_v2` tests) ----------

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

    #[test]
    fn classic_glyph_ramp_meets_contrast_floors_on_bg_base() {
        // The tool-row gray used to sit at WCAG 3.97, below the AA floor;
        // `gray_dim` stays rule-only on purpose.
        let t = Theme::deepseeknight();
        let bg = t.bg_base;
        for (name, c, floor) in [
            ("text_primary", t.text_primary, 16.0),
            ("md_text", t.md_text, 12.0),
            ("gray_bright", t.gray_bright, 7.5),
            ("gray", t.gray, 5.2),
        ] {
            let cr = contrast(c, bg);
            assert!(
                cr >= floor,
                "{name} contrast {cr:.2} on bg_base is below {floor}"
            );
        }
        let dim = contrast(t.gray_dim, bg);
        assert!(dim <= 3.0, "gray_dim must stay rule-only, got {dim:.2}");
    }

    #[test]
    fn classic_glyph_ramp_keeps_l_star_hierarchy() {
        // The rejected first candidate raised the background and collapsed
        // emphasis and body into one white; these gaps keep the hierarchy
        // the classic theme has always had.
        let t = Theme::deepseeknight();
        let primary = l_star(t.text_primary);
        let body = l_star(t.md_text);
        let gray = l_star(t.gray);
        assert!(
            primary - body >= 10.0,
            "text_primary({primary:.1}) - md_text({body:.1}) is under 10"
        );
        assert!(
            body - gray >= 25.0,
            "md_text({body:.1}) - gray({gray:.1}) is under 25"
        );
    }
}
