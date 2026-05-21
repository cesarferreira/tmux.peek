use clap::{Parser, Subcommand};

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
}
