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

/// Product default TUI theme (must match ThemeKind::DeepSeekNightNeutral display name).
pub const PRODUCT_THEME: &str = "deepseeknight-neutral";
/// Blue-tinted DeepSeek theme (original product skin; picker option 1).
pub const PRODUCT_THEME_BLUE: &str = "deepseeknight";
/// Env override for product theme name (passed as GROK_THEME to the agent).
pub const ENV_PRODUCT_THEME: &str = "DEEPSEEK_BUILD_THEME";

/// Product name shown in the terminal tab/window title (OSC 0).
///
/// The vendored agent rebrands its own tab title to this value (see
/// `xai-grok-pager` `notifications/title.rs`); we also emit it here before
/// exec so the tab carries the product name from the very first frame.
pub const PRODUCT_TITLE: &str = "DeepSeek Build";

/// OSC 0 (window/tab title) escape sequence for the product name.
///
/// Matches crossterm's `SetTitle` framing used by the vendored agent and by
/// peer CLIs (Claude Code, Codex) so iTerm2 / Terminal.app render the tab
/// title consistently.
pub fn product_title_escape() -> Vec<u8> {
    format!("\x1b]0;{PRODUCT_TITLE}\x07").into_bytes()
}

/// Emit the product tab title to stdout when it is a TTY (best-effort).
fn emit_product_title() {
    use std::io::{self, IsTerminal, Write};

    if !io::stdout().is_terminal() {
        return;
    }
    let _ = io::stdout().write_all(&product_title_escape());
    let _ = io::stdout().flush();
}

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
    ensure_product_agent_config_with_theme(home, PRODUCT_THEME)
}

