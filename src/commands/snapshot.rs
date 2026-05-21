use anyhow::Result;
use chrono::Utc;

use crate::cache;
use crate::scanner;
use crate::types::{AgentPane, AgentStatus, State};

pub fn run(json: bool, output_path: Option<String>) -> Result<()> {
    let state = cache::load().unwrap_or_else(|_| {
        let s = scanner::scan().unwrap_or_default();
        cache::save(&s).ok();
        s
    });

    let content = if json {
        to_json(&state)?
    } else {
        to_markdown(&state)
    };

    match output_path {
        Some(path) => {
            std::fs::write(&path, &content)?;
            eprintln!("Snapshot written to {}", path);
        }
        None => print!("{}", content),
    }

    Ok(())
}

fn to_markdown(state: &State) -> String {
    let ts = Utc::now().format("%Y-%m-%d %H:%M UTC");
    let mut out = format!("# tmux.peek snapshot — {}\n\n", ts);

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

        out.push_str(&format!("## {} ({})\n\n", group.label(), items.len()));

        for pane in items {
            let agent = pane.display_name();
            let repo = pane.repo_display();
            let branch = pane.branch_display();
            let elapsed = pane.elapsed_display();
            let reason = &pane.status_reason;

            out.push_str(&format!(
                "- **{}** @ {} → {} — *{}* ({})\n",
                agent, repo, branch, reason, elapsed
            ));
        }

        out.push('\n');
    }

    out
}

fn to_json(state: &State) -> Result<String> {
    #[derive(serde::Serialize)]
    struct SnapshotEntry<'a> {
        agent: &'a str,
        pane_id: &'a str,
        session: &'a str,
        repo: &'a str,
        branch: &'a str,
        status: &'a str,
        reason: &'a str,
        elapsed_secs: i64,
        confidence: f32,
    }

    #[derive(serde::Serialize)]
    struct Snapshot<'a> {
        timestamp: String,
        agents: Vec<SnapshotEntry<'a>>,
    }

    let entries: Vec<SnapshotEntry> = state
        .panes
        .iter()
        .map(|p| SnapshotEntry {
            agent: p.display_name(),
            pane_id: &p.pane_id,
            session: &p.session_name,
            repo: p.repo_display(),
            branch: p.branch_display(),
            status: match &p.status {
                AgentStatus::NeedsAttention => "needs_attention",
                AgentStatus::Running => "running",
                AgentStatus::Done => "done",
                AgentStatus::Error => "error",
                AgentStatus::Unknown => "unknown",
            },
            reason: &p.status_reason,
            elapsed_secs: Utc::now()
                .signed_duration_since(p.status_since)
                .num_seconds(),
            confidence: p.confidence,
        })
        .collect();

    let snapshot = Snapshot {
        timestamp: Utc::now().to_rfc3339(),
        agents: entries,
    };

    Ok(serde_json::to_string_pretty(&snapshot)?)
}
