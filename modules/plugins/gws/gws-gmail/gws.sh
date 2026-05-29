#!/usr/bin/env bash
#
# gws-gmail — gws/gws-gmail plugin entry (BWOC-76).
#
# A per-service plugin of the `gws` kind. Read-mostly Google Gmail adapter:
#   search  — threads.list (optionally with a Gmail `q` query), enriched per
#             thread via threads.get(metadata) → an array of Gmail thread entries.
#   show    — threads.get(metadata) for one thread → a single Gmail thread entry.
#   labels  — labels.list → the user's label set.
# Each thread is projected into the normative Gmail thread shape
# (docs/en/PLUGINS.en.md §"Workspace Resource Schema"). It NEVER sends mail,
# modifies labels, or mutates anything — the kind is read-mostly by design
# (BWOC-72 §Decision 4; write slices like `send` are deferred with a confirm gate).
#
# The `bwoc gws` CLI (BWOC-74) is the invoker; it spawns this script with
# BWOC_GWS_OPERATION set to one of: search | show | labels. The conceptual verb
# names from the EPIC-13 brief — threads (=search), message/messages (=show) —
# are accepted as aliases so direct invocation works either way.
#
# Sources the OAuth credential helpers from the sibling gws/gws-auth plugin so
# the Bearer-auth + rate-limit + refresh implementation lives exactly once
# (BWOC-72 §Decision 2). Sourcing is BASH_SOURCE-guarded on the sibling side, so
# importing the helpers does not run the gws-auth dispatcher.
#
# Contract:
#   stdin                  one-line JSON, e.g.
#                          {"operation":"search"}
#                          {"operation":"search","query":"from:me is:unread","max":25}
#                          {"operation":"show","thread_id":"18ab..."}
#                          {"operation":"labels"}
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
#   the sibling's gws_curl (which sets the Bearer header) and projects Gmail's
#   JSON response — never the credential — into the output.

set -euo pipefail

# ── source sibling auth helpers ────────────────────────────────────────────
# Prefer the explicit BWOC_PLUGIN_DIR (set by the framework dispatcher); fall
# back to a script-relative path so the plugin remains testable without the
# dispatcher. Hardcoded relative to the workspace plugin tree — no PATH games.
_gws_gmail_self_dir() {
  cd "$(dirname "${BASH_SOURCE[0]}")" && pwd
}

_gws_gmail_resolve_helpers() {
  local candidates=()
  if [[ -n "${BWOC_PLUGIN_DIR:-}" ]]; then
    candidates+=("${BWOC_PLUGIN_DIR%/}/../gws-auth/gws.sh")
  fi
  candidates+=("$(_gws_gmail_self_dir)/../gws-auth/gws.sh")
  local c
  for c in "${candidates[@]}"; do
    if [[ -n "$c" && -r "$c" ]]; then printf '%s' "$c"; return 0; fi
  done
  return 1
}

_AUTH_HELPERS="$(_gws_gmail_resolve_helpers || true)"
if [[ -z "$_AUTH_HELPERS" ]]; then
  printf '%s\n' "gws-gmail: sibling helpers gws/gws-auth/gws.sh not found — install gws/gws-auth alongside this plugin." >&2
  exit 1
fi
# shellcheck source=/dev/null
source "$_AUTH_HELPERS"

# The sourced helpers set PLUGIN="gws-auth"; override AFTER sourcing so this
# plugin's diagnostics name itself.
PLUGIN="gws-gmail"
API_BASE="https://gmail.googleapis.com/gmail/v1/users/me"
DEFAULT_MAX=100
PAGE_CAP=100   # Gmail threads.list page size we request per round-trip

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

