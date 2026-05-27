#!/usr/bin/env bash
#
# audit-iso-27001 — runtime. Emits attestation + SoA-driven sample findings
# per the BWOC-27 schema. The last EPIC-3 runtime; closes the epic.
#
# 27001 is the only standard that needs both evidence kinds AND a dynamic,
# operator-declared sampling population. Each criterion declares its
# `expected_evidence_kind` in criteria.toml (BWOC-26 / BWOC-29); the runtime
# reads it and routes:
#
#   attestation → [[plugins.audit-iso-27001.attestations]]   (main-body clauses:
#                 4.3 scope, 5.2 policy, 6.1.2 risk, 6.1.3 SoA, 9.2 internal audit)
#   sample      → [[plugins.audit-iso-27001.soa]] (scope) +
#                 [[plugins.audit-iso-27001.samples]] (record)  (Annex A controls:
#                 A.5.15 access, A.5.24 incident, A.5.29 continuity)
#
# SoA-driven sampling: the in-scope control set is `[[…soa]]` entries with
# applicable=true. Its size M is `sampled_of`; the count K of THIS plugin's
# Annex A controls that are in scope is `sampled_count`. The operator never
# types those numbers — they follow the SoA's scope decisions. An Annex A
# control absent from the SoA, or in scope without a recorded sample, fails;
# a justifiably-excluded control (applicable=false + justification) emits
# `status="not_applicable"`. See notes/2026-05-27_27001-soa-sampling.md.
#
# Env contract (set by `bwoc audit run` dispatcher):
#   BWOC_WORKSPACE       — absolute path to workspace root
#   BWOC_PLUGIN_DIR      — absolute path to this plugin dir
#   BWOC_AUDIT_OPERATION — expected "audit_run"
#
# Schema: PLUGINS.en.md §Audit Findings Schema (BWOC-11 + BWOC-27 extension).
#
# Exit 0 on success — non-pass findings are *findings*, not errors. A
# non-zero exit signals a framework-side problem (unreadable criteria.toml).

set -euo pipefail

# -- Resolve inputs ---------------------------------------------------------

if [[ -z "${BWOC_WORKSPACE:-}" ]]; then
  echo "audit-iso-27001: BWOC_WORKSPACE is unset" >&2
  exit 1
fi
if [[ -z "${BWOC_PLUGIN_DIR:-}" ]]; then
  BWOC_PLUGIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
fi
WORKSPACE="$BWOC_WORKSPACE"
CRITERIA_FILE="$BWOC_PLUGIN_DIR/criteria.toml"
WORKSPACE_TOML="$WORKSPACE/.bwoc/workspace.toml"

if [[ ! -f "$CRITERIA_FILE" ]]; then
  echo "audit-iso-27001: missing $CRITERIA_FILE" >&2
  exit 1
fi

# -- Parse criteria.toml ----------------------------------------------------
#
# Emit one TSV row per criterion in declaration order:
#   <id>\t<severity>\t<expected_evidence_kind>\t<annex_control>
#
# A criterion header is `[criterion.<id>]`; the per-kind contract subtable is
# `[criterion.<id>.<kind>]`. Since `<id>` is kebab-case (no dots), a header
# whose body contains a dot is a subtable and is skipped — its keys belong to
# the current criterion, not a new one. `annex_control` is empty for the
# main-body attestation criteria.

