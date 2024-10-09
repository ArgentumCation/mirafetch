class Mirafetch < Formula
  desc "No description, website, or topics provided"
  homepage "https://github.com/ArgentumCation/mirafetch"
  license "EUPL-1.2"
  head "https://github.com/ArgentumCation/mirafetch"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system bin/"mirafetch"
  end
end
