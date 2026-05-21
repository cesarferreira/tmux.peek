#!/usr/bin/env bash
# Build and install tmuxpeek with a tmux.peek symlink
set -e

INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

echo "Building release binary..."
cargo build --release

mkdir -p "$INSTALL_DIR"
cp target/release/tmuxpeek "$INSTALL_DIR/tmuxpeek"

# Create the tmux.peek symlink (dots in filenames are valid on Unix)
ln -sf "$INSTALL_DIR/tmuxpeek" "$INSTALL_DIR/tmux.peek"

echo "Installed:"
echo "  $INSTALL_DIR/tmuxpeek"
echo "  $INSTALL_DIR/tmux.peek  (symlink)"
echo ""
echo "Make sure $INSTALL_DIR is in your PATH."
echo ""
echo "Add to ~/.tmux.conf:"
echo "  bind-key G split-window -h -l 48 'tmux.peek tui --side-pane'"
echo "  set -g status-right '#(tmux.peek status) | %H:%M'"
