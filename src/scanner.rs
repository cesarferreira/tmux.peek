use anyhow::Result;
use chrono::Utc;

use crate::cache;
use crate::classifier::{self, is_known_agent_with_extras, is_shell};
use crate::commands::wrap as wrap_cmd;
use crate::config::Config;
use crate::git;
use crate::processes;
use crate::tmux;
use crate::types::{AgentPane, State};

/// Scan all tmux panes, classify agents, and return a fresh State.
pub fn scan() -> Result<State> {
    scan_with_config(&Config::load())
}

pub fn scan_with_config(config: &Config) -> Result<State> {
    let raw_panes = tmux::list_panes()?;
    let current_pane = tmux::current_pane_id();
    let all_procs = processes::get_all_processes();

    // Load previous state so we can preserve status_since timestamps
    let prev_state = cache::load().unwrap_or_default();
    let prev_map: std::collections::HashMap<&str, &AgentPane> = prev_state
        .panes
        .iter()
        .map(|p| (p.pane_id.as_str(), p))
        .collect();

    let mut panes = Vec::new();

    for raw in raw_panes {
        // Skip our own TUI pane
        if current_pane.as_deref() == Some(&raw.pane_id) {
            continue;
        }

        // Apply session filters from config
        if !config.session_allowed(&raw.session_name) {
            continue;
        }

        // Prefer wrap-record identity (100% reliable) over process-tree heuristics
        let wrap_agent = wrap_cmd::load_wrap_record(&raw.pane_id);
        let agent_info = wrap_agent.map(|(name, direct)| (name, direct)).or_else(|| {
            processes::detect_agent_with_extras(
                raw.pane_pid,
                &raw.current_command,
                &all_procs,
                &config.extra_agents,
            )
        });

        // Only include panes that have an agent (current or descendant)
        // OR panes where the shell command isn't a plain shell (could be a wrapper)
        let is_agent_pane = agent_info.is_some()
            || (!is_shell(&raw.current_command) && !raw.current_command.is_empty()
                && is_known_agent_with_extras(&raw.current_command, &config.extra_agents));

        if !is_agent_pane {
            continue;
        }

        let agent_name = agent_info.map(|(name, _)| name);
        let git_info = git::get_git_info(&raw.current_path);
        let output_lines = tmux::capture_pane(&raw.pane_id, 80).unwrap_or_default();

        let cmd = agent_name
            .as_deref()
            .unwrap_or(&raw.current_command);

        let classification = classifier::classify_with_config(
            cmd,
            &output_lines,
            agent_name.is_some(),
            Some(config),
        );

        let now = Utc::now();

        // Preserve status_since if the status hasn't changed
        let status_since = prev_map
            .get(raw.pane_id.as_str())
            .filter(|prev| prev.status == classification.status)
            .map(|prev| prev.status_since)
            .unwrap_or(now);

        panes.push(AgentPane {
            pane_id: raw.pane_id,
            window_id: raw.window_id,
            session_name: raw.session_name,
            window_name: raw.window_name,
            pane_pid: raw.pane_pid,
            agent: agent_name,
            cwd: raw.current_path,
            repo: git_info.repo_name,
            branch: git_info.branch,
            status: classification.status,
            status_reason: classification.reason,
            confidence: classification.confidence,
            last_output_lines: output_lines,
            last_seen: now,
            status_since,
        });
    }

    // Sort by status priority, then by elapsed time (oldest first within group)
    panes.sort_by(|a, b| {
        a.status
            .priority()
            .cmp(&b.status.priority())
            .then(b.status_since.cmp(&a.status_since))
    });

    Ok(State {
        panes,
        updated_at: Some(Utc::now()),
    })
}
