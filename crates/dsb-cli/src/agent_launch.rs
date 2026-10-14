//! Launch the DeepSeek Build full-screen agent TUI (product entry).
//!
//! `deepseek-build` / `dsb` with no args on a TTY **exec** the product agent
//! binary (`deepseek-build-agent`), which is the DeepSeek-branded composition
//! root built from the vendored agent tree (not a separate “Grok product” UI).

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use dsb_config::{BuildHome, Provider};
use dsb_provider_deepseek::ReasoningEffort;

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
/// Measured C-balanced DeepSeek palette (original v2 skin; picker option 2).
pub const PRODUCT_THEME_V2: &str = "deepseeknight-v2";
/// Hue-neutral DeepSeek ramp — same accents, gray-ramp legibility (option 3).
pub const PRODUCT_THEME_NEUTRAL: &str = "deepseeknight-neutral";
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

/// OpenRouter OpenAI-compatible base URL (first-run provider choice).
pub const OPENROUTER_API_BASE_URL: &str = "https://openrouter.ai/api/v1";

/// The two product model stanzas (`[model.<slot>]`). Slot keys stay the same
/// for every provider so routing (`/pro`, subagents, `[models] default`) never
/// has to know which endpoint is behind them.
const PRODUCT_MODEL_SLOTS: [&str; 2] = ["deepseek-v4-flash", "deepseek-v4-pro"];

/// Context window (tokens) both product slots report through the DeepSeek
/// API (`GET /models`): V4.1 Flash and V4 Pro both measure 1048576. One value
/// for both slots — the shared stanza template has no divergence to model.
const PRODUCT_CONTEXT_WINDOW: &str = "1048576";

/// Flat `reasoning_efforts` line injected into a DeepSeek model stanza that
/// declares no effort menu. Without this list the vendored `/effort` menu
/// falls back to the upstream legacy rows (xhigh/high/medium/low) and `max`
/// is unreachable for a DeepSeek slot. The duplicate heal removes this exact
/// text, so both sides read one constant.
const PRODUCT_REASONING_EFFORTS_MENU: &str = r#"reasoning_efforts = ["max", "high", "low"]"#;

/// `[models]` side-request keys that otherwise fall back to the vendored
/// default model instead of a product stanza. `prompt_suggestion` is left
/// out on purpose: unset, the per-turn suggestion call is skipped; pointing
/// it at Flash would start a new paid request on every turn.
const PRODUCT_AUX_MODEL_KEYS: [&str; 3] = ["session_summary", "web_search", "image_description"];

/// `[models]` keys that decide which rows the `/model` picker shows.
///
/// The picker renders the vendored ACP model catalog. With no models endpoint
/// configured the built-in Grok catalog merges in, so `disabled_models`
/// removes those rows and `hidden_models` keeps the Pro row out of the picker
/// (Pro stays in the catalog for `/pro`, `/preset max` and subagent routing).
const PRODUCT_PICKER_MODEL_KEYS: [(&str, &str); 2] = [
    ("disabled_models", r#"disabled_models = ["grok-*"]"#),
    ("hidden_models", r#"hidden_models = ["deepseek-v4-pro"]"#),
];

/// Base URL the product model stanzas use for `provider`.
pub fn provider_base_url(provider: Provider) -> &'static str {
    match provider {
        Provider::DeepSeek => DEEPSEEK_API_BASE_URL,
        Provider::OpenRouter => OPENROUTER_API_BASE_URL,
    }
}

/// Wire `model` id for a product slot. OpenRouter only accepts the
/// `provider/model` form, so the same pinned model gets the `deepseek/` prefix.
fn provider_wire_model(provider: Provider, slot: &str) -> String {
    match provider {
        Provider::DeepSeek => slot.to_string(),
        Provider::OpenRouter => format!("deepseek/{slot}"),
    }
}

fn product_slot_display_name(slot: &str) -> &'static str {
    match slot {
        "deepseek-v4-pro" => "DeepSeek V4 Pro",
        _ => "DeepSeek V4.1 Flash",
    }
}

/// One product `[model.<slot>]` stanza for `provider` (leading blank line).
fn product_model_stanza(slot: &str, provider: Provider, api_key: Option<&str>) -> String {
    let wire = provider_wire_model(provider, slot);
    let name = product_slot_display_name(slot);
    let base_url = provider_base_url(provider);
    let env_key = provider.env_key();
    let api_key_line = api_key
        .map(|k| format!("api_key = \"{}\"\n", escape_toml_basic(k)))
        .unwrap_or_default();
    format!(
        r#"
[model.{slot}]
model = "{wire}"
name = "{name}"
system_prompt_label = "{PRODUCT_SYSTEM_PROMPT_LABEL}"
context_window = {PRODUCT_CONTEXT_WINDOW}
api_backend = "chat_completions"
base_url = "{base_url}"
{api_key_line}env_key = "{env_key}"
# Spec 30 / VC008: Path A chat_completions must stamp reasoning_effort on wire.
supports_reasoning_effort = true
reasoning_effort = "high"
# Without this list the vendored /effort menu falls back to the upstream legacy
# rows (xhigh/high/medium/low) and `max` is unreachable for a DeepSeek slot.
reasoning_efforts = ["max", "high", "low"]
"#
    )
}

/// Identity label rendered by the vendored system prompt template
/// (`You are ${{ system_prompt_label }}.`).
///
/// The product must never render the upstream "Grok" fallback
/// (`DEFAULT_SYSTEM_PROMPT_LABEL` in the vendored tree) — vendor patch 0007
/// removed the hardcoded "released by xAI" suffix, and the seed/repair below
/// stamp this label on every DeepSeek model stanza so sessions open with
/// "You are DeepSeek Build." instead of "You are Grok released by xAI.".
pub const PRODUCT_SYSTEM_PROMPT_LABEL: &str = "DeepSeek Build";

