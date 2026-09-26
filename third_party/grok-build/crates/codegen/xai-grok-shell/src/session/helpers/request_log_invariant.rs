//! Request/log projection check (depth board U1.1).
//!
//! The session log is the conversation `build_request` read. The outgoing
//! request may then rewrite system messages (Spec 10 §1.10, memory reminder)
//! and trim old tool results. Every other item must still be the log item.
//! A log item that cannot be serialized fails closed: continuing would send
//! a request the session cannot restore.

use xai_chat_state::compaction_utils::AGENT_MESSAGE_MODEL_LABEL;
use xai_chat_state::{HARD_CLEAR_PLACEHOLDER, SOFT_TRIM_SEPARATOR};
use xai_grok_sampling_types::conversation::{ContentPart, ConversationItem, SyntheticReason};

/// Why the outgoing request must not be sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestLogDivergence {
    pub detail: String,
}

/// `Ok` when `request` is the log plus the legal projections: system-message
/// rewrites, tool-result trims, and the agent-message label prepended on the
/// request copy. System messages are not compared: Spec 10 writes and appends
/// them after the log is read.
pub fn check_request_projects_log(
    log: &[ConversationItem],
    request: &[ConversationItem],
) -> Result<(), RequestLogDivergence> {
    let log_volatile: Vec<&ConversationItem> = log.iter().filter(|item| !is_system(item)).collect();
    let request_volatile: Vec<&ConversationItem> =
        request.iter().filter(|item| !is_system(item)).collect();
    if log_volatile.len() != request_volatile.len() {
        return Err(diverge(format!(
            "request/log desync: log has {} non-system items, request has {}",
            log_volatile.len(),
            request_volatile.len()
        )));
    }
    for (index, (log_item, request_item)) in
        log_volatile.iter().zip(request_volatile.iter()).enumerate()
    {
        match (log_item, request_item) {
            (
                ConversationItem::ToolResult(log_result),
                ConversationItem::ToolResult(request_result),
            ) => {
                if log_result.tool_call_id != request_result.tool_call_id {
                    return Err(diverge(format!(
                        "request/log desync at non-system item {index}: tool_call_id"
                    )));
                }
                if !tool_content_projects(
                    log_result.content.as_ref(),
                    request_result.content.as_ref(),
                ) {
                    return Err(diverge(format!(
                        "request/log desync at non-system item {index}: tool result is not the logged result or a known trim"
                    )));
                }
                let log_images = serde_json::to_value(&log_result.images)
                    .map_err(|_| diverge(format!("log item {index} cannot be restored")))?;
                let request_images = serde_json::to_value(&request_result.images)
                    .map_err(|_| diverge(format!("request item {index} cannot be restored")))?;
                if log_images != request_images {
                    return Err(diverge(format!(
                        "request/log desync at non-system item {index}: tool result images"
                    )));
                }
            }
            _ => {
                if !non_system_projects(log_item, request_item)
                    .map_err(|detail| diverge(format!("{detail} at non-system item {index}")))?
                {
                    return Err(diverge(format!(
                        "request/log desync at non-system item {index}"
                    )));
                }
            }
        }
    }
    Ok(())
}

fn is_system(item: &ConversationItem) -> bool {
    matches!(item, ConversationItem::System(_))
}

/// `Ok(true)` when the request item is the log item, or the log item plus the
/// agent-message label that `ModelRequestHistory` prepends on the request copy.
fn non_system_projects(
    log_item: &ConversationItem,
    request_item: &ConversationItem,
) -> Result<bool, String> {
    let log_json =
        serde_json::to_value(log_item).map_err(|_| "log item cannot be restored".to_string())?;
    let request_json = serde_json::to_value(request_item)
        .map_err(|_| "request item cannot be restored".to_string())?;
    if log_json == request_json {
        return Ok(true);
    }
    let Some(stripped) = strip_agent_message_label(request_item) else {
        return Ok(false);
    };
    let stripped_json = serde_json::to_value(&stripped)
        .map_err(|_| "request item cannot be restored".to_string())?;
    Ok(log_json == stripped_json)
}

fn strip_agent_message_label(item: &ConversationItem) -> Option<ConversationItem> {
    let ConversationItem::User(user) = item else {
        return None;
    };
    if user.synthetic_reason != SyntheticReason::AgentMessage {
        return None;
    }
    let ContentPart::Text { text } = user.content.first()? else {
        return None;
    };
    if text.as_ref() != AGENT_MESSAGE_MODEL_LABEL {
        return None;
    }
    let mut user = user.clone();
    user.content.remove(0);
    Some(ConversationItem::User(user))
}

fn diverge(detail: String) -> RequestLogDivergence {
    RequestLogDivergence { detail }
}

/// The request tool text is the log text, the hard-clear placeholder, or a
/// head/tail trim that still occurs inside the logged text.
fn tool_content_projects(log: &str, request: &str) -> bool {
    if log == request {
        return true;
    }
    if request == HARD_CLEAR_PLACEHOLDER {
        return true;
    }
    let Some((head, tail)) = request.split_once(SOFT_TRIM_SEPARATOR) else {
        return false;
    };
    !head.is_empty()
        && !tail.is_empty()
        && request.len() < log.len()
        && log.contains(head)
        && log.contains(tail)
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_grok_sampling_types::conversation::ConversationItem;

    fn user(text: &str) -> ConversationItem {
        ConversationItem::user(text)
    }

    fn tool(id: &str, content: &str) -> ConversationItem {
        ConversationItem::tool_result(id, content)
    }

    #[test]
    fn matching_volatile_items_pass_and_system_rewrites_are_ignored() {
        let log = vec![
            ConversationItem::system("template"),
            user("hello"),
            tool("call-1", "full output"),
        ];
        let request = vec![
            ConversationItem::system("template\n\n## Tools\n[]"),
            user("hello"),
            tool("call-1", "full output"),
            ConversationItem::system("later body"),
        ];
        assert!(check_request_projects_log(&log, &request).is_ok());
    }

    #[test]
    fn an_extra_user_message_on_the_request_fails_loudly() {
        let log = vec![user("hello")];
        let request = vec![user("hello"), user("injected")];
        let err = check_request_projects_log(&log, &request).unwrap_err();
        assert!(err.detail.contains("desync"), "{}", err.detail);
    }

    #[test]
    fn a_changed_user_message_fails_loudly() {
        let log = vec![user("hello")];
        let request = vec![user("hello!")];
        assert!(check_request_projects_log(&log, &request).is_err());
    }

    #[test]
    fn a_known_tool_trim_projects_and_a_different_result_does_not() {
        let log_text = format!("HEAD-MARKER-{}-TAIL-MARKER", "x".repeat(80));
        let log = vec![tool("call-1", &log_text)];
        let trimmed = format!("HEAD-MARKER-{SOFT_TRIM_SEPARATOR}-TAIL-MARKER");
        let request = vec![tool("call-1", &trimmed)];
        assert!(check_request_projects_log(&log, &request).is_ok());

        let cleared = vec![tool("call-1", HARD_CLEAR_PLACEHOLDER)];
        assert!(check_request_projects_log(&log, &cleared).is_ok());

        let forged = vec![tool("call-1", "different")];
        assert!(check_request_projects_log(&log, &forged).is_err());
    }
}
