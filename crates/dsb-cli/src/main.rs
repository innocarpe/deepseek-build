//! DeepSeek Build CLI.
//!
//! Binaries (ADR 0006): **`deepseek-build`** (primary) and **`dsb`** (alias).
//! Version is always full SemVer from the workspace (`MAJOR.MINOR.PATCH`).

use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result, bail};
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use dsb_agent::{Agent, AgentConfig, Preset, SessionStore, TurnEvent};
use dsb_config::BuildHome;
use dsb_context::discover_skills_index;
use dsb_provider_deepseek::{Client, ClientConfig, ReasoningEffort};
use dsb_tools::{AskChoice, Scope};

mod agent_launch;
mod banner;
mod onboard;
mod terminal_tab_icon;
mod theme;
use banner::{BannerInfo, prompt as styled_prompt, render_welcome};
use theme::{Role, Theme};

/// Resolve invocation name for help/version (`deepseek-build` or `dsb`).
pub(crate) fn invocation_name() -> &'static str {
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
    about = "DeepSeek Build — DeepSeek-native full-screen coding agent",
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

    /// Trusted local dogfood profile: workspace write + bash execute under policy.
    /// Still denies write/delete outside the workspace. Prefer this for daily local coding.
    #[arg(long, global = true, default_value_t = false)]
    dogfood: bool,

    /// Force interactive permission prompts even for `run` (default: prompts on TTY chat only).
    #[arg(long, global = true, default_value_t = false)]
    ask_permissions: bool,

    /// Never prompt; treat ask as deny (default for non-TTY and `run` without --ask-permissions).
    #[arg(long, global = true, default_value_t = false)]
    no_ask_permissions: bool,

    /// Persist/resume multi-turn session id (JSONL under ~/.deepseek-build/sessions/).
    /// Creates the session if missing; resumes and repairs tool pairs if present.
    #[arg(long, global = true)]
    session: Option<String>,

    /// Resume a full-screen TUI session (bare `dsb` / `dsb agent` only).
    /// With no id, resumes the most recent TUI session. Conflicts with --session.
    #[arg(
        long,
        short = 'r',
        global = true,
        num_args = 0..=1,
        default_missing_value = "",
        conflicts_with = "session"
    )]
    resume: Option<String>,

    /// Start the TUI in minimal mode (bare `dsb` / `dsb agent` only).
    #[arg(long, global = true, default_value_t = false)]
    minimal: bool,

    /// Start the TUI fullscreen (bare `dsb` / `dsb agent` only).
    #[arg(
        long,
        global = true,
        default_value_t = false,
        conflicts_with = "minimal"
    )]
    fullscreen: bool,

    /// Reasoning effort: low | high | max (default: from preset / model).
    #[arg(long, global = true)]
    effort: Option<String>,

    /// Disable thinking mode for this process (default: thinking enabled).
    #[arg(long, global = true, default_value_t = false)]
    no_thinking: bool,

    /// Force thinking enabled (default when not --no-thinking).
    #[arg(long, global = true, default_value_t = false)]
    thinking: bool,

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
    /// Line-mode chat (old 1.x UI — not the product TUI). Prefer bare `dsb`.
    Chat,
    /// Alias for `chat` (line-mode only).
    Repl,
    /// Hidden alias for line-mode chat (old name). Prefer bare `dsb` for the product TUI.
    #[command(name = "repl-legacy", hide = true)]
    ReplLegacy,
    /// Open the DeepSeek Build full-screen TUI (same as no-args TTY).
    /// Extra args are forwarded to the agent.
    Agent {
        /// Arguments forwarded to the DeepSeek Build agent binary.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Manage persisted sessions (`~/.deepseek-build/sessions/*.jsonl`).
    #[command(subcommand)]
    Sessions(SessionsCmd),
    /// Skills index (stable prefix names; bodies load via tool `skill`).
    #[command(subcommand)]
    Skills(SkillsCmd),
    /// First-time setup: save DeepSeek API key (interactive).
    Setup {
        /// Non-interactive: write key from this flag (prefer env in CI).
        #[arg(long, env = "DEEPSEEK_API_KEY")]
        api_key: Option<String>,
    },
    /// Auth helpers (login / status / logout).
    #[command(subcommand)]
    Auth(AuthCmd),
}

