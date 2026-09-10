class Kport < Formula
  desc "⚡ Ultra-fast CLI & TUI to hunt down and kill processes hogging your ports"
  homepage "https://github.com/shyam/port-killer"
  version "0.2.0"
  license "MIT"

  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/shyam/port-killer/releases/download/v0.2.0/port-killer-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MAC_ARM"
    else
      url "https://github.com/shyam/port-killer/releases/download/v0.2.0/port-killer-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MAC_X64"
    end
  elsif OS.linux?
    if Hardware::CPU.arm?
      url "https://github.com/shyam/port-killer/releases/download/v0.2.0/port-killer-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM"
    else
      url "https://github.com/shyam/port-killer/releases/download/v0.2.0/port-killer-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X64"
    end
  end

  def install
    bin.install "port-killer" => "port-killer"
    bin.install_symlink bin/"port-killer" => "kport"
  end

  test do
    system "#{bin}/port-killer", "--list"
  end
end
