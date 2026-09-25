//! First-run / setup onboarding (provider + API key + config home).
//!
//! Product contract: bare `{inv}` opens the full-screen DeepSeek agent TUI
//! (Grok Build–class). `chat` is **legacy line-mode only** — never the
//! primary post-setup path.
//!
//! The wizard has two steps: choose how the agent reaches DeepSeek (DeepSeek
//! API or OpenRouter), then paste that provider's key.

use std::io::{self, BufRead, IsTerminal, Write};

use anyhow::{Context, Result, bail};
use dsb_config::{BuildHome, CredentialSource, Credentials, ENV_API_KEY, Provider};

/// Interactive first-run wizard. Saves provider + key to the credentials file.
///
/// `preset` skips step 1 (`setup --provider …`).
pub fn run_setup_wizard(home: &BuildHome, preset: Option<Provider>) -> Result<Credentials> {
    let inv = crate::invocation_name();
    if !io::stdin().is_terminal() {
        bail!(
            "setup needs an interactive terminal (stdin is not a TTY).\n\
             Run `{inv} setup` in a terminal, or `{inv} setup --provider <deepseek|openrouter> --api-key …`,\n\
             or set {ENV_API_KEY}, or create {}.",
            home.credentials_path().display()
        );
    }
    run_wizard(
        home,
        preset,
        &mut io::stdin().lock(),
        &mut io::stdout(),
        &mut io::stderr(),
    )
}

/// Wizard body over explicit streams (unit-tested without a TTY).
///
/// Instructions go to `out`; the two input prompts go to `err` (as the theme
/// picker does), so stdout stays clean for piped consumers.
fn run_wizard(
    home: &BuildHome,
    preset: Option<Provider>,
    input: &mut impl BufRead,
    out: &mut impl Write,
    err: &mut impl Write,
) -> Result<Credentials> {
    let inv = crate::invocation_name();
    let path = home.credentials_path();
    writeln!(out)?;
    writeln!(out, "Welcome to DeepSeek Build ({inv}).")?;
    writeln!(
        out,
        "First-time setup — choose how the agent reaches DeepSeek, then paste an API key."
    )?;
    let provider = match preset {
        Some(p) => p,
        None => prompt_provider(input, out, err, saved_provider(home).unwrap_or_default())?,
    };

    writeln!(out)?;
    writeln!(out, "Step 2/2 — {}", provider.key_label())?;
    writeln!(out, "  1. Create a key: {}", provider.key_url())?;
    writeln!(
        out,
        "  2. Paste it below (stored only in {})",
        path.display()
    )?;
    writeln!(out, "     mode 0600 · never committed to git")?;
    writeln!(
        out,
        "  Endpoint: {} · DeepSeek V4 Flash / Pro",
        crate::agent_launch::provider_base_url(provider)
    )?;
    if provider == Provider::DeepSeek {
        writeln!(
            out,
            "  Tip: you can also set {ENV_API_KEY} (wins over the file)."
        )?;
    }
    writeln!(out)?;
    out.flush()?;

    write!(err, "{}: ", provider.key_label())?;
    err.flush()?;
    let mut line = String::new();
    input
        .read_line(&mut line)
        .context("read API key from stdin")?;
    let key = line.trim();
    if key.is_empty() {
        bail!("empty API key — setup cancelled");
    }

    let creds = save_provider_key(home, provider, key)?;
    writeln!(out)?;
    writeln!(out, "Saved credentials → {}", path.display())?;
    writeln!(
        out,
        "Provider: {} · key: {}",
        provider.display_name(),
        creds.masked_key()
    )?;
    writeln!(out)?;
    writeln!(out, "Next (product entry — same idea as `grok`):")?;
    writeln!(out, "  {inv}")?;
    writeln!(out)?;
    writeln!(out, "  # full-screen DeepSeek agent TUI (default)")?;
    writeln!(out, "  # optional: {inv} auth status")?;
    if provider == Provider::DeepSeek {
        writeln!(
            out,
            "  # legacy line-mode only (not the product): {inv} chat"
        )?;
    }
    writeln!(out)?;
    out.flush()?;
    Ok(creds)
}

