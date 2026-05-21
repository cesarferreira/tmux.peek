/// `tmux.peek hook` — receive lifecycle events from agent hooks
///
/// Designed to be called from agent hook scripts:
///
///   Claude Code (.claude/settings.json):
///     "Stop": [{ "command": "tmux.peek hook --agent claude --event stop" }]
///
///   OpenCode:
///     Use the tmux.peek notify script.
///
/// Each call appends to ~/.local/state/tmux.peek/events.jsonl and
/// triggers an immediate cache refresh for the pane.
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum HookEvent {
    Start,
    Stop,
    Notification,
    ToolUse,
    Custom(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HookRecord {
    pub agent: String,
    pub event: String,
    pub pane_id: String,
    pub message: Option<String>,
    pub timestamp: String,
}

pub fn run(agent: &str, event: &str, message: Option<String>) -> Result<()> {
    let pane_id = std::env::var("TMUX_PANE").unwrap_or_else(|_| "unknown".to_string());

    let record = HookRecord {
        agent: agent.to_string(),
        event: event.to_string(),
        pane_id: pane_id.clone(),
        message,
        timestamp: Utc::now().to_rfc3339(),
    };

    append_event(&record)?;

    // Trigger a cache refresh so status picks up the new event
    if let Ok(mut state) = crate::cache::load() {
        // Update the matching pane's reason based on the event
        if let Some(pane) = state.panes.iter_mut().find(|p| p.pane_id == pane_id) {
            match event {
                "stop" => {
                    pane.status = crate::types::AgentStatus::Done;
                    pane.status_reason = "agent stopped (hook)".to_string();
                    pane.confidence = 0.99;
                }
                "notification" => {
                    pane.status = crate::types::AgentStatus::NeedsAttention;
                    pane.status_reason = record
                        .message
                        .clone()
                        .unwrap_or_else(|| "notification".to_string());
                    pane.confidence = 0.99;
                }
                "start" => {
                    pane.status = crate::types::AgentStatus::Running;
                    pane.status_reason = "started (hook)".to_string();
                    pane.confidence = 0.99;
                    pane.status_since = Utc::now();
                }
                _ => {}
            }
            state.updated_at = Some(Utc::now());
        }
        crate::cache::save(&state).ok();
    }

    Ok(())
}

pub fn events_path() -> PathBuf {
    crate::cache::state_path()
        .parent()
        .unwrap()
        .join("events.jsonl")
}

fn append_event(record: &HookRecord) -> Result<()> {
    let path = events_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let line = serde_json::to_string(record)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    writeln!(file, "{}", line)?;
    Ok(())
}
