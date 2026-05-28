#!/usr/bin/env bash
#
# gws-auth — gws/gws-auth plugin entry + shared OAuth credential helpers (BWOC-75).
#
# Two roles in one file (the gcloud-auth pattern, BWOC-53):
#
#   1. As an ENTRY SCRIPT (executed directly): dispatches the `status` verb per
#      BWOC_GWS_OPERATION + the one-line JSON request on stdin. Emits one JSON
#      object on stdout on success; a human diagnostic on stderr + non-zero exit
#      on error. `status` reports whether a token is present, its granted scopes,
#      the account, and expiry — but NEVER prints the token value.
#
#   2. As a HELPER LIBRARY (sourced by sibling gws-* plugins): exports the OAuth
#      credential surface — gws_resolve_token / gws_auth_header / gws_curl /
#      gws_classify_status / gws_refresh_if_expired — so gws-drive (and future
#      gws-gmail / gws-calendar) reuse one Bearer-auth + rate-limit + refresh
#      implementation instead of re-rolling it. The dispatcher is BASH_SOURCE-
#      guarded so sourcing is a pure import (no stdin consumed, no verb run).
#
# Contract:
#   stdin                  one-line JSON, e.g. {"operation":"status"}
#   BWOC_GWS_OPERATION     fallback for .operation when stdin is empty
#   BWOC_WORKSPACE         absolute workspace root (resolves the token file path)
#   BWOC_PLUGIN_DIR        absolute path to this plugin directory (informational)
#   BWOC_GWS_TOKEN         the OAuth2 access token — SECRET (inherited env)
#
# Auth source precedence (BWOC-72 design note §3):
#   1. BWOC_GWS_TOKEN env                      — transient / CI; no metadata.
#   2. <workspace>/.bwoc/secrets/gws-token.json — chmod 600, gitignored; holds
#      the access token (+ optional refresh_token / expiry / scopes / account /
#      client_id / client_secret). gws-auth refreshes when expired + refreshable.
#
# Security (Sila — Adinnaadana):
#   The token value is handed only to curl's `Authorization: Bearer` header. It
#   is never echoed to plugin OUTPUT, never logged, never serialized into any
#   JSON envelope. `gws_resolve_token` echoes the token only so a caller can
#   capture it into a variable (it never reaches stdout-as-result). auth.toml
#   ships SHAPE only — no value. A malformed auth.toml cannot leak a token
#   because the token never lives in any tracked file.

set -euo pipefail

# When sourced, sibling plugins override PLUGIN to their own identifier AFTER the
# source line — so this default only names the dispatcher in this file.
PLUGIN="gws-auth"

GWS_TOKEN_RELATIVE=".bwoc/secrets/gws-token.json"
GWS_OAUTH_TOKEN_URL="https://oauth2.googleapis.com/token"
GWS_MAX_ATTEMPTS=4           # 429 backoff: total tries before giving up

# Globals set by gws_curl (read by callers + gws_classify_status).
HTTP_STATUS=0
HTTP_BODY=""

# ── helpers (safe to source) ───────────────────────────────────────────────

# gws_token_file_path — absolute path to the workspace-local token JSON, if
# present. Empty string when absent. Honors BWOC_WORKSPACE.
gws_token_file_path() {
  local root="${BWOC_WORKSPACE:-.}"
  local p="${root%/}/${GWS_TOKEN_RELATIVE}"
  if [[ -f "$p" ]]; then printf '%s' "$p"; fi
}

# _gws_token_file_safe — true when the token file is absent OR owner-only.
# Refuses a group/world-readable secrets file (the figma BWOC-64 guard).
_gws_token_file_safe() {
  local f="$1"
  [[ -f "$f" ]] || return 0
  local perms
  perms="$(stat -f '%Lp' "$f" 2>/dev/null || stat -c '%a' "$f" 2>/dev/null || echo '')"
  if [[ -n "$perms" && "${perms: -2}" != "00" ]]; then
    printf '%s\n' "$PLUGIN: refusing to read $f — it is group/world-readable (mode $perms); run 'chmod 600 $f'." >&2
    return 1
  fi
  return 0
}

