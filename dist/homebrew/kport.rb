class Kport < Formula
  desc "⚡ Ultra-fast CLI & TUI to hunt down and kill processes hogging network ports"
  homepage "https://neuralshyam.github.io/kport/"
  version "0.2.1"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/neuralshyam/kport/releases/download/v0.2.1/kport-aarch64-apple-darwin.tar.gz"
      sha256 "18ecd1d497e6b49a37e4aface670c60eafde5e64dfc18e0cd0a6a4842355986e"
    else
      url "https://github.com/neuralshyam/kport/releases/download/v0.2.1/kport-x86_64-apple-darwin.tar.gz"
      sha256 "e872abd66b92dd75c646b08c9d02b29e1a20ac95a1696146e99e94eb73c12586"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/neuralshyam/kport/releases/download/v0.2.1/kport-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "3bf6a9066d67589f68834cd142d8ce3f95f59afdb9be702d79fc71d73df25105"
    else
      url "https://github.com/neuralshyam/kport/releases/download/v0.2.1/kport-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "fa4b6f13caa961a773e1dd48c28bf40c0953e929394efb12291247359be49d68"
    end
  end

  def install
    bin.install "kport"
    bin.install_symlink bin/"kport" => "port-killer"
  end

  test do
    system "#{bin}/kport", "--list"
  end
end
