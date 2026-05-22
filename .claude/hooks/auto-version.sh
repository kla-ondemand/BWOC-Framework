#!/usr/bin/env bash
# auto-version.sh — PostToolUse Write|Edit hook
#
# On every Claude Code Write|Edit:
# - If the edited file is *.rs / *.toml under a crate or Cargo workspace,
#   bump the PATCH component of [workspace.package].version in Cargo.toml,
#   and mirror it into VERSION.md's Software-Version line.
# - If the edited file is *.md (anywhere except the auto-managed files
#   themselves), bump the PATCH component of VERSION.md's Document-Version line.
# - Always update VERSION.md's Last-Updated stamp to current UTC ISO 8601.
#
# Self-modification guard: edits to Cargo.toml, Cargo.lock, VERSION.md, or
# anything under .claude/ are ignored. The hook itself does not go through
# Claude's Write|Edit, so its own edits do not retrigger.

set -euo pipefail

f=$(jq -r '.tool_input.file_path // empty')
[[ -z "$f" ]] && exit 0

# Compute repo root (this script lives at <root>/.claude/hooks/)
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
rel="${f#$repo_root/}"

# Guard: file must be inside the repo. If parameter substitution didn't
# strip the prefix, `rel` still starts with `/` and `$f` is an out-of-repo
# path (e.g., ~/.claude/projects/.../memory/*.md edits during the session).
case "$rel" in
  /*) exit 0 ;;
esac

# Guard: ignore self-managed files and Claude infrastructure
case "$rel" in
  Cargo.toml|Cargo.lock|VERSION.md|.claude/*|target/*)
    exit 0
    ;;
esac

now=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

bump_patch() {
  local cur="$1"
  local maj min pat
  IFS='.' read -r maj min pat <<<"$cur"
  echo "$maj.$min.$((pat + 1))"
}

# Update a line of the form: **Field:** `X.Y.Z`   *(optional trailing)*
# Portable sed across BSD (macOS) and GNU (Linux): use -i with .bak suffix.
replace_version_line() {
  local file="$1" field="$2" new="$3"
  sed -i.bak -E "s|^(\*\*${field}:\*\*[[:space:]]+\`)[0-9]+\.[0-9]+\.[0-9]+(\`.*)$|\1${new}\2|" "$file"
  rm -f "${file}.bak"
}

# Update the **Last-Updated:** line in VERSION.md
replace_timestamp_line() {
  local file="$1" ts="$2"
  sed -i.bak -E "s|^(\*\*Last-Updated:\*\*[[:space:]]+\`)[^\`]*(\`.*)$|\1${ts}\2|" "$file"
  rm -f "${file}.bak"
}

domain=""
new_version=""

# --- Software domain: Rust source or any TOML / Cargo file ---
case "$rel" in
  crates/*|*.rs|*.toml)
    cargo="$repo_root/Cargo.toml"
    if [[ -f "$cargo" ]]; then
      cur=$(grep -E '^version = "[0-9]+\.[0-9]+\.[0-9]+"' "$cargo" | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || true)
      if [[ -n "$cur" ]]; then
        new=$(bump_patch "$cur")
        # Bump Cargo.toml workspace version (portable across BSD and GNU sed).
        # The pattern `^version = "X.Y.Z"$` matches only the workspace.package
        # line because dependency entries have leading whitespace.
        sed -i.bak -E "s|^version = \"${cur//./\\.}\"$|version = \"${new}\"|" "$cargo"
        rm -f "${cargo}.bak"
        # Mirror into VERSION.md
        if [[ -f "$repo_root/VERSION.md" ]]; then
          replace_version_line "$repo_root/VERSION.md" "Software-Version" "$new"
        fi
        domain="software"
        new_version="$new"
      fi
    fi
    ;;
esac

# --- Document domain: Markdown ---
case "$rel" in
  *.md)
    if [[ -f "$repo_root/VERSION.md" ]]; then
      cur=$(grep -E '^\*\*Document-Version:\*\*[[:space:]]+`[0-9]+\.[0-9]+\.[0-9]+`' "$repo_root/VERSION.md" | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || true)
      if [[ -n "$cur" ]]; then
        new=$(bump_patch "$cur")
        replace_version_line "$repo_root/VERSION.md" "Document-Version" "$new"
        if [[ -n "$domain" ]]; then
          domain="${domain}+document"
        else
          domain="document"
        fi
        if [[ -n "$new_version" ]]; then
          new_version="${new_version} (sw) / ${new} (doc)"
        else
          new_version="$new"
        fi
      fi
    fi
    ;;
esac

# Always update Last-Updated when domain was touched
if [[ -n "$domain" ]] && [[ -f "$repo_root/VERSION.md" ]]; then
  replace_timestamp_line "$repo_root/VERSION.md" "$now"
  jq -n --arg d "$domain" --arg n "$new_version" --arg t "$now" \
    '{hookSpecificOutput:{hookEventName:"PostToolUse",additionalContext:("auto-version: " + $d + " → " + $n + " @ " + $t)}}'
fi

exit 0
