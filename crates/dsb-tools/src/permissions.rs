//! Side-effect permissions — spec 90 minimum.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Scope {
    ReadInCwd,
    ReadOutCwd,
    WriteInCwd,
    WriteOutCwd,
    DeleteInCwd,
    DeleteOutCwd,
    QueryGit,
    MutateGit,
    Network,
    Unknown,
}

impl Scope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadInCwd => "read-in-cwd",
            Self::ReadOutCwd => "read-out-cwd",
            Self::WriteInCwd => "write-in-cwd",
            Self::WriteOutCwd => "write-out-cwd",
            Self::DeleteInCwd => "delete-in-cwd",
            Self::DeleteOutCwd => "delete-out-cwd",
            Self::QueryGit => "query-git",
            Self::MutateGit => "mutate-git",
            Self::Network => "network",
            Self::Unknown => "unknown",
        }
    }

    /// Higher = more dangerous (fail-closed comparison).
    pub fn danger_rank(self) -> u8 {
        match self {
            Self::ReadInCwd => 1,
            Self::QueryGit => 2,
            Self::ReadOutCwd => 3,
            Self::WriteInCwd => 5,
            Self::DeleteInCwd => 6,
            Self::Network => 7,
            Self::MutateGit => 8,
            Self::WriteOutCwd => 9,
            Self::DeleteOutCwd => 10,
            Self::Unknown => 11,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "read-in-cwd" => Some(Self::ReadInCwd),
            "read-out-cwd" => Some(Self::ReadOutCwd),
            "write-in-cwd" => Some(Self::WriteInCwd),
            "write-out-cwd" => Some(Self::WriteOutCwd),
            "delete-in-cwd" => Some(Self::DeleteInCwd),
            "delete-out-cwd" => Some(Self::DeleteOutCwd),
            "query-git" => Some(Self::QueryGit),
            "mutate-git" => Some(Self::MutateGit),
            "network" => Some(Self::Network),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    Allow,
    Deny,
    Ask,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PermissionError {
    #[error("permission denied: {0}")]
    Denied(String),
    #[error("permission requires user confirmation: {0}")]
    NeedsAsk(String),
}

#[derive(Debug, Clone)]
pub struct PermissionPolicy {
    pub allow: BTreeSet<Scope>,
    pub ask: BTreeSet<Scope>,
    pub deny: BTreeSet<Scope>,
    pub default: Decision,
    /// When true, Ask is treated as Deny (headless).
    pub headless: bool,
}

impl PermissionPolicy {
    pub fn decide(&self, scopes: &[Scope]) -> Decision {
        if scopes.is_empty() {
            return self.default;
        }
        if scopes.iter().any(|s| self.deny.contains(s) || *s == Scope::Unknown && self.deny.contains(&Scope::Unknown)) {
            // unknown handled below too
        }
        if scopes.iter().any(|s| self.deny.contains(s)) {
            return Decision::Deny;
        }
        // unknown always ask unless denied above
        if scopes.iter().any(|s| *s == Scope::Unknown) {
            return self.apply_headless(Decision::Ask);
        }
        if scopes.iter().any(|s| self.ask.contains(s)) {
            return self.apply_headless(Decision::Ask);
        }
        if scopes.iter().all(|s| self.allow.contains(s)) {
            return Decision::Allow;
        }
        self.apply_headless(self.default)
    }

    fn apply_headless(&self, d: Decision) -> Decision {
        if self.headless && d == Decision::Ask {
            Decision::Deny
        } else {
            d
        }
    }
}

/// Product default coding profile (spec 90 §1.4) — not YOLO.
pub fn default_coding_policy(headless: bool) -> PermissionPolicy {
    let mut allow = BTreeSet::new();
    allow.insert(Scope::ReadInCwd);
    allow.insert(Scope::QueryGit);

    let mut ask = BTreeSet::new();
    ask.insert(Scope::WriteInCwd);
    ask.insert(Scope::DeleteInCwd);
    ask.insert(Scope::ReadOutCwd);
    ask.insert(Scope::Network);
    ask.insert(Scope::MutateGit);

    let mut deny = BTreeSet::new();
    deny.insert(Scope::WriteOutCwd);
    deny.insert(Scope::DeleteOutCwd);

    PermissionPolicy {
        allow,
        ask,
        deny,
        default: Decision::Ask,
        headless,
    }
}

/// Trusted local dogfood profile: allow workspace write/delete + read/query-git.
/// Still **denies** write/delete outside the workspace (fail-closed).
/// Pair with CLI `--dogfood` which also enables bash execution.
pub fn dogfood_coding_policy(headless: bool) -> PermissionPolicy {
    let mut p = default_coding_policy(headless);
    p.allow.insert(Scope::WriteInCwd);
    p.allow.insert(Scope::DeleteInCwd);
    p.ask.remove(&Scope::WriteInCwd);
    p.ask.remove(&Scope::DeleteInCwd);
    // Keep out-of-cwd write/delete denied.
    p.deny.insert(Scope::WriteOutCwd);
    p.deny.insert(Scope::DeleteOutCwd);
    p
}

pub fn decide(policy: &PermissionPolicy, scopes: &[Scope]) -> Decision {
    policy.decide(scopes)
}

/// Classify path relative to workspace → read/write/delete in/out.
pub fn scopes_for_path(
    workspace: &Path,
    path: &Path,
    kind: PathOp,
) -> Result<Vec<Scope>, std::io::Error> {
    let in_cwd = is_under_workspace(workspace, path)?;
    Ok(vec![match (kind, in_cwd) {
        (PathOp::Read, true) => Scope::ReadInCwd,
        (PathOp::Read, false) => Scope::ReadOutCwd,
        (PathOp::Write, true) => Scope::WriteInCwd,
        (PathOp::Write, false) => Scope::WriteOutCwd,
        (PathOp::Delete, true) => Scope::DeleteInCwd,
        (PathOp::Delete, false) => Scope::DeleteOutCwd,
    }])
}

#[derive(Debug, Clone, Copy)]
pub enum PathOp {
    Read,
    Write,
    Delete,
}

pub fn is_under_workspace(workspace: &Path, path: &Path) -> Result<bool, std::io::Error> {
    let root = workspace
        .canonicalize()
        .unwrap_or_else(|_| workspace.to_path_buf());
    // Prefer canonicalize when the path exists; otherwise join + normalize `..` without requiring existence.
    let abs = if path.is_absolute() {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    } else {
        let joined = root.join(path);
        joined
            .canonicalize()
            .unwrap_or_else(|_| normalize_logical(&joined))
    };
    Ok(abs.starts_with(&root))
}

fn normalize_logical(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

pub fn resolve_workspace_path(workspace: &Path, path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        p
    } else {
        workspace.join(p)
    }
}

/// Fail-closed merge of declared (advisory) and classified (authoritative).
pub fn effective_scopes(declared: &[Scope], classified: &[Scope]) -> Vec<Scope> {
    let mut classified: BTreeSet<Scope> = classified.iter().copied().collect();
    if classified.is_empty() {
        classified.insert(Scope::Unknown);
    }
    // If declared is lower-risk only, ignore for decision set — classified wins.
    // If declared has higher-risk extras, still use classified for decision
    // but we keep classified as authoritative per spec.
    let mut out: Vec<Scope> = classified.into_iter().collect();
    out.sort_by_key(|s| s.danger_rank());
    let _ = declared; // audit-only at call site
    out
}

/// Deterministic bash classifier (spec 90 §1.8).
pub fn classify_bash(command: &str) -> Vec<Scope> {
    let lower = command.to_ascii_lowercase();
    let mut scopes = BTreeSet::new();

    // Network
    for tok in [
        "curl ", "curl\t", "wget ", "npm i", "npm install", "pnpm i", "yarn add", "cargo install",
        "pip install", "git clone", "http://", "https://",
    ] {
        if lower.contains(tok) {
            scopes.insert(Scope::Network);
        }
    }

    // Git
    if lower.contains("git ") {
        if contains_any(
            &lower,
            &[
                "git commit",
                "git push",
                "git rebase",
                "git reset",
                "git tag ",
                "git merge",
                "git cherry-pick",
                "git clean",
            ],
        ) {
            scopes.insert(Scope::MutateGit);
        } else if contains_any(
            &lower,
            &["git status", "git log", "git show", "git diff", "git blame", "git branch"],
        ) {
            scopes.insert(Scope::QueryGit);
        } else {
            scopes.insert(Scope::Unknown);
        }
    }

    // Delete / write heuristics
    if contains_any(&lower, &["rm ", "rm\t", "rmdir ", "unlink "]) {
        scopes.insert(Scope::DeleteInCwd);
    }
    if contains_any(&lower, &[" >", ">>", " tee ", "mv ", "cp ", "touch ", "chmod ", "mkdir "]) {
        scopes.insert(Scope::WriteInCwd);
    }
    if lower.contains("sudo ") {
        scopes.insert(Scope::Unknown);
    }

    if scopes.is_empty() {
        // simple echo/true → no scopes → treat as unknown? Spec: unrecognized → unknown
        // pure read-ish like `ls` / `cat` — allow as read-in-cwd for common safe tools
        if contains_any(&lower, &["ls ", "ls\t", "cat ", "head ", "tail ", "pwd", "echo ", "true", "false"])
            || lower.trim() == "ls"
            || lower.trim() == "pwd"
        {
            scopes.insert(Scope::ReadInCwd);
        } else {
            scopes.insert(Scope::Unknown);
        }
    }

    scopes.into_iter().collect()
}

fn contains_any(hay: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| hay.contains(n))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn path_read_in_cwd_allow() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.rs");
        std::fs::write(&path, "fn main() {}\n").unwrap();
        let policy = default_coding_policy(true);
        let scopes = scopes_for_path(dir.path(), &path, PathOp::Read).unwrap();
        assert_eq!(scopes, vec![Scope::ReadInCwd]);
        assert_eq!(decide(&policy, &scopes), Decision::Allow);
    }

    #[test]
    fn path_write_out_cwd_deny() {
        let dir = tempdir().unwrap();
        let policy = default_coding_policy(true);
        let outside = PathBuf::from("/tmp/dsb-out-of-workspace-test-file");
        let scopes = scopes_for_path(dir.path(), &outside, PathOp::Write).unwrap();
        assert_eq!(scopes, vec![Scope::WriteOutCwd]);
        assert_eq!(decide(&policy, &scopes), Decision::Deny);
    }

    #[test]
    fn bash_rm_classified_write_or_delete() {
        let scopes = classify_bash("rm src/main.rs");
        assert!(scopes.contains(&Scope::DeleteInCwd));
    }

    #[test]
    fn bash_declare_lower_than_class_fail_closed() {
        let declared = vec![Scope::ReadInCwd];
        let classified = classify_bash("rm -rf build");
        let eff = effective_scopes(&declared, &classified);
        assert!(eff.iter().any(|s| s.danger_rank() >= Scope::DeleteInCwd.danger_rank()));
    }

    #[test]
    fn dogfood_allows_write_in_cwd_denies_out() {
        let p = dogfood_coding_policy(true);
        assert_eq!(decide(&p, &[Scope::WriteInCwd]), Decision::Allow);
        assert_eq!(decide(&p, &[Scope::DeleteInCwd]), Decision::Allow);
        assert_eq!(decide(&p, &[Scope::WriteOutCwd]), Decision::Deny);
        assert_eq!(decide(&p, &[Scope::DeleteOutCwd]), Decision::Deny);
    }

    #[test]
    fn bash_unknown_not_silent_allow() {
        let scopes = classify_bash("totally-unknown-bin --xyz");
        assert!(scopes.contains(&Scope::Unknown));
        let policy = default_coding_policy(true);
        assert_eq!(decide(&policy, &scopes), Decision::Deny); // headless ask→deny
    }

    #[test]
    fn policy_deny_beats_allow() {
        let mut p = default_coding_policy(false);
        p.allow.insert(Scope::WriteOutCwd);
        // still in deny set
        assert_eq!(decide(&p, &[Scope::WriteOutCwd]), Decision::Deny);
    }

    #[test]
    fn headless_ask_is_deny() {
        let p = default_coding_policy(true);
        assert_eq!(decide(&p, &[Scope::WriteInCwd]), Decision::Deny);
        let interactive = default_coding_policy(false);
        assert_eq!(decide(&interactive, &[Scope::WriteInCwd]), Decision::Ask);
    }
}
