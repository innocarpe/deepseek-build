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
    // surface read "DeepSeek"; slightly lower small-text legibility.
    pub const BG: Color = rgb(10, 10, 14);
    pub const BG_DARK: Color = rgb(12, 12, 18);
    pub const BG_STORM_DARK: Color = rgb(16, 17, 24);
    pub const BG_STORM: Color = rgb(18, 20, 28);
    pub const BG_HIGHLIGHT: Color = rgb(32, 36, 52);
    pub const FG: Color = rgb(232, 234, 246);
    pub const FG_DARK: Color = rgb(196, 200, 220);
    pub const FG_GUTTER: Color = rgb(70, 74, 96);
    pub const COMMENT: Color = rgb(110, 116, 140);
    pub const DARK3: Color = rgb(90, 96, 120);
    pub const DARK5: Color = rgb(130, 136, 160);

    // Per-field blue-ramp values that were previously inlined.
    pub const BG_SURFACE: Color = rgb(24, 26, 36); // bg_dark
    pub const BG_HOVER: Color = rgb(40, 44, 62);
    pub const BG_VISUAL: Color = rgb(40, 46, 70);
    pub const MD_CODE_BG: Color = rgb(28, 30, 42);
    pub const HOVER_BORDER: Color = rgb(30, 34, 48);
    pub const PROMPT_BORDER: Color = rgb(48, 54, 78);

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
            prompt_border_active: DEEPSEEK_BLUE,

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
        assert!(matches!(t.prompt_border_active, Color::Rgb(77, 107, 254)));
    }

    #[test]
    fn neutral_theme_keeps_official_blue_accent() {
        let t = Theme::deepseeknight_neutral();
        assert!(matches!(t.accent_user, Color::Rgb(77, 107, 254)));
        assert!(matches!(t.accent_system, Color::Rgb(77, 107, 254)));
        assert!(matches!(t.prompt_border_active, Color::Rgb(77, 107, 254)));
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
}
