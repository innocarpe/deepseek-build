//! DeepSeek Build CLI entrypoint.
//!
//! M1 vertical slice: `--version` / `--help` only. Provider, agent loop, and
//! tools land in later PRs under the same binary name `dsb`.

use clap::{Parser, Subcommand};

/// DeepSeek Build — DeepSeek-native terminal coding agent.
#[derive(Debug, Parser)]
#[command(
    name = "dsb",
    version,
    about = "DeepSeek Build — DeepSeek-native terminal coding agent",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Placeholder for headless one-shot runs (M1 follow-up PRs).
    Run {
        /// User message to send (not yet implemented).
        message: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        None => {
            // Bare `dsb` with no subcommand: clap already handled --help/--version
            // when those flags are present. Without them, print a short hint.
            eprintln!("dsb: no subcommand. Try `dsb --help` or `dsb --version`.");
            std::process::exit(2);
        }
        Some(Commands::Run { message: _ }) => {
            eprintln!("dsb run: not implemented yet (M1 provider/agent PRs).");
            std::process::exit(1);
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
}
