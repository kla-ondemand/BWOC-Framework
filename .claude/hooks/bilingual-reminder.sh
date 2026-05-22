#!/usr/bin/env bash
# bilingual-reminder.sh — PostToolUse Write|Edit hook
#
# Reminds the model to keep translation pairs in sync. Covers two patterns:
#
#   1. */docs/<lang>/<NAME>.<lang>.md — template + framework docs (both
#      directions: en→th and th→en).
#   2. <repo-root>/FILENAME.md ↔ <repo-root>/FILENAME.th.md — root-level
#      metadata (e.g., VISION.md ↔ VISION.th.md). The canonical→translation
#      direction only fires if the translation already exists, to avoid
#      noisy reminders for files like CHANGELOG.md that aren't paired.
#
# Pure nudge — non-blocking, no exit-2. Output is JSON additionalContext
# injected back into the model's next turn.

set -euo pipefail

f=$(jq -r '.tool_input.file_path // empty')
[[ -z "$f" ]] && exit 0

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
rel="${f#$repo_root/}"

# Skip files outside the repo.
case "$rel" in
  /*) exit 0 ;;
esac

counterpart=""
direction=""

# Pattern 1 — docs/<lang>/<NAME>.<lang>.md, both directions.
case "$f" in
  */docs/en/*.en.md)
    counterpart=$(echo "$f" | sed 's|/docs/en/|/docs/th/|; s|\.en\.md$|.th.md|')
    direction="TH"
    ;;
  */docs/th/*.th.md)
    counterpart=$(echo "$f" | sed 's|/docs/th/|/docs/en/|; s|\.th\.md$|.en.md|')
    direction="EN canonical"
    ;;
esac

# Pattern 2 — root-level FILENAME.md ↔ FILENAME.th.md, both directions.
# Applies only to files directly at the repo root (no subdirectory in `rel`).
if [ -z "$counterpart" ] && [[ "$rel" != */* ]]; then
  case "$rel" in
    *.th.md)
      candidate=$(echo "$f" | sed 's|\.th\.md$|.md|')
      counterpart="$candidate"
      direction="EN canonical"
      ;;
    *.md)
      candidate=$(echo "$f" | sed 's|\.md$|.th.md|')
      if [ -f "$candidate" ]; then
        counterpart="$candidate"
        direction="TH"
      fi
      ;;
  esac
fi

[ -z "$counterpart" ] && exit 0

if [ -f "$counterpart" ]; then
  msg="bilingual parity: also update $counterpart"
else
  msg="bilingual parity: matching $direction file is MISSING — create $counterpart"
fi
jq -n --arg m "$msg" '{hookSpecificOutput:{hookEventName:"PostToolUse",additionalContext:$m}}'
