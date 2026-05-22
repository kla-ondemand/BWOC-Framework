#!/usr/bin/env bash
# install.sh — one-command install (or upgrade) of the bwoc CLI.
#
# Builds and installs the `bwoc` binary to ~/.cargo/bin/ (must be on PATH).
# The agent template is embedded in the binary at compile time, so the
# installed `bwoc` works from any directory — no on-disk template required.
#
# Requires: a Rust toolchain (https://rustup.rs/).
#
# Usage (from a clone of bwoc-framwork):
#   ./scripts/install.sh           # install (or upgrade in place — always uses --force)
#
# Or one-liner from the repo root:
#   cargo install --path crates/bwoc-cli --locked --force
#
# Related:
#   ./scripts/bump-version.sh      # manual major/minor/patch SemVer bumps
#                                  # (patch is also auto-bumped on every edit by
#                                  # .claude/hooks/auto-version.sh)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: 'cargo' not found on PATH. Install Rust from https://rustup.rs/ first." >&2
  exit 1
fi

# Detect existing install so we can phrase the message accurately.
existing=""
if command -v bwoc >/dev/null 2>&1; then
  existing="$(bwoc --version 2>/dev/null | head -1 || true)"
fi

if [ -n "$existing" ]; then
  echo "Upgrading bwoc in place (was: $existing) from $REPO_ROOT/crates/bwoc-cli ..."
else
  echo "Installing bwoc from $REPO_ROOT/crates/bwoc-cli ..."
fi

# --force allows re-running this script to upgrade an existing install
# without manual `cargo uninstall` first.
cargo install --path crates/bwoc-cli --locked --force

echo ""
new_version="$(bwoc --version 2>/dev/null | head -1 || echo 'bwoc')"
echo "Installed: $new_version"
echo ""
echo "Verify with:"
echo "  bwoc --help"
echo ""
echo "If 'bwoc' is not found, add ~/.cargo/bin to your PATH."
