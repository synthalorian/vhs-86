class Vhs86 < Formula
  desc "Retro terminal file manager with synthwave aesthetics"
  homepage "https://github.com/synthalorian/vhs-86"
  url "https://github.com/synthalorian/vhs-86/archive/v0.9.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "MIT"
  head "https://github.com/synthalorian/vhs-86.git", branch: "main"

  depends_on "rust" => :build
  depends_on "pkg-config" => :build
  depends_on "libgit2"
  depends_on "openssl"

  def install
    system "cargo", "install", *std_cargo_args

    # Install man page
    man1.install "man/vhs-86.1.md"
    
    # Install shell completions (if generated)
    # bash_completion.install "completions/vhs-86.bash"
    # zsh_completion.install "completions/_vhs-86"
    # fish_completion.install "completions/vhs-86.fish"
  end

  test do
    # Test that the binary runs and shows version
    assert_match "vhs-86 #{version}", shell_output("#{bin}/vhs-86 --version")
    
    # Test that help is accessible
    assert_match "A retro terminal file manager", shell_output("#{bin}/vhs-86 --help")
  end
end
