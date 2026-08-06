//! Launch the vendored Grok Build pager as the product coding agent (2.0 entry).
//!
//! Product binaries `deepseek-build` / `dsb` remain this crate for setup/auth and
//! legacy thin REPL. Interactive no-args TTY entry **exec**s the Grok pager
//! composition root (`xai-grok-pager`) installed as `deepseek-build-agent`.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use dsb_config::BuildHome;

/// Env override for the full-screen agent binary.
pub const ENV_AGENT_BIN: &str = "DEEPSEEK_BUILD_AGENT_BIN";
/// Installed name next to product bins / under `~/.deepseek-build/bin/`.
pub const AGENT_BIN_NAME: &str = "deepseek-build-agent";
/// Upstream cargo artifact name from `xai-grok-pager-bin`.
pub const UPSTREAM_AGENT_BIN_NAME: &str = "xai-grok-pager";

/// Ordered candidate paths for the Grok-class agent binary.
pub fn agent_bin_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();

    if let Ok(p) = env::var(ENV_AGENT_BIN) {
        let p = p.trim();
        if !p.is_empty() {
            out.push(PathBuf::from(p));
        }
    }

    let home = BuildHome::resolve();
    out.push(home.path().join("bin").join(AGENT_BIN_NAME));
    out.push(home.path().join("bin").join(UPSTREAM_AGENT_BIN_NAME));

    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            out.push(dir.join(AGENT_BIN_NAME));
            out.push(dir.join(UPSTREAM_AGENT_BIN_NAME));
        }
    }

    if let Ok(cargo_home) = env::var("CARGO_HOME") {
        let d = PathBuf::from(cargo_home).join("bin");
        out.push(d.join(AGENT_BIN_NAME));
        out.push(d.join(UPSTREAM_AGENT_BIN_NAME));
    } else if let Some(h) = env::var_os("HOME") {
        let d = PathBuf::from(h).join(".cargo").join("bin");
        out.push(d.join(AGENT_BIN_NAME));
        out.push(d.join(UPSTREAM_AGENT_BIN_NAME));
    }

    // Dev tree: vendor release build (walk up from CARGO_MANIFEST_DIR when set).
    if let Ok(manifest) = env::var("CARGO_MANIFEST_DIR") {
        let root = Path::new(&manifest).join("../..");
        if let Ok(root) = root.canonicalize() {
            out.push(
                root.join("third_party/grok-build/target/release")
                    .join(UPSTREAM_AGENT_BIN_NAME),
            );
            out.push(root.join("target/release").join(UPSTREAM_AGENT_BIN_NAME));
        }
    }

    out
}

pub fn find_agent_bin() -> Option<PathBuf> {
    for p in agent_bin_candidates() {
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Prepare optional product agent config under product home (DeepSeek defaults).
/// Idempotent: does not overwrite an existing user `config.toml`.
pub fn ensure_product_agent_config(home: &BuildHome) -> Result<()> {
    home.ensure_dir().context("ensure product home")?;
    let config_path = home.path().join("config.toml");
    if config_path.exists() {
        return Ok(());
    }

    let api_key_line = match dsb_config::Credentials::load(home) {
        Ok(c) => format!("api_key = \"{}\"\n", escape_toml_basic(c.api_key())),
        Err(_) => String::new(),
    };

    // Chat Completions backend for DeepSeek (not Grok Responses default).
    let body = format!(
        r#"# DeepSeek Build product defaults (auto-created).
# Generated when launching the Grok-class agent for the first time.
# Product chrome: DeepSeek Build (not "Grok" as product name).

[models]
default = "deepseek-v4-flash"

[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
name = "DeepSeek V4 Flash"
context_window = 128000
api_backend = "chat_completions"
{api_key_line}env_key = "DEEPSEEK_API_KEY"

[model.deepseek-v4-pro]
model = "deepseek-v4-pro"
name = "DeepSeek V4 Pro"
context_window = 128000
api_backend = "chat_completions"
{api_key_line}env_key = "DEEPSEEK_API_KEY"

[endpoints]
xai_api_base_url = "https://api.deepseek.com"

[ui]
# Product default: do not enable YOLO.
"#
    );

    std::fs::write(&config_path, body)
        .with_context(|| format!("write {}", config_path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600));
    }

    Ok(())
}

fn escape_toml_basic(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Exec the Grok-class agent, replacing this process (Unix).
///
/// On failure to find the binary, returns an error with install guidance.
pub fn exec_agent(args: &[String]) -> Result<()> {
    let home = BuildHome::resolve();
    // Best-effort product config; missing credentials still allow agent UI.
    let _ = ensure_product_agent_config(&home);

    let Some(bin) = find_agent_bin() else {
        bail!(
            "Grok-class agent binary not found (looked for `{AGENT_BIN_NAME}` / `{UPSTREAM_AGENT_BIN_NAME}`).\n\
             Build and install:\n\
               ./scripts/build-grok-pager.sh release\n\
               ./scripts/install.sh\n\
             Or set {ENV_AGENT_BIN} to the `xai-grok-pager` binary path.\n\
             Candidates: {}",
            agent_bin_candidates()
                .into_iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    };

    let mut cmd = Command::new(&bin);
    cmd.args(args);
    // Bridge product home into Grok path resolution without mutating process env globally
    // when not needed — Command env is sufficient for the child/exec image.
    cmd.env("GROK_HOME", home.path());
    if env::var_os(dsb_config::ENV_API_KEY).is_none() {
        if let Ok(c) = dsb_config::Credentials::load(&home) {
            cmd.env(dsb_config::ENV_API_KEY, c.api_key());
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = cmd.exec();
        // exec only returns on error
        bail!("failed to exec {}: {err}", bin.display());
    }

    #[cfg(not(unix))]
    {
        let status = cmd
            .status()
            .with_context(|| format!("spawn {}", bin.display()))?;
        if status.success() {
            Ok(())
        } else {
            bail!(
                "agent {} exited with {}",
                bin.display(),
                status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "signal".into())
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidates_include_product_home_agent_name() {
        let cs = agent_bin_candidates();
        assert!(
            cs.iter().any(|p| p.ends_with(AGENT_BIN_NAME)),
            "expected {AGENT_BIN_NAME} in {cs:?}"
        );
    }

    #[test]
    fn toml_escape_quotes() {
        assert_eq!(escape_toml_basic(r#"a"b"#), r#"a\"b"#);
    }
}
