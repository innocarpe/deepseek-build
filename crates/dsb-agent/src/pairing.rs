//! Tool-call / tool-result pairing repair (spec 15 §1 / session load).
//!
//! A missing result after the assistant message recorded the call is an unknown
//! outcome: the body may have run. A live batch that never dispatched a call
//! records that the call was not started. Both are written before the step
//! returns, so a later request is not left with an unanswered call.

use std::collections::HashSet;

use dsb_provider_deepseek::{ChatMessage, Role, ToolCall};

/// Model-visible text when a recorded call has no durable result.
///
/// Matches the DeepSeek Harness recovery wording at `639ed015`
/// (`packages/core/session/src/repair.ts`, interrupted / started).
pub const OUTCOME_UNKNOWN_TEXT: &str = "The tool call was interrupted after it was recorded, but no result was durably recorded. Its outcome is unknown. Decide whether to retry from the tool semantics: retry only if the operation is read-only or idempotent; if it may have side effects, first verify external state or ask the user. Do not retry blindly.";

/// Model-visible text when this process never dispatched the call.
pub const NOT_STARTED_TEXT: &str = "The tool call was interrupted before it was recorded as started. Retry it if it is still needed.";

/// Synthetic tool result for a recorded call whose body may have run.
pub const PAIRING_INTERRUPTED_CONTENT: &str = concat!(
    r#"{"error":"tool_result_interrupted","code":"TOOL_OUTCOME_UNKNOWN","message":""#,
    "The tool call was interrupted after it was recorded, but no result was durably recorded. Its outcome is unknown. Decide whether to retry from the tool semantics: retry only if the operation is read-only or idempotent; if it may have side effects, first verify external state or ask the user. Do not retry blindly.",
    r#""}"#
);

/// Why a tool call has no result yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnansweredKind {
    /// The call was dispatched, or a saved transcript cannot prove it was not.
    OutcomeUnknown,
    /// The assistant requested it and this batch never started it.
    NotStarted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterruptedTool {
    pub tool_call_id: String,
    pub name: String,
    pub kind: UnansweredKind,
}

/// JSON body for one synthetic tool result.
pub fn recovery_content(kind: UnansweredKind) -> String {
    match kind {
        UnansweredKind::OutcomeUnknown => PAIRING_INTERRUPTED_CONTENT.to_string(),
        UnansweredKind::NotStarted => format!(
            r#"{{"error":"tool_not_started","code":"TOOL_NOT_STARTED","message":"{NOT_STARTED_TEXT}"}}"#
        ),
    }
}

/// Ensure every assistant `tool_calls` entry has a matching tool result message
/// before the next user/assistant turn or end of transcript.
///
/// Inserts `tool_result_interrupted` placeholders for holes. Never sends unpaired calls.
pub fn pair_tool_results(messages: &[ChatMessage]) -> (Vec<ChatMessage>, Vec<InterruptedTool>) {
    let mut out: Vec<ChatMessage> = Vec::with_capacity(messages.len());
    let mut interrupted = Vec::new();
    let mut i = 0;
    while i < messages.len() {
        let msg = &messages[i];
        out.push(msg.clone());

        if msg.role == Role::Assistant
            && let Some(calls) = &msg.tool_calls
            && !calls.is_empty()
        {
            let mut pending: Vec<ToolCall> = calls.clone();
            let mut j = i + 1;
            while j < messages.len() && messages[j].role == Role::Tool {
                if let Some(id) = &messages[j].tool_call_id {
                    pending.retain(|c| &c.id != id);
                }
                out.push(messages[j].clone());
                j += 1;
            }
            for call in pending {
                interrupted.push(InterruptedTool {
                    tool_call_id: call.id.clone(),
                    name: call.function.name.clone(),
                    kind: UnansweredKind::OutcomeUnknown,
                });
                out.push(ChatMessage::tool_result(
                    call.id,
                    PAIRING_INTERRUPTED_CONTENT,
                ));
            }
            i = j;
            continue;
        }
        i += 1;
    }
    (out, interrupted)
}

/// Append recovery results for calls in `calls` that have no tool result after
/// the latest assistant message.
///
/// `started` holds ids this batch dispatched. Those become
/// [`UnansweredKind::OutcomeUnknown`]. The rest become
/// [`UnansweredKind::NotStarted`]. Order follows `calls`.
pub fn close_unanswered_calls(
    messages: &mut Vec<ChatMessage>,
    calls: &[ToolCall],
    started: &HashSet<String>,
) -> Vec<InterruptedTool> {
    let mut answered = HashSet::new();
    for msg in messages.iter().rev() {
        if msg.role == Role::Assistant {
            break;
        }
        if msg.role == Role::Tool
            && let Some(id) = &msg.tool_call_id
        {
            answered.insert(id.clone());
        }
    }
    let mut recorded = Vec::new();
    for call in calls {
        if answered.contains(&call.id) {
            continue;
        }
        let kind = if started.contains(&call.id) {
            UnansweredKind::OutcomeUnknown
        } else {
            UnansweredKind::NotStarted
        };
        recorded.push(InterruptedTool {
            tool_call_id: call.id.clone(),
            name: call.function.name.clone(),
            kind,
        });
        messages.push(ChatMessage::tool_result(
            call.id.clone(),
            recovery_content(kind),
        ));
        answered.insert(call.id.clone());
    }
    recorded
}

