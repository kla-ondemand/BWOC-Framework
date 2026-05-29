#!/usr/bin/env bash
#
# gws-calendar — gws/gws-calendar plugin entry (BWOC-76).
#
# A per-service plugin of the `gws` kind. Read-mostly Google Calendar adapter:
#   calendars — calendarList.list → the calendars this token can see.
#   events    — events.list on one calendar → an array of Calendar event entries.
# Each event is projected into the normative Calendar event shape
# (docs/en/PLUGINS.en.md §"Workspace Resource Schema"). It NEVER creates,
# updates, or deletes events — the kind is read-mostly by design (BWOC-72
# §Decision 4; write slices like `events.insert` are deferred with a confirm gate).
#
# The `bwoc gws` CLI (BWOC-74) is the invoker; it spawns this script with
# BWOC_GWS_OPERATION set to one of: calendars | events. The CLI subcommand
# `calendar list` maps to the `calendars` operation; `list` is accepted as an
# alias so direct invocation works either way.
#
# Sources the OAuth credential helpers from the sibling gws/gws-auth plugin so
# the Bearer-auth + rate-limit + refresh implementation lives exactly once
# (BWOC-72 §Decision 2). Sourcing is BASH_SOURCE-guarded on the sibling side, so
# importing the helpers does not run the gws-auth dispatcher.
#
# Contract:
#   stdin                  one-line JSON, e.g.
#                          {"operation":"calendars"}
#                          {"operation":"events"}
#                          {"operation":"events","calendar_id":"primary","max":50}
#   BWOC_GWS_OPERATION     fallback for .operation when stdin is empty
#   BWOC_WORKSPACE         absolute workspace root (token file resolution)
#   BWOC_PLUGIN_DIR        absolute path to THIS plugin's directory
#                          (used to find ../gws-auth/gws.sh)
#   BWOC_GWS_TOKEN         the OAuth2 access token — SECRET (inherited env)
#
# On success: exit 0 + a single JSON object on stdout. On error: a human
# message on stderr + non-zero exit (the CLI surfaces it).
#
# Security (Sila — Adinnaadana):
#   This plugin never reads or prints the token value. It hands the request to
#   the sibling's gws_curl (which sets the Bearer header) and projects Calendar's
#   JSON response — never the credential — into the output.

set -euo pipefail

# ── source sibling auth helpers ────────────────────────────────────────────
# Prefer the explicit BWOC_PLUGIN_DIR (set by the framework dispatcher); fall
# back to a script-relative path so the plugin remains testable without the
# dispatcher. Hardcoded relative to the workspace plugin tree — no PATH games.
_gws_cal_self_dir() {
  cd "$(dirname "${BASH_SOURCE[0]}")" && pwd
}

_gws_cal_resolve_helpers() {
  local candidates=()
  if [[ -n "${BWOC_PLUGIN_DIR:-}" ]]; then
    candidates+=("${BWOC_PLUGIN_DIR%/}/../gws-auth/gws.sh")
  fi
  candidates+=("$(_gws_cal_self_dir)/../gws-auth/gws.sh")
  local c
  for c in "${candidates[@]}"; do
    if [[ -n "$c" && -r "$c" ]]; then printf '%s' "$c"; return 0; fi
  done
  return 1
}

_AUTH_HELPERS="$(_gws_cal_resolve_helpers || true)"
if [[ -z "$_AUTH_HELPERS" ]]; then
  printf '%s\n' "gws-calendar: sibling helpers gws/gws-auth/gws.sh not found — install gws/gws-auth alongside this plugin." >&2
  exit 1
fi
# shellcheck source=/dev/null
source "$_AUTH_HELPERS"

# The sourced helpers set PLUGIN="gws-auth"; override AFTER sourcing so this
# plugin's diagnostics name itself.
PLUGIN="gws-calendar"
API_BASE="https://www.googleapis.com/calendar/v3"
DEFAULT_MAX=100
PAGE_CAP=100   # Calendar list page size we request per round-trip

# ── stdin + dependencies ───────────────────────────────────────────────────

for cmd in jq curl; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    printf '%s\n' "$PLUGIN: required command '$cmd' not found on PATH — install it, then retry." >&2
    exit 1
  fi
done

REQUEST="$(cat || true)"
req() { printf '%s' "$REQUEST" | jq -r "$1" 2>/dev/null || true; }

OPERATION=""
if [[ -n "$REQUEST" ]]; then OPERATION="$(req '.operation // empty')"; fi
if [[ -z "$OPERATION" ]]; then OPERATION="${BWOC_GWS_OPERATION:-}"; fi
if [[ -z "$OPERATION" ]]; then
  printf '%s\n' "$PLUGIN: no operation (set BWOC_GWS_OPERATION or pipe a JSON request carrying .operation)" >&2
  exit 2
fi

