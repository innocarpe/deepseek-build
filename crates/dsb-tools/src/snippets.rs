//! Session snippet store — spec 45.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Snippet {
    pub snippet_id: String,
    pub path: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub version: String,
    pub scope: String,
    pub preview: String,
    /// Spec 45 §1.9: the newline convention `edit` restores when writing this file.
    #[serde(default)]
    pub line_ending: LineEnding,
}

/// The newline convention a file is written back with (spec 45 §1.9).
///
/// Defaults to `Lf`, which is also how a file with no line break and a
/// pre-§1.9 serialized snippet (no `line_ending` field) are read.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LineEnding {
    /// `\n` — including a file that contains no line break at all.
    #[default]
    Lf,
    /// `\r\n`, chosen when the file bytes contain at least one.
    Crlf,
}

impl LineEnding {
    /// `crlf` iff the raw bytes contain at least one `\r\n` (spec 45 §1.9 step 1).
    pub fn detect(raw: &str) -> Self {
        if raw.contains("\r\n") {
            Self::Crlf
        } else {
            Self::Lf
        }
    }

    /// Text as the model should see it: `\n` only (spec 45 §1.9 step 2).
    ///
    /// Borrows when there is nothing to change, so an LF file costs no copy.
    pub fn normalize<'a>(self, text: &'a str) -> std::borrow::Cow<'a, str> {
        if text.contains('\r') {
            std::borrow::Cow::Owned(text.replace("\r\n", "\n").replace('\r', "\n"))
        } else {
            std::borrow::Cow::Borrowed(text)
        }
    }

    /// Text as this file stores it: every `\n` becomes the file's convention
    /// (spec 45 §1.9 step 4).
    ///
    /// Normalizes first, so a `new_string` that arrived with `\r\n` cannot
    /// double the `\r`.
    pub fn restore(self, text: &str) -> String {
        match self {
            Self::Lf => self.normalize(text).into_owned(),
            Self::Crlf => self.normalize(text).replace('\n', "\r\n"),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "lf",
            Self::Crlf => "crlf",
        }
    }
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum EditError {
    #[error("snippet_not_found")]
    NotFound,
    #[error("snippet_stale")]
    Stale,
    #[error("no_match")]
    NoMatch,
    #[error("ambiguous_match")]
    Ambiguous,
    #[error("expected_count_mismatch")]
    CountMismatch,
    #[error("empty_old_string")]
    EmptyOld,
    #[error("io: {0}")]
    Io(String),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WriteError {
    #[error("path_exists_use_edit")]
    Exists,
    #[error("io: {0}")]
    Io(String),
}

#[derive(Debug, Default)]
pub struct SnippetStore {
    by_id: HashMap<String, Snippet>,
}

impl SnippetStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, id: &str) -> Option<&Snippet> {
        self.by_id.get(id)
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Expire all snippets for a path.
    pub fn expire_path(&mut self, path: &Path) {
        self.by_id.retain(|_, s| s.path != path);
    }

    /// Expire all snippets (e.g. unknown bash mutation).
    pub fn expire_all(&mut self) {
        self.by_id.clear();
    }

    pub fn issue_for_file(
        &mut self,
        path: &Path,
        start_line: Option<usize>,
        end_line: Option<usize>,
    ) -> Result<(Snippet, String), EditError> {
        let raw = fs::read_to_string(path).map_err(|e| EditError::Io(e.to_string()))?;
        let version = file_version(path)?;
        // Spec 45 §1.9: the model sees one newline convention; the file keeps its own.
        let line_ending = LineEnding::detect(&raw);
        let content = line_ending.normalize(&raw).into_owned();
        let lines: Vec<&str> = content.split('\n').collect();
        let start = start_line.unwrap_or(1).max(1);
        let end = end_line.unwrap_or(lines.len().max(1)).max(start);
        let end = end.min(lines.len().max(1));
        let preview = lines
            .get(start.saturating_sub(1)..end.min(lines.len()))
            .map(|s| s.join("\n"))
            .unwrap_or_default();
        let preview = truncate(&preview, 200);
        let snippet = Snippet {
            snippet_id: format!("snp_{}", Ulid::new()),
            path: path.to_path_buf(),
            start_line: start,
            end_line: end,
            version,
            scope: if start == 1 && end >= lines.len().max(1) {
                "whole_file".into()
            } else {
                "lines".into()
            },
            preview,
            line_ending,
        };
        self.by_id
            .insert(snippet.snippet_id.clone(), snippet.clone());
        Ok((snippet, content))
    }