# _gws_token_field <jq-filter> — echo a field from the token JSON file, or
# nothing. Empty when the file is absent/unsafe/malformed. Never used for the
# token value in any OUTPUT path — only to set headers / report metadata.
_gws_token_field() {
  local f
  f="$(gws_token_file_path)"
  [[ -n "$f" ]] || return 0
  _gws_token_file_safe "$f" || return 0
  command -v jq >/dev/null 2>&1 || return 0
  jq -r "$1 // empty" "$f" 2>/dev/null || true
}

# gws_active_source — echoes one of: env | secrets-file | none.
# Precedence: BWOC_GWS_TOKEN env -> secrets file -> none.
gws_active_source() {
  if [[ -n "${BWOC_GWS_TOKEN:-}" ]]; then printf 'env'; return 0; fi
  if [[ -n "$(gws_token_file_path)" ]]; then printf 'secrets-file'; return 0; fi
  printf 'none'
}

# gws_resolve_token — echo the OAuth2 access token (env first, then the secrets
# file). Empty string when none. Echoes the value ONLY so a caller can capture
# it into a variable for the Authorization header — it is never an OUTPUT field.
gws_resolve_token() {
  if [[ -n "${BWOC_GWS_TOKEN:-}" ]]; then printf '%s' "$BWOC_GWS_TOKEN"; return 0; fi
  _gws_token_field '.access_token'
}

# gws_token_scopes — space-separated granted scopes from the secrets file
# (.scopes[] array or a .scope space-string). Empty when unknown (an env token
# carries no scope metadata).
gws_token_scopes() {
  local f
  f="$(gws_token_file_path)"
  [[ -n "$f" ]] || return 0
  _gws_token_file_safe "$f" || return 0
  command -v jq >/dev/null 2>&1 || return 0
  jq -r '
    if (.scopes | type) == "array" then (.scopes | join(" "))
    elif (.scope | type) == "string" then .scope
    else empty end' "$f" 2>/dev/null || true
}

# gws_token_account — account email from the secrets file (.account). Empty
# when unknown.
gws_token_account() { _gws_token_field '.account'; }

# gws_token_expiry — RFC3339 / epoch expiry from the secrets file (.expiry).
# Empty when unknown.
gws_token_expiry() { _gws_token_field '.expiry'; }

# _gws_epoch <when> — parse an ISO-8601 / RFC3339 / epoch timestamp into epoch
# seconds; empty on failure. GNU vs BSD `date` is detected explicitly (BSD
# `date -d` silently misparses rather than erroring, so probing it is unsafe).
_gws_epoch() {
  local w="$1"
  [[ -n "$w" ]] || return 0
  if [[ "$w" =~ ^[0-9]+$ ]]; then printf '%s' "$w"; return 0; fi
  if date --version >/dev/null 2>&1; then
    date -d "$w" +%s 2>/dev/null || true            # GNU date
  else
    local trimmed="${w%Z}"; trimmed="${trimmed%%.*}"  # strip fractional + Z
    date -j -f "%Y-%m-%dT%H:%M:%S" "$trimmed" +%s 2>/dev/null || true  # BSD date
  fi
}

# gws_token_expired — return 0 (true) when the secrets-file token has an expiry
# in the past (60s skew). Returns 1 when not expired OR expiry is unknown (an
# absent expiry is treated as "not known to be expired").
gws_token_expired() {
  local exp now exp_s
  exp="$(gws_token_expiry)"
  [[ -n "$exp" ]] || return 1
  exp_s="$(_gws_epoch "$exp")"
  [[ -n "$exp_s" ]] || return 1
  now="$(date +%s)"
  if (( exp_s <= now + 60 )); then return 0; fi
  return 1
}