#[derive(Debug, Subcommand)]
enum AuthCmd {
    /// Interactive login (same as `setup`).
    Login {
        #[arg(long, env = "DEEPSEEK_API_KEY")]
        api_key: Option<String>,
    },
    /// Show whether a key is configured (masked).
    Status,
    /// Remove saved credentials file (does not unset env).
    Logout,
}

#[derive(Debug, Subcommand)]
enum SkillsCmd {
    /// List discovered skills (name + description), sorted.
    List,
}

#[derive(Debug, Subcommand)]
enum SessionsCmd {
    /// List sessions (most recently updated first).
    List,
    /// Show message count / path for a session id.
    Show { id: String },
    /// Delete a session file.
    Delete { id: String },
}

fn parse_cli() -> Cli {
    let name = invocation_name();
    let ver = env!("CARGO_PKG_VERSION");
    // Forbidden bare form for the "never write 0.7" teaching example (MAJOR.MINOR only).
    let bare = {
        let mut parts = ver.split('.');
        match (parts.next(), parts.next()) {
            (Some(maj), Some(min)) => format!("{maj}.{min}"),
            _ => "0.7".to_string(),
        }
    };
    let long_about = format!(
        "DeepSeek Build — DeepSeek-native full-screen coding agent TUI.\n\n\
Product entry: type `{name}` on a TTY → DeepSeek Build TUI (whale + DeepSeek blue).\n\
First-run: `{name} setup` stores API key under ~/.deepseek-build/ (0600).\n\
Line-mode only (old UI, not the product): `{name} chat`.\n\
Commands: `deepseek-build` (primary) and `dsb` (alias) are the same program.\n\
Version is always full SemVer (MAJOR.MINOR.PATCH), e.g. {ver} — never bare \"{bare}\".\n\n\
Examples:\n  \
  {name}                    # DeepSeek Build full-screen TUI (default)\n  \
  {name} setup              # first-run API key\n  \
  {name} chat               # line-mode chat only (legacy)\n  \
  {name} --dogfood          # trusted local coding\n  \
  {name} --resume <id>      # resume a full-screen TUI session\n  \
  {name} run \"explain this repo\"\n  \
  dsb"
    );
    let cmd = Cli::command()
        .name(name)
        .bin_name(name)
        .long_about(long_about);
    let matches = cmd.try_get_matches().unwrap_or_else(|e| e.exit());
    Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit())
}

/// Flags forwarded to the vendored TUI binary on the bare / `agent` paths.
fn tui_forward_flags(cli: &Cli) -> Vec<String> {
    let mut out = Vec::new();
    if cli.minimal {
        out.push("--minimal".to_string());
    } else if cli.fullscreen {
        out.push("--fullscreen".to_string());
    }
    if let Some(id) = cli.resume.as_deref() {
        if id.is_empty() {
            // `dsb --resume` (no id) → resume the most recent TUI session.
            out.push("--resume".to_string());
        } else {
            out.push("--resume".to_string());
            out.push(id.to_string());
        }
    }
    // Spec 30 / VC008: product `--effort` must reach Path A Grok agent wire.
    // Grok pager accepts `--reasoning-effort` and `--effort` as aliases.
    if let Some(effort) = cli.effort.as_deref() {
        out.push("--reasoning-effort".to_string());
        out.push(effort.to_string());
    }
    out
}

/// `--resume` / `--minimal` / `--fullscreen` target the full-screen TUI only.
fn reject_tui_only_flags(cli: &Cli) -> Result<()> {
    if cli.minimal || cli.fullscreen || cli.resume.is_some() {
        bail!(
            "--resume/--minimal/--fullscreen are TUI-only flags (use bare `{inv}` or `{inv} agent`).\n\
             Line-mode sessions use `--session <id>` instead.",
            inv = invocation_name()
        );
    }
    Ok(())
}

