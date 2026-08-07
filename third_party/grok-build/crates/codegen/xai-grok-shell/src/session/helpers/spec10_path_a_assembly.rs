//! Spec 10 Path A turn assembly (DeepSeek Build VC007).
//!
//! Grok Path A turns historically assembled conversation via chat-state without
//! the product Spec 10 stable-prefix library path. This module **mirrors**
//! `dsb_context::assemble_path_a_context` layout + epoch rules inside the Grok
//! shell (same fusion pattern as Spec 15 `repair_tool_arguments_one_pass`):
//!
//! ```text
//! messages_to_api =
//!   stable_prefix   // system body with ordered Spec 10 sections
//!   + volatile_tail // user / assistant / tool chain
//! ```
//!
//! Stable section order (normative Spec 10 §1.1):
//! 1. System prompt body
//! 2. Tools document (canonical JSON, sorted object keys)
//! 3. Skills index only (name + description; sorted by name)
//! 4. Environment summary (os family + cwd; no wall-clock)
//! 5. Standing project instructions
//!
//! See: `docs/specs/10-cache-contract.md`, VC007 evidence under
//! `docs/product/evidence/VC007_*`.

use sha2::{Digest, Sha256};
use xai_grok_sampling_types::ToolSpec;

/// One skills-index row for Spec 10 stable prefix (index only — no body).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec10SkillIndexEntry {
    pub name: String,
    pub description: String,
}

/// Clock-free environment summary for Spec 10 §1.1 item 4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec10EnvironmentSummary {
    pub os_family: String,
    /// Workspace cwd normalized (prefer project-relative / absolute clean path).
    pub cwd: String,
}

/// Inputs that fully determine the Spec 10 stable prefix for one Path A turn.
#[derive(Debug, Clone)]
pub struct Spec10PathAInputs {
    pub system_prompt: String,
    pub tools: Vec<ToolSpec>,
    pub skills_index: Vec<Spec10SkillIndexEntry>,
    pub environment: Spec10EnvironmentSummary,
    pub project_instructions: String,
    /// Volatile: current user/assistant/tool chain (not in stable prefix / epoch).
    pub volatile_tail_text: Vec<String>,
}

/// Assembled Spec 10 Path A context for one turn.
#[derive(Debug, Clone)]
pub struct Spec10PathAAssembled {
    /// Full system-shaped stable body (ordered Spec 10 sections).
    pub stable_body: String,
    /// UTF-8 bytes of the canonical stable prefix document (epoch input).
    pub stable_prefix_bytes: Vec<u8>,
    /// Full SHA-256 hex of `stable_prefix_bytes`.
    pub epoch_sha256_hex: String,
    /// Number of volatile tail items (does not affect epoch).
    pub volatile_count: usize,
}

impl Spec10PathAAssembled {
    /// First 16 hex chars for logs (`prefix_epoch=`).
    pub fn epoch_short(&self) -> &str {
        let n = self.epoch_sha256_hex.len().min(16);
        &self.epoch_sha256_hex[..n]
    }

    pub fn log_label(&self) -> String {
        format!("prefix_epoch={}", self.epoch_short())
    }
}

/// Assemble Path A Spec 10 stable prefix + epoch for a Grok turn.
///
/// Stable sections are byte-stable for identical inputs; volatile tail does not
/// affect the epoch.
pub fn assemble_spec10_path_a_turn(inputs: &Spec10PathAInputs) -> Spec10PathAAssembled {
    let tools_doc = tools_document(&inputs.tools);
    let skills_doc = skills_document(&inputs.skills_index);
    let env_doc = env_document(&inputs.environment);

    let mut stable_body = inputs.system_prompt.trim_end().to_string();
    stable_body.push_str("\n\n## Tools\n");
    stable_body.push_str(&tools_doc);
    stable_body.push_str("\n\n## Skills index\n");
    stable_body.push_str(&skills_doc);
    stable_body.push_str("\n\n## Environment\n");
    stable_body.push_str(&env_doc);
    if !inputs.project_instructions.trim().is_empty() {
        stable_body.push_str("\n\n## Project instructions\n");
        stable_body.push_str(inputs.project_instructions.trim_end());
        stable_body.push('\n');
    }

    // Epoch is over the stable body alone (Spec 10 §1.3 / §1.5). Volatile
    // strings are tracked for call-site honesty but never hashed.
    let stable_prefix_bytes = stable_body.as_bytes().to_vec();
    let epoch_sha256_hex = sha256_hex(&stable_prefix_bytes);

    Spec10PathAAssembled {
        stable_body,
        stable_prefix_bytes,
        epoch_sha256_hex,
        volatile_count: inputs.volatile_tail_text.len(),
    }
}

