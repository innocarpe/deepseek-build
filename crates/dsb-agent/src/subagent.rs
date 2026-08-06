//! In-process subagents + worker cache law (spec 60 / G5).

use dsb_context::{
    DEFAULT_SYSTEM_PROMPT, EnvironmentSummary, PrefixBuildInputs, PrefixBuilder, StablePrefix,
    discover_project_instructions, discover_skills_index,
};
use dsb_tools::{
    PermissionPolicy, Scope, ToolExecutor, ToolName, ToolRequest, ToolResponse,
    default_coding_policy, tool_definitions,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SubagentError {
    #[error("unknown kind: {0}")]
    UnknownKind(String),
    #[error("context: {0}")]
    Context(#[from] dsb_context::PrefixError),
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerKind {
    Explore,
    Implement,
}

impl WorkerKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "explore" => Some(Self::Explore),
            "implement" => Some(Self::Implement),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Explore => "explore",
            Self::Implement => "implement",
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkerOutcome {
    pub kind: WorkerKind,
    pub summary: String,
    pub tool_rounds: u32,
    pub mutated: bool,
    pub prefix_epoch_short: String,
}

/// Build the same stable prefix the parent would for worker cache law tests.
pub fn worker_stable_prefix(
    workspace: &std::path::Path,
    user_skills_root: Option<&std::path::Path>,
    tools: Vec<dsb_provider_deepseek::ToolDefinition>,
) -> Result<StablePrefix, SubagentError> {
    let project_instructions = discover_project_instructions(workspace)?;
    let environment = EnvironmentSummary::detect(workspace);
    let skills_index = discover_skills_index(workspace, user_skills_root).unwrap_or_default();
    let inputs = PrefixBuildInputs {
        system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
        tools,
        skills_index,
        environment,
        project_instructions,
    };
    Ok(PrefixBuilder::new().build(&inputs)?)
}

fn explore_policy(headless: bool) -> PermissionPolicy {
    let mut p = default_coding_policy(headless);
    // Explore: deny all writes/deletes/bash-ish scopes.
    p.deny.insert(Scope::WriteInCwd);
    p.deny.insert(Scope::WriteOutCwd);
    p.deny.insert(Scope::DeleteInCwd);
    p.deny.insert(Scope::DeleteOutCwd);
    p.deny.insert(Scope::MutateGit);
    p.deny.insert(Scope::Network);
    p.allow.remove(&Scope::WriteInCwd);
    p.ask.clear();
    p
}

/// Run a minimal worker that executes a **single** tool-shaped task for tests / v1.
///
/// Product path: explore uses grep/read only; implement may write under policy.
pub fn run_worker(
    kind: WorkerKind,
    workspace: &std::path::Path,
    task: &str,
    parent_policy: &PermissionPolicy,
    user_skills_root: Option<&std::path::Path>,
) -> Result<WorkerOutcome, SubagentError> {
    let tools = tool_definitions();
    let stable = worker_stable_prefix(workspace, user_skills_root, tools.clone())?;
    let policy = match kind {
        WorkerKind::Explore => explore_policy(parent_policy.headless),
        WorkerKind::Implement => parent_policy.clone(),
    };
    let mut ex = ToolExecutor::new(workspace.to_path_buf(), policy);
    ex.user_skills_root = user_skills_root.map(|p| p.to_path_buf());
    if matches!(kind, WorkerKind::Implement) {
        ex.bash_execute = false; // still require explicit parent dogfood for shell
    }

    // v1 heuristic: if task looks like grep, run grep; if write/create, try write; else read path token.
    let (resp, rounds) = dispatch_heuristic(kind, &mut ex, task)?;
    Ok(WorkerOutcome {
        kind,
        summary: resp.content,
        tool_rounds: rounds,
        mutated: resp.mutated,
        prefix_epoch_short: stable.epoch.short().to_string(),
    })
}

fn dispatch_heuristic(
    kind: WorkerKind,
    ex: &mut ToolExecutor,
    task: &str,
) -> Result<(ToolResponse, u32), SubagentError> {
    let lower = task.to_ascii_lowercase();
    if lower.contains("grep:") || lower.starts_with("search ") {
        let pattern = task
            .split_once(':')
            .map(|(_, p)| p.trim())
            .unwrap_or(task)
            .trim();
        let resp = ex
            .execute(&ToolRequest {
                name: ToolName::Grep,
                arguments: json!({"pattern": pattern}),
            })
            .map_err(|e| SubagentError::Other(e.to_string()))?;
        return Ok((resp, 1));
    }
    if matches!(kind, WorkerKind::Implement)
        && (lower.contains("write:") || lower.starts_with("create "))
    {
        // write:path=... content=...
        let path = extract_kv(task, "path").unwrap_or("worker_out.txt");
        let content = extract_kv(task, "content").unwrap_or("worker-ok\n");
        let resp = ex
            .execute(&ToolRequest {
                name: ToolName::Write,
                arguments: json!({"path": path, "content": content}),
            })
            .map_err(|e| SubagentError::Other(e.to_string()))?;
        return Ok((resp, 1));
    }
    // default: grep for a distinctive token from task
    let pattern = task.chars().take(32).collect::<String>();
    let resp = ex
        .execute(&ToolRequest {
            name: ToolName::Grep,
            arguments: json!({"pattern": if pattern.is_empty() { "fn " } else { &pattern }}),
        })
        .map_err(|e| SubagentError::Other(e.to_string()))?;
    Ok((resp, 1))
}

fn extract_kv<'a>(task: &'a str, key: &str) -> Option<&'a str> {
    let marker = format!("{key}=");
    let idx = task.find(&marker)?;
    let rest = &task[idx + marker.len()..];
    let end = rest.find(' ').unwrap_or(rest.len());
    Some(rest[..end].trim())
}

/// Parent helper: expire snippets after implement worker mutates.
pub fn parent_after_worker(parent_tools: &mut ToolExecutor, outcome: &WorkerOutcome) {
    if outcome.mutated {
        parent_tools.snippets.expire_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dsb_tools::dogfood_coding_policy;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn explore_cannot_write() {
        let dir = tempdir().unwrap();
        let policy = default_coding_policy(true);
        let _ = policy;
        let mut ex = ToolExecutor::new(dir.path().to_path_buf(), explore_policy(true));
        let r = ex.execute(&ToolRequest {
            name: ToolName::Write,
            arguments: json!({"path": "x.txt", "content": "n"}),
        });
        assert!(r.is_err());
    }

    #[test]
    fn cache_law_epochs_match() {
        let dir = tempdir().unwrap();
        let tools = tool_definitions();
        let a = worker_stable_prefix(dir.path(), None, tools.clone()).unwrap();
        let b = worker_stable_prefix(dir.path(), None, tools).unwrap();
        assert_eq!(a.epoch.sha256_hex, b.epoch.sha256_hex);
    }

    #[test]
    fn implement_write_mutates() {
        let dir = tempdir().unwrap();
        let mut policy = dogfood_coding_policy(true);
        policy.allow.insert(Scope::WriteInCwd);
        policy.ask.remove(&Scope::WriteInCwd);
        let out = run_worker(
            WorkerKind::Implement,
            dir.path(),
            "write:path=out.txt content=hello-worker\n",
            &policy,
            None,
        )
        .unwrap();
        assert!(out.mutated);
        assert!(dir.path().join("out.txt").exists());
        let mut parent = ToolExecutor::new(dir.path().to_path_buf(), policy);
        let path = dir.path().join("seed.txt");
        fs::write(&path, "seed\n").unwrap();
        let (snip, _) = parent.snippets.issue_for_file(&path, None, None).unwrap();
        parent_after_worker(&mut parent, &out);
        assert!(parent.snippets.get(&snip.snippet_id).is_none());
    }
}
