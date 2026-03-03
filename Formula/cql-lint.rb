# typed: false
# frozen_string_literal: true

# This formula is a template for hiboma/homebrew-tap.
# To publish, copy this file to the tap repository and update
# the version, url, and sha256 values.
#
# Usage:
#   brew tap hiboma/tap
#   brew install cql-lint
class CqlLint < Formula
  desc "A linter for CrowdStrike LogScale query language"
  homepage "https://github.com/hiboma/cql-lint"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/hiboma/cql-lint/releases/download/v#{version}/cql-lint-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    end

    on_intel do
      url "https://github.com/hiboma/cql-lint/releases/download/v#{version}/cql-lint-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/hiboma/cql-lint/releases/download/v#{version}/cql-lint-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    end
  end

  def install
    bin.install "cql-lint"
  end

  test do
    output = shell_output("echo 'count()' | #{bin}/cql-lint --verbose")
    assert_match "no issues found", output
  end
end