/// Canonical tools document: JSON array with sorted object keys (Spec 10 §1.1–1.3).
/// Tool order follows input order (caller freezes order).
pub fn tools_document(tools: &[ToolSpec]) -> String {
    let value = tools_to_value(tools);
    canonicalize_json_string(&value)
}

/// Skills index document: sorted by name; index only.
pub fn skills_document(skills: &[Spec10SkillIndexEntry]) -> String {
    let mut sorted = skills.to_vec();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    let value = serde_json::Value::Array(
        sorted
            .into_iter()
            .map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "description": s.description,
                })
            })
            .collect(),
    );
    canonicalize_json_string(&value)
}

/// Environment document (no wall-clock fields).
pub fn env_document(env: &Spec10EnvironmentSummary) -> String {
    let value = serde_json::json!({
        "cwd": env.cwd,
        "os_family": env.os_family,
    });
    canonicalize_json_string(&value)
}

/// Recursively sort object keys and emit compact JSON.
pub fn canonicalize_json_string(value: &serde_json::Value) -> String {
    let sorted = sort_keys(value.clone());
    serde_json::to_string(&sorted).unwrap_or_else(|_| "{}".to_string())
}

/// Recursively sort object keys lexicographically (Spec 10 §1.3).
pub fn sort_keys(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<_> = map.keys().cloned().collect();
            keys.sort();
            let mut out = serde_json::Map::new();
            for k in keys {
                if let Some(v) = map.get(&k) {
                    out.insert(k, sort_keys(v.clone()));
                }
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(sort_keys).collect())
        }
        other => other,
    }
}

