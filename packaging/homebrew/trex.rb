class Trex < Formula
  desc "Tmux Restore Extreme - persist and restore tmux sessions"
  homepage "https://github.com/tahasadough/trex"
  license "MIT"
  version "0.7.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/tahasadough/trex/releases/download/v#{version}/trex-aarch64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    else
      url "https://github.com/tahasadough/trex/releases/download/v#{version}/trex-x86_64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    if Hardware::CPU.arm? && Hardware::CPU.is_64_bit?
      url "https://github.com/tahasadough/trex/releases/download/v#{version}/trex-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    else
      url "https://github.com/tahasadough/trex/releases/download/v#{version}/trex-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "tmux"

  def install
    bin.install "trex"
    man1.install "trex.1"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/trex --version")
  end
end
