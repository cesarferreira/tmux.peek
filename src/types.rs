use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    NeedsAttention,
    Running,
    Done,
    Error,
    Unknown,
}

impl AgentStatus {
    pub fn priority(&self) -> u8 {
        match self {
            AgentStatus::NeedsAttention => 0,
            AgentStatus::Error => 1,
            AgentStatus::Running => 2,
            AgentStatus::Done => 3,
            AgentStatus::Unknown => 4,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            AgentStatus::NeedsAttention => "NEEDS ATTENTION",
            AgentStatus::Running => "RUNNING",
            AgentStatus::Done => "DONE",
            AgentStatus::Error => "ERROR",
            AgentStatus::Unknown => "UNKNOWN",
        }
    }

    #[allow(dead_code)]
    pub fn short_label(&self) -> &'static str {
        match self {
            AgentStatus::NeedsAttention => "!",
            AgentStatus::Running => "…",
            AgentStatus::Done => "✓",
            AgentStatus::Error => "✗",
            AgentStatus::Unknown => "?",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPane {
    pub pane_id: String,
    pub window_id: String,
    pub session_name: String,
    pub window_name: String,
    pub pane_pid: u32,
    pub agent: Option<String>,
    pub cwd: String,
    pub repo: Option<String>,
    pub branch: Option<String>,
    pub status: AgentStatus,
    pub status_reason: String,
    pub confidence: f32,
    pub last_output_lines: Vec<String>,
    pub last_seen: DateTime<Utc>,
    pub status_since: DateTime<Utc>,
}

impl AgentPane {
    pub fn elapsed_display(&self) -> String {
        let elapsed = Utc::now()
            .signed_duration_since(self.status_since)
            .num_seconds()
            .max(0) as u64;
        format_elapsed(elapsed)
    }

    pub fn display_name(&self) -> &str {
        self.agent.as_deref().unwrap_or("shell")
    }

    pub fn repo_display(&self) -> &str {
        self.repo
            .as_deref()
            .unwrap_or(self.session_name.as_str())
    }

    pub fn branch_display(&self) -> &str {
        self.branch.as_deref().unwrap_or("–")
    }

    pub fn reason_display(&self) -> String {
        self.status_reason.clone()
    }
}

pub fn format_elapsed(secs: u64) -> String {
    if secs < 60 {
        format!("{:02}s", secs)
    } else if secs < 3600 {
        format!("{:02}m", secs / 60)
    } else {
        format!("{}h", secs / 3600)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct State {
    pub panes: Vec<AgentPane>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl State {
    pub fn attention_count(&self) -> usize {
        self.panes
            .iter()
            .filter(|p| p.status == AgentStatus::NeedsAttention)
            .count()
    }

    pub fn running_count(&self) -> usize {
        self.panes
            .iter()
            .filter(|p| p.status == AgentStatus::Running)
            .count()
    }

    pub fn done_count(&self) -> usize {
        self.panes
            .iter()
            .filter(|p| p.status == AgentStatus::Done)
            .count()
    }
}
