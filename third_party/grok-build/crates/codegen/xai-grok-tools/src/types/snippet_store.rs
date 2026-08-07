//! Session-local Spec 45 snippet table for Path A (Grok tool path).
//!
//! Hosted on per-session [`super::resources::Resources`] via `get_or_default`
//! and **not** registered for Spec 10 stable-prefix / resources persistence.
//! Each agent session's `SharedResources` owns its own table — not process-global.
//!
//! VC003: issue on successful text `read_file` only. Edit require (VC004),
//! write/bash invalidation (VC005), and resume/fork restore (VC006) are out of scope.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use uuid::Uuid;

/// Opaque session snippet record (ADR 0010 §2 shape; Path A host).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSnippet {
    pub snippet_id: String,
    pub path: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    /// Full-file `hex(sha256(bytes))` at issue time (alias of tool `file_version`).
    pub version: String,
    /// `lines` or `whole_file`.
    pub scope: String,
    pub preview: String,
    pub encoding: String,
    /// Audit-only issuance counter within this store (not Spec 10 / cache).
    pub issued_at_turn: u64,
}

/// Session-owned snippet table. Ephemeral; discarded with the session Resources.
#[derive(Debug, Default)]
pub struct SessionSnippetStore {
    by_id: HashMap<String, SessionSnippet>,
    /// Monotonic issuance counter used as `issued_at_turn` until a real turn
    /// index is plumbed (audit only).
    mint_seq: u64,
}

/// Preview cap matching thin `dsb-tools` / ADR 0010 (Unicode scalars + ellipsis).
pub const SNIPPET_PREVIEW_MAX_SCALARS: usize = 200;

impl SessionSnippetStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, id: &str) -> Option<&SessionSnippet> {
        self.by_id.get(id)
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.by_id.contains_key(id)
    }

    /// Mint a new opaque `snippet_id` for a successful text read.
    ///
    /// Repeated calls always insert a **new** id (ADR 0010 §2 Multiple IDs).
    pub fn issue(
        &mut self,
        path: &Path,
        start_line: usize,
        end_line: usize,
        version: impl Into<String>,
        scope_text: &str,
        total_lines: usize,
    ) -> SessionSnippet {
        let start_line = start_line.max(1);
        let end_line = end_line.max(start_line);
        let version = version.into();
        let whole_file = start_line == 1 && end_line >= total_lines.max(1);
        let scope = if whole_file {
            "whole_file".to_string()
        } else {
            "lines".to_string()
        };
        self.mint_seq = self.mint_seq.saturating_add(1);
        let snippet = SessionSnippet {
            snippet_id: new_snippet_id(),
            path: path.to_path_buf(),
            start_line,
            end_line,
            version,
            scope,
            preview: truncate_preview(scope_text, SNIPPET_PREVIEW_MAX_SCALARS),
            encoding: "utf-8".to_string(),
            issued_at_turn: self.mint_seq,
        };
        self.by_id
            .insert(snippet.snippet_id.clone(), snippet.clone());
        snippet
    }
}

/// `snp_` + opaque unique id (UUID v7, no hyphens). Matches ADR opacity class
/// (`snp_<ulid>` spirit); Path A uses workspace-available uuid v7.
pub fn new_snippet_id() -> String {
    format!("snp_{}", Uuid::now_v7().simple())
}

fn truncate_preview(text: &str, max_scalars: usize) -> String {
    let mut out = String::new();
    let mut count = 0usize;
    for ch in text.chars() {
        if count >= max_scalars {
            out.push('…');
            break;
        }
        out.push(ch);
        count += 1;
    }
    out
}

/// Inclusive 1-based line range covered by a read window.
pub fn snippet_line_range(
    total_lines: usize,
    offset: Option<usize>,
    limit: Option<usize>,
) -> (usize, usize) {
    if total_lines == 0 {
        return (1, 1);
    }
    let start = offset.unwrap_or(1).max(1);
    let start = start.min(total_lines);
    let end = match limit {
        Some(lim) if lim > 0 => start.saturating_add(lim).saturating_sub(1).min(total_lines),
        _ => total_lines,
    };
    let end = end.max(start);
    (start, end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn issue_mints_snp_prefix_and_stores() {
        let mut store = SessionSnippetStore::new();
        let snip = store.issue(Path::new("/tmp/a.txt"), 1, 2, "ab".repeat(32), "hello\n", 2);
        assert!(snip.snippet_id.starts_with("snp_"));
        assert_eq!(snip.scope, "whole_file");
        assert_eq!(store.len(), 1);
        assert!(store.contains(&snip.snippet_id));
    }

    #[test]
    fn repeated_issue_distinct_ids() {
        let mut store = SessionSnippetStore::new();
        let a = store.issue(Path::new("/tmp/a.txt"), 1, 1, "v1", "x", 1);
        let b = store.issue(Path::new("/tmp/a.txt"), 1, 1, "v1", "x", 1);
        assert_ne!(a.snippet_id, b.snippet_id);
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn stores_are_independent_not_process_global() {
        let mut a = SessionSnippetStore::new();
        let b = SessionSnippetStore::new();
        let snip = a.issue(Path::new("/tmp/a.txt"), 1, 1, "v", "x", 1);
        assert!(a.contains(&snip.snippet_id));
        assert!(!b.contains(&snip.snippet_id));
        assert!(b.is_empty());
    }

    #[test]
    fn line_range_empty_file() {
        assert_eq!(snippet_line_range(0, None, None), (1, 1));
    }

    #[test]
    fn line_range_window() {
        assert_eq!(snippet_line_range(10, Some(3), Some(2)), (3, 4));
        assert_eq!(snippet_line_range(10, None, None), (1, 10));
    }
}