    pub fn edit(
        &mut self,
        snippet_id: &str,
        old_string: &str,
        new_string: &str,
        expected_count: Option<usize>,
    ) -> Result<String, EditError> {
        if old_string.is_empty() {
            return Err(EditError::EmptyOld);
        }
        let snippet = self
            .by_id
            .get(snippet_id)
            .cloned()
            .ok_or(EditError::NotFound)?;
        let current_version = file_version(&snippet.path)?;
        if current_version != snippet.version {
            return Err(EditError::Stale);
        }
        let content =
            fs::read_to_string(&snippet.path).map_err(|e| EditError::Io(e.to_string()))?;
        // Spec 45 §1.9: match on LF-normalized text, write back in the file's convention.
        let content = snippet.line_ending.normalize(&content).into_owned();
        let old_string = snippet.line_ending.normalize(old_string).into_owned();
        let (before, scope, after) = split_scope(&content, snippet.start_line, snippet.end_line);
        let count = scope.matches(old_string.as_str()).count();
        match (count, expected_count) {
            (0, _) => return Err(EditError::NoMatch),
            (1, _) => {}
            (n, Some(exp)) if n == exp => {}
            (n, Some(_)) if n > 1 => return Err(EditError::CountMismatch),
            (n, None) if n > 1 => return Err(EditError::Ambiguous),
            _ => return Err(EditError::NoMatch),
        }
        let new_scope = if let Some(exp) = expected_count {
            // replace exactly exp times left-to-right
            replace_n(&scope, old_string.as_str(), new_string, exp)
        } else {
            scope.replacen(old_string.as_str(), new_string, 1)
        };
        let new_content = format!("{before}{new_scope}{after}");
        // Model text is LF; the file gets its own convention back (step 4).
        let new_content = snippet.line_ending.restore(&new_content);
        atomic_write(&snippet.path, &new_content)?;
        self.expire_path(&snippet.path);
        Ok(new_content)
    }

    pub fn write_new(&mut self, path: &Path, content: &str) -> Result<(), WriteError> {
        if path.exists() {
            return Err(WriteError::Exists);
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| WriteError::Io(e.to_string()))?;
        }
        atomic_write(path, content).map_err(|e| WriteError::Io(e.to_string()))?;
        Ok(())
    }
}

pub fn file_version(path: &Path) -> Result<String, EditError> {
    let bytes = fs::read(path).map_err(|e| EditError::Io(e.to_string()))?;
    let mut h = Sha256::new();
    h.update(&bytes);
    Ok(hex::encode(h.finalize()))
}

fn split_scope(content: &str, start_line: usize, end_line: usize) -> (String, String, String) {
    // Preserve whether file ended with newline by working on lines without forcing trailing nl
    let ends_with_nl = content.ends_with('\n');
    let lines: Vec<&str> = content.split('\n').collect();
    // If content ends with \n, split yields trailing empty; keep structure via rejoin
    let n = if ends_with_nl && lines.last() == Some(&"") {
        lines.len() - 1
    } else {
        lines.len()
    };
    let start = start_line.saturating_sub(1).min(n);
    let end = end_line.min(n);
    let before = if start == 0 {
        String::new()
    } else {
        let mut s = lines[..start].join("\n");
        s.push('\n');
        s
    };
    let scope = lines[start..end].join("\n");
    let after = if end >= n {
        if ends_with_nl && !scope.is_empty() {
            // file had trailing newline after last line in scope
            String::new()
        } else if ends_with_nl && scope.is_empty() {
            String::new()
        } else {
            String::new()
        }
    } else {
        let mut s = String::from("\n");
        s.push_str(&lines[end..n].join("\n"));
        if ends_with_nl {
            s.push('\n');
        }
        s
    };
    // Fix trailing newline for whole-file case
    let after = if end >= n && ends_with_nl {
        "\n".to_string()
    } else {
        after
    };
    (before, scope, after)
}

fn replace_n(hay: &str, old: &str, new: &str, n: usize) -> String {
    let mut out = String::new();
    let mut rest = hay;
    for _ in 0..n {
        if let Some(i) = rest.find(old) {
            out.push_str(&rest[..i]);
            out.push_str(new);
            rest = &rest[i + old.len()..];
        }
    }
    out.push_str(rest);
    out
}

