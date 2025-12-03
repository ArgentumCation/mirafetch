class Mirafetch < Formula
  desc "A Rust reimplementation of Hyfetch wih a focus on speed"
  homepage "https://github.com/ArgentumCation/mirafetch"
  url "https://github.com/ArgentumCation/mirafetch/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "7b18e17093321e2801bc15bb913fafc8e4afc824357c909423952eb200c4fe79"
  license "EUPL-1.2"
  head "https://github.com/ArgentumCation/mirafetch.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "mirafetch"
  end
end
