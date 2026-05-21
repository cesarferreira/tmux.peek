#!/usr/bin/env bash
# tmux.peek plugin script
# Source this from your tmux.conf or run it once to register keybindings.
#
# Usage:
#   run-shell ~/.config/tmux/plugins/peek/tmux/peek.tmux

CURRENT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Side pane: open a 48-column split on the right
tmux bind-key G split-window -h -l 48 'tmux.peek tui --side-pane'

# Popup: 90% wide, 80% tall
tmux bind-key g display-popup -E -w 90% -h 80% 'tmux.peek tui --popup'

# Status line (add to status-right in tmux.conf):
#   set -g status-right '#(tmux.peek status) | %H:%M'
