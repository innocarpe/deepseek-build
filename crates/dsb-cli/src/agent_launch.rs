//! Launch the DeepSeek Build full-screen agent TUI (product entry).
//!
//! `deepseek-build` / `dsb` with no args on a TTY **exec** the product agent
//! binary (`deepseek-build-agent`), which is the DeepSeek-branded composition
//! root built from the vendored agent tree (not a separate “Grok product” UI).

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use dsb_config::BuildHome;

/// Env override for the full-screen agent binary.
pub const ENV_AGENT_BIN: &str = "DEEPSEEK_BUILD_AGENT_BIN";
/// Installed name next to product bins / under `~/.deepseek-build/bin/`.
pub const AGENT_BIN_NAME: &str = "deepseek-build-agent";
/// Upstream cargo artifact name from `xai-grok-pager-bin`.
pub const UPSTREAM_AGENT_BIN_NAME: &str = "xai-grok-pager";

/// Ordered candidate paths for the Grok-class agent binary.
pub fn agent_bin_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();

    if let Ok(p) = env::var(ENV_AGENT_BIN) {
        let p = p.trim();
        if !p.is_empty() {
            out.push(PathBuf::from(p));
        }
    }

    let home = BuildHome::resolve();
    out.push(home.path().join("bin").join(AGENT_BIN_NAME));
    out.push(home.path().join("bin").join(UPSTREAM_AGENT_BIN_NAME));

    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            out.push(dir.join(AGENT_BIN_NAME));
            out.push(dir.join(UPSTREAM_AGENT_BIN_NAME));
        }
    }

    if let Ok(cargo_home) = env::var("CARGO_HOME") {
        let d = PathBuf::from(cargo_home).join("bin");
        out.push(d.join(AGENT_BIN_NAME));
        out.push(d.join(UPSTREAM_AGENT_BIN_NAME));
    } else if let Some(h) = env::var_os("HOME") {
        let d = PathBuf::from(h).join(".cargo").join("bin");
        out.push(d.join(AGENT_BIN_NAME));
        out.push(d.join(UPSTREAM_AGENT_BIN_NAME));
    }

    // Dev tree: vendor release build (walk up from CARGO_MANIFEST_DIR when set).
    if let Ok(manifest) = env::var("CARGO_MANIFEST_DIR") {
        let root = Path::new(&manifest).join("../..");
        if let Ok(root) = root.canonicalize() {
            out.push(
                root.join("third_party/grok-build/target/release")
                    .join(UPSTREAM_AGENT_BIN_NAME),
            );
            out.push(root.join("target/release").join(UPSTREAM_AGENT_BIN_NAME));
        }
    }

    out
}

