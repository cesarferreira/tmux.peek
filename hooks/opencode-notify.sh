#!/usr/bin/env bash
# OpenCode notification adapter for tmux.peek
#
# Add to opencode config or call from your opencode wrapper:
#   export OPENCODE_ON_NOTIFY="$HOME/.config/tmux.peek/hooks/opencode-notify.sh"
#
# Or wrap opencode:
#   tmux.peek wrap opencode "$@"   ← preferred

EVENT="${1:-notification}"
MESSAGE="${2:-}"

tmux.peek hook --agent opencode --event "$EVENT" ${MESSAGE:+--message "$MESSAGE"}
