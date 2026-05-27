#!/usr/bin/env bash
#
# audit-iso-20000-1 — runtime. Emits attestation + sample findings per the
# BWOC-27 schema.
#
# 20000-1 is the first runtime that mixes two evidence kinds. Each criterion
# declares its `expected_evidence_kind` in criteria.toml (BWOC-26 / BWOC-29);
# the runtime reads it and routes to the matching .bwoc/workspace.toml array:
#
#   attestation → [[plugins.audit-iso-20000-1.attestations]]  (documented-artifact
#                 clauses: 4.3 scope, 5.2 policy, 8.3.1 catalogue)
#   sample      → [[plugins.audit-iso-20000-1.samples]]        (operational-rate
#                 clauses: 8.3.3 SLA, 8.5.1 change, 8.6.1 incident, 8.6.3
#                 problem, 10.2 improvement)
#
# A criterion with complete operator-provided evidence emits status="pass"
# (evidence.kind = attestation | sample). A criterion without it — or with
# incomplete/invalid evidence — emits status="fail" pointing the operator at
# workspace.toml (evidence.kind = "file": the file the operator must edit IS
# the reproducible referent; an empty-value attestation/sample is forbidden
# by the schema, and "none" is forbidden with "fail").
#
# The runtime does NOT impose an SLA threshold on samples — a recorded sample
# is evidence and passes; the rate is surfaced for the human auditor to judge.
# Thresholds are organisational policy, out of scope for v0.2.0.
#
# Env contract (set by `bwoc audit run` dispatcher):
#   BWOC_WORKSPACE       — absolute path to workspace root
#   BWOC_PLUGIN_DIR      — absolute path to this plugin dir
#   BWOC_AUDIT_OPERATION — expected "audit_run"
#
# Schema: PLUGINS.en.md §Audit Findings Schema (BWOC-11 + BWOC-27 extension).
# Design: notes/2026-05-27_20000-1-sample-source.md.
#
# Exit 0 on success — non-pass findings are *findings*, not errors. A
# non-zero exit signals a framework-side problem (unreadable criteria.toml).

set -euo pipefail

# -- Resolve inputs ---------------------------------------------------------

if [[ -z "${BWOC_WORKSPACE:-}" ]]; then
  echo "audit-iso-20000-1: BWOC_WORKSPACE is unset" >&2
  exit 1
fi
if [[ -z "${BWOC_PLUGIN_DIR:-}" ]]; then
  BWOC_PLUGIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
fi
WORKSPACE="$BWOC_WORKSPACE"
CRITERIA_FILE="$BWOC_PLUGIN_DIR/criteria.toml"
WORKSPACE_TOML="$WORKSPACE/.bwoc/workspace.toml"

if [[ ! -f "$CRITERIA_FILE" ]]; then
  echo "audit-iso-20000-1: missing $CRITERIA_FILE" >&2
  exit 1
fi

# -- Parse criteria.toml ----------------------------------------------------
#
# Emit one TSV row per criterion in declaration order:
#   <id>\t<severity>\t<expected_evidence_kind>
#
# A criterion header is `[criterion.<id>]`; the per-kind contract subtable is
# `[criterion.<id>.<kind>]`. Since `<id>` is kebab-case (no dots), a header
# whose body contains a dot is a subtable and is skipped — its keys belong to
# the current criterion, not a new one.

