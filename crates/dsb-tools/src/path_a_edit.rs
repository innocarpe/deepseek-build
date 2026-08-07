//! Path A (default Grok agent) snippet-safe edit gate — Spec 45 spirit.
//!
//! The product default entry runs `deepseek-build-agent` with Grok
//! `search_replace` (file_path / old_string / new_string / replace_all).
//! Free-form whole-file primary edit without a session snippet (or Spec 45
//! equivalent) is **not** allowed under this gate.
//!
//! Thin Path B (`dsb-tools` ToolExecutor `edit`) already uses [`SnippetStore`].
//! This module is the **contract adapter** for Grok-shaped requests so heart
//! fusion tests and future agent wiring share one fail-closed rule.
//!
//! See: `docs/architecture/HEART_3X_SPEC_BINDING.md`,
//! `docs/product/HEART_3X_P0_TEST_PLAN.md` (H45.*).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::snippets::{EditError, SnippetStore, file_version};

/// Grok `search_replace`-shaped edit request (product Path A).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrokPathEditRequest {
    pub file_path: PathBuf,
    pub old_string: String,
    pub new_string: String,
    #[serde(default)]
    pub replace_all: bool,
    /// Session snippet from a prior read (Spec 45). Required when
    /// [`PathAEditPolicy::require_snippet`] is true (product default).
    #[serde(default)]
    pub snippet_id: Option<String>,
    /// Spec 45 equivalent: full-file sha256 hex at read time (optional if
    /// `snippet_id` is set; the store version wins).
    #[serde(default)]
    pub file_version: Option<String>,
}

/// Policy for Path A edits under DeepSeek Build heart fusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathAEditPolicy {
    /// When true (product default), free-form edit without `snippet_id` is rejected.
    pub require_snippet: bool,
    /// When true, empty `old_string` cannot overwrite a non-empty existing file.
    pub reject_empty_old_overwrite: bool,
}

impl Default for PathAEditPolicy {
    fn default() -> Self {
        Self::product_default()
    }
}

impl PathAEditPolicy {
    /// Product 3.x default: snippet-safe + no free-form whole-file overwrite.
    pub const fn product_default() -> Self {
        Self {
            require_snippet: true,
            reject_empty_old_overwrite: true,
        }
    }

