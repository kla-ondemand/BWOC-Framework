#!/usr/bin/env bash
# queue-bump.sh — queue a SemVer level for the next auto-version hook fire.
#
# The auto-version hook (.claude/hooks/auto-version.sh) bumps PATCH by default
# on every Claude Code Write|Edit. To make the NEXT bump be MINOR or MAJOR
# instead, write a one-shot sentinel file. This script does that for you.
#
# Usage:
#   ./scripts/queue-bump.sh <software|document> <minor|major|patch>
#   ./scripts/queue-bump.sh <software|document> --clear
#   ./scripts/queue-bump.sh --status
#
# Examples:
#   ./scripts/queue-bump.sh software minor   # next .rs/.toml edit → minor
#   ./scripts/queue-bump.sh document major   # next .md edit → major
#   ./scripts/queue-bump.sh software --clear # cancel a queued bump
#   ./scripts/queue-bump.sh --status         # show pending sentinels
#
# The sentinel is consumed (deleted) by the hook after one bump. Subsequent
# edits revert to patch. Pair with scripts/bump-version.sh when you want an
# immediate manual bump without involving the hook.

set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
bwoc_dir="$repo_root/.bwoc"

usage() {
  cat <<EOF
Usage: $0 <software|document> <minor|major|patch>
       $0 <software|document> --clear
       $0 --status

Queues a one-shot bump level for the next auto-version hook fire.
The sentinel is consumed after one bump and the domain reverts to patch.
EOF
  exit 2
}

show_status() {
  local any=0
  for domain in software document; do
    local f="$bwoc_dir/next-bump.${domain}"
    if [[ -f "$f" ]]; then
      local level
      level=$(tr -d '[:space:]' <"$f")
      echo "queued: ${domain} → ${level}"
      any=1
    fi
  done
  [[ $any -eq 0 ]] && echo "no pending bumps"
}

[[ $# -lt 1 ]] && usage

if [[ "$1" == "--status" || "$1" == "-s" ]]; then
  show_status
  exit 0
fi

[[ $# -ne 2 ]] && usage

domain="$1"
case "$domain" in
  software|document) ;;
  -h|--help) usage ;;
  *) echo "error: unknown domain '$domain' (expected software|document)" >&2; usage ;;
esac

action="$2"
sentinel="$bwoc_dir/next-bump.${domain}"

case "$action" in
  --clear)
    if [[ -f "$sentinel" ]]; then
      rm -f "$sentinel"
      echo "cleared queued bump for ${domain}"
    else
      echo "no pending bump for ${domain}"
    fi
    ;;
  major|minor|patch)
    mkdir -p "$bwoc_dir"
    echo "$action" >"$sentinel"
    echo "queued: ${domain} → ${action} (one-shot; next hook fire consumes)"
    ;;
  *)
    echo "error: unknown action '$action' (expected major|minor|patch|--clear)" >&2
    usage
    ;;
esac
