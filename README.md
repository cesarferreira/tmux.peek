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

  <img src="assets/recording.gif" width="880" alt="tmuxpeek in action">
</div>

---

## Why tmux.peek

When you're running multiple AI agents across tmux panes — Claude Code, OpenCode, Aider, Codex — you constantly switch between them to check if they're still working or waiting for input. **tmuxpeek** watches all of them and tells you which ones need your attention.

- **Attention at a glance.** Agents blocked on a question, permission prompt, or error rise to the top.
- **Live activity reasons.** See what each agent is actually doing: `edit main.rs`, `$ cargo build`, `Reviewed PR #360…`.
- **Hook-powered accuracy.** Claude Code's `PreToolUse` hook writes activity directly — no guessing from output.
- **ANSI preview pane.** Press `p` to see the agent's live terminal output, colors and all.
- **Jump instantly.** Press `Enter` to switch focus to the selected agent's tmux pane.
- **Zero config.** Detects Claude, Codex, Aider, OpenCode, Goose, Gemini, and more automatically.

## Install

```bash
git clone https://github.com/cesarferreira/tmuxpeek
cd tmuxpeek
bash install.sh
```

This builds a release binary and installs it to `~/.local/bin/tmuxpeek` with a `tmuxpeek` symlink. Make sure `~/.local/bin` is in your `PATH`.

Add to `~/.tmux.conf` to open as a side pane or status line widget:

```tmux
bind-key G split-window -h -l 48 'tmuxpeek tui'
set -g status-right '#(tmuxpeek status) | %H:%M'
```

<a id="quickstart"></a>
## Quickstart

Open the TUI inside your tmux session:

```bash
tmuxpeek
```

Check agent status in the terminal (no TUI):

```bash
tmuxpeek list
```

Take a JSON snapshot of all agent panes:

```bash
tmuxpeek snapshot
```

Watch continuously, refreshing every 5 seconds:

```bash
tmuxpeek watch
```

<a id="hooks"></a>
## Hooks

Hooks give tmuxpeek accurate, real-time activity data instead of relying on output parsing. When Claude Code calls a tool, it writes exactly what it's doing to tmuxpeek's state — `edit main.rs`, `$ cargo test`, `web: docs.rust-lang.org` — and the TUI picks it up immediately.

### Claude Code

Add to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "",
        "hooks": [{ "type": "command", "command": "tmuxpeek hook --agent claude --event pre_tool_use", "timeout": 5 }]
      }
    ],
    "Stop": [
      {
        "matcher": "",
        "hooks": [{ "type": "command", "command": "tmuxpeek hook --agent claude --event stop" }]
      }
    ],
    "Notification": [
      {
        "matcher": "",
        "hooks": [{ "type": "command", "command": "tmuxpeek hook --agent claude --event notification --message \"$CLAUDE_NOTIFICATION\"" }]
      }
    ]
  }
}
```

`PreToolUse` fires on every tool call and updates the pane's activity reason. `Stop` marks the pane Done immediately. `Notification` marks it as needing attention.

### Wrap command (any agent)

For agents without hook support, wrap them so tmuxpeek knows when they start and stop:

```bash
tmuxpeek wrap aider -- aider --model gpt-4o
```

<a id="configuration"></a>
## Configuration

Run `tmuxpeek config --init` to create `~/.config/tmuxpeek/config.toml` with all options documented. The main knobs:

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
tmuxpeek completions bash >> ~/.bashrc

# Zsh
tmuxpeek completions zsh >> ~/.zshrc

# Fish
tmuxpeek completions fish > ~/.config/fish/completions/tmuxpeek.fish
```

## Development

```bash
cargo build
cargo test
```

## License

MIT &copy; Cesar Ferreira