# gws_token_refreshable — return 0 when the secrets file carries the trio needed
# for an offline refresh (refresh_token + client_id + client_secret).
gws_token_refreshable() {
  [[ -n "$(_gws_token_field '.refresh_token')" \
     && -n "$(_gws_token_field '.client_id')" \
     && -n "$(_gws_token_field '.client_secret')" ]]
}

# gws_refresh_if_expired — when the secrets-file token is expired AND
# refreshable, POST a refresh_token grant to Google's OAuth2 endpoint, then
# rewrite the token file in place (chmod 600, other fields preserved). No-op for
# an env token, a non-expired token, or a token with no refresh trio. Returns 0
# on no-op or successful refresh; non-zero (with a clear stderr note) when a
# refresh was needed but could not be performed — the caller proceeds with the
# stale token and surfaces the resulting 401 cleanly.
gws_refresh_if_expired() {
  local f
  f="$(gws_token_file_path)"
  [[ -n "$f" ]] || return 0                       # env token / none — nothing to refresh
  _gws_token_file_safe "$f" || return 0
  gws_token_expired || return 0                   # still valid (or expiry unknown)

  if ! gws_token_refreshable; then
    printf '%s\n' "$PLUGIN: token in $f is expired and not refreshable (need refresh_token + client_id + client_secret) — re-authorize and update the file." >&2
    return 1
  fi
  if ! command -v curl >/dev/null 2>&1 || ! command -v jq >/dev/null 2>&1; then
    printf '%s\n' "$PLUGIN: cannot refresh token — 'curl' and 'jq' are required." >&2
    return 1
  fi

  local refresh client_id client_secret resp access expires_in
  refresh="$(_gws_token_field '.refresh_token')"
  client_id="$(_gws_token_field '.client_id')"
  client_secret="$(_gws_token_field '.client_secret')"

  resp="$(curl -sS -X POST "$GWS_OAUTH_TOKEN_URL" \
    --data-urlencode "client_id=${client_id}" \
    --data-urlencode "client_secret=${client_secret}" \
    --data-urlencode "refresh_token=${refresh}" \
    --data-urlencode "grant_type=refresh_token" 2>/dev/null || true)"
  access="$(printf '%s' "$resp" | jq -r '.access_token // empty' 2>/dev/null || true)"
  if [[ -z "$access" ]]; then
    printf '%s\n' "$PLUGIN: token refresh failed — Google's OAuth2 endpoint did not return an access_token. Re-authorize and update $f." >&2
    return 1
  fi
  expires_in="$(printf '%s' "$resp" | jq -r '.expires_in // empty' 2>/dev/null || true)"

  # Rewrite the file: new access_token + recomputed expiry; preserve all other
  # fields. Written via a chmod-600 temp file + atomic mv so the secret is never
  # briefly world-readable on disk.
  local new_expiry="" tmp
  if [[ "$expires_in" =~ ^[0-9]+$ ]]; then
    new_expiry="$(_gws_iso_in "$expires_in")"
  fi
  tmp="$(mktemp "${TMPDIR:-/tmp}/gws-token.XXXXXX")"
  chmod 600 "$tmp" 2>/dev/null || true
  if jq --arg tok "$access" --arg exp "$new_expiry" \
       '.access_token=$tok | (if $exp=="" then . else .expiry=$exp end)' \
       "$f" >"$tmp" 2>/dev/null; then
    mv "$tmp" "$f"
    chmod 600 "$f" 2>/dev/null || true
  else
    rm -f "$tmp"
    printf '%s\n' "$PLUGIN: token refreshed but could not rewrite $f — leaving it unchanged." >&2
    return 1
  fi
  return 0
}

# _gws_iso_in <seconds> — RFC3339 UTC timestamp <seconds> from now. GNU vs BSD
# `date` detected explicitly (same reason as _gws_epoch).
_gws_iso_in() {
  local s="$1" t=$(( $(date +%s) + s ))
  if date --version >/dev/null 2>&1; then
    date -u -d "@${t}" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || true  # GNU date
  else
    date -u -r "${t}" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || true   # BSD date
  fi
}