parse_criteria() {
  awk '
    function string_value(line) {
      sub(/^[^=]*=[ \t]*"/, "", line)
      sub(/"[^"]*$/, "", line)
      return line
    }
    function emit() {
      if (id != "") printf "%s\t%s\t%s\n", id, severity, ek
    }
    /^[ \t]*#/ { next }
    /^[ \t]*$/ { next }
    /^\[criterion\./ {
      hdr = $0
      sub(/^\[criterion\./, "", hdr)
      sub(/\][ \t]*$/, "", hdr)
      if (hdr ~ /\./) { next }   # subtable header (<id>.<kind>) — keep current criterion
      emit()
      id = hdr
      severity = ""
      ek = ""
      next
    }
    /^\[/ {
      emit()
      id = ""; severity = ""; ek = ""
      next
    }
    /^[ \t]*severity[ \t]*=/               { severity = string_value($0); next }
    /^[ \t]*expected_evidence_kind[ \t]*=/ { ek = string_value($0); next }
    END { emit() }
  ' "$CRITERIA_FILE"
}

# -- Parse workspace.toml arrays --------------------------------------------
#
# Two array-of-tables under [plugins.audit-iso-20000-1]:
#   [[plugins.audit-iso-20000-1.attestations]]
#   [[plugins.audit-iso-20000-1.samples]]
#
# Each parser walks its own block; any other `[`-line ends the current entry
# (so the two arrays never bleed into each other). Workspace.toml absent →
# no evidence → every criterion fails with its missing-evidence remedy. The
# free-text field is emitted last so a stray tab cannot shift earlier columns.
#
# string_value extracts between the first quote after `=` and the last quote
# on the line — tolerant of a trailing inline comment.

parse_attestations() {
  if [[ ! -f "$WORKSPACE_TOML" ]]; then
    return 0
  fi
  awk '
    function string_value(line) {
      sub(/^[^=]*=[ \t]*"/, "", line)
      sub(/"[^"]*$/, "", line)
      return line
    }
    function emit() {
      if (in_block && criterion_id != "") {
        printf "%s\t%s\t%s\t%s\t%s\n", criterion_id, signer, signed_at, valid_through, statement
      }
      criterion_id = ""; signer = ""; signed_at = ""; valid_through = ""; statement = ""
    }
    /^[ \t]*#/ { next }
    /^[ \t]*$/ { next }
    /^\[\[plugins\.audit-iso-20000-1\.attestations\]\]/ { emit(); in_block = 1; next }
    /^\[/ { emit(); in_block = 0; next }
    in_block && /^[ \t]*criterion_id[ \t]*=/  { criterion_id  = string_value($0); next }
    in_block && /^[ \t]*signer[ \t]*=/        { signer        = string_value($0); next }
    in_block && /^[ \t]*signed_at[ \t]*=/     { signed_at     = string_value($0); next }
    in_block && /^[ \t]*valid_through[ \t]*=/ { valid_through = string_value($0); next }
    in_block && /^[ \t]*statement[ \t]*=/     { statement     = string_value($0); next }
    END { emit() }
  ' "$WORKSPACE_TOML"
}

parse_samples() {
  if [[ ! -f "$WORKSPACE_TOML" ]]; then
    return 0
  fi
  awk '
    function string_value(line) {
      sub(/^[^=]*=[ \t]*"/, "", line)
      sub(/"[^"]*$/, "", line)
      return line
    }
    function int_value(line) {
      sub(/^[^=]*=[ \t]*/, "", line)
      sub(/[ \t]*#.*$/, "", line)
      sub(/[ \t]*$/, "", line)
      return line
    }
    function emit() {
      if (in_block && criterion_id != "") {
        printf "%s\t%s\t%s\t%s\t%s\n", criterion_id, sampled_count, sampled_of, window, summary
      }
      criterion_id = ""; sampled_count = ""; sampled_of = ""; window = ""; summary = ""
    }
    /^[ \t]*#/ { next }
    /^[ \t]*$/ { next }
    /^\[\[plugins\.audit-iso-20000-1\.samples\]\]/ { emit(); in_block = 1; next }
    /^\[/ { emit(); in_block = 0; next }
    in_block && /^[ \t]*criterion_id[ \t]*=/  { criterion_id  = string_value($0); next }
    in_block && /^[ \t]*summary[ \t]*=/       { summary       = string_value($0); next }
    in_block && /^[ \t]*sampled_count[ \t]*=/ { sampled_count = int_value($0);    next }
    in_block && /^[ \t]*sampled_of[ \t]*=/    { sampled_of    = int_value($0);    next }
    in_block && /^[ \t]*window[ \t]*=/        { window        = string_value($0); next }
    END { emit() }
  ' "$WORKSPACE_TOML"
}

# -- JSON helpers -----------------------------------------------------------

json_escape() {
  local s="$1"
  s="${s//\\/\\\\}"
  s="${s//\"/\\\"}"
  s="${s//$'\n'/\\n}"
  s="${s//$'\t'/\\t}"
  printf '%s' "$s"
}

# -- Build evidence lookup tables -------------------------------------------
#
# Bash 3.2 (macOS default) lacks associative arrays, so each lookup is a TSV
# file keyed by criterion_id. First occurrence wins (duplicate criterion_id
# is a workspace-author bug; bwoc check is the right place to surface that).
# One field is extracted at a time because `read -r` with IFS=$'\t' collapses
# consecutive tabs, mangling empty optional fields; awk over a single-digit-N
# TSV is cheap.

ATTESTATIONS_TSV="$(mktemp -t audit-iso-20000-1-att.XXXXXX)"
SAMPLES_TSV="$(mktemp -t audit-iso-20000-1-smp.XXXXXX)"
trap 'rm -f "$ATTESTATIONS_TSV" "$SAMPLES_TSV"' EXIT

parse_attestations > "$ATTESTATIONS_TSV"
parse_samples > "$SAMPLES_TSV"

lookup_field() {
  local tsv="$1" cid="$2" field_index="$3"
  awk -F '\t' -v target="$cid" -v idx="$field_index" '
    $1 == target {
      printf "%s", $idx
      exit
    }
  ' "$tsv"
}

# -- Build findings ---------------------------------------------------------
#
# Each builder prints a single-line finding object (no separator). The joiner
# below adds commas + indentation, so a builder always produces exactly one
# valid finding — no dangling-comma risk from the routing branch.

build_fail_file() {
  local id="$1" severity="$2" remedy="$3"
  printf '{"criterion_id":"%s","severity":"%s","status":"fail","evidence":{"kind":"file","value":".bwoc/workspace.toml"},"remedy":"%s"}' \
    "$(json_escape "$id")" "$(json_escape "$severity")" "$(json_escape "$remedy")"
}

build_attestation() {
  local id="$1" severity="$2"
  local have signer signed_at valid_through statement missing remedy
  have="$(lookup_field "$ATTESTATIONS_TSV" "$id" 1)"
  if [[ -z "$have" ]]; then
    remedy="Provide a signed attestation in .bwoc/workspace.toml under [[plugins.audit-iso-20000-1.attestations]] with criterion_id=\"${id}\", statement, signer, and signed_at (ISO 8601 date) to satisfy this criterion."
    build_fail_file "$id" "$severity" "$remedy"
    return
  fi
  signer="$(lookup_field "$ATTESTATIONS_TSV" "$id" 2)"
  signed_at="$(lookup_field "$ATTESTATIONS_TSV" "$id" 3)"
  valid_through="$(lookup_field "$ATTESTATIONS_TSV" "$id" 4)"
  statement="$(lookup_field "$ATTESTATIONS_TSV" "$id" 5)"
  if [[ -z "$statement" || -z "$signer" || -z "$signed_at" ]]; then
    missing=""
    [[ -z "$statement" ]] && missing="$missing statement"
    [[ -z "$signer"    ]] && missing="$missing signer"
    [[ -z "$signed_at" ]] && missing="$missing signed_at"
    remedy="Attestation for ${id} in .bwoc/workspace.toml is missing required field(s):${missing}. Provide statement, signer, and signed_at (ISO 8601 date) to satisfy this criterion."
    build_fail_file "$id" "$severity" "$remedy"
    return
  fi
  if [[ -n "$valid_through" ]]; then
    printf '{"criterion_id":"%s","severity":"%s","status":"pass","evidence":{"kind":"attestation","value":"%s","signer":"%s","signed_at":"%s","valid_through":"%s"}}' \
      "$(json_escape "$id")" "$(json_escape "$severity")" "$(json_escape "$statement")" \
      "$(json_escape "$signer")" "$(json_escape "$signed_at")" "$(json_escape "$valid_through")"
  else
    printf '{"criterion_id":"%s","severity":"%s","status":"pass","evidence":{"kind":"attestation","value":"%s","signer":"%s","signed_at":"%s"}}' \
      "$(json_escape "$id")" "$(json_escape "$severity")" "$(json_escape "$statement")" \
      "$(json_escape "$signer")" "$(json_escape "$signed_at")"
  fi
}

build_sample() {
  local id="$1" severity="$2"
  local have count of window summary missing remedy
  have="$(lookup_field "$SAMPLES_TSV" "$id" 1)"
  if [[ -z "$have" ]]; then
    remedy="Provide a recorded sample in .bwoc/workspace.toml under [[plugins.audit-iso-20000-1.samples]] with criterion_id=\"${id}\", summary, sampled_count, and sampled_of (integers) to satisfy this criterion."
    build_fail_file "$id" "$severity" "$remedy"
    return
  fi
  count="$(lookup_field "$SAMPLES_TSV" "$id" 2)"
  of="$(lookup_field "$SAMPLES_TSV" "$id" 3)"
  window="$(lookup_field "$SAMPLES_TSV" "$id" 4)"
  summary="$(lookup_field "$SAMPLES_TSV" "$id" 5)"
  if [[ -z "$summary" || -z "$count" || -z "$of" ]]; then
    missing=""
    [[ -z "$summary" ]] && missing="$missing summary"
    [[ -z "$count"   ]] && missing="$missing sampled_count"
    [[ -z "$of"      ]] && missing="$missing sampled_of"
    remedy="Sample for ${id} in .bwoc/workspace.toml is missing required field(s):${missing}. Provide summary, sampled_count, and sampled_of (integers) to satisfy this criterion."
    build_fail_file "$id" "$severity" "$remedy"
    return
  fi
  if ! [[ "$count" =~ ^[0-9]+$ && "$of" =~ ^[0-9]+$ ]]; then
    remedy="Sample for ${id} in .bwoc/workspace.toml has non-integer sampled_count/sampled_of (\"${count}\"/\"${of}\"). Provide non-negative integers to satisfy this criterion."
    build_fail_file "$id" "$severity" "$remedy"
    return
  fi
  if (( of < count )); then
    remedy="Sample for ${id} in .bwoc/workspace.toml has sampled_of (${of}) < sampled_count (${count}); sampled_of is the population and must be >= sampled_count."
    build_fail_file "$id" "$severity" "$remedy"
    return
  fi
  # Pass — sampled_count / sampled_of are JSON numbers (no quotes).
  if [[ -n "$window" ]]; then
    printf '{"criterion_id":"%s","severity":"%s","status":"pass","evidence":{"kind":"sample","value":"%s","sampled_count":%s,"sampled_of":%s,"window":"%s"}}' \
      "$(json_escape "$id")" "$(json_escape "$severity")" "$(json_escape "$summary")" \
      "$count" "$of" "$(json_escape "$window")"
  else
    printf '{"criterion_id":"%s","severity":"%s","status":"pass","evidence":{"kind":"sample","value":"%s","sampled_count":%s,"sampled_of":%s}}' \
      "$(json_escape "$id")" "$(json_escape "$severity")" "$(json_escape "$summary")" \
      "$count" "$of"
  fi
}

build_kind_error() {
  local id="$1" severity="$2" ek="$3" remedy
  if [[ -z "$ek" ]]; then
    remedy="Criterion ${id} declares no expected_evidence_kind in criteria.toml; the runtime cannot route it. Declare expected_evidence_kind = \"attestation\" or \"sample\"."
  else
    remedy="Criterion ${id} declares expected_evidence_kind=\"${ek}\" in criteria.toml, which this runtime does not handle (supported: attestation, sample)."
  fi
  build_fail_file "$id" "$severity" "$remedy"
}

# -- Emit findings ----------------------------------------------------------

findings=()
while IFS=$'\t' read -r id severity ek; do
  [[ -z "$id" ]] && continue
  case "$ek" in
    attestation) findings+=("$(build_attestation "$id" "$severity")") ;;
    sample)      findings+=("$(build_sample "$id" "$severity")") ;;
    *)           findings+=("$(build_kind_error "$id" "$severity" "$ek")") ;;
  esac
done < <(parse_criteria)

printf '['
first=1
if [[ ${#findings[@]} -gt 0 ]]; then
  for f in "${findings[@]}"; do
    if [[ $first -eq 1 ]]; then first=0; else printf ','; fi
    printf '\n  %s' "$f"
  done
fi
printf '\n]\n'
