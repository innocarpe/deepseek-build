//! iTerm2 terminal tab icon integration (self-installing).
//!
//! iTerm2 (Tahoe tab style, macOS 15+) renders a per-process icon in each tab
//! from a process-name → icon mapping: `graphic_icons.json` + a
//! `graphic_<name>.png` under `~/Library/Application Support/iTerm2`, tinted
//! by `graphic_colors.json`. When the DeepSeek Build agent runs, its
//! foreground process name is `deepseek-build-agent`, so installing this
//! mapping makes the tab show the official DeepSeek whale logo — no border or
//! background — with no extra user step (works right after `npm i -g`).
//!
//! The logo is embedded in the binary (`include_bytes!`), so the install
//! works from any distribution path (npm prebuilt, cargo, install script).
//! The install is idempotent and only touches the three iTerm2 files this
//! product owns.

use std::fs;
use std::path::{Path, PathBuf};

/// Official DeepSeek whale logo (transparent PNG, DeepSeek CDN favicon).
const LOGO_PNG: &[u8] = include_bytes!("../assets/graphic_deepseek.png");

/// iTerm2 Application Support directory relative to `$HOME`.
const ITERM_APP_SUPPORT_REL: &str = "Library/Application Support/iTerm2";
/// Icon image filename iTerm2 loads for the logical icon name.
const ICON_FILE: &str = "graphic_deepseek.png";
const ICONS_JSON: &str = "graphic_icons.json";
const COLORS_JSON: &str = "graphic_colors.json";
/// Logical icon name (prefixes the image file: `graphic_<name>.png`).
const LOGICAL_NAME: &str = "deepseek";
/// Process names the agent can appear as in the terminal tab.
const COMMANDS: &[&str] = &[
    "deepseek-build-agent",
    "deepseek-build",
    "dsb",
    "xai-grok-pager",
];
/// DeepSeek brand blue; iTerm2 tints the tab icon monochrome with this color.
const BRAND_BLUE: &str = "#4D6BFE";

/// Outcome of [`ensure_iterm2_tab_icon`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabIconStatus {
    /// iTerm2 files were written (first install).
    Installed,
    /// Mapping was already present; nothing changed.
    AlreadyInstalled,
    /// Not macOS / no iTerm2 installation detected.
    Skipped,
    /// iTerm2 present but the install failed (e.g. read-only home).
    Failed,
}

/// Idempotently install the DeepSeek Build tab icon for iTerm2.
///
/// Best-effort: never panics, never returns an error. Callers may log the
/// [`TabIconStatus`] for visibility but must not fail the launch on it.
pub fn ensure_iterm2_tab_icon() -> TabIconStatus {
    match iterm_app_support_dir() {
        Some(dir) => ensure_in_dir(&dir),
        None => TabIconStatus::Skipped,
    }
}

/// Remove the DeepSeek Build tab icon mapping (keeps unrelated iTerm2 files).
pub fn remove_iterm2_tab_icon() -> bool {
    match iterm_app_support_dir() {
        Some(dir) => remove_in_dir(&dir),
        None => false,
    }
}

/// iTerm2 app-support dir, only when iTerm2 is actually installed (macOS).
fn iterm_app_support_dir() -> Option<PathBuf> {
    if !cfg!(target_os = "macos") {
        return None;
    }
    let home = std::env::var_os("HOME")?;
    let dir = Path::new(&home).join(ITERM_APP_SUPPORT_REL);
    dir.is_dir().then_some(dir)
}

/// Core install against an explicit app-support dir (testable, no `$HOME`
/// dependency). Never panics.
fn ensure_in_dir(dir: &Path) -> TabIconStatus {
    if already_installed(dir) {
        return TabIconStatus::AlreadyInstalled;
    }
    let ok = fs::write(dir.join(ICON_FILE), LOGO_PNG).is_ok()
        && merge_json_key(
            &dir.join(ICONS_JSON),
            LOGICAL_NAME,
            serde_json::json!(COMMANDS),
        )
        && merge_json_key(
            &dir.join(COLORS_JSON),
            LOGICAL_NAME,
            serde_json::json!(BRAND_BLUE),
        );
    if ok {
        TabIconStatus::Installed
    } else {
        TabIconStatus::Failed
    }
}

/// Core removal against an explicit app-support dir.
fn remove_in_dir(dir: &Path) -> bool {
    let icon_removed = remove_json_key(&dir.join(ICONS_JSON), LOGICAL_NAME);
    let color_removed = remove_json_key(&dir.join(COLORS_JSON), LOGICAL_NAME);
    let file_removed = match fs::remove_file(dir.join(ICON_FILE)) {
        Ok(()) => true,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => true,
        Err(_) => false,
    };
    icon_removed && color_removed && file_removed
}

/// True when the icon file and both mapping keys are already in place.
fn already_installed(dir: &Path) -> bool {
    dir.join(ICON_FILE).is_file()
        && json_has_key(&dir.join(ICONS_JSON), LOGICAL_NAME)
        && json_has_key(&dir.join(COLORS_JSON), LOGICAL_NAME)
}

/// Tolerant JSON object loader: iTerm2's bundled JSON files use trailing
/// commas (invalid strict JSON), so strip them before parsing.
fn load_json_object(path: &Path) -> serde_json::Map<String, serde_json::Value> {
    match fs::read_to_string(path) {
        Ok(text) => {
            match serde_json::from_str::<serde_json::Value>(&strip_trailing_commas(&text)) {
                Ok(serde_json::Value::Object(map)) => map,
                _ => serde_json::Map::new(),
            }
        }
        Err(_) => serde_json::Map::new(),
    }
}

fn json_has_key(path: &Path, key: &str) -> bool {
    load_json_object(path).contains_key(key)
}

