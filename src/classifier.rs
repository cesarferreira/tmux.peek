use crate::config::Config;
use crate::types::AgentStatus;
use regex::Regex;

pub struct ClassificationResult {
    pub status: AgentStatus,
    pub reason: String,
    pub confidence: f32,
}

#[allow(dead_code)]
pub fn classify(command: &str, output_lines: &[String], is_agent: bool) -> ClassificationResult {
    classify_with_config(command, output_lines, is_agent, None)
}

pub fn classify_with_config(
    command: &str,
    output_lines: &[String],
    is_agent: bool,
    config: Option<&Config>,
) -> ClassificationResult {
    if let Some(result) = check_needs_attention(output_lines) {
        return result;
    }

    if let Some(result) = check_error(output_lines) {
        return result;
    }

    if is_shell(command) && !is_agent {
        return ClassificationResult {
            status: AgentStatus::Done,
            reason: "returned to shell".to_string(),
            confidence: 0.78,
        };
    }

    if let Some(result) = check_done(output_lines) {
        return result;
    }

    if let Some(result) = check_running(output_lines, is_agent) {
        return result;
    }

    // User-supplied custom patterns (from config)
    if let Some(cfg) = config {
        if let Some(result) = check_custom_patterns(output_lines, cfg) {
            return result;
        }
    }

    ClassificationResult {
        status: AgentStatus::Unknown,
        reason: "no clear signal".to_string(),
        confidence: 0.40,
    }
}

fn check_custom_patterns(lines: &[String], config: &Config) -> Option<ClassificationResult> {
    let tail: String = lines.iter().rev().take(8).cloned().collect::<Vec<_>>().join("\n");

    for p in &config.attention_patterns {
        if matches_pattern(&tail, &p.pattern) {
            return Some(ClassificationResult {
                status: AgentStatus::NeedsAttention,
                reason: p.reason.clone(),
                confidence: 0.85,
            });
        }
    }

    for p in &config.done_patterns {
        if matches_pattern(&tail, &p.pattern) {
            return Some(ClassificationResult {
                status: AgentStatus::Done,
                reason: p.reason.clone(),
                confidence: 0.85,
            });
        }
    }

    None
}

fn check_needs_attention(lines: &[String]) -> Option<ClassificationResult> {
    let tail: String = lines.iter().rev().take(8).cloned().collect::<Vec<_>>().join("\n");

    let strong: &[(&str, &str)] = &[
        (r"\[y/N\]|\[Y/n\]|\[yes/no\]|\(y/n\)", "waiting for approval"),
        (r"(?i)press (enter|any key) to continue", "waiting for input"),
        (r"(?i)approve:", "needs approval"),
        (r"(?i)(permission (required|needed|denied))", "permission issue"),
        (r"(?i)blocked", "blocked"),
        (r"(?i)waiting for (your |human )?confirmation", "waiting for confirmation"),
        (r"(?i)do you want to .{3,60}\?", "approval needed"),
        (r"(?i)shall i |should i ", "approval needed"),
        (r"Continue\? \[", "approval needed"),
    ];

    for (pattern, reason) in strong {
        if matches_pattern(&tail, pattern) {
            return Some(ClassificationResult {
                status: AgentStatus::NeedsAttention,
                reason: reason.to_string(),
                confidence: 0.93,
            });
        }
    }

    // Extract the question text from the last non-empty line ending with '?'
    for line in lines.iter().rev().take(5) {
        let t = line.trim();
        if t.ends_with('?') && t.len() > 8 && t.len() < 120 {
            // Must look like a question, not a code comment
            let has_alpha = t.chars().filter(|c| c.is_alphabetic()).count();
            if has_alpha > 5 {
                return Some(ClassificationResult {
                    status: AgentStatus::NeedsAttention,
                    reason: truncate(t, 60),
                    confidence: 0.72,
                });
            }
        }
    }

    None
}

fn check_error(lines: &[String]) -> Option<ClassificationResult> {
    let tail: String = lines.iter().rev().take(12).cloned().collect::<Vec<_>>().join("\n");

    let patterns: &[(&str, &str)] = &[
        (r"thread '\w+' panicked", "panicked"),
        (r"(?i)^fatal:", "fatal error"),
        (r"(?i)^error\[", "compilation error"),
        (r"(?i)non-zero exit|exit status [1-9]", "non-zero exit"),
        (r"(?i)segmentation fault", "segfault"),
    ];

    for (pattern, reason) in patterns {
        if matches_pattern(&tail, pattern) {
            return Some(ClassificationResult {
                status: AgentStatus::Error,
                reason: reason.to_string(),
                confidence: 0.88,
            });
        }
    }

    None
}

