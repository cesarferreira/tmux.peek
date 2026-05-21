#!/usr/bin/env bash
set -e

INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

echo "Building release binary..."
cargo build --release

mkdir -p "$INSTALL_DIR"
cp target/release/tmux-peek "$INSTALL_DIR/tmux-peek"

echo "Installed: $INSTALL_DIR/tmux-peek"
echo ""
echo "Make sure $INSTALL_DIR is in your PATH."
echo ""
echo "Add to ~/.tmux.conf:"
echo "  bind-key G split-window -h -l 48 'tmux-peek tui'"
echo "  set -g status-right '#(tmux-peek status) | %H:%M'"
