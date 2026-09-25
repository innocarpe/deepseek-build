//! Config and credentials for DeepSeek Build.
//!
//! Secrets load order (ADR 0004):
//! 1. `DEEPSEEK_API_KEY` environment variable
//! 2. `~/.deepseek-build/credentials.json` (mode ideally `0600`)
//!
//! First-run onboarding writes the credentials file via CLI `setup` / `auth login`,
//! together with the chosen [`Provider`]. An OpenRouter choice in the file is
//! never overridden by `DEEPSEEK_API_KEY` (that variable holds a DeepSeek key).
//! Never commit secrets. Project trees are not a secret store.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Env override for user config root (default `~/.deepseek-build`).
pub const ENV_HOME: &str = "DEEPSEEK_BUILD_HOME";
/// Primary API key env var.
pub const ENV_API_KEY: &str = "DEEPSEEK_API_KEY";
/// Env var the OpenRouter model stanzas name as `env_key`.
///
/// Not a credential source for [`Credentials::load`]: it is commonly exported
/// for other tools, and must not silently replace the key chosen in setup.
pub const ENV_OPENROUTER_API_KEY: &str = "OPENROUTER_API_KEY";

/// Endpoint the agent reaches the pinned DeepSeek models through.
///
/// Both serve the same Flash/Pro models (ADR 0005); OpenRouter is an alternate
/// OpenAI-compatible endpoint, not a second model vendor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    #[default]
    DeepSeek,
    OpenRouter,
}

impl Provider {
    pub const ALL: [Self; 2] = [Self::DeepSeek, Self::OpenRouter];

    /// Parse a CLI / config spelling (`deepseek`, `openrouter`).
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "deepseek" => Some(Self::DeepSeek),
            "openrouter" => Some(Self::OpenRouter),
            _ => None,
        }
    }

    /// Stable spelling stored in `credentials.json`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeepSeek => "deepseek",
            Self::OpenRouter => "openrouter",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::DeepSeek => "DeepSeek API",
            Self::OpenRouter => "OpenRouter",
        }
    }

    /// Name of the secret, as prompted in setup.
    pub fn key_label(self) -> &'static str {
        match self {
            Self::DeepSeek => "DeepSeek API key",
            Self::OpenRouter => "OpenRouter API key",
        }
    }

    /// Where a user creates a key for this provider.
    pub fn key_url(self) -> &'static str {
        match self {
            Self::DeepSeek => "https://platform.deepseek.com/api_keys",
            Self::OpenRouter => "https://openrouter.ai/keys",
        }
    }

    /// Env var named by this provider's model stanzas (`env_key`).
    pub fn env_key(self) -> &'static str {
        match self {
            Self::DeepSeek => ENV_API_KEY,
            Self::OpenRouter => ENV_OPENROUTER_API_KEY,
        }
    }
}

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
    #[error("invalid API key: contains spaces or non-ASCII characters")]
    MalformedApiKey,
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
    /// Absent in files written before provider choice existed → DeepSeek.
    #[serde(default)]
    provider: Provider,
}

/// Loaded API credentials (never log the key).
#[derive(Debug, Clone)]
pub struct Credentials {
    api_key: String,
    source: CredentialSource,
    provider: Provider,
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

    pub fn provider(&self) -> Provider {
        self.provider
    }

    /// Load API key from env first, then credentials file.
    pub fn load(home: &BuildHome) -> Result<Self, ConfigError> {
        let from_env = std::env::var(ENV_API_KEY).ok();
        Self::load_with(home, from_env.as_deref())
    }

