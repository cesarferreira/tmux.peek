use std::path::Path;
use std::process::Command;

pub struct GitInfo {
    pub repo_name: Option<String>,
    pub branch: Option<String>,
}

pub fn get_git_info(cwd: &str) -> GitInfo {
    GitInfo {
        repo_name: get_repo_name(cwd),
        branch: get_current_branch(cwd),
    }
}

fn get_repo_name(cwd: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", cwd, "rev-parse", "--show-toplevel"])
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Path::new(&root)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
}

fn get_current_branch(cwd: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", cwd, "branch", "--show-current"])
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() {
        None
    } else {
        Some(branch)
    }
}
