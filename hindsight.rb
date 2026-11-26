class Hindsight < Formula
  desc "20/20 vision for your shell history"
  homepage "https://github.com/Maoshan1/hindsight"
  url "https://github.com/Maoshan1/hindsight/archive/refs/tags/v0.2.0.tar.gz"
  sha256 "d8d969ff813575dec005dc49a8910f896bffd4703995c2f9f26ec93bfb2e97cc"
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
