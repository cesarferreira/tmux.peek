use anyhow::Result;

use crate::cache;
use crate::config::Config;
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

    let cfg = Config::load();
    println!("{}", format_status_line(&state, &cfg.status_format));
    Ok(())
}

pub fn format_status_line(state: &State, format: &str) -> String {
    let attention = state.attention_count();
    let running = state.running_count();
    let done = state.done_count();

    if attention == 0 && running == 0 && done == 0 {
        return match format {
            "minimal" => String::new(),
            "text" => String::new(),
            _ => "🤖 –".to_string(),
        };
    }

    match format {
        "text" => {
            let mut parts = Vec::new();
            if attention > 0 { parts.push(format!("{} need-you", attention)); }
            if running > 0   { parts.push(format!("{} running", running)); }
            if done > 0      { parts.push(format!("{} done", done)); }
            format!("agents: {}", parts.join("  "))
        }
        "minimal" => {
            let mut parts = Vec::new();
            if attention > 0 { parts.push(format!("{}!", attention)); }
            if running > 0   { parts.push(format!("{}…", running)); }
            parts.join(" ")
        }
        _ => {
            // emoji (default)
            let mut parts = Vec::new();
            if attention > 0 { parts.push(format!("{}!", attention)); }
            if running > 0   { parts.push(format!("{}…", running)); }
            if done > 0      { parts.push(format!("{}✓", done)); }
            format!("🤖 {}", parts.join(" · "))
        }
    }
}
