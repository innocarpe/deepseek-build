//! DeepSeek blue theme v1 — readable default (not Grok near-black monochrome).
//!
//! Normative product intent: MASTER_PLAN §5 / docs/product/DESIGN.md.
//! Roles: content, reasoning, tool, model, error, accent.

use std::io::{self, IsTerminal, Write};
use std::sync::OnceLock;

/// DeepSeek brand accent (approx. DeepSeek blue family).
pub const DEEPSEEK_BLUE_RGB: (u8, u8, u8) = (77, 107, 254);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Content,
    Reasoning,
    Tool,
    Model,
    Error,
    Accent,
    Warn,
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub enabled: bool,
}

impl Theme {
    /// Default product theme: color when stderr/stdout is a TTY and `NO_COLOR` is unset.
    pub fn default_readable() -> Self {
        let no_color = std::env::var_os("NO_COLOR").is_some();
        let tty = io::stdout().is_terminal() || io::stderr().is_terminal();
        Self {
            enabled: tty && !no_color,
        }
    }

    pub fn plain() -> Self {
        Self { enabled: false }
    }

    pub fn paint(&self, role: Role, text: &str) -> String {
        if !self.enabled || text.is_empty() {
            return text.to_string();
        }
        let (r, g, b) = role_rgb(role);
        format!("\x1b[38;2;{r};{g};{b}m{text}\x1b[0m")
    }

    pub fn paint_to(&self, out: &mut dyn Write, role: Role, text: &str) -> io::Result<()> {
        write!(out, "{}", self.paint(role, text))
    }
}

fn role_rgb(role: Role) -> (u8, u8, u8) {
    match role {
        // Readable light-terminal friendly defaults (not near-black monochrome).
        Role::Content => (232, 236, 242),  // near-white body
        Role::Reasoning => (148, 163, 184), // slate secondary
        Role::Tool => DEEPSEEK_BLUE_RGB,    // brand accent
        Role::Model => (99, 140, 255),      // lighter blue
        Role::Error => (248, 113, 113),     // soft red
        Role::Accent => DEEPSEEK_BLUE_RGB,
        Role::Warn => (251, 191, 36), // amber
    }
}

/// Process-wide default theme (lazy).
pub fn global() -> Theme {
    static T: OnceLock<Theme> = OnceLock::new();
    *T.get_or_init(Theme::default_readable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_theme_is_identity() {
        let t = Theme::plain();
        assert_eq!(t.paint(Role::Tool, "hi"), "hi");
    }

    #[test]
    fn colored_theme_wraps_ansi() {
        let t = Theme { enabled: true };
        let s = t.paint(Role::Tool, "x");
        assert!(s.contains("\x1b[38;2;77;107;254m"));
        assert!(s.ends_with("\x1b[0m"));
        assert!(s.contains('x'));
    }

    #[test]
    fn deepseek_blue_constant() {
        assert_eq!(DEEPSEEK_BLUE_RGB, (77, 107, 254));
    }
}