/// Prepare product agent config under product home (DeepSeek defaults + theme).
///
/// - Creates `config.toml` when missing (full product seed for the provider
///   chosen in setup; DeepSeek API when nothing was chosen).
/// - If present: inject missing theme; ensure DeepSeek model `base_url` is set
///   so live agent turns hit `api.deepseek.com` (not Grok proxy). Switching an
///   existing file to another provider is [`apply_provider_to_agent_config`].
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

    let creds = dsb_config::Credentials::load(home).ok();
    let provider = creds.as_ref().map_or(Provider::DeepSeek, |c| c.provider());
    let api_key = creds.as_ref().map(|c| c.api_key());
    let base_url = provider_base_url(provider);
    let aux_lines: String = PRODUCT_AUX_MODEL_KEYS
        .iter()
        .map(|key| format!("{key} = \"deepseek-v4-flash\"\n"))
        .collect();
    let picker_lines: String = PRODUCT_PICKER_MODEL_KEYS
        .iter()
        .map(|(_, line)| format!("{line}\n"))
        .collect();
    let stanzas: String = PRODUCT_MODEL_SLOTS
        .iter()
        .map(|slot| product_model_stanza(slot, provider, api_key))
        .collect();

    // Chat Completions backend for DeepSeek (not Grok Responses default).
    // `base_url` on each model is load-bearing for OpenAI-compat providers.
    let body = format!(
        r#"# DeepSeek Build product defaults (auto-created).
# Product chrome: DeepSeek Build + DeepSeek Night theme (#4D6BFE).

[models]
default = "deepseek-v4-flash"
# `/model` picker: drop the vendored Grok catalog rows, and keep the Pro row
# out of the picker (Pro stays reachable via `/pro` and `/preset max`).
{picker_lines}# Side requests (session titles, web search, image description) use the
# product Flash stanza instead of the vendored default model.
{aux_lines}{stanzas}
[endpoints]
xai_api_base_url = "{base_url}"

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
/// 1. DeepSeek Night (classic) theme when no `theme` key exists (product default)
/// 2. `base_url` on DeepSeek model stanzas (or appends full model blocks)
/// 3. Explicit `yolo = false` when the key is missing (Spec 90 product default)
/// 4. Spec 30 / VC008: `supports_reasoning_effort` + default `reasoning_effort`
///    on DeepSeek model stanzas when those keys are missing
/// 5. `[models]` side-request keys (session titles, web search, image
///    description) pointing at the Flash stanza when those keys are missing
/// 6. DeepSeek model display `name`: the stale product default
///    `DeepSeek V4 Flash` becomes `DeepSeek V4.1 Flash` (the model the slot
///    maps to); a user-set name is preserved
/// 7. API-measured `context_window` (1048576) on DeepSeek model stanzas:
///    migrates the legacy `128000` seed, preserves other explicit values,
///    injects the key when missing
/// 8. `[models]` picker selectors (`disabled_models`, `hidden_models`) so the
///    `/model` picker shows the Flash row only
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

    // Product identity: a `[ui] fork_secondary_model` pointing at a Grok model
    // is always stale (DeepSeek Build routes every model to api.deepseek.com).
    // Drop the leftover so the vendored agent uses its own default fork target.
    next = scrub_grok_fork_secondary_model(next);

    next = ensure_deepseek_model_base_url(next, "deepseek-v4-flash");
    next = ensure_deepseek_model_base_url(next, "deepseek-v4-pro");
    next = ensure_deepseek_model_context_window(next, "deepseek-v4-flash");
    next = ensure_deepseek_model_context_window(next, "deepseek-v4-pro");
    next = ensure_deepseek_model_reasoning_effort(next, "deepseek-v4-flash");
    next = ensure_deepseek_model_reasoning_effort(next, "deepseek-v4-pro");
    next = ensure_deepseek_model_system_prompt_label(next, "deepseek-v4-flash");
    next = ensure_deepseek_model_system_prompt_label(next, "deepseek-v4-pro");
    next = ensure_deepseek_model_display_name(next, "deepseek-v4-flash");
    next = ensure_deepseek_model_display_name(next, "deepseek-v4-pro");
    next = ensure_product_aux_models(next);
    // The picker selectors also create a missing `[models]` table.
    next = ensure_product_picker_models(next);

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
fn ensure_deepseek_model_base_url(body: String, model_id: &str) -> String {
    let header = format!("[model.{model_id}]");
    if !body.contains(&header) {
        let mut next = body;
        next.push_str(&product_model_stanza(model_id, Provider::DeepSeek, None));
        return next;
    }

    // Inject base_url into the model section if missing (section-scoped).
    inject_model_section_keys_if_missing(
        body,
        &header,
        &[(
            "base_url",
            &format!("base_url = \"{DEEPSEEK_API_BASE_URL}\""),
        )],
    )
}

/// Context window: DeepSeek model stanzas must carry the API-measured window
/// ([`PRODUCT_CONTEXT_WINDOW`]).
///
/// An install seeded before the V4.1 window keeps `128000` forever — the
/// provider retarget rewrites `base_url` / `model` / `env_key` / `api_key`
/// only — so the legacy value is migrated on launch. Other explicit values
/// are preserved; a missing key is injected because the vendored
/// `ModelEntryConfig.context_window` is a required `NonZeroU64` with no serde
/// default. Idempotent.
fn ensure_deepseek_model_context_window(body: String, model_id: &str) -> String {
    let header = format!("[model.{model_id}]");
    if !body.contains(&header) {
        return body;
    }
    let mut out: Vec<String> = Vec::new();
    let mut in_section = false;
    let mut found = false;
    let mut patched = false;
    for line in body.lines() {
        let t = line.trim();
        if t == header {
            in_section = true;
            found = false;
            out.push(line.to_string());
            continue;
        }
        if in_section && t.starts_with('[') {
            if !found {
                out.push(format!("context_window = {PRODUCT_CONTEXT_WINDOW}"));
                patched = true;
            }
            in_section = false;
        }
        let key = t.split_once('=').map(|(key, _)| key.trim());
        if in_section && key == Some("context_window") {
            found = true;
            let legacy = t
                .split_once('=')
                .is_some_and(|(_, rhs)| toml_rhs_without_comment(rhs).trim() == "128000");
            if legacy {
                out.push(format!("context_window = {PRODUCT_CONTEXT_WINDOW}"));
                patched = true;
                continue;
            }
        }
        out.push(line.to_string());
    }
    // Section ran to end of file with no key and no closing header.
    if in_section && !found {
        out.push(format!("context_window = {PRODUCT_CONTEXT_WINDOW}"));
        patched = true;
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

/// Point `[models]` side requests at the Flash stanza when the keys are absent.
///
/// Only edits an existing `[models]` table; never overwrites a user value.
fn ensure_product_aux_models(body: String) -> String {
    let lines: Vec<(&str, String)> = PRODUCT_AUX_MODEL_KEYS
        .iter()
        .map(|key| (*key, format!("{key} = \"deepseek-v4-flash\"")))
        .collect();
    let keys: Vec<(&str, &str)> = lines.iter().map(|(k, l)| (*k, l.as_str())).collect();
    inject_model_section_keys_if_missing(body, "[models]", &keys)
}

/// Restrict the `/model` picker to the product rows for an existing home.
///
/// The picker renders the vendored ACP model catalog. Nothing in this repo
/// sets a models endpoint, so the built-in Grok catalog applies and its rows
/// (`grok-*`) merge in. `disabled_models` removes those rows from the catalog
/// and `hidden_models` keeps the Pro row out of the picker; the Pro stanza
/// itself stays, so `/pro` and `/preset max` still route to it. Existing
/// values are never rewritten, and a second run is a no-op.
fn ensure_product_picker_models(body: String) -> String {
    let mut lines: Vec<String> = body.lines().map(str::to_string).collect();
    if let Some((start, end)) = toml_section_range(&lines, "[models]") {
        let mut insert_at = end;
        let mut patched = false;
        for (key, full_line) in PRODUCT_PICKER_MODEL_KEYS {
            if toml_section_key_index(&lines, (start, end), key).is_none() {
                lines.insert(insert_at, full_line.to_string());
                insert_at += 1;
                patched = true;
            }
        }
        if !patched {
            return body;
        }
        let mut next = lines.join("\n");
        if !next.ends_with('\n') {
            next.push('\n');
        }
        return next;
    }

    // No `[models]` table at all: append a product one at end of file so the
    // picker loses the Grok rows on the next launch.
    let mut next = body;
    if !next.ends_with('\n') {
        next.push('\n');
    }
    next.push_str("\n[models]\n");
    for (_, full_line) in PRODUCT_PICKER_MODEL_KEYS {
        next.push_str(full_line);
        next.push('\n');
    }
    next
}

/// Spec 30 / VC008: ensure DeepSeek model stanzas enable effort, default high,
/// and expose the DeepSeek effort menu (`max`/`high`/`low`) so a standing
/// `default_reasoning_effort = "max"` is selectable in the vendored `/effort`.
///
/// Does not overwrite an explicit user `reasoning_effort` /
/// `reasoning_efforts` / `supports_reasoning_effort` — a stanza whose menu is
/// the documented array of tables keeps it, and the duplicate flat line an
/// older version injected next to it is dropped first.
fn ensure_deepseek_model_reasoning_effort(body: String, model_id: &str) -> String {
    let header = format!("[model.{model_id}]");
    if !body.contains(&header) {
        return body;
    }
    let body = drop_duplicate_flat_reasoning_efforts(body, model_id);
    inject_model_section_keys_if_missing(
        body,
        &header,
        &[
            (
                "supports_reasoning_effort",
                "supports_reasoning_effort = true",
            ),
            ("reasoning_effort", "reasoning_effort = \"high\""),
            ("reasoning_efforts", PRODUCT_REASONING_EFFORTS_MENU),
        ],
    )
}

/// Drop the flat product effort menu from a stanza that declares
/// `reasoning_efforts` as an array of tables.
///
/// Older versions read the presence scan as "key missing" when the stanza's
/// only menu was `[[model.<id>.reasoning_efforts]]` entries (the scan stops at
/// the first header line) and injected the flat menu next to them: a duplicate
/// key that makes `config.toml` unparseable, so the vendored agent cannot
/// start. Only the exact product line is dropped; any other flat value is
/// hand-written, and both expressions are left for the user to resolve.
fn drop_duplicate_flat_reasoning_efforts(body: String, model_id: &str) -> String {
    let header = format!("[model.{model_id}]");
    if !body.contains(&header) {
        return body;
    }
    let parent = format!("model.{model_id}");
    let table_form_present = body.lines().any(|line| {
        matches!(
            table_header_child_key(line, &parent),
            Some(("reasoning_efforts", true))
        )
    });
    if !table_form_present {
        return body;
    }
    let mut out: Vec<&str> = Vec::with_capacity(body.lines().count());
    let mut in_section = false;
    let mut changed = false;
    for line in body.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_section = t == header;
            out.push(line);
            continue;
        }
        if in_section && is_product_reasoning_efforts_menu(t) {
            changed = true;
            continue;
        }
        out.push(line);
    }
    if !changed {
        return body;
    }
    let mut s = out.join("\n");
    if !s.ends_with('\n') {
        s.push('\n');
    }
    s
}

/// True when `t` is the flat effort menu line [`PRODUCT_REASONING_EFFORTS_MENU`]
/// writes (spacing-insensitive).
fn is_product_reasoning_efforts_menu(t: &str) -> bool {
    let Some((key, rhs)) = t.split_once('=') else {
        return false;
    };
    key.trim() == "reasoning_efforts"
        && toml_rhs_without_comment(rhs).trim() == r#"["max", "high", "low"]"#
}

/// Standing reasoning effort for the product Flash slot, read from `config.toml`.
///
/// Mirrors the vendored resolution (`remote_config/resolution.rs`): the
/// `[models] default_reasoning_effort` pin applies to the entry named by
/// `[models] default`; without that pin the stanza's own `reasoning_effort`
/// stands. `None` keeps the shipped default (`ReasoningEffort::High`).
pub fn configured_default_reasoning_effort(home: &BuildHome) -> Option<ReasoningEffort> {
    let body = std::fs::read_to_string(home.path().join("config.toml")).ok()?;
    let lines: Vec<String> = body.lines().map(str::to_string).collect();
    let section_value = |header: &str, key: &str| -> Option<String> {
        let range = toml_section_range(&lines, header)?;
        let idx = toml_section_key_index(&lines, range, key)?;
        toml_rhs_string(&lines[idx]).map(str::to_string)
    };
    if section_value("[models]", "default").as_deref() == Some("deepseek-v4-flash")
        && let Some(effort) = section_value("[models]", "default_reasoning_effort")
    {
        return Some(ReasoningEffort::from_product(&effort));
    }
    section_value("[model.deepseek-v4-flash]", "reasoning_effort")
        .as_deref()
        .map(ReasoningEffort::from_product)
}

/// Product identity: DeepSeek model stanzas must carry the DeepSeek
/// system-prompt label, never the vendored "Grok" fallback.
///
/// Adds the label when the section lacks it, and replaces a stale
/// `grok`/`xai`-valued label (e.g. a model stanza copied from a real Grok CLI
/// config). A user-set non-Grok label is preserved. Idempotent.
fn ensure_deepseek_model_system_prompt_label(body: String, model_id: &str) -> String {
    let header = format!("[model.{model_id}]");
    if !body.contains(&header) {
        return body;
    }
    let mut out: Vec<String> = Vec::new();
    let mut in_section = false;
    let mut found = false;
    let mut patched = false;
    for line in body.lines() {
        let t = line.trim();
        if t == header {
            in_section = true;
            found = false;
            out.push(line.to_string());
            continue;
        }
        if in_section && t.starts_with('[') {
            if !found {
                out.push(format!(
                    "system_prompt_label = \"{PRODUCT_SYSTEM_PROMPT_LABEL}\""
                ));
                patched = true;
            }
            in_section = false;
        }
        let key = t.split_once('=').map(|(key, _)| key.trim());
        if in_section && key == Some("system_prompt_label") {
            found = true;
            if toml_rhs_contains_any(t, &["grok", "xai"]) {
                out.push(format!(
                    "system_prompt_label = \"{PRODUCT_SYSTEM_PROMPT_LABEL}\""
                ));
                patched = true;
                continue;
            }
        }
        out.push(line.to_string());
    }
    // Section ran to end of file with no label and no closing header.
    if in_section && !found {
        out.push(format!(
            "system_prompt_label = \"{PRODUCT_SYSTEM_PROMPT_LABEL}\""
        ));
        patched = true;
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

/// Display name earlier product versions seeded for `slot`. Repair replaces
/// exactly this value with the current one and leaves any other name alone.
fn product_slot_stale_display_name(slot: &str) -> &'static str {
    match slot {
        "deepseek-v4-pro" => "DeepSeek V4 Pro",
        _ => "DeepSeek V4 Flash",
    }
}

/// Product display name: DeepSeek model stanzas must carry the current
/// product name (`DeepSeek V4.1 Flash`), not the stale seed from earlier
/// versions.
///
/// Adds the name when the section lacks it, replaces the stale product
/// default, and preserves any user-set name. Idempotent.
fn ensure_deepseek_model_display_name(body: String, model_id: &str) -> String {
    let header = format!("[model.{model_id}]");
    if !body.contains(&header) {
        return body;
    }
    let name = product_slot_display_name(model_id);
    let stale = product_slot_stale_display_name(model_id);
    let mut out: Vec<String> = Vec::new();
    let mut in_section = false;
    let mut found = false;
    let mut patched = false;
    for line in body.lines() {
        let t = line.trim();
        if t == header {
            in_section = true;
            found = false;
            out.push(line.to_string());
            continue;
        }
        if in_section && t.starts_with('[') {
            if !found {
                out.push(format!("name = \"{name}\""));
                patched = true;
            }
            in_section = false;
        }
        let key = t.split_once('=').map(|(key, _)| key.trim());
        if in_section && key == Some("name") {
            found = true;
            if toml_rhs_string(t) == Some(stale) {
                out.push(format!("name = \"{name}\""));
                patched = true;
                continue;
            }
        }
        out.push(line.to_string());
    }
    // Section ran to end of file with no name and no closing header.
    if in_section && !found {
        out.push(format!("name = \"{name}\""));
        patched = true;
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

/// Remove a Grok-model leftover from `[ui] fork_secondary_model`.
///
/// DeepSeek Build routes every model to `api.deepseek.com`; a `grok-*` fork
/// target is always stale (typically copied from a real Grok CLI
/// `config.toml`). Dropping the key lets the vendored agent use its own
/// default fork behavior. Idempotent.
fn scrub_grok_fork_secondary_model(body: String) -> String {
    let mut out: Vec<&str> = Vec::with_capacity(body.lines().count());
    let mut changed = false;
    let mut in_ui_section = false;
    for line in body.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_ui_section = t == "[ui]";
        }
        let key = t.split_once('=').map(|(key, _)| key.trim());
        if in_ui_section
            && key == Some("fork_secondary_model")
            && toml_rhs_contains_any(t, &["grok"])
        {
            changed = true;
            continue;
        }
        out.push(line);
    }
    if changed {
        let mut s = out.join("\n");
        if !s.ends_with('\n') {
            s.push('\n');
        }
        s
    } else {
        body
    }
}

fn toml_rhs_contains_any(line: &str, needles: &[&str]) -> bool {
    let Some((_, rhs)) = line.split_once('=') else {
        return false;
    };
    let value = toml_rhs_without_comment(rhs).trim();
    let value = value
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .or_else(|| value.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
        .unwrap_or(value);
    let lower = value.to_ascii_lowercase();
    needles.iter().any(|needle| lower.contains(needle))
}

fn toml_rhs_without_comment(rhs: &str) -> &str {
    let mut string_quote: Option<char> = None;
    let mut escaped = false;
    for (idx, ch) in rhs.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if string_quote == Some('"') => escaped = true,
            '"' | '\'' if string_quote == Some(ch) => string_quote = None,
            '"' | '\'' if string_quote.is_none() => string_quote = Some(ch),
            '#' if string_quote.is_none() => return &rhs[..idx],
            _ => {}
        }
    }
    rhs
}

/// Child key path when `line` is a table header inside `parent`'s namespace:
/// `[<parent>.<key>]` and `[[<parent>.<key>]]` (array of tables) both return
/// `(<key>.<sub…>, is_array)`. The parent's own header and unrelated tables
/// return `None`.
///
/// The vendored schema declares some keys as tables — the documented
/// `model.<id>.reasoning_efforts` form is an array of tables — so a presence
/// scan for such a key has to see these headers, not only flat key lines.
fn table_header_child_key<'a>(line: &'a str, parent: &str) -> Option<(&'a str, bool)> {
    let trimmed = toml_rhs_without_comment(line.trim()).trim();
    let (inner, array) = if let Some(rest) = trimmed.strip_prefix("[[") {
        (rest.strip_suffix("]]")?, true)
    } else {
        (trimmed.strip_prefix('[')?.strip_suffix(']')?, false)
    };
    let child = inner.strip_prefix(parent)?.strip_prefix('.')?;
    if child.is_empty() {
        return None;
    }
    Some((child, array))
}

