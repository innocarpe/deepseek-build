//! Tool-call / tool-result pairing repair (spec 15 §1 / session load).

use dsb_provider_deepseek::{ChatMessage, Role, ToolCall};

/// Content written into synthetic tool results for interrupted calls.
pub const PAIRING_INTERRUPTED_CONTENT: &str =
    r#"{"error":"tool_result_interrupted","message":"tool call had no result; session repaired"}"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterruptedTool {
    pub tool_call_id: String,
    pub name: String,
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

        if msg.role == Role::Assistant {
            if let Some(calls) = &msg.tool_calls {
                if !calls.is_empty() {
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
                        });
                        out.push(ChatMessage::tool_result(
                            call.id,
                            PAIRING_INTERRUPTED_CONTENT,
                        ));
                    }
                    i = j;
                    continue;
                }
            }
        }
        i += 1;
    }
    (out, interrupted)
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
        assert!(c2
            .content
            .as_ref()
            .unwrap()
            .contains("tool_result_interrupted"));
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
        assert_eq!(
            fixed[0].reasoning_content.as_deref(),
            Some("think")
        );
    }
}
