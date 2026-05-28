class Hindsight < Formula
  desc "20/20 vision for your shell history"
  homepage "https://github.com/Maoshan1/hindsight"
  url "https://github.com/Maoshan1/hindsight/archive/refs/tags/v0.1.1.tar.gz"
  sha256 "3d6e27306fb0d9a1e2cf217d4e85c874fb61c5da49a6d950c0e16f13c143e85c"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args

    pkgshare.install "hindsight.zsh"
    pkgshare.install "hindsight.bash"
  end

  def caveats
    <<~EOS
      To enable shell history recording, add to your ~/.zshrc:

        source #{opt_pkgshare}/hindsight.zsh

      Or for bash, add to ~/.bashrc:

        source #{opt_pkgshare}/hindsight.bash
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/hindsight --version")
  end
end