# gws_auth_header — echo the full `Authorization: Bearer <token>` header line so
# a sibling can pass it straight to curl `-H "$(gws_auth_header)"`. Empty when no
# token. The token never reaches OUTPUT — only curl's header.
gws_auth_header() {
  local tok
  tok="$(gws_resolve_token)"
  [[ -n "$tok" ]] || return 0
  printf 'Authorization: Bearer %s' "$tok"
}

# gws_assert_token — exit-style guard. Returns 2 with a clear diagnostic when no
# token is resolvable from any source.
gws_assert_token() {
  if [[ -z "$(gws_resolve_token)" ]]; then
    printf '%s\n' "$PLUGIN: no OAuth token — set BWOC_GWS_TOKEN, or drop an access token into \$BWOC_WORKSPACE/$GWS_TOKEN_RELATIVE (chmod 600). Never commit the token." >&2
    return 2
  fi
  return 0
}

# gws_curl <curl args...> — perform an authenticated Workspace REST request.
# Refreshes an expired+refreshable token first, prepends the Bearer + JSON Accept
# headers, and honors HTTP 429 Retry-After (squared fallback) up to
# GWS_MAX_ATTEMPTS. Sets HTTP_STATUS (0 on transport failure) + HTTP_BODY.
# Requires curl; the caller validates jq/curl presence up front.
gws_curl() {
  gws_refresh_if_expired || true     # best-effort; a stale token surfaces as 401
  local hdr tmp_h tmp_b tmp_e attempt=0 code ra
  hdr="$(gws_auth_header)"
  tmp_h="$(mktemp "${TMPDIR:-/tmp}/gws-h.XXXXXX")"
  tmp_b="$(mktemp "${TMPDIR:-/tmp}/gws-b.XXXXXX")"
  tmp_e="$(mktemp "${TMPDIR:-/tmp}/gws-e.XXXXXX")"
  while :; do
    attempt=$((attempt + 1))
    : >"$tmp_h"; : >"$tmp_b"; : >"$tmp_e"
    code="$(curl -sS \
      -H "$hdr" \
      -H "Accept: application/json" \
      -D "$tmp_h" -o "$tmp_b" -w '%{http_code}' \
      "$@" 2>"$tmp_e")" || code=""
    if [[ -z "$code" ]]; then
      HTTP_STATUS=0
      HTTP_BODY="$(cat "$tmp_e" 2>/dev/null || true)"
      rm -f "$tmp_h" "$tmp_b" "$tmp_e"
      return 0
    fi
    HTTP_STATUS="$code"
    HTTP_BODY="$(cat "$tmp_b" 2>/dev/null || true)"
    if [[ "$code" == "429" && $attempt -lt $GWS_MAX_ATTEMPTS ]]; then
      ra="$(awk 'tolower($1)=="retry-after:"{gsub(/\r/,"",$2); print $2; exit}' "$tmp_h")"
      [[ "$ra" =~ ^[0-9]+$ ]] || ra=$((attempt * attempt))
      sleep "$ra"
      continue
    fi
    rm -f "$tmp_h" "$tmp_b" "$tmp_e"
    return 0
  done
}

