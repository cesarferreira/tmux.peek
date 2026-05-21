use anyhow::{Context, Result};
use std::process::Command;

#[derive(Debug)]
pub struct RawPane {
    pub pane_id: String,
    pub window_id: String,
    pub session_name: String,
    pub window_name: String,
    pub pane_pid: u32,
    pub current_command: String,
    pub current_path: String,
}

const FORMAT: &str =
    "#{pane_id}\t#{window_id}\t#{session_name}\t#{window_name}\t#{pane_pid}\t#{pane_current_command}\t#{pane_current_path}";

pub fn list_panes() -> Result<Vec<RawPane>> {
    let output = Command::new("tmux")
        .args(["list-panes", "-a", "-F", FORMAT])
        .output()
        .context("Failed to run tmux list-panes — is tmux running and $TMUX set?")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("tmux list-panes failed: {}", err.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut panes = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.splitn(7, '\t').collect();
        if parts.len() < 7 {
            continue;
        }

        let pid = parts[4].trim().parse::<u32>().unwrap_or(0);
        panes.push(RawPane {
            pane_id: parts[0].to_string(),
            window_id: parts[1].to_string(),
            session_name: parts[2].to_string(),
            window_name: parts[3].to_string(),
            pane_pid: pid,
            current_command: parts[5].to_string(),
            current_path: parts[6].trim().to_string(),
        });
    }

    Ok(panes)
}

pub fn capture_pane(pane_id: &str, lines: u32) -> Result<Vec<String>> {
    let output = Command::new("tmux")
        .args([
            "capture-pane",
            "-p",
            "-e",            // include ANSI escape sequences
            "-t",
            pane_id,
            "-S",
            &format!("-{}", lines),
        ])
        .output()
        .context("Failed to run tmux capture-pane")?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    // Return raw ANSI output — callers strip when needed for text matching
    let raw = String::from_utf8_lossy(&output.stdout).into_owned();
    Ok(raw.lines().map(|l| l.to_string()).collect())
}

pub fn jump_to_pane(session: &str, window_id: &str, pane_id: &str) -> Result<()> {
    Command::new("tmux")
        .args(["switch-client", "-t", session])
        .status()
        .ok();

    Command::new("tmux")
        .args(["select-window", "-t", window_id])
        .status()
        .ok();

    Command::new("tmux")
        .args(["select-pane", "-t", pane_id])
        .status()
        .ok();

    Ok(())
}

pub fn kill_pane(pane_id: &str) -> Result<()> {
    let status = Command::new("tmux")
        .args(["kill-pane", "-t", pane_id])
        .status()
        .context("Failed to execute tmux kill-pane")?;

    if !status.success() {
        anyhow::bail!("tmux kill-pane failed for pane {}", pane_id);
    }

    Ok(())
}

pub fn is_inside_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

pub fn current_pane_id() -> Option<String> {
    std::env::var("TMUX_PANE").ok()
}

pub fn strip_ansi_str(s: &str) -> String {
    let stripped = strip_ansi_escapes::strip(s.as_bytes());
    String::from_utf8_lossy(&stripped).into_owned()
}
