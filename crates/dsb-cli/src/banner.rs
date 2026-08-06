//! Welcome banner: DeepSeek whale mark + DeepSeek-blue chrome.
//!
//! The whale mark is a **braille raster of the official DeepSeek logo
//! silhouette** (rounded body, belly cutout, eye/smile, fluke, fin) at CLI
//! scale — derived from the public brand mark shape used on deepseek.com /
//! the DeepSeek GitHub org avatar. Accent color `#4D6BFE`.
//!
//! Not a full TUI — line-oriented ANSI only. Respects `NO_COLOR` / non-TTY.

use crate::theme::{Role, Theme};

/// Official DeepSeek whale silhouette as braille (rasterized logo; ~14×8 cells).
///
/// Generated from the public solid-fill whale mark so the body curve, belly
/// cutout, and fluke remain recognizable at terminal scale.
pub const WHALE_MARK: &[&str] = &[
    "⠀⠀⣀⣤⣤⣤⣶⠂⠀⢸⣄⠀⠀⢀",
    "⢀⣾⣿⣿⣿⣿⣿⣦⡀⢸⣿⣶⣾⡿",
    "⣼⣿⣿⣿⣿⣿⣿⣿⣷⡄⢻⣿⠟⠁",
    "⣿⠀⠀⠉⠻⣿⣿⣯⠹⣿⣿⡏",
    "⣿⡆⠀⠀⠀⠙⣿⣿⣤⣿⣿⠇",
    "⢸⣷⡀⠀⢀⠀⠸⣿⣿⣿⡟",
    "⠀⠻⣷⣄⣸⣷⣄⠙⣿⣿⣄",
    "⠀⠀⠙⠻⣿⣿⣿⠿⠋⠉⠉",
];

/// Compact raster for narrow terminals (`COLUMNS` < 64).
pub const WHALE_MARK_COMPACT: &[&str] =
    &["⢀⣤⣶⣶⣇⠀⣧⣀⣠", "⣾⢿⣿⣿⣿⣧⣹⡿⠋", "⣧⠀⠈⢻⣿⣙⣿⡇", "⢻⣆⠀⣀⢻⣿⡿", "⠀⠻⢷⣿⣶⠟⠛"];

/// Horizontal rule character used inside the card.
const HR: &str = "─";

#[derive(Debug, Clone)]
pub struct BannerInfo {
    pub product: String,
    pub version: String,
    pub invocation: String,
    pub tagline: String,
    pub cwd: String,
    pub profile: String,
    pub epoch: String,
    pub session: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<String>,
    pub tips: String,
}

impl BannerInfo {
    pub fn default_product(version: &str, invocation: &str) -> Self {
        Self {
            product: "DeepSeek Build".into(),
            version: version.into(),
            invocation: invocation.into(),
            tagline: "DeepSeek-native coding agent".into(),
            cwd: ".".into(),
            profile: "safe".into(),
            epoch: String::new(),
            session: None,
            effort: None,
            thinking: None,
            tips: "/help  ·  /pro  ·  /flash  ·  /quit".into(),
        }
    }
}

/// Detect approximate terminal width (columns). Falls back to 80.
pub fn terminal_cols() -> usize {
    // Keep dep-free (no `terminal_size`); `COLUMNS` is the portable override.
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|&n| n >= 40)
        .unwrap_or(80)
}

fn visible_width(s: &str) -> usize {
    // Banner lines are ASCII + common Unicode symbols (box, braille).
    // Count Unicode scalar values as width 1 (good enough for our fixed glyph set).
    s.chars().count()
}

fn pad_right(s: &str, width: usize) -> String {
    let w = visible_width(s);
    if w >= width {
        s.to_string()
    } else {
        format!("{s}{}", " ".repeat(width - w))
    }
}

fn truncate_path(path: &str, max: usize) -> String {
    if visible_width(path) <= max {
        return path.to_string();
    }
    if max <= 3 {
        return "…".to_string();
    }
    let chars: Vec<char> = path.chars().collect();
    let keep = max - 1;
    let start = chars.len().saturating_sub(keep);
    format!("…{}", chars[start..].iter().collect::<String>())
}

