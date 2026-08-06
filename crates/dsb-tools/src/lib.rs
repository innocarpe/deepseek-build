//! Tool runtime for DeepSeek Build.
//!
//! - Spec **40**: core tools surface (catalog + wire schemas)
//! - Spec **45**: session snippets + `read` / `edit` / `write`
//! - Spec **90** minimum: path scopes, policy, bash classifier
//! - Spec **70**: on-demand `skill` body load
//! - Daily coding: `grep` + dogfood profile (`dogfood_coding_policy`)

mod bg_shell;
mod grants;
mod mcp;
mod permissions;
mod plan;
mod snippets;
mod tools;

pub use bg_shell::{BgJobStore, JobSnapshot};
pub use grants::{AskChoice, PermissionGrants, GRANTS_FILE};
pub use mcp::{
    catalog_from_config, catalog_tool_definitions, load_mcp_config, wire_name, McpCatalog,
    McpCatalogEntry, McpConfigFile, McpError,
};
pub use permissions::{
    classify_bash, decide, default_coding_policy, dogfood_coding_policy, Decision, PermissionError,
    PermissionPolicy, Scope,
};
pub use plan::{PlanError, PlanItem, PlanStore};
pub use snippets::{EditError, Snippet, SnippetStore, WriteError};
pub use tools::{
    core_tool_names, tool_definitions, tool_definitions_with_options, tool_definitions_with_plan,
    AskCallback, ToolError, ToolExecutor, ToolName, ToolRequest, ToolResponse, CORE_TOOL_NAMES,
};