/// Same as [`ensure_product_agent_config`] but with an explicit theme canonical
/// (used by the first-launch picker; defaults to `PRODUCT_THEME`).
pub fn ensure_product_agent_config_with_theme(home: &BuildHome, theme: &str) -> Result<()> {
    home.ensure_dir().context("ensure product home")?;
    let config_path = home.path().join("config.toml");
    if config_path.exists() {
        let body = std::fs::read_to_string(&config_path)
            .with_context(|| format!("read {}", config_path.display()))?;
        let next = repair_product_agent_config_with_theme(&body, theme);
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
# Product chrome: DeepSeek Build + DeepSeek Night theme (#4D6BFE).

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
theme = "{theme}"
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
/// 1. DeepSeek Night theme when no `theme` key exists (default neutral skin)
/// 2. `base_url` on DeepSeek model stanzas (or appends full model blocks)
/// 3. Explicit `yolo = false` when the key is missing (Spec 90 product default)
fn repair_product_agent_config(body: &str) -> String {
    repair_product_agent_config_with_theme(body, PRODUCT_THEME)
}

/// Same as [`repair_product_agent_config`] but injects the given theme canonical.
fn repair_product_agent_config_with_theme(body: &str, theme: &str) -> String {
    let mut next = body.to_string();
    if !next.ends_with('\n') {
        next.push('\n');
    }

    if !next.contains("theme") {
        if !next.contains("[ui]") {
            next.push_str("\n[ui]\n");
        }
        next.push_str(&format!("theme = \"{theme}\"\n"));
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

/// First-launch theme picker (fresh product home + interactive tty only).
///
/// Asks the user to choose between the blue-tinted `deepseeknight` skin and
/// the default neutral `deepseeknight-neutral` skin. Returns the chosen
/// canonical name, or `None` when:
/// - the home already has a `config.toml` (theme already chosen / configured)
/// - stdin is not a terminal (scripted / CI launches)
/// - the user hits EOF or keeps answering invalid input
///
/// `None` callers should fall back to [`PRODUCT_THEME`]. Prompt goes to stderr
/// (splash precedent) so stdout stays clean for piped consumers.
fn prompt_first_launch_theme(home: &BuildHome) -> Option<&'static str> {
    use std::io::{self, IsTerminal, Write};

    if home.path().join("config.toml").exists() || !io::stdin().is_terminal() {
        return None;
    }
    let mut out = io::stderr();
    for _ in 0..3 {
        let _ = writeln!(out, "Choose your DeepSeek Build theme:");
        let _ = writeln!(
            out,
            "  1) DeepSeek Night         — blue-tinted dark, original product skin"
        );
        let _ = writeln!(
            out,
            "  2) DeepSeek Night Neutral — neutral canvas, blue accents (default)"
        );
        let _ = write!(out, "Select [2]: ");
        let _ = out.flush();
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) => return None, // EOF: keep product default
            Ok(_) => {
                if let Some(theme) = picker_answer_to_theme(line.trim()) {
                    return Some(theme);
                }
                // Invalid input: re-prompt.
            }
            Err(_) => return None,
        }
    }
    None
}

/// Pure mapping from picker input to a theme canonical ("" => neutral default).
fn picker_answer_to_theme(answer: &str) -> Option<&'static str> {
    match answer.trim() {
        "1" | PRODUCT_THEME_BLUE | "deepseek-night" | "dsb" => Some(PRODUCT_THEME_BLUE),
        "2" | "" | PRODUCT_THEME | "deepseek-night-neutral" | "dsb-neutral" => Some(PRODUCT_THEME),
        _ => None,
    }
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

/// Stamp Path A Spec 10 prefix epoch under product home (owner-bar G008).
///
/// Production call site for [`dsb_context::assemble_path_a_context`] so Path A
/// linkage is not test-only. Best-effort; failures never block agent launch.
fn stamp_path_a_prefix_epoch(home: &BuildHome) {
    use dsb_context::{
        EnvironmentSummary, PathAContextInputs, SkillIndexEntry, assemble_path_a_context,
        discover_skills_index,
    };
    use dsb_provider_deepseek::{ChatMessage, ToolDefinition, ToolFunction};
    use serde_json::json;
    use std::path::PathBuf;

    let cwd_path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cwd = cwd_path.display().to_string();
    let os = std::env::consts::OS;
    let tool = |name: &str, params: serde_json::Value| ToolDefinition {
        type_: "function".into(),
        function: ToolFunction {
            name: name.into(),
            description: Some(format!("tool {name}")),
            parameters: Some(params),
        },
    };
    // Minimal product-shaped inputs: system + Standard tool schema + skills index.
    // Volatile user turn is empty for the stamp (epoch ignores volatile tail).
    let tools = vec![
        tool(
            "read_file",
            json!({"type":"object","properties":{"target_file":{"type":"string"}}}),
        ),
        tool(
            "search_replace",
            json!({
                "type":"object",
                "properties":{
                    "file_path":{"type":"string"},
                    "old_string":{"type":"string"},
                    "new_string":{"type":"string"},
                    "file_version":{"type":"string"}
                }
            }),
        ),
    ];
    let skills_index = discover_skills_index(&cwd_path, None).unwrap_or_else(|_| {
        vec![SkillIndexEntry {
            name: "product-skills".into(),
            description: "placeholder".into(),
        }]
    });
    let inputs = PathAContextInputs {
        system_prompt: "DeepSeek Build Path A stable prefix".into(),
        tools,
        skills_index,
        environment: EnvironmentSummary {
            os_family: os.into(),
            cwd,
        },
        project_instructions: String::new(),
        volatile_user_and_tools: vec![ChatMessage::user("")],
    };
    let Ok(assembled) = assemble_path_a_context(&inputs) else {
        return;
    };
    let stamp = format!(
        "path_a_prefix_epoch={}\npath_a_prefix_epoch_short={}\n",
        assembled.epoch().sha256_hex,
        assembled.epoch_short()
    );
    let path = home.path().join("path_a_prefix_epoch.txt");
    let _ = std::fs::write(path, stamp);
}

/// Stamp Path A Spec 20 routing defaults under product home (owner-bar G009).
///
/// Production call site for [`dsb_agent::path_a_default_router`] /
/// [`dsb_agent::route_path_a_turn`] so Flash-default + `/pro` one-shot are not
/// test-only. Best-effort; failures never block agent launch.
fn stamp_path_a_routing(home: &BuildHome) {
    use dsb_agent::{
        apply_routing_command, path_a_default_router, path_a_flash_wire_id, path_a_pro_wire_id,
        route_path_a_turn,
    };

    let mut router = path_a_default_router();
    let flash = route_path_a_turn(&mut router, "path-a-routing-stamp");
    let (pro_text, _) = apply_routing_command(&mut router, "/pro stamp-pro-once");
    let pro = route_path_a_turn(&mut router, &pro_text);
    let after = route_path_a_turn(&mut router, "return-to-flash");

    let stamp = format!(
        "path_a_default_model={}\n\
         path_a_pro_model={}\n\
         flash_visibility={}\n\
         pro_once_visibility={}\n\
         after_pro_visibility={}\n\
         flash_wire_id={}\n\
         pro_wire_id={}\n",
        flash.wire_model,
        pro.wire_model,
        flash.visibility_line(),
        pro.visibility_line(),
        after.visibility_line(),
        path_a_flash_wire_id(),
        path_a_pro_wire_id(),
    );
    let path = home.path().join("path_a_routing.txt");
    let _ = std::fs::write(path, stamp);
}

/// Stamp Path A L3 schedule + worker-cache defaults (owner-bar G010).
///
/// Production call site for [`dsb_agent::is_mutating_tool`] /
/// [`dsb_agent::partition_indices`] / [`dsb_agent::worker_stable_prefix`] so
/// Spec 50/60 hearts are not test-only. Best-effort; never blocks launch.
fn stamp_path_a_l3(home: &BuildHome) {
    use dsb_agent::{
        MAX_PARALLEL_READONLY, WorkerKind, is_mutating_tool, partition_indices, worker_stable_prefix,
    };
    use serde_json::json;
    use std::path::PathBuf;

    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    // Spec 50: RO parallel / mutate serial classification (fail-closed unknown/bash/MCP).
    let batch = vec![
        ("read_file".into(), json!({"target_file": "a.txt"})),
        ("search_replace".into(), json!({"file_path": "a.txt"})),
        ("run_terminal_command".into(), json!({"command": "echo x"})),
        ("mcp__demo__ping".into(), json!({})),
        ("grep".into(), json!({"pattern": "x"})),
        ("unknown_tool_xyz".into(), json!({})),
    ];
    // Map product names → classifier (also accepts short ToolName aliases).
    let class_input: Vec<(String, serde_json::Value)> = batch
        .iter()
        .map(|(n, a): &(String, serde_json::Value)| {
            let short = match n.as_str() {
                "read_file" => "read",
                "search_replace" => "edit",
                "run_terminal_command" => "bash",
                other => other,
            };
            (short.to_string(), a.clone())
        })
        .collect();
    let (ro, mu) = partition_indices(&class_input);
    let bash_mut = is_mutating_tool("bash", &json!({"command": "true"}));
    let mcp_mut = is_mutating_tool("mcp__x__y", &json!({}));
    let unk_mut = is_mutating_tool("totally_unknown", &json!({}));

    // Spec 60: worker cache law — identical tools/env → same epoch.
    let tools = dsb_tools::tool_definitions();
    let epoch_a = worker_stable_prefix(&cwd, None, tools.clone())
        .map(|s| s.epoch.sha256_hex)
        .unwrap_or_else(|_| "error".into());
    let epoch_b = worker_stable_prefix(&cwd, None, tools)
        .map(|s| s.epoch.sha256_hex)
        .unwrap_or_else(|_| "error".into());

    let config_body = std::fs::read_to_string(home.path().join("config.toml")).unwrap_or_default();
    let subagents_enabled = config_body.contains("[subagents]")
        && config_body
            .lines()
            .any(|l| l.trim() == "enabled = true" || l.trim() == "enabled=true");

    let stamp = format!(
        "max_parallel_readonly={MAX_PARALLEL_READONLY}\n\
         ro_indices={ro:?}\n\
         mu_indices={mu:?}\n\
         bash_mutating={bash_mut}\n\
         mcp_mutating={mcp_mut}\n\
         unknown_mutating={unk_mut}\n\
         worker_kind_explore={}\n\
         worker_kind_implement={}\n\
         worker_epoch_a={epoch_a}\n\
         worker_epoch_b={epoch_b}\n\
         worker_epochs_match={}\n\
         subagents_enabled_in_config={subagents_enabled}\n\
         worktree_product=opt_in\n\
         bare_dsb_session=single\n",
        WorkerKind::Explore.as_str(),
        WorkerKind::Implement.as_str(),
        epoch_a == epoch_b && epoch_a != "error",
    );
    let path = home.path().join("path_a_l3.txt");
    let _ = std::fs::write(path, stamp);
}

/// Exec the Grok-class agent, replacing this process (Unix).
///
/// On failure to find the binary, returns an error with install guidance.
pub fn exec_agent(args: &[String]) -> Result<()> {
    let home = BuildHome::resolve();
    // Best-effort product config; missing credentials still allow agent UI.
    // Fresh homes get the interactive theme picker before the seed is written.
    let theme = prompt_first_launch_theme(&home).unwrap_or(PRODUCT_THEME);
    let _ = ensure_product_agent_config_with_theme(&home, theme);
    // Spec 10 / G008: production Path A prefix epoch stamp (non-blocking).
    stamp_path_a_prefix_epoch(&home);
    // Spec 20 / G009: production Path A Flash/Pro routing stamp (non-blocking).
    stamp_path_a_routing(&home);
    // Spec 50/60 / G010: production Path A L3 schedule + worker cache stamp.
    stamp_path_a_l3(&home);
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
    // Product SemVer for TUI title + update checks (not vendor pager 0.2.x).
    // Runtime env wins if already set (e.g. npm wrapper).
    if env::var_os("DEEPSEEK_BUILD_VERSION").is_none() {
        cmd.env("DEEPSEEK_BUILD_VERSION", env!("CARGO_PKG_VERSION"));
    }
    // Product theme lives in `[ui].theme` of the seeded product config, so
    // in-pager `/theme` changes persist across launches. Only force a theme via
    // env when the user explicitly asked (DEEPSEEK_BUILD_THEME / GROK_THEME).
    if let Some(theme) = env::var(ENV_PRODUCT_THEME)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| env::var("GROK_THEME").ok().filter(|s| !s.trim().is_empty()))
    {
        cmd.env("GROK_THEME", &theme);
        cmd.env("LC_GROK_THEME", &theme);
    }
    // Brand the vendored resume hints: `dsb --resume <id>` instead of `grok --resume <id>`.
    cmd.env("GROK_INVOCATION_NAME", crate::invocation_name());
    if env::var_os(dsb_config::ENV_API_KEY).is_none() {
        if let Ok(c) = dsb_config::Credentials::load(&home) {
            cmd.env(dsb_config::ENV_API_KEY, c.api_key());
        }
    }

    // Set the tab/window title to the product name before handing off to the
    // agent (the agent re-emits it on its own title ticks).
    emit_product_title();

    // iTerm2 tab icon: install the DeepSeek logo mapping on first run so the
    // tab shows the official whale right after `npm i -g` — no extra step.
    // Best-effort, silent; `check`/`remove` via scripts/install-iterm-tab-icon.sh.
    match crate::terminal_tab_icon::ensure_iterm2_tab_icon() {
        crate::terminal_tab_icon::TabIconStatus::Installed => {
            eprintln!("Installed DeepSeek Build tab icon for iTerm2.");
        }
        _ => {}
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
    fn product_title_escape_is_osc0_with_bel_terminator() {
        let esc = product_title_escape();
        let s = String::from_utf8(esc).expect("title escape is valid UTF-8");
        assert_eq!(s, "\x1b]0;DeepSeek Build\x07");
    }

    #[test]
    fn toml_escape_quotes() {
        assert_eq!(escape_toml_basic(r#"a"b"#), r#"a\"b"#);
    }

    /// G008 / L2-10-6: production stamp call site writes epoch under product home.
    #[test]
    fn stamp_path_a_prefix_epoch_writes_stable_file() {
        let dir = tempfile::tempdir().unwrap();
        let home = dsb_config::BuildHome::from_path(dir.path());
        stamp_path_a_prefix_epoch(&home);
        let path = dir.path().join("path_a_prefix_epoch.txt");
        let body = std::fs::read_to_string(&path).expect("epoch stamp file");
        assert!(
            body.contains("path_a_prefix_epoch="),
            "missing epoch line: {body}"
        );
        assert!(
            body.contains("path_a_prefix_epoch_short="),
            "missing short epoch: {body}"
        );
        // Second stamp with same cwd/tools → identical file (byte-stable).
        stamp_path_a_prefix_epoch(&home);
        let body2 = std::fs::read_to_string(&path).unwrap();
        assert_eq!(body, body2, "stamp must be byte-stable for identical inputs");
        let hex = body
            .lines()
            .find_map(|l| l.strip_prefix("path_a_prefix_epoch="))
            .expect("epoch hex");
        assert_eq!(hex.len(), 64, "sha256 hex length");
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// G009 / L2-20: production routing stamp Flash → Pro once → Flash.
    #[test]
    fn stamp_path_a_routing_flash_pro_once() {
        let dir = tempfile::tempdir().unwrap();
        let home = dsb_config::BuildHome::from_path(dir.path());
        stamp_path_a_routing(&home);
        let body = std::fs::read_to_string(dir.path().join("path_a_routing.txt"))
            .expect("routing stamp file");
        assert!(
            body.contains("path_a_default_model=deepseek-v4-flash"),
            "{body}"
        );
        assert!(body.contains("path_a_pro_model=deepseek-v4-pro"), "{body}");
        assert!(
            body.contains("after_pro_visibility=model=deepseek-v4-flash"),
            "must return to Flash after /pro once: {body}"
        );
        assert!(body.contains("effort="), "visibility must show effort: {body}");
        assert!(body.contains("thinking="), "visibility must show thinking: {body}");
    }

    /// G010 / L3-50+60: production L3 stamp — fail-closed classify + worker cache law.
    #[test]
    fn stamp_path_a_l3_schedule_and_worker_cache() {
        let dir = tempfile::tempdir().unwrap();
        let home = dsb_config::BuildHome::from_path(dir.path());
        // Seed config so subagents_enabled can be true.
        ensure_product_agent_config(&home).unwrap();
        stamp_path_a_l3(&home);
        let body =
            std::fs::read_to_string(dir.path().join("path_a_l3.txt")).expect("l3 stamp file");
        assert!(body.contains("bash_mutating=true"), "{body}");
        assert!(body.contains("mcp_mutating=true"), "{body}");
        assert!(body.contains("unknown_mutating=true"), "{body}");
        assert!(body.contains("worker_epochs_match=true"), "{body}");
        assert!(body.contains("subagents_enabled_in_config=true"), "{body}");
        assert!(body.contains("worktree_product=opt_in"), "{body}");
        assert!(body.contains("worker_kind_explore=explore"), "{body}");
        assert!(body.contains("worker_kind_implement=implement"), "{body}");
        // Mutating indices must include edit/bash/mcp/unknown (1,2,3,5 after map).
        assert!(body.contains("mu_indices="), "{body}");
        assert!(body.contains("ro_indices="), "{body}");
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
    assert!(
        body.contains("theme = \"deepseeknight-neutral\""),
        "seed missing neutral default theme: {body}"
    );
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
    assert!(body2.contains("theme = \"deepseeknight-neutral\""));
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

#[test]
fn picker_maps_numbers_names_and_defaults() {
    assert_eq!(picker_answer_to_theme("1"), Some(PRODUCT_THEME_BLUE));
    assert_eq!(picker_answer_to_theme("2"), Some(PRODUCT_THEME));
    assert_eq!(picker_answer_to_theme(""), Some(PRODUCT_THEME));
    assert_eq!(picker_answer_to_theme("  "), Some(PRODUCT_THEME));
    assert_eq!(
        picker_answer_to_theme(PRODUCT_THEME_BLUE),
        Some(PRODUCT_THEME_BLUE)
    );
    assert_eq!(picker_answer_to_theme(PRODUCT_THEME), Some(PRODUCT_THEME));
    assert_eq!(
        picker_answer_to_theme("deepseek-night"),
        Some(PRODUCT_THEME_BLUE)
    );
    assert_eq!(
        picker_answer_to_theme("deepseek-night-neutral"),
        Some(PRODUCT_THEME)
    );
    assert_eq!(picker_answer_to_theme("bogus"), None);
    assert_eq!(picker_answer_to_theme("0"), None);
}

#[test]
fn picker_skipped_when_config_already_exists() {
    let dir = tempfile::tempdir().unwrap();
    let home = dsb_config::BuildHome::from_path(dir.path());
    // No config yet -> would prompt (can't assert tty here, but the existence
    // branch is covered by seeding first and confirming the skip below).
    std::fs::write(dir.path().join("config.toml"), "keep=1\n").unwrap();
    assert_eq!(prompt_first_launch_theme(&home), None);
}

#[test]
fn seed_and_repair_honor_explicit_theme() {
    let dir = tempfile::tempdir().unwrap();
    let home = dsb_config::BuildHome::from_path(dir.path());
    // Fresh seed with the blue skin.
    ensure_product_agent_config_with_theme(&home, PRODUCT_THEME_BLUE).unwrap();
    let body = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(
        body.contains(&format!("theme = \"{PRODUCT_THEME_BLUE}\"")),
        "seed should use the requested blue theme: {body}"
    );
    // Repair with a different theme does not clobber an existing choice.
    ensure_product_agent_config_with_theme(&home, PRODUCT_THEME).unwrap();
    let body2 = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert_eq!(body, body2, "repair must not overwrite an explicit theme");
    // Repair on a theme-less file injects the requested theme.
    let fixed = repair_product_agent_config_with_theme("keep=1\n", PRODUCT_THEME_BLUE);
    assert!(fixed.contains(&format!("theme = \"{PRODUCT_THEME_BLUE}\"")));
    assert!(fixed.contains("keep=1"));
}
