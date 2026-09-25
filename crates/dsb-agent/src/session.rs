//! Session persistence — multi-turn JSONL under `~/.deepseek-build/sessions/`.
//!
//! On load, tool-call / tool-result pairs are repaired (spec 15) via
//! [`pair_tool_results`] before the transcript is restored.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use dsb_provider_deepseek::ChatMessage;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::cache_totals::CacheSessionTotals;
use crate::pairing::{InterruptedTool, pair_tool_results};

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("session io {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("session {path}: invalid JSONL line {line}: {source}")]
    Json {
        path: PathBuf,
        line: usize,
        #[source]
        source: serde_json::Error,
    },
    #[error("session not found: {0}")]
    NotFound(String),
    #[error("invalid session id: {0}")]
    InvalidId(String),
}

/// One line in a session JSONL file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionRecord {
    Meta {
        id: String,
        created_at_unix: u64,
        #[serde(default)]
        workspace: Option<String>,
        /// Prefix shape of the build that produced this transcript
        /// (spec 10 §1.5.1 rule 5). Absent in files written before that
        /// contract, which is why it is optional: a resume makes no
        /// attribution claim without a baseline. `meta` never enters the
        /// API request.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        prefix_snapshot: Option<PrefixSnapshot>,
        /// Session-cumulative cache totals (spec 10 §1.5.2). Additive and
        /// optional: a file written before this contract loads with a zeroed
        /// counter, which is the honest reading — those turns' evidence was
        /// never recorded, and inventing totals for them would be a
        /// measurement of nothing. `meta` never enters the API request.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cache_totals: Option<CacheSessionTotals>,
    },
    Message {
        message: ChatMessage,
    },
}

/// Stored baseline for cache-change attribution (spec 10 §1.5.1).
///
/// `epoch_short` is the *short* form the log prints; the full epoch is
/// recomputed by the resuming process. Carrying the short form keeps the
/// stored record small and keeps the comparison honest — it compares what the
/// log will show, not a second hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrefixSnapshot {
    pub epoch_short: String,
    pub shape: dsb_context::PrefixShape,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub created_at_unix: u64,
    pub updated_at_unix: u64,
    pub message_count: usize,
    pub path: PathBuf,
}

/// Filesystem-backed session store.
#[derive(Debug, Clone)]
pub struct SessionStore {
    root: PathBuf,
}

