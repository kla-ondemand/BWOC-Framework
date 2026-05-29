---
title: gws-drive — Google Drive Files (Read-Mostly)
aliases:
  - gws-drive
tags:
  - group/framework-plugins
  - type/plugin
  - kind/gws
  - domain/integration
  - integration/google-workspace
maturity: L1
---

# gws-drive — Google Drive Files (Read-Mostly)

> [!abstract] A per-service plugin of the `gws` kind (`BWOC-EPIC-13`). It reads **Google Drive** — `list` (Drive `files.list`) and `get` (Drive `files.get` metadata) — and projects each result into the normative [[../../../docs/en/PLUGINS.en#Workspace Resource Schema|Drive file shape]]. It **never** writes back to Drive (read-mostly by design — write slices like upload are deferred, design note §Decision 4). It sources the OAuth credential helpers from the [[../gws-auth/SPEC|`gws-auth`]] foundation, so it carries no auth code of its own. Requires the `drive.readonly` scope. Full framing: [[../../../notes/2026-05-28_google-workspace-plugin-architecture|BWOC-72 design note]].

## Verbs

| Operation | Direction | Drive endpoint | Side effect |
|---|---|---|---|
| `list` | read | `GET /drive/v3/files` (`files.list`) | None — paginates under the hood; `--max` caps total results. |
| `get` | read | `GET /drive/v3/files/{fileId}` (`files.get`) | None — metadata only; never downloads content. |

Both project the Drive REST object into the Drive file entry shape (`file_id`, `name`, `mime_type`, `modified_time`, optional `owners` / `web_view_link`). `get` returns a single entry; `list` returns an array.

## How it runs

The `bwoc gws` CLI (`BWOC-74`) spawns `gws.sh` from this directory:

| Channel | What it carries |
|---|---|
| `BWOC_GWS_OPERATION` (env) | `list` \| `get` — fallback for `.operation` when stdin is empty. |
| `BWOC_WORKSPACE` (env) | Absolute workspace root (token file resolution, via the sibling). |
| `BWOC_PLUGIN_DIR` (env) | Absolute path to this plugin's directory — used to find `../gws-auth/gws.sh`. |
| `BWOC_GWS_TOKEN` (env) | The OAuth2 access token — **secret**, consumed by the sibling helpers. |
| stdin | One-line JSON request — see the contract examples below. |

```jsonc
{"operation":"list"}
{"operation":"list","query":"mimeType='application/pdf'","max":50}
{"operation":"get","file_id":"1AbC_dEfGhIjKlMnOpQrStUvWxYz"}
```

`.query` is a Drive [search query](https://developers.google.com/drive/api/guides/search-files) passed as the `q` parameter; `.max` caps the number of files returned (default `100`).

On success: exit `0`, one JSON object on stdout. On error: human diagnostic on stderr + non-zero exit.

## Authentication & scope

This plugin holds **no credential code**. It sources `gws-auth/gws.sh` and calls `gws_curl`, which resolves the token (env → token file), refreshes it if expired, sets `Authorization: Bearer`, and handles rate limits. See [[../gws-auth/SPEC|gws-auth]] for the token model.

It requires the **`https://www.googleapis.com/auth/drive.readonly`** scope. Because OAuth scopes are per-service and consent-bound, a token granted only Gmail or Calendar scope returns HTTP 403 here — surfaced as `token lacks the required scope for Drive files`, not a bare failure.

> [!danger] **Sila — Adinnādāna.** The token never enters this plugin's output. It is handed to curl by the sibling helper as a request header only; the plugin projects Drive's JSON response — never the credential — into the Drive file entries.

## Output shapes

### `list`

```json
{
  "ok": true,
  "plugin": "gws-drive",
  "operation": "list",
  "total": 2,
  "files": [
    {
      "file_id": "1AbC_dEfGhIjKlMnOpQrStUvWxYz",
      "name": "BWOC Architecture.gdoc",
      "mime_type": "application/vnd.google-apps.document",
      "modified_time": "2026-05-27T09:00:00Z",
      "web_view_link": "https://docs.google.com/document/d/1AbC_dEfGhIjKlMnOpQrStUvWxYz/edit"
    },
    {
      "file_id": "2XyZ...",
      "name": "notes.pdf",
      "mime_type": "application/pdf",
      "modified_time": "2026-05-26T11:00:00Z",
      "owners": ["me@example.com"]
    }
  ]
}
```

### `get`

```json
{
  "ok": true,
  "plugin": "gws-drive",
  "operation": "get",
  "file": {
    "file_id": "1AbC_dEfGhIjKlMnOpQrStUvWxYz",
    "name": "BWOC Architecture.gdoc",
    "mime_type": "application/vnd.google-apps.document",
    "modified_time": "2026-05-27T09:00:00Z",
    "web_view_link": "https://docs.google.com/document/d/1AbC_dEfGhIjKlMnOpQrStUvWxYz/edit"
  }
}
```

Optional fields (`owners`, `web_view_link`) are **omitted when absent**, never `null` — per the framework resource-schema convention.

## Pagination & rate limits

`list` paginates under the hood: it requests pages of up to 100 files, following Drive's `nextPageToken`, and stops as soon as `--max` is reached (or the listing is exhausted). It returns a single bounded envelope so an agent never pulls an unbounded Drive. Rate limiting (HTTP 429) is handled by the sibling's `gws_curl` — it honors `Retry-After` with a squared fallback, up to four attempts, before surfacing a retryable error.

## Error classes

Inherited from the sibling `gws_classify_status` (so all `gws-*` plugins map HTTP identically):

| Exit | Class | Meaning |
|---|---|---|
| `0` | success | One JSON object on stdout. |
| `1` | dependency | `jq` or `curl` missing from PATH. |
| `2` | usage / no-token | Unknown / missing operation, missing `.file_id`, an invalid `file_id`, or no resolvable token. |
| `3` | auth / scope | HTTP 401 (token invalid) or 403 (lacks `drive.readonly`). |
| `4` | rate-limited | HTTP 429 after the backoff budget. |
| `5` | not-found | HTTP 404 (no such file). |
| `6` | transport / unexpected | Network failure or an unmapped HTTP status. |

A crafted `file_id` cannot inject into the request URL — `get` rejects any id outside `[A-Za-z0-9_-]` before issuing the call.

## Configuration

```toml
# workspace.toml
[plugins.gws-drive]
enabled = true
```

No `[config.schema]` — the plugin holds no config of its own; credentials resolve through the `gws-auth` foundation.

## Lifecycle mapping

Per [[../../../docs/en/PLUGINS.en#Lifecycle|PLUGINS.en.md §Lifecycle]], the `gws` kind's owner is the **agent** calling out via the `bwoc gws` CLI. `init`/`teardown` are per-invocation around `invoke`. The plugin holds no local state.

| Phase | What this plugin does |
|---|---|
| `init` | Implicit per invocation; verifies `jq` + `curl` on PATH and the sibling helpers are present. |
| `invoke` | Reads the request, calls Drive via the sibling `gws_curl`, projects the response into Drive file entries. |
| `teardown` | Implicit; no state to release. |

## Idempotency

Both verbs are read-only and order-stable across replays. `list` is deterministic for a fixed Drive state + query + `max`; pagination is internal and never partially mutates anything.

## Maturity

Declared **L1** — first runnable `gws/gws-drive` reference plugin; both verbs functional. Bumps to **L2** once the `bwoc check` extension (`BWOC-77`) and smoke tests exercise it end-to-end against an operator OAuth token with `drive.readonly`.

> [!warning] Live-test gap. Live verification (a real `drive.readonly` token reading real files) gates on an operator-provided OAuth token (design note §Status). v0.1.0 is verified by: `bash -n gws.sh`, the missing-dependency + missing-token + bad-`file_id` paths erroring cleanly, and `bwoc check` accepting the manifest.

## Neutrality

Manifest values name no LLM backend or model. `kind = "gws"` is the framework's own enum value. "Google Drive" / "Google Workspace" appear only in `description` (where integration-target names are tolerated) and in this SPEC's prose — never in `kind`, `entry`, or config keys. Satisfies **Samānattatā**.

## See Also

- [[../gws-auth/SPEC|gws-auth SPEC]] — the OAuth credential foundation this plugin sources.
- [[../../../notes/2026-05-28_google-workspace-plugin-architecture|BWOC-72 design note]] — full EPIC-13 framing (decisions 1–5).
- [[../../../docs/en/PLUGINS.en#Workspace Resource Schema|PLUGINS.en.md §Workspace Resource Schema]] — the normative Drive file shape this plugin emits.
- [[SPEC.th|SPEC.th.md]] — Thai counterpart (bilingual parity).
