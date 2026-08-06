//! DeepSeek Build CLI (`dsb`).
//!
//! M1: `dsb run "…"`, multi-turn REPL, `--pro` / model visibility.

use std::io::{self, BufRead, Read, Write};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use dsb_agent::{Agent, AgentConfig, Preset, TurnEvent};
use dsb_config::{BuildHome, Credentials};
use dsb_provider_deepseek::{Client, ClientConfig};

/// DeepSeek Build — DeepSeek-native terminal coding agent.
#[derive(Debug, Parser)]
#[command(
    name = "dsb",
    version,
    about = "DeepSeek Build — DeepSeek-native terminal coding agent",
    long_about = "Headless DeepSeek coding agent (M1).\n\n\
Set DEEPSEEK_API_KEY or ~/.deepseek-build/credentials.json.\n\
Examples:\n  \
  dsb run \"explain this repo\"\n  \
  dsb run --pro \"design the architecture\"\n  \
  dsb chat"
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

#[tokio::main]
async fn main() {
    if let Err(e) = real_main().await {
        eprintln!("dsb error: {e:#}");
        std::process::exit(1);
    }
}

async fn real_main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        None => {
            eprintln!("dsb: no subcommand. Try `dsb --help`, `dsb run \"…\"`, or `dsb chat`.");
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

    // Ensure final newline after streamed content.
    println!();
    // Always print a clear model-used footer (Pro path must be user-visible).
    eprintln!(
        "[model_used={} {}]",
        outcome.model_used,
        outcome.route.visibility_line()
    );
    Ok(())
}

async fn run_repl(cli: &Cli) -> Result<()> {
    let mut agent = build_agent(cli).await?;
    println!("dsb chat — DeepSeek Build M1 (Flash default). /pro /preset max|flash /quit");
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
                if outcome.assistant_text.is_empty() && outcome.tool_rounds == 0 {
                    // command-only
                    eprintln!("[{}]", outcome.route.visibility_line());
                    continue;
                }
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
            eprintln!("[tool] {name} (M1 stub — not executed)");
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
    fn binary_name_is_dsb() {
        let cmd = Cli::command();
        assert_eq!(cmd.get_name(), "dsb");
    }

    #[test]
    fn run_subcommand_exists() {
        let cmd = Cli::command();
        let subs: Vec<_> = cmd.get_subcommands().map(|c| c.get_name()).collect();
        assert!(subs.contains(&"run"));
        assert!(subs.contains(&"chat"));
    }
}
