use agent_client_protocol as acp;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
pub use xai_grok_permission_rules::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionEvent {
    pub tool_id: String,
    pub tool_name: String,
    pub access_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_detail: Option<String>,
    pub yolo_mode: bool,
    pub auto_approved: bool,
    pub user_prompted: bool,
    pub decision: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reject_reason: Option<String>,
    pub timestamp: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classifier_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classifier_latency_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_denials_consecutive: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_denials_total: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wait_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue_depth: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security_findings: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classifier_verdict: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remember_tool_approvals: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct PermissionResolution {
    pub decision: Decision,
    pub event: Option<PermissionEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ClientType {
    #[default]
    #[serde(rename = "generic", alias = "grok-shell", alias = "grok_shell")]
    Generic,
    #[serde(rename = "grok-tui", alias = "grok_tui")]
    GrokTUI,
    #[serde(rename = "grok_web")]
    GrokWeb,
    #[serde(rename = "nebula")]
    Nebula,
    #[serde(rename = "extension")]
    Extension,
    #[serde(rename = "grok-pager", alias = "grok_pager")]
    GrokPager,
    #[serde(rename = "grok_desktop")]
    Desktop,
}

impl ClientType {
    pub fn user_agent_label(&self) -> &'static str {
        match self {
            Self::Generic => "grok-shell",
            Self::GrokTUI => "grok-tui",
            Self::GrokWeb => "grok-web",
            Self::Nebula => "nebula",
            Self::Extension => "grok-code-extension",
            Self::GrokPager => "grok-pager",
            Self::Desktop => "grok-desktop",
        }
    }

    pub fn from_client_identifier(id: Option<&str>) -> Self {
        match id {
            Some("grok-web") => Self::GrokWeb,
            Some("nebula") => Self::Nebula,
            Some("grok-code-extension") => Self::Extension,
            Some("grok-desktop") => Self::Desktop,
            Some("grok-pager") => Self::GrokPager,
            _ => Self::Generic,
        }
    }

    pub fn feedback_label(&self) -> &'static str {
        match self {
            Self::GrokTUI | Self::GrokPager => "tui",
            Self::GrokWeb => "web",
            Self::Nebula => "nebula",
            Self::Extension => "extension",
            Self::Generic => "agent",
            Self::Desktop => "desktop",
        }
    }

    pub const fn can_present_permission_prompt(self) -> bool {
        !matches!(self, Self::Generic)
    }
}

#[derive(Debug, Clone)]
pub struct RequestPathContext {
    pub real_cwd: std::path::PathBuf,
    pub display_cwd: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookAsk {
    pub hook_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

pub const HOOK_ASK_META_KEY: &str = "hookAsk";

const HOOK_ASK_SEPARATOR: &str = " — ";

impl HookAsk {
    pub fn ask_line(&self) -> String {
        let hook_name = &self.hook_name;
        let reason = self.reason.as_deref().unwrap_or_default();
        let reason = reason.split_whitespace().collect::<Vec<_>>().join(" ");
        if reason.is_empty() {
            format!("hook '{hook_name}' asks for confirmation")
        } else {
            format!("hook '{hook_name}' asks: {reason}")
        }
    }

    pub fn prompt_header(&self, action: &str) -> String {
        format!("{action}{HOOK_ASK_SEPARATOR}{}", self.ask_line())
    }

    pub fn strip_prompt_header<'a>(&self, title: &'a str) -> &'a str {
        title
            .strip_suffix(self.ask_line().as_str())
            .and_then(|action| action.strip_suffix(HOOK_ASK_SEPARATOR))
            .unwrap_or(title)
    }
}

#[derive(Debug, Clone)]
pub struct PermissionRequest {
    pub access: AccessKind,
    pub tool_call_update: acp::ToolCallUpdate,
    pub path_context: Option<RequestPathContext>,
    pub session_id: Option<String>,
    pub subagent_type: Option<String>,
    pub subagent_description: Option<String>,
    pub hook_ask: Option<HookAsk>,
}

