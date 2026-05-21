use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Extra agent binary names beyond the built-in list
    #[serde(default)]
    pub extra_agents: Vec<String>,

    /// Sessions to skip entirely
    #[serde(default)]
    pub exclude_sessions: Vec<String>,

    /// If non-empty, only watch these sessions
    #[serde(default)]
    pub include_sessions: Vec<String>,

    /// Custom patterns that classify as needs_attention
    #[serde(default)]
    pub attention_patterns: Vec<CustomPattern>,

    /// Custom patterns that classify as done
    #[serde(default)]
    pub done_patterns: Vec<CustomPattern>,

    /// Status line format: "emoji" (default), "text", "minimal"
    #[serde(default = "default_status_format")]
    pub status_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPattern {
    pub pattern: String,
    pub reason: String,
}

fn default_status_format() -> String {
    "emoji".to_string()
}

impl Config {
    pub fn load() -> Self {
        match try_load() {
            Ok(cfg) => cfg,
            Err(_) => Config::default(),
        }
    }

    pub fn path() -> PathBuf {
        let base = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                PathBuf::from(home).join(".config")
            });
        base.join("tmux.peek").join("config.toml")
    }

    pub fn write_default() -> Result<PathBuf> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let default = include_str!("../config.default.toml");
        fs::write(&path, default)?;
        Ok(path)
    }

    pub fn session_allowed(&self, session: &str) -> bool {
        if self.exclude_sessions.iter().any(|s| s == session) {
            return false;
        }
        if !self.include_sessions.is_empty()
            && !self.include_sessions.iter().any(|s| s == session)
        {
            return false;
        }
        true
    }
}

fn try_load() -> Result<Config> {
    let path = Config::path();
    let text = fs::read_to_string(&path)?;
    let cfg: Config = toml::from_str(&text)?;
    Ok(cfg)
}
