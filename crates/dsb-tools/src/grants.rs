//! Permission grants — session once + persistent always-allow (spec 90 full / Wave B 0.9.0).
//!
//! Persists under the user config root (`~/.deepseek-build/permission-grants.json` by default).
//! Grants **never** override deny (e.g. write-out-cwd stays denied).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::permissions::Scope;

/// File name under the config home.
pub const GRANTS_FILE: &str = "permission-grants.json";

/// User response to an interactive permission ask.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AskChoice {
    /// Allow this tool call only (session memory for matching scopes).
    AllowOnce,
    /// Persist scopes as always-allow (subject to deny list).
    AllowAlways,
    /// Deny this call.
    Deny,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct GrantsFile {
    /// Scopes the user always allows (kebab-case strings on disk).
    #[serde(default)]
    allow_scopes: Vec<String>,
}

/// In-memory + optional on-disk permission grants.
#[derive(Debug, Clone, Default)]
pub struct PermissionGrants {
    /// Session-only allow (allow-once).
    session: BTreeSet<Scope>,
    /// Loaded / updated always-allow set.
    always: BTreeSet<Scope>,
    path: Option<PathBuf>,
}

impl PermissionGrants {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load grants from `home_dir/permission-grants.json` if present.
    pub fn load(home_dir: &Path) -> Self {
        let path = home_dir.join(GRANTS_FILE);
        let mut g = Self {
            path: Some(path.clone()),
            ..Self::default()
        };
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(file) = serde_json::from_str::<GrantsFile>(&raw) {
                for s in file.allow_scopes {
                    if let Some(scope) = Scope::parse(&s) {
                        g.always.insert(scope);
                    }
                }
            }
        }
        g
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn always_scopes(&self) -> &BTreeSet<Scope> {
        &self.always
    }

    pub fn session_scopes(&self) -> &BTreeSet<Scope> {
        &self.session
    }

    /// True when every scope is covered by session-once or always grants.
    pub fn covers(&self, scopes: &[Scope]) -> bool {
        !scopes.is_empty()
            && scopes
                .iter()
                .all(|s| self.session.contains(s) || self.always.contains(s))
    }

    pub fn grant_once(&mut self, scopes: &[Scope]) {
        for s in scopes {
            self.session.insert(*s);
        }
    }

    /// Grant always-allow for scopes that are **not** hard-denied.
    /// Returns the scopes actually persisted.
    pub fn grant_always(
        &mut self,
        scopes: &[Scope],
        deny: &BTreeSet<Scope>,
    ) -> Result<Vec<Scope>, std::io::Error> {
        let mut added = Vec::new();
        for s in scopes {
            if deny.contains(s) {
                continue;
            }
            if self.always.insert(*s) {
                added.push(*s);
            }
            // also cover session for immediate effect
            self.session.insert(*s);
        }
        self.save()?;
        Ok(added)
    }

    fn save(&self) -> Result<(), std::io::Error> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = GrantsFile {
            allow_scopes: self.always.iter().map(|s| s.as_str().to_string()).collect(),
        };
        let body = serde_json::to_string_pretty(&file)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, body)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn covers_session_and_always() {
        let mut g = PermissionGrants::new();
        g.grant_once(&[Scope::WriteInCwd]);
        assert!(g.covers(&[Scope::WriteInCwd]));
        assert!(!g.covers(&[Scope::Network]));
    }

    #[test]
    fn persist_always_skips_deny_and_reloads() {
        let dir = tempdir().unwrap();
        let mut g = PermissionGrants::load(dir.path());
        let mut deny = BTreeSet::new();
        deny.insert(Scope::WriteOutCwd);
        let added = g
            .grant_always(&[Scope::WriteInCwd, Scope::WriteOutCwd], &deny)
            .unwrap();
        assert_eq!(added, vec![Scope::WriteInCwd]);
        assert!(g.always.contains(&Scope::WriteInCwd));
        assert!(!g.always.contains(&Scope::WriteOutCwd));

        let reloaded = PermissionGrants::load(dir.path());
        assert!(reloaded.always.contains(&Scope::WriteInCwd));
        assert!(!reloaded.always.contains(&Scope::WriteOutCwd));
    }
}