/// True if `s` contains any Unicode braille pattern (U+2800–U+28FF).
#[cfg(test)]
fn contains_braille(s: &str) -> bool {
    s.chars().any(|c| ('\u{2800}'..='\u{28FF}').contains(&c))
}

/// Role for each right-hand text line (title vs meta hierarchy).
#[derive(Clone, Copy)]
enum LineRole {
    Title,
    Tagline,
    Meta,
    Tips,
    Blank,
}

fn role_for_text(line_role: LineRole) -> Role {
    match line_role {
        LineRole::Title => Role::Accent,
        LineRole::Tagline | LineRole::Meta | LineRole::Tips | LineRole::Blank => Role::Model,
    }
}

/// One content row inside the card: optional whale slice + optional text.
struct BodyRow {
    left: String,
    right: String,
    text_role: LineRole,
}

/// Render the full welcome card. When `theme` is plain, still prints structure
/// without ANSI color.
pub fn render_welcome(theme: &Theme, info: &BannerInfo) -> String {
    let cols = terminal_cols();
    let narrow = cols < 64;

    let mark = if narrow {
        WHALE_MARK_COMPACT
    } else {
        WHALE_MARK
    };
    let mark_w = mark.iter().map(|l| visible_width(l)).max().unwrap_or(0);

    // Right-hand text lines (product card body) + roles for color hierarchy.
    let mut right: Vec<(String, LineRole)> = Vec::new();
    right.push((
        format!("{}  v{}", info.product, info.version),
        LineRole::Title,
    ));
    right.push((info.tagline.clone(), LineRole::Tagline));
    right.push((String::new(), LineRole::Blank));
    right.push((format!("cmd      {}", info.invocation), LineRole::Meta));
    right.push((
        format!(
            "cwd      {}",
            truncate_path(&info.cwd, if narrow { 28 } else { 42 })
        ),
        LineRole::Meta,
    ));
    right.push((
        format!(
            "profile  {}  ·  epoch {}",
            info.profile,
            if info.epoch.is_empty() {
                "—"
            } else {
                info.epoch.as_str()
            }
        ),
        LineRole::Meta,
    ));
    if let Some(sid) = &info.session {
        right.push((format!("session  {sid}"), LineRole::Meta));
    }
    if info.effort.is_some() || info.thinking.is_some() {
        right.push((
            format!(
                "effort   {}  ·  thinking {}",
                info.effort.as_deref().unwrap_or("default"),
                info.thinking.as_deref().unwrap_or("on")
            ),
            LineRole::Meta,
        ));
    }

    // Side-by-side: whale | gap | text. Align rows to max height.
    let gap_n = if narrow { 2 } else { 3 };
    let gap = " ".repeat(gap_n);
    let rows = mark.len().max(right.len());
    let mut body: Vec<BodyRow> = Vec::with_capacity(rows + 2);

    for i in 0..rows {
        let left = pad_right(mark.get(i).copied().unwrap_or(""), mark_w);
        let (r, role) = right
            .get(i)
            .map(|(s, role)| (s.clone(), *role))
            .unwrap_or((String::new(), LineRole::Blank));
        body.push(BodyRow {
            left,
            right: r,
            text_role: role,
        });
    }

    body.push(BodyRow {
        left: pad_right("", mark_w),
        right: String::new(),
        text_role: LineRole::Blank,
    });
    // Tips span full width under the mark column (indent to text column).
    let tips_indent = mark_w + gap_n;
    body.push(BodyRow {
        left: String::new(),
        right: format!("{}{}", " ".repeat(tips_indent), info.tips),
        text_role: LineRole::Tips,
    });

    // Measure plain visible width of each assembled row.
    let plain_row = |row: &BodyRow| -> String {
        if row.left.is_empty() {
            row.right.clone()
        } else if row.right.is_empty() {
            row.left.clone()
        } else {
            format!("{}{}{}", row.left, gap, row.right)
        }
    };

    let content_w = body
        .iter()
        .map(|r| visible_width(&plain_row(r)))
        .max()
        .unwrap_or(40)
        .clamp(40, cols.saturating_sub(4).max(40));

    let mut out = String::new();
    out.push('\n');

    let top = format!("╭{}╮", HR.repeat(content_w + 2));
    let bot = format!("╰{}╯", HR.repeat(content_w + 2));
    out.push_str(&theme.paint(Role::Accent, &top));
    out.push('\n');

    for row in &body {
        let plain = plain_row(row);
        let pad = content_w.saturating_sub(visible_width(&plain));
        let pad_s = " ".repeat(pad);

        let painted_body = if theme.enabled {
            if row.left.is_empty() {
                // Tips / full-width text only.
                format!(
                    "{}{}",
                    theme.paint(role_for_text(row.text_role), &row.right),
                    pad_s
                )
            } else {
                let left_c = theme.paint(Role::Accent, &row.left);
                let right_c = if row.right.is_empty() {
                    String::new()
                } else {
                    theme.paint(role_for_text(row.text_role), &row.right)
                };
                if right_c.is_empty() {
                    format!("{left_c}{pad_s}")
                } else {
                    format!("{left_c}{gap}{right_c}{pad_s}")
                }
            }
        } else {
            format!("{plain}{pad_s}")
        };

        let line = if theme.enabled {
            format!(
                "{} {} {}",
                theme.paint(Role::Accent, "│"),
                painted_body,
                theme.paint(Role::Accent, "│"),
            )
        } else {
            format!("│ {plain}{pad_s} │")
        };
        out.push_str(&line);
        out.push('\n');
    }

    out.push_str(&theme.paint(Role::Accent, &bot));
    out.push('\n');
    out.push('\n');
    out
}

