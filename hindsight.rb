class Hindsight < Formula
  desc "20/20 vision for your shell history"
  homepage "https://github.com/Maoshan1/hindsight"
  url "https://github.com/Maoshan1/hindsight/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "8f481e8e3db9431fc06a8dd223de665a31a9289946d2db522e07d43aef431d70"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args

    # Install shell integration files
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