parse_criteria() {
  awk '
    function string_value(line) {
      sub(/^[^=]*=[ \t]*"/, "", line)
      sub(/"[^"]*$/, "", line)
      return line
    }
    function emit() {
      if (id != "") printf "%s\t%s\t%s\t%s\n", id, severity, ek, annex
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
      annex = ""
      next
    }
    /^\[/ {
      emit()
      id = ""; severity = ""; ek = ""; annex = ""
      next
    }
    /^[ \t]*severity[ \t]*=/               { severity = string_value($0); next }
    /^[ \t]*expected_evidence_kind[ \t]*=/ { ek = string_value($0); next }
    /^[ \t]*annex_control[ \t]*=/          { annex = string_value($0); next }
    END { emit() }
  ' "$CRITERIA_FILE"
}

# -- Parse workspace.toml arrays --------------------------------------------
#
# Three array-of-tables under [plugins.audit-iso-27001]:
#   [[plugins.audit-iso-27001.attestations]]  (main-body clauses)
#   [[plugins.audit-iso-27001.soa]]           (Statement of Applicability — scope)
#   [[plugins.audit-iso-27001.samples]]       (Annex A audit-sample records)
#
# Each parser walks its own block; any other `[`-line ends the current entry
# (so the arrays never bleed into each other). Workspace.toml absent → no
# evidence → every criterion fails / every Annex A control is unaddressed.
# Free-text fields are emitted last so a stray tab cannot shift columns.

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
    /^\[\[plugins\.audit-iso-27001\.attestations\]\]/ { emit(); in_block = 1; next }
    /^\[/ { emit(); in_block = 0; next }
    in_block && /^[ \t]*criterion_id[ \t]*=/  { criterion_id  = string_value($0); next }
    in_block && /^[ \t]*signer[ \t]*=/        { signer        = string_value($0); next }
    in_block && /^[ \t]*signed_at[ \t]*=/     { signed_at     = string_value($0); next }
    in_block && /^[ \t]*valid_through[ \t]*=/ { valid_through = string_value($0); next }
    in_block && /^[ \t]*statement[ \t]*=/     { statement     = string_value($0); next }
    END { emit() }
  ' "$WORKSPACE_TOML"
}

# SoA rows:  <control>\t<applicable>\t<justification>
# `applicable` is a TOML boolean (true/false, unquoted); bool_value strips the
# key, any inline comment, surrounding whitespace, and tolerates a quoted form.
parse_soa() {
  if [[ ! -f "$WORKSPACE_TOML" ]]; then
    return 0
  fi
  awk '
    function string_value(line) {
      sub(/^[^=]*=[ \t]*"/, "", line)
      sub(/"[^"]*$/, "", line)
      return line
    }
    function bool_value(line) {
      sub(/^[^=]*=[ \t]*/, "", line)
      sub(/[ \t]*#.*$/, "", line)
      sub(/[ \t]*$/, "", line)
      gsub(/"/, "", line)
      return line
    }
    function emit() {
      if (in_block && control != "") {
        printf "%s\t%s\t%s\n", control, applicable, justification
      }
      control = ""; applicable = ""; justification = ""
    }
    /^[ \t]*#/ { next }
    /^[ \t]*$/ { next }
    /^\[\[plugins\.audit-iso-27001\.soa\]\]/ { emit(); in_block = 1; next }
    /^\[/ { emit(); in_block = 0; next }
    in_block && /^[ \t]*control[ \t]*=/       { control       = string_value($0); next }
    in_block && /^[ \t]*applicable[ \t]*=/    { applicable    = bool_value($0);   next }
    in_block && /^[ \t]*justification[ \t]*=/ { justification = string_value($0); next }
    END { emit() }
  ' "$WORKSPACE_TOML"
}

# Sample rows:  <criterion_id>\t<window>\t<summary>
# Unlike 20000-1, the operator does NOT supply sampled_count/sampled_of — they
# are SoA-derived (see the K/M computation below). The entry only records that
# the control was sampled, plus a human summary and optional window.
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
    function emit() {
      if (in_block && criterion_id != "") {
        printf "%s\t%s\t%s\n", criterion_id, window, summary
      }
      criterion_id = ""; window = ""; summary = ""
    }
    /^[ \t]*#/ { next }
    /^[ \t]*$/ { next }
    /^\[\[plugins\.audit-iso-27001\.samples\]\]/ { emit(); in_block = 1; next }
    /^\[/ { emit(); in_block = 0; next }
    in_block && /^[ \t]*criterion_id[ \t]*=/ { criterion_id = string_value($0); next }
    in_block && /^[ \t]*window[ \t]*=/       { window       = string_value($0); next }
    in_block && /^[ \t]*summary[ \t]*=/      { summary       = string_value($0); next }
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

# -- Build lookup tables ----------------------------------------------------
#
# Bash 3.2 (macOS default) lacks associative arrays, so each lookup is a TSV
# file. First occurrence wins (duplicate keys are a workspace-author bug that
# bwoc check should surface). One field is extracted at a time because
# `read -r` with IFS=$'\t' collapses consecutive tabs, mangling empty optional
# fields; awk over a single-digit-N TSV is cheap.

CRITERIA_TSV="$(mktemp -t audit-iso-27001-crit.XXXXXX)"
ATTESTATIONS_TSV="$(mktemp -t audit-iso-27001-att.XXXXXX)"
SOA_TSV="$(mktemp -t audit-iso-27001-soa.XXXXXX)"
SAMPLES_TSV="$(mktemp -t audit-iso-27001-smp.XXXXXX)"
trap 'rm -f "$CRITERIA_TSV" "$ATTESTATIONS_TSV" "$SOA_TSV" "$SAMPLES_TSV"' EXIT

parse_criteria > "$CRITERIA_TSV"
parse_attestations > "$ATTESTATIONS_TSV"
parse_soa > "$SOA_TSV"
parse_samples > "$SAMPLES_TSV"

lookup_field() {
  local tsv="$1" key="$2" field_index="$3"
  awk -F '\t' -v target="$key" -v idx="$field_index" '
    $1 == target {
      printf "%s", $idx
      exit
    }
  ' "$tsv"
}

# -- SoA-driven population (K of M) -----------------------------------------
#
# M = number of in-scope controls in the SoA (applicable == true). This is the
#     sampling population — `sampled_of`.
# K = number of THIS plugin's Annex A controls that are in scope. This is the
#     plugin's sampling breadth — `sampled_count`. K <= M by construction (each
#     in-scope plugin control is itself counted in M). K is computed by SCOPE,
#     not by evidence completeness, so a finding's K never depends on a sibling
#     control's sample (findings stay independent — PLUGINS.en.md §schema-rules).

SOA_IN_SCOPE_COUNT="$(awk -F '\t' '$2 == "true" { n++ } END { print n + 0 }' "$SOA_TSV")"

PLUGIN_IN_SCOPE_COUNT=0
while IFS=$'\t' read -r _id _severity ek annex; do
  [[ "$ek" == "sample" && -n "$annex" ]] || continue
  if [[ "$(lookup_field "$SOA_TSV" "$annex" 2)" == "true" ]]; then
    PLUGIN_IN_SCOPE_COUNT=$((PLUGIN_IN_SCOPE_COUNT + 1))
  fi
done < "$CRITERIA_TSV"

# -- Finding builders -------------------------------------------------------
#
# Each builder prints exactly one single-line finding object (no separator).
# The joiner below adds commas + indentation, so the routing branch can never
# emit a dangling comma.

build_fail_file() {
  local id="$1" severity="$2" remedy="$3"
  printf '{"criterion_id":"%s","severity":"%s","status":"fail","evidence":{"kind":"file","value":".bwoc/workspace.toml"},"remedy":"%s"}' \
    "$(json_escape "$id")" "$(json_escape "$severity")" "$(json_escape "$remedy")"
}

build_not_applicable() {
  local id="$1" severity="$2" remedy="$3"
  printf '{"criterion_id":"%s","severity":"%s","status":"not_applicable","evidence":{"kind":"none","value":""},"remedy":"%s"}' \
    "$(json_escape "$id")" "$(json_escape "$severity")" "$(json_escape "$remedy")"
}

build_attestation() {
  local id="$1" severity="$2"
  local have signer signed_at valid_through statement missing remedy
  have="$(lookup_field "$ATTESTATIONS_TSV" "$id" 1)"
  if [[ -z "$have" ]]; then
    remedy="Provide a signed attestation in .bwoc/workspace.toml under [[plugins.audit-iso-27001.attestations]] with criterion_id=\"${id}\", statement, signer, and signed_at (ISO 8601 date) to satisfy this criterion."
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

# Annex A control criterion — SoA-gated sample. The control reference comes
# from criteria.toml `annex_control`; the SoA decides scope.
build_annex_sample() {
  local id="$1" severity="$2" annex="$3"
  local present applicable justification have_sample summary window remedy

  if [[ -z "$annex" ]]; then
    remedy="Criterion ${id} is declared expected_evidence_kind=\"sample\" but has no annex_control in criteria.toml; the runtime cannot resolve it against the Statement of Applicability. Declare annex_control = \"A.x.y\"."
    build_fail_file "$id" "$severity" "$remedy"
    return
  fi

  present="$(lookup_field "$SOA_TSV" "$annex" 1)"
  if [[ -z "$present" ]]; then
    remedy="Control ${annex} is not addressed in the Statement of Applicability. ISO/IEC 27001 6.1.3 requires the SoA to address every Annex A control. Declare it in .bwoc/workspace.toml under [[plugins.audit-iso-27001.soa]] with control=\"${annex}\", applicable (true/false), and justification."
    build_fail_file "$id" "$severity" "$remedy"
    return
  fi

  applicable="$(lookup_field "$SOA_TSV" "$annex" 2)"
  justification="$(lookup_field "$SOA_TSV" "$annex" 3)"

  case "$applicable" in
    false)
      if [[ -z "$justification" ]]; then
        remedy="Control ${annex} is marked applicable=false in the Statement of Applicability but has no justification. ISO/IEC 27001 6.1.3 requires a justification for exclusions. Add justification to the [[plugins.audit-iso-27001.soa]] entry for ${annex}."
        build_fail_file "$id" "$severity" "$remedy"
        return
      fi
      remedy="Control ${annex} is excluded from the ISMS scope per the Statement of Applicability: \"${justification}\". Re-confirm this exclusion remains justified at the next audit cycle."
      build_not_applicable "$id" "$severity" "$remedy"
      return
      ;;
    true)
      if [[ -z "$justification" ]]; then
        remedy="Control ${annex} is marked applicable=true in the Statement of Applicability but has no justification. ISO/IEC 27001 6.1.3 requires a justification for inclusions. Add justification to the [[plugins.audit-iso-27001.soa]] entry for ${annex}."
        build_fail_file "$id" "$severity" "$remedy"
        return
      fi
      have_sample="$(lookup_field "$SAMPLES_TSV" "$id" 1)"
      if [[ -z "$have_sample" ]]; then
        remedy="Control ${annex} is in scope per the Statement of Applicability but has no recorded audit sample. Provide one in .bwoc/workspace.toml under [[plugins.audit-iso-27001.samples]] with criterion_id=\"${id}\" and summary to record that the control was sampled this audit cycle."
        build_fail_file "$id" "$severity" "$remedy"
        return
      fi
      window="$(lookup_field "$SAMPLES_TSV" "$id" 2)"
      summary="$(lookup_field "$SAMPLES_TSV" "$id" 3)"
      if [[ -z "$summary" ]]; then
        remedy="Sample for ${id} in .bwoc/workspace.toml is missing the required summary field. Provide a short summary of what was sampled for control ${annex} to satisfy this criterion."
        build_fail_file "$id" "$severity" "$remedy"
        return
      fi
      # Pass — SoA-driven sample. sampled_count/sampled_of are JSON numbers.
      if [[ -n "$window" ]]; then
        printf '{"criterion_id":"%s","severity":"%s","status":"pass","evidence":{"kind":"sample","value":"%s","sampled_count":%s,"sampled_of":%s,"window":"%s"}}' \
          "$(json_escape "$id")" "$(json_escape "$severity")" "$(json_escape "$summary")" \
          "$PLUGIN_IN_SCOPE_COUNT" "$SOA_IN_SCOPE_COUNT" "$(json_escape "$window")"
      else
        printf '{"criterion_id":"%s","severity":"%s","status":"pass","evidence":{"kind":"sample","value":"%s","sampled_count":%s,"sampled_of":%s}}' \
          "$(json_escape "$id")" "$(json_escape "$severity")" "$(json_escape "$summary")" \
          "$PLUGIN_IN_SCOPE_COUNT" "$SOA_IN_SCOPE_COUNT"
      fi
      ;;
    *)
      remedy="Control ${annex} in the Statement of Applicability has an invalid applicable value (\"${applicable}\"); it must be a TOML boolean true or false. Fix the [[plugins.audit-iso-27001.soa]] entry for ${annex}."
      build_fail_file "$id" "$severity" "$remedy"
      ;;
  esac
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
while IFS=$'\t' read -r id severity ek annex; do
  [[ -z "$id" ]] && continue
  case "$ek" in
    attestation) findings+=("$(build_attestation "$id" "$severity")") ;;
    sample)      findings+=("$(build_annex_sample "$id" "$severity" "$annex")") ;;
    *)           findings+=("$(build_kind_error "$id" "$severity" "$ek")") ;;
  esac
done < "$CRITERIA_TSV"

printf '['
first=1
if [[ ${#findings[@]} -gt 0 ]]; then
  for f in "${findings[@]}"; do
    if [[ $first -eq 1 ]]; then first=0; else printf ','; fi
    printf '\n  %s' "$f"
  done
fi
printf '\n]\n'