/// Styled REPL prompt (`❯ ` in DeepSeek blue when color is on).
pub fn prompt(theme: &Theme) -> String {
    theme.paint(Role::Accent, "❯ ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whale_mark_is_non_empty_braille() {
        assert!(WHALE_MARK.len() >= 5);
        let joined = WHALE_MARK.join("\n");
        assert!(contains_braille(&joined), "mark should use braille density");
        let w = WHALE_MARK.iter().map(|l| visible_width(l)).max().unwrap();
        assert!(w >= 10);
        assert!(w <= 22);
    }

    #[test]
    fn compact_mark_is_braille() {
        assert!(contains_braille(&WHALE_MARK_COMPACT.join("\n")));
    }

    #[test]
    fn plain_banner_has_no_ansi() {
        let t = Theme::plain();
        let info = BannerInfo::default_product("1.2.0", "dsb");
        let s = render_welcome(&t, &info);
        assert!(!s.contains("\x1b["));
        assert!(s.contains("DeepSeek Build"));
        assert!(s.contains("1.2.0"));
        assert!(s.contains('╭') && s.contains('╰'));
        assert!(contains_braille(&s), "whale braille in banner");
    }

    #[test]
    fn colored_banner_uses_deepseek_blue() {
        let t = Theme { enabled: true };
        let info = BannerInfo {
            cwd: "/tmp/project".into(),
            profile: "dogfood".into(),
            epoch: "ab12cd".into(),
            session: Some("sess-1".into()),
            ..BannerInfo::default_product("1.2.0", "deepseek-build")
        };
        let s = render_welcome(&t, &info);
        assert!(s.contains("\x1b[38;2;77;107;254m"));
        assert!(s.contains("dogfood"));
        assert!(s.contains("sess-1"));
        assert!(s.contains("/tmp/project"));
    }

    #[test]
    fn truncate_path_keeps_suffix() {
        let long = "/Users/me/very/long/path/to/repo";
        let t = truncate_path(long, 12);
        assert!(t.starts_with('…'));
        assert!(t.ends_with("to/repo") || t.contains("repo"));
        assert!(visible_width(&t) <= 12);
    }

    #[test]
    fn prompt_is_plain_without_color() {
        assert_eq!(prompt(&Theme::plain()), "❯ ");
    }

    #[test]
    fn brand_rgb_matches_official_hex() {
        // Official DeepSeek product accent #4D6BFE.
        let (r, g, b) = crate::theme::DEEPSEEK_BLUE_RGB;
        assert_eq!((r, g, b), (0x4d, 0x6b, 0xfe));
    }
}
