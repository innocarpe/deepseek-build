//! Path A (default Grok agent + DeepSeek models) turn economics — Spec 15 + 20.
//!
//! Thin Path B already owns [`repair_tool_arguments`] and [`ModelRouter`].
//! Heart fusion (G007) binds those as the **default DeepSeek turn** policy
//! under the full-screen agent: repair before tool execute; Flash-first with
//! dogfoodable Pro escalate and per-turn wire-model visibility.
//!
//! See: `docs/architecture/HEART_3X_SPEC_BINDING.md`,
//! `docs/product/HEART_3X_P0_TEST_PLAN.md` (H15.*, H20.*).

use serde_json::Value;

use crate::repair::{RepairError, RepairOutcome, repair_tool_arguments};
use crate::routing::{ModelRouter, Preset, RouteDecision};

/// One tool call from the model on a Path A DeepSeek turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathAToolCall {
    pub name: String,
    pub raw_arguments: String,
}

/// Result of preparing a tool call for execute under Spec 15.
#[derive(Debug, Clone, PartialEq)]
pub enum PathAToolPrep {
    /// Safe to execute with repaired/validated args.
    Ready {
        name: String,
        arguments: Value,
        repair_applied: bool,
    },
    /// Do **not** execute; structured error for the model.
    Reject {
        name: String,
        error: String,
    },
}

/// Product Path A defaults for a new agent session (Spec 20).
pub fn path_a_default_router() -> ModelRouter {
    let mut r = ModelRouter::new(Preset::Flash);
    // Auto-router optional; product default is Flash with explicit /pro escalate.
    r.set_auto_router(false);
    r
}

/// Spec 15: repair then fail closed — never execute invalid args on Path A.
pub fn prepare_path_a_tool_call(
    call: &PathAToolCall,
    schema: Option<&Value>,
) -> PathAToolPrep {
    match repair_tool_arguments(&call.raw_arguments, schema) {
        Ok(RepairOutcome {
            arguments,
            repair_applied,
            ..
        }) => PathAToolPrep::Ready {
            name: call.name.clone(),
            arguments,
            repair_applied,
        },
        Err(e) => PathAToolPrep::Reject {
            name: call.name.clone(),
            error: e.to_string(),
        },
    }
}

/// Route a Path A turn and return the visibility line for logs/UI (Spec 20 §2.3).
pub fn route_path_a_turn(router: &mut ModelRouter, user_text: &str) -> RouteDecision {
    router.route_turn(user_text)
}

/// Wire model ids used by product (ADR 0005) — both must have base_url in config.
pub fn path_a_flash_wire_id() -> &'static str {
    dsb_provider_deepseek::MODEL_FLASH
}

pub fn path_a_pro_wire_id() -> &'static str {
    dsb_provider_deepseek::MODEL_PRO
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::apply_routing_command;
    use serde_json::json;

    #[test]
    fn h15_1_trailing_comma_repairs_then_ready() {
        let call = PathAToolCall {
            name: "search_replace".into(),
            raw_arguments: r#"{"file_path":"a.rs","old_string":"x","new_string":"y",}"#.into(),
        };
        match prepare_path_a_tool_call(&call, None) {
            PathAToolPrep::Ready {
                name,
                arguments,
                repair_applied,
            } => {
                assert_eq!(name, "search_replace");
                assert!(repair_applied);
                assert_eq!(arguments["file_path"], "a.rs");
            }
            PathAToolPrep::Reject { error, .. } => panic!("expected ready: {error}"),
        }
    }

    #[test]
    fn h15_2_missing_required_no_execute() {
        let schema = json!({
            "type": "object",
            "properties": {
                "file_path": {"type": "string"},
                "old_string": {"type": "string"},
                "new_string": {"type": "string"}
            },
            "required": ["file_path", "old_string", "new_string"],
            "additionalProperties": false
        });
        let call = PathAToolCall {
            name: "search_replace".into(),
            raw_arguments: r#"{"file_path":"a.rs"}"#.into(),
        };
        match prepare_path_a_tool_call(&call, Some(&schema)) {
            PathAToolPrep::Reject { name, error } => {
                assert_eq!(name, "search_replace");
                assert!(error.contains("missing") || error.contains("required"));
            }
            PathAToolPrep::Ready { .. } => panic!("must not execute incomplete args"),
        }
    }

    #[test]
    fn h15_3_does_not_rename_tool() {
        let call = PathAToolCall {
            name: "search_replace".into(),
            raw_arguments: r#"{"x":1}"#.into(),
        };
        match prepare_path_a_tool_call(&call, None) {
            PathAToolPrep::Ready { name, .. } => assert_eq!(name, "search_replace"),
            PathAToolPrep::Reject { name, .. } => assert_eq!(name, "search_replace"),
        }
    }

    #[test]
    fn h15_unrepairable_reject() {
        let call = PathAToolCall {
            name: "bash".into(),
            raw_arguments: "not-json".into(),
        };
        assert!(matches!(
            prepare_path_a_tool_call(&call, None),
            PathAToolPrep::Reject { .. }
        ));
        // Ensure underlying error type is still Spec 15 shaped
        let _ = RepairError::InvalidJson("x".into());
    }

    #[test]
    fn h20_1_default_flash() {
        let mut r = path_a_default_router();
        let d = route_path_a_turn(&mut r, "hello");
        assert_eq!(d.wire_model, path_a_flash_wire_id());
        assert!(d.visibility_line().contains(path_a_flash_wire_id()));
    }

    #[test]
    fn h20_2_pro_once_then_return_flash() {
        let mut r = path_a_default_router();
        let (text, cmd) = apply_routing_command(&mut r, "/pro design this");
        assert_eq!(cmd, Some("pro_once"));
        let d1 = route_path_a_turn(&mut r, &text);
        assert_eq!(d1.wire_model, path_a_pro_wire_id());
        let d2 = route_path_a_turn(&mut r, "next");
        assert_eq!(d2.wire_model, path_a_flash_wire_id());
    }

    #[test]
    fn h20_3_visibility_line_shows_wire_model() {
        let mut r = path_a_default_router();
        let d = route_path_a_turn(&mut r, "hi");
        let line = d.visibility_line();
        assert!(line.contains("model="));
        assert!(line.contains(path_a_flash_wire_id()));
    }

    #[test]
    fn h20_4_both_wire_ids_are_deepseek_v4() {
        assert_eq!(path_a_flash_wire_id(), "deepseek-v4-flash");
        assert_eq!(path_a_pro_wire_id(), "deepseek-v4-pro");
    }
}
