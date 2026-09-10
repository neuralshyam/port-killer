class Kport < Formula
  desc "⚡ Ultra-fast CLI & TUI to hunt down and kill processes hogging network ports"
  homepage "https://neuralshyam.github.io/kport/"
  version "0.2.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/neuralshyam/kport/releases/download/v0.2.0/port-killer-aarch64-apple-darwin.tar.gz"
      sha256 "0f661e53b9b613197a3a8716b1dc95171e21b8b201a4e107dfca0cf67fcb3602"
    else
      url "https://github.com/neuralshyam/kport/releases/download/v0.2.0/port-killer-x86_64-apple-darwin.tar.gz"
      sha256 "95edfe6a787b8c0496328f95c478a87b7a63ce73f8d689622d148e658ae24068"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/neuralshyam/kport/releases/download/v0.2.0/port-killer-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "66410f0d41a41a6360e74ce1cf4aa3fdb5be7ff8e096ff8be94f4d2f8319f3a6"
    else
      url "https://github.com/neuralshyam/kport/releases/download/v0.2.0/port-killer-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "32b986900b96cb35c65c342f0d9c490a02ef495fa0451a24896fe2225e365cb9"
    end
  end

  def install
    bin.install "port-killer" => "kport"
    bin.install_symlink bin/"kport" => "port-killer"
  end

  test do
    system "#{bin}/kport", "--list"
  end
end
