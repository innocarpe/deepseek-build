//! First-run / setup onboarding (API key + config home).
//!
//! Product contract: bare `{inv}` opens the full-screen DeepSeek agent TUI
//! (Grok Build–class). `chat` is **legacy line-mode only** — never the
//! primary post-setup path.

use std::io::{self, IsTerminal, Write};

use anyhow::{Context, Result, bail};
use dsb_config::{BuildHome, CredentialSource, Credentials, ENV_API_KEY};

/// Interactive first-run wizard. Saves key to credentials file.
pub fn run_setup_wizard(home: &BuildHome) -> Result<Credentials> {
    let inv = crate::invocation_name();
    let path = home.credentials_path();
    println!();
    println!("Welcome to DeepSeek Build ({inv}).");
    println!("First-time setup — store a DeepSeek API key for the agent.");
    println!();
    println!("  1. Create a key: https://platform.deepseek.com/api_keys");
    println!("  2. Paste it below (stored only in {})", path.display());
    println!("     mode 0600 · never committed to git");
    println!();
    println!("Tip: you can also set {ENV_API_KEY} (wins over the file).");
    println!();

    if !io::stdin().is_terminal() {
        bail!(
            "no API key configured and stdin is not a TTY.\n\
             Run `{inv} setup` in a terminal, or set {ENV_API_KEY}, or create {}.",
            path.display()
        );
    }

    eprint!("DeepSeek API key: ");
    let _ = io::stderr().flush();
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .context("read API key from stdin")?;
    let key = line.trim();
    if key.is_empty() {
        bail!("empty API key — setup cancelled");
    }

    let creds = Credentials::save(home, key).context("save credentials")?;
    println!();
    println!("Saved credentials → {}", path.display());
    println!("Source: credentials file · key: {}", creds.masked_key());
    println!();
    println!("Next (product entry — same idea as `grok`):");
    println!("  {inv}");
    println!();
    println!("  # full-screen DeepSeek agent TUI (default)");
    println!("  # optional: {inv} auth status");
    println!("  # legacy line-mode only (not the product): {inv} chat");
    println!();
    Ok(creds)
}

/// Load credentials; on missing key + interactive TTY, run setup automatically.
pub fn load_or_setup(interactive: bool) -> Result<Credentials> {
    let home = BuildHome::resolve();
    match Credentials::load(&home) {
        Ok(c) => Ok(c),
        Err(dsb_config::ConfigError::MissingApiKey) if interactive => {
            eprintln!("No API key found — starting first-time setup…");
            run_setup_wizard(&home)
        }
        Err(dsb_config::ConfigError::MissingApiKey) => {
            let inv = crate::invocation_name();
            bail!(
                "missing API key.\n\
                 Run `{inv} setup` (interactive) or set {ENV_API_KEY},\n\
                 or create {} with {{\"api_key\":\"…\"}} (mode 0600).",
                home.credentials_path().display()
            );
        }
        Err(e) => Err(e.into()),
    }
}

pub fn print_auth_status() -> Result<()> {
    let home = BuildHome::resolve();
    println!("config home: {}", home.path().display());
    println!("credentials file: {}", home.credentials_path().display());
    match Credentials::load(&home) {
        Ok(c) => {
            let src = match c.source() {
                CredentialSource::Env => format!("environment ({ENV_API_KEY})"),
                CredentialSource::CredentialsFile => "credentials.json".into(),
            };
            println!("status: configured");
            println!("source: {src}");
            println!("key: {}", c.masked_key());
        }
        Err(dsb_config::ConfigError::MissingApiKey) => {
            println!("status: not configured");
            println!("hint: run `{} setup`", crate::invocation_name());
        }
        Err(e) => return Err(e.into()),
    }
    Ok(())
}

pub fn logout() -> Result<()> {
    let home = BuildHome::resolve();
    let removed = Credentials::clear_file(&home)?;
    if removed {
        println!(
            "removed credentials file {}",
            home.credentials_path().display()
        );
    } else {
        println!("no credentials file to remove");
    }
    if std::env::var(ENV_API_KEY).is_ok() {
        println!("note: {ENV_API_KEY} is still set in this shell (logout does not unset env)");
    }
    Ok(())
}

/// True when we should offer interactive setup (TTY stdin).
pub fn can_prompt_setup() -> bool {
    io::stdin().is_terminal()
}
