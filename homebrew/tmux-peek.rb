class TmuxPeek < Formula
  desc "Read-only TUI that watches AI coding agents in tmux and shows which need attention"
  homepage "https://github.com/cesarferreira/peek"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/cesarferreira/peek/releases/download/v#{version}/tmuxpeek-aarch64-apple-darwin.tar.gz"
      # sha256 "<update after release>"
    end
    on_intel do
      url "https://github.com/cesarferreira/peek/releases/download/v#{version}/tmuxpeek-x86_64-apple-darwin.tar.gz"
      # sha256 "<update after release>"
    end
  end

  on_linux do
    url "https://github.com/cesarferreira/peek/releases/download/v#{version}/tmuxpeek-x86_64-unknown-linux-gnu.tar.gz"
    # sha256 "<update after release>"
  end

  def install
    bin.install "tmuxpeek"
    # Create the tmux.peek symlink (dots in names are valid on Unix)
    bin.install_symlink "#{bin}/tmuxpeek" => "tmux.peek"
  end

  def caveats
    <<~EOS
      Add to ~/.tmux.conf:
        bind-key G split-window -h -l 48 'tmux.peek tui --side-pane'
        set -g status-right '#(tmux.peek status) | %H:%M'

      For shell completions:
        # zsh
        tmux.peek completions zsh >> ~/.zshrc

        # bash
        tmux.peek completions bash >> ~/.bashrc
    EOS
  end

  test do
    system "#{bin}/tmuxpeek", "--version"
    system "#{bin}/tmux.peek", "--version"
  end
end
