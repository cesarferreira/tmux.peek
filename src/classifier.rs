use std::sync::OnceLock;

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
        // Use the last meaningful output line as the reason instead of "agent active"
        let last_line = last_meaningful_line(lines).unwrap_or_else(|| "agent active".to_string());
        return Some(ClassificationResult {
            status: AgentStatus::Running,
            reason: last_line,
            confidence: 0.58,
        });
    }

    None
}

/// Extract the best available activity description from pane output.
fn last_meaningful_line(lines: &[String]) -> Option<String> {
    // Try agent-specific extractors first (they produce clean, structured reasons)
    if let Some(r) = extract_claude_code_activity(lines) {
        return Some(r);
    }
    if let Some(r) = extract_opencode_activity(lines) {
        return Some(r);
    }

    // Generic fallback: scan up to 40 lines in reverse, skip TUI chrome and bare path fragments
    lines
        .iter()
        .rev()
        .take(40)
        .map(|l| l.trim())
        .find(|t| {
            !is_tui_chrome(t)
                && t.chars().filter(|c| c.is_alphabetic()).count() >= 4
        })
        .map(|t| truncate(t, 55))
}

fn is_tui_chrome(line: &str) -> bool {
    let t = line.trim();
    if t.is_empty() || t.len() < 3 {
        return true;
    }
    // Separator / drawing lines (horizontal rules, box-drawing, block fill)
    let non_drawing = t.chars().filter(|c| {
        !matches!(*c,
            '-' | '=' | ' ' | '·' | '•'
            | '─' | '━' | '╌' | '┄' | '╍' | '┅'
            | '▀' | '▄' | '█' | '░' | '▒' | '▓'
            | '╹' | '╸' | '╺' | '╻' | '┃' | '│' | '┤' | '├'
        )
    }).count();
    if non_drawing == 0 { return true; }

    // Lines where ≥80 % of chars are block/drawing characters → TUI frame
    let total = t.chars().count();
    let drawing = total - non_drawing;
    if total > 4 && drawing * 100 / total >= 80 { return true; }

    // Claude Code TUI chrome
    if t.contains("bypass permissions on") { return true; }
    if t.contains("shift+tab to cycle") { return true; }
    if t.contains("autoAcceptEdits") { return true; }
    if t.starts_with(">>") && t.contains("mode") { return true; }
    if t.starts_with("cwd:") { return true; }
    if t.starts_with("No MCP servers") { return true; }
    if t.contains("tokens left") { return true; }
    // Claude Code bottom status bar
    if t.starts_with("Model:") { return true; }
    if t.contains("Cost: $") { return true; }
    if t.contains("Ctx:") && t.contains("Cost:") { return true; }
    // Claude Code tool result headers — useful only via extract_claude_code_activity,
    // not for the generic fallback which would pick up random prose from these lines
    if t.starts_with("⏺") { return true; }
    // Word-wrapped path continuation fragments (e.g. "/config)" from a long recap line)
    if t.starts_with('/') && !t.contains(' ') { return true; }

    // OpenCode TUI chrome — status bar fragments
    if t.contains("ctrl+p commands") { return true; }
    if t.contains("• OpenCode") { return true; }
    if Regex::new(r"^\d+\.\d+K \(\d+%\)").unwrap().is_match(t) { return true; }

    // Bare prompts or prompt+typed input ("❯ create a pr", "› some cmd")
    if matches!(t, "$" | "%" | "#" | ">" | "❯" | "›") {
        return true;
    }
    if t.starts_with("❯ ") || t.starts_with("› ") {
        return true;
    }

    false
}

/// Extract a clean activity string from Claude Code's pane output.
fn extract_claude_code_activity(lines: &[String]) -> Option<String> {
    static BULLET_RE: OnceLock<Regex> = OnceLock::new();
    static TOOL_RE: OnceLock<Regex> = OnceLock::new();
    static COGITATE_RE: OnceLock<Regex> = OnceLock::new();

    // Claude Code uses ⏺ (U+23FA) as tool header indicator
    let bullet_re = BULLET_RE.get_or_init(|| {
        Regex::new(r"⏺\s+(Read|Write|Edit|Update|Bash|Create|Delete|List|Search|Glob|Grep|Task|Agent|TodoWrite|TodoRead)\((.{0,60})\)").unwrap()
    });
    // Start-of-line only — prevents matching conversational prose mid-sentence
    let tool_re = TOOL_RE.get_or_init(|| {
        Regex::new(r"(?i)^(?:running|editing|reading|writing|creating|deleting)\s+\S.{0,50}$").unwrap()
    });
    // "✻ Cogitated for 18s" — thinking phase indicator
    let cogitate_re = COGITATE_RE.get_or_init(|| {
        Regex::new(r"✻\s+Cogitated for (.+)").unwrap()
    });

    for line in lines.iter().rev().take(40) {
        let t = line.trim();

        // ⏺ tool headers — skip is_tui_chrome since these are filtered there but useful here
        if let Some(cap) = bullet_re.captures(t) {
            let arg = truncate(cap[2].trim(), 40);
            return Some(format!("{} {}", cap[1].to_lowercase(), arg));
        }
        // ※ recap: <summary text> — extract the summary
        if let Some(rest) = t.strip_prefix("※ recap:") {
            let summary = rest.trim();
            if !summary.is_empty() {
                return Some(truncate(summary, 55));
            }
        }
        // ✻ Cogitated for Xs
        if let Some(cap) = cogitate_re.captures(t) {
            return Some(format!("thinking ({})", cap[1].trim()));
        }

        if is_tui_chrome(t) { continue; }

        if let Some(m) = tool_re.find(t) {
            return Some(truncate(m.as_str(), 55));
        }
        if t.starts_with("$ ") && t.len() > 4 {
            return Some(truncate(&format!("running: {}", &t[2..]), 55));
        }
    }
    None
}

/// Extract a clean activity string from OpenCode's pane output.
fn extract_opencode_activity(lines: &[String]) -> Option<String> {
    static FILE_RE: OnceLock<Regex> = OnceLock::new();
    static TASK_RE: OnceLock<Regex> = OnceLock::new();

    // "some/file.ext  119.9K (30%)  ctrl+p commands" → "editing file.ext"
    let file_re = FILE_RE.get_or_init(|| {
        Regex::new(r"^(\S+\.\w+)\s+\d+\.\d+[KMG]").unwrap()
    });

    // "▣  Build · GPT-5.5 Fast · 2m 9s"  or  "┃  Build · GPT-5.5 Fast OpenAI · high"
    // → first dot-segment is the task name
    let task_re = TASK_RE.get_or_init(|| {
        Regex::new(r"[▣┃►]\s+([^·\n]{2,30})\s*·").unwrap()
    });

    for line in lines.iter().rev().take(15) {
        let t = line.trim();
        if let Some(cap) = file_re.captures(t) {
            return Some(format!("editing {}", &cap[1]));
        }
        if let Some(cap) = task_re.captures(t) {
            let task = cap[1].trim();
            // Attach timing if present: "2m 9s" or "42s"
            let timing = Regex::new(r"\b(\d+m \d+s|\d+s)\b").ok()
                .and_then(|re| re.find(t).map(|m| format!(" ({})", m.as_str())));
            return Some(format!("{}{}", task, timing.unwrap_or_default()));
        }
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
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", chars[..max_len.saturating_sub(1)].iter().collect::<String>())
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
