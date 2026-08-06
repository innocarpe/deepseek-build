//! Tool runtime for DeepSeek Build (M2).
//!
//! - Spec **45**: session snippets + `read` / `edit` / `write`
//! - Spec **90** minimum: path scopes, policy, bash classifier

mod permissions;
mod snippets;
mod tools;

pub use permissions::{
    classify_bash, decide, default_coding_policy, Decision, PermissionError, PermissionPolicy,
    Scope,
};
pub use snippets::{EditError, Snippet, SnippetStore, WriteError};
pub use tools::{
    tool_definitions, ToolError, ToolExecutor, ToolName, ToolRequest, ToolResponse,
};
