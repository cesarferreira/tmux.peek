use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::types::State;

pub fn state_path() -> PathBuf {
    let base = std::env::var("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs_or_home().join(".local").join("state")
        });
    base.join("tmux.peek").join("state.json")
}

fn dirs_or_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

pub fn load() -> Result<State> {
    let path = state_path();
    let data = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read cache at {}", path.display()))?;
    let state: State = serde_json::from_str(&data)
        .with_context(|| "Failed to parse cache JSON")?;
    Ok(state)
}

pub fn save(state: &State) -> Result<()> {
    let path = state_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create cache dir {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(state)?;
    fs::write(&path, json)
        .with_context(|| format!("Failed to write cache to {}", path.display()))?;
    Ok(())
}
