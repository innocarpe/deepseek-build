//! Built-in tools: read / edit / write (+ bash classify gate stub).

use std::path::{Path, PathBuf};
use std::process::Command;

use dsb_provider_deepseek::{ToolDefinition, ToolFunction};
use serde_json::{json, Value};
use thiserror::Error;

use crate::permissions::{
    classify_bash, decide, effective_scopes, resolve_workspace_path, scopes_for_path, Decision,
    PathOp, PermissionPolicy, Scope,
};
use crate::snippets::{EditError, SnippetStore, WriteError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolName {
    Read,
    Edit,
    Write,
    Bash,
}

impl ToolName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Edit => "edit",
            Self::Write => "write",
            Self::Bash => "bash",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "read" => Some(Self::Read),
            "edit" => Some(Self::Edit),
            "write" => Some(Self::Write),
            "bash" => Some(Self::Bash),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolRequest {
    pub name: ToolName,
    pub arguments: Value,
}

#[derive(Debug, Clone)]
pub struct ToolResponse {
    pub ok: bool,
    pub content: String,
    /// True if filesystem may have been mutated (snippet invalidation already applied).
    pub mutated: bool,
}

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("unknown tool: {0}")]
    UnknownTool(String),
    #[error("permission: {0}")]
    Permission(String),
    #[error("invalid arguments: {0}")]
    Args(String),
    #[error("{0}")]
    Other(String),
}

pub struct ToolExecutor {
    pub workspace: PathBuf,
    pub policy: PermissionPolicy,
    pub snippets: SnippetStore,
    /// When false, bash only classifies/permission-checks (dry-run).
    pub bash_execute: bool,
}

impl ToolExecutor {
    pub fn new(workspace: PathBuf, policy: PermissionPolicy) -> Self {
        Self {
            workspace,
            policy,
            snippets: SnippetStore::new(),
            bash_execute: false,
        }
    }

    pub fn check(&self, scopes: &[Scope]) -> Result<(), ToolError> {
        match decide(&self.policy, scopes) {
            Decision::Allow => Ok(()),
            Decision::Deny => Err(ToolError::Permission(format!(
                "denied scopes={:?}",
                scopes.iter().map(|s| s.as_str()).collect::<Vec<_>>()
            ))),
            Decision::Ask => Err(ToolError::Permission(format!(
                "needs confirmation scopes={:?}",
                scopes.iter().map(|s| s.as_str()).collect::<Vec<_>>()
            ))),
        }
    }

    pub fn execute(&mut self, req: &ToolRequest) -> Result<ToolResponse, ToolError> {
        match req.name {
            ToolName::Read => self.read(&req.arguments),
            ToolName::Edit => self.edit(&req.arguments),
            ToolName::Write => self.write(&req.arguments),
            ToolName::Bash => self.bash(&req.arguments),
        }
    }

    fn read(&mut self, args: &Value) -> Result<ToolResponse, ToolError> {
        let path = arg_str(args, "path")?;
        let full = resolve_workspace_path(&self.workspace, path);
        let scopes =
            scopes_for_path(&self.workspace, &full, PathOp::Read).map_err(|e| ToolError::Other(e.to_string()))?;
        self.check(&scopes)?;
        let start = arg_usize_opt(args, "start_line");
        let end = arg_usize_opt(args, "end_line");
        let (snip, content) = self
            .snippets
            .issue_for_file(&full, start, end)
            .map_err(|e| ToolError::Other(e.to_string()))?;
        let body = json!({
            "path": path_display(&self.workspace, &full),
            "snippet_id": snip.snippet_id,
            "version": snip.version,
            "start_line": snip.start_line,
            "end_line": snip.end_line,
            "scope": snip.scope,
            "preview": snip.preview,
            "content": content,
        });
        Ok(ToolResponse {
            ok: true,
            content: body.to_string(),
            mutated: false,
        })
    }

