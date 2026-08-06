//! Config and credentials for DeepSeek Build.
//!
//! Secrets load order (ADR 0004):
//! 1. `DEEPSEEK_API_KEY` environment variable
//! 2. `~/.deepseek-build/credentials.json` (mode ideally `0600`)
//!
//! First-run onboarding writes the credentials file via CLI `setup` / `auth login`.
//! Never commit secrets. Project trees are not a secret store.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Env override for user config root (default `~/.deepseek-build`).
pub const ENV_HOME: &str = "DEEPSEEK_BUILD_HOME";
/// Primary API key env var.
pub const ENV_API_KEY: &str = "DEEPSEEK_API_KEY";

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(
        "missing API key: set {ENV_API_KEY} or create credentials.json under the user config root"
    )]
    MissingApiKey,
    #[error("credentials file {path}: {source}")]
    CredentialsIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("credentials file {path}: invalid JSON: {source}")]
    CredentialsJson {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("credentials file {path}: empty api_key")]
    EmptyApiKey { path: PathBuf },
    #[error("invalid API key: empty after trim")]
    InvalidApiKey,
}

/// Resolved user config home directory.
#[derive(Debug, Clone)]
pub struct BuildHome {
    path: PathBuf,
}

impl BuildHome {
    /// Resolve from `DEEPSEEK_BUILD_HOME` or platform default.
    pub fn resolve() -> Self {
        let from_env = std::env::var(ENV_HOME).ok();
        Self::resolve_with(from_env.as_deref())
    }

    /// Resolve with an explicit home override (tests inject env values here).
    pub fn resolve_with(home_override: Option<&str>) -> Self {
        if let Some(p) = home_override {
            let trimmed = p.trim();
            if !trimmed.is_empty() {
                return Self {
                    path: PathBuf::from(trimmed),
                };
            }
        }
        Self {
            path: default_home_path(),
        }
    }

    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn credentials_path(&self) -> PathBuf {
        self.path.join("credentials.json")
    }

    /// Multi-turn session transcripts (JSONL) live under `sessions/`.
    pub fn sessions_dir(&self) -> PathBuf {
        self.path.join("sessions")
    }

    /// Ensure the config home directory exists (`0700` on Unix when created).
    pub fn ensure_dir(&self) -> Result<(), ConfigError> {
        fs::create_dir_all(&self.path).map_err(|source| ConfigError::CredentialsIo {
            path: self.path.clone(),
            source,
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&self.path, fs::Permissions::from_mode(0o700));
        }
        Ok(())
    }

    /// True when env or credentials file can supply a non-empty key.
    pub fn has_credentials(&self) -> bool {
        Credentials::load(self).is_ok()
    }
}

fn default_home_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".deepseek-build")
}

#[derive(Debug, Serialize, Deserialize)]
struct CredentialsFile {
    api_key: String,
}

/// Loaded API credentials (never log the key).
#[derive(Debug, Clone)]
pub struct Credentials {
    api_key: String,
    source: CredentialSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialSource {
    Env,
    CredentialsFile,
}

impl Credentials {
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn source(&self) -> CredentialSource {
        self.source
    }

    /// Load API key from env first, then credentials file.
    pub fn load(home: &BuildHome) -> Result<Self, ConfigError> {
        let from_env = std::env::var(ENV_API_KEY).ok();
        Self::load_with(home, from_env.as_deref())
    }

    /// Load with an explicit env key value (tests inject here; no process env mutation).
    pub fn load_with(home: &BuildHome, env_api_key: Option<&str>) -> Result<Self, ConfigError> {
        if let Some(key) = env_api_key {
            let key = key.trim();
            if !key.is_empty() {
                return Ok(Self {
                    api_key: key.to_string(),
                    source: CredentialSource::Env,
                });
            }
        }
        Self::load_from_file(&home.credentials_path())
    }