fn atomic_write(path: &Path, content: &str) -> Result<(), EditError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let tmp = parent.join(format!(".dsb-tmp-{}", Ulid::new()));
    fs::write(&tmp, content).map_err(|e| EditError::Io(e.to_string()))?;
    fs::rename(&tmp, path).map_err(|e| EditError::Io(e.to_string()))?;
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect::<String>() + "…"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    fn write_file(path: &Path, s: &str) {
        let mut f = fs::File::create(path).unwrap();
        f.write_all(s.as_bytes()).unwrap();
    }

    #[test]
    fn read_returns_snippet_id() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        write_file(&path, "hello\nworld\n");
        let mut store = SnippetStore::new();
        let (snip, content) = store.issue_for_file(&path, None, None).unwrap();
        assert!(snip.snippet_id.starts_with("snp_"));
        assert!(!snip.version.is_empty());
        assert!(content.contains("hello"));
    }

    #[test]
    fn edit_requires_snippet_id() {
        let mut store = SnippetStore::new();
        let err = store.edit("missing", "a", "b", None).unwrap_err();
        assert_eq!(err, EditError::NotFound);
    }

    #[test]
    fn edit_applies_within_scope() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        write_file(&path, "aaa\nbbb\naaa\n");
        let mut store = SnippetStore::new();
        // scope only line 1
        let (snip, _) = store.issue_for_file(&path, Some(1), Some(1)).unwrap();
        store.edit(&snip.snippet_id, "aaa", "XXX", None).unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert!(after.starts_with("XXX\n"));
        assert!(after.contains("bbb"));
        // second aaa untouched
        assert!(after.lines().filter(|l| *l == "aaa").count() >= 1);
    }

    #[test]
    fn edit_stale_version_rejected() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        write_file(&path, "hello\n");
        let mut store = SnippetStore::new();
        let (snip, _) = store.issue_for_file(&path, None, None).unwrap();
        write_file(&path, "changed\n");
        let err = store
            .edit(&snip.snippet_id, "hello", "x", None)
            .unwrap_err();
        assert_eq!(err, EditError::Stale);
        assert_eq!(fs::read_to_string(&path).unwrap(), "changed\n");
    }

    #[test]
    fn edit_ambiguous_no_guess() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        write_file(&path, "xx\nxx\n");
        let mut store = SnippetStore::new();
        let (snip, _) = store.issue_for_file(&path, None, None).unwrap();
        let err = store.edit(&snip.snippet_id, "xx", "yy", None).unwrap_err();
        assert_eq!(err, EditError::Ambiguous);
        assert_eq!(fs::read_to_string(&path).unwrap(), "xx\nxx\n");
    }

    #[test]
    fn edit_expected_count() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        write_file(&path, "xx\nxx\n");
        let mut store = SnippetStore::new();
        let (snip, _) = store.issue_for_file(&path, None, None).unwrap();
        store.edit(&snip.snippet_id, "xx", "yy", Some(2)).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "yy\nyy\n");
    }

    #[test]
    fn write_create_new_ok() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("new.txt");
        let mut store = SnippetStore::new();
        store.write_new(&path, "hi\n").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "hi\n");
    }

    // --- Spec 45 §1.9: line endings -----------------------------------------

    #[test]
    fn read_normalizes_crlf_content() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        write_file(&path, "alpha\r\nbeta\r\ngamma\r\n");
        let mut store = SnippetStore::new();
        let (snip, content) = store.issue_for_file(&path, None, None).unwrap();
        // The model sees LF only; the file's convention travels as metadata.
        assert!(!content.contains('\r'));
        assert_eq!(content, "alpha\nbeta\ngamma\n");
        assert!(!snip.preview.contains('\r'));
        assert_eq!(snip.line_ending, LineEnding::Crlf);
        assert_eq!(snip.scope, "whole_file");
    }

    #[test]
    fn edit_multiline_crlf_file_with_lf_old_string() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        write_file(&path, "alpha\r\nbeta\r\ngamma\r\n");
        let mut store = SnippetStore::new();
        let (snip, _) = store.issue_for_file(&path, None, None).unwrap();
        // Pre-§1.9 this returned NoMatch: the scope held "beta\r\ngamma".
        store
            .edit(&snip.snippet_id, "beta\ngamma", "BETA\nGAMMA", None)
            .unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert_eq!(after, "alpha\r\nBETA\r\nGAMMA\r\n");
    }

    #[test]
    fn edit_preserves_crlf_on_write() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        write_file(&path, "alpha\r\nbeta\r\ngamma\r\n");
        let mut store = SnippetStore::new();
        let (snip, _) = store.issue_for_file(&path, None, None).unwrap();
        store.edit(&snip.snippet_id, "beta", "BETA", None).unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert_eq!(after, "alpha\r\nBETA\r\ngamma\r\n");
        // Every break is CRLF; no line was left as LF.
        assert!(!after.replace("\r\n", "").contains('\n'));
    }

    #[test]
    fn edit_lf_file_stays_lf() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        write_file(&path, "alpha\nbeta\ngamma\n");
        let mut store = SnippetStore::new();
        let (snip, _) = store.issue_for_file(&path, None, None).unwrap();
        assert_eq!(snip.line_ending, LineEnding::Lf);
        store.edit(&snip.snippet_id, "beta", "BETA", None).unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert_eq!(after, "alpha\nBETA\ngamma\n");
        assert!(!after.contains('\r'));
    }

    #[test]
    fn edit_normalizes_crlf_new_string() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        write_file(&path, "alpha\r\nbeta\r\ngamma\r\n");
        let mut store = SnippetStore::new();
        let (snip, _) = store.issue_for_file(&path, None, None).unwrap();
        // A model that echoed CRLF back must not produce "BETA\r\r\nGAMMA":
        // the CR arrives on every line of the old_string, not just the seam.
        store
            .edit(&snip.snippet_id, "beta\r\ngamma", "BETA\r\nGAMMA", None)
            .unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert_eq!(after, "alpha\r\nBETA\r\nGAMMA\r\n");
        assert!(!after.contains("\r\r"));
    }

    #[test]
    fn edit_uniformizes_mixed_endings() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        write_file(&path, "one\r\ntwo\nthree\r\n");
        let mut store = SnippetStore::new();
        let (snip, _) = store.issue_for_file(&path, None, None).unwrap();
        assert_eq!(snip.line_ending, LineEnding::Crlf);
        store.edit(&snip.snippet_id, "two", "TWO", None).unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert_eq!(after, "one\r\nTWO\r\nthree\r\n");
        assert!(!after.replace("\r\n", "").contains('\n'));
    }

    #[test]
    fn edit_no_trailing_newline_file_keeps_shape() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        write_file(&path, "alpha\r\nbeta");
        let mut store = SnippetStore::new();
        let (snip, _) = store.issue_for_file(&path, None, None).unwrap();
        store.edit(&snip.snippet_id, "beta", "BETA", None).unwrap();
        // No newline is invented at the end of the file.
        assert_eq!(fs::read_to_string(&path).unwrap(), "alpha\r\nBETA");
    }

    #[test]
    fn line_ending_detect_and_restore() {
        assert_eq!(LineEnding::detect("no breaks"), LineEnding::Lf);
        assert_eq!(LineEnding::detect("a\nb"), LineEnding::Lf);
        assert_eq!(LineEnding::detect("a\r\nb"), LineEnding::Crlf);
        assert_eq!(LineEnding::detect("a\r\nb\nc"), LineEnding::Crlf);
        assert_eq!(LineEnding::Lf.restore("a\nb"), "a\nb");
        assert_eq!(LineEnding::Crlf.restore("a\nb"), "a\r\nb");
        assert_eq!(LineEnding::Crlf.restore("a\r\nb"), "a\r\nb");
        assert_eq!(LineEnding::Lf.restore("a\r\nb"), "a\nb");
        assert_eq!(LineEnding::Lf.normalize("a\r\nb"), "a\nb");
        assert_eq!(LineEnding::Lf.as_str(), "lf");
        assert_eq!(LineEnding::Crlf.as_str(), "crlf");
    }

    #[test]
    fn write_existing_denied_by_default() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        write_file(&path, "old\n");
        let mut store = SnippetStore::new();
        let err = store.write_new(&path, "new\n").unwrap_err();
        assert_eq!(err, WriteError::Exists);
        assert_eq!(fs::read_to_string(&path).unwrap(), "old\n");
    }

    #[test]
    fn bash_mutation_expires_snippets() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        write_file(&path, "hello\n");
        let mut store = SnippetStore::new();
        let (snip, _) = store.issue_for_file(&path, None, None).unwrap();
        store.expire_path(&path);
        let err = store
            .edit(&snip.snippet_id, "hello", "x", None)
            .unwrap_err();
        assert_eq!(err, EditError::NotFound);
    }
}
