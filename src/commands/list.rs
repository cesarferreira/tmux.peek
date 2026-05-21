use anyhow::Result;
use crossterm::terminal;

use crate::scanner;
use crate::cache;
use crate::types::{AgentPane, AgentStatus, State};

pub fn run(use_cache: bool) -> Result<()> {
    let state = if use_cache {
        cache::load().unwrap_or_else(|_| refresh_and_return())
    } else {
        refresh_and_return()
    };

    if state.panes.is_empty() {
        println!("No agent panes found.");
        return Ok(());
    }

    print_table(&state);
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        format!("{}…", chars[..max.saturating_sub(1)].iter().collect::<String>())
    }
}

fn refresh_and_return() -> State {
    match scanner::scan() {
        Ok(state) => {
            cache::save(&state).ok();
            state
        }
        Err(e) => {
            eprintln!("Warning: scan failed — {}", e);
            State::default()
        }
    }
}

fn print_table(state: &State) {
    // Fixed columns: 2 indicator + 8 agent + 14 repo + 18 branch + 6 elapsed + 5 spacing gaps
    const FIXED_COLS: usize = 2 + 8 + 14 + 18 + 6 + 5;
    let term_width = terminal::size().map(|(w, _)| w as usize).unwrap_or(120);
    let reason_width = term_width.saturating_sub(FIXED_COLS).max(20);

    let groups = [
        AgentStatus::NeedsAttention,
        AgentStatus::Error,
        AgentStatus::Running,
        AgentStatus::Done,
        AgentStatus::Unknown,
    ];

    for group in &groups {
        let items: Vec<&AgentPane> = state
            .panes
            .iter()
            .filter(|p| &p.status == group)
            .collect();

        if items.is_empty() {
            continue;
        }

        println!("\n  {} [{}]", group.label(), items.len());
        println!("  {}", "─".repeat(70));

        for pane in items {
            let agent = truncate(pane.display_name(), 8);
            let repo = truncate(pane.repo_display(), 14);
            let branch = truncate(pane.branch_display(), 18);
            let elapsed = pane.elapsed_display();
            let reason = truncate(&pane.status_reason, reason_width);

            println!(
                "  {:<8} {:<14} {:<18} {:<6} {}",
                agent, repo, branch, elapsed, reason
            );
        }
    }

    if let Some(ts) = state.updated_at {
        println!(
            "\n  Updated: {}",
            ts.format("%H:%M:%S")
        );
    }
}
