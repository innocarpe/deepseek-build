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
    },
    Message {
        message: ChatMessage,
    },
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
        self.ensure_root()?;
        let path = self.path_for(id)?;
        let created = if path.exists() {
            self.load(id)
                .ok()
                .and_then(|(_, _, m)| match m {
                    Some(SessionRecord::Meta {
                        created_at_unix, ..
                    }) => Some(created_at_unix),
                    _ => None,
                })
                .unwrap_or_else(now_unix)
        } else {
            now_unix()
        };
        let tmp = path.with_extension("jsonl.tmp");
        {
            let mut f = File::create(&tmp).map_err(|source| SessionError::Io {
                path: tmp.clone(),
                source,
            })?;
            let meta = SessionRecord::Meta {
                id: id.to_string(),
                created_at_unix: created,
                workspace: workspace.map(|s| s.to_string()),
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
