#!/usr/bin/env bash
#
# gcloud-compute — workflow/gcloud-compute plugin entry (BWOC-69).
#
# Verbs: list (read) | start (write) | stop (write).
#
# The first write-capable gcloud slice (EPIC-9). `start` / `stop` flip a real
# VM — they cost money and interrupt workloads — so they carry an operator-
# confirm gate. Per docs/en/PLUGINS.en.md §"Write verbs" the gate lives at the
# CLI boundary (`bwoc gcloud compute`, BWOC-68); this plugin does NOT re-prompt.
# It DOES refuse a write unless the CLI-set confirmation marker
# BWOC_GCLOUD_CONFIRM is present (design note §Decision 3) — defense-in-depth so
# a direct plugin invocation can never bypass the gate. `delete` is deliberately
# not shipped (deferred, §Decision 2).
#
# Sources credential helpers from the sibling workflow/gcloud-auth plugin — the
# foundation auth resolution lives there exactly once (EPIC-8 §Decision 2).
# Sourcing is BASH_SOURCE-guarded on the sibling side, so importing the helpers
# does not run the gcloud-auth dispatcher.
#
# Contract:
#   stdin                  one-line JSON, e.g.
#                          {"operation":"list"}
#                          {"operation":"list","zone":"us-central1-a","project":"my-proj"}
#                          {"operation":"start","instance":"vm-1","zone":"us-central1-a"}
#                          {"operation":"stop","instance":"vm-1","zone":"us-central1-a"}
#   BWOC_GCLOUD_OPERATION  fallback for .operation when stdin is empty
#   BWOC_GCLOUD_CONFIRM    write-confirmation marker — set to a non-empty value
#                          by the CLI after the operator confirms a write. Read
#                          verbs ignore it; write verbs refuse without it.
#   BWOC_WORKSPACE         absolute workspace root (resolves the SA JSON path)
#   BWOC_PLUGIN_DIR        absolute path to THIS plugin's directory
#                          (used to find ../gcloud-auth/gcloud.sh)
#
# Hardening (#92): every user-supplied positional (the instance name) is passed
# to `gcloud` after a `--` end-of-options separator; every user-supplied flag
# value (zone, project) is bound with `=` in a single argv token. Neither can
# be parsed by `gcloud` as a flag.
#
# Security (Sila — Adinnaadana):
#   This plugin never reads any credential value. It only asks the local
#   `gcloud` CLI to act and surfaces its output.

set -euo pipefail

# ── source sibling auth helpers ────────────────────────────────────────────
# Prefer the explicit BWOC_PLUGIN_DIR (set by the framework dispatcher); fall
# back to a script-relative path so the plugin remains testable without the
# dispatcher. The source path is hardcoded relative to the workspace plugin
# tree — no PATH games (EPIC-8 design note §Decision 2).
_gcloud_compute_self_dir() {
  cd "$(dirname "${BASH_SOURCE[0]}")" && pwd
}

_gcloud_compute_resolve_helpers() {
  local candidates=()
  if [[ -n "${BWOC_PLUGIN_DIR:-}" ]]; then
    candidates+=("${BWOC_PLUGIN_DIR%/}/../gcloud-auth/gcloud.sh")
  fi
  candidates+=("$(_gcloud_compute_self_dir)/../gcloud-auth/gcloud.sh")
  local c
  for c in "${candidates[@]}"; do
    if [[ -n "$c" && -r "$c" ]]; then printf '%s' "$c"; return 0; fi
  done
  return 1
}

_AUTH_HELPERS="$(_gcloud_compute_resolve_helpers || true)"
if [[ -z "$_AUTH_HELPERS" ]]; then
  printf '%s\n' "gcloud-compute: sibling helpers workflow/gcloud-auth/gcloud.sh not found — install workflow/gcloud-auth alongside this plugin." >&2
  exit 1
fi
# shellcheck source=/dev/null
source "$_AUTH_HELPERS"

# The sourced helpers set PLUGIN="gcloud-auth"; override AFTER sourcing so this
# plugin's diagnostics name itself.
PLUGIN="gcloud-compute"

# ── helpers ──────────────────────────────────────────────────────────────────

# _gcloud_compute_field — extract a string field from the request; empty string
# when absent. (Kept side-effect-free: callers validate required fields
# explicitly so a missing value exits the main shell, not a subshell.)
_gcloud_compute_field() {
  local request="$1" field="$2"
  printf '%s' "$request" | jq -r --arg f "$field" '.[$f] // empty' 2>/dev/null || true
}

# _gcloud_compute_assert_confirmed — write-gate guard. The CLI sets
# BWOC_GCLOUD_CONFIRM to a non-empty value after the operator confirms. Without
# it, refuse the write and report "no change" with the reason (Dhammanupassana —
# never a bare failure, never a silent write). Exit 5.
_gcloud_compute_assert_confirmed() {
  local verb="$1"
  if [[ -n "${BWOC_GCLOUD_CONFIRM:-}" ]]; then return 0; fi
  jq -n --arg op "$verb" '{
    ok: false,
    plugin: "gcloud-compute",
    operation: $op,
    changed: false,
    reason: "unconfirmed-write",
    message: ("write verb '" + $op + "' requires operator confirmation; the bwoc gcloud compute CLI sets BWOC_GCLOUD_CONFIRM after a y/N prompt. Direct plugin invocation of a write verb is refused — no instance was changed.")
  }'
  printf '%s\n' "$PLUGIN $verb: refused — no confirmation marker (BWOC_GCLOUD_CONFIRM unset); no change made." >&2
  exit 5
}