impl PermissionRequest {
    pub fn new(access: AccessKind, tool_call_update: acp::ToolCallUpdate) -> Self {
        Self {
            access,
            tool_call_update,
            path_context: None,
            session_id: None,
            subagent_type: None,
            subagent_description: None,
            hook_ask: None,
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub enum PermissionCommand {
    Request {
        request: PermissionRequest,
        respond_to: oneshot::Sender<PermissionResolution>,
    },
    SetYoloMode(bool),
    SetAutoMode(bool),
    SetClassifier(Option<std::sync::Arc<dyn super::auto_mode::PermissionClassifier>>),
    SetClassifierTranscript(Vec<super::auto_mode::ClassifierTurn>),
    SetProjectInstructions(Option<String>),
    ResetState,
    Shutdown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_ask_header_keeps_the_action_and_names_the_hook() {
        let with_reason = HookAsk {
            hook_name: "guard".to_owned(),
            reason: Some("confirm this".to_owned()),
        };
        let header = with_reason.prompt_header("Run `deploy`");
        assert_eq!(header, "Run `deploy` — hook 'guard' asks: confirm this");
        assert_eq!(with_reason.strip_prompt_header(&header), "Run `deploy`");
        let bare = HookAsk {
            hook_name: "guard".to_owned(),
            reason: None,
        };
        assert_eq!(
            bare.prompt_header("Run `deploy`"),
            "Run `deploy` — hook 'guard' asks for confirmation"
        );
        let blank = HookAsk {
            hook_name: "guard".to_owned(),
            reason: Some("  \n".to_owned()),
        };
        assert_eq!(blank.ask_line(), bare.ask_line());
        let multiline = HookAsk {
            hook_name: "guard".to_owned(),
            reason: Some("confirm\nthis".to_owned()),
        };
        assert_eq!(multiline.ask_line(), with_reason.ask_line());
    }

    #[test]
    fn permission_event_subagent_fields_default_to_none() {
        let json = r#"{
            "tool_id": "tc1",
            "tool_name": "bash",
            "access_kind": "bash",
            "yolo_mode": false,
            "auto_approved": false,
            "user_prompted": true,
            "decision": "allow",
            "timestamp": "2026-03-24T00:00:00Z"
        }"#;
        let event: PermissionEvent = serde_json::from_str(json).unwrap();
        assert!(event.subagent_session_id.is_none());
        assert!(event.subagent_type.is_none());
        assert!(event.subagent_description.is_none());
        assert!(event.permission_mode.is_none());
        assert!(event.decision_reason.is_none());
        assert!(event.classifier_source.is_none());
        assert!(event.classifier_latency_ms.is_none());
        assert!(event.auto_denials_consecutive.is_none());
        assert!(event.auto_denials_total.is_none());
        assert!(event.wait_ms.is_none());
        assert!(event.queue_depth.is_none());
        assert!(event.security_findings.is_none());
        assert!(event.classifier_verdict.is_none());
    }

    #[test]
    fn permission_event_findings_none_vs_some_empty_are_distinct() {
        let base = r#"{
            "tool_id": "tc1",
            "tool_name": "bash",
            "access_kind": "bash",
            "yolo_mode": false,
            "auto_approved": false,
            "user_prompted": true,
            "decision": "allow",
            "timestamp": "2026-03-24T00:00:00Z",
            "security_findings": [],
            "classifier_verdict": "block"
        }"#;
        let event: PermissionEvent = serde_json::from_str(base).unwrap();
        assert_eq!(event.security_findings.as_deref(), Some([].as_slice()));
        assert_eq!(event.classifier_verdict.as_deref(), Some("block"));

        let with_tokens: PermissionEvent = serde_json::from_str(&base.replace(
            "\"security_findings\": []",
            "\"security_findings\": [\"opaque_shell\"]",
        ))
        .unwrap();
        assert_eq!(
            with_tokens.security_findings.as_deref(),
            Some(["opaque_shell".to_owned()].as_slice())
        );
    }

    #[test]
    fn permission_event_with_subagent_attribution() {
        let event = PermissionEvent {
            tool_id: "tc1".into(),
            tool_name: "bash".into(),
            access_kind: "bash".into(),
            access_detail: None,
            yolo_mode: false,
            auto_approved: false,
            user_prompted: true,
            decision: "allow".into(),
            prompt_outcome: None,
            reject_reason: None,
            timestamp: Utc::now(),
            subagent_session_id: Some("child-1".into()),
            subagent_type: Some("explore".into()),
            subagent_description: Some("Find endpoints".into()),
            permission_mode: Some("ask".into()),
            decision_reason: Some("needs_user".into()),
            classifier_source: Some("llm".into()),
            classifier_latency_ms: Some(42),
            auto_denials_consecutive: Some(2),
            auto_denials_total: Some(5),
            wait_ms: Some(1234),
            queue_depth: Some(3),
            security_findings: Some(vec!["opaque_shell".into()]),
            classifier_verdict: Some("block".into()),
            remember_tool_approvals: Some(true),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(
            json.get("subagent_session_id").and_then(|v| v.as_str()),
            Some("child-1")
        );
        assert_eq!(
            json.get("subagent_type").and_then(|v| v.as_str()),
            Some("explore")
        );
        assert_eq!(
            json.get("subagent_description").and_then(|v| v.as_str()),
            Some("Find endpoints")
        );
        assert_eq!(
            json.get("permission_mode").and_then(|v| v.as_str()),
            Some("ask")
        );
        assert_eq!(
            json.get("decision_reason").and_then(|v| v.as_str()),
            Some("needs_user")
        );
        assert_eq!(
            json.get("classifier_source").and_then(|v| v.as_str()),
            Some("llm")
        );
        assert_eq!(
            json.get("classifier_latency_ms"),
            Some(&serde_json::json!(42))
        );
        assert_eq!(
            json.get("auto_denials_consecutive"),
            Some(&serde_json::json!(2))
        );
        assert_eq!(json.get("auto_denials_total"), Some(&serde_json::json!(5)));
        assert_eq!(json.get("wait_ms"), Some(&serde_json::json!(1234)));
        assert_eq!(json.get("queue_depth"), Some(&serde_json::json!(3)));
        assert_eq!(
            json.get("security_findings")
                .and_then(|v| v.get(0))
                .and_then(|v| v.as_str()),
            Some("opaque_shell")
        );
        assert_eq!(
            json.get("classifier_verdict").and_then(|v| v.as_str()),
            Some("block")
        );
        assert_eq!(
            json.get("remember_tool_approvals"),
            Some(&serde_json::json!(true))
        );
    }

    #[test]
    fn permission_event_skips_none_optional_fields() {
        let event = PermissionEvent {
            tool_id: "tc1".into(),
            tool_name: "bash".into(),
            access_kind: "bash".into(),
            access_detail: None,
            yolo_mode: false,
            auto_approved: true,
            user_prompted: false,
            decision: "allow".into(),
            prompt_outcome: None,
            reject_reason: None,
            timestamp: Utc::now(),
            subagent_session_id: None,
            subagent_type: None,
            subagent_description: None,
            permission_mode: None,
            decision_reason: None,
            classifier_source: None,
            classifier_latency_ms: None,
            auto_denials_consecutive: None,
            auto_denials_total: None,
            wait_ms: None,
            queue_depth: None,
            security_findings: None,
            classifier_verdict: None,
            remember_tool_approvals: None,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(!json.contains("subagent_session_id"));
        assert!(!json.contains("subagent_type"));
        assert!(!json.contains("permission_mode"));
        assert!(!json.contains("decision_reason"));
        assert!(!json.contains("classifier_source"));
        assert!(!json.contains("classifier_latency_ms"));
        assert!(!json.contains("auto_denials_consecutive"));
        assert!(!json.contains("auto_denials_total"));
        assert!(!json.contains("wait_ms"));
        assert!(!json.contains("queue_depth"));
        assert!(!json.contains("security_findings"));
        assert!(!json.contains("classifier_verdict"));
        assert!(!json.contains("remember_tool_approvals"));
    }
    #[test]
    fn hashline_edit_maps_to_edit_access() {
        use xai_grok_tools::implementations::grok_build_hashline::edit::types::HashlineEditInput;
        use xai_grok_tools::types::ToolInput;
        let input = ToolInput::HashlineEdit(HashlineEditInput {
            file_path: "src/main.rs".into(),
            edits: vec![],
        });
        let access = AccessKind::from(&input);
        assert!(
            matches!(access, AccessKind::Edit(ref p) if p == "src/main.rs"),
            "HashlineEdit should produce AccessKind::Edit with the file path, got {access:?}"
        );
    }
    #[test]
    fn bash_maps_to_bash_access() {
        use xai_grok_tools::implementations::grok_build::bash::BashToolInput;
        use xai_grok_tools::types::ToolInput;
        let input = ToolInput::Bash(BashToolInput {
            command: "cargo test".into(),
            timeout: None,
            description: "run tests".into(),
            is_background: false,
        });
        let access = AccessKind::from(&input);
        assert!(
            matches!(access, AccessKind::Bash(ref cmd) if cmd == "cargo test"),
            "Bash should produce AccessKind::Bash with the command, got {access:?}"
        );
    }
    #[test]
    fn active_agent_message_maps_to_dedicated_access_without_text() {
        use xai_grok_tools::implementations::grok_build::send_subagent_message::SendSubagentMessageInput;
        use xai_grok_tools::types::ToolInput;
        let text = "private follow-up";
        let access = AccessKind::from(&ToolInput::SendSubagentMessage(SendSubagentMessageInput {
            subagent_id: "sub-1".into(),
            text: text.into(),
            delivery: None,
            queue: false,
        }));
        let AccessKind::AgentMessage { subagent_id } = access else {
            panic!("active agent messages must use dedicated access")
        };
        assert_eq!(subagent_id, "sub-1");
        assert!(!subagent_id.contains(text));
    }
    #[test]
    fn send_feedback_maps_to_tool_access() {
        use xai_grok_tools::types::ToolInput;
        let input: ToolInput = serde_json::from_value(serde_json::json!({
            "variant": "SendFeedback",
            "title": "Draft title",
            "details": "What happened:\n- The tool failed.",
            "type": "bug"
        }))
        .unwrap();
        assert!(matches!(
            AccessKind::from(&input),
            AccessKind::Tool(name) if name == "send_feedback"
        ));
        assert!(matches!(
            AccessKind::from(&ToolInput::Dynamic(serde_json::json!({
                "name": "send_feedback",
                "title": "Draft title",
                "details": "What happened",
            }))),
            AccessKind::Tool(name) if name == "send_feedback"
        ));
    }
    #[test]
    fn use_tool_maps_to_mcp_tool_access() {
        use xai_grok_tools::implementations::use_tool::UseToolInput;
        use xai_grok_tools::types::ToolInput;
        let input = ToolInput::UseTool(UseToolInput::Inline(
            xai_grok_tools::implementations::use_tool::InlineMcpInvocation {
                tool_name: "linear__save_issue".into(),
                tool_input: serde_json::json!({ "title" : "test" }),
            },
        ));
        let access = AccessKind::from(&input);
        assert!(
            matches!(
                access,
                AccessKind::MCPTool { ref name, ref input }
                    if name == "linear__save_issue"
                        && input.get("title").and_then(|v| v.as_str()) == Some("test")
            ),
            "UseTool should produce AccessKind::MCPTool carrying the inner tool name and args, got {access:?}"
        );
    }
    #[test]
    fn monitor_maps_to_bash_access() {
        use xai_grok_tools::implementations::grok_build::monitor::types::MonitorInput;
        use xai_grok_tools::types::ToolInput;
        let input = ToolInput::Monitor(MonitorInput {
            command: "tail -f /var/log/syslog".into(),
            description: "watch syslog".into(),
            timeout_ms: None,
            persistent: false,
        });
        let access = AccessKind::from(&input);
        assert!(
            matches!(access, AccessKind::Bash(ref cmd) if cmd == "tail -f /var/log/syslog"),
            "Monitor runs shell and must map to AccessKind::Bash (not Read), got {access:?}"
        );
    }
    #[test]
    fn search_replace_maps_to_edit_access() {
        use xai_grok_tools::implementations::grok_build::search_replace::SearchReplaceInput;
        use xai_grok_tools::types::ToolInput;
        let input = ToolInput::SearchReplace(SearchReplaceInput {
            file_path: "lib.rs".into(),
            old_string: "old".into(),
            new_string: "new".into(),
            replace_all: false,
            file_version: None,
            snippet_id: None,
        });
        let access = AccessKind::from(&input);
        assert!(
            matches!(access, AccessKind::Edit(ref p) if p == "lib.rs"),
            "SearchReplace should produce AccessKind::Edit, got {access:?}"
        );
    }
    #[test]
    fn web_fetch_maps_to_web_fetch_access() {
        use xai_grok_tools::implementations::grok_build::web_fetch::WebFetchInput;
        use xai_grok_tools::types::ToolInput;
        let input = ToolInput::WebFetch(WebFetchInput {
            url: "https://custom.example.com/api".into(),
        });
        let access = AccessKind::from(&input);
        assert!(
            matches!(access, AccessKind::WebFetch(ref u) if u == "https://custom.example.com/api"),
            "WebFetch should produce AccessKind::WebFetch with the URL, got {access:?}"
        );
    }
    #[test]
    fn web_search_maps_to_web_search_access() {
        use xai_grok_tools::implementations::grok_build::web_search::WebSearchInput;
        use xai_grok_tools::types::ToolInput;
        let input = ToolInput::WebSearch(WebSearchInput {
            query: "rust lang".into(),
            allowed_domains: None,
        });
        let access = AccessKind::from(&input);
        assert!(
            matches!(access, AccessKind::WebSearch(ref q) if q == "rust lang"),
            "WebSearch should produce AccessKind::WebSearch with the query, got {access:?}"
        );
    }
    /// The patch text names its files; no grant scope can vouch for them, so a patch prompts every time.
    #[test]
    fn apply_patch_maps_to_tool_access() {
        use xai_grok_tools::implementations::codex::apply_patch::ApplyPatchInput;
        use xai_grok_tools::types::ToolInput;
        let input = ToolInput::ApplyPatch(ApplyPatchInput {
            patch: "*** Begin Patch\n*** Update File: /home/user/.grok/mcp.json\n*** End Patch"
                .to_owned(),
        });
        assert!(matches!(
            AccessKind::from(&input),
            AccessKind::Tool(name) if name == "apply_patch"
        ));
    }
    #[test]
    fn write_tool_maps_to_edit_access() {
        use xai_grok_tools::implementations::opencode::write::WriteInput;
        use xai_grok_tools::types::ToolInput;
        let input = ToolInput::Write(WriteInput {
            file_path: "/tmp/secret.txt".into(),
            content: "overwritten".into(),
        });
        let access = AccessKind::from(&input);
        assert!(
            matches!(access, AccessKind::Edit(ref p) if p == "/tmp/secret.txt"),
            "Write should produce AccessKind::Edit with the file path, got {access:?}"
        );
    }
    #[test]
    fn write_scoped_and_dynamic_inputs_map_to_edit_not_read() {
        use xai_grok_tools::implementations::opencode::edit::EditInput;
        use xai_grok_tools::types::ToolInput;
        use xai_tool_types::TaskToolInput;
        let edit = ToolInput::from(EditInput {
            file_path: "/tmp/denied.txt".into(),
            old_string: "ORIGINAL".into(),
            new_string: "BYPASS".into(),
            replace_all: false,
        });
        assert!(matches!(
            &edit,
            ToolInput::SearchReplace(sr) if sr.file_path == "/tmp/denied.txt"
        ));
        assert!(matches!(
            AccessKind::from(&edit),
            AccessKind::Edit(p) if p == "/tmp/denied.txt"
        ));
        assert!(matches!(
            AccessKind::from(&ToolInput::Task(TaskToolInput {
                prompt: "edit config.toml".into(),
                description: "spawn".into(),
                subagent_type: "general-purpose".into(),
                subagent_type_specified: false,
                run_in_background: false,
                capability_mode: None,
                isolation: None,
                resume_from: None,
                cwd: None,
                model: None,
                workspace: None,
                task_id: None,
            })),
            AccessKind::Tool(name) if name == "task"
        ));
        assert!(matches!(
            AccessKind::from(&ToolInput::Dynamic(serde_json::json!({
                "filePath": "/tmp/denied.txt",
                "oldString": "a",
                "newString": "b",
            }))),
            AccessKind::Edit(p) if p == "/tmp/denied.txt"
        ));
        assert!(matches!(
            AccessKind::from(&ToolInput::Dynamic(serde_json::json!({
                "filePath": "src/main.rs"
            }))),
            AccessKind::Read(Some(p)) if p == "src/main.rs"
        ));
        assert!(matches!(
            AccessKind::from(&ToolInput::Dynamic(serde_json::json!({
                "command": "rm -rf /"
            }))),
            AccessKind::Bash(c) if c == "rm -rf /"
        ));
    }
    #[test]
    fn client_type_deserializes_grok_shell_as_generic() {
        assert_eq!(
            serde_json::from_value::<ClientType>("grok-shell".into()).unwrap(),
            ClientType::Generic,
        );
        assert_eq!(
            serde_json::from_value::<ClientType>("grok_shell".into()).unwrap(),
            ClientType::Generic,
        );
        assert_eq!(
            serde_json::from_value::<ClientType>("generic".into()).unwrap(),
            ClientType::Generic,
        );
    }
}
