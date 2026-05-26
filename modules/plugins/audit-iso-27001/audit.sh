#!/usr/bin/env bash
#
# audit-iso-27001 — stub. Runtime deferred to BWOC-EPIC-3.
#
# Invoked by `bwoc audit run`. Reads the plugin's own directory from
# BWOC_PLUGIN_DIR (or, when hand-invoked, derives it from the script
# location). Emits one `not_implemented` finding per criterion declared in
# criteria.toml, in declaration order. Schema: PLUGINS.en.md §Audit
# Findings Schema (BWOC-11). BWOC_WORKSPACE is intentionally unread — the
# stub does not inspect the workspace, and pretending to would falsify the
# audit (Musāvāda).
#
# Exit 0 on success — non-pass findings are *findings*, not errors. A
# non-zero exit signals a framework-side problem (unreadable criteria.toml)
# and rejects the run.

set -euo pipefail

if [[ -z "${BWOC_PLUGIN_DIR:-}" ]]; then
  BWOC_PLUGIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
fi
CRITERIA_FILE="$BWOC_PLUGIN_DIR/criteria.toml"

if [[ ! -f "$CRITERIA_FILE" ]]; then
  echo "audit-iso-27001: missing $CRITERIA_FILE" >&2
  exit 1
fi

# Parse criteria.toml — emit one TSV row per criterion in declaration order:
#
#   <id>\t<severity>
#
# `id` and `severity` are the only fields the stub needs. The richer fields
# (`reference`, `title`, `description`) are documentation for the future
# EPIC-3 runtime and are ignored here.
parse_criteria() {
  awk '
    function emit() {
      if (id != "") printf "%s\t%s\n", id, severity
    }
    /^[ \t]*#/ { next }
    /^[ \t]*$/ { next }
    /^\[criterion\./ {
      emit()
      id = $0
      sub(/^\[criterion\./, "", id)
      sub(/\][ \t]*$/, "", id)
      severity = ""
      next
    }
    /^[ \t]*severity[ \t]*=/ {
      v = $0
      sub(/^[^=]*=[ \t]*"/, "", v)
      sub(/"[ \t]*$/, "", v)
      severity = v
      next
    }
    END { emit() }
  ' "$CRITERIA_FILE"
}

# Inline JSON escape — id and severity are tightly constrained (kebab-case
# id, closed enum severity) so only quote and backslash matter in practice.
json_escape() {
  local s="$1"
  s="${s//\\/\\\\}"
  s="${s//\"/\\\"}"
  printf '%s' "$s"
}

REMEDY="Runtime deferred to BWOC-EPIC-3."

printf '['
first=1
while IFS=$'\t' read -r id severity; do
  [[ -z "$id" ]] && continue
  if [[ $first -eq 1 ]]; then first=0; else printf ','; fi
  printf '\n  {"criterion_id":"%s","severity":"%s","status":"not_implemented","evidence":{"kind":"none","value":""},"remedy":"%s"}' \
    "$(json_escape "$id")" \
    "$(json_escape "$severity")" \
    "$(json_escape "$REMEDY")"
done < <(parse_criteria)
printf '\n]\n'
