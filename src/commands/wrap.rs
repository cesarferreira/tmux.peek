/// `tmux.peek wrap <agent> [args...]`
///
/// A thin wrapper that:
/// 1. Records agent identity + cwd in ~/.local/state/tmux.peek/wrapped/<pane_id>.json
/// 2. Runs the agent
/// 3. On exit, marks the wrap record as finished
///
/// This gives the scanner reliable lifecycle metadata even when process-tree
/// detection would be ambiguous (e.g. node-based agents, wrapper scripts).
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
struct WrapRecord {
    agent: String,
    pane_id: String,
    session: String,
    cwd: String,
    started_at: String,
    finished_at: Option<String>,
    exit_code: Option<i32>,
}

pub fn run(agent: &str, args: &[String]) -> Result<()> {
    let pane_id = std::env::var("TMUX_PANE").unwrap_or_else(|_| "unknown".to_string());
    let session = std::env::var("TMUX").unwrap_or_default();
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    let record = WrapRecord {
        agent: agent.to_string(),
        pane_id: pane_id.clone(),
        session,
        cwd,
        started_at: Utc::now().to_rfc3339(),
        finished_at: None,
        exit_code: None,
    };

    let path = wrap_path(&pane_id);
    save_record(&path, &record)?;

    // Run the agent
    let status = Command::new(agent)
        .args(args)
        .status();

    let (finished_at, exit_code) = match status {
        Ok(s) => (Utc::now().to_rfc3339(), s.code()),
        Err(e) => {
            eprintln!("tmux.peek wrap: failed to run '{}': {}", agent, e);
            (Utc::now().to_rfc3339(), Some(127))
        }
    };

    // Update record with exit info
    let finished = WrapRecord {
        finished_at: Some(finished_at),
        exit_code,
        ..record
    };
    save_record(&path, &finished)?;

    std::process::exit(exit_code.unwrap_or(0));
}

pub fn wrap_path(pane_id: &str) -> PathBuf {
    let safe = pane_id.trim_start_matches('%');
    crate::cache::state_path()
        .parent()
        .unwrap()
        .join("wrapped")
        .join(format!("{}.json", safe))
}

pub fn load_wrap_record(pane_id: &str) -> Option<(String, bool)> {
    let path = wrap_path(pane_id);
    let text = fs::read_to_string(path).ok()?;
    let record: WrapRecord = serde_json::from_str(&text).ok()?;

    // If already finished, don't surface it
    if record.finished_at.is_some() {
        return None;
    }

    Some((record.agent, true))
}

fn save_record(path: &PathBuf, record: &WrapRecord) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(record)?;
    fs::write(path, json)?;
    Ok(())
}
