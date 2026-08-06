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
pub use grants::{AskChoice, GRANTS_FILE, PermissionGrants};
pub use mcp::{
    McpCatalog, McpCatalogEntry, McpConfigFile, McpError, catalog_from_config,
    catalog_tool_definitions, load_mcp_config, wire_name,
};
pub use permissions::{
    Decision, PermissionError, PermissionPolicy, Scope, classify_bash, decide,
    default_coding_policy, dogfood_coding_policy,
};
pub use plan::{PlanError, PlanItem, PlanStore};
pub use snippets::{EditError, Snippet, SnippetStore, WriteError};
pub use tools::{
    AskCallback, CORE_TOOL_NAMES, ToolError, ToolExecutor, ToolName, ToolRequest, ToolResponse,
    core_tool_names, tool_definitions, tool_definitions_with_options, tool_definitions_with_plan,
};
