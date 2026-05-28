class Hindsight < Formula
  desc "20/20 vision for your shell history"
  homepage "https://github.com/yourname/hindsight"
  url "https://github.com/yourname/hindsight/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args

    # Install shell integration files
    zsh.install "hindsight.zsh"
    bash.install "hindsight.bash"
  end

  def caveats
    <<~EOS
      To enable shell history recording, add to your ~/.zshrc:

        source #{zsh}/hindsight.zsh

      Or for bash, add to ~/.bashrc:

        source #{bash}/hindsight.bash
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/hindsight --version")
  end
end
