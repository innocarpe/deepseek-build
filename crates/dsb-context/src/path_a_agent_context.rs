//! Path A agent context assembly — Spec 10 spirit under the Grok agent path.
//!
//! Thin Path B already uses [`PrefixBuilder`] / [`PrefixEpoch`]. Heart fusion
//! (G006) requires the **same stable-prefix + volatile-tail discipline** to be
//! the contract for the default full-screen agent (Path A), not only `dsb run`.
//!
//! This module is the product-side assembly API that Path A must call (or
//! mirror) when building DeepSeek Chat Completions messages. Grok compaction
//! may still run; it must not inject wall-clock / random IDs into the stable
//! prefix section defined here.
//!
//! See: `docs/architecture/HEART_3X_SPEC_BINDING.md`,
//! `docs/product/HEART_3X_P0_TEST_PLAN.md` (H10.*).

use dsb_provider_deepseek::{ChatMessage, ToolDefinition};

use crate::prefix::{
    EnvironmentSummary, PrefixBuildInputs, PrefixBuilder, PrefixError, SkillIndexEntry,
    StablePrefix, VolatileTail,
};
use crate::{PrefixEpoch, assemble_messages};

/// Inputs for one agent turn under Path A (Grok shell + DeepSeek models).
#[derive(Debug, Clone)]
pub struct PathAContextInputs {
    pub system_prompt: String,
    pub tools: Vec<ToolDefinition>,
    pub skills_index: Vec<SkillIndexEntry>,
    pub environment: EnvironmentSummary,
    pub project_instructions: String,
    /// Volatile: current user turn + tool chain (not in stable prefix).
    pub volatile_user_and_tools: Vec<ChatMessage>,
}

impl PathAContextInputs {
    fn prefix_inputs(&self) -> PrefixBuildInputs {
        PrefixBuildInputs {
            system_prompt: self.system_prompt.clone(),
            tools: self.tools.clone(),
            skills_index: self.skills_index.clone(),
            environment: self.environment.clone(),
            project_instructions: self.project_instructions.clone(),
        }
    }
}

/// Built Path A request: stable prefix + epoch + full messages list.
#[derive(Debug, Clone)]
pub struct PathAAssembledContext {
    pub stable: StablePrefix,
    pub messages: Vec<ChatMessage>,
}

impl PathAAssembledContext {
    pub fn epoch(&self) -> &PrefixEpoch {
        &self.stable.epoch
    }

    pub fn epoch_short(&self) -> String {
        self.stable.epoch.short().to_string()
    }
}

/// Assemble Path A agent context (Spec 10).
///
/// Stable sections are byte-stable for identical inputs; volatile tail does not
/// affect the epoch.
pub fn assemble_path_a_context(
    inputs: &PathAContextInputs,
) -> Result<PathAAssembledContext, PrefixError> {
    let builder = PrefixBuilder::new();
    let stable = builder.build(&inputs.prefix_inputs())?;
    let tail = VolatileTail {
        messages: inputs.volatile_user_and_tools.clone(),
    };
    let messages = assemble_messages(&stable, &tail);
    Ok(PathAAssembledContext { stable, messages })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prefix::tool_from_params;
    use dsb_provider_deepseek::ChatMessage;
    use serde_json::json;

    fn base_inputs() -> PathAContextInputs {
        PathAContextInputs {
            system_prompt: "SYSTEM_FIXED".into(),
            tools: vec![tool_from_params(
                "search_replace",
                json!({"type":"object","properties":{"file_path":{"type":"string"}}}),
            )],
            skills_index: vec![SkillIndexEntry {
                name: "pr-authoring".into(),
                description: "Write PRs".into(),
            }],
            environment: EnvironmentSummary {
                os_family: "macos".into(),
                cwd: "/proj".into(),
            },
            project_instructions: "AGENTS.md body".into(),
            volatile_user_and_tools: vec![ChatMessage::user("hello")],
        }
    }

    #[test]
    fn h10_1_identical_inputs_same_epoch() {
        let a = assemble_path_a_context(&base_inputs()).unwrap();
        let b = assemble_path_a_context(&base_inputs()).unwrap();
        assert_eq!(a.stable.bytes, b.stable.bytes);
        assert_eq!(a.epoch().sha256_hex, b.epoch().sha256_hex);
    }

    #[test]
    fn h10_2_tool_schema_change_changes_epoch() {
        let mut inputs = base_inputs();
        let a = assemble_path_a_context(&inputs).unwrap();
        inputs.tools[0].function.description = Some("edit files carefully".into());
        let b = assemble_path_a_context(&inputs).unwrap();
        assert_ne!(a.epoch().sha256_hex, b.epoch().sha256_hex);
    }

    #[test]
    fn h10_3_volatile_tail_does_not_change_epoch() {
        let mut inputs = base_inputs();
        let a = assemble_path_a_context(&inputs).unwrap();
        inputs.volatile_user_and_tools = vec![
            ChatMessage::user("turn 1"),
            ChatMessage::assistant("ok"),
            ChatMessage::user("turn 2"),
        ];
        let b = assemble_path_a_context(&inputs).unwrap();
        assert_eq!(a.epoch().sha256_hex, b.epoch().sha256_hex);
        assert!(b.messages.len() > a.messages.len());
    }

    #[test]
    fn h10_skills_index_change_changes_epoch() {
        let mut inputs = base_inputs();
        let a = assemble_path_a_context(&inputs).unwrap();
        inputs.skills_index.push(SkillIndexEntry {
            name: "new-skill".into(),
            description: "x".into(),
        });
        let b = assemble_path_a_context(&inputs).unwrap();
        assert_ne!(a.epoch().sha256_hex, b.epoch().sha256_hex);
    }

    #[test]
    fn path_a_messages_start_with_stable_prefix() {
        let assembled = assemble_path_a_context(&base_inputs()).unwrap();
        assert!(!assembled.stable.messages.is_empty());
        assert!(assembled.messages.len() >= assembled.stable.messages.len());
        assert_eq!(
            &assembled.messages[..assembled.stable.messages.len()],
            assembled.stable.messages.as_slice()
        );
    }
}