# gws_classify_status <verb> [resource] — return 0 on 2xx; otherwise emit a clear
# diagnostic and exit. 403 names the scope gap (an OAuth token consented to one
# service scope cannot read another), never a bare failure.
gws_classify_status() {
  local verb="$1" resource="${2:-resource}" scopes
  case "$HTTP_STATUS" in
    2*) return 0 ;;
    0)
      printf '%s\n' "$PLUGIN $verb: network/transport error reaching Google Workspace: $(printf '%s' "$HTTP_BODY" | head -c 300)" >&2
      exit 6 ;;
    401)
      printf '%s\n' "$PLUGIN $verb: authentication failed (HTTP 401) — the OAuth token is missing/expired/revoked. Refresh it or set a fresh BWOC_GWS_TOKEN." >&2
      exit 3 ;;
    403)
      scopes="$(gws_token_scopes)"
      printf '%s\n' "$PLUGIN $verb: token lacks the required scope for $resource (HTTP 403) — Workspace scopes are per-service and consent-bound; a token granted only one scope cannot read another. Granted: [${scopes:-unknown}]. See auth.toml [gws.auth.scopes]." >&2
      exit 3 ;;
    404)
      printf '%s\n' "$PLUGIN $verb: $resource not found (HTTP 404)." >&2
      exit 5 ;;
    429)
      printf '%s\n' "$PLUGIN $verb: rate limited (HTTP 429) after $GWS_MAX_ATTEMPTS attempts — retryable; back off and retry later." >&2
      exit 4 ;;
    *)
      printf '%s\n' "$PLUGIN $verb: Google Workspace returned HTTP $HTTP_STATUS: $(printf '%s' "$HTTP_BODY" | head -c 300)" >&2
      exit 6 ;;
  esac
}

# ── verbs (executed only when this file is the entry script) ───────────────

_gws_auth_status() {
  # `status` does not REQUIRE a token — it reports state including "no token".
  local source token account scopes expiry has_token expired refreshable env_present file_path
  source="$(gws_active_source)"
  token="$(gws_resolve_token)"
  account="$(gws_token_account)"
  scopes="$(gws_token_scopes)"
  expiry="$(gws_token_expiry)"
  file_path="$(gws_token_file_path)"

  has_token=false
  [[ -n "$token" ]] && has_token=true

  expired=false
  if gws_token_expired; then expired=true; fi

  refreshable=false
  if gws_token_refreshable; then refreshable=true; fi

  env_present=false
  [[ -n "${BWOC_GWS_TOKEN:-}" ]] && env_present=true

  # scopes -> JSON array (space-split); empty string -> [].
  local scopes_json
  if [[ -n "$scopes" ]]; then
    scopes_json="$(printf '%s' "$scopes" | jq -R 'split(" ") | map(select(length>0))')"
  else
    scopes_json="[]"
  fi

  jq -n \
    --arg src "$source" \
    --argjson has "$has_token" \
    --arg account "$account" \
    --argjson scopes "$scopes_json" \
    --arg expiry "$expiry" \
    --argjson expired "$expired" \
    --argjson refreshable "$refreshable" \
    --argjson env_present "$env_present" \
    --arg file_path "$file_path" \
    '{
      ok: true,
      plugin: "gws-auth",
      operation: "status",
      active_source: $src,
      has_token: $has,
      account: (if $account == "" then null else $account end),
      scopes: $scopes,
      expiry: (if $expiry == "" then null else $expiry end),
      expired: $expired,
      refreshable: $refreshable,
      sources: {
        env:          { present: $env_present, var: "BWOC_GWS_TOKEN" },
        secrets_file: { present: ($file_path | length > 0), path: (if $file_path == "" then null else $file_path end) }
      }
    }'
}

_gws_auth_main() {
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
  if [[ -z "$operation" ]]; then operation="${BWOC_GWS_OPERATION:-}"; fi
  if [[ -z "$operation" ]]; then
    printf '%s\n' "$PLUGIN: no operation (set BWOC_GWS_OPERATION or pipe a JSON request carrying .operation)" >&2
    exit 2
  fi

  case "$operation" in
    status) _gws_auth_status ;;
    *)
      printf '%s\n' "$PLUGIN: unknown operation '$operation' (expected status)" >&2
      exit 2 ;;
  esac
}

# Only dispatch when executed directly. Sourcing this file imports the helpers
# without side-effects on the caller's stdin / argv.
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  _gws_auth_main
fi