/// Merge `key` → `value` into the JSON object at `path` (creating it if
/// missing). Returns false only on write failure.
fn merge_json_key(path: &Path, key: &str, value: serde_json::Value) -> bool {
    let mut map = load_json_object(path);
    map.insert(key.to_string(), value);
    let body = match serde_json::to_string_pretty(&serde_json::Value::Object(map)) {
        Ok(b) => b,
        Err(_) => return false,
    };
    fs::write(path, format!("{body}\n")).is_ok()
}

/// Remove `key` from the JSON object at `path`; deletes the file if empty.
fn remove_json_key(path: &Path, key: &str) -> bool {
    let mut map = load_json_object(path);
    if map.remove(key).is_none() {
        return true; // nothing to remove
    }
    if map.is_empty() {
        return match fs::remove_file(path) {
            Ok(()) => true,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => true,
            Err(_) => false,
        };
    }
    let body = match serde_json::to_string_pretty(&serde_json::Value::Object(map)) {
        Ok(b) => b,
        Err(_) => return false,
    };
    fs::write(path, format!("{body}\n")).is_ok()
}

/// Remove trailing commas (e.g. `"a": [1, 2,]`) so strict JSON parsers accept
/// iTerm2's bundled files. String literals are respected.
fn strip_trailing_commas(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut in_string = false;
    let mut escaped = false;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if in_string {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                out.push(c);
            }
            ',' => {
                // Skip the comma when the next non-whitespace char closes a
                // container (trailing comma).
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                    // drop
                } else {
                    out.push(c);
                }
            }
            _ => out.push(c),
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> PathBuf {
        tempfile::tempdir().unwrap().into_path()
    }

    #[test]
    fn strips_trailing_commas_in_objects_and_arrays() {
        let input = r#"{
  "claude_code": [
    "claude",
  ],
  "deepseek": ["dsb", "dsb",],
  "note": "keep, this, literal",
}"#;
        let cleaned = strip_trailing_commas(input);
        let parsed: serde_json::Value = serde_json::from_str(&cleaned).expect("valid JSON");
        assert_eq!(parsed["claude_code"][0], "claude");
        assert_eq!(parsed["deepseek"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["note"], "keep, this, literal");
    }

    #[test]
    fn merge_preserves_existing_keys() {
        let dir = temp_dir();
        let icons = dir.join(ICONS_JSON);
        fs::write(&icons, "{\n  \"claude_code\": [\"claude\",],\n}\n").unwrap();

        assert!(merge_json_key(
            &icons,
            LOGICAL_NAME,
            serde_json::json!(COMMANDS)
        ));

        let map = load_json_object(&icons);
        assert_eq!(map["claude_code"][0], "claude", "existing key preserved");
        let deepseek: Vec<&str> = map[LOGICAL_NAME]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(deepseek, COMMANDS, "deepseek commands mapped");
    }

    #[test]
    fn remove_deletes_only_our_key() {
        let dir = temp_dir();
        let icons = dir.join(ICONS_JSON);
        fs::write(&icons, r#"{"git": ["git"], "deepseek": ["dsb"]}"#).unwrap();

        assert!(remove_json_key(&icons, LOGICAL_NAME));
        let map = load_json_object(&icons);
        assert!(map.contains_key("git"), "unrelated key kept");
        assert!(!map.contains_key("deepseek"));
        assert!(icons.is_file(), "file kept when other keys remain");
    }

    #[test]
    fn remove_deletes_file_when_empty() {
        let dir = temp_dir();
        let icons = dir.join(ICONS_JSON);
        fs::write(&icons, r#"{"deepseek": ["dsb"]}"#).unwrap();

        assert!(remove_json_key(&icons, LOGICAL_NAME));
        assert!(!icons.exists(), "empty JSON file removed");
    }

    #[test]
    fn embedded_logo_is_valid_png() {
        // The tab icon must stay a real PNG (iTerm2 renders it at 16pt).
        assert!(LOGO_PNG.starts_with(b"\x89PNG\r\n\x1a\n"), "PNG magic");
        assert!(LOGO_PNG.len() > 1000, "non-trivial logo payload");
    }

    #[test]
    fn ensure_is_idempotent_in_temp_dir() {
        let dir = temp_dir();

        let first = ensure_in_dir(&dir);
        assert_eq!(first, TabIconStatus::Installed);

        // Re-run: no changes.
        assert_eq!(ensure_in_dir(&dir), TabIconStatus::AlreadyInstalled);

        // Files land exactly where iTerm2 reads them.
        assert!(dir.join(ICON_FILE).is_file());
        let map = load_json_object(&dir.join(ICONS_JSON));
        assert!(map.contains_key(LOGICAL_NAME));
        let colors = load_json_object(&dir.join(COLORS_JSON));
        assert_eq!(colors[LOGICAL_NAME], serde_json::json!(BRAND_BLUE));

        // Removal is clean and idempotent.
        assert!(remove_in_dir(&dir));
        assert!(remove_in_dir(&dir), "second remove is a no-op success");
        assert!(!dir.join(ICON_FILE).exists());
        assert!(!dir.join(ICONS_JSON).exists());
        assert!(!dir.join(COLORS_JSON).exists());
    }

    #[test]
    fn ensure_in_dir_creates_dir_and_returns_installed() {
        // ensure_in_dir writes into whatever dir it is given (in real use the
        // iTerm2 dir already exists). A fresh temp dir exercises the same
        // path without touching the user's $HOME.
        let dir = temp_dir();
        assert_eq!(ensure_in_dir(&dir), TabIconStatus::Installed);
        assert!(dir.join(ICON_FILE).is_file());
    }
}