impl SessionStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn ensure_root(&self) -> Result<(), SessionError> {
        fs::create_dir_all(&self.root).map_err(|source| SessionError::Io {
            path: self.root.clone(),
            source,
        })
    }

    pub fn path_for(&self, id: &str) -> Result<PathBuf, SessionError> {
        validate_session_id(id)?;
        Ok(self.root.join(format!("{id}.jsonl")))
    }

    /// Create a new empty session file (meta line only). Returns the id.
    pub fn create(
        &self,
        id: Option<&str>,
        workspace: Option<&str>,
    ) -> Result<String, SessionError> {
        self.create_with_snapshot(id, workspace, None)
    }

    /// [`Self::create`] recording the prefix baseline (spec 10 §1.5.1 rule 5).
    pub fn create_with_snapshot(
        &self,
        id: Option<&str>,
        workspace: Option<&str>,
        snapshot: Option<&PrefixSnapshot>,
    ) -> Result<String, SessionError> {
        self.create_with_snapshot_and_totals(id, workspace, snapshot, None)
    }

    /// [`Self::create_with_snapshot`] also recording the session's cache totals
    /// (spec 10 §1.5.2).
    pub fn create_with_snapshot_and_totals(
        &self,
        id: Option<&str>,
        workspace: Option<&str>,
        snapshot: Option<&PrefixSnapshot>,
        cache_totals: Option<&CacheSessionTotals>,
    ) -> Result<String, SessionError> {
        self.ensure_root()?;
        let id = match id {
            Some(s) => {
                validate_session_id(s)?;
                s.to_string()
            }
            None => generate_session_id(),
        };
        let path = self.path_for(&id)?;
        if path.exists() {
            return Ok(id);
        }
        let now = now_unix();
        let meta = SessionRecord::Meta {
            id: id.clone(),
            created_at_unix: now,
            workspace: workspace.map(|s| s.to_string()),
            prefix_snapshot: snapshot.cloned(),
            cache_totals: cache_totals.cloned(),
        };
        let mut f = File::create(&path).map_err(|source| SessionError::Io {
            path: path.clone(),
            source,
        })?;
        writeln!(f, "{}", serde_json::to_string(&meta).unwrap()).map_err(|source| {
            SessionError::Io {
                path: path.clone(),
                source,
            }
        })?;
        Ok(id)
    }

    /// Load transcript, repairing unpaired tool calls (spec 15).
    pub fn load(
        &self,
        id: &str,
    ) -> Result<
        (
            Vec<ChatMessage>,
            Vec<InterruptedTool>,
            Option<SessionRecord>,
        ),
        SessionError,
    > {
        let path = self.path_for(id)?;
        if !path.exists() {
            return Err(SessionError::NotFound(id.to_string()));
        }
        let file = File::open(&path).map_err(|source| SessionError::Io {
            path: path.clone(),
            source,
        })?;
        let reader = BufReader::new(file);
        let mut meta: Option<SessionRecord> = None;
        let mut messages = Vec::new();
        for (idx, line) in reader.lines().enumerate() {
            let line = line.map_err(|source| SessionError::Io {
                path: path.clone(),
                source,
            })?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let rec: SessionRecord =
                serde_json::from_str(line).map_err(|source| SessionError::Json {
                    path: path.clone(),
                    line: idx + 1,
                    source,
                })?;
            match rec {
                SessionRecord::Meta { .. } => meta = Some(rec),
                SessionRecord::Message { message } => messages.push(message),
            }
        }
        let (paired, holes) = pair_tool_results(&messages);
        Ok((paired, holes, meta))
    }

    /// Rewrite the session file with meta + messages (atomic-ish replace).
    pub fn save(
        &self,
        id: &str,
        messages: &[ChatMessage],
        workspace: Option<&str>,
    ) -> Result<(), SessionError> {
        self.save_with_snapshot(id, messages, workspace, None)
    }

    /// [`Self::save`] with an explicit prefix baseline.
    ///
    /// `snapshot: None` **preserves** whatever the file already holds — a save
    /// that drops the baseline would silently disable attribution for the rest
    /// of that session's life, which is the failure this field exists to catch.
    /// Pass `Some(..)` only when the prefix was rebuilt for this transcript.
    pub fn save_with_snapshot(
        &self,
        id: &str,
        messages: &[ChatMessage],
        workspace: Option<&str>,
        snapshot: Option<&PrefixSnapshot>,
    ) -> Result<(), SessionError> {
        self.save_with_snapshot_and_totals(id, messages, workspace, snapshot, None)
    }

    /// [`Self::save_with_snapshot`] also recording the session's cache totals
    /// (spec 10 §1.5.2).
    ///
    /// `cache_totals: None` **preserves** what the file already holds, for the
    /// same reason the snapshot does: dropping the counter would silently
    /// restart a resumed session's totals.
    pub fn save_with_snapshot_and_totals(
        &self,
        id: &str,
        messages: &[ChatMessage],
        workspace: Option<&str>,
        snapshot: Option<&PrefixSnapshot>,
        cache_totals: Option<&CacheSessionTotals>,
    ) -> Result<(), SessionError> {
        self.ensure_root()?;
        let path = self.path_for(id)?;
        let existing = self.load(id).ok().and_then(|(_, _, m)| match m {
            Some(SessionRecord::Meta {
                created_at_unix,
                workspace: existing_workspace,
                prefix_snapshot,
                cache_totals,
                ..
            }) => Some((
                created_at_unix,
                existing_workspace,
                prefix_snapshot,
                cache_totals,
            )),
            _ => None,
        });
        let (created, existing_workspace, existing_snapshot, existing_totals) = match existing {
            Some((created, ws, snap, totals)) => (created, ws, snap, totals),
            None => (now_unix(), None, None, None),
        };
        // An explicit snapshot wins; otherwise keep what the file had.
        let snapshot = snapshot.cloned().or(existing_snapshot);
        let cache_totals = cache_totals.cloned().or(existing_totals);
        let workspace = workspace.map(|s| s.to_string()).or(existing_workspace);
        let tmp = path.with_extension("jsonl.tmp");
        {
            let mut f = File::create(&tmp).map_err(|source| SessionError::Io {
                path: tmp.clone(),
                source,
            })?;
            let meta = SessionRecord::Meta {
                id: id.to_string(),
                created_at_unix: created,
                workspace,
                prefix_snapshot: snapshot,
                cache_totals,
            };
            writeln!(f, "{}", serde_json::to_string(&meta).unwrap()).map_err(|source| {
                SessionError::Io {
                    path: tmp.clone(),
                    source,
                }
            })?;
            for message in messages {
                let rec = SessionRecord::Message {
                    message: message.clone(),
                };
                writeln!(f, "{}", serde_json::to_string(&rec).unwrap()).map_err(|source| {
                    SessionError::Io {
                        path: tmp.clone(),
                        source,
                    }
                })?;
            }
            f.flush().map_err(|source| SessionError::Io {
                path: tmp.clone(),
                source,
            })?;
        }
        fs::rename(&tmp, &path).map_err(|source| SessionError::Io {
            path: path.clone(),
            source,
        })?;
        Ok(())
    }

    /// Append a single message (faster path for streaming turns).
    pub fn append_message(&self, id: &str, message: &ChatMessage) -> Result<(), SessionError> {
        let path = self.path_for(id)?;
        if !path.exists() {
            self.create(Some(id), None)?;
        }
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|source| SessionError::Io {
                path: path.clone(),
                source,
            })?;
        let rec = SessionRecord::Message {
            message: message.clone(),
        };
        writeln!(f, "{}", serde_json::to_string(&rec).unwrap()).map_err(|source| {
            SessionError::Io {
                path: path.clone(),
                source,
            }
        })?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<SessionSummary>, SessionError> {
        self.ensure_root()?;
        let mut out = Vec::new();
        let entries = fs::read_dir(&self.root).map_err(|source| SessionError::Io {
            path: self.root.clone(),
            source,
        })?;
        for ent in entries.flatten() {
            let path = ent.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }
            let id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            if id.is_empty() {
                continue;
            }
            let meta = fs::metadata(&path).map_err(|source| SessionError::Io {
                path: path.clone(),
                source,
            })?;
            let (messages, _, rec) = match self.load(&id) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let created = match rec {
                Some(SessionRecord::Meta {
                    created_at_unix, ..
                }) => created_at_unix,
                _ => 0,
            };
            let updated = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(created);
            out.push(SessionSummary {
                id,
                created_at_unix: created,
                updated_at_unix: updated,
                message_count: messages.len(),
                path,
            });
        }
        out.sort_by(|a, b| b.updated_at_unix.cmp(&a.updated_at_unix));
        Ok(out)
    }

    pub fn delete(&self, id: &str) -> Result<(), SessionError> {
        let path = self.path_for(id)?;
        if !path.exists() {
            return Err(SessionError::NotFound(id.to_string()));
        }
        fs::remove_file(&path).map_err(|source| SessionError::Io {
            path: path.clone(),
            source,
        })
    }
}

