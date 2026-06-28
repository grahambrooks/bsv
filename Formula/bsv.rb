# typed: false
# frozen_string_literal: true

# Homebrew formula for bsv, kept in-repo so this repository doubles as a tap.
#
# Install without tapping:
#   brew install https://raw.githubusercontent.com/grahambrooks/bsv/main/Formula/bsv.rb
#
# Or tap this repo and install by name:
#   brew tap grahambrooks/bsv https://github.com/grahambrooks/bsv
#   brew install bsv
#
# The version and sha256 values below are updated automatically by the release
# workflow (.github/workflows/release.yml) on each tagged release.
class Bsv < Formula
  desc "Backstage Entity Visualizer - TUI for exploring catalog-info.yaml files"
  homepage "https://github.com/grahambrooks/bsv"
  version "0.0.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/grahambrooks/bsv/releases/download/v#{version}/bsv-aarch64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/grahambrooks/bsv/releases/download/v#{version}/bsv-x86_64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/grahambrooks/bsv/releases/download/v#{version}/bsv-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  def install
    bin.install "bsv"
  end

  test do
    assert_match "bsv #{version}", shell_output("#{bin}/bsv --version")
  end
end