# ── verbs ──────────────────────────────────────────────────────────────────

_gcloud_compute_list() {
  local request="$1"
  gcloud_assert_cli || exit 1
  gcloud_assert_authenticated || exit 3

  local zone project
  zone="$(_gcloud_compute_field "$request" "zone")"
  project="$(_gcloud_compute_field "$request" "project")"

  # Read verb — no positional user args. Optional filters bind with `=` in a
  # single argv token, so a `-`-leading value can never be parsed as a flag.
  local args=(compute instances list --format=json)
  [[ -n "$zone" ]] && args+=("--zones=$zone")
  [[ -n "$project" ]] && args+=("--project=$project")

  local raw
  if ! raw="$(gcloud "${args[@]}" 2>&1)"; then
    printf '%s\n' "$PLUGIN list: 'gcloud compute instances list' failed: $(printf '%s' "$raw" | head -c 300)" >&2
    exit 6
  fi

  printf '%s' "$raw" | jq '{
    ok: true,
    plugin: "gcloud-compute",
    operation: "list",
    total: (length),
    instances: [ .[] | {
      name:               (.name // null),
      zone:               (if .zone then (.zone | split("/") | last) else null end),
      machine_type:       (if .machineType then (.machineType | split("/") | last) else null end),
      status:             (.status // null),
      internal_ip:        (.networkInterfaces[0].networkIP // null),
      creation_timestamp: (.creationTimestamp // null)
    } ]
  }'
}

# _gcloud_compute_lifecycle — shared start/stop body. $1 = "start" | "stop".
_gcloud_compute_lifecycle() {
  local verb="$1" request="$2"
  # The write gate is the primary guard — check it first so a direct-invoke
  # bypass attempt is refused regardless of auth/CLI state (no silent write).
  _gcloud_compute_assert_confirmed "$verb"
  gcloud_assert_cli || exit 1
  gcloud_assert_authenticated || exit 3

  local instance zone project
  instance="$(_gcloud_compute_field "$request" "instance")"
  zone="$(_gcloud_compute_field "$request" "zone")"
  project="$(_gcloud_compute_field "$request" "project")"
  if [[ -z "$instance" ]]; then
    printf '%s\n' "$PLUGIN $verb: .instance required (pass {\"instance\":\"<name>\"})" >&2
    exit 2
  fi
  if [[ -z "$zone" ]]; then
    printf '%s\n' "$PLUGIN $verb: .zone required (compute instances are zonal; pass {\"zone\":\"<zone>\"})" >&2
    exit 2
  fi

  # Flag values (zone, project) bind with `=`; the instance name is the only
  # positional and goes after `--` (#92 hardening) so neither can be parsed as a
  # gcloud flag.
  local args=(compute instances "$verb" "--zone=$zone" --format=json)
  [[ -n "$project" ]] && args+=("--project=$project")
  args+=(-- "$instance")

  local raw
  if ! raw="$(gcloud "${args[@]}" 2>&1)"; then
    printf '%s\n' "$PLUGIN $verb: 'gcloud compute instances $verb $instance' failed: $(printf '%s' "$raw" | head -c 300)" >&2
    exit 6
  fi

  # `gcloud ... --format=json` emits the operation/instance result; surface it
  # under .result rather than re-deriving a BWOC-owned shape (workflow passthrough).
  local result
  if ! result="$(printf '%s' "$raw" | jq '.' 2>/dev/null)"; then
    result="null"
  fi
  jq -n \
    --arg op "$verb" \
    --arg instance "$instance" \
    --arg zone "$zone" \
    --argjson result "$result" '{
      ok: true,
      plugin: "gcloud-compute",
      operation: $op,
      instance: $instance,
      zone: $zone,
      changed: true,
      result: $result
    }'
}

# ── dispatch ───────────────────────────────────────────────────────────────

_gcloud_compute_main() {
  if ! command -v jq >/dev/null 2>&1; then
    printf '%s\n' "$PLUGIN: required command 'jq' not found on PATH" >&2
    exit 1
  fi

  local request operation
  request="$(cat || true)"
  operation=""
  if [[ -n "$request" ]]; then
    operation="$(printf '%s' "$request" | jq -r '.operation // empty' 2>/dev/null || true)"
  fi
  if [[ -z "$operation" ]]; then operation="${BWOC_GCLOUD_OPERATION:-}"; fi
  if [[ -z "$operation" ]]; then
    printf '%s\n' "$PLUGIN: no operation (set BWOC_GCLOUD_OPERATION or pipe a JSON request carrying .operation)" >&2
    exit 2
  fi

  case "$operation" in
    list)  _gcloud_compute_list "$request" ;;
    start) _gcloud_compute_lifecycle "start" "$request" ;;
    stop)  _gcloud_compute_lifecycle "stop" "$request" ;;
    *)
      printf '%s\n' "$PLUGIN: unknown operation '$operation' (expected list | start | stop)" >&2
      exit 2 ;;
  esac
}

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  _gcloud_compute_main
fi
