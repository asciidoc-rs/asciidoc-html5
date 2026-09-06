#!/bin/bash
set -euo pipefail

# Claude Code on the web only: local checkouts manage their own toolchains.
if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

# The container image can lag behind the latest stable Rust, while CI runs
# clippy and the test matrix on latest stable — a lint that exists only in
# the newer clippy then fails CI after passing locally. Update so gate runs
# inside the session match CI. Idempotent, and a no-op once the cached
# container is current.
rustup update stable --no-self-update

# Nightly rustfmt is the formatting gate (see CLAUDE.md); the image ships
# nightly, so just make sure it's present after a cache refresh.
rustup toolchain list | grep -q nightly || rustup toolchain install nightly --profile minimal --component rustfmt --no-self-update
