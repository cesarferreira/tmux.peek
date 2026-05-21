# tmux.peek

> See which agent needs you before switching panes.

A read-only, zero-config Rust TUI that watches your coding agents in tmux and tells you which ones need your attention — without you having to switch panes.

```
╭─ tmux.peek ─────────────────────────────────── 6 agents · 2 need you ─╮
│                                                                         │
│  NEEDS ATTENTION                                                  [2]   │
│  ───────────────────────────────────────────────────────────────────   │
│  ▶ claude   stax        fix/undo-copy   03m   approve: run cargo test? │
│    codex    backend     feat/auth       11m   waiting: confirm patch    │
│                                                                         │
│  RUNNING                                                          [3]   │
│  ───────────────────────────────────────────────────────────────────   │
│    claude   byedroid    ui/logcat-filter  14s  editing src/filter.rs   │
│    codex    mobile      fix/nav-crash     41s  running cargo test       │
│    hermes   brain       –                 2m   writing daily report     │
│                                                                         │
│  DONE                                                             [1]   │
│  ───────────────────────────────────────────────────────────────────   │
│    claude   scripts     main            08m   task complete             │
│                                                                         │
╰─────────────────────────────────────────────────────────────────────── ╯
╭─ preview · stax · %12 ──────────────────────────────────────────────── ╮
│  ✓ clippy passed                                                         │
│  ✓ cargo build                                                           │
│                                                                         │
│  Do you want to run the tests? [y/N] █                                  │
╰─────────────────────────────────────────────────────────────────────── ╯
  enter:jump  p:preview  s:snapshot  k:kill  /:filter  1:attn  2:all  q:quit
```

## Install

```bash
# Install from source
git clone https://github.com/cesarferreira/peek
cd peek
bash install.sh
```

This builds `tmuxpeek` and creates a `tmux.peek` symlink in `~/.local/bin`.

Or with cargo:

```bash
cargo install tmux-peek
# Then create the symlink:
ln -sf ~/.cargo/bin/tmuxpeek ~/.cargo/bin/tmux.peek
```

## Commands

```bash
tmux.peek                   # open TUI dashboard (default)
tmux.peek list              # print table to stdout
tmux.peek status            # one-line summary for tmux status-right
tmux.peek snapshot          # pasteable Markdown summary
tmux.peek snapshot --json   # machine-readable JSON
tmux.peek watch             # continuously refresh list output
```

## Tmux integration

Add to `~/.tmux.conf`:

```tmux
# Side pane (48 cols)
bind-key G split-window -h -l 48 'tmux.peek tui --side-pane'

# Popup
bind-key g display-popup -E -w 90% -h 80% 'tmux.peek tui --popup'

# Status line
set -g status-right '#(tmux.peek status) | %H:%M'
```

Or use the plugin script:

```tmux
run-shell ~/.config/tmux/plugins/peek/tmux/peek.tmux
```

## Keyboard shortcuts

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `Enter` | Jump to agent pane |
| `p` | Toggle preview pane |
| `s` | Save snapshot to /tmp |
| `k` | Kill agent pane (with confirmation) |
| `/` | Filter by name / repo / branch |
| `1` | Show only agents needing attention |
| `2` | Show all agents |
| `r` / `F5` | Force refresh |
| `q` / `Esc` | Quit |

## Status classification

| Status | Signals |
|--------|---------|
| `NEEDS ATTENTION` | `[y/N]` prompts, approval requests, blocked |
| `RUNNING` | Spinner activity, keywords like "editing", "building" |
| `DONE` | Completion phrases, returned to shell |
| `ERROR` | Panic, non-zero exit, fatal errors |
| `UNKNOWN` | Heuristics inconclusive |

## Detected agents

`claude`, `codex`, `aider`, `hermes`, `opencode`, `gemini`, `goose`, `amp`, `cursor`, `cline`

## Reliable mode — `wrap`

For 100% reliable agent identity (useful when agents run via wrapper scripts):

```bash
# Instead of running claude directly, use:
tmux.peek wrap claude

# With arguments:
tmux.peek wrap claude -- --model sonnet task.md
```

The wrap command records the agent's identity in `~/.local/state/tmux.peek/wrapped/<pane>.json`. The scanner reads this file first, so identity is always correct regardless of the process tree.

## Agent hook integration

### Claude Code

Add to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "Stop": [{ "command": "tmux.peek hook --agent claude --event stop" }],
    "Notification": [{ "command": "tmux.peek hook --agent claude --event notification --message \"$CLAUDE_NOTIFICATION\"" }]
  }
}
```

See `hooks/claude-code-settings.json` for a ready-to-merge snippet.  
When hooks fire, the cache updates immediately with confidence 0.99 — no waiting for the 5s scan cycle.

### OpenCode / Aider

See `hooks/opencode-notify.sh` and `hooks/aider-notify.sh`. The simplest approach for any agent is just `tmux.peek wrap <agent>`.

## Configuration

```bash
tmux.peek config --init   # write default config to ~/.config/tmux.peek/config.toml
tmux.peek config          # show current settings
tmux.peek config --show   # print config file contents
```

Example `~/.config/tmux.peek/config.toml`:

```toml
extra_agents = ["my-internal-bot"]
exclude_sessions = ["scratch"]
status_format = "minimal"   # emoji | text | minimal

[[attention_patterns]]
pattern = "waiting for review"
reason  = "review needed"
```

## Shell completions

```bash
# zsh
tmux.peek completions zsh >> ~/.zshrc

# bash
tmux.peek completions bash >> ~/.bashrc

# fish
tmux.peek completions fish > ~/.config/fish/completions/tmux.peek.fish
```

## What it is not

Not an orchestrator. Not a web dashboard. Not another agent runner. Read-only by default (except `kill`).
