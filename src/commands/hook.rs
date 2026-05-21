/// `tmux.peek hook` — receive lifecycle events from agent hooks
///
/// Designed to be called from agent hook scripts:
///
///   Claude Code (.claude/settings.json):
///     "PreToolUse": [{ "command": "tmux.peek hook --agent claude --event pre_tool_use" }]
///     "Stop":       [{ "command": "tmux.peek hook --agent claude --event stop" }]
///
/// PreToolUse reads JSON from stdin to extract what tool is running.
/// Each call also appends to ~/.local/state/tmux.peek/events.jsonl.
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum HookEvent {
    Start,
    Stop,
    Notification,
    PreToolUse,
    PostToolUse,
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

/// Written to ~/.local/state/tmux.peek/activity/<pane_id>.json by PreToolUse hooks.
/// The scanner reads this to show what Claude Code is currently doing.
#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityRecord {
    pub pane_id: String,
    pub activity: String,
    pub tool: String,
    pub timestamp: String,
}

#[derive(Debug, Deserialize, Default)]
struct ClaudeHookPayload {
    tool_name: Option<String>,
    tool_input: Option<serde_json::Value>,
}

pub fn run(agent: &str, event: &str, message: Option<String>) -> Result<()> {
    let pane_id = std::env::var("TMUX_PANE").unwrap_or_else(|_| "unknown".to_string());

    if event == "pre_tool_use" || event == "post_tool_use" {
        handle_tool_event(&pane_id, event)?;
        return Ok(());
    }

    let record = HookRecord {
        agent: agent.to_string(),
        event: event.to_string(),
        pane_id: pane_id.clone(),
        message: message.clone(),
        timestamp: Utc::now().to_rfc3339(),
    };

    append_event(&record)?;

    if let Ok(mut state) = crate::cache::load() {
        if let Some(pane) = state.panes.iter_mut().find(|p| p.pane_id == pane_id) {
            match event {
                "stop" => {
                    pane.status = crate::types::AgentStatus::Done;
                    pane.status_reason = "agent stopped (hook)".to_string();
                    pane.confidence = 0.99;
                    pane.status_since = Utc::now();
                    // Remove any stale activity file so Done status shows cleanly
                    let _ = fs::remove_file(activity_path(&pane_id));
                }
                "notification" => {
                    pane.status = crate::types::AgentStatus::NeedsAttention;
                    pane.status_reason = message
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

fn handle_tool_event(pane_id: &str, event: &str) -> Result<()> {
    let mut stdin_buf = String::new();
    std::io::stdin().read_to_string(&mut stdin_buf).ok();

    let payload: ClaudeHookPayload = serde_json::from_str(&stdin_buf).unwrap_or_default();
    let tool = payload.tool_name.as_deref().unwrap_or("unknown").to_string();
    let activity = activity_label(&tool, payload.tool_input.as_ref());

    let rec = ActivityRecord {
        pane_id: pane_id.to_string(),
        activity: activity.clone(),
        tool: tool.clone(),
        timestamp: Utc::now().to_rfc3339(),
    };

    let path = activity_path(pane_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_string(&rec)?)?;

    // Also update the live cache so TUI refreshes without waiting for a full scan
    if let Ok(mut state) = crate::cache::load() {
        if let Some(pane) = state.panes.iter_mut().find(|p| p.pane_id == pane_id) {
            pane.status = crate::types::AgentStatus::Running;
            pane.status_reason = activity;
            pane.confidence = 0.97;
            if event == "pre_tool_use" {
                pane.status_since = Utc::now();
            }
            state.updated_at = Some(Utc::now());
        }
        crate::cache::save(&state).ok();
    }

    Ok(())
}

/// Convert a Claude Code tool name + input into a short human-readable string.
fn activity_label(tool: &str, input: Option<&serde_json::Value>) -> String {
    let get_str = |key: &str| -> Option<&str> {
        input?.get(key)?.as_str()
    };

    match tool {
        "Edit" | "Write" | "Read" | "NotebookEdit" => {
            if let Some(path) = get_str("file_path") {
                let name = std::path::Path::new(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(path);
                format!("{} {}", tool.to_lowercase(), name)
            } else {
                tool.to_lowercase()
            }
        }
        "Bash" => {
            if let Some(cmd) = get_str("command") {
                let short = cmd.trim();
                let end = short.char_indices().nth(50).map(|(i, _)| i).unwrap_or(short.len());
                format!("$ {}", &short[..end])
            } else {
                "running command".to_string()
            }
        }
        "WebFetch" | "WebSearch" => {
            if let Some(url) = get_str("url").or_else(|| get_str("query")) {
                let short = url.trim();
                let end = short.char_indices().nth(40).map(|(i, _)| i).unwrap_or(short.len());
                format!("web: {}", &short[..end])
            } else {
                "web request".to_string()
            }
        }
        "Agent" => "spawning sub-agent".to_string(),
        "TodoWrite" | "TodoRead" => "updating todos".to_string(),
        _ => tool.to_lowercase(),
    }
}

pub fn activity_path(pane_id: &str) -> PathBuf {
    let safe_id = pane_id.replace('%', "pct");
    crate::cache::state_path()
        .parent()
        .unwrap()
        .join("activity")
        .join(format!("{}.json", safe_id))
}

/// Load the most recent activity record for a pane, if written within `max_age_secs`.
pub fn load_activity(pane_id: &str, max_age_secs: i64) -> Option<ActivityRecord> {
    let path = activity_path(pane_id);
    let data = fs::read_to_string(&path).ok()?;
    let rec: ActivityRecord = serde_json::from_str(&data).ok()?;

    let ts = chrono::DateTime::parse_from_rfc3339(&rec.timestamp).ok()?;
    let age = Utc::now().signed_duration_since(ts).num_seconds();
    if age <= max_age_secs {
        Some(rec)
    } else {
        None
    }
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
