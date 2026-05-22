#!/usr/bin/env bash
# bump-version.sh — manual SemVer bump for Software-Version and/or Document-Version.
#
# Usage:
#   ./scripts/bump-version.sh <level> [target]
#
# Levels:
#   major   X.Y.Z → (X+1).0.0
#   minor   X.Y.Z → X.(Y+1).0
#   patch   X.Y.Z → X.Y.(Z+1)
#
# Targets:
#   --software   only bump Cargo.toml [workspace.package].version
#                (mirrored into VERSION.md Software-Version)
#   --document   only bump VERSION.md Document-Version
#   --both       (default) bump both
#
# Notes:
#   - Patch-level edits are also auto-bumped on every Claude Code Write/Edit by
#     .claude/hooks/auto-version.sh. Use this script when you want an explicit
#     MAJOR or MINOR jump (e.g. on a release boundary).
#   - Always updates VERSION.md Last-Updated stamp.
#   - This script edits files via shell, not via Claude's Write/Edit tools, so
#     the auto-version hook does NOT fire and re-bump on top of you.

set -euo pipefail

usage() {
  cat <<EOF
Usage: $0 <major|minor|patch> [--software|--document|--both]
EOF
  exit 2
}

[ $# -lt 1 ] && usage

level="$1"
target="${2:---both}"

case "$level" in
  major|minor|patch) ;;
  -h|--help) usage ;;
  *) echo "error: unknown level '$level' (expected major|minor|patch)" >&2; usage ;;
esac

case "$target" in
  --software|--document|--both) ;;
  *) echo "error: unknown target '$target' (expected --software|--document|--both)" >&2; usage ;;
esac

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cargo_toml="$repo_root/Cargo.toml"
version_md="$repo_root/VERSION.md"

bump() {
  local cur="$1" lvl="$2"
  local maj min pat
  IFS='.' read -r maj min pat <<<"$cur"
  case "$lvl" in
    major) printf '%d.0.0' $((maj + 1)) ;;
    minor) printf '%d.%d.0' "$maj" $((min + 1)) ;;
    patch) printf '%d.%d.%d' "$maj" "$min" $((pat + 1)) ;;
  esac
}

# Read current versions.
sw_cur=$(grep -E '^version = "[0-9]+\.[0-9]+\.[0-9]+"' "$cargo_toml" | head -1 \
         | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || true)
doc_cur=$(grep -E '^\*\*Document-Version:\*\*[[:space:]]+`[0-9]+\.[0-9]+\.[0-9]+`' "$version_md" \
          | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || true)

if [ -z "$sw_cur" ]; then
  echo "error: could not parse current Software-Version from $cargo_toml" >&2
  exit 1
fi
if [ -z "$doc_cur" ]; then
  echo "error: could not parse current Document-Version from $version_md" >&2
  exit 1
fi

sw_new="$sw_cur"
doc_new="$doc_cur"

# Compute new values where requested.
if [ "$target" = "--software" ] || [ "$target" = "--both" ]; then
  sw_new=$(bump "$sw_cur" "$level")
fi
if [ "$target" = "--document" ] || [ "$target" = "--both" ]; then
  doc_new=$(bump "$doc_cur" "$level")
fi

# Apply: write Cargo.toml + VERSION.md atomically (sed -i.bak for portability).
replace_version_line() {
  local file="$1" field="$2" new="$3"
  sed -i.bak -E "s|^(\*\*${field}:\*\*[[:space:]]+\`)[0-9]+\.[0-9]+\.[0-9]+(\`.*)$|\1${new}\2|" "$file"
  rm -f "${file}.bak"
}

if [ "$sw_new" != "$sw_cur" ]; then
  # Cargo.toml workspace version. Anchored regex matches only the [workspace.package] line.
  sed -i.bak -E "s|^version = \"${sw_cur//./\\.}\"$|version = \"${sw_new}\"|" "$cargo_toml"
  rm -f "${cargo_toml}.bak"
  replace_version_line "$version_md" "Software-Version" "$sw_new"
fi

if [ "$doc_new" != "$doc_cur" ]; then
  replace_version_line "$version_md" "Document-Version" "$doc_new"
fi

# Last-Updated stamp (always refreshed when anything bumped).
if [ "$sw_new" != "$sw_cur" ] || [ "$doc_new" != "$doc_cur" ]; then
  now=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  sed -i.bak -E "s|^(\*\*Last-Updated:\*\*[[:space:]]+\`)[^\`]*(\`.*)$|\1${now}\2|" "$version_md"
  rm -f "${version_md}.bak"
fi

# Report.
echo ""
echo "Bump level: $level   target: $target"
if [ "$sw_new" != "$sw_cur" ]; then
  echo "  Software-Version:  $sw_cur → $sw_new"
else
  echo "  Software-Version:  $sw_cur (unchanged)"
fi
if [ "$doc_new" != "$doc_cur" ]; then
  echo "  Document-Version:  $doc_cur → $doc_new"
else
  echo "  Document-Version:  $doc_cur (unchanged)"
fi
echo ""
echo "Next: review the diff and commit (e.g. \`git commit -m 'chore(release): vX.Y.Z'\`)."