pub fn find_agent_bin() -> Option<PathBuf> {
    for p in agent_bin_candidates() {
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Product default TUI theme (must match ThemeKind::DeepSeekNight display name).
pub const PRODUCT_THEME: &str = "deepseeknight";
/// Env override for product theme name (passed as GROK_THEME to the agent).
pub const ENV_PRODUCT_THEME: &str = "DEEPSEEK_BUILD_THEME";

/// Default DeepSeek OpenAI-compatible base URL (ADR 0005).
///
/// Must appear on each `[model.deepseek-*]` as `base_url`. Setting only
/// `[endpoints].xai_api_base_url` is **not** enough — the agent still routes
/// those models through the Grok CLI proxy (`cli-chat-proxy.grok.com`).
pub const DEEPSEEK_API_BASE_URL: &str = "https://api.deepseek.com";

/// Prepare product agent config under product home (DeepSeek defaults + theme).
///
/// - Creates `config.toml` when missing (full DeepSeek product seed).
/// - If present: inject missing theme; ensure DeepSeek model `base_url` is set
///   so live agent turns hit `api.deepseek.com` (not Grok proxy).
pub fn ensure_product_agent_config(home: &BuildHome) -> Result<()> {
    home.ensure_dir().context("ensure product home")?;
    let config_path = home.path().join("config.toml");
    if config_path.exists() {
        let body = std::fs::read_to_string(&config_path)
            .with_context(|| format!("read {}", config_path.display()))?;
        let next = repair_product_agent_config(&body);
        if next != body {
            std::fs::write(&config_path, next)
                .with_context(|| format!("update {}", config_path.display()))?;
        }
        return Ok(());
    }

    let api_key_line = match dsb_config::Credentials::load(home) {
        Ok(c) => format!("api_key = \"{}\"\n", escape_toml_basic(c.api_key())),
        Err(_) => String::new(),
    };

    // Chat Completions backend for DeepSeek (not Grok Responses default).
    // `base_url` on each model is load-bearing for OpenAI-compat providers.
    let body = format!(
        r#"# DeepSeek Build product defaults (auto-created).
# Product chrome: DeepSeek Build + DeepSeekNight theme (#4D6BFE).

[models]
default = "deepseek-v4-flash"

[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
name = "DeepSeek V4 Flash"
context_window = 128000
api_backend = "chat_completions"
base_url = "{DEEPSEEK_API_BASE_URL}"
{api_key_line}env_key = "DEEPSEEK_API_KEY"

[model.deepseek-v4-pro]
model = "deepseek-v4-pro"
name = "DeepSeek V4 Pro"
context_window = 128000
api_backend = "chat_completions"
base_url = "{DEEPSEEK_API_BASE_URL}"
{api_key_line}env_key = "DEEPSEEK_API_KEY"

[endpoints]
xai_api_base_url = "{DEEPSEEK_API_BASE_URL}"

[ui]
theme = "{PRODUCT_THEME}"
# Product default: Spec 90 — not YOLO-only (G005 / Path A / 3.0.0).
yolo = false

# L3 product defaults (4.0.0): subagents on; worktree remains opt-in CLI.
[subagents]
enabled = true
"#
    );

    std::fs::write(&config_path, body)
        .with_context(|| format!("write {}", config_path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600));
    }

    Ok(())
}

/// Best-effort repair of an existing product `config.toml`.
///
/// Idempotent. Does not rewrite unrelated user settings. Ensures:
/// 1. DeepSeekNight theme when no `theme` key exists
/// 2. `base_url` on DeepSeek model stanzas (or appends full model blocks)
/// 3. Explicit `yolo = false` when the key is missing (Spec 90 product default)
fn repair_product_agent_config(body: &str) -> String {
    let mut next = body.to_string();
    if !next.ends_with('\n') {
        next.push('\n');
    }

    if !next.contains("theme") {
        if !next.contains("[ui]") {
            next.push_str("\n[ui]\n");
        }
        next.push_str(&format!("theme = \"{PRODUCT_THEME}\"\n"));
    }

    // Product default is not YOLO. Only inject when the key is absent so we do
    // not clobber an explicit user `yolo = true` (CLI `--yolo` / opt-in).
    if !next.lines().any(|l| {
        let t = l.trim();
        !t.starts_with('#') && t.starts_with("yolo")
    }) {
        if !next.contains("[ui]") {
            next.push_str("\n[ui]\n");
        }
        next.push_str("yolo = false\n");
    }

    next = ensure_deepseek_model_base_url(next, "deepseek-v4-flash", "DeepSeek V4 Flash");
    next = ensure_deepseek_model_base_url(next, "deepseek-v4-pro", "DeepSeek V4 Pro");

    if !next.contains("xai_api_base_url") {
        if !next.contains("[endpoints]") {
            next.push_str("\n[endpoints]\n");
        }
        next.push_str(&format!("xai_api_base_url = \"{DEEPSEEK_API_BASE_URL}\"\n"));
    }

    // L3: ensure subagents stay enabled unless user already set [subagents].
    if !next.contains("[subagents]") {
        next.push_str("\n[subagents]\nenabled = true\n");
    }

    next
}

/// Ensure `[model.<id>]` exists and contains `base_url = api.deepseek.com`.
fn ensure_deepseek_model_base_url(body: String, model_id: &str, display_name: &str) -> String {
    let header = format!("[model.{model_id}]");
    if !body.contains(&header) {
        let mut next = body;
        next.push_str(&format!(
            r#"
{header}
model = "{model_id}"
name = "{display_name}"
context_window = 128000
api_backend = "chat_completions"
base_url = "{DEEPSEEK_API_BASE_URL}"
env_key = "DEEPSEEK_API_KEY"
"#
        ));
        return next;
    }

    // Inject base_url into the model section if missing (section-scoped).
    let lines: Vec<&str> = body.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len() + 2);
    let mut i = 0;
    let mut patched = false;
    while i < lines.len() {
        let line = lines[i];
        out.push(line.to_string());
        if line.trim() == header {
            // Scan this section for base_url; inject after header if absent.
            let mut j = i + 1;
            let mut has_base = false;
            while j < lines.len() {
                let t = lines[j].trim();
                if t.starts_with('[') && !t.starts_with("[model.") {
                    // end of contiguous model sections? any new table ends section
                    break;
                }
                if t.starts_with('[') {
                    break;
                }
                if t.starts_with("base_url") {
                    has_base = true;
                    break;
                }
                j += 1;
            }
            if !has_base {
                out.push(format!("base_url = \"{DEEPSEEK_API_BASE_URL}\""));
                patched = true;
            }
        }
        i += 1;
    }
    if patched {
        let mut s = out.join("\n");
        if !s.ends_with('\n') {
            s.push('\n');
        }
        s
    } else {
        body
    }
}

fn escape_toml_basic(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Print DeepSeek whale splash (DeepSeek blue) before handing off to the TUI.
pub fn print_product_splash() {
    use std::io::{self, IsTerminal, Write};

    if !io::stdout().is_terminal() || env::var_os("NO_COLOR").is_some() {
        return;
    }
    // DeepSeek blue #4D6BFE
    const BLUE: &str = "\x1b[38;2;77;107;254m";
    const BOLD: &str = "\x1b[1m";
    const RESET: &str = "\x1b[0m";
    let whale = crate::banner::WHALE_MARK;
    let mut out = io::stderr();
    let _ = writeln!(out);
    for line in whale {
        let _ = writeln!(out, "{BLUE}{line}{RESET}");
    }
    let _ = writeln!(
        out,
        "{BLUE}{BOLD}  DeepSeek Build{RESET}  ·  coding agent TUI  ·  #4D6BFE"
    );
    let _ = writeln!(out);
    let _ = out.flush();
}

/// Exec the Grok-class agent, replacing this process (Unix).
///
/// On failure to find the binary, returns an error with install guidance.
pub fn exec_agent(args: &[String]) -> Result<()> {
    let home = BuildHome::resolve();
    // Best-effort product config; missing credentials still allow agent UI.
    let _ = ensure_product_agent_config(&home);
    print_product_splash();

    let Some(bin) = find_agent_bin() else {
        bail!(
            "DeepSeek Build agent binary not found (looked for `{AGENT_BIN_NAME}` / `{UPSTREAM_AGENT_BIN_NAME}`).\n\
             Build and install:\n\
               ./scripts/build-grok-pager.sh release\n\
               ./scripts/install.sh\n\
             Or set {ENV_AGENT_BIN} to the `xai-grok-pager` binary path.\n\
             Candidates: {}",
            agent_bin_candidates()
                .into_iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    };

    let mut cmd = Command::new(&bin);
    cmd.args(args);
    // Bridge product home into Grok path resolution without mutating process env globally
    // when not needed — Command env is sufficient for the child/exec image.
    cmd.env("GROK_HOME", home.path());
    // Force product theme unless user already set GROK_THEME or DEEPSEEK_BUILD_THEME.
    let theme = env::var(ENV_PRODUCT_THEME)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| env::var("GROK_THEME").ok().filter(|s| !s.trim().is_empty()))
        .unwrap_or_else(|| PRODUCT_THEME.to_string());
    cmd.env("GROK_THEME", &theme);
    cmd.env("LC_GROK_THEME", &theme);
    if env::var_os(dsb_config::ENV_API_KEY).is_none() {
        if let Ok(c) = dsb_config::Credentials::load(&home) {
            cmd.env(dsb_config::ENV_API_KEY, c.api_key());
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = cmd.exec();
        // exec only returns on error
        bail!("failed to exec {}: {err}", bin.display());
    }

    #[cfg(not(unix))]
    {
        let status = cmd
            .status()
            .with_context(|| format!("spawn {}", bin.display()))?;
        if status.success() {
            Ok(())
        } else {
            bail!(
                "agent {} exited with {}",
                bin.display(),
                status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "signal".into())
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidates_include_product_home_agent_name() {
        let cs = agent_bin_candidates();
        assert!(
            cs.iter().any(|p| p.ends_with(AGENT_BIN_NAME)),
            "expected {AGENT_BIN_NAME} in {cs:?}"
        );
    }

    #[test]
    fn toml_escape_quotes() {
        assert_eq!(escape_toml_basic(r#"a"b"#), r#"a\"b"#);
    }
}

#[test]
fn product_config_seed_contains_deepseek_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let home = dsb_config::BuildHome::from_path(dir.path());
    ensure_product_agent_config(&home).unwrap();
    let body = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(body.contains("deepseek-v4-flash"));
    assert!(body.contains("api.deepseek.com"));
    assert!(body.contains("chat_completions"));
    assert!(body.contains("DEEPSEEK_API_KEY"));
    assert!(body.contains("deepseeknight"));
    // Spec 90 / G005: product default is not YOLO.
    assert!(
        body.contains("yolo = false"),
        "seed missing yolo = false: {body}"
    );
    // L3 / 4.0.0: subagents enabled as product default.
    assert!(
        body.contains("[subagents]") && body.contains("enabled = true"),
        "seed missing subagents enabled: {body}"
    );
    // Load-bearing: model-level base_url (not only endpoints.xai_api_base_url).
    assert!(
        body.contains(&format!("base_url = \"{DEEPSEEK_API_BASE_URL}\"")),
        "seed missing model base_url: {body}"
    );
    // Existing file without theme gets theme injected (not full rewrite).
    std::fs::write(dir.path().join("config.toml"), "keep=1\n").unwrap();
    ensure_product_agent_config(&home).unwrap();
    let body2 = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(body2.contains("keep=1"));
    assert!(body2.contains("theme = \"deepseeknight\""));
    assert!(body2.contains("yolo = false"));
    // Repair also adds DeepSeek model blocks with base_url.
    assert!(body2.contains("[model.deepseek-v4-flash]"));
    assert!(body2.contains("base_url = \"https://api.deepseek.com\""));
    ensure_product_agent_config(&home).unwrap();
    let body3 = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert_eq!(body2, body3);
}

#[test]
fn repair_injects_yolo_false_when_missing_but_preserves_true() {
    let missing = "[ui]\ntheme = \"deepseeknight\"\n";
    let fixed = repair_product_agent_config(missing);
    assert!(fixed.contains("yolo = false"));
    let user_yolo = "[ui]\ntheme = \"deepseeknight\"\nyolo = true\n";
    let kept = repair_product_agent_config(user_yolo);
    assert!(kept.contains("yolo = true"));
    assert!(!kept.lines().any(|l| l.trim() == "yolo = false"));
}

#[test]
fn repair_injects_base_url_into_existing_deepseek_models() {
    let raw = r#"
[models]
default = "deepseek-v4-flash"

[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
api_backend = "chat_completions"
env_key = "DEEPSEEK_API_KEY"

[model.deepseek-v4-pro]
model = "deepseek-v4-pro"
api_backend = "chat_completions"

[ui]
theme = "deepseeknight"
"#;
    let fixed = repair_product_agent_config(raw);
    assert!(fixed.contains("base_url = \"https://api.deepseek.com\""));
    // Both model sections get base_url
    let flash_idx = fixed.find("[model.deepseek-v4-flash]").unwrap();
    let pro_idx = fixed.find("[model.deepseek-v4-pro]").unwrap();
    let flash_sec = &fixed[flash_idx..pro_idx];
    let pro_sec = &fixed[pro_idx..];
    assert!(flash_sec.contains("base_url = \"https://api.deepseek.com\""));
    assert!(pro_sec.contains("base_url = \"https://api.deepseek.com\""));
    // Idempotent
    assert_eq!(repair_product_agent_config(&fixed), fixed);
}
