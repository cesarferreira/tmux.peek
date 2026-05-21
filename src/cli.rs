use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "tmux.peek",
    about = "Watch your coding agents in tmux and see which ones need your attention",
    version,
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Open the TUI dashboard (default)
    Tui {
        /// Open as a side pane (affects initial width hint, behaviour unchanged)
        #[arg(long)]
        side_pane: bool,

        /// Open as a popup
        #[arg(long)]
        popup: bool,
    },

    /// Print a table of agent panes to stdout
    List {
        /// Use cached state instead of rescanning
        #[arg(long)]
        cached: bool,
    },

    /// Print a one-line summary for tmux status-right
    Status {
        /// Force a rescan instead of reading cache
        #[arg(long)]
        refresh: bool,
    },

    /// Generate a pasteable summary
    Snapshot {
        /// Output as JSON instead of Markdown
        #[arg(long)]
        json: bool,

        /// Write to this file instead of stdout
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Continuously scan and print the list (watch mode)
    Watch {
        /// Refresh interval in seconds
        #[arg(short, long, default_value = "5")]
        interval: u64,

        /// Scan git worktrees in this directory
        #[arg(value_name = "DIR")]
        dir: Option<String>,
    },

    /// Wrap an agent command for reliable lifecycle tracking
    Wrap {
        /// Agent binary to run (e.g. claude, codex, aider)
        #[arg(value_name = "AGENT")]
        agent: String,

        /// Arguments forwarded to the agent
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Receive a lifecycle event from an agent hook script
    Hook {
        /// Agent name
        #[arg(long)]
        agent: String,

        /// Event type: start | stop | notification | tool_use
        #[arg(long)]
        event: String,

        /// Optional message / payload
        #[arg(long)]
        message: Option<String>,
    },

    /// Manage configuration
    Config {
        /// Write a default config file
        #[arg(long)]
        init: bool,

        /// Print the current config file
        #[arg(long)]
        show: bool,
    },

    /// Print shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}
