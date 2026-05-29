#!/usr/bin/env bash
#
# gws-drive — gws/gws-drive plugin entry (BWOC-75).
#
# A per-service plugin of the `gws` kind. Read-mostly Google Drive adapter:
# lists files (Drive files.list) and reads a single file's metadata
# (files.get) — each projected into the normative Drive file shape
# (docs/en/PLUGINS.en.md §"Workspace Resource Schema"). It NEVER writes back to
# Drive — the kind is read-mostly by design (BWOC-72 §Decision 4).
#
# Sources the OAuth credential helpers from the sibling gws/gws-auth plugin so
# the Bearer-auth + rate-limit + refresh implementation lives exactly once
# (BWOC-72 §Decision 2, the gcloud-* family shape). Sourcing is BASH_SOURCE-
# guarded on the sibling side, so importing the helpers does not run the
# gws-auth dispatcher.
#
# Contract:
#   stdin                  one-line JSON, e.g.
#                          {"operation":"list"}
#                          {"operation":"list","query":"mimeType='application/pdf'","max":50}
#                          {"operation":"get","file_id":"1AbC_dEf"}
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
#   the sibling's gws_curl (which sets the Bearer header) and projects Drive's
#   JSON response — never the credential — into the output.

set -euo pipefail

# ── source sibling auth helpers ────────────────────────────────────────────
# Prefer the explicit BWOC_PLUGIN_DIR (set by the framework dispatcher); fall
# back to a script-relative path so the plugin remains testable without the
# dispatcher. Hardcoded relative to the workspace plugin tree — no PATH games.
_gws_drive_self_dir() {
  cd "$(dirname "${BASH_SOURCE[0]}")" && pwd
}

_gws_drive_resolve_helpers() {
  local candidates=()
  if [[ -n "${BWOC_PLUGIN_DIR:-}" ]]; then
    candidates+=("${BWOC_PLUGIN_DIR%/}/../gws-auth/gws.sh")
  fi
  candidates+=("$(_gws_drive_self_dir)/../gws-auth/gws.sh")
  local c
  for c in "${candidates[@]}"; do
    if [[ -n "$c" && -r "$c" ]]; then printf '%s' "$c"; return 0; fi
  done
  return 1
}

_AUTH_HELPERS="$(_gws_drive_resolve_helpers || true)"
if [[ -z "$_AUTH_HELPERS" ]]; then
  printf '%s\n' "gws-drive: sibling helpers gws/gws-auth/gws.sh not found — install gws/gws-auth alongside this plugin." >&2
  exit 1
fi
# shellcheck source=/dev/null
source "$_AUTH_HELPERS"

# The sourced helpers set PLUGIN="gws-auth"; override AFTER sourcing so this
# plugin's diagnostics name itself.
PLUGIN="gws-drive"
API_BASE="https://www.googleapis.com/drive/v3"
DEFAULT_MAX=100
PAGE_CAP=100   # Drive files.list page size we request per round-trip

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

# Field projection shared by `list` and `get`: a Drive REST file object → the
# normative Drive file entry (PLUGINS.en.md §Workspace Resource Schema). Optional
# fields (owners, web_view_link) are omitted when absent — never null.
DRIVE_ENTRY='
  def drive_entry:
    { file_id: .id, name: .name, mime_type: .mimeType, modified_time: .modifiedTime }
    + (if ((.owners // []) | length) > 0 then { owners: [ .owners[].emailAddress ] } else {} end)
    + (if ((.webViewLink // "") | length) > 0 then { web_view_link: .webViewLink } else {} end);
'

# ── Verb: list — Drive files.list → Drive file entries (paginated) ──────────

do_list() {
  gws_assert_token || exit 2

  local query max page_size
  query="$(req '.query // empty')"
  max="$(req '.max // empty')"
  [[ "$max" =~ ^[0-9]+$ ]] || max=$DEFAULT_MAX
  (( max > 0 )) || max=$DEFAULT_MAX
  if (( max < PAGE_CAP )); then page_size=$max; else page_size=$PAGE_CAP; fi

  local collected page_token got
  collected="$(mktemp "${TMPDIR:-/tmp}/gws-drive.XXXXXX")"
  : >"$collected"
  page_token=""

  while :; do
    local args=(-G "${API_BASE}/files"
      --data-urlencode "pageSize=${page_size}"
      --data-urlencode "fields=nextPageToken,files(id,name,mimeType,modifiedTime,owners(emailAddress),webViewLink)"
      --data-urlencode "supportsAllDrives=true"
      --data-urlencode "includeItemsFromAllDrives=true")
    [[ -n "$query" ]] && args+=(--data-urlencode "q=${query}")
    [[ -n "$page_token" ]] && args+=(--data-urlencode "pageToken=${page_token}")

    gws_curl "${args[@]}"
    gws_classify_status "list" "Drive files"

    printf '%s' "$HTTP_BODY" | jq -c '.files[]?' >>"$collected" 2>/dev/null || true
    got="$(awk 'END{print NR+0}' "$collected")"
    page_token="$(printf '%s' "$HTTP_BODY" | jq -r '.nextPageToken // empty' 2>/dev/null || true)"
    if [[ -z "$page_token" ]] || (( got >= max )); then break; fi
  done

  local out
  out="$(jq -s --argjson max "$max" "
    $DRIVE_ENTRY
    (.[0:\$max]) as \$f
    | { ok: true, plugin: \"gws-drive\", operation: \"list\",
        total: (\$f | length), files: [ \$f[] | drive_entry ] }
  " "$collected")"
  rm -f "$collected"
  printf '%s\n' "$out"
}

# ── Verb: get — Drive files.get metadata → one Drive file entry ────────────

do_get() {
  gws_assert_token || exit 2

  local file_id
  file_id="$(req '.file_id // empty')"
  if [[ -z "$file_id" ]]; then
    printf '%s\n' "$PLUGIN get: .file_id is required (pass {\"file_id\":\"<id>\"})" >&2
    exit 2
  fi
  # Drive file ids are URL-safe base64-ish; reject anything else so a crafted id
  # can never inject a path segment or query into the request URL.
  if [[ ! "$file_id" =~ ^[A-Za-z0-9_-]+$ ]]; then
    printf '%s\n' "$PLUGIN get: invalid file_id '$file_id' (expected [A-Za-z0-9_-])" >&2
    exit 2
  fi

  gws_curl -G "${API_BASE}/files/${file_id}" \
    --data-urlencode "fields=id,name,mimeType,modifiedTime,owners(emailAddress),webViewLink" \
    --data-urlencode "supportsAllDrives=true"
  gws_classify_status "get" "Drive file '${file_id}'"

  printf '%s' "$HTTP_BODY" | jq "
    $DRIVE_ENTRY
    { ok: true, plugin: \"gws-drive\", operation: \"get\", file: (. | drive_entry) }
  "
}

# ── Dispatch ───────────────────────────────────────────────────────────────

case "$OPERATION" in
  list) do_list ;;
  get)  do_get ;;
  *)
    printf '%s\n' "$PLUGIN: unknown operation '$OPERATION' (expected list | get)" >&2
    exit 2 ;;
esac