    fn load_from_file(path: &Path) -> Result<Self, ConfigError> {
        let raw = fs::read_to_string(path).map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                ConfigError::MissingApiKey
            } else {
                ConfigError::CredentialsIo {
                    path: path.to_path_buf(),
                    source,
                }
            }
        })?;
        let parsed: CredentialsFile =
            serde_json::from_str(&raw).map_err(|source| ConfigError::CredentialsJson {
                path: path.to_path_buf(),
                source,
            })?;
        let api_key = parsed.api_key.trim().to_string();
        if api_key.is_empty() {
            return Err(ConfigError::EmptyApiKey {
                path: path.to_path_buf(),
            });
        }
        Ok(Self {
            api_key,
            source: CredentialSource::CredentialsFile,
        })
    }

    /// Persist API key to `credentials.json` with mode `0600` on Unix.
    ///
    /// Does not write the key to process env. Env still wins on next `load` if set.
    pub fn save(home: &BuildHome, api_key: &str) -> Result<Self, ConfigError> {
        let api_key = api_key.trim();
        if api_key.is_empty() {
            return Err(ConfigError::InvalidApiKey);
        }
        home.ensure_dir()?;
        let path = home.credentials_path();
        let body = CredentialsFile {
            api_key: api_key.to_string(),
        };
        let json =
            serde_json::to_string_pretty(&body).map_err(|source| ConfigError::CredentialsJson {
                path: path.clone(),
                source,
            })?;
        fs::write(&path, format!("{json}\n")).map_err(|source| ConfigError::CredentialsIo {
            path: path.clone(),
            source,
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }
        Ok(Self {
            api_key: api_key.to_string(),
            source: CredentialSource::CredentialsFile,
        })
    }

    /// Remove credentials file if present (logout). Env key is unchanged.
    pub fn clear_file(home: &BuildHome) -> Result<bool, ConfigError> {
        let path = home.credentials_path();
        match fs::remove_file(&path) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(source) => Err(ConfigError::CredentialsIo { path, source }),
        }
    }

    /// Mask for display: first 4 + `…` + last 4 when long enough.
    pub fn masked_key(&self) -> String {
        let k = self.api_key.as_str();
        if k.len() <= 10 {
            return "********".into();
        }
        format!("{}…{}", &k[..4], &k[k.len().saturating_sub(4)..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_override_from_injected_env() {
        let home = BuildHome::resolve_with(Some("/tmp/dsb-test-home"));
        assert_eq!(home.path(), Path::new("/tmp/dsb-test-home"));
    }

    #[test]
    fn empty_home_override_falls_back() {
        let home = BuildHome::resolve_with(Some("  "));
        assert!(home.path().ends_with(".deepseek-build"));
    }

    #[test]
    fn load_prefers_env_over_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("credentials.json"),
            r#"{"api_key":"file-key"}"#,
        )
        .unwrap();
        let home = BuildHome::from_path(dir.path());
        let creds = Credentials::load_with(&home, Some("env-key")).unwrap();
        assert_eq!(creds.api_key(), "env-key");
        assert_eq!(creds.source(), CredentialSource::Env);
    }

    #[test]
    fn load_from_credentials_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("credentials.json"),
            r#"{"api_key":"file-secret"}"#,
        )
        .unwrap();
        let home = BuildHome::from_path(dir.path());
        let creds = Credentials::load_with(&home, None).unwrap();
        assert_eq!(creds.api_key(), "file-secret");
        assert_eq!(creds.source(), CredentialSource::CredentialsFile);
    }

    #[test]
    fn missing_key_errors() {
        let dir = tempfile::tempdir().unwrap();
        let home = BuildHome::from_path(dir.path());
        let err = Credentials::load_with(&home, None).unwrap_err();
        assert!(matches!(err, ConfigError::MissingApiKey));
    }

    #[test]
    fn empty_env_falls_through_to_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("credentials.json"),
            r#"{"api_key":"from-file"}"#,
        )
        .unwrap();
        let home = BuildHome::from_path(dir.path());
        let creds = Credentials::load_with(&home, Some("  ")).unwrap();
        assert_eq!(creds.api_key(), "from-file");
    }

    #[test]
    fn save_and_reload_credentials() {
        let dir = tempfile::tempdir().unwrap();
        let home = BuildHome::from_path(dir.path());
        let saved = Credentials::save(&home, "  sk-test-secret-key  ").unwrap();
        assert_eq!(saved.api_key(), "sk-test-secret-key");
        assert_eq!(saved.source(), CredentialSource::CredentialsFile);
        let loaded = Credentials::load_with(&home, None).unwrap();
        assert_eq!(loaded.api_key(), "sk-test-secret-key");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(home.credentials_path())
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn clear_file_removes_credentials() {
        let dir = tempfile::tempdir().unwrap();
        let home = BuildHome::from_path(dir.path());
        Credentials::save(&home, "sk-x").unwrap();
        assert!(Credentials::clear_file(&home).unwrap());
        assert!(matches!(
            Credentials::load_with(&home, None),
            Err(ConfigError::MissingApiKey)
        ));
    }

    #[test]
    fn masked_key_hides_middle() {
        let c = Credentials {
            api_key: "sk-abcdefghijklmnop".into(),
            source: CredentialSource::Env,
        };
        let m = c.masked_key();
        assert!(m.starts_with("sk-a"));
        assert!(m.contains('…'));
        assert!(!m.contains("efghij"));
    }
}
