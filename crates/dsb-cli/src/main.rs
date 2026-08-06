//! DeepSeek Build CLI.
//!
//! Binaries (ADR 0006): **`deepseek-build`** (primary) and **`dsb`** (alias).
//! Version is always full SemVer from the workspace (`MAJOR.MINOR.PATCH`).

use std::io::{self, BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use dsb_agent::{Agent, AgentConfig, Preset, TurnEvent};
use dsb_config::{BuildHome, Credentials};
use dsb_provider_deepseek::{Client, ClientConfig};

/// Resolve invocation name for help/version (`deepseek-build` or `dsb`).
fn invocation_name() -> &'static str {
    let arg0 = std::env::args().next().unwrap_or_default();
    let base = Path::new(&arg0)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("deepseek-build");
    if base == "dsb" || base.starts_with("dsb-") {
        "dsb"
    } else {
        "deepseek-build"
    }
}

/// DeepSeek Build — DeepSeek-native terminal coding agent.
#[derive(Debug, Parser)]
#[command(
    name = "deepseek-build",
    version,
    about = "DeepSeek Build — DeepSeek-native terminal coding agent",
    long_about = None
)]
struct Cli {
    /// Workspace root for project instructions / cwd summary (default: cwd).
    #[arg(long, global = true)]
    cwd: Option<PathBuf>,

    /// Session preset: flash | balanced | max (default: flash).
    #[arg(long, global = true, default_value = "flash")]
    preset: String,

    /// Base URL override (tests / proxies). Default: https://api.deepseek.com
    #[arg(long, global = true, env = "DEEPSEEK_BASE_URL")]
    base_url: Option<String>,

    /// Hide model= visibility lines.
    #[arg(long, global = true, default_value_t = false)]
    quiet_model: bool,

    /// Print reasoning_content deltas to stderr.
    #[arg(long, global = true, default_value_t = false)]
    show_reasoning: bool,

    /// Allow workspace write/delete without interactive ask (headless).
    /// Still denies write/delete outside the workspace.
    #[arg(long, global = true, default_value_t = false)]
    allow_workspace_write: bool,

    /// Actually execute bash (default: classify + permission only).
    #[arg(long, global = true, default_value_t = false)]
    bash_execute: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// One-shot user message (non-interactive).
    Run {
        /// User message. If omitted, read stdin.
        message: Option<String>,
        /// One-shot Pro model for this turn (`deepseek-v4-pro`).
        #[arg(long, default_value_t = false)]
        pro: bool,
    },
    /// Multi-turn REPL (Flash default). Commands: /pro /flash /preset /quit
    Chat,
    /// Alias for `chat`.
    Repl,
}

fn parse_cli() -> Cli {
    let name = invocation_name();
    let long_about = format!(
        "DeepSeek Build — DeepSeek-native terminal coding agent.\n\n\
Set DEEPSEEK_API_KEY or ~/.deepseek-build/credentials.json.\n\
Commands: `deepseek-build` (primary) and `dsb` (alias) are the same program.\n\
Version is always full SemVer (MAJOR.MINOR.PATCH), e.g. 0.1.0 — never bare \"1.0\".\n\n\
Examples:\n  \
  {name} run \"explain this repo\"\n  \
  {name} run --pro \"design the architecture\"\n  \
  dsb chat"
    );
    let cmd = Cli::command()
        .name(name)
        .bin_name(name)
        .long_about(long_about);
    let matches = cmd.try_get_matches().unwrap_or_else(|e| e.exit());
    Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit())
}

