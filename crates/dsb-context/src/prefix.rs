//! Prefix builder: stable sections + volatile tail (spec 10).

use std::fs;
use std::path::Path;

use dsb_provider_deepseek::{ChatMessage, ToolDefinition};
use serde::Serialize;
use thiserror::Error;

use crate::canonicalize::stable_prefix_bytes;
use crate::epoch::PrefixEpoch;
use crate::shape::{PrefixShape, sub_hash};

/// Default system prompt template (no wall-clock, no random IDs).
pub const DEFAULT_SYSTEM_PROMPT: &str = "\
You are DeepSeek Build (dsb), a DeepSeek-native coding agent.\n\
Follow tool contracts exactly. Prefer precise, minimal changes.\n\
Never invent secrets or commit credentials.\n";

#[derive(Debug, Error)]
pub enum PrefixError {
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SkillIndexEntry {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EnvironmentSummary {
    /// OS family: "macos" | "linux" | "windows" | other.
    pub os_family: String,
    /// Workspace cwd normalized (project-relative preferred).
    pub cwd: String,
}

impl EnvironmentSummary {
    pub fn detect(workspace_root: &Path) -> Self {
        let os_family = std::env::consts::OS.to_string();
        let cwd = normalize_path(workspace_root, workspace_root);
        Self { os_family, cwd }
    }
}

/// Inputs that fully determine the stable prefix (no clocks/random).
#[derive(Debug, Clone)]
pub struct PrefixBuildInputs {
    pub system_prompt: String,
    pub tools: Vec<ToolDefinition>,
    pub skills_index: Vec<SkillIndexEntry>,
    pub environment: EnvironmentSummary,
    /// Concatenated standing project instructions (already discovered).
    pub project_instructions: String,
}

impl Default for PrefixBuildInputs {
    fn default() -> Self {
        Self {
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
            tools: Vec::new(),
            skills_index: Vec::new(),
            environment: EnvironmentSummary {
                os_family: "test".into(),
                cwd: ".".into(),
            },
            project_instructions: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StablePrefix {
    pub messages: Vec<ChatMessage>,
    pub bytes: Vec<u8>,
    pub epoch: PrefixEpoch,
    /// Per-component hashes over the same documents that composed `bytes`
    /// (spec 10 §1.5.1). Observational: it never affects `bytes` or `epoch`.
    pub shape: PrefixShape,
}

#[derive(Debug, Clone, Default)]
pub struct VolatileTail {
    pub messages: Vec<ChatMessage>,
}

impl VolatileTail {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_user(&mut self, content: impl Into<String>) {
        self.messages.push(ChatMessage::user(content));
    }

    pub fn push(&mut self, msg: ChatMessage) {
        self.messages.push(msg);
    }
}

#[derive(Debug, Default)]
pub struct PrefixBuilder;

impl PrefixBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn build(&self, inputs: &PrefixBuildInputs) -> Result<StablePrefix, PrefixError> {
        // Component documents, in §1.1 order. The body is assembled from exactly
        // these strings and the shape hashes exactly these strings, so a
        // component hash cannot describe bytes other than the ones emitted.
        let system_component = inputs.system_prompt.trim_end().to_string();
        let tools_component = tools_document(&inputs.tools)?;
        let skills_component = skills_document(&inputs.skills_index)?;
        let environment_component = env_document(&inputs.environment)?;
        let project_instructions_component = inputs.project_instructions.trim_end().to_string();

        let mut system_body = system_component.clone();
        system_body.push_str("\n\n");
        system_body.push_str("## Tools\n");
        system_body.push_str(&tools_component);
        system_body.push_str("\n\n## Skills index\n");
        system_body.push_str(&skills_component);
        system_body.push_str("\n\n## Environment\n");
        system_body.push_str(&environment_component);
        if !inputs.project_instructions.trim().is_empty() {
            system_body.push_str("\n\n## Project instructions\n");
            system_body.push_str(&project_instructions_component);
            system_body.push('\n');
        }

        let shape = PrefixShape {
            system: sub_hash(system_component.as_bytes()),
            tools: sub_hash(tools_component.as_bytes()),
            skills: sub_hash(skills_component.as_bytes()),
            environment: sub_hash(environment_component.as_bytes()),
            project_instructions: sub_hash(project_instructions_component.as_bytes()),
            tool_names: named_tool_hashes(&inputs.tools)?,
            skill_names: named_skill_hashes(&inputs.skills_index)?,
            environment_axes: environment_hashes(&inputs.environment),
        };

        let messages = vec![ChatMessage::system(system_body)];
        let messages_json = serde_json::to_value(&messages)?;
        let bytes = stable_prefix_bytes(&messages_json)?;
        let epoch = PrefixEpoch::from_bytes(&bytes);
        Ok(StablePrefix {
            messages,
            bytes,
            epoch,
            shape,
        })
    }
}

/// name → hash of that tool's canonical document. A repeated name hashes the
/// concatenation, so a change to either occurrence is still named rather than
/// silently dropped.
fn named_tool_hashes(
    tools: &[ToolDefinition],
) -> Result<std::collections::BTreeMap<String, String>, PrefixError> {
    let mut grouped: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for tool in tools {
        let value = serde_json::to_value(tool)?;
        let doc = String::from_utf8(crate::canonicalize::canonicalize_json(&value)?)
            .expect("json is utf-8");
        grouped
            .entry(tool.function.name.clone())
            .or_default()
            .push(doc);
    }
    Ok(grouped
        .into_iter()
        .map(|(name, docs)| (name, sub_hash(docs.join("\n").as_bytes())))
        .collect())
}

/// name → hash of that skill's index entry (name + description).
fn named_skill_hashes(
    skills: &[SkillIndexEntry],
) -> Result<std::collections::BTreeMap<String, String>, PrefixError> {
    let mut out = std::collections::BTreeMap::new();
    for skill in skills {
        let value = serde_json::to_value(skill)?;
        let doc = crate::canonicalize::canonicalize_json(&value)?;
        out.insert(skill.name.clone(), sub_hash(&doc));
    }
    Ok(out)
}

/// Per-field hashes for the environment component. Keys are the sub-axis names
/// the log may print; the values are never logged (spec 10 §1.5.1 rule 3).
fn environment_hashes(env: &EnvironmentSummary) -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    out.insert("cwd".to_string(), sub_hash(env.cwd.as_bytes()));
    out.insert("os_family".to_string(), sub_hash(env.os_family.as_bytes()));
    out
}

fn tools_document(tools: &[ToolDefinition]) -> Result<String, PrefixError> {
    // Canonical JSON array with sorted keys; tool order is input order (caller sorts if needed).
    // Spec: tool schemas document with sorted object keys.
    let value = serde_json::to_value(tools)?;
    let bytes = crate::canonicalize::canonicalize_json(&value)?;
    Ok(String::from_utf8(bytes).expect("json is utf-8"))
}

fn skills_document(skills: &[SkillIndexEntry]) -> Result<String, PrefixError> {
    let mut sorted = skills.to_vec();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    let value = serde_json::to_value(&sorted)?;
    let bytes = crate::canonicalize::canonicalize_json(&value)?;
    Ok(String::from_utf8(bytes).expect("json is utf-8"))
}

fn env_document(env: &EnvironmentSummary) -> Result<String, PrefixError> {
    let value = serde_json::to_value(env)?;
    let bytes = crate::canonicalize::canonicalize_json(&value)?;
    Ok(String::from_utf8(bytes).expect("json is utf-8"))
}

/// Discover standing project instructions (spec 10 §1.4).
///
/// Order:
/// 1. `./DEEPSEEK.md` or `./DEEPSEEK_BUILD.md` (first found)
/// 2. `./AGENTS.md`
/// 3. `./.deepseek-build/instructions.md`
pub fn discover_project_instructions(workspace_root: &Path) -> Result<String, PrefixError> {
    let mut parts: Vec<String> = Vec::new();

    let primary = [
        workspace_root.join("DEEPSEEK.md"),
        workspace_root.join("DEEPSEEK_BUILD.md"),
    ];
    for p in primary {
        if p.is_file() {
            parts.push(read_labeled(&p)?);
            break;
        }
    }

    let agents = workspace_root.join("AGENTS.md");
    if agents.is_file() {
        parts.push(read_labeled(&agents)?);
    }

    let nested = workspace_root
        .join(".deepseek-build")
        .join("instructions.md");
    if nested.is_file() {
        parts.push(read_labeled(&nested)?);
    }

    Ok(parts.join("\n\n---\n\n"))
}

fn read_labeled(path: &Path) -> Result<String, PrefixError> {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("instructions");
    let body = fs::read_to_string(path)?;
    // Normalize newlines to \n only.
    let body = body.replace("\r\n", "\n").replace('\r', "\n");
    Ok(format!("### {name}\n\n{}", body.trim_end()))
}

/// Prefer project-relative path when under workspace root; else absolute normalized.
pub fn normalize_path(workspace_root: &Path, path: &Path) -> String {
    let root = match workspace_root.canonicalize() {
        Ok(r) => r,
        Err(_) => workspace_root.to_path_buf(),
    };
    let abs = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    };
    if let Ok(rel) = abs.strip_prefix(&root) {
        if rel.as_os_str().is_empty() {
            return ".".to_string();
        }
        return path_to_unix(rel);
    }
    path_to_unix(&abs)
}

fn path_to_unix(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
        .trim_start_matches("./")
        .to_string()
}

/// Helper for tests: fixture tools with intentionally shuffled keys when built from Value.
#[cfg(test)]
pub fn tool_from_params(name: &str, params: serde_json::Value) -> ToolDefinition {
    ToolDefinition {
        type_: "function".into(),
        function: dsb_provider_deepseek::ToolFunction {
            name: name.into(),
            description: Some(format!("tool {name}")),
            parameters: Some(params),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_inputs() -> PrefixBuildInputs {
        PrefixBuildInputs {
            system_prompt: "SYSTEM_FIXED".into(),
            tools: vec![tool_from_params(
                "read",
                json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}),
            )],
            skills_index: vec![
                SkillIndexEntry {
                    name: "beta".into(),
                    description: "B".into(),
                },
                SkillIndexEntry {
                    name: "alpha".into(),
                    description: "A".into(),
                },
            ],
            environment: EnvironmentSummary {
                os_family: "macos".into(),
                cwd: ".".into(),
            },
            project_instructions: "Be careful.".into(),
        }
    }

    #[test]
    fn stable_prefix_bytes_golden_lock() {
        // Measured before the §1.5.1 attribution work landed and unchanged by it.
        // If this fails, the shared prefix bytes moved: every existing session's
        // provider cache is invalidated by that change, and the PR that moves it
        // must say so in its cache-impact section (spec 10 §4).
        let p = PrefixBuilder::new().build(&sample_inputs()).unwrap();
        assert_eq!(
            p.epoch.sha256_hex, "c4b3cc9b5c5e847f9e57bb5b21cf3674a0a72859a3fb48dbb8669028c6aba20e",
            "stable prefix bytes moved — this is cache-breaking for live sessions"
        );
        assert_eq!(p.bytes.len(), 463);
    }

    #[test]
    fn shape_is_observational_not_byte_affecting() {
        // The shape must be derivable from the same inputs without perturbing the
        // bytes it describes: rebuild twice and compare bytes, epoch, and shape.
        let b = PrefixBuilder::new();
        let inputs = sample_inputs();
        let p1 = b.build(&inputs).unwrap();
        let p2 = b.build(&inputs).unwrap();
        assert_eq!(p1.bytes, p2.bytes);
        assert_eq!(p1.epoch, p2.epoch);
        assert_eq!(p1.shape, p2.shape);
        assert!(p1.shape.components_equal(&p2.shape));
    }

    #[test]
    fn shape_names_one_axis_per_changed_component() {
        use crate::shape::PrefixChange;
        let b = PrefixBuilder::new();
        let base = sample_inputs();

        // Each mutation touches exactly one §1.1 component, and only that axis
        // must be reported.
        let mut system = base.clone();
        system.system_prompt = "SYSTEM_CHANGED".into();
        let mut tools = base.clone();
        tools.tools.push(tool_from_params(
            "write",
            json!({"type":"object","properties":{}}),
        ));
        let mut skills = base.clone();
        skills.skills_index.push(SkillIndexEntry {
            name: "gamma".into(),
            description: "G".into(),
        });
        let mut environment = base.clone();
        environment.environment.cwd = "/elsewhere".into();
        let mut instructions = base.clone();
        instructions.project_instructions = "Be very careful.".into();

        let base_build = b.build(&base).unwrap();
        for (inputs, expected) in [
            (system, "system"),
            (tools, "tools"),
            (skills, "skills"),
            (environment, "environment"),
            (instructions, "project_instructions"),
        ] {
            let cur = b.build(&inputs).unwrap();
            assert_ne!(
                base_build.epoch.sha256_hex, cur.epoch.sha256_hex,
                "{expected} change must move the epoch"
            );
            let change = PrefixChange::between(&base_build.shape, &cur.shape, true);
            assert_eq!(
                change.label(),
                expected,
                "{expected} change reported the wrong axis set"
            );
            assert!(!change.unattributed);
        }
    }

    #[test]
    fn attribute_detail_holds_names_not_content() {
        use crate::shape::PrefixChange;
        let b = PrefixBuilder::new();
        let base = sample_inputs();
        let mut cur = base.clone();
        cur.skills_index.push(SkillIndexEntry {
            name: "gamma".into(),
            description: "SECRET_SKILL_DESCRIPTION".into(),
        });
        cur.project_instructions = "SECRET_INSTRUCTION_BODY".into();
        let prev_build = b.build(&base).unwrap();
        let cur_build = b.build(&cur).unwrap();
        let change = PrefixChange::between(&prev_build.shape, &cur_build.shape, true);
        let detail = change.detail_label().expect("detail for named axes");
        assert!(detail.contains("skills.added=gamma"));
        assert!(
            !detail.contains("SECRET_SKILL_DESCRIPTION"),
            "detail must not carry skill content: {detail}"
        );
        assert!(
            !detail.contains("SECRET_INSTRUCTION_BODY"),
            "detail must not carry instruction content: {detail}"
        );
    }

    #[test]
    fn prefix_stable_across_two_builds() {
        let b = PrefixBuilder::new();
        let inputs = sample_inputs();
        let p1 = b.build(&inputs).unwrap();
        let p2 = b.build(&inputs).unwrap();
        assert_eq!(p1.bytes, p2.bytes);
        assert_eq!(p1.epoch, p2.epoch);
    }

    #[test]
    fn prefix_changes_when_tool_added() {
        let b = PrefixBuilder::new();
        let mut inputs = sample_inputs();
        let p1 = b.build(&inputs).unwrap();
        inputs.tools.push(tool_from_params(
            "write",
            json!({"type":"object","properties":{}}),
        ));
        let p2 = b.build(&inputs).unwrap();
        assert_ne!(p1.bytes, p2.bytes);
        assert_ne!(p1.epoch.sha256_hex, p2.epoch.sha256_hex);
    }

    #[test]
    fn prefix_no_timestamp() {
        let b = PrefixBuilder::new();
        let p = b.build(&sample_inputs()).unwrap();
        let s = String::from_utf8_lossy(&p.bytes);
        // Fixture system template has no clock / random
        assert!(!s.contains("Utc::now"));
        assert!(!s.to_ascii_lowercase().contains("timestamp"));
        assert_eq!(p.messages.len(), 1);
        assert!(
            p.messages[0]
                .content
                .as_ref()
                .unwrap()
                .starts_with("SYSTEM_FIXED")
        );
    }

    #[test]
    fn sorted_tool_schema_keys_same_bytes() {
        let b = PrefixBuilder::new();
        let mut a = sample_inputs();
        a.tools = vec![tool_from_params(
            "t",
            json!({"b":1,"a":{"z":true,"y":false}}),
        )];
        let mut b_in = sample_inputs();
        b_in.tools = vec![tool_from_params(
            "t",
            json!({"a":{"y":false,"z":true},"b":1}),
        )];
        let p1 = b.build(&a).unwrap();
        let p2 = b.build(&b_in).unwrap();
        assert_eq!(p1.bytes, p2.bytes);
    }

    #[test]
    fn skills_sorted_by_name() {
        let b = PrefixBuilder::new();
        let p = b.build(&sample_inputs()).unwrap();
        let s = p.messages[0].content.as_ref().unwrap();
        let alpha = s.find("\"name\":\"alpha\"").unwrap();
        let beta = s.find("\"name\":\"beta\"").unwrap();
        assert!(alpha < beta, "skills must be sorted by name");
    }

    #[test]
    fn discover_project_instructions_order() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("DEEPSEEK.md"), "deep").unwrap();
        fs::write(dir.path().join("AGENTS.md"), "agents").unwrap();
        fs::create_dir_all(dir.path().join(".deepseek-build")).unwrap();
        fs::write(dir.path().join(".deepseek-build/instructions.md"), "nested").unwrap();
        let text = discover_project_instructions(dir.path()).unwrap();
        assert!(text.contains("### DEEPSEEK.md"));
        assert!(text.contains("deep"));
        assert!(text.contains("### AGENTS.md"));
        assert!(text.contains("agents"));
        assert!(text.contains("### instructions.md"));
        assert!(text.contains("nested"));
        // DEEPSEEK_BUILD.md should not also load when DEEPSEEK.md exists
        assert!(!text.contains("DEEPSEEK_BUILD"));
    }

    #[test]
    fn discover_skips_missing() {
        let dir = tempfile::tempdir().unwrap();
        let text = discover_project_instructions(dir.path()).unwrap();
        assert!(text.is_empty());
    }
}