fn validate_session_id(id: &str) -> Result<(), SessionError> {
    if id.is_empty() || id.len() > 128 {
        return Err(SessionError::InvalidId(id.to_string()));
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(SessionError::InvalidId(id.to_string()));
    }
    Ok(())
}

fn generate_session_id() -> String {
    let t = now_unix();
    // lightweight uniqueness without extra deps
    let n = std::process::id();
    format!("s{t:x}-{n:x}")
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dsb_provider_deepseek::{FunctionCall, ToolCall};
    use tempfile::tempdir;

    fn call(id: &str) -> ToolCall {
        ToolCall {
            id: id.into(),
            type_: "function".into(),
            function: FunctionCall {
                name: "read".into(),
                arguments: "{}".into(),
            },
        }
    }

    #[test]
    fn save_load_roundtrip() {
        let dir = tempdir().unwrap();
        let store = SessionStore::new(dir.path());
        let id = store.create(Some("demo"), Some("/tmp/ws")).unwrap();
        let msgs = vec![ChatMessage::user("hi"), ChatMessage::assistant("hello")];
        store.save(&id, &msgs, Some("/tmp/ws")).unwrap();
        let (loaded, holes, meta) = store.load(&id).unwrap();
        assert!(holes.is_empty());
        assert_eq!(loaded.len(), 2);
        assert!(matches!(meta, Some(SessionRecord::Meta { .. })));
    }

    fn sample_shape(system: &str) -> dsb_context::PrefixShape {
        dsb_context::PrefixShape {
            system: dsb_context::sub_hash(system.as_bytes()),
            ..dsb_context::PrefixShape::default()
        }
    }

    fn snapshot(system: &str) -> PrefixSnapshot {
        PrefixSnapshot {
            epoch_short: format!("epoch-{system}"),
            shape: sample_shape(system),
        }
    }

    /// Baselines are additive: old files load, new files carry the shape.
    fn meta_of(store: &SessionStore, id: &str) -> SessionRecord {
        let (_, _, meta) = store.load(id).unwrap();
        meta.expect("meta line")
    }

    fn snapshot_of(store: &SessionStore, id: &str) -> Option<PrefixSnapshot> {
        match meta_of(store, id) {
            SessionRecord::Meta {
                prefix_snapshot, ..
            } => prefix_snapshot,
            other => panic!("expected meta, got {other:?}"),
        }
    }

    #[test]
    fn prefix_snapshot_roundtrip() {
        let dir = tempdir().unwrap();
        let store = SessionStore::new(dir.path());
        let snap = snapshot("sys-a");
        let id = store
            .create_with_snapshot(Some("snap"), Some("/tmp/ws"), Some(&snap))
            .unwrap();
        store
            .save_with_snapshot(
                &id,
                &[ChatMessage::user("hi")],
                Some("/tmp/ws"),
                Some(&snap),
            )
            .unwrap();
        let loaded = snapshot_of(&store, &id).expect("snapshot stored");
        assert_eq!(loaded, snap);
    }

    #[test]
    fn save_without_snapshot_preserves_the_existing_baseline() {
        let dir = tempdir().unwrap();
        let store = SessionStore::new(dir.path());
        let snap = snapshot("sys-a");
        let id = store.create(Some("keep"), Some("/tmp/ws")).unwrap();
        store
            .save_with_snapshot(
                &id,
                &[ChatMessage::user("hi")],
                Some("/tmp/ws"),
                Some(&snap),
            )
            .unwrap();
        // A plain save (the older call shape) must not drop the baseline —
        // dropping it would silently disable attribution for the session.
        store
            .save(
                &id,
                &[ChatMessage::user("hi"), ChatMessage::assistant("ok")],
                None,
            )
            .unwrap();
        assert_eq!(snapshot_of(&store, &id), Some(snap));
    }

    #[test]
    fn legacy_meta_without_snapshot_loads() {
        let dir = tempdir().unwrap();
        let store = SessionStore::new(dir.path());
        let id = "legacy";
        std::fs::write(
            dir.path().join(format!("{id}.jsonl")),
            concat!(
                r#"{"type":"meta","id":"legacy","created_at_unix":1,"workspace":"/tmp/ws"}"#,
                "\n",
                r#"{"type":"message","message":{"role":"user","content":"hi"}}"#,
                "\n",
            ),
        )
        .unwrap();
        let (msgs, holes, meta) = store.load(id).unwrap();
        assert!(holes.is_empty());
        assert_eq!(msgs.len(), 1);
        assert_eq!(
            snapshot_of(&store, id),
            None,
            "a pre-§1.5.1 file carries no baseline, and must not be invented"
        );
        assert!(matches!(meta, Some(SessionRecord::Meta { .. })));
    }

    #[test]
    fn cache_totals_roundtrip_with_the_session() {
        let dir = tempdir().unwrap();
        let store = SessionStore::new(dir.path());
        let mut totals = CacheSessionTotals::default();
        totals.record(Some(&dsb_provider_deepseek::CacheEvidence::UsageFields {
            cache_hit_tokens: Some(80),
            cache_miss_tokens: Some(20),
        }));
        totals.record(None);
        let id = store.create(Some("totals"), Some("/tmp/ws")).unwrap();
        store
            .save_with_snapshot_and_totals(
                &id,
                &[ChatMessage::user("hi")],
                Some("/tmp/ws"),
                None,
                Some(&totals),
            )
            .unwrap();

        let (_, _, meta) = store.load(&id).unwrap();
        let Some(SessionRecord::Meta {
            cache_totals: Some(loaded),
            ..
        }) = meta
        else {
            panic!("expected stored totals");
        };
        assert_eq!(loaded, totals);
        assert_eq!(loaded.hit_tokens(), 80);
        assert_eq!(loaded.unreported(), 1);
        assert_eq!(loaded.rate_pct(), Some(80));
    }

    /// A save that does not mention totals must not drop them — dropping the
    /// counter silently restarts a resumed session's numbers.
    #[test]
    fn save_without_totals_preserves_the_existing_counter() {
        let dir = tempdir().unwrap();
        let store = SessionStore::new(dir.path());
        let mut totals = CacheSessionTotals::default();
        totals.record(Some(&dsb_provider_deepseek::CacheEvidence::UsageFields {
            cache_hit_tokens: Some(5),
            cache_miss_tokens: Some(5),
        }));
        let id = store.create(Some("keep-totals"), Some("/tmp/ws")).unwrap();
        store
            .save_with_snapshot_and_totals(
                &id,
                &[ChatMessage::user("a")],
                None,
                None,
                Some(&totals),
            )
            .unwrap();
        // The older call shape, with no totals argument.
        store
            .save(&id, &[ChatMessage::user("a"), ChatMessage::user("b")], None)
            .unwrap();

        let (_, _, meta) = store.load(&id).unwrap();
        match meta {
            Some(SessionRecord::Meta { cache_totals, .. }) => {
                assert_eq!(cache_totals, Some(totals));
            }
            other => panic!("expected meta, got {other:?}"),
        }
    }

    /// A file written before §1.5.2 carries no counter; it loads, and the
    /// absence is not turned into an invented total.
    #[test]
    fn legacy_meta_without_cache_totals_loads() {
        let dir = tempdir().unwrap();
        let store = SessionStore::new(dir.path());
        let id = "legacy-totals";
        std::fs::write(
            dir.path().join(format!("{id}.jsonl")),
            concat!(
                r#"{"type":"meta","id":"legacy-totals","created_at_unix":1,"workspace":"/tmp/ws"}"#,
                "\n",
                r#"{"type":"message","message":{"role":"user","content":"hi"}}"#,
                "\n",
            ),
        )
        .unwrap();
        let (msgs, holes, meta) = store.load(id).unwrap();
        assert!(holes.is_empty());
        assert_eq!(msgs.len(), 1);
        match meta {
            Some(SessionRecord::Meta { cache_totals, .. }) => assert_eq!(
                cache_totals, None,
                "a pre-§1.5.2 file carries no counter, and must not be invented"
            ),
            other => panic!("expected meta, got {other:?}"),
        }
    }

    #[test]
    fn load_repairs_unpaired_tool_calls() {
        let dir = tempdir().unwrap();
        let store = SessionStore::new(dir.path());
        let id = store.create(Some("broken"), None).unwrap();
        let msgs = vec![
            ChatMessage::user("read it"),
            ChatMessage::assistant_with_reasoning(
                Some("".into()),
                Some("think".into()),
                Some(vec![call("c1")]),
            ),
            // missing tool result — simulate crash mid-tool
        ];
        store.save(&id, &msgs, None).unwrap();
        let (loaded, holes, _) = store.load(&id).unwrap();
        assert_eq!(holes.len(), 1);
        assert_eq!(holes[0].tool_call_id, "c1");
        assert!(loaded.iter().any(|m| {
            m.tool_call_id.as_deref() == Some("c1")
                && m.content
                    .as_ref()
                    .is_some_and(|c| c.contains("tool_result_interrupted"))
        }));
    }

    #[test]
    fn list_and_delete() {
        let dir = tempdir().unwrap();
        let store = SessionStore::new(dir.path());
        store.create(Some("a"), None).unwrap();
        store.save("a", &[ChatMessage::user("x")], None).unwrap();
        let list = store.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "a");
        store.delete("a").unwrap();
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn rejects_bad_id() {
        let dir = tempdir().unwrap();
        let store = SessionStore::new(dir.path());
        assert!(store.create(Some("../evil"), None).is_err());
        assert!(store.create(Some("has space"), None).is_err());
    }
}