fn tools_to_value(tools: &[ToolSpec]) -> serde_json::Value {
    serde_json::Value::Array(
        tools
            .iter()
            .map(|t| {
                let mut obj = serde_json::Map::new();
                obj.insert("name".into(), serde_json::Value::String(t.name.clone()));
                if let Some(desc) = &t.description {
                    obj.insert(
                        "description".into(),
                        serde_json::Value::String(desc.clone()),
                    );
                }
                obj.insert("parameters".into(), t.parameters.clone());
                serde_json::Value::Object(obj)
            })
            .collect(),
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

/// Best-effort stamp of Spec 10 turn epoch under product home or a provided dir.
///
/// When `DEEPSEEK_BUILD_HOME` is set (Path A product launch), write
/// `path_a_turn_prefix_epoch.txt`. Failures never block the turn.
pub fn stamp_spec10_turn_epoch(assembled: &Spec10PathAAssembled, stamp_dir: Option<&std::path::Path>) {
    let dir = stamp_dir
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("DEEPSEEK_BUILD_HOME").map(std::path::PathBuf::from));
    let Some(dir) = dir else {
        return;
    };
    let path = dir.join("path_a_turn_prefix_epoch.txt");
    let body = format!(
        "path_a_turn_prefix_epoch={}\npath_a_turn_prefix_epoch_short={}\npath_a_turn_volatile_count={}\n",
        assembled.epoch_sha256_hex,
        assembled.epoch_short(),
        assembled.volatile_count,
    );
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(path, body);
}

/// Build Spec 10 inputs from a live Grok turn request surface.
///
/// - `system_prompt`: leading system message text (or empty)
/// - `tools`: tool definitions attached to the request (order preserved)
/// - `cwd`: session display / workspace path
/// - `volatile_count`: number of non-system conversation items (tail size proxy)
pub fn path_a_inputs_from_turn(
    system_prompt: &str,
    tools: &[ToolSpec],
    cwd: &str,
    skills_index: Vec<Spec10SkillIndexEntry>,
    project_instructions: String,
    volatile_count: usize,
) -> Spec10PathAInputs {
    let volatile_tail_text = (0..volatile_count)
        .map(|i| format!("volatile-item-{i}"))
        .collect();
    Spec10PathAInputs {
        system_prompt: system_prompt.to_string(),
        tools: tools.to_vec(),
        skills_index,
        environment: Spec10EnvironmentSummary {
            os_family: std::env::consts::OS.to_string(),
            cwd: cwd.to_string(),
        },
        project_instructions,
        volatile_tail_text,
    }
}

/// Strip a previously applied Spec 10 section block so re-assembly is idempotent.
///
/// The product base system prompt is everything before the first
/// `\n\n## Tools\n` marker introduced by this module.
pub fn extract_base_system_prompt(system_content: &str) -> String {
    const MARKER: &str = "\n\n## Tools\n";
    match system_content.find(MARKER) {
        Some(i) => system_content[..i].to_string(),
        None => system_content.to_string(),
    }
}

/// Discover standing project instructions (Spec 10 §1.4) under `workspace_root`.
///
/// Order: DEEPSEEK.md | DEEPSEEK_BUILD.md (first found), AGENTS.md,
/// `.deepseek-build/instructions.md`. Missing files skip. Best-effort IO.
pub fn discover_project_instructions(workspace_root: &std::path::Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    for name in ["DEEPSEEK.md", "DEEPSEEK_BUILD.md"] {
        let p = workspace_root.join(name);
        if p.is_file() {
            if let Ok(body) = std::fs::read_to_string(&p) {
                let body = body.replace("\r\n", "\n").replace('\r', "\n");
                parts.push(format!("### {name}\n\n{}", body.trim_end()));
            }
            break;
        }
    }
    let agents = workspace_root.join("AGENTS.md");
    if agents.is_file() {
        if let Ok(body) = std::fs::read_to_string(&agents) {
            let body = body.replace("\r\n", "\n").replace('\r', "\n");
            parts.push(format!("### AGENTS.md\n\n{}", body.trim_end()));
        }
    }
    let nested = workspace_root
        .join(".deepseek-build")
        .join("instructions.md");
    if nested.is_file() {
        if let Ok(body) = std::fs::read_to_string(&nested) {
            let body = body.replace("\r\n", "\n").replace('\r', "\n");
            parts.push(format!(
                "### instructions.md\n\n{}",
                body.trim_end()
            ));
        }
    }
    parts.join("\n\n---\n\n")
}

/// Discover skills **index only** (name + one-line description) under workspace.
///
/// Roots: `{ws}/skills/*/SKILL.md`, `{ws}/.deepseek-build/skills/*/SKILL.md`,
/// optional `user_skills_root`. Sorted by name. Best-effort IO.
pub fn discover_skills_index(
    workspace: &std::path::Path,
    user_skills_root: Option<&std::path::Path>,
) -> Vec<Spec10SkillIndexEntry> {
    let mut roots = vec![
        workspace.join("skills"),
        workspace.join(".deepseek-build").join("skills"),
    ];
    if let Some(u) = user_skills_root {
        roots.push(u.to_path_buf());
    }
    let mut by_name: std::collections::BTreeMap<String, Spec10SkillIndexEntry> =
        std::collections::BTreeMap::new();
    for root in roots {
        let Ok(rd) = std::fs::read_dir(&root) else {
            continue;
        };
        for ent in rd.flatten() {
            let path = ent.path();
            if !path.is_dir() {
                continue;
            }
            let skill_md = path.join("SKILL.md");
            if !skill_md.is_file() {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            if name.is_empty() || name.starts_with('.') {
                continue;
            }
            let Ok(raw) = std::fs::read_to_string(&skill_md) else {
                continue;
            };
            let description = extract_skill_description(&raw);
            by_name.insert(
                name.clone(),
                Spec10SkillIndexEntry { name, description },
            );
        }
    }
    by_name.into_values().collect()
}

fn extract_skill_description(raw: &str) -> String {
    // Prefer YAML frontmatter description: lines; else first non-empty body line.
    let text = raw.trim();
    if let Some(rest) = text.strip_prefix("---") {
        if let Some(end) = rest.find("\n---") {
            let fm = &rest[..end];
            for line in fm.lines() {
                let line = line.trim();
                if let Some(v) = line.strip_prefix("description:") {
                    let v = v.trim().trim_matches('"').trim_matches('\'');
                    if !v.is_empty() {
                        return v.to_string();
                    }
                }
            }
        }
    }
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') || t.starts_with("---") {
            continue;
        }
        return t.chars().take(120).collect();
    }
    String::new()
}

