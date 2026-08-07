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
    let inputs = path_a_inputs_from_turn(
        system_prompt,
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
        })),
    );
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
}
