#!/usr/bin/env bash
#
# audit-iso-29110 — Basic profile file-existence audit.
#
# Invoked by `bwoc audit run`. Reads the operator's workspace path from the
# BWOC_WORKSPACE env var, the plugin's own directory from BWOC_PLUGIN_DIR,
# and the operation name from BWOC_AUDIT_OPERATION (expected: "audit_run").
# Emits a JSON array of findings to stdout, in criterion-declaration order
# from criteria.toml. Schema: PLUGINS.en.md §Audit Findings Schema (BWOC-11).
#
# Exit 0 on success — non-pass findings are *findings*, not errors. A
# non-zero exit signals a framework-side problem (missing env, unreadable
# criteria.toml) and rejects the run; the BWOC-12 dispatcher in audit.rs
# treats that as a plugin bug (PLUGINS.en.md line 59).

set -euo pipefail

# -- Resolve inputs ---------------------------------------------------------

if [[ -z "${BWOC_WORKSPACE:-}" ]]; then
  echo "audit-iso-29110: BWOC_WORKSPACE is unset" >&2
  exit 1
fi
if [[ -z "${BWOC_PLUGIN_DIR:-}" ]]; then
  # The dispatcher always sets BWOC_PLUGIN_DIR. Fall back to the script's
  # directory so the entry can be hand-invoked for debugging.
  BWOC_PLUGIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
fi
WORKSPACE="$BWOC_WORKSPACE"
CRITERIA_FILE="$BWOC_PLUGIN_DIR/criteria.toml"

if [[ ! -f "$CRITERIA_FILE" ]]; then
  echo "audit-iso-29110: missing $CRITERIA_FILE" >&2
  exit 1
fi

# -- TOML extraction --------------------------------------------------------
#
# criteria.toml uses a constrained shape (single-line scalars; arrays on one
# line). awk walks the file, emits one TSV row per criterion in declaration
# order:
#
#   <id>\t<severity>\t<process>\t<work_product>\t<description>\t<candidate1>|<candidate2>|...
#
# `|` separates candidate paths (paths themselves are TOML strings and never
# contain `|` in our authored data; the parser would reject any that did).

parse_criteria() {
  awk -v FS='' '
    function trim(s) { gsub(/^[ \t]+|[ \t]+$/, "", s); return s }
    function string_value(line,   v) {
      sub(/^[^=]*=[ \t]*"/, "", line)
      sub(/"[ \t]*$/, "", line)
      return line
    }
    function array_value(line,   v) {
      sub(/^[^=]*=[ \t]*\[/, "", line)
      sub(/\][ \t]*$/, "", line)
      gsub(/"[ \t]*,[ \t]*"/, "|", line)
      gsub(/^"|"$/, "", line)
      return line
    }
    function emit() {
      if (id == "") return
      printf "%s\t%s\t%s\t%s\t%s\t%s\n", id, severity, process, work_product, description, candidates
    }
    /^[ \t]*#/ { next }
    /^[ \t]*$/ { next }
    /^\[criterion\./ {
      emit()
      id = $0
      sub(/^\[criterion\./, "", id)
      sub(/\][ \t]*$/, "", id)
      severity = ""; process = ""; work_product = ""; description = ""; candidates = ""
      next
    }
    /^[ \t]*severity[ \t]*=/      { severity = string_value($0); next }
    /^[ \t]*process[ \t]*=/       { process = string_value($0); next }
    /^[ \t]*work_product[ \t]*=/  { work_product = string_value($0); next }
    /^[ \t]*description[ \t]*=/   { description = string_value($0); next }
    /^[ \t]*candidates[ \t]*=/    { candidates = array_value($0); next }
    END { emit() }
  ' "$CRITERIA_FILE"
}

# -- JSON helpers -----------------------------------------------------------
#
# Inline JSON escape for the limited set of characters that appear in our
# authored strings (quote, backslash, newline, tab). The audit.rs validator
# re-parses every finding, so any escape bug surfaces as a structured error
# rather than silent corruption.

json_escape() {
  local s="$1"
  s="${s//\\/\\\\}"
  s="${s//\"/\\\"}"
  s="${s//$'\n'/\\n}"
  s="${s//$'\t'/\\t}"
  printf '%s' "$s"
}

# -- Main loop --------------------------------------------------------------

TMP="$(mktemp -t audit-iso-29110.XXXXXX)"
trap 'rm -f "$TMP"' EXIT

parse_criteria > "$TMP"

printf '['
first=1
while IFS=$'\t' read -r id severity process work_product description candidates_joined; do
  [[ -z "$id" ]] && continue

  # Pipe-separated candidate list → array.
  IFS='|' read -r -a candidates <<< "$candidates_joined"
  primary="${candidates[0]:-}"

  # Probe each candidate against the workspace; first hit wins.
  found=""
  for cand in "${candidates[@]}"; do
    [[ -z "$cand" ]] && continue
    if [[ -e "$WORKSPACE/$cand" ]]; then
      found="$cand"
      break
    fi
  done

  # Comma between findings (criterion-declaration order = report order).
  if [[ $first -eq 1 ]]; then first=0; else printf ','; fi

  if [[ -n "$found" ]]; then
    printf '\n  {"criterion_id":"%s","severity":"%s","status":"pass","evidence":{"kind":"file","value":"%s"}}' \
      "$(json_escape "$id")" \
      "$(json_escape "$severity")" \
      "$(json_escape "$found")"
  else
    # PLUGINS.en.md example (line 106-113): a failing file-evidence finding
    # carries the expected path as evidence.value plus an actionable remedy.
    alt_list=""
    if (( ${#candidates[@]} > 1 )); then
      alt_list=" (or one of: $(IFS=', '; echo "${candidates[*]:1}"))"
    fi
    remedy="Create ${primary} documenting the ${work_product} work product${alt_list}. ${description}"
    printf '\n  {"criterion_id":"%s","severity":"%s","status":"fail","evidence":{"kind":"file","value":"%s"},"remedy":"%s"}' \
      "$(json_escape "$id")" \
      "$(json_escape "$severity")" \
      "$(json_escape "$primary")" \
      "$(json_escape "$remedy")"
  fi
done < "$TMP"
printf '\n]\n'
