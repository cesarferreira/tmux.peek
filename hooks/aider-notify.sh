#!/usr/bin/env bash
# Aider notification adapter for tmux.peek
#
# Usage — wrap aider via tmux.peek:
#   tmux.peek wrap aider "$@"
#
# Or call this script from aider's --exec-before-commit / --exec-after hook:
#   aider --after-tool-cmd "$HOME/.config/tmux.peek/hooks/aider-notify.sh notification"

EVENT="${1:-notification}"
MESSAGE="${2:-}"

tmux.peek hook --agent aider --event "$EVENT" ${MESSAGE:+--message "$MESSAGE"}