/// Step 1: pick the provider. Enter keeps `default` — the provider already
/// saved in this home, or DeepSeek API on a first run — so re-running setup
/// never flips a saved OpenRouter choice by accident.
fn prompt_provider(
    input: &mut impl BufRead,
    out: &mut impl Write,
    err: &mut impl Write,
    default: Provider,
) -> Result<Provider> {
    writeln!(out)?;
    writeln!(out, "Step 1/2 — API provider")?;
    let mark = |p: Provider| if p == default { " (default)" } else { "" };
    writeln!(
        out,
        "  1) DeepSeek API — straight to api.deepseek.com{}",
        mark(Provider::DeepSeek)
    )?;
    writeln!(
        out,
        "  2) OpenRouter   — the same DeepSeek models via openrouter.ai{}",
        mark(Provider::OpenRouter)
    )?;
    out.flush()?;
    let default_number = match default {
        Provider::DeepSeek => 1,
        Provider::OpenRouter => 2,
    };
    for _ in 0..3 {
        write!(err, "Select [{default_number}]: ")?;
        err.flush()?;
        let mut line = String::new();
        if input
            .read_line(&mut line)
            .context("read provider choice from stdin")?
            == 0
        {
            bail!("no provider chosen — setup cancelled");
        }
        if let Some(provider) = picker_answer_to_provider(&line, default) {
            return Ok(provider);
        }
        writeln!(err, "Answer 1 (DeepSeek API) or 2 (OpenRouter).")?;
    }
    bail!("no valid provider chosen — setup cancelled")
}

/// Pure mapping from picker input to a provider ("" => `default`).
fn picker_answer_to_provider(answer: &str, default: Provider) -> Option<Provider> {
    match answer.trim() {
        "" => Some(default),
        "1" => Some(Provider::DeepSeek),
        "2" => Some(Provider::OpenRouter),
        other => Provider::parse(other),
    }
}

/// Save `key` for `provider` and point an existing agent config at it.
pub fn save_provider_key(home: &BuildHome, provider: Provider, key: &str) -> Result<Credentials> {
    let creds = Credentials::save(home, provider, key).context("save credentials")?;
    let kept_custom =
        crate::agent_launch::apply_provider_to_agent_config(home, provider, creds.api_key())
            .context("point agent config at the chosen provider")?;
    for slot in kept_custom {
        eprintln!(
            "note: [model.{slot}] in {} is customized for another endpoint — left unchanged; \
             edit it to use {}.",
            home.path().join("config.toml").display(),
            provider.display_name()
        );
    }
    Ok(creds)
}

/// Provider recorded in `credentials.json` (env ignored): the user's last
/// explicit choice, which every provider-less setup path keeps.
pub fn saved_provider(home: &BuildHome) -> Option<Provider> {
    Credentials::load_with(home, None)
        .ok()
        .map(|c| c.provider())
}

/// First-run gate for the TUI paths: no saved or env key, and the default
/// model stanza cannot authenticate from its own `env_key` either.
pub fn needs_setup(home: &BuildHome) -> bool {
    needs_setup_with(home, |name| std::env::var(name).ok())
}

fn needs_setup_with(home: &BuildHome, env_lookup: impl Fn(&str) -> Option<String>) -> bool {
    let has_key = Credentials::load_with(home, env_lookup(ENV_API_KEY).as_deref()).is_ok();
    !has_key && !crate::agent_launch::configured_model_env_key_available_in(home, env_lookup)
}