    /// Legacy Grok-like: allow free-form (not used as product default).
    pub const fn legacy_free_form() -> Self {
        Self {
            require_snippet: false,
            reject_empty_old_overwrite: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PathAEditError {
    #[error("free_form_primary_rejected: snippet_id required on Path A (Spec 45)")]
    FreeFormPrimaryRejected,
    #[error("empty_old_string_overwrite_rejected")]
    EmptyOldOverwriteRejected,
    #[error("file_version_mismatch")]
    FileVersionMismatch,
    #[error("snippet: {0}")]
    Snippet(#[from] EditError),
}

/// Apply a Grok-shaped edit under Path A policy using the session snippet store.
///
/// # Rules (product default)
/// 1. Non-create edits require `snippet_id` (fail closed on free-form primary).
/// 2. Empty `old_string` on existing non-empty file is rejected.
/// 3. Snippet version / optional `file_version` must match current file.
/// 4. Replacement runs only inside snippet scope via [`SnippetStore::edit`].
pub fn apply_path_a_edit(
    store: &mut SnippetStore,
    policy: PathAEditPolicy,
    req: &GrokPathEditRequest,
) -> Result<String, PathAEditError> {
    let path = req.file_path.as_path();
    let is_create = req.old_string.is_empty() && !path.exists();

    if req.old_string.is_empty() && path.exists() && policy.reject_empty_old_overwrite {
        let meta_empty = std::fs::metadata(path)
            .map(|m| m.len() == 0)
            .unwrap_or(false);
        if !meta_empty {
            return Err(PathAEditError::EmptyOldOverwriteRejected);
        }
    }

    if is_create {
        // Create-new is write_new territory; still not free-form overwrite.
        store.write_new(path, &req.new_string).map_err(|e| {
            PathAEditError::Snippet(EditError::Io(e.to_string()))
        })?;
        return Ok(req.new_string.clone());
    }

    if policy.require_snippet {
        let Some(snippet_id) = req.snippet_id.as_deref() else {
            return Err(PathAEditError::FreeFormPrimaryRejected);
        };

        if let Some(expected) = req.file_version.as_deref() {
            let current = file_version(path)?;
            if current != expected {
                return Err(PathAEditError::FileVersionMismatch);
            }
        }

        let expected_count = if req.replace_all {
            None // store treats multi without expected_count as Ambiguous — count explicitly
        } else {
            None
        };

        if req.replace_all {
            // Spec 45 default forbids ambiguous; replace_all is an explicit multi-match mode.
            // Issue: SnippetStore::edit without expected_count rejects n>1. Expand here.
            return replace_all_in_snippet(store, snippet_id, &req.old_string, &req.new_string);
        }

        let content = store.edit(snippet_id, &req.old_string, &req.new_string, expected_count)?;
        Ok(content)
    } else {
        // Legacy free-form (tests only) — still optional file_version check.
        if let Some(expected) = req.file_version.as_deref() {
            let current = file_version(path)?;
            if current != expected {
                return Err(PathAEditError::FileVersionMismatch);
            }
        }
        Err(PathAEditError::FreeFormPrimaryRejected)
    }
}

fn replace_all_in_snippet(
    store: &mut SnippetStore,
    snippet_id: &str,
    old: &str,
    new: &str,
) -> Result<String, PathAEditError> {
    let snippet = store
        .get(snippet_id)
        .cloned()
        .ok_or(EditError::NotFound)?;
    let content = std::fs::read_to_string(&snippet.path).map_err(|e| EditError::Io(e.to_string()))?;
    let current = file_version(&snippet.path)?;
    if current != snippet.version {
        return Err(PathAEditError::Snippet(EditError::Stale));
    }
    // Count matches in full file scope of snippet via store.edit with expected_count.
    let n = {
        let lines: Vec<&str> = content.split('\n').collect();
        let start = snippet.start_line.saturating_sub(1).min(lines.len());
        let end = snippet.end_line.min(lines.len());
        let scope = lines[start..end].join("\n");
        scope.matches(old).count()
    };
    if n == 0 {
        return Err(PathAEditError::Snippet(EditError::NoMatch));
    }
    let content = store.edit(snippet_id, old, new, Some(n))?;
    Ok(content)
}

/// Pure check used by vendor-facing contract tests: free-form primary must fail.
pub fn reject_free_form_primary(policy: PathAEditPolicy, has_snippet_id: bool) -> bool {
    policy.require_snippet && !has_snippet_id
}

/// Hash helper re-export for agent/read integration tests.
pub fn path_file_version(path: &Path) -> Result<String, EditError> {
    file_version(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    fn write_file(dir: &Path, name: &str, body: &str) -> PathBuf {
        let p = dir.join(name);
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        p
    }

    #[test]
    fn h45_1_free_form_without_snippet_rejected() {
        let dir = tempdir().unwrap();
        let path = write_file(dir.path(), "a.rs", "fn main() {}\n");
        let mut store = SnippetStore::new();
        let req = GrokPathEditRequest {
            file_path: path,
            old_string: "main".into(),
            new_string: "entry".into(),
            replace_all: false,
            snippet_id: None,
            file_version: None,
        };
        let err = apply_path_a_edit(&mut store, PathAEditPolicy::product_default(), &req)
            .unwrap_err();
        assert_eq!(err, PathAEditError::FreeFormPrimaryRejected);
        assert!(reject_free_form_primary(PathAEditPolicy::product_default(), false));
    }

    #[test]
    fn h45_2_empty_old_overwrite_rejected() {
        let dir = tempdir().unwrap();
        let path = write_file(dir.path(), "a.rs", "keep me\n");
        let mut store = SnippetStore::new();
        let (snip, _) = store.issue_for_file(&path, None, None).unwrap();
        let req = GrokPathEditRequest {
            file_path: path.clone(),
            old_string: String::new(),
            new_string: "wiped\n".into(),
            replace_all: false,
            snippet_id: Some(snip.snippet_id),
            file_version: None,
        };
        let err = apply_path_a_edit(&mut store, PathAEditPolicy::product_default(), &req)
            .unwrap_err();
        assert_eq!(err, PathAEditError::EmptyOldOverwriteRejected);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "keep me\n");
    }

    #[test]
    fn h45_3_snippet_edit_unique_match_ok() {
        let dir = tempdir().unwrap();
        let path = write_file(dir.path(), "a.rs", "alpha\nbeta\nalpha\n");
        let mut store = SnippetStore::new();
        // Scope only line 1 so "alpha" is unique in scope.
        let (snip, _) = store.issue_for_file(&path, Some(1), Some(1)).unwrap();
        let ver = file_version(&path).unwrap();
        let req = GrokPathEditRequest {
            file_path: path.clone(),
            old_string: "alpha".into(),
            new_string: "ALPHA".into(),
            replace_all: false,
            snippet_id: Some(snip.snippet_id),
            file_version: Some(ver),
        };
        apply_path_a_edit(&mut store, PathAEditPolicy::product_default(), &req).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "ALPHA\nbeta\nalpha\n");
    }

    #[test]
    fn h45_3_ambiguous_without_replace_all_fails() {
        let dir = tempdir().unwrap();
        let path = write_file(dir.path(), "a.rs", "alpha\nbeta\nalpha\n");
        let mut store = SnippetStore::new();
        let (snip, _) = store.issue_for_file(&path, None, None).unwrap();
        let req = GrokPathEditRequest {
            file_path: path,
            old_string: "alpha".into(),
            new_string: "ALPHA".into(),
            replace_all: false,
            snippet_id: Some(snip.snippet_id),
            file_version: None,
        };
        let err = apply_path_a_edit(&mut store, PathAEditPolicy::product_default(), &req)
            .unwrap_err();
        assert!(matches!(err, PathAEditError::Snippet(EditError::Ambiguous)));
    }

    #[test]
    fn h45_4_stale_version_fails() {
        let dir = tempdir().unwrap();
        let path = write_file(dir.path(), "a.rs", "hello\n");
        let mut store = SnippetStore::new();
        let (snip, _) = store.issue_for_file(&path, None, None).unwrap();
        std::fs::write(&path, "changed\n").unwrap();
        let req = GrokPathEditRequest {
            file_path: path,
            old_string: "hello".into(),
            new_string: "hi".into(),
            replace_all: false,
            snippet_id: Some(snip.snippet_id),
            file_version: None,
        };
        let err = apply_path_a_edit(&mut store, PathAEditPolicy::product_default(), &req)
            .unwrap_err();
        assert!(matches!(err, PathAEditError::Snippet(EditError::Stale)));
    }

    #[test]
    fn h45_4_file_version_mismatch_fails() {
        let dir = tempdir().unwrap();
        let path = write_file(dir.path(), "a.rs", "hello\n");
        let mut store = SnippetStore::new();
        let (snip, _) = store.issue_for_file(&path, None, None).unwrap();
        let req = GrokPathEditRequest {
            file_path: path,
            old_string: "hello".into(),
            new_string: "hi".into(),
            replace_all: false,
            snippet_id: Some(snip.snippet_id),
            file_version: Some("deadbeef".into()),
        };
        let err = apply_path_a_edit(&mut store, PathAEditPolicy::product_default(), &req)
            .unwrap_err();
        assert_eq!(err, PathAEditError::FileVersionMismatch);
    }

    #[test]
    fn h45_create_new_without_snippet_allowed() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("new.rs");
        let mut store = SnippetStore::new();
        let req = GrokPathEditRequest {
            file_path: path.clone(),
            old_string: String::new(),
            new_string: "fn x() {}\n".into(),
            replace_all: false,
            snippet_id: None,
            file_version: None,
        };
        apply_path_a_edit(&mut store, PathAEditPolicy::product_default(), &req).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "fn x() {}\n");
    }

    #[test]
    fn product_policy_rejects_free_form_primary_flag() {
        assert!(reject_free_form_primary(PathAEditPolicy::product_default(), false));
        assert!(!reject_free_form_primary(PathAEditPolicy::product_default(), true));
        assert!(!reject_free_form_primary(PathAEditPolicy::legacy_free_form(), false));
    }
}
