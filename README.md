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

## What it is not

Not an orchestrator. Not a web dashboard. Not another agent runner. Read-only by default (except `kill`).