# Projection: a Gmail threads.get(metadata) object → the normative Gmail thread
# entry (PLUGINS.en.md §Workspace Resource Schema). Required: thread_id, subject,
# from, last_message_time. Optional: snippet, labels (omitted when empty, never
# null). $snip overrides the snippet (threads.list carries one; show uses the
# latest message's snippet). Header lookup is case-insensitive; `from` is the
# latest message's sender, `subject` the first non-empty Subject; the timestamp
# comes from the latest message's internalDate (epoch ms → ISO 8601 via todate).
THREAD_ENTRY='
  def hdr($msgs; $name):
    [ $msgs[]? | .payload.headers[]?
      | select((.name // "" | ascii_downcase) == ($name | ascii_downcase))
      | .value ];
  def thread_entry($snip):
    (.messages // []) as $m
    | ($m | last) as $latest
    | (hdr($m; "subject") | map(select(length > 0)) | first // "(no subject)") as $subject
    | ((hdr([$latest]; "from") | first) // (hdr($m; "from") | last) // "(unknown sender)") as $from
    | (($latest.internalDate // "0") | tonumber / 1000 | floor | todate) as $when
    | ([ $m[]? | .labelIds[]? ] | unique) as $labels
    | (if ($snip | length) > 0 then $snip else ($latest.snippet // "") end) as $snippet
    | { thread_id: .id, subject: $subject, from: $from, last_message_time: $when }
      + (if ($snippet | length) > 0 then { snippet: $snippet } else {} end)
      + (if ($labels  | length) > 0 then { labels:  $labels  } else {} end);
'

# thread_get_meta <thread_id> — fetch threads.get(format=metadata) for one id,
# leaving the response in HTTP_STATUS / HTTP_BODY. Headers requested are limited
# to Subject/From/Date so the response stays small.
thread_get_meta() {
  gws_curl -G "${API_BASE}/threads/$1" \
    --data-urlencode "format=metadata" \
    --data-urlencode "metadataHeaders=Subject" \
    --data-urlencode "metadataHeaders=From" \
    --data-urlencode "metadataHeaders=Date"
}

# require a thread id to be a safe path segment before it reaches the URL.
assert_thread_id() {
  local id="$1"
  if [[ -z "$id" ]]; then
    printf '%s\n' "$PLUGIN $2: .thread_id is required (pass {\"thread_id\":\"<id>\"})" >&2
    exit 2
  fi
  if [[ ! "$id" =~ ^[A-Za-z0-9_-]+$ ]]; then
    printf '%s\n' "$PLUGIN $2: invalid thread_id '$id' (expected [A-Za-z0-9_-])" >&2
    exit 2
  fi
}

# ── Verb: search — threads.list (+ per-thread enrich) → thread entries ──────

do_search() {
  gws_assert_token || exit 2

  local query max page_size
  query="$(req '.query // empty')"
  max="$(req '.max // empty')"
  [[ "$max" =~ ^[0-9]+$ ]] || max=$DEFAULT_MAX
  (( max > 0 )) || max=$DEFAULT_MAX
  if (( max < PAGE_CAP )); then page_size=$max; else page_size=$PAGE_CAP; fi

  # 1. Page threads.list, collecting {id, snippet} stubs up to `max`.
  local stubs page_token got
  stubs="$(mktemp "${TMPDIR:-/tmp}/gws-gmail-l.XXXXXX")"
  : >"$stubs"
  page_token=""
  while :; do
    local args=(-G "${API_BASE}/threads"
      --data-urlencode "maxResults=${page_size}")
    [[ -n "$query" ]] && args+=(--data-urlencode "q=${query}")
    [[ -n "$page_token" ]] && args+=(--data-urlencode "pageToken=${page_token}")

    gws_curl "${args[@]}"
    gws_classify_status "search" "Gmail threads"

    printf '%s' "$HTTP_BODY" | jq -c '.threads[]? | {id, snippet}' >>"$stubs" 2>/dev/null || true
    got="$(awk 'END{print NR+0}' "$stubs")"
    page_token="$(printf '%s' "$HTTP_BODY" | jq -r '.nextPageToken // empty' 2>/dev/null || true)"
    if [[ -z "$page_token" ]] || (( got >= max )); then break; fi
  done

  # 2. Enrich each stub (up to max) via threads.get(metadata). A single thread
  # that 404s (deleted between list and get) is skipped; systemic auth/rate
  # errors still abort via gws_classify_status.
  local entries id snip
  entries="$(mktemp "${TMPDIR:-/tmp}/gws-gmail-e.XXXXXX")"
  : >"$entries"
  local n=0
  while IFS= read -r stub; do
    (( n < max )) || break
    n=$((n + 1))
    id="$(printf '%s' "$stub" | jq -r '.id // empty')"
    snip="$(printf '%s' "$stub" | jq -r '.snippet // ""')"
    [[ -n "$id" ]] || continue
    thread_get_meta "$id"
    case "$HTTP_STATUS" in
      2*) : ;;
      401|403|429|0) gws_classify_status "search" "Gmail thread '${id}'" ;;
      *)
        printf '%s\n' "$PLUGIN search: skipping thread '${id}' (HTTP $HTTP_STATUS)" >&2
        continue ;;
    esac
    printf '%s' "$HTTP_BODY" | jq -c --arg snip "$snip" "$THREAD_ENTRY thread_entry(\$snip)" >>"$entries" 2>/dev/null || true
  done <"$stubs"

  jq -s '{ ok: true, plugin: "gws-gmail", operation: "search",
           total: length, threads: . }' "$entries"
  rm -f "$stubs" "$entries"
}

# ── Verb: show — threads.get(metadata) for one thread → one thread entry ────

do_show() {
  gws_assert_token || exit 2

  local thread_id
  thread_id="$(req '.thread_id // empty')"
  assert_thread_id "$thread_id" "show"

  thread_get_meta "$thread_id"
  gws_classify_status "show" "Gmail thread '${thread_id}'"

  # The CLI's human formatter reads the thread fields at the top level, so the
  # entry is spread into the envelope rather than nested under a "thread" key.
  printf '%s' "$HTTP_BODY" | jq "
    $THREAD_ENTRY
    { ok: true, plugin: \"gws-gmail\", operation: \"show\" } + thread_entry(\"\")
  "
}

# ── Verb: labels — labels.list → the user's label set ──────────────────────

do_labels() {
  gws_assert_token || exit 2

  gws_curl -G "${API_BASE}/labels"
  gws_classify_status "labels" "Gmail labels"

  printf '%s' "$HTTP_BODY" | jq '
    { ok: true, plugin: "gws-gmail", operation: "labels",
      total: ((.labels // []) | length),
      labels: [ .labels[]? | { label_id: .id, name: .name, type: .type } ] }
  '
}

# ── Dispatch ───────────────────────────────────────────────────────────────

case "$OPERATION" in
  search|threads)        do_search ;;
  show|message|messages) do_show ;;
  labels)                do_labels ;;
  *)
    printf '%s\n' "$PLUGIN: unknown operation '$OPERATION' (expected search | show | labels)" >&2
    exit 2 ;;
esac