# ── Verb: calendars — calendarList.list → the visible calendars ────────────

do_calendars() {
  gws_assert_token || exit 2

  local collected page_token got max
  max=$DEFAULT_MAX
  collected="$(mktemp "${TMPDIR:-/tmp}/gws-cal-c.XXXXXX")"
  : >"$collected"
  page_token=""

  while :; do
    local args=(-G "${API_BASE}/users/me/calendarList"
      --data-urlencode "maxResults=${PAGE_CAP}")
    [[ -n "$page_token" ]] && args+=(--data-urlencode "pageToken=${page_token}")

    gws_curl "${args[@]}"
    gws_classify_status "calendars" "calendar list"

    printf '%s' "$HTTP_BODY" | jq -c '.items[]?' >>"$collected" 2>/dev/null || true
    got="$(awk 'END{print NR+0}' "$collected")"
    page_token="$(printf '%s' "$HTTP_BODY" | jq -r '.nextPageToken // empty' 2>/dev/null || true)"
    if [[ -z "$page_token" ]] || (( got >= max )); then break; fi
  done

  local out
  out="$(jq -s '
    def cal_entry:
      { calendar_id: .id, summary: (.summary // .summaryOverride // "(unnamed)") }
      + (if (.primary // false) then { primary: true } else {} end)
      + (if ((.accessRole // "") | length) > 0 then { access_role: .accessRole } else {} end);
    { ok: true, plugin: "gws-calendar", operation: "calendars",
      total: length, calendars: [ .[] | cal_entry ] }
  ' "$collected")"
  rm -f "$collected"
  printf '%s\n' "$out"
}

# ── Verb: events — events.list on one calendar → Calendar event entries ─────

do_events() {
  gws_assert_token || exit 2

  local calendar_id max page_size cal_enc
  calendar_id="$(req '.calendar_id // empty')"
  [[ -n "$calendar_id" ]] || calendar_id="primary"
  # Reject anything that could break out of the path segment; the CLI validates
  # too, but the plugin is defensible on its own (the gws-drive file_id guard).
  if [[ ! "$calendar_id" =~ ^[A-Za-z0-9_.@-]+$ ]]; then
    printf '%s\n' "$PLUGIN events: invalid calendar_id '$calendar_id' (expected [A-Za-z0-9_.@-])" >&2
    exit 2
  fi
  # Percent-encode for the URL path (an email-style id carries '@').
  cal_enc="$(jq -rn --arg s "$calendar_id" '$s|@uri')"

  max="$(req '.max // empty')"
  [[ "$max" =~ ^[0-9]+$ ]] || max=$DEFAULT_MAX
  (( max > 0 )) || max=$DEFAULT_MAX
  if (( max < PAGE_CAP )); then page_size=$max; else page_size=$PAGE_CAP; fi

  local collected page_token got
  collected="$(mktemp "${TMPDIR:-/tmp}/gws-cal-e.XXXXXX")"
  : >"$collected"
  page_token=""

  while :; do
    local args=(-G "${API_BASE}/calendars/${cal_enc}/events"
      --data-urlencode "maxResults=${page_size}"
      --data-urlencode "singleEvents=true"
      --data-urlencode "orderBy=startTime")
    [[ -n "$page_token" ]] && args+=(--data-urlencode "pageToken=${page_token}")

    gws_curl "${args[@]}"
    gws_classify_status "events" "calendar '${calendar_id}' events"

    printf '%s' "$HTTP_BODY" | jq -c '.items[]?' >>"$collected" 2>/dev/null || true
    got="$(awk 'END{print NR+0}' "$collected")"
    page_token="$(printf '%s' "$HTTP_BODY" | jq -r '.nextPageToken // empty' 2>/dev/null || true)"
    if [[ -z "$page_token" ]] || (( got >= max )); then break; fi
  done

  local out
  out="$(jq -s --argjson max "$max" --arg cal "$calendar_id" '
    def event_entry($cal):
      { event_id: .id,
        calendar_id: $cal,
        summary: (.summary // "(no title)"),
        start: (.start.dateTime // .start.date // ""),
        end:   (.end.dateTime   // .end.date   // "") }
      + (if ((.attendees // []) | length) > 0 then { attendees_count: (.attendees | length) } else {} end);
    (.[0:$max]) as $e
    | { ok: true, plugin: "gws-calendar", operation: "events",
        total: ($e | length), events: [ $e[] | event_entry($cal) ] }
  ' "$collected")"
  rm -f "$collected"
  printf '%s\n' "$out"
}

# ── Dispatch ───────────────────────────────────────────────────────────────

case "$OPERATION" in
  calendars|list) do_calendars ;;
  events)         do_events ;;
  *)
    printf '%s\n' "$PLUGIN: unknown operation '$OPERATION' (expected calendars | events)" >&2
    exit 2 ;;
esac