fn check_done(lines: &[String]) -> Option<ClassificationResult> {
    let tail: String = lines.iter().rev().take(6).cloned().collect::<Vec<_>>().join("\n");

    let patterns: &[(&str, &str)] = &[
        (
            r"(?i)(task complete|all done|completed successfully)",
            "task complete",
        ),
        (r"(?i)changes (committed|pushed)", "changes committed"),
        (r"(?i)(pull request created|pr created|pr opened)", "PR created"),
        (r"(?i)goodbye|bye!", "agent exited"),
        (r"(?i)✓ done|done\.", "done"),
        (r"(?i)no (more )?changes needed", "no changes needed"),
    ];

    for (pattern, reason) in patterns {
        if matches_pattern(&tail, pattern) {
            return Some(ClassificationResult {
                status: AgentStatus::Done,
                reason: reason.to_string(),
                confidence: 0.86,
            });
        }
    }

    None
}

fn check_running(lines: &[String], is_agent: bool) -> Option<ClassificationResult> {
    if !is_agent {
        return None;
    }

    let tail: String = lines.iter().rev().take(6).cloned().collect::<Vec<_>>().join("\n");

    // Braille spinner characters used by many CLI tools
    let spinners = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏', '⣾', '⣽', '⣻', '⢿', '⡿'];
    for ch in &spinners {
        if tail.contains(*ch) {
            return Some(ClassificationResult {
                status: AgentStatus::Running,
                reason: "working".to_string(),
                confidence: 0.87,
            });
        }
    }

    let patterns: &[(&str, &str)] = &[
        (r"(?i)compiling|cargo build|cargo test", "building"),
        (r"(?i)\bediting\b|\bwriting\b", "editing"),
        (r"(?i)\bgenerating\b|\bcreating\b", "generating"),
        (r"(?i)\banalyzing\b|\bsearching\b|\breading\b", "analyzing"),
        (r"(?i)\binstalling\b|\bdownloading\b|\bfetching\b", "downloading"),
        (r"(?i)\brunning\b.{0,30}test", "running tests"),
        (r"(?i)\bthinking\b|\breasoning\b", "thinking"),
    ];

    for (pattern, reason) in patterns {
        if matches_pattern(&tail, pattern) {
            return Some(ClassificationResult {
                status: AgentStatus::Running,
                reason: reason.to_string(),
                confidence: 0.73,
            });
        }
    }

    if is_agent && !lines.is_empty() {
        return Some(ClassificationResult {
            status: AgentStatus::Running,
            reason: "agent active".to_string(),
            confidence: 0.58,
        });
    }

    None
}

pub fn is_known_agent(command: &str) -> bool {
    is_known_agent_with_extras(command, &[])
}

pub fn is_known_agent_with_extras(command: &str, extras: &[String]) -> bool {
    let cmd = command
        .split('/')
        .next_back()
        .unwrap_or(command)
        .to_lowercase();

    let builtin = matches!(
        cmd.as_str(),
        "claude"
            | "codex"
            | "aider"
            | "hermes"
            | "opencode"
            | "cursor"
            | "cline"
            | "continue"
            | "goose"
            | "amp"
            | "gemini"
    );

    builtin || extras.iter().any(|e| e.to_lowercase() == cmd)
}

pub fn is_shell(command: &str) -> bool {
    matches!(
        command.to_lowercase().as_str(),
        "bash" | "zsh" | "fish" | "sh" | "dash" | "ksh" | "tcsh" | "nu"
    )
}

fn matches_pattern(text: &str, pattern: &str) -> bool {
    Regex::new(pattern)
        .map(|re| re.is_match(text))
        .unwrap_or(false)
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let cut = s.char_indices()
            .take(max_len)
            .last()
            .map(|(i, _)| i + 1)
            .unwrap_or(max_len);
        format!("{}…", &s[..cut])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(input: &[&str]) -> Vec<String> {
        input.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn detects_yn_prompt() {
        let output = lines(&["✓ cargo build", "Do you want to run the tests? [y/N]"]);
        let result = classify("claude", &output, true);
        assert_eq!(result.status, AgentStatus::NeedsAttention);
        assert!(result.confidence > 0.8);
    }

    #[test]
    fn detects_question_prompt() {
        let output = lines(&["I found 3 issues.", "Should I fix them automatically?"]);
        let result = classify("claude", &output, true);
        assert_eq!(result.status, AgentStatus::NeedsAttention);
    }

    #[test]
    fn detects_panic() {
        let output = lines(&[
            "running tests",
            "thread 'main' panicked at 'called `unwrap()` on `Err`'",
        ]);
        let result = classify("claude", &output, true);
        assert_eq!(result.status, AgentStatus::Error);
    }

    #[test]
    fn detects_running_spinner() {
        let output = lines(&["⠋ Compiling..."]);
        let result = classify("claude", &output, true);
        assert_eq!(result.status, AgentStatus::Running);
    }

    #[test]
    fn shell_with_no_agent_is_done() {
        let output = lines(&["$ "]);
        let result = classify("zsh", &output, false);
        assert_eq!(result.status, AgentStatus::Done);
    }

    #[test]
    fn known_agents() {
        assert!(is_known_agent("claude"));
        assert!(is_known_agent("codex"));
        assert!(is_known_agent("aider"));
        assert!(!is_known_agent("bash"));
        assert!(!is_known_agent("vim"));
    }
}