/// Run Spec 10 Path A turn assembly from request-shaped inputs and stamp/log.
///
/// Production Path A call site helper (VC007). Best-effort; never panics.
pub fn apply_spec10_path_a_turn_assembly(
    system_prompt: &str,
    tools: &[ToolSpec],
    cwd: &str,
    skills_index: Vec<Spec10SkillIndexEntry>,
    project_instructions: String,
    volatile_count: usize,
) -> Spec10PathAAssembled {
    let base = extract_base_system_prompt(system_prompt);
    let inputs = path_a_inputs_from_turn(
        &base,
        tools,
        cwd,
        skills_index,
        project_instructions,
        volatile_count,
    );
    let assembled = assemble_spec10_path_a_turn(&inputs);
    stamp_spec10_turn_epoch(&assembled, None);
    tracing::debug!(
        epoch = %assembled.epoch_short(),
        volatile_count = assembled.volatile_count,
        stable_bytes = assembled.stable_prefix_bytes.len(),
        "Spec 10 Path A turn assembly (VC007)"
    );
    xai_grok_telemetry::unified_log::debug(
        "shell.turn.spec10_path_a_assembly",
        None,
        Some(serde_json::json!({
            "prefix_epoch": assembled.epoch_short(),
            "prefix_epoch_full": assembled.epoch_sha256_hex,
            "volatile_count": assembled.volatile_count,
            "stable_bytes": assembled.stable_prefix_bytes.len(),
            "wire_system_rewritten": true,
        })),
    );
    assembled
}

