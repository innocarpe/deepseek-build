//! Prefix builder: stable sections + volatile tail (spec 10).

use std::fs;
use std::path::Path;

use dsb_provider_deepseek::{ChatMessage, ToolDefinition};
use serde::Serialize;
use thiserror::Error;

use crate::canonicalize::stable_prefix_bytes;
use crate::epoch::PrefixEpoch;

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
        let mut system_body = inputs.system_prompt.trim_end().to_string();
        system_body.push_str("\n\n");
        system_body.push_str("## Tools\n");
        system_body.push_str(&tools_document(&inputs.tools)?);
        system_body.push_str("\n\n## Skills index\n");
        system_body.push_str(&skills_document(&inputs.skills_index)?);
        system_body.push_str("\n\n## Environment\n");
        system_body.push_str(&env_document(&inputs.environment)?);
        if !inputs.project_instructions.trim().is_empty() {
            system_body.push_str("\n\n## Project instructions\n");
            system_body.push_str(inputs.project_instructions.trim_end());
            system_body.push('\n');
        }

        let messages = vec![ChatMessage::system(system_body)];
        let messages_json = serde_json::to_value(&messages)?;
        let bytes = stable_prefix_bytes(&messages_json)?;
        let epoch = PrefixEpoch::from_bytes(&bytes);
        Ok(StablePrefix {
            messages,
            bytes,
            epoch,
        })
    }
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