/// Inject TOML keys into a `[model.*]` section when absent (section-scoped).
///
/// `keys` are `(prefix, full_line)` where `prefix` matches the start of a
/// non-comment line in the section (e.g. `"base_url"`, `"reasoning_effort"`),
/// or the first path segment of a child table header
/// (`[[model.<id>.reasoning_efforts]]`).
fn inject_model_section_keys_if_missing(
    body: String,
    header: &str,
    keys: &[(&str, &str)],
) -> String {
    let lines: Vec<&str> = body.lines().collect();
    let parent = header.trim_start_matches('[').trim_end_matches(']');
    let mut out: Vec<String> = Vec::with_capacity(lines.len() + keys.len());
    let mut i = 0;
    let mut patched = false;
    while i < lines.len() {
        let line = lines[i];
        out.push(line.to_string());
        if line.trim() == header {
            let mut j = i + 1;
            let mut present = vec![false; keys.len()];
            while j < lines.len() {
                let t = lines[j].trim();
                if t.starts_with('[') {
                    break;
                }
                for (idx, (prefix, _)) in keys.iter().enumerate() {
                    if t.starts_with(prefix) {
                        present[idx] = true;
                    }
                }
                j += 1;
            }
            // Child tables may sit anywhere later in the body — the flat
            // region above ends at their header, so a stanza whose only
            // `reasoning_efforts` is `[[model.<id>.reasoning_efforts]]` entries
            // would otherwise read as "key missing".
            for line in &lines[i + 1..] {
                let Some((child, _)) = table_header_child_key(line, parent) else {
                    continue;
                };
                for (idx, (prefix, _)) in keys.iter().enumerate() {
                    if child.starts_with(prefix) {
                        present[idx] = true;
                    }
                }
            }
            for (idx, (_, full_line)) in keys.iter().enumerate() {
                if !present[idx] {
                    out.push(full_line.to_string());
                    patched = true;
                }
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

/// Point an existing product `config.toml` at `provider` after setup saved
/// `api_key` for it. No-op when the file does not exist yet (the seed in
/// [`ensure_product_agent_config`] follows the saved provider).
///
/// Returns the product stanzas that were left alone because they are
/// hand-edited for another endpoint (see [`retarget_product_config`]).
pub fn apply_provider_to_agent_config(
    home: &BuildHome,
    provider: Provider,
    api_key: &str,
) -> Result<Vec<&'static str>> {
    let config_path = home.path().join("config.toml");
    if !config_path.exists() {
        return Ok(Vec::new());
    }
    let body = std::fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let (next, kept_custom) = retarget_product_config(&body, provider, api_key);
    if next != body {
        std::fs::write(&config_path, next)
            .with_context(|| format!("update {}", config_path.display()))?;
    }
    Ok(kept_custom)
}

/// Pure rewrite behind [`apply_provider_to_agent_config`].
///
/// Each product stanza (`[model.deepseek-v4-flash]`, `[model.deepseek-v4-pro]`)
/// is handled as a whole:
/// - still the *other* provider's product default (`base_url`, `model`,
///   `env_key`; a missing `base_url` / `model` counts as DeepSeek because
///   repair and the agent fall back to that) → rewritten to `provider`'s
///   defaults with the new `api_key`;
/// - already on `provider`'s product URL (possibly hand-tuned) → only an
///   inline `api_key` is replaced, so a stale key cannot shadow the new one;
/// - anything else is hand-edited for another endpoint → left untouched and
///   returned, so setup can say so instead of half-rewriting it.
///
/// A missing stanza is appended for `provider`. `[endpoints]
/// xai_api_base_url` follows only when no stanza was kept custom.
fn retarget_product_config(
    body: &str,
    provider: Provider,
    api_key: &str,
) -> (String, Vec<&'static str>) {
    let other = match provider {
        Provider::DeepSeek => Provider::OpenRouter,
        Provider::OpenRouter => Provider::DeepSeek,
    };
    let new_base = provider_base_url(provider);
    let mut lines: Vec<String> = body.lines().map(str::to_string).collect();
    let mut appended = String::new();
    let mut kept_custom = Vec::new();
    let mut changed = false;

    for slot in PRODUCT_MODEL_SLOTS {
        let header = format!("[model.{slot}]");
        let Some(range) = toml_section_range(&lines, &header) else {
            // A header spelled another way (quoted key, …) still declares the
            // table; appending one would make the file declare it twice.
            if lines
                .iter()
                .any(|l| l.trim_start().starts_with('[') && l.contains(slot))
            {
                kept_custom.push(slot);
            } else {
                appended.push_str(&product_model_stanza(slot, provider, Some(api_key)));
            }
            continue;
        };
        // `Some(None)`: the key is there but not a one-line string (array,
        // multi-line value, …) — hand-written, so the stanza is not ours.
        let field = |key: &str| {
            toml_section_key_index(&lines, range, key)
                .map(|i| toml_rhs_string(&lines[i]).map(str::to_string))
        };
        let [base, model, env_key, inline_key] =
            ["base_url", "model", "env_key", "api_key"].map(field);
        if [&base, &model, &env_key, &inline_key]
            .iter()
            .any(|f| matches!(f, Some(None)))
        {
            kept_custom.push(slot);
            continue;
        }
        let base = base
            .flatten()
            .unwrap_or_else(|| DEEPSEEK_API_BASE_URL.to_string());
        let model = model.flatten().unwrap_or_else(|| slot.to_string());
        let env_key = env_key.flatten();
        let has_inline_key = inline_key.is_some();
        let is_default_for = |p: Provider| {
            base == provider_base_url(p)
                && model == provider_wire_model(p, slot)
                && env_key.as_deref().is_none_or(|e| e == p.env_key())
        };
        let wanted: Vec<(&str, String)> = if is_default_for(other) {
            vec![
                ("base_url", new_base.to_string()),
                ("model", provider_wire_model(provider, slot)),
                ("env_key", provider.env_key().to_string()),
                ("api_key", api_key.to_string()),
            ]
        } else if base == new_base {
            if has_inline_key {
                vec![("api_key", api_key.to_string())]
            } else {
                Vec::new()
            }
        } else {
            kept_custom.push(slot);
            continue;
        };
        for (key, value) in wanted {
            let line = format!("{key} = \"{}\"", escape_toml_basic(&value));
            let range = toml_section_range(&lines, &header).unwrap_or(range);
            match toml_section_key_index(&lines, range, key) {
                Some(i) => changed |= set_line(&mut lines, i, line),
                None => {
                    lines.insert(range.0, line);
                    changed = true;
                }
            }
        }
    }

    if kept_custom.is_empty()
        && let Some(range) = toml_section_range(&lines, "[endpoints]")
        && let Some(i) = toml_section_key_index(&lines, range, "xai_api_base_url")
        && toml_rhs_string(&lines[i]) == Some(provider_base_url(other))
    {
        changed |= set_line(&mut lines, i, format!("xai_api_base_url = \"{new_base}\""));
    }

    if !changed && appended.is_empty() {
        return (body.to_string(), kept_custom);
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out.push_str(&appended);
    (out, kept_custom)
}

/// Replace line `i`; true when the text actually changed.
fn set_line(lines: &mut [String], i: usize, line: String) -> bool {
    if lines[i] == line {
        return false;
    }
    lines[i] = line;
    true
}

/// Provider the existing agent config is pointed at, read from the product
/// stanzas' `base_url`.
///
/// When `credentials.json` is gone (`auth logout`, a lost file) the choice is
/// still visible in the config the home was switched to, so a provider-less
/// `setup --api-key …` keeps that endpoint instead of saving a key for the
/// *other* provider and rewriting the stanzas to it.
pub fn configured_provider(home: &BuildHome) -> Option<Provider> {
    let body = std::fs::read_to_string(home.path().join("config.toml")).ok()?;
    provider_of_config(&body)
}

/// The product stanzas that carry a `base_url` must all name the same
/// provider; no stanza, a mixture, a value that is not a one-line string, or
/// any other endpoint returns `None` rather than guessing.
fn provider_of_config(body: &str) -> Option<Provider> {
    let lines: Vec<String> = body.lines().map(str::to_string).collect();
    let mut found: Option<Provider> = None;
    for slot in PRODUCT_MODEL_SLOTS {
        let Some(range) = toml_section_range(&lines, &format!("[model.{slot}]")) else {
            continue;
        };
        let Some(i) = toml_section_key_index(&lines, range, "base_url") else {
            continue;
        };
        let provider = match toml_rhs_string(&lines[i])? {
            DEEPSEEK_API_BASE_URL => Provider::DeepSeek,
            OPENROUTER_API_BASE_URL => Provider::OpenRouter,
            _ => return None,
        };
        match found {
            Some(seen) if seen != provider => return None,
            _ => found = Some(provider),
        }
    }
    found
}

/// True when the default model stanza (`[models] default`) has an `env_key`
/// whose variable is set — the agent will authenticate from the environment
/// even without `credentials.json` / `DEEPSEEK_API_KEY` (e.g. a hand-written
/// OpenRouter stanza with its own variable). Inline `api_key` is ignored on
/// purpose: after `auth logout` the product-written copy should not
/// suppress setup.
pub fn configured_model_env_key_available(home: &BuildHome) -> bool {
    configured_model_env_key_available_in(home, |name| env::var(name).ok())
}

/// [`configured_model_env_key_available`] with an injected env lookup.
pub(crate) fn configured_model_env_key_available_in(
    home: &BuildHome,
    env_lookup: impl Fn(&str) -> Option<String>,
) -> bool {
    let Ok(body) = std::fs::read_to_string(home.path().join("config.toml")) else {
        return false;
    };
    configured_model_env_key_available_with(&body, env_lookup)
}

fn configured_model_env_key_available_with(
    body: &str,
    env_lookup: impl Fn(&str) -> Option<String>,
) -> bool {
    let lines: Vec<String> = body.lines().map(str::to_string).collect();
    let default_slot = toml_section_range(&lines, "[models]")
        .and_then(|r| toml_section_key_index(&lines, r, "default"))
        .and_then(|i| toml_rhs_string(&lines[i]).map(str::to_string))
        .unwrap_or_else(|| PRODUCT_MODEL_SLOTS[0].to_string());
    let Some(range) = toml_section_range(&lines, &format!("[model.{default_slot}]")) else {
        return false;
    };
    let Some(i) = toml_section_key_index(&lines, range, "env_key") else {
        return false;
    };
    let Some((_, rhs)) = lines[i].split_once('=') else {
        return false;
    };
    // `env_key` is a string or an array of strings (vendored config schema).
    toml_rhs_without_comment(rhs)
        .split(['"', '\''])
        .skip(1)
        .step_by(2)
        .any(|name| env_lookup(name).is_some_and(|v| !v.trim().is_empty()))
}

/// `[start, end)` line indices of the body of `header`'s table.
///
/// Matches the header ignoring whitespace and a trailing comment
/// (`[ model.x ]  # main` is `[model.x]`).
fn toml_section_range(lines: &[String], header: &str) -> Option<(usize, usize)> {
    let is_header = |l: &String| {
        let code = l.split('#').next().unwrap_or_default();
        code.chars()
            .filter(|c| !c.is_whitespace())
            .eq(header.chars())
    };
    let start = lines.iter().position(is_header)? + 1;
    let end = lines[start..]
        .iter()
        .position(|l| l.trim_start().starts_with('['))
        .map_or(lines.len(), |i| start + i);
    Some((start, end))
}

/// Index of the exact `key = …` line inside `range` (comments skipped).
fn toml_section_key_index(lines: &[String], range: (usize, usize), key: &str) -> Option<usize> {
    (range.0..range.1).find(|&i| {
        let t = lines[i].trim();
        !t.starts_with('#') && t.split_once('=').is_some_and(|(k, _)| k.trim() == key)
    })
}

/// Unquoted value of a `key = "value"` line (basic or literal string).
fn toml_rhs_string(line: &str) -> Option<&str> {
    let (_, rhs) = line.split_once('=')?;
    let value = toml_rhs_without_comment(rhs).trim();
    value
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .or_else(|| value.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
}

fn escape_toml_basic(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// First-launch theme picker (fresh product home + interactive tty only).
///
/// Asks the user to choose between the original blue-tinted `deepseeknight`
/// skin (default), the measured `deepseeknight-v2` palette, and the hue-neutral
/// `deepseeknight-neutral` ramp. Returns the chosen canonical name, or `None` when:
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
            "  1) DeepSeek Night (classic) — original blue-tinted skin (default)"
        );
        let _ = writeln!(
            out,
            "  2) DeepSeek Night v2         — measured C-balanced palette"
        );
        let _ = writeln!(
            out,
            "  3) DeepSeek Night Neutral   — hue-neutral gray ramp, blue accents"
        );
        let _ = write!(out, "Select [1]: ");
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

/// Pure mapping from picker input to a theme canonical ("" => product default).
fn picker_answer_to_theme(answer: &str) -> Option<&'static str> {
    match answer.trim() {
        "1" | "" | PRODUCT_THEME | "deepseeknight" | "deepseek-night" | "deepseek" | "dsb" => {
            Some(PRODUCT_THEME)
        }
        "2" | PRODUCT_THEME_V2 | "deepseeknight-v2" | "deepseek-night-v2" | "dsb2" => {
            Some(PRODUCT_THEME_V2)
        }
        "3"
        | PRODUCT_THEME_NEUTRAL
        | "deepseeknight-neutral"
        | "deepseek-neutral"
        | "dsb-neutral" => Some(PRODUCT_THEME_NEUTRAL),
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
    let _ = writeln!(out);
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
    router.set_default_effort(configured_default_reasoning_effort(home));
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
        MAX_PARALLEL_READONLY, WorkerKind, is_mutating_tool, partition_indices,
        worker_stable_prefix,
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

/// Seed the pager's changelog disk cache with the product changelog.
///
/// The vendored pager (`xai-grok-pager`) fetches per-version changelogs from
/// Grok's CDN (`https://x.ai/cli/changelogs/{version}.external.md`) and falls
/// back to `$GROK_HOME/CHANGELOG.md` on a miss. Product versions (`5.x.y`)
/// are never published there, so without a seed the welcome-screen CHANGELOG
/// click and `/release-notes` silently no-op. Writing the repo `CHANGELOG.md`
/// (embedded at build time) into that cache path makes both open the product
/// changelog — offline, matching the Grok Build UX.
///
/// Best-effort: writes only when missing or stale; failures never block
/// agent launch.
fn seed_product_changelog(home: &BuildHome) {
    const PRODUCT_CHANGELOG: &str = include_str!("../../../CHANGELOG.md");
    let _ = home.ensure_dir();
    let path = home.path().join("CHANGELOG.md");
    let up_to_date = std::fs::read_to_string(&path)
        .map(|existing| existing == PRODUCT_CHANGELOG)
        .unwrap_or(false);
    if !up_to_date {
        let _ = std::fs::write(&path, PRODUCT_CHANGELOG);
    }
}

/// Saved key to hand the agent as `(variable, value)`: under the provider's own
/// `env_key` variable, and only when the shell did not set it. Keyed by
/// provider so an OpenRouter key is never exported as DEEPSEEK_API_KEY (and
/// vice versa).
fn agent_key_env(
    creds: &dsb_config::Credentials,
    is_set: impl Fn(&str) -> bool,
) -> Option<(&'static str, &str)> {
    let var = creds.provider().env_key();
    (!is_set(var)).then_some((var, creds.api_key()))
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
    // Changelog port fix: the Grok CDN has no changelog for product versions
    // (5.x.y), so seed the pager's disk cache — otherwise the welcome-screen
    // CHANGELOG click and `/release-notes` silently no-op (non-blocking).
    seed_product_changelog(&home);
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
    if let Ok(c) = dsb_config::Credentials::load(&home)
        && let Some((var, key)) = agent_key_env(&c, |name| env::var_os(name).is_some())
    {
        cmd.env(var, key);
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
        assert_eq!(
            body, body2,
            "stamp must be byte-stable for identical inputs"
        );
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
        assert!(
            body.contains("effort="),
            "visibility must show effort: {body}"
        );
        assert!(
            body.contains("thinking="),
            "visibility must show thinking: {body}"
        );
    }

    /// The standing `max` pin (`[models] default_reasoning_effort`) must reach
    /// the production routing stamp on the Flash slot.
    #[test]
    fn stamp_path_a_routing_honours_standing_max_pin() {
        let dir = tempfile::tempdir().unwrap();
        let home = dsb_config::BuildHome::from_path(dir.path());
        std::fs::write(
            dir.path().join("config.toml"),
            r#"[models]
default = "deepseek-v4-flash"
default_reasoning_effort = "max"

[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
reasoning_effort = "high"
"#,
        )
        .unwrap();
        stamp_path_a_routing(&home);
        let body = std::fs::read_to_string(dir.path().join("path_a_routing.txt"))
            .expect("routing stamp file");
        assert!(
            body.contains("flash_visibility=model=deepseek-v4-flash thinking=on effort=max"),
            "pin must win over the stanza's high: {body}"
        );
        assert!(
            body.contains("after_pro_visibility=model=deepseek-v4-flash thinking=on effort=max"),
            "Flash returns at the pinned effort: {body}"
        );
    }

    /// `configured_default_reasoning_effort` mirrors the vendored resolution:
    /// the `[models]` pin applies to the Flash entry it names; the Flash
    /// stanza's own `reasoning_effort` stands otherwise.
    #[test]
    fn configured_default_reasoning_effort_sources() {
        let dir = tempfile::tempdir().unwrap();
        let home = dsb_config::BuildHome::from_path(dir.path());
        assert_eq!(
            configured_default_reasoning_effort(&home),
            None,
            "no config file keeps the shipped default"
        );
        std::fs::write(
            dir.path().join("config.toml"),
            r#"[models]
default = "deepseek-v4-flash"
default_reasoning_effort = "max"

[model.deepseek-v4-flash]
reasoning_effort = "low"
"#,
        )
        .unwrap();
        assert_eq!(
            configured_default_reasoning_effort(&home),
            Some(ReasoningEffort::Max),
            "the [models] pin wins over the stanza value"
        );
        std::fs::write(
            dir.path().join("config.toml"),
            r#"[models]
default = "deepseek-v4-pro"
default_reasoning_effort = "max"

[model.deepseek-v4-flash]
reasoning_effort = "low"
"#,
        )
        .unwrap();
        assert_eq!(
            configured_default_reasoning_effort(&home),
            Some(ReasoningEffort::Low),
            "a pin aimed at another entry leaves the Flash stanza standing"
        );
        std::fs::write(
            dir.path().join("config.toml"),
            "[models]\ndefault = \"deepseek-v4-flash\"\n",
        )
        .unwrap();
        assert_eq!(
            configured_default_reasoning_effort(&home),
            None,
            "no pin and no stanza effort keeps the shipped default"
        );
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

    /// Changelog port fix: pager cache seed writes the repo changelog.
    #[test]
    fn seed_product_changelog_writes_repo_changelog_to_pager_cache() {
        let dir = tempfile::tempdir().unwrap();
        let home = dsb_config::BuildHome::from_path(dir.path());
        seed_product_changelog(&home);
        let path = dir.path().join("CHANGELOG.md");
        let body = std::fs::read_to_string(&path).expect("seeded changelog file");
        assert!(
            body.starts_with("# Changelog"),
            "must be the repo CHANGELOG.md: {body}"
        );
        assert!(
            body.contains("## Unreleased"),
            "newest-first changelog invariant missing: {body}"
        );
        // Up-to-date cache is not rewritten (byte-stable, no churn).
        seed_product_changelog(&home);
        let body2 = std::fs::read_to_string(&path).unwrap();
        assert_eq!(body, body2, "re-seed must not rewrite an up-to-date cache");
        // Stale cache (e.g. leftover from an older release) is refreshed.
        std::fs::write(&path, "stale\n").unwrap();
        seed_product_changelog(&home);
        let body3 = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            body3, body,
            "stale cache must be refreshed to repo changelog"
        );
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
        body.contains("theme = \"deepseeknight\""),
        "seed missing classic default theme: {body}"
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
    // Spec 30 / VC008: Path A wire effort defaults.
    assert!(
        body.contains("supports_reasoning_effort = true"),
        "seed missing supports_reasoning_effort: {body}"
    );
    assert!(
        body.contains("reasoning_effort = \"high\""),
        "seed missing reasoning_effort high: {body}"
    );
    // Defect 4: without an explicit menu the vendored /effort falls back to the
    // upstream legacy rows (xhigh/high/medium/low) and can never offer (max).
    assert!(
        body.contains("reasoning_efforts = [\"max\", \"high\", \"low\"]"),
        "seed missing effort menu: {body}"
    );
    // Defect 2: the Flash slot displays the model it maps to
    // (`DeepSeek V4.1 Flash`), never the older `DeepSeek V4 Flash` seed.
    assert!(
        body.contains("name = \"DeepSeek V4.1 Flash\""),
        "seed missing current Flash display name: {body}"
    );
    assert!(
        !body.contains("name = \"DeepSeek V4 Flash\""),
        "seed leaked stale Flash display name: {body}"
    );
    // V4.1 Flash / V4 Pro API window: both stanzas seed 1048576, never 128000.
    let flash_idx = body.find("[model.deepseek-v4-flash]").unwrap();
    let pro_idx = body.find("[model.deepseek-v4-pro]").unwrap();
    assert!(
        body[flash_idx..pro_idx].contains(&format!("context_window = {PRODUCT_CONTEXT_WINDOW}")),
        "seed missing flash context_window: {body}"
    );
    assert!(
        body[pro_idx..].contains(&format!("context_window = {PRODUCT_CONTEXT_WINDOW}")),
        "seed missing pro context_window: {body}"
    );
    assert!(
        !body.contains("128000"),
        "legacy 128000 window must not be seeded: {body}"
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
    // The picker `[models]` table is appended on pass 1; the side-request keys
    // then land inside it on pass 2, and pass 3 is the fixpoint.
    ensure_product_agent_config(&home).unwrap();
    let body3 = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    ensure_product_agent_config(&home).unwrap();
    let body4 = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert_eq!(body3, body4);
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
fn repair_injects_reasoning_effort_defaults_without_clobber() {
    let raw = r#"
[models]
default = "deepseek-v4-flash"

[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
api_backend = "chat_completions"
base_url = "https://api.deepseek.com"
env_key = "DEEPSEEK_API_KEY"

[model.deepseek-v4-pro]
model = "deepseek-v4-pro"
api_backend = "chat_completions"
base_url = "https://api.deepseek.com"
reasoning_effort = "max"
reasoning_efforts = ["low"]

[ui]
theme = "deepseeknight"
yolo = false
"#;
    let fixed = repair_product_agent_config(raw);
    let flash_idx = fixed.find("[model.deepseek-v4-flash]").unwrap();
    let pro_idx = fixed.find("[model.deepseek-v4-pro]").unwrap();
    let flash_sec = &fixed[flash_idx..pro_idx];
    let pro_sec = &fixed[pro_idx..];
    assert!(
        flash_sec.contains("supports_reasoning_effort = true"),
        "flash missing supports: {flash_sec}"
    );
    assert!(
        flash_sec.contains("reasoning_effort = \"high\""),
        "flash missing default high: {flash_sec}"
    );
    assert!(
        flash_sec.contains("reasoning_efforts = [\"max\", \"high\", \"low\"]"),
        "flash missing effort menu: {flash_sec}"
    );
    assert!(
        pro_sec.contains("reasoning_effort = \"max\""),
        "pro explicit max must be preserved: {pro_sec}"
    );
    // A user-set menu survives; the default one is not injected over it.
    assert!(
        pro_sec.contains("reasoning_efforts = [\"low\"]"),
        "pro explicit menu must be preserved: {pro_sec}"
    );
    assert!(
        !pro_sec.contains("reasoning_efforts = [\"max\", \"high\", \"low\"]"),
        "pro must not get the default menu when one is set: {pro_sec}"
    );
    assert!(
        !pro_sec.contains("reasoning_effort = \"high\""),
        "pro must not get high when max already set: {pro_sec}"
    );
    // supports still injected when missing on pro
    assert!(
        pro_sec.contains("supports_reasoning_effort = true"),
        "pro missing supports: {pro_sec}"
    );
    assert_eq!(repair_product_agent_config(&fixed), fixed);
}

#[test]
fn repair_keeps_table_form_reasoning_efforts_without_flat_duplicate() {
    let raw = r#"
[models]
default = "deepseek-v4-flash"

[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
api_backend = "chat_completions"
base_url = "https://api.deepseek.com"
supports_reasoning_effort = true

[[model.deepseek-v4-flash.reasoning_efforts]]
value = "max"

[[model.deepseek-v4-flash.reasoning_efforts]]
value = "high"

[[model.deepseek-v4-flash.reasoning_efforts]]
value = "low"
default = true

[ui]
theme = "deepseeknight"
"#;
    let fixed = repair_product_agent_config(raw);
    let flash_idx = fixed.find("[model.deepseek-v4-flash]").unwrap();
    let pro_idx = fixed.find("[model.deepseek-v4-pro]").unwrap();
    let flash_sec = &fixed[flash_idx..pro_idx];
    assert!(
        !flash_sec
            .lines()
            .any(|l| l.trim().starts_with("reasoning_efforts =")),
        "flat menu must not be injected next to the table form: {flash_sec}"
    );
    assert!(
        !flash_sec.contains("reasoning_effort = \"high\""),
        "an explicit menu suppresses the default effort: {flash_sec}"
    );
    assert_eq!(
        flash_sec
            .matches("[[model.deepseek-v4-flash.reasoning_efforts]]")
            .count(),
        3,
        "the user's table entries must stay: {flash_sec}"
    );
    toml::from_str::<toml::Value>(&fixed).expect("repaired config must parse");
    assert_eq!(repair_product_agent_config(&fixed), fixed);
}

#[test]
fn repair_injects_flat_effort_menu_when_only_unrelated_child_tables_exist() {
    let raw = r#"
[models]
default = "deepseek-v4-flash"

[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
api_backend = "chat_completions"
base_url = "https://api.deepseek.com"

[model.deepseek-v4-flash.notice]
severity = "warning"
text = "Use /pro for harder tasks."

[model.deepseek-v4-pro]
model = "deepseek-v4-pro"
api_backend = "chat_completions"
base_url = "https://api.deepseek.com"

[ui]
theme = "deepseeknight"
"#;
    let fixed = repair_product_agent_config(raw);
    let flash_idx = fixed.find("[model.deepseek-v4-flash]").unwrap();
    let pro_idx = fixed.find("[model.deepseek-v4-pro]").unwrap();
    let flash_sec = &fixed[flash_idx..pro_idx];
    let pro_sec = &fixed[pro_idx..];
    assert!(
        flash_sec.contains("reasoning_efforts = [\"max\", \"high\", \"low\"]"),
        "menu must still be injected when no form declares it: {flash_sec}"
    );
    assert!(
        pro_sec.contains("reasoning_efforts = [\"max\", \"high\", \"low\"]"),
        "pro gets the menu when nothing declares it: {pro_sec}"
    );
    toml::from_str::<toml::Value>(&fixed).expect("repaired config must parse");
    assert_eq!(repair_product_agent_config(&fixed), fixed);
}

#[test]
fn repair_heals_flat_duplicate_beside_table_form_reasoning_efforts() {
    // The incident shape (2026-10-15): the flat menu was injected into a
    // stanza whose only menu is the documented array of tables, and the
    // duplicate key made the whole file unparseable for the vendored loader.
    let broken = r#"
[models]
default = "deepseek-v4-flash"

[model.deepseek-v4-flash]
reasoning_efforts = ["max", "high", "low"]
supports_reasoning_effort = true
reasoning_effort = "max"
model = "deepseek-v4-flash"
api_backend = "chat_completions"
base_url = "https://api.deepseek.com"

[[model.deepseek-v4-flash.reasoning_efforts]]
value = "max"

[[model.deepseek-v4-flash.reasoning_efforts]]
value = "high"

[[model.deepseek-v4-flash.reasoning_efforts]]
value = "low"
default = true
"#;
    assert!(
        toml::from_str::<toml::Value>(broken).is_err(),
        "precondition: the duplicate key is broken TOML"
    );
    let fixed = repair_product_agent_config(broken);
    toml::from_str::<toml::Value>(&fixed).expect("healed config must parse");
    let flash_idx = fixed.find("[model.deepseek-v4-flash]").unwrap();
    let pro_idx = fixed.find("[model.deepseek-v4-pro]").unwrap();
    let flash_sec = &fixed[flash_idx..pro_idx];
    assert!(
        !flash_sec
            .lines()
            .any(|l| l.trim().starts_with("reasoning_efforts =")),
        "the injected flat menu is dropped: {flash_sec}"
    );
    assert_eq!(
        flash_sec
            .matches("[[model.deepseek-v4-flash.reasoning_efforts]]")
            .count(),
        3,
        "the user's table entries survive: {flash_sec}"
    );
    assert_eq!(repair_product_agent_config(&fixed), fixed);
}

#[test]
fn repair_migrates_legacy_context_window_in_both_product_sections() {
    let raw = r#"
[models]
default = "deepseek-v4-flash"

[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
api_backend = "chat_completions"
base_url = "https://api.deepseek.com"
env_key = "DEEPSEEK_API_KEY"
context_window = 128000

[model.deepseek-v4-pro]
model = "deepseek-v4-pro"
api_backend = "chat_completions"
base_url = "https://api.deepseek.com"
context_window = 128000

[ui]
theme = "deepseeknight"
yolo = false
"#;
    let fixed = repair_product_agent_config(raw);
    let flash_idx = fixed.find("[model.deepseek-v4-flash]").unwrap();
    let pro_idx = fixed.find("[model.deepseek-v4-pro]").unwrap();
    let flash_sec = &fixed[flash_idx..pro_idx];
    let pro_sec = &fixed[pro_idx..];
    assert!(
        flash_sec.contains(&format!("context_window = {PRODUCT_CONTEXT_WINDOW}")),
        "flash must migrate off the legacy 128000: {flash_sec}"
    );
    assert!(
        !flash_sec.contains("128000"),
        "legacy 128000 must be gone from flash: {flash_sec}"
    );
    assert!(
        pro_sec.contains(&format!("context_window = {PRODUCT_CONTEXT_WINDOW}")),
        "pro must migrate off the legacy 128000: {pro_sec}"
    );
    assert!(
        !pro_sec.contains("128000"),
        "legacy 128000 must be gone from pro: {pro_sec}"
    );
    // Idempotent
    assert_eq!(repair_product_agent_config(&fixed), fixed);
}

#[test]
fn repair_injects_context_window_when_missing() {
    let raw = r#"
[models]
default = "deepseek-v4-flash"

[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
api_backend = "chat_completions"
base_url = "https://api.deepseek.com"
env_key = "DEEPSEEK_API_KEY"

[model.deepseek-v4-pro]
model = "deepseek-v4-pro"
api_backend = "chat_completions"
base_url = "https://api.deepseek.com"

[ui]
theme = "deepseeknight"
yolo = false
"#;
    let fixed = repair_product_agent_config(raw);
    let flash_idx = fixed.find("[model.deepseek-v4-flash]").unwrap();
    let pro_idx = fixed.find("[model.deepseek-v4-pro]").unwrap();
    let flash_sec = &fixed[flash_idx..pro_idx];
    let pro_sec = &fixed[pro_idx..];
    assert!(
        flash_sec.contains(&format!("context_window = {PRODUCT_CONTEXT_WINDOW}")),
        "flash must get the window injected: {flash_sec}"
    );
    assert_eq!(
        flash_sec.matches("context_window").count(),
        1,
        "flash must get exactly one injected key: {flash_sec}"
    );
    assert!(
        pro_sec.contains(&format!("context_window = {PRODUCT_CONTEXT_WINDOW}")),
        "pro must get the window injected: {pro_sec}"
    );
    assert_eq!(
        pro_sec.matches("context_window").count(),
        1,
        "pro must get exactly one injected key: {pro_sec}"
    );
    // Idempotent
    assert_eq!(repair_product_agent_config(&fixed), fixed);
}

#[test]
fn repair_preserves_custom_context_window_and_other_model_sections() {
    let raw = r#"
[models]
default = "deepseek-v4-flash"

[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
api_backend = "chat_completions"
base_url = "https://api.deepseek.com"
env_key = "DEEPSEEK_API_KEY"
context_window = 128000

[model.deepseek-v4-pro]
model = "deepseek-v4-pro"
api_backend = "chat_completions"
base_url = "https://api.deepseek.com"
context_window = 256000

[model.local-llm]
model = "local-llm"
base_url = "https://localhost:8080/v1"
context_window = 128000

[ui]
theme = "deepseeknight"
yolo = false
"#;
    let fixed = repair_product_agent_config(raw);
    let flash_idx = fixed.find("[model.deepseek-v4-flash]").unwrap();
    let pro_idx = fixed.find("[model.deepseek-v4-pro]").unwrap();
    let other_idx = fixed.find("[model.local-llm]").unwrap();
    let flash_sec = &fixed[flash_idx..pro_idx];
    let pro_sec = &fixed[pro_idx..other_idx];
    let other_sec = &fixed[other_idx..];
    assert!(
        flash_sec.contains(&format!("context_window = {PRODUCT_CONTEXT_WINDOW}")),
        "flash must migrate while pro stays custom: {flash_sec}"
    );
    assert!(
        !flash_sec.contains("128000"),
        "legacy 128000 must be gone from flash: {flash_sec}"
    );
    assert!(
        pro_sec.contains("context_window = 256000"),
        "explicit user value must be preserved: {pro_sec}"
    );
    assert!(
        other_sec.contains("context_window = 128000"),
        "non-product model sections must be untouched: {other_sec}"
    );
    // Idempotent
    assert_eq!(repair_product_agent_config(&fixed), fixed);
}

#[test]
fn picker_maps_numbers_names_and_defaults() {
    assert_eq!(picker_answer_to_theme("1"), Some(PRODUCT_THEME));
    assert_eq!(picker_answer_to_theme("2"), Some(PRODUCT_THEME_V2));
    assert_eq!(picker_answer_to_theme(""), Some(PRODUCT_THEME));
    assert_eq!(picker_answer_to_theme("  "), Some(PRODUCT_THEME));
    assert_eq!(picker_answer_to_theme(PRODUCT_THEME), Some(PRODUCT_THEME));
    assert_eq!(
        picker_answer_to_theme(PRODUCT_THEME_V2),
        Some(PRODUCT_THEME_V2)
    );
    assert_eq!(
        picker_answer_to_theme("deepseek-night"),
        Some(PRODUCT_THEME)
    );
    assert_eq!(
        picker_answer_to_theme("deepseeknight-v2"),
        Some(PRODUCT_THEME_V2)
    );
    assert_eq!(picker_answer_to_theme("3"), Some(PRODUCT_THEME_NEUTRAL));
    assert_eq!(
        picker_answer_to_theme(PRODUCT_THEME_NEUTRAL),
        Some(PRODUCT_THEME_NEUTRAL)
    );
    assert_eq!(
        picker_answer_to_theme("dsb-neutral"),
        Some(PRODUCT_THEME_NEUTRAL)
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
    // Fresh seed with the v2 palette (distinct from the classic default).
    ensure_product_agent_config_with_theme(&home, PRODUCT_THEME_V2).unwrap();
    let body = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(
        body.contains(&format!("theme = \"{PRODUCT_THEME_V2}\"")),
        "seed should use the requested v2 theme: {body}"
    );
    // Repair with a different theme does not clobber an existing choice.
    ensure_product_agent_config_with_theme(&home, PRODUCT_THEME).unwrap();
    let body2 = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert_eq!(body, body2, "repair must not overwrite an explicit theme");
    // Repair on a theme-less file injects the requested theme.
    let fixed = repair_product_agent_config_with_theme("keep=1\n", PRODUCT_THEME_V2);
    assert!(fixed.contains(&format!("theme = \"{PRODUCT_THEME_V2}\"")));
    assert!(fixed.contains("keep=1"));
}

#[test]
fn seed_injects_deepseek_system_prompt_label_and_no_grok_branding() {
    let dir = tempfile::tempdir().unwrap();
    let home = dsb_config::BuildHome::from_path(dir.path());
    ensure_product_agent_config(&home).unwrap();
    let body = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    // Both DeepSeek model stanzas carry the product identity label.
    assert_eq!(
        body.matches(&format!(
            "system_prompt_label = \"{PRODUCT_SYSTEM_PROMPT_LABEL}\""
        ))
        .count(),
        2,
        "seed must label both models: {body}"
    );
    // The product seed must never render Grok/xAI identity text. (The
    // `xai_api_base_url` key itself is a vendored schema name — the ADR 0008
    // config-home bridge — not branding, and the `disabled_models` picker
    // selector is the one allowed `grok` occurrence.)
    assert_no_grok_branding(&body);
}

#[test]
fn repair_injects_system_prompt_label_into_existing_models() {
    let raw = r#"
[models]
default = "deepseek-v4-flash"

[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
api_backend = "chat_completions"
base_url = "https://api.deepseek.com"
env_key = "DEEPSEEK_API_KEY"

[model.deepseek-v4-pro]
model = "deepseek-v4-pro"
api_backend = "chat_completions"
base_url = "https://api.deepseek.com"
env_key = "DEEPSEEK_API_KEY"
"#;
    let fixed = repair_product_agent_config(raw);
    assert_eq!(
        fixed
            .matches(&format!(
                "system_prompt_label = \"{PRODUCT_SYSTEM_PROMPT_LABEL}\""
            ))
            .count(),
        2,
        "repair must label both models: {fixed}"
    );
    // Idempotent.
    assert_eq!(repair_product_agent_config(&fixed), fixed);
}

#[test]
fn repair_replaces_grok_valued_system_prompt_label() {
    let raw = r#"
[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
system_prompt_label = "Grok"
"#;
    let fixed = repair_product_agent_config(raw);
    assert!(
        fixed.contains(&format!(
            "system_prompt_label = \"{PRODUCT_SYSTEM_PROMPT_LABEL}\""
        )),
        "stale Grok label must be replaced: {fixed}"
    );
    assert!(!fixed.contains("system_prompt_label = \"Grok\""));
}

#[test]
fn repair_ignores_grok_valued_system_prompt_label_backup() {
    let raw = r#"
[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
system_prompt_label_backup = "Grok"

[other]
keep = true
"#;
    let fixed = ensure_deepseek_model_system_prompt_label(raw.to_string(), "deepseek-v4-flash");
    assert!(
        fixed.contains("system_prompt_label_backup = \"Grok\""),
        "backup label key must be preserved: {fixed}"
    );
    assert!(
        fixed.contains(&format!(
            "system_prompt_label = \"{PRODUCT_SYSTEM_PROMPT_LABEL}\"\n[other]"
        )),
        "missing true label must be injected before the next section: {fixed}"
    );
}

#[test]
fn repair_keeps_custom_non_grok_system_prompt_label() {
    let raw = r#"
[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
system_prompt_label = "My Custom Agent"
"#;
    let fixed = repair_product_agent_config(raw);
    assert!(
        fixed.contains("system_prompt_label = \"My Custom Agent\""),
        "custom label must be preserved: {fixed}"
    );
}

#[test]
fn repair_keeps_custom_system_prompt_label_when_comment_mentions_grok() {
    let raw = r#"
[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
system_prompt_label = "My Custom Agent" # not Grok
"#;
    let fixed = repair_product_agent_config(raw);
    assert!(
        fixed.contains("system_prompt_label = \"My Custom Agent\" # not Grok"),
        "comment text must not make a custom label stale: {fixed}"
    );
}

#[test]
fn repair_replaces_stale_flash_display_name_and_keeps_pro() {
    let raw = r#"
[models]
default = "deepseek-v4-flash"

[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
name = "DeepSeek V4 Flash"

[model.deepseek-v4-pro]
model = "deepseek-v4-pro"
name = "DeepSeek V4 Pro"
"#;
    let fixed = repair_product_agent_config(raw);
    assert!(
        fixed.contains("name = \"DeepSeek V4.1 Flash\""),
        "stale Flash display name must be replaced: {fixed}"
    );
    assert!(
        !fixed.contains("name = \"DeepSeek V4 Flash\""),
        "stale Flash display name must not survive: {fixed}"
    );
    assert!(
        fixed.contains("name = \"DeepSeek V4 Pro\""),
        "Pro display name must be preserved: {fixed}"
    );
    // Idempotent.
    assert_eq!(repair_product_agent_config(&fixed), fixed);
}

#[test]
fn repair_keeps_custom_display_name() {
    let raw = r#"
[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
name = "My Flash"
"#;
    let fixed = repair_product_agent_config(raw);
    assert!(
        fixed.contains("name = \"My Flash\""),
        "custom display name must be preserved: {fixed}"
    );
}

#[test]
fn repair_injects_missing_display_name_before_next_section() {
    let raw = r#"
[model.deepseek-v4-flash]
model = "deepseek-v4-flash"

[other]
keep = true
"#;
    let fixed = ensure_deepseek_model_display_name(raw.to_string(), "deepseek-v4-flash");
    assert!(
        fixed.contains("name = \"DeepSeek V4.1 Flash\"\n[other]"),
        "missing name must be injected before the next section: {fixed}"
    );
}

#[test]
fn repair_scrubs_grok_fork_secondary_model() {
    let raw = r#"
[models]
default = "deepseek-v4-flash"

[ui]
theme = "deepseeknight"
fork_secondary_model = "grok-4.5"
yolo = false
"#;
    let fixed = repair_product_agent_config(raw);
    assert!(
        !fixed.contains("grok-4.5"),
        "grok fork leftover must be scrubbed: {fixed}"
    );
    assert!(fixed.contains("theme = \"deepseeknight\""), "{fixed}");
    assert!(fixed.contains("yolo = false"), "{fixed}");
    // Idempotent: an already-scrubbed config is untouched.
    assert_eq!(repair_product_agent_config(&fixed), fixed);
}

#[test]
fn scrub_keeps_non_grok_fork_target() {
    let raw = "[ui]\nfork_secondary_model = \"deepseek-v4-pro\"\n";
    let fixed = repair_product_agent_config(raw);
    assert!(
        fixed.contains("fork_secondary_model = \"deepseek-v4-pro\""),
        "non-grok fork target must be preserved: {fixed}"
    );
}

#[test]
fn scrub_keeps_non_grok_fork_target_when_comment_mentions_grok() {
    let raw = "[ui]\nfork_secondary_model = \"deepseek-v4-pro\" # replaces Grok\n";
    let fixed = repair_product_agent_config(raw);
    assert!(
        fixed.contains("fork_secondary_model = \"deepseek-v4-pro\" # replaces Grok"),
        "comment text must not make a non-grok fork target stale: {fixed}"
    );
}

#[test]
fn scrub_keeps_grok_fork_secondary_model_outside_ui() {
    let raw = r#"
[custom]
fork_secondary_model = "grok-4.5"
"#;
    let fixed = repair_product_agent_config(raw);
    assert!(
        fixed.contains("[custom]\nfork_secondary_model = \"grok-4.5\""),
        "custom section fork target must be preserved: {fixed}"
    );
}

#[test]
fn scrub_only_removes_grok_fork_secondary_model_from_ui() {
    let raw = r#"
[ui]
fork_secondary_model = "grok-4.5"
fork_secondary_model_backup = "grok-4.5"

[custom]
fork_secondary_model = "grok-4.5"
"#;
    let fixed = repair_product_agent_config(raw);
    assert!(
        !fixed.contains("[ui]\nfork_secondary_model = \"grok-4.5\""),
        "ui grok fork target must be scrubbed: {fixed}"
    );
    assert!(
        fixed.contains("fork_secondary_model_backup = \"grok-4.5\""),
        "ui backup key must be preserved: {fixed}"
    );
    assert!(
        fixed.contains("[custom]\nfork_secondary_model = \"grok-4.5\""),
        "custom grok fork target must be preserved: {fixed}"
    );
}

#[cfg(test)]
fn section_of<'a>(body: &'a str, header: &str) -> &'a str {
    let start = body
        .find(header)
        .unwrap_or_else(|| panic!("{header} missing: {body}"));
    let rest = &body[start + header.len()..];
    let end = rest.find("\n[").map_or(rest.len(), |i| i + 1);
    &rest[..end]
}

/// Assert the product config carries no Grok/xAI identity text.
///
/// The `disabled_models = ["grok-*"]` picker selector is the one allowed
/// non-comment `grok` occurrence; comments are skipped.
#[cfg(test)]
fn assert_no_grok_branding(body: &str) {
    assert!(
        !body.to_ascii_lowercase().contains("released by xai"),
        "released-by-xAI branding leaked: {body}"
    );
    assert!(
        !body.contains("system_prompt_label = \"Grok"),
        "grok-valued label leaked: {body}"
    );
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || !trimmed.to_ascii_lowercase().contains("grok") {
            continue;
        }
        assert!(
            trimmed.starts_with("disabled_models"),
            "grok branding leaked: {line}"
        );
    }
}

#[test]
fn openrouter_seed_routes_both_stanzas_to_openrouter() {
    let dir = tempfile::tempdir().unwrap();
    let home = dsb_config::BuildHome::from_path(dir.path());
    dsb_config::Credentials::save(&home, Provider::OpenRouter, "sk-or-v1-seedkey").unwrap();
    ensure_product_agent_config(&home).unwrap();
    let body = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    for slot in PRODUCT_MODEL_SLOTS {
        let sec = section_of(&body, &format!("[model.{slot}]"));
        assert!(
            sec.contains(&format!("model = \"deepseek/{slot}\"")),
            "{sec}"
        );
        assert!(
            sec.contains(&format!("base_url = \"{OPENROUTER_API_BASE_URL}\"")),
            "{sec}"
        );
        assert!(sec.contains("env_key = \"OPENROUTER_API_KEY\""), "{sec}");
        assert!(sec.contains("api_key = \"sk-or-v1-seedkey\""), "{sec}");
        assert!(sec.contains("reasoning_effort = \"high\""), "{sec}");
    }
    assert!(body.contains(&format!("xai_api_base_url = \"{OPENROUTER_API_BASE_URL}\"")));
    assert!(!body.contains(DEEPSEEK_API_BASE_URL), "{body}");
    assert!(!body.contains("DEEPSEEK_API_KEY"), "{body}");
    assert_no_grok_branding(&body);
    // Launch-time repair must not drag an OpenRouter seed back to DeepSeek.
    ensure_product_agent_config(&home).unwrap();
    let again = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert_eq!(body, again);
}

#[test]
fn seed_points_side_requests_at_flash_but_not_prompt_suggestion() {
    let dir = tempfile::tempdir().unwrap();
    let home = dsb_config::BuildHome::from_path(dir.path());
    ensure_product_agent_config(&home).unwrap();
    let body = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    let models = section_of(&body, "[models]");
    for key in PRODUCT_AUX_MODEL_KEYS {
        assert!(
            models.contains(&format!("{key} = \"deepseek-v4-flash\"")),
            "{models}"
        );
    }
    // Unset keeps the vendored per-turn suggestion call skipped (no new cost).
    assert!(!body.contains("prompt_suggestion"), "{body}");
}

#[test]
fn repair_injects_aux_models_without_clobber() {
    let raw = "[models]\ndefault = \"deepseek-v4-flash\"\nsession_summary = \"deepseek-v4-pro\"\n";
    let fixed = repair_product_agent_config(raw);
    let models = section_of(&fixed, "[models]");
    assert!(
        models.contains("session_summary = \"deepseek-v4-pro\""),
        "{models}"
    );
    assert!(
        !models.contains("session_summary = \"deepseek-v4-flash\""),
        "{models}"
    );
    assert!(
        models.contains("web_search = \"deepseek-v4-flash\""),
        "{models}"
    );
    assert!(
        models.contains("image_description = \"deepseek-v4-flash\""),
        "{models}"
    );
    assert_eq!(fixed, repair_product_agent_config(&fixed));
    // No [models] table: aux keys are not invented outside one.
    let bare = repair_product_agent_config("keep=1\n");
    assert!(!bare.contains("session_summary"), "{bare}");
}

#[test]
fn retarget_moves_pristine_seed_to_openrouter_and_back() {
    let dir = tempfile::tempdir().unwrap();
    let home = dsb_config::BuildHome::from_path(dir.path());
    dsb_config::Credentials::save(&home, Provider::DeepSeek, "sk-deepseek-old").unwrap();
    ensure_product_agent_config(&home).unwrap();
    let deepseek_seed = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();

    let (or_body, kept) =
        retarget_product_config(&deepseek_seed, Provider::OpenRouter, "sk-or-v1-new");
    assert!(kept.is_empty());
    for slot in PRODUCT_MODEL_SLOTS {
        let sec = section_of(&or_body, &format!("[model.{slot}]"));
        assert!(
            sec.contains(&format!("model = \"deepseek/{slot}\"")),
            "{sec}"
        );
        assert!(
            sec.contains(&format!("base_url = \"{OPENROUTER_API_BASE_URL}\"")),
            "{sec}"
        );
        assert!(sec.contains("env_key = \"OPENROUTER_API_KEY\""), "{sec}");
        assert_eq!(sec.matches("api_key = ").count(), 1, "{sec}");
        assert!(sec.contains("api_key = \"sk-or-v1-new\""), "{sec}");
    }
    assert!(
        !or_body.contains("sk-deepseek-old"),
        "stale key survived: {or_body}"
    );
    assert!(!or_body.contains(DEEPSEEK_API_BASE_URL), "{or_body}");
    // Idempotent, and launch-time repair keeps it on OpenRouter.
    assert_eq!(
        retarget_product_config(&or_body, Provider::OpenRouter, "sk-or-v1-new").0,
        or_body
    );
    assert!(!repair_product_agent_config(&or_body).contains(DEEPSEEK_API_BASE_URL));

    let (back, kept) = retarget_product_config(&or_body, Provider::DeepSeek, "sk-deepseek-new");
    assert!(kept.is_empty());
    for slot in PRODUCT_MODEL_SLOTS {
        let sec = section_of(&back, &format!("[model.{slot}]"));
        assert!(sec.contains(&format!("model = \"{slot}\"")), "{sec}");
        assert!(
            sec.contains(&format!("base_url = \"{DEEPSEEK_API_BASE_URL}\"")),
            "{sec}"
        );
        assert!(sec.contains("env_key = \"DEEPSEEK_API_KEY\""), "{sec}");
        assert!(sec.contains("api_key = \"sk-deepseek-new\""), "{sec}");
    }
    assert!(back.contains(&format!("xai_api_base_url = \"{DEEPSEEK_API_BASE_URL}\"")));
    assert!(!back.contains("openrouter"), "{back}");
}

#[test]
fn retarget_keeps_hand_edited_stanzas() {
    // Shape of a hand-tuned OpenRouter home: custom model id + own env var.
    let raw = r#"[models]
default = "deepseek-v4-flash"

[model.deepseek-v4-flash]
model = "deepseek/deepseek-v4.1-flash"
base_url = "https://openrouter.ai/api/v1"
env_key = "OPENROUTER_DEEPSEEK_BUILD_KEY"
reasoning_effort = "max"

[[model.deepseek-v4-flash.reasoning_efforts]]
id = "max"

[model.deepseek-v4-pro]
model = "deepseek/deepseek-v4.1-flash"
base_url = "https://openrouter.ai/api/v1"
env_key = "OPENROUTER_DEEPSEEK_BUILD_KEY"

[endpoints]
xai_api_base_url = "https://openrouter.ai/api/v1"
"#;
    // Choosing OpenRouter again: already there, no inline key → untouched.
    let (same, kept) = retarget_product_config(raw, Provider::OpenRouter, "sk-or-v1-x");
    assert_eq!(same, raw);
    assert!(kept.is_empty());
    // Choosing DeepSeek: never half-rewrite a custom model onto api.deepseek.com.
    let (same, kept) = retarget_product_config(raw, Provider::DeepSeek, "sk-ds-x");
    assert_eq!(same, raw);
    assert_eq!(kept, vec!["deepseek-v4-flash", "deepseek-v4-pro"]);
}

#[test]
fn retarget_replaces_stale_inline_key_on_same_provider() {
    let raw = r#"[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
base_url = "https://api.deepseek.com"
api_key = "sk-old"
env_key = "DEEPSEEK_API_KEY"

[model.deepseek-v4-pro]
model = "deepseek-v4-pro"
env_key = "DEEPSEEK_API_KEY"
"#;
    let (fixed, kept) = retarget_product_config(raw, Provider::DeepSeek, "sk-new");
    assert!(kept.is_empty());
    let flash = section_of(&fixed, "[model.deepseek-v4-flash]");
    assert!(
        flash.contains("api_key = \"sk-new\"") && !fixed.contains("sk-old"),
        "{fixed}"
    );
    // A stanza that carried no inline key keeps authenticating via env_key.
    let pro = section_of(&fixed, "[model.deepseek-v4-pro]");
    assert!(!pro.contains("api_key"), "{pro}");
}

#[test]
fn configured_provider_reads_the_product_stanzas() {
    let openrouter = format!(
        "[model.deepseek-v4-flash]\nbase_url = \"{OPENROUTER_API_BASE_URL}\"\n\n\
         [model.deepseek-v4-pro]\nbase_url = \"{OPENROUTER_API_BASE_URL}\"\n"
    );
    assert_eq!(provider_of_config(&openrouter), Some(Provider::OpenRouter));
    let deepseek = format!("[model.deepseek-v4-pro]\nbase_url = \"{DEEPSEEK_API_BASE_URL}\"\n");
    assert_eq!(provider_of_config(&deepseek), Some(Provider::DeepSeek));
    // `auth logout` leaves the stanzas behind: the choice is still readable.
    let dir = tempfile::tempdir().unwrap();
    let home = dsb_config::BuildHome::from_path(dir.path());
    dsb_config::Credentials::save(&home, Provider::OpenRouter, "sk-or-v1-k").unwrap();
    ensure_product_agent_config(&home).unwrap();
    dsb_config::Credentials::clear_file(&home).unwrap();
    assert_eq!(configured_provider(&home), Some(Provider::OpenRouter));
}

#[test]
fn configured_provider_refuses_to_guess() {
    // No product stanza at all.
    assert_eq!(provider_of_config("[models]\ndefault = \"x\"\n"), None);
    assert_eq!(provider_of_config(""), None);
    // Stanzas disagree (half-switched file).
    let mixed = format!(
        "[model.deepseek-v4-flash]\nbase_url = \"{OPENROUTER_API_BASE_URL}\"\n\n\
         [model.deepseek-v4-pro]\nbase_url = \"{DEEPSEEK_API_BASE_URL}\"\n"
    );
    assert_eq!(provider_of_config(&mixed), None);
    // Somebody else's endpoint is neither provider.
    assert_eq!(
        provider_of_config("[model.deepseek-v4-flash]\nbase_url = \"https://example.test/v1\"\n"),
        None
    );
    // Not a one-line string: hand-written, not ours to judge.
    assert_eq!(
        provider_of_config("[model.deepseek-v4-flash]\nbase_url = [\"a\", \"b\"]\n"),
        None
    );
    // A stanza without base_url does not veto the other one.
    let partial = format!(
        "[model.deepseek-v4-flash]\nbase_url = \"{OPENROUTER_API_BASE_URL}\"\n\n\
         [model.deepseek-v4-pro]\nmodel = \"deepseek/deepseek-v4-pro\"\n"
    );
    assert_eq!(provider_of_config(&partial), Some(Provider::OpenRouter));
}

#[test]
fn retarget_treats_missing_base_url_as_deepseek_and_appends_missing_stanza() {
    let raw = "[model.deepseek-v4-flash]\nmodel = \"deepseek-v4-flash\"\n";
    let (fixed, kept) = retarget_product_config(raw, Provider::OpenRouter, "sk-or-v1-k");
    assert!(kept.is_empty());
    let flash = section_of(&fixed, "[model.deepseek-v4-flash]");
    assert!(
        flash.contains(&format!("base_url = \"{OPENROUTER_API_BASE_URL}\"")),
        "{flash}"
    );
    assert!(
        flash.contains("model = \"deepseek/deepseek-v4-flash\""),
        "{flash}"
    );
    assert!(flash.contains("api_key = \"sk-or-v1-k\""), "{flash}");
    let pro = section_of(&fixed, "[model.deepseek-v4-pro]");
    assert!(
        pro.contains("model = \"deepseek/deepseek-v4-pro\""),
        "{pro}"
    );
    assert!(pro.contains("api_key = \"sk-or-v1-k\""), "{pro}");
    // Repair must not add a DeepSeek base_url next to the OpenRouter one.
    let repaired = repair_product_agent_config(&fixed);
    for slot in PRODUCT_MODEL_SLOTS {
        let sec = section_of(&repaired, &format!("[model.{slot}]"));
        assert!(!sec.contains(DEEPSEEK_API_BASE_URL), "{sec}");
    }
}

#[test]
fn configured_env_key_follows_default_stanza() {
    let env = |name: &str| (name == "MY_OR_KEY").then(|| "set".to_string());
    let custom = r#"[models]
default = "deepseek-v4-flash"

[model.deepseek-v4-flash]
env_key = "MY_OR_KEY" # own variable
"#;
    assert!(configured_model_env_key_available_with(custom, env));
    // Array form (vendored schema accepts string or array).
    let array = "[model.deepseek-v4-flash]\nenv_key = [\"UNSET_A\", \"MY_OR_KEY\"]\n";
    assert!(configured_model_env_key_available_with(array, env));
    // Variable unset → setup still needed.
    let unset = "[model.deepseek-v4-flash]\nenv_key = \"UNSET_A\"\n";
    assert!(!configured_model_env_key_available_with(unset, env));
    // Only the default stanza counts, and an inline api_key alone does not.
    let pro_default = r#"[models]
default = "deepseek-v4-pro"

[model.deepseek-v4-flash]
env_key = "MY_OR_KEY"

[model.deepseek-v4-pro]
api_key = "sk-inline"
env_key = "UNSET_A"
"#;
    assert!(!configured_model_env_key_available_with(pro_default, env));
    assert!(!configured_model_env_key_available_with("", env));
}

#[test]
fn agent_key_env_uses_the_providers_own_variable() {
    let dir = tempfile::tempdir().unwrap();
    let home = dsb_config::BuildHome::from_path(dir.path());
    let unset = |_: &str| false;
    dsb_config::Credentials::save(&home, Provider::OpenRouter, "sk-or-v1-k").unwrap();
    let or = dsb_config::Credentials::load_with(&home, None).unwrap();
    assert_eq!(
        agent_key_env(&or, unset),
        Some(("OPENROUTER_API_KEY", "sk-or-v1-k"))
    );
    // A shell value wins; the saved key is not forced over it.
    assert_eq!(agent_key_env(&or, |n| n == "OPENROUTER_API_KEY"), None);
    // DEEPSEEK_API_KEY being set does not stop the OpenRouter hand-off.
    assert!(agent_key_env(&or, |n| n == "DEEPSEEK_API_KEY").is_some());
    dsb_config::Credentials::save(&home, Provider::DeepSeek, "sk-ds-k").unwrap();
    let ds = dsb_config::Credentials::load_with(&home, None).unwrap();
    assert_eq!(
        agent_key_env(&ds, unset),
        Some(("DEEPSEEK_API_KEY", "sk-ds-k"))
    );
}

#[test]
fn retarget_keeps_product_stanza_with_its_own_env_key() {
    // Product URL + model, but the user's own variable: not a product default.
    let raw = r#"[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
base_url = "https://api.deepseek.com"
env_key = "MY_DS_KEY"
"#;
    let (same, kept) = retarget_product_config(raw, Provider::OpenRouter, "sk-or-v1-k");
    assert!(same.starts_with(raw), "{same}");
    assert!(kept.contains(&"deepseek-v4-flash"));
}

#[test]
fn retarget_never_declares_a_product_table_twice() {
    // Trailing comment / inner spaces: still found, rewritten in place.
    let commented = r#"[ model.deepseek-v4-flash ]  # main
model = "deepseek-v4-flash"
base_url = "https://api.deepseek.com"
env_key = "DEEPSEEK_API_KEY"
"#;
    let (fixed, _) = retarget_product_config(commented, Provider::OpenRouter, "sk-or-v1-k");
    assert_eq!(fixed.matches("deepseek-v4-flash ]").count(), 1, "{fixed}");
    assert!(!fixed.contains("\n[model.deepseek-v4-flash]\n"), "{fixed}");
    assert!(fixed.contains(&format!("base_url = \"{OPENROUTER_API_BASE_URL}\"")));
    // Quoted key spelling: not matched, so never appended again either.
    let quoted = "[model.\"deepseek-v4-flash\"]\nmodel = \"deepseek-v4-flash\"\n";
    let (fixed, kept) = retarget_product_config(quoted, Provider::OpenRouter, "sk-or-v1-k");
    assert!(!fixed.contains("[model.deepseek-v4-flash]"), "{fixed}");
    assert!(kept.contains(&"deepseek-v4-flash"));
}

#[test]
fn retarget_keeps_stanzas_with_non_string_values() {
    let raw = r#"[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
base_url = "https://api.deepseek.com"
env_key = [
  "MY_DS_KEY",
]

[model.deepseek-v4-pro]
model = "deepseek-v4-pro"
base_url = "https://api.deepseek.com"
env_key = ["DEEPSEEK_API_KEY"]
"#;
    let (same, kept) = retarget_product_config(raw, Provider::OpenRouter, "sk-or-v1-k");
    assert_eq!(same, raw);
    assert_eq!(kept, vec!["deepseek-v4-flash", "deepseek-v4-pro"]);
}

#[test]
fn seed_pins_picker_model_selectors() {
    let dir = tempfile::tempdir().unwrap();
    let home = dsb_config::BuildHome::from_path(dir.path());
    ensure_product_agent_config(&home).unwrap();
    let body = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    let models = section_of(&body, "[models]");
    for (key, full_line) in PRODUCT_PICKER_MODEL_KEYS {
        assert!(models.contains(full_line), "{key} missing: {models}");
    }
}

#[test]
fn repair_injects_picker_selectors_into_existing_models() {
    let raw = "[models]\ndefault = \"deepseek-v4-flash\"\ndisabled_models = [\"grok-4.6\"]\n";
    let fixed = repair_product_agent_config(raw);
    let models = section_of(&fixed, "[models]");
    assert!(
        models.contains("disabled_models = [\"grok-4.6\"]"),
        "explicit user selector must be preserved: {models}"
    );
    assert!(
        models.contains("hidden_models = [\"deepseek-v4-pro\"]"),
        "{models}"
    );
    assert_eq!(fixed.matches("disabled_models").count(), 1, "{fixed}");
    assert_eq!(repair_product_agent_config(&fixed), fixed);
}

#[test]
fn repair_appends_picker_models_table_when_missing() {
    let fixed = repair_product_agent_config("keep=1\n");
    assert!(fixed.contains("keep=1"), "{fixed}");
    assert_eq!(fixed.matches("[models]").count(), 1, "{fixed}");
    let models = section_of(&fixed, "[models]");
    for (key, full_line) in PRODUCT_PICKER_MODEL_KEYS {
        assert!(models.contains(full_line), "{key} missing: {models}");
    }
}

#[test]
fn repair_picker_repairs_are_idempotent() {
    // Existing `[models]`: one pass is a fixpoint.
    let with_models = "[models]\ndefault = \"deepseek-v4-flash\"\n";
    let once = repair_product_agent_config(with_models);
    assert_eq!(once, repair_product_agent_config(&once));
    // No `[models]` at all: the picker table lands on the first pass and the
    // aux side-request keys follow into it on the second.
    let bare = repair_product_agent_config("keep=1\n");
    assert_eq!(bare.matches("[models]").count(), 1, "{bare}");
    let bare2 = repair_product_agent_config(&bare);
    let bare3 = repair_product_agent_config(&bare2);
    assert_eq!(bare2, bare3);
}

#[test]
fn product_slot_display_names_follow_the_picker_surface() {
    assert_eq!(
        product_slot_display_name("deepseek-v4-flash"),
        "DeepSeek V4.1 Flash"
    );
    assert_eq!(
        product_slot_display_name("deepseek-v4-pro"),
        "DeepSeek V4 Pro"
    );
}