/// Paths that run the vendored full-screen TUI (bare invocation or `agent`).
fn is_tui_path(cli: &Cli) -> bool {
    matches!(&cli.command, None | Some(Commands::Agent { .. }))
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
    // TUI-only flags are meaningless outside the bare / `agent` TUI paths.
    if !is_tui_path(&cli) {
        reject_tui_only_flags(&cli)?;
    }
    match cli.command {
        // Product: bare TTY → DeepSeek Build full-screen TUI only.
        None => {
            if !io::stdin().is_terminal() {
                eprintln!(
                    "{inv}: opens the DeepSeek Build TUI (requires a TTY).\n\
                     Pipe/CI: `{inv} run \"…\"` · line-mode: `{inv} chat`\n\
                     Setup: `{inv} setup` · help: `{inv} --help`"
                );
                std::process::exit(2);
            }
            // First-run: ensure credentials when possible (TTY), then agent UI.
            if !BuildHome::resolve().has_credentials() && onboard::can_prompt_setup() {
                let home = BuildHome::resolve();
                let _ = onboard::run_setup_wizard(&home);
            }
            agent_launch::exec_agent(&tui_forward_flags(&cli))?;
        }
        Some(Commands::Agent { ref args }) => {
            if !BuildHome::resolve().has_credentials()
                && io::stdin().is_terminal()
                && onboard::can_prompt_setup()
            {
                let home = BuildHome::resolve();
                let _ = onboard::run_setup_wizard(&home);
            }
            let mut fwd = tui_forward_flags(&cli);
            fwd.extend(args.iter().cloned());
            agent_launch::exec_agent(&fwd)?;
        }
        Some(Commands::Setup { ref api_key }) => {
            run_setup_cmd(api_key.as_deref())?;
        }
        Some(Commands::Auth(ref cmd)) => {
            run_auth_cmd(cmd)?;
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
        Some(Commands::Chat | Commands::Repl | Commands::ReplLegacy) => {
            run_repl(&cli).await?;
        }
        Some(Commands::Sessions(ref cmd)) => {
            run_sessions_cmd(cmd)?;
        }
        Some(Commands::Skills(ref cmd)) => {
            run_skills_cmd(&cli, cmd)?;
        }
    }
    Ok(())
}

fn run_setup_cmd(api_key: Option<&str>) -> Result<()> {
    let home = BuildHome::resolve();
    if let Some(key) = api_key.map(str::trim).filter(|k| !k.is_empty()) {
        let creds = dsb_config::Credentials::save(&home, key)?;
        println!(
            "Saved credentials → {} ({})",
            home.credentials_path().display(),
            creds.masked_key()
        );
        return Ok(());
    }
    onboard::run_setup_wizard(&home)?;
    Ok(())
}

fn run_auth_cmd(cmd: &AuthCmd) -> Result<()> {
    match cmd {
        AuthCmd::Login { api_key } => run_setup_cmd(api_key.as_deref()),
        AuthCmd::Status => onboard::print_auth_status(),
        AuthCmd::Logout => onboard::logout(),
    }
}

fn run_skills_cmd(cli: &Cli, cmd: &SkillsCmd) -> Result<()> {
    match cmd {
        SkillsCmd::List => {
            let home = BuildHome::resolve();
            let workspace = cli
                .cwd
                .clone()
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            let user_skills = {
                let p = home.path().join("skills");
                if p.is_dir() { Some(p) } else { None }
            };
            let idx = discover_skills_index(&workspace, user_skills.as_deref())
                .context("discover skills")?;
            if idx.is_empty() {
                println!(
                    "(no skills found under skills/, .deepseek-build/skills/, or ~/.deepseek-build/skills/)"
                );
                return Ok(());
            }
            let t = Theme::default_readable();
            for ent in idx {
                println!(
                    "{}",
                    t.paint(Role::Tool, &format!("{:<24} {}", ent.name, ent.description))
                );
            }
        }
    }
    Ok(())
}

fn session_store() -> SessionStore {
    let home = BuildHome::resolve();
    SessionStore::new(home.sessions_dir())
}

fn run_sessions_cmd(cmd: &SessionsCmd) -> Result<()> {
    let store = session_store();
    match cmd {
        SessionsCmd::List => {
            let list = store.list().context("list sessions")?;
            if list.is_empty() {
                println!("(no sessions under {})", store.root().display());
                return Ok(());
            }
            for s in list {
                println!(
                    "{}\tmessages={}\tupdated={}\t{}",
                    s.id,
                    s.message_count,
                    s.updated_at_unix,
                    s.path.display()
                );
            }
        }
        SessionsCmd::Show { id } => {
            let (msgs, holes, _) = store
                .load(id)
                .with_context(|| format!("load session {id}"))?;
            println!("id={id}");
            println!("messages={}", msgs.len());
            println!("repaired_tool_holes_on_load={}", holes.len());
            println!("path={}", store.path_for(id)?.display());
        }
        SessionsCmd::Delete { id } => {
            store
                .delete(id)
                .with_context(|| format!("delete session {id}"))?;
            println!("deleted session {id}");
        }
    }
    Ok(())
}

/// Bind session id: create if missing, resume if present. Returns Some(id) when active.
fn bind_session(agent: &mut Agent, cli: &Cli) -> Result<Option<String>> {
    let Some(raw) = cli.session.as_deref() else {
        return Ok(None);
    };
    let store = session_store();
    let id = store
        .create(Some(raw), Some(&agent_workspace(cli)))
        .context("create/open session")?;
    let path = store.path_for(&id)?;
    // load only if file has messages beyond meta
    match store.load(&id) {
        Ok((msgs, holes, _)) if !msgs.is_empty() => {
            agent.load_transcript(msgs);
            eprintln!(
                "[session={id} resume messages; repaired_holes={} path={}]",
                holes.len(),
                path.display()
            );
        }
        Ok(_) => {
            eprintln!("[session={id} new path={}]", path.display());
        }
        Err(e) => return Err(e.into()),
    }
    Ok(Some(id))
}

fn agent_workspace(cli: &Cli) -> String {
    cli.cwd
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .display()
        .to_string()
}

fn persist_if_needed(agent: &Agent, session_id: Option<&str>) -> Result<()> {
    let Some(id) = session_id else {
        return Ok(());
    };
    let store = session_store();
    agent
        .persist_session(&store, id)
        .with_context(|| format!("persist session {id}"))?;
    Ok(())
}

async fn build_agent(cli: &Cli) -> Result<Agent> {
    let home = BuildHome::resolve();
    // First-run: chat/run on TTY open setup wizard instead of a dead-end error.
    // Chat (including bare CLI default) and TTY run: offer setup wizard if needed.
    let interactive = matches!(
        &cli.command,
        None | Some(
            Commands::Chat
                | Commands::Repl
                | Commands::ReplLegacy
                | Commands::Agent { .. }
                | Commands::Run { .. },
        )
    ) && onboard::can_prompt_setup();
    let creds = onboard::load_or_setup(interactive).context("credentials")?;
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
    let effort_override = cli.effort.as_deref().map(ReasoningEffort::from_product);
    let thinking_enabled = if cli.no_thinking {
        Some(false)
    } else if cli.thinking {
        Some(true)
    } else {
        None
    };
    let user_skills_root = {
        let p = home.path().join("skills");
        if p.is_dir() { Some(p) } else { None }
    };
    let interactive = wants_interactive_permissions(cli);
    let ask_callback = if interactive {
        Some(tty_ask_callback())
    } else {
        None
    };
    let agent_cfg = AgentConfig {
        workspace_root: workspace,
        preset,
        show_model: !cli.quiet_model,
        allow_workspace_write: cli.allow_workspace_write || cli.dogfood,
        bash_execute: cli.bash_execute || cli.dogfood,
        dogfood: cli.dogfood,
        headless: !interactive,
        grants_home: Some(home.path().to_path_buf()),
        ask_callback,
        user_skills_root,
        discover_skills: true,
        effort_override,
        thinking_enabled,
        ..AgentConfig::default()
    };
    Ok(Agent::new(client, agent_cfg)?)
}

/// Interactive prompts: TTY chat/repl by default; `run` stays headless unless --ask-permissions.
fn wants_interactive_permissions(cli: &Cli) -> bool {
    if cli.no_ask_permissions || cli.dogfood || cli.allow_workspace_write {
        return false;
    }
    if cli.ask_permissions {
        return io::stdin().is_terminal();
    }
    match &cli.command {
        None
        | Some(Commands::Chat | Commands::Repl | Commands::ReplLegacy | Commands::Agent { .. }) => {
            io::stdin().is_terminal()
        }
        _ => false,
    }
}

/// Stdin-backed allow once / always / deny prompt (mutex for Send+Sync callback).
fn tty_ask_callback() -> dsb_tools::AskCallback {
    let lock = Arc::new(Mutex::new(()));
    Arc::new(move |scopes: &[Scope]| {
        let _g = lock.lock().unwrap_or_else(|e| e.into_inner());
        let list = scopes
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let th = Theme::default_readable();
        eprintln!();
        eprintln!(
            "{}",
            th.paint(
                Role::Warn,
                &format!("[permission] scopes need approval: {list}")
            )
        );
        eprintln!(
            "{}",
            th.paint(
                Role::Accent,
                "  [a] allow once   [A] allow always   [d] deny"
            )
        );
        eprint!(
            "{}",
            th.paint(Role::Accent, "[permission] choice [a/A/d]: ")
        );
        let _ = io::stderr().flush();
        let mut line = String::new();
        if io::stdin().lock().read_line(&mut line).is_err() {
            return AskChoice::Deny;
        }
        match line.trim() {
            "a" | "once" | "y" | "Y" => AskChoice::AllowOnce,
            "A" | "always" => AskChoice::AllowAlways,
            _ => AskChoice::Deny,
        }
    })
}

async fn run_once(cli: &Cli, message: &str, pro: bool) -> Result<()> {
    let mut agent = build_agent(cli).await?;
    let session_id = bind_session(&mut agent, cli)?;
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
    persist_if_needed(&agent, session_id.as_deref())?;

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
    let session_id = bind_session(&mut agent, cli)?;
    print_welcome_banner(cli, &agent, session_id.as_deref());

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let prompt = styled_prompt(&Theme::default_readable());
    loop {
        print!("{prompt}");
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
        if line == "/quit" || line == "/exit" || line == ":q" || line == "/q" {
            persist_if_needed(&agent, session_id.as_deref())?;
            let t = Theme::default_readable();
            println!("{}", t.paint(Role::Model, "bye."));
            break;
        }
        if line == "/help" || line == "/?" {
            print_repl_help();
            continue;
        }

        let show_reasoning = cli.show_reasoning;
        match agent
            .run_turn(line, |ev| render_event(ev, show_reasoning))
            .await
        {
            Ok(outcome) => {
                println!();
                eprintln!(
                    "{}",
                    Theme::default_readable().paint(
                        Role::Model,
                        &format!("[{}]", outcome.route.visibility_line())
                    )
                );
                if let Err(e) = persist_if_needed(&agent, session_id.as_deref()) {
                    eprintln!("[warn] session persist failed: {e:#}");
                }
            }
            Err(e) => {
                eprintln!("error: {e}");
            }
        }
    }
    Ok(())
}

fn print_welcome_banner(cli: &Cli, agent: &Agent, session_id: Option<&str>) {
    let inv = invocation_name();
    let ver = env!("CARGO_PKG_VERSION");
    let t = Theme::default_readable();
    let cwd = cli
        .cwd
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let dog = if cli.dogfood { "dogfood" } else { "safe" };

    let mut info = BannerInfo::default_product(ver, inv);
    info.cwd = cwd.display().to_string();
    info.profile = dog.into();
    info.epoch = agent.prefix_epoch_short().to_string();
    info.session = session_id.map(|s| s.to_string());
    if cli.effort.is_some() || cli.no_thinking || cli.thinking {
        info.effort = Some(cli.effort.clone().unwrap_or_else(|| "default".into()));
        info.thinking = Some(if cli.no_thinking {
            "off".into()
        } else {
            "on".into()
        });
    }

    print!("{}", render_welcome(&t, &info));
    let _ = io::stdout().flush();
}

fn print_repl_help() {
    let t = Theme::default_readable();
    println!(
        "{}",
        t.paint(
            Role::Accent,
            "Commands:  /pro <msg>  /flash <msg>  /preset flash|balanced|max  /help  /quit"
        )
    );
    println!(
        "{}",
        t.paint(
            Role::Model,
            "Flags next launch: --dogfood  --session ID  --effort high  --show-reasoning"
        )
    );
}

fn render_event(ev: TurnEvent, show_reasoning: bool) {
    let t = theme::global();
    match ev {
        TurnEvent::ModelVisibility(s) => {
            eprintln!("{}", t.paint(Role::Model, &format!("[{s}]")));
        }
        TurnEvent::PrefixEpoch(s) => {
            eprintln!("{}", t.paint(Role::Model, &format!("[{s}]")));
        }
        TurnEvent::CacheEvidence(s) => {
            eprintln!("{}", t.paint(Role::Model, &format!("[{s}]")));
        }
        TurnEvent::ReasoningDelta(s) => {
            if show_reasoning {
                eprint!("{}", t.paint(Role::Reasoning, &s));
                let _ = io::stderr().flush();
            }
        }
        TurnEvent::ContentDelta(s) => {
            // Content stays unstyled for maximum readability in light/dark terminals.
            print!("{s}");
            let _ = io::stdout().flush();
        }
        TurnEvent::Warning(w) => {
            eprintln!("{}", t.paint(Role::Warn, &format!("[warn] {w}")));
        }
        TurnEvent::ToolCallProposed { name, .. } => {
            eprintln!("{}", t.paint(Role::Tool, &format!("[tool] {name}")));
        }
        TurnEvent::ToolRepairApplied { name } => {
            eprintln!("{}", t.paint(Role::Accent, &format!("[repair] {name}")));
        }
        TurnEvent::ToolError { name, error } => {
            eprintln!(
                "{}",
                t.paint(Role::Error, &format!("[tool-error] {name}: {error}"))
            );
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
    fn theme_module_deepseek_blue() {
        assert_eq!(theme::DEEPSEEK_BLUE_RGB, (77, 107, 254));
        let plain = Theme::plain().paint(Role::Tool, "t");
        assert_eq!(plain, "t");
    }

    #[test]
    fn welcome_banner_contains_whale_and_version() {
        let t = Theme::plain();
        let mut info = BannerInfo::default_product(env!("CARGO_PKG_VERSION"), "deepseek-build");
        info.cwd = "/tmp".into();
        info.epoch = "deadbeef".into();
        let s = render_welcome(&t, &info);
        assert!(
            s.chars().any(|c| ('\u{2800}'..='\u{28FF}').contains(&c)),
            "whale braille mark"
        );
        assert!(s.contains("DeepSeek Build"));
        assert!(s.contains(env!("CARGO_PKG_VERSION")));
        assert!(s.contains("deadbeef"));
    }

    #[test]
    fn run_subcommand_exists() {
        let cmd = Cli::command();
        let subs: Vec<_> = cmd.get_subcommands().map(|c| c.get_name()).collect();
        assert!(subs.contains(&"run"));
        assert!(subs.contains(&"chat"));
        assert!(subs.contains(&"repl-legacy"));
        assert!(subs.contains(&"agent"));
        assert!(subs.contains(&"sessions"));
    }

    #[test]
    fn tui_forward_flags_empty_by_default() {
        let cli = Cli::try_parse_from(["dsb"]).unwrap();
        assert!(tui_forward_flags(&cli).is_empty());
    }

    #[test]
    fn tui_forward_flags_resume_with_id() {
        let cli = Cli::try_parse_from(["dsb", "--resume", "sess-abc"]).unwrap();
        assert_eq!(tui_forward_flags(&cli), vec!["--resume", "sess-abc"]);
    }

    #[test]
    fn tui_forward_flags_resume_most_recent() {
        let cli = Cli::try_parse_from(["dsb", "--resume"]).unwrap();
        assert_eq!(tui_forward_flags(&cli), vec!["--resume"]);
    }

    #[test]
    fn tui_forward_flags_short_resume() {
        let cli = Cli::try_parse_from(["dsb", "-r", "sess-abc"]).unwrap();
        assert_eq!(tui_forward_flags(&cli), vec!["--resume", "sess-abc"]);
    }

    #[test]
    fn tui_forward_flags_minimal_then_fullscreen_exclusive() {
        let minimal = Cli::try_parse_from(["dsb", "--minimal"]).unwrap();
        assert_eq!(tui_forward_flags(&minimal), vec!["--minimal"]);
        let fullscreen = Cli::try_parse_from(["dsb", "--fullscreen"]).unwrap();
        assert_eq!(tui_forward_flags(&fullscreen), vec!["--fullscreen"]);
        // --minimal + --fullscreen is a clap conflict.
        assert!(Cli::try_parse_from(["dsb", "--minimal", "--fullscreen"]).is_err());
    }

    #[test]
    fn tui_forward_flags_combine_minimal_resume() {
        let cli = Cli::try_parse_from(["dsb", "--minimal", "--resume", "sess-abc"]).unwrap();
        assert_eq!(
            tui_forward_flags(&cli),
            vec!["--minimal", "--resume", "sess-abc"]
        );
    }

    #[test]
    fn tui_forward_flags_effort_as_reasoning_effort() {
        let cli = Cli::try_parse_from(["dsb", "--effort", "high"]).unwrap();
        assert_eq!(tui_forward_flags(&cli), vec!["--reasoning-effort", "high"]);
        let cli = Cli::try_parse_from(["dsb", "--effort", "max", "--minimal"]).unwrap();
        assert_eq!(
            tui_forward_flags(&cli),
            vec!["--minimal", "--reasoning-effort", "max"]
        );
    }

    #[test]
    fn resume_conflicts_with_session() {
        assert!(Cli::try_parse_from(["dsb", "--session", "s1", "--resume"]).is_err());
        assert!(Cli::try_parse_from(["dsb", "--resume", "x", "--session", "s1"]).is_err());
    }

    #[test]
    fn reject_tui_only_flags_on_line_mode() {
        let cli = Cli::try_parse_from(["dsb", "run", "hi", "--resume"]).unwrap();
        assert!(reject_tui_only_flags(&cli).is_err());
        let ok = Cli::try_parse_from(["dsb", "run", "hi"]).unwrap();
        assert!(reject_tui_only_flags(&ok).is_ok());
    }

    #[test]
    fn tui_path_routing_skips_rejection() {
        // Bare / `agent` are TUI paths: the guard never calls reject_tui_only_flags.
        assert!(is_tui_path(&Cli::try_parse_from(["dsb"]).unwrap()));
        assert!(is_tui_path(
            &Cli::try_parse_from(["dsb", "--resume"]).unwrap()
        ));
        assert!(is_tui_path(
            &Cli::try_parse_from(["dsb", "agent", "--resume", "sess-abc"]).unwrap()
        ));
        // Line-mode subcommands are not TUI paths → rejection applies.
        assert!(!is_tui_path(
            &Cli::try_parse_from(["dsb", "run", "hi"]).unwrap()
        ));
        assert!(!is_tui_path(&Cli::try_parse_from(["dsb", "chat"]).unwrap()));
        assert!(!is_tui_path(
            &Cli::try_parse_from(["dsb", "sessions", "list"]).unwrap()
        ));
    }

    #[test]
    fn agent_forwards_resume_then_args() {
        // `--resume` is re-emitted by tui_forward_flags; unknown args pass through.
        // `--dogfood` is a wrapper global flag, so it is consumed (pre-existing behavior).
        let cli =
            Cli::try_parse_from(["dsb", "agent", "--resume", "sess-abc", "--dogfood"]).unwrap();
        let Commands::Agent { args } = cli.command.as_ref().unwrap() else {
            panic!("expected agent subcommand");
        };
        let mut fwd = tui_forward_flags(&cli);
        fwd.extend(args.iter().cloned());
        assert_eq!(fwd, vec!["--resume", "sess-abc"]);

        let cli = Cli::try_parse_from(["dsb", "agent", "--some-tui-only-flag"]).unwrap();
        let Commands::Agent { args } = cli.command.as_ref().unwrap() else {
            panic!("expected agent subcommand");
        };
        let mut fwd = tui_forward_flags(&cli);
        fwd.extend(args.iter().cloned());
        assert_eq!(fwd, vec!["--some-tui-only-flag"]);
    }

    #[test]
    fn package_version_is_full_semver() {
        let v = env!("CARGO_PKG_VERSION");
        let re = regex_lite_semver(v);
        assert!(re, "CARGO_PKG_VERSION must be MAJOR.MINOR.PATCH, got {v}");
    }

    fn regex_lite_semver(v: &str) -> bool {
        let parts: Vec<_> = v
            .split('-')
            .next()
            .unwrap_or(v)
            .split('+')
            .next()
            .unwrap_or(v)
            .split('.')
            .collect();
        if parts.len() != 3 {
            return false;
        }
        parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
    }
}
