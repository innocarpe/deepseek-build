//! Built-in tools: read / edit / write / grep / skill / bash (spec **40** + 45/90/70).

use std::fs;
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

/// Canonical built-in tool names in **stable prefix order** (spec 40 §3.3).
///
/// Do not reorder without starting a new cache epoch (spec 10).
pub const CORE_TOOL_NAMES: &[&str] = &["read", "edit", "write", "grep", "skill", "bash"];

/// Returns [`CORE_TOOL_NAMES`] (stable order).
pub fn core_tool_names() -> &'static [&'static str] {
    CORE_TOOL_NAMES
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolName {
    Read,
    Edit,
    Write,
    Grep,
    /// On-demand skill body load (not in stable prefix).
    Skill,
    Bash,
}

impl ToolName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Edit => "edit",
            Self::Write => "write",
            Self::Grep => "grep",
            Self::Skill => "skill",
            Self::Bash => "bash",
        }
    }

    /// Parse a model-supplied tool name. Accepts aliases from spec 40 §1.1.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "read" => Some(Self::Read),
            "edit" => Some(Self::Edit),
            "write" => Some(Self::Write),
            "grep" | "search" => Some(Self::Grep),
            "skill" | "load_skill" => Some(Self::Skill),
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
    /// Optional user skills root (`~/.deepseek-build/skills`).
    pub user_skills_root: Option<PathBuf>,
}

impl ToolExecutor {
    pub fn new(workspace: PathBuf, policy: PermissionPolicy) -> Self {
        Self {
            workspace,
            policy,
            snippets: SnippetStore::new(),
            bash_execute: false,
            user_skills_root: None,
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
            ToolName::Grep => self.grep(&req.arguments),
            ToolName::Skill => self.skill(&req.arguments),
            ToolName::Bash => self.bash(&req.arguments),
        }
    }

    fn skill(&mut self, args: &Value) -> Result<ToolResponse, ToolError> {
        let name = arg_str(args, "name")?;
        // skill body load is a read of trusted skill paths only (no out-of-cwd free path)
        match dsb_context::load_skill_body(
            &self.workspace,
            self.user_skills_root.as_deref(),
            name,
        ) {
            Ok(body) => Ok(ToolResponse {
                ok: true,
                content: json!({
                    "name": name,
                    "body": body,
                    "note": "skill body loaded on-demand; stable skills index unchanged"
                })
                .to_string(),
                mutated: false,
            }),
            Err(e) => Ok(ToolResponse {
                ok: false,
                content: json!({"error": e.to_string(), "name": name}).to_string(),
                mutated: false,
            }),
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

    fn grep(&mut self, args: &Value) -> Result<ToolResponse, ToolError> {
        let pattern = arg_str(args, "pattern")?;
        if pattern.is_empty() {
            return Err(ToolError::Args("pattern must be non-empty".into()));
        }
        let path_arg = args
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");
        let case_insensitive = args
            .get("case_insensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let max_matches = args
            .get("max_matches")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize)
            .unwrap_or(50)
            .clamp(1, 500);
        let glob_ext = args
            .get("glob")
            .and_then(|v| v.as_str())
            .map(|g| g.trim_start_matches("*.").to_string());

        let root = resolve_workspace_path(&self.workspace, path_arg);
        let scopes = scopes_for_path(&self.workspace, &root, PathOp::Read)
            .map_err(|e| ToolError::Other(e.to_string()))?;
        self.check(&scopes)?;

        if !root.exists() {
            return Ok(ToolResponse {
                ok: false,
                content: json!({"error": "path_not_found", "path": path_arg}).to_string(),
                mutated: false,
            });
        }

        let needle = if case_insensitive {
            pattern.to_ascii_lowercase()
        } else {
            pattern.to_string()
        };

        let mut matches: Vec<Value> = Vec::new();
        let mut truncated = false;
        let mut files_scanned = 0usize;
        let workspace = self.workspace.clone();

        walk_text_files(&root, glob_ext.as_deref(), &mut |file| {
            if matches.len() >= max_matches {
                truncated = true;
                return;
            }
            files_scanned += 1;
            let Ok(text) = fs::read_to_string(file) else {
                return;
            };
            // Skip likely-binary / huge blobs
            if text.chars().take(4096).any(|c| c == '\0') {
                return;
            }
            for (idx, line) in text.lines().enumerate() {
                if matches.len() >= max_matches {
                    truncated = true;
                    break;
                }
                let hay = if case_insensitive {
                    line.to_ascii_lowercase()
                } else {
                    line.to_string()
                };
                if hay.contains(&needle) {
                    matches.push(json!({
                        "path": path_display(&workspace, file),
                        "line": idx + 1,
                        "text": line.chars().take(400).collect::<String>(),
                    }));
                }
            }
        });

        Ok(ToolResponse {
            ok: true,
            content: json!({
                "pattern": pattern,
                "path": path_arg,
                "match_count": matches.len(),
                "files_scanned": files_scanned,
                "truncated": truncated,
                "matches": matches,
            })
            .to_string(),
            mutated: false,
        })
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
                    "message": "bash execution disabled (permission/classify only); use --dogfood or --bash-execute"
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
                "dry_run": false,
            })
            .to_string(),
            mutated: may_mutate,
        })
    }
}

