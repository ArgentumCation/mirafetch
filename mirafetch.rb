class Mirafetch < Formula
  # TODO: add desc
  desc "No description, website, or topics provided"
  homepage "https://github.com/ArgentumCation/mirafetch"
  url "https://github.com/ArgentumCation/mirafetch/archive/refs/heads/main.tar.gz"
  version "0.0.0"
  # TODO: add sha256 for 0.0.1
  # sha256 "66bf24d6876f1460acb6b817450620c4ea9f095c95a7c4e38d4bea1729229728"
  license "EUPL-1.2"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  # TODO: better test
  test do
    system "#{bin}/mirafetch"
  end
end