#[tokio::main]
async fn main() {
    if let Err(e) = real_main().await {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

async fn real_main() -> Result<()> {
    let cli = parse_cli();
    let inv = invocation_name();
    match cli.command {
        None => {
            eprintln!(
                "{inv}: no subcommand. Try `{inv} --help`, `{inv} run \"…\"`, or `{inv} chat`."
            );
            std::process::exit(2);
        }
        Some(Commands::Run { ref message, pro }) => {
            let text = match message {
                Some(m) => m.clone(),
                None => {
                    let mut buf = String::new();
                    io::stdin().read_to_string(&mut buf)?;
                    buf
                }
            };
            if text.trim().is_empty() {
                bail!("empty message");
            }
            run_once(&cli, text.as_str(), pro).await?;
        }
        Some(Commands::Chat | Commands::Repl) => {
            run_repl(&cli).await?;
        }
    }
    Ok(())
}

async fn build_agent(cli: &Cli) -> Result<Agent> {
    let home = BuildHome::resolve();
    let creds = Credentials::load(&home).context(
        "missing API key — set DEEPSEEK_API_KEY or create ~/.deepseek-build/credentials.json",
    )?;
    let mut cfg = ClientConfig::new(creds.api_key());
    if let Some(url) = &cli.base_url {
        cfg = cfg.with_base_url(url);
    }
    let client = Arc::new(Client::new(cfg)?);

    let workspace = cli
        .cwd
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let preset = Preset::parse(&cli.preset).unwrap_or(Preset::Flash);
    let agent_cfg = AgentConfig {
        workspace_root: workspace,
        preset,
        show_model: !cli.quiet_model,
        allow_workspace_write: cli.allow_workspace_write,
        bash_execute: cli.bash_execute,
        headless: true,
        ..AgentConfig::default()
    };
    Ok(Agent::new(client, agent_cfg)?)
}

async fn run_once(cli: &Cli, message: &str, pro: bool) -> Result<()> {
    let mut agent = build_agent(cli).await?;
    let input = if pro {
        if message.trim_start().starts_with("/pro") {
            message.to_string()
        } else {
            format!("/pro {message}")
        }
    } else {
        message.to_string()
    };

    let show_reasoning = cli.show_reasoning;
    let outcome = agent
        .run_turn(&input, |ev| render_event(ev, show_reasoning))
        .await?;

    println!();
    eprintln!(
        "[model_used={} {}]",
        outcome.model_used,
        outcome.route.visibility_line()
    );
    Ok(())
}

async fn run_repl(cli: &Cli) -> Result<()> {
    let mut agent = build_agent(cli).await?;
    let inv = invocation_name();
    println!("{inv} chat — DeepSeek Build (Flash default). /pro /preset max|flash /quit");
    eprintln!("[prefix_epoch={}]", agent.prefix_epoch_short());

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        print!("> ");
        stdout.flush()?;
        let mut line = String::new();
        let n = stdin.lock().read_line(&mut line)?;
        if n == 0 {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line == "/quit" || line == "/exit" || line == ":q" {
            break;
        }

        let show_reasoning = cli.show_reasoning;
        match agent
            .run_turn(line, |ev| render_event(ev, show_reasoning))
            .await
        {
            Ok(outcome) => {
                println!();
                eprintln!("[{}]", outcome.route.visibility_line());
            }
            Err(e) => {
                eprintln!("error: {e}");
            }
        }
    }
    Ok(())
}

fn render_event(ev: TurnEvent, show_reasoning: bool) {
    match ev {
        TurnEvent::ModelVisibility(s) => {
            eprintln!("[{s}]");
        }
        TurnEvent::PrefixEpoch(s) => {
            eprintln!("[{s}]");
        }
        TurnEvent::CacheEvidence(s) => {
            eprintln!("[{s}]");
        }
        TurnEvent::ReasoningDelta(s) => {
            if show_reasoning {
                eprint!("{s}");
                let _ = io::stderr().flush();
            }
        }
        TurnEvent::ContentDelta(s) => {
            print!("{s}");
            let _ = io::stdout().flush();
        }
        TurnEvent::Warning(w) => {
            eprintln!("[warn] {w}");
        }
        TurnEvent::ToolCallProposed { name, .. } => {
            eprintln!("[tool] {name}");
        }
        TurnEvent::ToolRepairApplied { name } => {
            eprintln!("[repair] {name}");
        }
        TurnEvent::ToolError { name, error } => {
            eprintln!("[tool-error] {name}: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn clap_command_builds() {
        Cli::command().debug_assert();
    }

    #[test]
    fn default_clap_name_is_deepseek_build() {
        let cmd = Cli::command();
        assert_eq!(cmd.get_name(), "deepseek-build");
    }

    #[test]
    fn run_subcommand_exists() {
        let cmd = Cli::command();
        let subs: Vec<_> = cmd.get_subcommands().map(|c| c.get_name()).collect();
        assert!(subs.contains(&"run"));
        assert!(subs.contains(&"chat"));
    }

    #[test]
    fn package_version_is_full_semver() {
        let v = env!("CARGO_PKG_VERSION");
        let re = regex_lite_semver(v);
        assert!(re, "CARGO_PKG_VERSION must be MAJOR.MINOR.PATCH, got {v}");
    }

    fn regex_lite_semver(v: &str) -> bool {
        let parts: Vec<_> = v.split('-').next().unwrap_or(v).split('+').next().unwrap_or(v).split('.').collect();
        if parts.len() != 3 {
            return false;
        }
        parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
    }
}
