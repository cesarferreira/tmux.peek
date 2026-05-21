use anyhow::Result;

use crate::cache;
use crate::scanner;
use crate::types::State;

/// Prints one-line summary for tmux status-right.
/// Reads cache if fresh enough, otherwise re-scans.
pub fn run(refresh: bool) -> Result<()> {
    let state = if refresh {
        let s = scanner::scan()?;
        cache::save(&s).ok();
        s
    } else {
        cache::load().unwrap_or_else(|_| {
            let s = scanner::scan().unwrap_or_default();
            cache::save(&s).ok();
            s
        })
    };

    println!("{}", format_status_line(&state));
    Ok(())
}

pub fn format_status_line(state: &State) -> String {
    let attention = state.attention_count();
    let running = state.running_count();
    let done = state.done_count();

    if attention == 0 && running == 0 && done == 0 {
        return "🤖 –".to_string();
    }

    let mut parts = Vec::new();
    if attention > 0 {
        parts.push(format!("{}!", attention));
    }
    if running > 0 {
        parts.push(format!("{}…", running));
    }
    if done > 0 {
        parts.push(format!("{}✓", done));
    }

    format!("🤖 {}", parts.join(" · "))
}