/// Recursively visit text-ish files under `root`. Optional extension filter: `"rs"` matches `*.rs`.
fn walk_text_files(root: &Path, ext_filter: Option<&str>, visit: &mut dyn FnMut(&Path)) {
    const SKIP_DIRS: &[&str] = &[
        ".git",
        "target",
        "node_modules",
        ".deepseek-build",
        "dist",
        "build",
    ];
    if root.is_file() {
        if ext_ok(root, ext_filter) {
            visit(root);
        }
        return;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            if SKIP_DIRS.contains(&name) {
                continue;
            }
            walk_text_files(&path, ext_filter, visit);
        } else if path.is_file() && ext_ok(&path, ext_filter) {
            visit(&path);
        }
    }
}

fn ext_ok(path: &Path, ext_filter: Option<&str>) -> bool {
    match ext_filter {
        None => true,
        Some(ext) => path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case(ext)),
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

/// OpenAI-style tool definitions for the model (spec 40).
///
/// Order matches [`CORE_TOOL_NAMES`] and is part of the stable prefix (spec 10).
/// Schemas use canonical names only; aliases are parse-only ([`ToolName::parse`]).
pub fn tool_definitions() -> Vec<ToolDefinition> {
    let defs = vec![
        ToolDefinition {
            type_: "function".into(),
            function: ToolFunction {
                name: "read".into(),
                description: Some(
                    "Read a text file and obtain a snippet_id required for edit (spec 45).".into(),
                ),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "start_line": {"type": "integer", "description": "1-based inclusive start"},
                        "end_line": {"type": "integer", "description": "1-based inclusive end"}
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
                name: "grep".into(),
                description: Some(
                    "Search workspace text files for a literal substring (not regex). Prefer over bash grep. Optional path and file extension filter (e.g. \"rs\").".into(),
                ),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "pattern": {"type": "string"},
                        "path": {"type": "string", "description": "File or directory under workspace (default \".\")"},
                        "glob": {"type": "string", "description": "File extension filter without star, e.g. \"rs\" or \"md\""},
                        "case_insensitive": {"type": "boolean"},
                        "max_matches": {"type": "integer", "description": "Default 50, clamp 1..=500"}
                    },
                    "required": ["pattern"],
                    "additionalProperties": false
                })),
            },
        },
        ToolDefinition {
            type_: "function".into(),
            function: ToolFunction {
                name: "skill".into(),
                description: Some(
                    "Load the full body of a skill by name from the skills index (on-demand). Does not modify the stable prefix. Call when you need detailed skill instructions.".into(),
                ),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string", "description": "Skill directory name from the skills index"}
                    },
                    "required": ["name"],
                    "additionalProperties": false
                })),
            },
        },
        ToolDefinition {
            type_: "function".into(),
            function: ToolFunction {
                name: "bash".into(),
                description: Some(
                    "Run a shell command. Declare side_effects scopes; classifier is authoritative (spec 90). Execution requires --bash-execute or --dogfood.".into(),
                ),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "command": {"type": "string"},
                        "side_effects": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Advisory scope strings; classifier is authoritative"
                        }
                    },
                    "required": ["command", "side_effects"],
                    "additionalProperties": false
                })),
            },
        },
    ];
    debug_assert_eq!(
        defs.iter()
            .map(|d| d.function.name.as_str())
            .collect::<Vec<_>>(),
        CORE_TOOL_NAMES.to_vec()
    );
    defs
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

    #[test]
    fn grep_finds_literal_matches() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/a.rs"), "fn hello() {}\nfn other() {}\n").unwrap();
        fs::write(dir.path().join("src/b.txt"), "hello world\n").unwrap();
        let mut ex = ToolExecutor::new(dir.path().to_path_buf(), default_coding_policy(true));
        let req = ToolRequest {
            name: ToolName::Grep,
            arguments: json!({"pattern": "hello", "glob": "rs"}),
        };
        let resp = ex.execute(&req).unwrap();
        assert!(resp.ok);
        let v: Value = serde_json::from_str(&resp.content).unwrap();
        assert_eq!(v["match_count"], 1);
        assert!(v["matches"][0]["path"].as_str().unwrap().contains("a.rs"));
        assert_eq!(v["matches"][0]["line"], 1);
    }

    #[test]
    fn grep_empty_pattern_args_error() {
        let dir = tempdir().unwrap();
        let mut ex = ToolExecutor::new(dir.path().to_path_buf(), default_coding_policy(true));
        let req = ToolRequest {
            name: ToolName::Grep,
            arguments: json!({"pattern": ""}),
        };
        let err = ex.execute(&req).unwrap_err();
        assert!(matches!(err, ToolError::Args(_)));
    }

    #[test]
    fn bash_execute_runs_when_enabled() {
        let dir = tempdir().unwrap();
        let mut ex = ToolExecutor::new(dir.path().to_path_buf(), default_coding_policy(true));
        ex.bash_execute = true;
        let req = ToolRequest {
            name: ToolName::Bash,
            arguments: json!({"command": "echo dogfood-ok", "side_effects": ["read-in-cwd"]}),
        };
        let resp = ex.execute(&req).unwrap();
        assert!(resp.ok);
        let v: Value = serde_json::from_str(&resp.content).unwrap();
        assert_eq!(v["dry_run"], false);
        assert!(v["stdout"].as_str().unwrap().contains("dogfood-ok"));
    }

    #[test]
    fn bash_dry_run_when_execute_disabled() {
        let dir = tempdir().unwrap();
        let mut ex = ToolExecutor::new(dir.path().to_path_buf(), default_coding_policy(true));
        assert!(!ex.bash_execute);
        let req = ToolRequest {
            name: ToolName::Bash,
            arguments: json!({"command": "echo no", "side_effects": ["read-in-cwd"]}),
        };
        let resp = ex.execute(&req).unwrap();
        let v: Value = serde_json::from_str(&resp.content).unwrap();
        assert_eq!(v["dry_run"], true);
    }

    #[test]
    fn write_out_of_cwd_denied_under_write_allow() {
        let dir = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let mut ex = ToolExecutor::new(dir.path().to_path_buf(), policy_allow_write());
        let outside_path = outside.path().join("escape.txt");
        let req = ToolRequest {
            name: ToolName::Write,
            arguments: json!({
                "path": outside_path.to_string_lossy(),
                "content": "nope"
            }),
        };
        let err = ex.execute(&req).unwrap_err();
        assert!(matches!(err, ToolError::Permission(_)));
        assert!(!outside_path.exists());
    }

    #[test]
    fn search_alias_parses_as_grep() {
        assert_eq!(ToolName::parse("search"), Some(ToolName::Grep));
        assert_eq!(ToolName::parse("grep"), Some(ToolName::Grep));
    }

    #[test]
    fn skill_alias_parses_as_skill() {
        assert_eq!(ToolName::parse("skill"), Some(ToolName::Skill));
        assert_eq!(ToolName::parse("load_skill"), Some(ToolName::Skill));
        assert_eq!(ToolName::parse("unknown_tool_xyz"), None);
    }

    #[test]
    fn skill_tool_loads_body_on_demand() {
        let dir = tempdir().unwrap();
        let skill_dir = dir.path().join("skills").join("demo");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# Demo\n\nHELLO_SKILL\n").unwrap();
        let mut ex = ToolExecutor::new(dir.path().to_path_buf(), default_coding_policy(true));
        let req = ToolRequest {
            name: ToolName::Skill,
            arguments: json!({"name": "demo"}),
        };
        let resp = ex.execute(&req).unwrap();
        assert!(resp.ok);
        assert!(resp.content.contains("HELLO_SKILL"));
        assert!(!resp.mutated);
    }

    /// Spec 40 T1: registry name set is exactly the six canonical tools in stable order.
    #[test]
    fn registry_names_match_core_catalog() {
        let defs = tool_definitions();
        let names: Vec<&str> = defs.iter().map(|d| d.function.name.as_str()).collect();
        assert_eq!(names, CORE_TOOL_NAMES);
        assert_eq!(core_tool_names(), CORE_TOOL_NAMES);
        assert_eq!(names.len(), 6);
    }

    /// Spec 40 T3: required argument sets match the wire table.
    #[test]
    fn registry_required_fields_match_spec40() {
        let expected: &[(&str, &[&str])] = &[
            ("read", &["path"]),
            ("edit", &["snippet_id", "old_string", "new_string"]),
            ("write", &["path", "content"]),
            ("grep", &["pattern"]),
            ("skill", &["name"]),
            ("bash", &["command", "side_effects"]),
        ];
        let defs = tool_definitions();
        for (name, reqs) in expected {
            let def = defs
                .iter()
                .find(|d| d.function.name == *name)
                .unwrap_or_else(|| panic!("missing tool {name}"));
            let params = def.function.parameters.as_ref().expect("parameters");
            assert_eq!(params["type"], "object");
            assert_eq!(params["additionalProperties"], false);
            let required: Vec<&str> = params["required"]
                .as_array()
                .expect("required array")
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect();
            assert_eq!(&required, reqs, "required mismatch for {name}");
        }
    }

    /// Spec 40 T11: tool schema JSON is byte-stable across consecutive builds.
    #[test]
    fn registry_schema_json_is_byte_stable() {
        let a = serde_json::to_vec(&tool_definitions()).unwrap();
        let b = serde_json::to_vec(&tool_definitions()).unwrap();
        assert_eq!(a, b);
        // Pin schema size band so accidental schema explosion is noticed.
        assert!(a.len() > 400, "schema unexpectedly small: {}", a.len());
        assert!(a.len() < 16_384, "schema unexpectedly large: {}", a.len());
    }

    #[test]
    fn tool_name_as_str_roundtrips_core_catalog() {
        for name in CORE_TOOL_NAMES {
            let tn = ToolName::parse(name).expect(name);
            assert_eq!(tn.as_str(), *name);
        }
    }
}
