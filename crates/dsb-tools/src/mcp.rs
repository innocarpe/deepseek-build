//! MCP catalog + schema fingerprint (spec 80).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("invalid MCP config: {0}")]
    Config(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpServerConfig {
    pub name: String,
    #[serde(default = "default_transport")]
    pub transport: String,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    /// Optional static tools for tests / offline catalogs (no live process).
    #[serde(default)]
    pub static_tools: Vec<McpStaticTool>,
}

fn default_transport() -> String {
    "stdio".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpStaticTool {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "empty_object")]
    pub input_schema: Value,
}

fn empty_object() -> Value {
    json!({"type": "object", "properties": {}})
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct McpConfigFile {
    #[serde(default)]
    pub servers: Vec<McpServerConfig>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct McpCatalogEntry {
    pub wire_name: String,
    pub server: String,
    pub remote_name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone, Default)]
pub struct McpCatalog {
    pub entries: Vec<McpCatalogEntry>,
    pub fingerprint_hex: String,
}

impl McpCatalog {
    pub fn from_entries(mut entries: Vec<McpCatalogEntry>) -> Self {
        entries.sort_by(|a, b| a.wire_name.cmp(&b.wire_name));
        let fingerprint_hex = fingerprint_entries(&entries);
        Self {
            entries,
            fingerprint_hex,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Document for stable prefix / epoch (canonical JSON array).
    pub fn schema_document(&self) -> Value {
        Value::Array(
            self.entries
                .iter()
                .map(|e| {
                    json!({
                        "wire_name": e.wire_name,
                        "server": e.server,
                        "remote_name": e.remote_name,
                        "description": e.description,
                        "input_schema": e.input_schema,
                    })
                })
                .collect(),
        )
    }
}

fn fingerprint_entries(entries: &[McpCatalogEntry]) -> String {
    let doc = Value::Array(
        entries
            .iter()
            .map(|e| {
                json!({
                    "wire_name": e.wire_name,
                    "server": e.server,
                    "remote_name": e.remote_name,
                    "description": e.description,
                    "input_schema": e.input_schema,
                })
            })
            .collect(),
    );
    // Compact deterministic JSON (serde_json map order is insertion; we control keys).
    let bytes = serde_json::to_vec(&doc).unwrap_or_default();
    let mut h = Sha256::new();
    h.update(&bytes);
    hex::encode(h.finalize())
}

/// Load MCP config from workspace then user home (workspace wins on name collision).
pub fn load_mcp_config(
    workspace: &Path,
    user_home: Option<&Path>,
) -> Result<McpConfigFile, McpError> {
    let mut merged = McpConfigFile::default();
    let mut by_name: BTreeMap<String, McpServerConfig> = BTreeMap::new();
    let paths: Vec<std::path::PathBuf> = [
        user_home.map(|h| h.join("mcp.json")),
        Some(workspace.join(".deepseek-build").join("mcp.json")),
    ]
    .into_iter()
    .flatten()
    .collect();
    // Apply user first, workspace second so workspace overrides.
    for path in paths {
        if !path.is_file() {
            continue;
        }
        let raw = fs::read_to_string(&path)?;
        let file: McpConfigFile = serde_json::from_str(&raw)?;
        for s in file.servers {
            validate_server_name(&s.name)?;
            by_name.insert(s.name.clone(), s);
        }
    }
    merged.servers = by_name.into_values().collect();
    Ok(merged)
}

pub fn validate_server_name(name: &str) -> Result<(), McpError> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
    {
        return Err(McpError::Config(format!(
            "invalid server name '{name}' (use [a-z0-9_-]+)"
        )));
    }
    Ok(())
}

pub fn sanitize_tool_name(name: &str) -> Option<String> {
    if name.is_empty() {
        return None;
    }
    if name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        Some(name.to_string())
    } else {
        None
    }
}

pub fn wire_name(server: &str, tool: &str) -> Option<String> {
    let t = sanitize_tool_name(tool)?;
    Some(format!("mcp__{server}__{t}"))
}

/// Build catalog from static_tools in config (v1 offline path; live stdio can fill later).
pub fn catalog_from_config(cfg: &McpConfigFile) -> Result<McpCatalog, McpError> {
    let mut entries = Vec::new();
    for srv in &cfg.servers {
        validate_server_name(&srv.name)?;
        for tool in &srv.static_tools {
            let Some(wire) = wire_name(&srv.name, &tool.name) else {
                continue;
            };
            entries.push(McpCatalogEntry {
                wire_name: wire,
                server: srv.name.clone(),
                remote_name: tool.name.clone(),
                description: tool.description.clone(),
                input_schema: tool.input_schema.clone(),
            });
        }
    }
    Ok(McpCatalog::from_entries(entries))
}

/// Convert catalog entries to OpenAI-style tool definitions for the model.
pub fn catalog_tool_definitions(
    catalog: &McpCatalog,
) -> Vec<dsb_provider_deepseek::ToolDefinition> {
    use dsb_provider_deepseek::{ToolDefinition, ToolFunction};
    catalog
        .entries
        .iter()
        .map(|e| ToolDefinition {
            type_: "function".into(),
            function: ToolFunction {
                name: e.wire_name.clone(),
                description: Some(format!("[MCP {}] {}", e.server, e.description)),
                parameters: Some(e.input_schema.clone()),
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn fingerprint_stable_and_changes() {
        let a = McpCatalog::from_entries(vec![McpCatalogEntry {
            wire_name: "mcp__s__t".into(),
            server: "s".into(),
            remote_name: "t".into(),
            description: "d".into(),
            input_schema: json!({"type": "object"}),
        }]);
        let b = McpCatalog::from_entries(vec![McpCatalogEntry {
            wire_name: "mcp__s__t".into(),
            server: "s".into(),
            remote_name: "t".into(),
            description: "d".into(),
            input_schema: json!({"type": "object"}),
        }]);
        assert_eq!(a.fingerprint_hex, b.fingerprint_hex);
        let c = McpCatalog::from_entries(vec![
            a.entries[0].clone(),
            McpCatalogEntry {
                wire_name: "mcp__s__u".into(),
                server: "s".into(),
                remote_name: "u".into(),
                description: "other".into(),
                input_schema: json!({"type": "object"}),
            },
        ]);
        assert_ne!(a.fingerprint_hex, c.fingerprint_hex);
    }

    #[test]
    fn invalid_server_name() {
        assert!(validate_server_name("Bad Name").is_err());
        assert!(validate_server_name("good_1").is_ok());
    }

    #[test]
    fn load_workspace_config() {
        let dir = tempdir().unwrap();
        let cfg_dir = dir.path().join(".deepseek-build");
        fs::create_dir_all(&cfg_dir).unwrap();
        fs::write(
            cfg_dir.join("mcp.json"),
            r#"{
              "servers": [{
                "name": "demo",
                "static_tools": [{
                  "name": "ping",
                  "description": "Ping",
                  "input_schema": {"type": "object", "properties": {}}
                }]
              }]
            }"#,
        )
        .unwrap();
        let cfg = load_mcp_config(dir.path(), None).unwrap();
        let cat = catalog_from_config(&cfg).unwrap();
        assert_eq!(cat.entries.len(), 1);
        assert_eq!(cat.entries[0].wire_name, "mcp__demo__ping");
        assert!(!cat.fingerprint_hex.is_empty());
    }
}