    fn edit(&mut self, args: &Value) -> Result<ToolResponse, ToolError> {
        let snippet_id = arg_str(args, "snippet_id")?;
        let old = arg_str(args, "old_string")?;
        let new = arg_str(args, "new_string")?;
        let expected = arg_usize_opt(args, "expected_count");
        let snip = self
            .snippets
            .get(snippet_id)
            .cloned()
            .ok_or_else(|| ToolError::Other(EditError::NotFound.to_string()))?;
        let scopes = scopes_for_path(&self.workspace, &snip.path, PathOp::Write)
            .map_err(|e| ToolError::Other(e.to_string()))?;
        self.check(&scopes)?;
        match self.snippets.edit(snippet_id, old, new, expected) {
            Ok(_) => Ok(ToolResponse {
                ok: true,
                content: json!({"ok": true, "snippet_id": snippet_id}).to_string(),
                mutated: true,
            }),
            Err(e) => Ok(ToolResponse {
                ok: false,
                content: json!({"error": e.to_string()}).to_string(),
                mutated: false,
            }),
        }
    }

    fn write(&mut self, args: &Value) -> Result<ToolResponse, ToolError> {
        let path = arg_str(args, "path")?;
        let content = arg_str(args, "content")?;
        let full = resolve_workspace_path(&self.workspace, path);
        let scopes = scopes_for_path(&self.workspace, &full, PathOp::Write)
            .map_err(|e| ToolError::Other(e.to_string()))?;
        self.check(&scopes)?;
        match self.snippets.write_new(&full, content) {
            Ok(()) => Ok(ToolResponse {
                ok: true,
                content: json!({"ok": true, "path": path}).to_string(),
                mutated: true,
            }),
            Err(WriteError::Exists) => Ok(ToolResponse {
                ok: false,
                content: json!({"error": "path_exists_use_edit"}).to_string(),
                mutated: false,
            }),
            Err(e) => Err(ToolError::Other(e.to_string())),
        }
    }

    fn bash(&mut self, args: &Value) -> Result<ToolResponse, ToolError> {
        let command = arg_str(args, "command")?;
        let declared = parse_declared_scopes(args);
        let classified = classify_bash(command);
        let effective = effective_scopes(&declared, &classified);
        // Denied bash must not expire snippets (spec 90 test).
        match decide(&self.policy, &effective) {
            Decision::Deny | Decision::Ask => {
                return Err(ToolError::Permission(format!(
                    "bash denied/ask classified={:?} declared={:?}",
                    classified.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                    declared.iter().map(|s| s.as_str()).collect::<Vec<_>>()
                )));
            }
            Decision::Allow => {}
        }

        let may_mutate = effective.iter().any(|s| {
            matches!(
                s,
                Scope::WriteInCwd
                    | Scope::WriteOutCwd
                    | Scope::DeleteInCwd
                    | Scope::DeleteOutCwd
                    | Scope::MutateGit
                    | Scope::Unknown
            )
        });
        if may_mutate {
            // M2 default: expire all workspace snippets on mutating bash
            self.snippets.expire_all();
        }

        if !self.bash_execute {
            return Ok(ToolResponse {
                ok: true,
                content: json!({
                    "ok": true,
                    "dry_run": true,
                    "classified": classified.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                    "message": "bash execution disabled (permission/classify only)"
                })
                .to_string(),
                mutated: may_mutate,
            });
        }

        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&self.workspace)
            .output()
            .map_err(|e| ToolError::Other(e.to_string()))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(ToolResponse {
            ok: output.status.success(),
            content: json!({
                "exit_code": output.status.code(),
                "stdout": stdout,
                "stderr": stderr,
            })
            .to_string(),
            mutated: may_mutate,
        })
    }
}

fn parse_declared_scopes(args: &Value) -> Vec<Scope> {
    args.get("side_effects")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str())
                .filter_map(Scope::parse)
                .collect()
        })
        .unwrap_or_default()
}

fn arg_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, ToolError> {
    args.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::Args(format!("missing string field {key}")))
}

fn arg_usize_opt(args: &Value, key: &str) -> Option<usize> {
    args.get(key).and_then(|v| v.as_u64()).map(|n| n as usize)
}

fn path_display(workspace: &Path, full: &Path) -> String {
    full.strip_prefix(workspace)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| full.display().to_string())
}

