//! Tool runtime for DeepSeek Build.
//!
//! - Spec **40**: core tools surface (catalog + wire schemas)
//! - Spec **45**: session snippets + `read` / `edit` / `write`
//! - Spec **90** minimum: path scopes, policy, bash classifier
//! - Spec **70**: on-demand `skill` body load
//! - Daily coding: `grep` + dogfood profile (`dogfood_coding_policy`)

mod permissions;
mod snippets;
mod tools;

pub use permissions::{
    classify_bash, decide, default_coding_policy, dogfood_coding_policy, Decision, PermissionError,
    PermissionPolicy, Scope,
};
pub use snippets::{EditError, Snippet, SnippetStore, WriteError};
pub use tools::{
    core_tool_names, tool_definitions, ToolError, ToolExecutor, ToolName, ToolRequest,
    ToolResponse, CORE_TOOL_NAMES,
};
