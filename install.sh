#!/usr/bin/env bash
set -e

INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

echo "Building release binary..."
cargo build --release

mkdir -p "$INSTALL_DIR"
cp target/release/tmuxpeek "$INSTALL_DIR/tmuxpeek"

echo "Installed: $INSTALL_DIR/tmuxpeek"
echo ""
echo "Make sure $INSTALL_DIR is in your PATH."
echo ""
echo "Add to ~/.tmux.conf:"
echo "  bind-key G split-window -h -l 48 'tmuxpeek tui'"
echo "  set -g status-right '#(tmuxpeek status) | %H:%M'"
