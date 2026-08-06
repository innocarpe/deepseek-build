//! DeepSeekNight theme — product default for DeepSeek Build.
//!
//! Neutral dark chrome with **DeepSeek blue `#4D6BFE`** as the primary accent
//! (user prompt, system, skill, fuzzy, selection). Replaces GrokNight as the
//! default product skin while keeping readable contrast.

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
    /// DeepSeek Build product night theme (`#4D6BFE` accents).
    pub const fn deepseeknight() -> Self {
        Self {
            bg_base: BG_STORM,
            bg_light: BG_HIGHLIGHT,
            bg_dark: rgb(24, 26, 36),
            bg_highlight: BG_HIGHLIGHT,
            bg_hover: rgb(40, 44, 62),
            bg_terminal: BG,

            accent_user: DEEPSEEK_BLUE,
            accent_assistant: DEEPSEEK_BLUE_BRIGHT,
            accent_thinking: MAGENTA,
            accent_tool: DARK5,
            accent_system: DEEPSEEK_BLUE,
            accent_error: RED,
            accent_success: GREEN,
            accent_running: DEEPSEEK_BLUE_BRIGHT,
            accent_skill: DEEPSEEK_BLUE,

            text_primary: FG,
            text_secondary: FG_DARK,

            gray_dim: FG_GUTTER,
            gray: COMMENT,
            gray_bright: DARK5,

            command: YELLOW,
            path: ORANGE,
            running: CYAN,
            warning: YELLOW,

            fuzzy_accent: DEEPSEEK_BLUE,

            accent_plan: rgb(255, 219, 141),
            accent_verify: MAGENTA,
            accent_feedback: GREEN1,
            accent_remember: Color::Rgb(139, 195, 74),

            selection_border: DEEPSEEK_BLUE_DIM,
            hover_border: rgb(30, 34, 48),
            prompt_border: rgb(48, 54, 78),
            prompt_border_active: DEEPSEEK_BLUE,

            accent_model: TEAL,

            scrollbar_bg: BG_STORM_DARK,
            scrollbar_fg: BG_HIGHLIGHT,

            diff_delete_bg: RED_DARK,
            diff_delete_fg: RED,
            diff_insert_bg: GREEN_DARK,
            diff_insert_fg: GREEN,
            diff_equal_fg: COMMENT,
            diff_gutter_fg: COMMENT,

            bg_visual: rgb(40, 46, 70),

            paste_bg: BG_STORM_DARK,
            paste_fg: FG_DARK,
            paste_dim: FG_GUTTER,

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
            md_code_bg: rgb(28, 30, 42),
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
}