/// Whether the transcript currently has tools in play (assistant tool_calls present).
pub fn tools_in_play(messages: &[ChatMessage]) -> bool {
    messages.iter().any(|m| {
        m.role == Role::Assistant
            && m.tool_calls
                .as_ref()
                .map(|t| !t.is_empty())
                .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dsb_provider_deepseek::{FunctionCall, ToolCall};

    fn assistant_with_calls(calls: Vec<ToolCall>) -> ChatMessage {
        ChatMessage::assistant_with_reasoning(Some("".into()), Some("think".into()), Some(calls))
    }

    fn call(id: &str, name: &str) -> ToolCall {
        ToolCall {
            id: id.into(),
            type_: "function".into(),
            function: FunctionCall {
                name: name.into(),
                arguments: "{}".into(),
            },
        }
    }

    #[test]
    fn pairing_inserts_interrupted() {
        let msgs = vec![
            ChatMessage::user("hi"),
            assistant_with_calls(vec![call("c1", "read"), call("c2", "write")]),
            ChatMessage::tool_result("c1", "ok"),
            // c2 missing
            ChatMessage::user("continue"),
        ];
        let (fixed, holes) = pair_tool_results(&msgs);
        assert_eq!(holes.len(), 1);
        assert_eq!(holes[0].tool_call_id, "c2");
        // tool result for c2 inserted before next user
        let tool_ids: Vec<_> = fixed
            .iter()
            .filter(|m| m.role == Role::Tool)
            .filter_map(|m| m.tool_call_id.clone())
            .collect();
        assert!(tool_ids.contains(&"c1".to_string()));
        assert!(tool_ids.contains(&"c2".to_string()));
        let c2 = fixed
            .iter()
            .find(|m| m.tool_call_id.as_deref() == Some("c2"))
            .unwrap();
        let body = c2.content.as_ref().unwrap();
        assert!(body.contains("tool_result_interrupted"));
        assert!(body.contains("TOOL_OUTCOME_UNKNOWN"));
        assert!(body.contains("Do not retry blindly."));
        assert!(body.contains(OUTCOME_UNKNOWN_TEXT));
    }

    #[test]
    fn close_unanswered_distinguishes_started_from_skipped() {
        use std::collections::HashSet;

        let mut messages = vec![
            ChatMessage::user("go"),
            assistant_with_calls(vec![
                call("c1", "read"),
                call("c2", "write"),
                call("c3", "bash"),
            ]),
            ChatMessage::tool_result("c1", "ok"),
        ];
        let calls = vec![call("c1", "read"), call("c2", "write"), call("c3", "bash")];
        let started = HashSet::from([String::from("c2")]);
        let recorded = close_unanswered_calls(&mut messages, &calls, &started);
        assert_eq!(recorded.len(), 2);
        assert_eq!(recorded[0].tool_call_id, "c2");
        assert_eq!(recorded[0].kind, UnansweredKind::OutcomeUnknown);
        assert_eq!(recorded[1].tool_call_id, "c3");
        assert_eq!(recorded[1].kind, UnansweredKind::NotStarted);
        let c2 = messages
            .iter()
            .find(|m| m.tool_call_id.as_deref() == Some("c2"))
            .unwrap();
        let c3 = messages
            .iter()
            .find(|m| m.tool_call_id.as_deref() == Some("c3"))
            .unwrap();
        assert!(
            c2.content
                .as_ref()
                .unwrap()
                .contains("TOOL_OUTCOME_UNKNOWN")
        );
        assert!(c3.content.as_ref().unwrap().contains("TOOL_NOT_STARTED"));
        assert!(
            c3.content
                .as_ref()
                .unwrap()
                .contains("Retry it if it is still needed.")
        );
        let again = close_unanswered_calls(&mut messages, &calls, &started);
        assert!(again.is_empty());
    }

    #[test]
    fn no_hole_when_paired() {
        let msgs = vec![
            assistant_with_calls(vec![call("c1", "read")]),
            ChatMessage::tool_result("c1", "ok"),
        ];
        let (fixed, holes) = pair_tool_results(&msgs);
        assert!(holes.is_empty());
        assert_eq!(fixed.len(), 2);
    }

    #[test]
    fn preserves_reasoning_on_assistant() {
        let msgs = vec![assistant_with_calls(vec![call("c1", "read")])];
        let (fixed, _) = pair_tool_results(&msgs);
        assert_eq!(fixed[0].reasoning_content.as_deref(), Some("think"));
    }
}
