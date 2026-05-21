<div align="center">
  <h1>tmux.peek</h1>

  <p><strong>See which AI coding agents need you — without switching panes.</strong></p>

  <p>
    <img alt="License" src="https://img.shields.io/badge/license-MIT-green">
    <img alt="Built with Rust" src="https://img.shields.io/badge/built%20with-Rust-orange">
  </p>

  <p>
    <a href="#install">Install</a>
    &nbsp;·&nbsp;
    <a href="#quickstart">Quickstart</a>
    &nbsp;·&nbsp;
    <a href="#hooks">Hooks</a>
    &nbsp;·&nbsp;
    <a href="#configuration">Configuration</a>
  </p>

  <br>

  <img src="assets/recording.gif" width="880" alt="tmux-peek in action">
</div>

---

## Why tmux.peek

When you're running multiple AI agents across tmux panes — Claude Code, OpenCode, Aider, Codex — you constantly switch between them to check if they're still working or waiting for input. **tmux-peek** watches all of them and tells you which ones need your attention.

- **Attention at a glance.** Agents blocked on a question, permission prompt, or error rise to the top.
- **Live activity reasons.** See what each agent is actually doing: `edit main.rs`, `$ cargo build`, `Reviewed PR #360…`.
- **Hook-powered accuracy.** Claude Code's `PreToolUse` hook writes activity directly — no guessing from output.
- **ANSI preview pane.** Press `p` to see the agent's live terminal output, colors and all.
- **Jump instantly.** Press `Enter` to switch focus to the selected agent's tmux pane.
- **Zero config.** Detects Claude, Codex, Aider, OpenCode, Goose, Gemini, and more automatically.

## Install

```bash
git clone https://github.com/cesarferreira/tmux-peek
cd tmux-peek
bash install.sh
```

This builds a release binary and installs it to `~/.local/bin/tmux-peek` with a `tmux-peek` symlink. Make sure `~/.local/bin` is in your `PATH`.

Add to `~/.tmux.conf` to open as a side pane or status line widget:

```tmux
bind-key G split-window -h -l 48 'tmux-peek tui'
set -g status-right '#(tmux-peek status) | %H:%M'
```

<a id="quickstart"></a>
## Quickstart

Open the TUI inside your tmux session:

```bash
tmux-peek
```

Check agent status in the terminal (no TUI):

```bash
tmux-peek list
```

Take a JSON snapshot of all agent panes:

```bash
tmux-peek snapshot
```

Watch continuously, refreshing every 5 seconds:

```bash
tmux-peek watch
```

<a id="hooks"></a>
## Hooks

Hooks give tmux-peek accurate, real-time activity data instead of relying on output parsing. When Claude Code calls a tool, it writes exactly what it's doing to tmux-peek's state — `edit main.rs`, `$ cargo test`, `web: docs.rust-lang.org` — and the TUI picks it up immediately.

### Claude Code

Add to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "",
        "hooks": [{ "type": "command", "command": "tmux-peek hook --agent claude --event pre_tool_use", "timeout": 5 }]
      }
    ],
    "Stop": [
      {
        "matcher": "",
        "hooks": [{ "type": "command", "command": "tmux-peek hook --agent claude --event stop" }]
      }
    ],
    "Notification": [
      {
        "matcher": "",
        "hooks": [{ "type": "command", "command": "tmux-peek hook --agent claude --event notification --message \"$CLAUDE_NOTIFICATION\"" }]
      }
    ]
  }
}
```

`PreToolUse` fires on every tool call and updates the pane's activity reason. `Stop` marks the pane Done immediately. `Notification` marks it as needing attention.

### Wrap command (any agent)

For agents without hook support, wrap them so tmux-peek knows when they start and stop:

```bash
tmux-peek wrap aider -- aider --model gpt-4o
```

<a id="configuration"></a>
## Configuration

Run `tmux-peek config --init` to create `~/.config/tmux-peek/config.toml` with all options documented. The main knobs:

```toml
# Add custom agent binaries to detect
extra_agents = ["my-agent", "llm-runner"]

# Only watch specific tmux sessions
session_filter = ["work", "side-project"]

# Custom patterns that mark a pane as needing attention
[[attention_patterns]]
pattern = "(?i)awaiting review"
reason  = "needs review"

# Custom patterns that mark a pane as done
[[done_patterns]]
pattern = "(?i)deployment complete"
reason  = "deployed"
```

## Controls

| Key | Action |
|---|---|
| `↑ / ↓` or `j / k` | Move selection |
| `Enter` | Jump to agent's tmux pane |
| `p` | Toggle preview pane |
| `s` | Save JSON snapshot |
| `k` | Kill agent pane (with confirmation) |
| `/` | Filter by name |
| `1` | Toggle attention-only view |
| `r` | Force refresh |
| `q` | Quit |

## Status

| Status | Meaning |
|---|---|
| `NEEDS ATTENTION` | Agent is asking a question or waiting for input |
| `RUNNING` | Agent is actively working |
| `DONE` | Agent finished or returned to shell |
| `ERROR` | Crash, panic, or non-zero exit |
| `UNKNOWN` | No clear signal yet |

## Shell Completions

```bash
# Bash
tmux-peek completions bash >> ~/.bashrc

# Zsh
tmux-peek completions zsh >> ~/.zshrc

# Fish
tmux-peek completions fish > ~/.config/fish/completions/tmux-peek.fish
```

## Development

```bash
cargo build
cargo test
```

## License

MIT &copy; Cesar Ferreira
