mod cache;
mod classifier;
mod cli;
mod commands;
mod config;
mod git;
mod processes;
mod scanner;
mod tmux;
mod tui;
mod types;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, shells};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};

use cli::{Cli, Command, Shell};
use tui::{app::App, events::{self, Action}, ui};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Tui {
        side_pane: false,
        popup: false,
    }) {
        Command::Tui { .. } => run_tui(),
        Command::List { cached } => commands::list::run(cached),
        Command::Status { refresh } => commands::status::run(refresh),
        Command::Snapshot { json, output } => commands::snapshot::run(json, output),
        Command::Watch { interval, .. } => run_watch(interval),
        Command::Wrap { agent, args } => commands::wrap::run(&agent, &args),
        Command::Hook { agent, event, message } => commands::hook::run(&agent, &event, message),
        Command::Config { init, show } => commands::config_cmd::run(init, show),
        Command::Completions { shell } => run_completions(shell),
    }
}

fn run_tui() -> Result<()> {
    if !tmux::is_inside_tmux() {
        eprintln!("tmux.peek: not inside a tmux session (TMUX not set).");
        std::process::exit(1);
    }

    let state = scanner::scan().unwrap_or_default();
    cache::save(&state).ok();
    let mut app = App::new(state);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let refresh_interval = Duration::from_secs(5);
    let mut last_refresh = Instant::now();

    let result = run_loop(&mut terminal, &mut app, refresh_interval, &mut last_refresh);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    refresh_interval: Duration,
    last_refresh: &mut Instant,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        match events::handle_event(app)? {
            Action::Quit => break,
            Action::Refresh => {
                events::do_refresh(app);
                *last_refresh = Instant::now();
            }
            Action::Continue => {
                if last_refresh.elapsed() >= refresh_interval {
                    events::do_refresh(app);
                    *last_refresh = Instant::now();
                }
            }
        }
    }
    Ok(())
}

fn run_watch(interval_secs: u64) -> Result<()> {
    if !tmux::is_inside_tmux() {
        eprintln!("tmux.peek: not inside a tmux session (TMUX not set).");
        std::process::exit(1);
    }

    let interval = Duration::from_secs(interval_secs);
    println!("Watching agents every {}s — Ctrl-C to stop\n", interval_secs);

    loop {
        print!("\x1B[2J\x1B[1;1H");
        commands::list::run(false)?;
        std::thread::sleep(interval);
    }
}

fn run_completions(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    match shell {
        Shell::Bash => generate(shells::Bash, &mut cmd, &name, &mut io::stdout()),
        Shell::Zsh => generate(shells::Zsh, &mut cmd, &name, &mut io::stdout()),
        Shell::Fish => generate(shells::Fish, &mut cmd, &name, &mut io::stdout()),
        Shell::PowerShell => {
            generate(shells::PowerShell, &mut cmd, &name, &mut io::stdout())
        }
    }
    Ok(())
}