/// Load credentials; on missing key + interactive TTY, run setup automatically.
pub fn load_or_setup(interactive: bool) -> Result<Credentials> {
    let home = BuildHome::resolve();
    match Credentials::load(&home) {
        Ok(c) => Ok(c),
        Err(dsb_config::ConfigError::MissingApiKey) if interactive => {
            eprintln!("No API key found — starting first-time setup…");
            run_setup_wizard(&home, None)
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
            println!(
                "provider: {} ({})",
                c.provider().display_name(),
                crate::agent_launch::provider_base_url(c.provider())
            );
            println!("source: {src}");
            println!("key: {}", c.masked_key());
        }
        Err(dsb_config::ConfigError::MissingApiKey) => {
            println!("status: not configured");
            if crate::agent_launch::configured_model_env_key_available(&home) {
                println!(
                    "note: config.toml's default model reads its key from env_key (set in this shell)"
                );
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    struct Run {
        result: Result<Credentials>,
        out: String,
        err: String,
    }

    fn wizard(home: &BuildHome, preset: Option<Provider>, input: &str) -> Run {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let result = run_wizard(home, preset, &mut input.as_bytes(), &mut out, &mut err);
        Run {
            result,
            out: String::from_utf8(out).unwrap(),
            err: String::from_utf8(err).unwrap(),
        }
    }

    #[test]
    fn openrouter_branch_saves_provider_and_asks_for_openrouter_key() {
        let dir = tempfile::tempdir().unwrap();
        let home = BuildHome::from_path(dir.path());
        let run = wizard(&home, None, "2\nsk-or-v1-0123456789abcdef\n");
        let creds = run.result.unwrap();
        assert_eq!(creds.provider(), Provider::OpenRouter);
        assert!(run.out.contains("Step 1/2 — API provider"), "{}", run.out);
        assert!(
            run.out.contains("Step 2/2 — OpenRouter API key"),
            "{}",
            run.out
        );
        assert!(
            run.out.contains("https://openrouter.ai/keys"),
            "{}",
            run.out
        );
        assert!(run.err.contains("Select [1]: "), "{}", run.err);
        assert!(run.err.contains("OpenRouter API key: "), "{}", run.err);
        // Masked on screen; line mode is not offered for OpenRouter.
        assert!(!run.out.contains("0123456789abcdef"), "{}", run.out);
        assert!(!run.out.contains(" chat"), "{}", run.out);
        let saved = Credentials::load_with(&home, None).unwrap();
        assert_eq!(saved.provider(), Provider::OpenRouter);
        assert_eq!(saved.api_key(), "sk-or-v1-0123456789abcdef");
    }

    #[test]
    fn enter_keeps_deepseek_default() {
        let dir = tempfile::tempdir().unwrap();
        let home = BuildHome::from_path(dir.path());
        let run = wizard(&home, None, "\nsk-deepseek-0123456789\n");
        assert_eq!(run.result.unwrap().provider(), Provider::DeepSeek);
        assert!(
            run.out.contains("Step 2/2 — DeepSeek API key"),
            "{}",
            run.out
        );
        assert!(
            run.out.contains("https://platform.deepseek.com/api_keys"),
            "{}",
            run.out
        );
        assert!(run.err.contains("DeepSeek API key: "), "{}", run.err);
    }

    #[test]
    fn invalid_choice_reprompts_then_accepts_name() {
        let dir = tempfile::tempdir().unwrap();
        let home = BuildHome::from_path(dir.path());
        let run = wizard(&home, None, "9\nopenrouter\nsk-or-v1-k\n");
        assert_eq!(run.result.unwrap().provider(), Provider::OpenRouter);
        assert_eq!(run.err.matches("Select [1]: ").count(), 2, "{}", run.err);
    }

    #[test]
    fn eof_or_empty_key_cancels_without_saving() {
        let dir = tempfile::tempdir().unwrap();
        let home = BuildHome::from_path(dir.path());
        assert!(wizard(&home, None, "").result.is_err());
        assert!(wizard(&home, None, "2\n\n").result.is_err());
        assert!(!home.credentials_path().exists());
    }

    #[test]
    fn preset_provider_skips_step_one() {
        let dir = tempfile::tempdir().unwrap();
        let home = BuildHome::from_path(dir.path());
        let run = wizard(&home, Some(Provider::OpenRouter), "sk-or-v1-k\n");
        assert_eq!(run.result.unwrap().provider(), Provider::OpenRouter);
        assert!(!run.out.contains("Step 1/2"), "{}", run.out);
        assert!(!run.err.contains("Select"), "{}", run.err);
    }

    #[test]
    fn choosing_openrouter_retargets_an_existing_deepseek_config() {
        let dir = tempfile::tempdir().unwrap();
        let home = BuildHome::from_path(dir.path());
        // A previous launch seeded DeepSeek defaults before any key existed.
        crate::agent_launch::ensure_product_agent_config(&home).unwrap();
        wizard(&home, None, "2\nsk-or-v1-k\n").result.unwrap();
        let body = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
        assert!(
            body.contains("model = \"deepseek/deepseek-v4-flash\""),
            "{body}"
        );
        assert!(
            body.contains("model = \"deepseek/deepseek-v4-pro\""),
            "{body}"
        );
        assert!(!body.contains("https://api.deepseek.com"), "{body}");
    }

    #[test]
    fn picker_maps_numbers_names_and_default() {
        let ds = Provider::DeepSeek;
        let or = Provider::OpenRouter;
        assert_eq!(picker_answer_to_provider("", ds), Some(ds));
        assert_eq!(picker_answer_to_provider("", or), Some(or));
        assert_eq!(picker_answer_to_provider(" 1 ", or), Some(ds));
        assert_eq!(picker_answer_to_provider("2", ds), Some(or));
        assert_eq!(picker_answer_to_provider("OpenRouter", ds), Some(or));
        assert_eq!(picker_answer_to_provider("deepseek", or), Some(ds));
        assert_eq!(picker_answer_to_provider("3", ds), None);
    }

    #[test]
    fn rerun_with_saved_openrouter_still_asks_but_defaults_to_openrouter() {
        let dir = tempfile::tempdir().unwrap();
        let home = BuildHome::from_path(dir.path());
        Credentials::save(&home, Provider::OpenRouter, "sk-or-v1-old").unwrap();
        let run = wizard(&home, None, "\nsk-or-v1-new\n");
        assert_eq!(run.result.unwrap().provider(), Provider::OpenRouter);
        assert!(run.out.contains("Step 1/2 — API provider"), "{}", run.out);
        assert!(run.out.contains("openrouter.ai (default)"), "{}", run.out);
        assert!(run.err.contains("Select [2]: "), "{}", run.err);
        // Switching away is still one keystroke.
        let run = wizard(&home, None, "1\nsk-deepseek-new\n");
        assert_eq!(run.result.unwrap().provider(), Provider::DeepSeek);
    }

    #[test]
    fn needs_setup_only_when_no_usable_key() {
        let none = |_: &str| None;
        let dir = tempfile::tempdir().unwrap();
        let home = BuildHome::from_path(dir.path());
        assert!(needs_setup_with(&home, none));
        // DEEPSEEK_API_KEY in the environment.
        assert!(!needs_setup_with(&home, |n: &str| {
            (n == ENV_API_KEY).then(|| "sk-env".to_string())
        }));
        // Hand-written stanza with its own variable: set vs unset.
        std::fs::write(
            dir.path().join("config.toml"),
            "[model.deepseek-v4-flash]\nenv_key = \"MY_OR_KEY\"\n",
        )
        .unwrap();
        assert!(!needs_setup_with(&home, |n: &str| {
            (n == "MY_OR_KEY").then(|| "set".to_string())
        }));
        assert!(needs_setup_with(&home, none));
        // Saved credentials.
        Credentials::save(&home, Provider::OpenRouter, "sk-or-v1-k").unwrap();
        assert!(!needs_setup_with(&home, none));
    }
}
