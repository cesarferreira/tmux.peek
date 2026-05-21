use std::collections::{HashMap, HashSet};
use std::process::Command;

use crate::classifier::{is_known_agent, is_shell};

/// Builds a map of pid -> (ppid, comm) from `ps` output.
pub fn get_all_processes() -> HashMap<u32, (u32, String)> {
    let output = Command::new("ps")
        .args(["-ax", "-o", "pid=,ppid=,comm="])
        .stderr(std::process::Stdio::null())
        .output();

    let mut procs: HashMap<u32, (u32, String)> = HashMap::new();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return procs,
    };

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let parts: Vec<&str> = line.trim().splitn(3, char::is_whitespace).collect();
        if parts.len() < 3 {
            continue;
        }
        if let (Ok(pid), Ok(ppid)) = (
            parts[0].trim().parse::<u32>(),
            parts[1].trim().parse::<u32>(),
        ) {
            let comm = parts[2].trim().to_string();
            procs.insert(pid, (ppid, comm));
        }
    }

    procs
}

/// Walk descendants of `root_pid` and return the first known agent name found.
pub fn find_agent_descendant(
    root_pid: u32,
    all_procs: &HashMap<u32, (u32, String)>,
) -> Option<String> {
    // Build children map
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    for (&pid, (ppid, _)) in all_procs {
        children.entry(*ppid).or_default().push(pid);
    }

    // BFS from root_pid
    let mut queue = vec![root_pid];
    let mut visited: HashSet<u32> = HashSet::new();

    while let Some(pid) = queue.pop() {
        if !visited.insert(pid) {
            continue;
        }

        if let Some((_, comm)) = all_procs.get(&pid) {
            let base = comm
                .split('/')
                .next_back()
                .unwrap_or(comm.as_str());
            if is_known_agent(base) {
                return Some(base.to_lowercase());
            }
        }

        if let Some(kids) = children.get(&pid) {
            for &child in kids {
                if !visited.contains(&child) {
                    queue.push(child);
                }
            }
        }
    }

    None
}

/// For a given pane, determine the running agent name if any.
/// Returns (agent_name, is_directly_agent) — the bool is true when
/// pane_current_command itself is the agent, false when found via process tree.
pub fn detect_agent(
    pane_pid: u32,
    current_command: &str,
    all_procs: &HashMap<u32, (u32, String)>,
) -> Option<(String, bool)> {
    let base = current_command
        .split('/')
        .next_back()
        .unwrap_or(current_command);

    if is_known_agent(base) {
        return Some((base.to_lowercase(), true));
    }

    // If the foreground command is a shell or generic process, look at children
    if is_shell(base) || !is_known_agent(base) {
        if let Some(name) = find_agent_descendant(pane_pid, all_procs) {
            return Some((name, false));
        }
    }

    None
}