/// OpenAI-style tool definitions for the model (stable schema — keep key order stable via serde).
pub fn tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            type_: "function".into(),
            function: ToolFunction {
                name: "read".into(),
                description: Some(
                    "Read a text file and obtain a snippet_id required for edit.".into(),
                ),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "start_line": {"type": "integer"},
                        "end_line": {"type": "integer"}
                    },
                    "required": ["path"],
                    "additionalProperties": false
                })),
            },
        },
        ToolDefinition {
            type_: "function".into(),
            function: ToolFunction {
                name: "edit".into(),
                description: Some(
                    "Edit within a snippet scope. Requires snippet_id from read. Do not guess on ambiguous matches.".into(),
                ),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "snippet_id": {"type": "string"},
                        "old_string": {"type": "string"},
                        "new_string": {"type": "string"},
                        "expected_count": {"type": "integer"}
                    },
                    "required": ["snippet_id", "old_string", "new_string"],
                    "additionalProperties": false
                })),
            },
        },
        ToolDefinition {
            type_: "function".into(),
            function: ToolFunction {
                name: "write".into(),
                description: Some(
                    "Create a new file only. Overwriting existing paths is denied; use read+edit.".into(),
                ),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "content": {"type": "string"}
                    },
                    "required": ["path", "content"],
                    "additionalProperties": false
                })),
            },
        },
        ToolDefinition {
            type_: "function".into(),
            function: ToolFunction {
                name: "bash".into(),
                description: Some(
                    "Run a shell command. Declare side_effects scopes; classifier is authoritative.".into(),
                ),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "command": {"type": "string"},
                        "side_effects": {
                            "type": "array",
                            "items": {"type": "string"}
                        }
                    },
                    "required": ["command", "side_effects"],
                    "additionalProperties": false
                })),
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::default_coding_policy;
    use std::fs;
    use tempfile::tempdir;

    fn policy_allow_write() -> PermissionPolicy {
        let mut p = default_coding_policy(true);
        p.allow.insert(Scope::WriteInCwd);
        p.ask.remove(&Scope::WriteInCwd);
        p
    }

    #[test]
    fn denied_bash_no_snippet_expiry() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        fs::write(&path, "hello\n").unwrap();
        let mut ex = ToolExecutor::new(dir.path().to_path_buf(), default_coding_policy(true));
        let (snip, _) = ex.snippets.issue_for_file(&path, None, None).unwrap();
        let req = ToolRequest {
            name: ToolName::Bash,
            arguments: json!({"command": "rm a.txt", "side_effects": ["read-in-cwd"]}),
        };
        let err = ex.execute(&req).unwrap_err();
        assert!(matches!(err, ToolError::Permission(_)));
        // snippet still present
        assert!(ex.snippets.get(&snip.snippet_id).is_some());
    }

    #[test]
    fn allowed_mutating_bash_flags_dirty() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        fs::write(&path, "hello\n").unwrap();
        let mut policy = policy_allow_write();
        policy.allow.insert(Scope::DeleteInCwd);
        policy.ask.remove(&Scope::DeleteInCwd);
        let mut ex = ToolExecutor::new(dir.path().to_path_buf(), policy);
        let (snip, _) = ex.snippets.issue_for_file(&path, None, None).unwrap();
        let req = ToolRequest {
            name: ToolName::Bash,
            arguments: json!({"command": "rm a.txt", "side_effects": ["delete-in-cwd"]}),
        };
        let resp = ex.execute(&req).unwrap();
        assert!(resp.ok);
        assert!(ex.snippets.get(&snip.snippet_id).is_none());
    }

    #[test]
    fn edit_without_snippet_id_args_error() {
        let dir = tempdir().unwrap();
        let mut ex = ToolExecutor::new(dir.path().to_path_buf(), policy_allow_write());
        let req = ToolRequest {
            name: ToolName::Edit,
            arguments: json!({"old_string": "a", "new_string": "b"}),
        };
        let err = ex.execute(&req).unwrap_err();
        assert!(matches!(err, ToolError::Args(_)));
    }
}