/// Apply Spec 10 assembly **to a live ConversationRequest** (mutates system message).
///
/// This is the Path A wire-honest path for V2-10-1: `messages_to_api` leading
/// system content becomes Spec 10 ordered stable prefix layout. Tools remain
/// in the API `tools[]` field as well (Grok hybrid; document also in system).
///
/// Returns the assembled epoch metadata. Best-effort; never panics.
pub fn apply_spec10_to_conversation_request(
    request: &mut xai_grok_sampling_types::ConversationRequest,
    cwd: &str,
    workspace_root: Option<&std::path::Path>,
    user_skills_root: Option<&std::path::Path>,
) -> Spec10PathAAssembled {
    let system_prompt = request
        .items
        .iter()
        .find_map(|item| match item {
            xai_grok_sampling_types::ConversationItem::System(sys) => {
                Some(sys.content.as_ref().to_string())
            }
            _ => None,
        })
        .unwrap_or_default();
    let volatile_count = request
        .items
        .iter()
        .filter(|item| {
            !matches!(
                item,
                xai_grok_sampling_types::ConversationItem::System(_)
            )
        })
        .count();

    let root = workspace_root
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(cwd));
    let skills_index = discover_skills_index(&root, user_skills_root);
    let project_instructions = discover_project_instructions(&root);

    let assembled = apply_spec10_path_a_turn_assembly(
        &system_prompt,
        &request.tools,
        cwd,
        skills_index,
        project_instructions,
        volatile_count,
    );

    // Mutate wire: leading System content = Spec 10 stable body.
    let mut found_system = false;
    for item in &mut request.items {
        if let xai_grok_sampling_types::ConversationItem::System(sys) = item {
            sys.content = std::sync::Arc::<str>::from(assembled.stable_body.as_str());
            found_system = true;
            break;
        }
    }
    if !found_system {
        request.items.insert(
            0,
            xai_grok_sampling_types::ConversationItem::system(assembled.stable_body.clone()),
        );
    }
    assembled
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tool(name: &str, params: serde_json::Value) -> ToolSpec {
        ToolSpec {
            name: name.into(),
            description: Some(format!("tool {name}")),
            parameters: params,
        }
    }

    fn base_inputs() -> Spec10PathAInputs {
        Spec10PathAInputs {
            system_prompt: "SYSTEM_FIXED".into(),
            tools: vec![tool(
                "search_replace",
                json!({"type":"object","properties":{"file_path":{"type":"string"}}}),
            )],
            skills_index: vec![Spec10SkillIndexEntry {
                name: "pr-authoring".into(),
                description: "Write PRs".into(),
            }],
            environment: Spec10EnvironmentSummary {
                os_family: "macos".into(),
                cwd: "/proj".into(),
            },
            project_instructions: "AGENTS.md body".into(),
            volatile_tail_text: vec!["hello".into()],
        }
    }

    #[test]
    fn vc007_identical_inputs_same_epoch() {
        let a = assemble_spec10_path_a_turn(&base_inputs());
        let b = assemble_spec10_path_a_turn(&base_inputs());
        assert_eq!(a.stable_prefix_bytes, b.stable_prefix_bytes);
        assert_eq!(a.epoch_sha256_hex, b.epoch_sha256_hex);
        assert_eq!(a.epoch_short().len(), 16);
    }

    #[test]
    fn vc007_volatile_tail_does_not_change_epoch() {
        let mut inputs = base_inputs();
        let a = assemble_spec10_path_a_turn(&inputs);
        inputs.volatile_tail_text = vec!["t1".into(), "t2".into(), "t3".into()];
        let b = assemble_spec10_path_a_turn(&inputs);
        assert_eq!(a.epoch_sha256_hex, b.epoch_sha256_hex);
        assert!(b.volatile_count > a.volatile_count);
    }

    #[test]
    fn vc007_tool_schema_change_changes_epoch() {
        let mut inputs = base_inputs();
        let a = assemble_spec10_path_a_turn(&inputs);
        inputs.tools[0].description = Some("edit carefully".into());
        let b = assemble_spec10_path_a_turn(&inputs);
        assert_ne!(a.epoch_sha256_hex, b.epoch_sha256_hex);
    }

    #[test]
    fn vc007_sorted_tool_schema_keys() {
        let t1 = tool(
            "read_file",
            json!({"type":"object","properties":{"b":1,"a":{"z":1,"y":2}}}),
        );
        let t2 = tool(
            "read_file",
            json!({"type":"object","properties":{"a":{"y":2,"z":1},"b":1}}),
        );
        assert_eq!(tools_document(&[t1]), tools_document(&[t2]));
    }

    #[test]
    fn vc007_layout_order_sections() {
        let assembled = assemble_spec10_path_a_turn(&base_inputs());
        let body = &assembled.stable_body;
        let i_sys = body.find("SYSTEM_FIXED").expect("system");
        let i_tools = body.find("## Tools\n").expect("tools");
        let i_skills = body.find("## Skills index\n").expect("skills");
        let i_env = body.find("## Environment\n").expect("env");
        let i_proj = body.find("## Project instructions\n").expect("project");
        assert!(i_sys < i_tools);
        assert!(i_tools < i_skills);
        assert!(i_skills < i_env);
        assert!(i_env < i_proj);
    }

    #[test]
    fn vc007_no_wall_clock_in_stable_body() {
        let assembled = assemble_spec10_path_a_turn(&base_inputs());
        let body = &assembled.stable_body;
        for needle in [
            "Utc::now",
            "SystemTime",
            "timestamp",
            "unix_time",
            "Current Date",
            "2026-08-08T",
        ] {
            assert!(
                !body.contains(needle),
                "stable body must not contain wall-clock marker {needle:?}"
            );
        }
    }

    #[test]
    fn vc007_skills_index_sorted_by_name() {
        let mut inputs = base_inputs();
        inputs.skills_index = vec![
            Spec10SkillIndexEntry {
                name: "zeta".into(),
                description: "z".into(),
            },
            Spec10SkillIndexEntry {
                name: "alpha".into(),
                description: "a".into(),
            },
        ];
        let assembled = assemble_spec10_path_a_turn(&inputs);
        let skills_pos = assembled.stable_body.find("## Skills index\n").unwrap();
        let skills_section = &assembled.stable_body[skills_pos..];
        let alpha = skills_section.find("alpha").expect("alpha");
        let zeta = skills_section.find("zeta").expect("zeta");
        assert!(alpha < zeta, "skills index must sort by name");
    }

    #[test]
    fn vc007_stamp_writes_under_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        let assembled = assemble_spec10_path_a_turn(&base_inputs());
        stamp_spec10_turn_epoch(&assembled, Some(dir.path()));
        let path = dir.path().join("path_a_turn_prefix_epoch.txt");
        let body = std::fs::read_to_string(&path).expect("stamp file");
        assert!(body.contains("path_a_turn_prefix_epoch="));
        assert!(body.contains(&assembled.epoch_sha256_hex));
        assert!(body.contains("path_a_turn_volatile_count=1"));
    }

    #[test]
    fn vc007_apply_helper_returns_epoch() {
        let tools = vec![tool(
            "read_file",
            json!({"type":"object","properties":{"target_file":{"type":"string"}}}),
        )];
        let a = apply_spec10_path_a_turn_assembly(
            "SYS",
            &tools,
            "/ws",
            vec![],
            String::new(),
            2,
        );
        let b = apply_spec10_path_a_turn_assembly(
            "SYS",
            &tools,
            "/ws",
            vec![],
            String::new(),
            9,
        );
        assert_eq!(a.epoch_sha256_hex, b.epoch_sha256_hex);
        assert_eq!(a.volatile_count, 2);
        assert_eq!(b.volatile_count, 9);
    }

    #[test]
    fn vc007_extract_base_system_idempotent() {
        let assembled = assemble_spec10_path_a_turn(&base_inputs());
        let base = extract_base_system_prompt(&assembled.stable_body);
        assert_eq!(base, "SYSTEM_FIXED");
        // Re-assemble from extracted base → same epoch
        let mut inputs = base_inputs();
        inputs.system_prompt = base;
        let again = assemble_spec10_path_a_turn(&inputs);
        assert_eq!(again.epoch_sha256_hex, assembled.epoch_sha256_hex);
    }

    #[test]
    fn vc007_wire_rewrite_mutates_system_message() {
        let tools = vec![tool(
            "read_file",
            json!({"type":"object","properties":{"target_file":{"type":"string"}}}),
        )];
        let mut req = xai_grok_sampling_types::ConversationRequest {
            items: vec![
                xai_grok_sampling_types::ConversationItem::system("GROK_BASE_TEMPLATE"),
                xai_grok_sampling_types::ConversationItem::user("hello"),
            ],
            tools: tools.clone(),
            hosted_tools: vec![],
            tool_choice: None,
            model: Some("deepseek-v4-flash".into()),
            temperature: None,
            max_output_tokens: None,
            top_p: None,
            x_grok_conv_id: None,
            x_grok_req_id: None,
            x_grok_session_id: None,
            x_grok_turn_idx: None,
            x_grok_agent_id: None,
            x_grok_deployment_id: None,
            x_grok_user_id: None,
            trace: None,
            reasoning_effort: None,
            json_schema: None,
            prompt_cache_key: None,
        };
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("AGENTS.md"), "project rules").unwrap();
        let assembled = apply_spec10_to_conversation_request(
            &mut req,
            dir.path().to_str().unwrap(),
            Some(dir.path()),
            None,
        );
        let sys = match &req.items[0] {
            xai_grok_sampling_types::ConversationItem::System(s) => s.content.as_ref(),
            _ => panic!("expected system first"),
        };
        assert!(sys.contains("GROK_BASE_TEMPLATE"));
        assert!(sys.contains("## Tools\n"));
        assert!(sys.contains("## Skills index\n"));
        assert!(sys.contains("## Environment\n"));
        assert!(sys.contains("## Project instructions\n"));
        assert!(sys.contains("project rules"));
        assert_eq!(sys, assembled.stable_body.as_str());
        // Second apply is idempotent (epoch stable).
        let e1 = assembled.epoch_sha256_hex.clone();
        let a2 = apply_spec10_to_conversation_request(
            &mut req,
            dir.path().to_str().unwrap(),
            Some(dir.path()),
            None,
        );
        assert_eq!(a2.epoch_sha256_hex, e1);
    }

    #[test]
    fn vc007_discover_project_and_skills() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("AGENTS.md"), "agents body").unwrap();
        let skills = dir.path().join("skills").join("demo");
        std::fs::create_dir_all(&skills).unwrap();
        std::fs::write(
            skills.join("SKILL.md"),
            "---\ndescription: Demo skill\n---\n# Demo\n",
        )
        .unwrap();
        let proj = discover_project_instructions(dir.path());
        assert!(proj.contains("agents body"));
        let idx = discover_skills_index(dir.path(), None);
        assert_eq!(idx.len(), 1);
        assert_eq!(idx[0].name, "demo");
        assert_eq!(idx[0].description, "Demo skill");
    }
}
