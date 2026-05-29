---
title: gws-auth — Google Workspace OAuth Credential Foundation
aliases:
  - gws-auth
tags:
  - group/framework-plugins
  - type/plugin
  - kind/gws
  - domain/integration
  - integration/google-workspace
maturity: L1
---

# gws-auth — Google Workspace OAuth Credential Foundation

> [!abstract] The **credential foundation** of the `gws` plugin kind (`BWOC-EPIC-13`). It owns the OAuth2 surface — token presence, granted scopes, account, and expiry — and exports the Bearer-auth + rate-limit + refresh helpers the per-service plugins source. Verb: `status` (read-only; **never prints the token value**). It is the Workspace analogue of [[../../workflow/gcloud-auth/SPEC|`gcloud-auth`]] — but a different auth family entirely: OAuth2 **user-consent** scopes over the Workspace REST APIs, not ADC / service-account over the local `gcloud` CLI. Pairs with [[../gws-drive/SPEC|`gws-drive`]] (and the future `gws-gmail` / `gws-calendar`), which source the helpers declared here. Full framing: [[../../../notes/2026-05-28_google-workspace-plugin-architecture|BWOC-72 design note]].

## Why a foundation plugin, why the `gws` kind

`gws-auth` ships as a **separate** plugin so every per-service plugin (`gws-drive`, and the future `gws-gmail` / `gws-calendar`) reuses one OAuth surface — token resolution, the `Authorization: Bearer` header, 429 handling, refresh-if-expired — without re-rolling it (design note §Decision 2, the `gcloud-*` family shape). The `gws` kind (rather than reusing `workflow` like `gcloud`) is correct because BWOC owns a normative schema over the integration: the per-service [[../../../docs/en/PLUGINS.en#Workspace Resource Schema|Workspace resource schemas]] (a Drive file, a Gmail thread, a Calendar event) + the OAuth scope model. The rule: **own-kind when BWOC defines a normative schema over the integration; `workflow`-reuse for a passthrough with no BWOC-owned shape** (design note §Decision 1).

It is **not** part of `gcloud`: gcloud reaches GCP *infrastructure* through the local `gcloud` CLI with ADC / service-account; `gws` reaches productivity *apps* through the Workspace REST APIs with OAuth2 user-consent scopes. Different auth family, different surface, different lifecycle.

## Verbs

| Operation | Direction | Auth | Side effect |
|---|---|---|---|
| `status` | read | none required | None — reports token presence, granted scopes, account, expiry, and whether a refresh is possible; **never** prints the token value. |

`status` is the canonical agent-facing read. Token acquisition (the OAuth consent flow) is an **operator** action out of band — the plugin consumes a token, it does not mint one.

## How it runs

The `bwoc gws` CLI (`BWOC-74`) spawns `gws.sh` from this directory (mirroring how `bwoc gcloud` dispatches `gcloud-auth` and `bwoc figma` dispatches `figma-rest`):

| Channel | What it carries |
|---|---|
| `BWOC_GWS_OPERATION` (env) | `status` — fallback for `.operation` when stdin is empty. |
| `BWOC_WORKSPACE` (env) | Absolute workspace root; resolves the token file path. |
| `BWOC_PLUGIN_DIR` (env) | Absolute path to this plugin's directory (informational). |
| `BWOC_GWS_TOKEN` (env) | The OAuth2 access token — **secret**, highest precedence. |
| stdin | One-line JSON request, e.g. `{"operation":"status"}`. |

On success: exit `0`, one JSON object on stdout. On error: human diagnostic on stderr + non-zero exit (the CLI surfaces it as `plugin '<name>' exited <code>`).

## Authentication

The plugin **never serializes the token into any output**. It reads the token only to set the `Authorization: Bearer` header on outbound REST calls (for the sibling service plugins that source it). Precedence (first source that resolves wins; reflected in `status.active_source`):

1. **`BWOC_GWS_TOKEN`** env — transient / CI; carries no metadata (scopes, expiry, account are unknown for an env token).
2. **Token file** — `${BWOC_WORKSPACE}/.bwoc/secrets/gws-token.json`. Gitignored at the workspace level (BWOC-53 secret store), `chmod 600`, never committed. A JSON object holding at least `access_token`, optionally `refresh_token` / `expiry` / `scopes` / `account` / `client_id` / `client_secret`.

[[auth|auth.toml]] declares the **shape** — the env-var name, the token-file path + recognized fields, and the per-service readonly scopes — with **no values**. `bwoc check` (`BWOC-77`) refuses to pass if any value-looking field appears.

> [!danger] **Sila — Adinnādāna.** A malformed `auth.toml` cannot leak a token because no token ever lives in any tracked file. The token enters the plugin only via the environment or a gitignored, owner-only file, and leaves it only as a curl request header. It is never echoed, logged, or placed in any JSON envelope. The `status` verb deliberately surfaces metadata (scopes, account, expiry, source) — **never** the token value.

### Scopes

OAuth scopes are **per-service and consent-bound**: a token granted only `drive.readonly` cannot read Gmail or Calendar. `auth.toml [gws.auth.scopes]` declares the readonly scope each service needs:

| Service | Required scope |
|---|---|
| Drive | `https://www.googleapis.com/auth/drive.readonly` |
| Gmail | `https://www.googleapis.com/auth/gmail.readonly` |
| Calendar | `https://www.googleapis.com/auth/calendar.readonly` |

`status.scopes` reports the scopes the *current* token carries (from the token file; empty for an env token). A service verb whose scope is absent surfaces `token lacks <scope> for <service>` on the resulting HTTP 403, never a bare failure (see [[../gws-drive/SPEC|gws-drive]] error handling).

## Refresh-if-expired

When the token file carries `expiry` in the past **and** the refresh trio (`refresh_token` + `client_id` + `client_secret`), `gws-auth` performs an offline `refresh_token` grant against Google's OAuth2 endpoint, then rewrites the token file in place (new `access_token` + recomputed `expiry`; all other fields preserved; written via a `chmod 600` temp file + atomic `mv` so the secret is never briefly world-readable). This happens transparently inside `gws_curl` before each request, so a sibling never sees a stale token. When a refresh is needed but impossible (env token, or no refresh trio), the request proceeds and the resulting HTTP 401 is surfaced with a clear "re-authorize" message — never a panic.

## Output shape

### `status`

```json
{
  "ok": true,
  "plugin": "gws-auth",
  "operation": "status",
  "active_source": "secrets-file",
  "has_token": true,
  "account": "me@example.com",
  "scopes": ["https://www.googleapis.com/auth/drive.readonly"],
  "expiry": "2026-05-28T18:00:00Z",
  "expired": false,
  "refreshable": true,
  "sources": {
    "env":          { "present": false, "var": "BWOC_GWS_TOKEN" },
    "secrets_file": { "present": true, "path": "/abs/workspace/.bwoc/secrets/gws-token.json" }
  }
}
```

`active_source` is one of `env | secrets-file | none`. `account` / `expiry` are `null` when unknown; `scopes` is `[]` when unknown (e.g. an env token). `expired` is `true` only when a known `expiry` is in the past; `refreshable` is `true` only when the refresh trio is present. The envelope always emits cleanly (no panic) even with no token at all.

## Shared helpers

`gws.sh` exports the OAuth credential surface for sibling `gws/gws-*` plugins to source (design note §Decision 2):

| Function | Returns / Side-effect |
|---|---|
| `gws_resolve_token` | Echoes the access token (env first, then token file). Empty when none. **Never an output field** — captured into a variable only. |
| `gws_auth_header` | Echoes the full `Authorization: Bearer <token>` header line for curl. Empty when no token. |
| `gws_assert_token` | Returns `2` (with clear stderr) when no token is resolvable. |
| `gws_token_scopes` / `gws_token_account` / `gws_token_expiry` | Metadata accessors from the token file (empty for an env token). |
| `gws_refresh_if_expired` | Refreshes an expired+refreshable token in place; no-op otherwise. |
| `gws_curl` | Authenticated request: refresh → Bearer + JSON Accept headers → 429 `Retry-After` backoff (up to 4 attempts). Sets `HTTP_STATUS` / `HTTP_BODY`. |
| `gws_classify_status` | Maps `HTTP_STATUS` to a clear diagnostic + exit code (401 auth, 403 scope, 404, 429, transport). |

The dispatcher (`_gws_auth_main`) is `BASH_SOURCE`-guarded, so sourcing this file is a pure import — no stdin consumed, no verb run. [[../gws-drive/gws|`gws-drive/gws.sh`]] sources this file via `$BWOC_PLUGIN_DIR/../gws-auth/gws.sh` (with a script-relative fallback so the plugin stays testable outside the framework dispatcher).

## Error classes

| Exit | Class | Meaning |
|---|---|---|
| `0` | success | One JSON object on stdout. |
| `1` | dependency | `jq` missing from PATH. |
| `2` | usage / no-token | Unknown / missing operation, or `gws_assert_token` found no token. |
| `3` | auth / scope | `gws_classify_status` saw HTTP 401 (token invalid) or 403 (scope gap). |
| `4` | rate-limited | HTTP 429 after the backoff budget. |
| `5` | not-found | HTTP 404. |
| `6` | transport / unexpected | Network failure or an unmapped HTTP status. |

Missing `jq` fails **gracefully** with a clear stderr message; the plugin never panics or leaves the process half-done.

## Configuration

```toml
# workspace.toml
[plugins.gws-auth]
enabled = true
```

No `[config.schema]` — credential resolution is environment-driven, not config-driven (design note §Decision 3). The only workspace-level surface is the universal `enabled` key.

## Lifecycle mapping

Per [[../../../docs/en/PLUGINS.en#Lifecycle|PLUGINS.en.md §Lifecycle]], the `gws` kind's owner is the **agent** calling out via the `bwoc gws` CLI. `init`/`teardown` are per-invocation around `invoke`. The plugin holds **no local state** beyond the operator-provided token file.

| Phase | What this plugin does |
|---|---|
| `init` | Implicit per invocation; verifies `jq` on PATH. |
| `invoke` | Reads the request, inspects token state, emits JSON (`status`); or, when sourced, performs authenticated requests for a sibling. |
| `teardown` | Implicit; no state to release. |

## Idempotency

- `status` is read-only and order-stable across replays.
- `gws_refresh_if_expired` converges: a non-expired token is a no-op; an expired+refreshable token is refreshed exactly once per call and the file rewritten atomically.

## Maturity

Declared **L1** — first runnable `gws/gws-auth` reference plugin; the `status` verb + the shared helper surface are functional. Bumps to **L2** once the `bwoc check` extension (`BWOC-77`) and smoke tests exercise it end-to-end against an operator OAuth token.

> [!warning] Live-test gap. Live end-to-end verification (a real consented token reading real Workspace data) gates on an operator-provided OAuth token at `.bwoc/secrets/gws-token.json` or `BWOC_GWS_TOKEN` (design note §Status). v0.1.0 is verified by: `bash -n gws.sh`, the no-token `status` envelope emitting cleanly (`has_token:false`), the missing-`jq` path erroring cleanly, and `bwoc check` accepting the manifest.

## Neutrality

Manifest values name no LLM backend or model. `kind = "gws"` is the framework's own enum value. "Google Workspace" / service names appear only in `description` (where integration-target names are tolerated per [[../../../docs/en/PLUGINS.en#Neutrality constraint (HARD)|PLUGINS.en.md §Neutrality]]) and in this SPEC's prose — never in `kind`, `entry`, or config keys. Satisfies **Samānattatā**.

## See Also

- [[../../../notes/2026-05-28_google-workspace-plugin-architecture|BWOC-72 design note]] — full framing for the EPIC-13 foundation (decisions 1–5).
- [[../gws-drive/SPEC|gws-drive SPEC]] — sibling service plugin (Drive files); sources the helpers declared here.
- [[../../workflow/gcloud-auth/SPEC|gcloud-auth SPEC]] — the Google *infrastructure* analogue; explicitly **not** the same auth family.
- [[../../../docs/en/PLUGINS.en|PLUGINS.en.md]] — the plugin spec; `gws` kind row + Workspace Resource Schema.
- [[auth|auth.toml]] — the auth contract (shape only; no values).
- [[SPEC.th|SPEC.th.md]] — Thai counterpart (bilingual parity).