    /// Load with an explicit env key value (tests inject here; no process env mutation).
    ///
    /// `env_api_key` is the `DEEPSEEK_API_KEY` value. It wins over a DeepSeek
    /// credentials file, but never over a file that chose OpenRouter.
    pub fn load_with(home: &BuildHome, env_api_key: Option<&str>) -> Result<Self, ConfigError> {
        let file = Self::load_from_file(&home.credentials_path());
        if let Ok(creds) = &file
            && creds.provider == Provider::OpenRouter
        {
            return file;
        }
        if let Some(key) = env_api_key.map(str::trim).filter(|k| !k.is_empty()) {
            return Ok(Self {
                api_key: key.to_string(),
                source: CredentialSource::Env,
                provider: Provider::DeepSeek,
            });
        }
        file
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
            provider: parsed.provider,
        })
    }

    /// Persist API key and provider to `credentials.json` with mode `0600` on Unix.
    ///
    /// Does not write the key to process env. For DeepSeek, env still wins on
    /// next `load` if set.
    pub fn save(home: &BuildHome, provider: Provider, api_key: &str) -> Result<Self, ConfigError> {
        let api_key = api_key.trim();
        if api_key.is_empty() {
            return Err(ConfigError::InvalidApiKey);
        }
        // The key is inlined into TOML and masked by byte offset; real keys
        // are printable ASCII, so anything else is a paste accident.
        if !api_key.bytes().all(|b| b.is_ascii_graphic()) {
            return Err(ConfigError::MalformedApiKey);
        }
        home.ensure_dir()?;
        let path = home.credentials_path();
        let body = CredentialsFile {
            api_key: api_key.to_string(),
            provider,
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
            provider,
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
        let saved = Credentials::save(&home, Provider::DeepSeek, "  sk-test-secret-key  ").unwrap();
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
        Credentials::save(&home, Provider::DeepSeek, "sk-x").unwrap();
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
            provider: Provider::DeepSeek,
        };
        let m = c.masked_key();
        assert!(m.starts_with("sk-a"));
        assert!(m.contains('…'));
        assert!(!m.contains("efghij"));
    }

    #[test]
    fn file_without_provider_is_deepseek() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("credentials.json"),
            r#"{"api_key":"sk-legacy"}"#,
        )
        .unwrap();
        let home = BuildHome::from_path(dir.path());
        let creds = Credentials::load_with(&home, None).unwrap();
        assert_eq!(creds.provider(), Provider::DeepSeek);
    }

    #[test]
    fn save_openrouter_round_trips_provider() {
        let dir = tempfile::tempdir().unwrap();
        let home = BuildHome::from_path(dir.path());
        Credentials::save(&home, Provider::OpenRouter, "sk-or-v1-test").unwrap();
        let raw = fs::read_to_string(home.credentials_path()).unwrap();
        assert!(raw.contains(r#""provider": "openrouter""#), "{raw}");
        let loaded = Credentials::load_with(&home, None).unwrap();
        assert_eq!(loaded.provider(), Provider::OpenRouter);
        assert_eq!(loaded.api_key(), "sk-or-v1-test");
        assert_eq!(loaded.source(), CredentialSource::CredentialsFile);
    }

    #[test]
    fn openrouter_file_is_not_overridden_by_deepseek_env() {
        let dir = tempfile::tempdir().unwrap();
        let home = BuildHome::from_path(dir.path());
        Credentials::save(&home, Provider::OpenRouter, "sk-or-v1-file").unwrap();
        let creds = Credentials::load_with(&home, Some("sk-deepseek-env")).unwrap();
        assert_eq!(creds.provider(), Provider::OpenRouter);
        assert_eq!(creds.api_key(), "sk-or-v1-file");
    }

    #[test]
    fn deepseek_env_still_wins_over_deepseek_file() {
        let dir = tempfile::tempdir().unwrap();
        let home = BuildHome::from_path(dir.path());
        Credentials::save(&home, Provider::DeepSeek, "sk-file").unwrap();
        let creds = Credentials::load_with(&home, Some("sk-env")).unwrap();
        assert_eq!(creds.api_key(), "sk-env");
        assert_eq!(creds.provider(), Provider::DeepSeek);
        assert_eq!(creds.source(), CredentialSource::Env);
    }

    #[test]
    fn env_key_survives_corrupt_credentials_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("credentials.json"), "{not json").unwrap();
        let home = BuildHome::from_path(dir.path());
        let creds = Credentials::load_with(&home, Some("sk-env")).unwrap();
        assert_eq!(creds.api_key(), "sk-env");
        assert!(matches!(
            Credentials::load_with(&home, None),
            Err(ConfigError::CredentialsJson { .. })
        ));
    }

    #[test]
    fn save_rejects_keys_with_spaces_or_non_ascii() {
        let dir = tempfile::tempdir().unwrap();
        let home = BuildHome::from_path(dir.path());
        for bad in ["sk-abc def", "sk-\u{d0a4}", "sk-tab\tkey"] {
            assert!(
                matches!(
                    Credentials::save(&home, Provider::OpenRouter, bad),
                    Err(ConfigError::MalformedApiKey)
                ),
                "{bad:?}"
            );
        }
        assert!(!home.credentials_path().exists());
    }

    #[test]
    fn provider_parse_and_spelling_round_trip() {
        for p in Provider::ALL {
            assert_eq!(Provider::parse(p.as_str()), Some(p));
        }
        assert_eq!(Provider::parse(" OpenRouter "), Some(Provider::OpenRouter));
        assert_eq!(Provider::parse("anthropic"), None);
    }
}
