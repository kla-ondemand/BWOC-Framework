---
title: gws-gmail — Google Gmail Threads & Labels (Read-Mostly)
aliases:
  - gws-gmail
tags:
  - group/framework-plugins
  - type/plugin
  - kind/gws
  - domain/integration
  - integration/google-workspace
maturity: L1
---

# gws-gmail — Google Gmail Threads & Labels (Read-Mostly)

> [!abstract] A per-service plugin of the `gws` kind (`BWOC-EPIC-13`). It reads **Google Gmail** — `search` (Gmail `threads.list`, enriched per thread via `threads.get`), `show` (one thread via `threads.get`), and `labels` (`labels.list`) — and projects each thread into the normative [[../../../docs/en/PLUGINS.en#Workspace Resource Schema|Gmail thread shape]]. It **never** sends mail or modifies labels (read-mostly by design — write slices like `send` are deferred, design note §Decision 4). It sources the OAuth credential helpers from the [[../gws-auth/SPEC|`gws-auth`]] foundation, so it carries no auth code of its own. Requires the `gmail.readonly` scope. Full framing: [[../../../notes/2026-05-28_google-workspace-plugin-architecture|BWOC-72 design note]].

## Verbs

| Operation | Aliases | Direction | Gmail endpoint | Side effect |
|---|---|---|---|---|
| `search` | `threads` | read | `GET /users/me/threads` (`threads.list`) + `threads.get` per thread | None — paginates under the hood; `--max` caps total results. |
| `show` | `message`, `messages` | read | `GET /users/me/threads/{id}` (`threads.get`, metadata) | None — one thread's metadata. |
| `labels` | — | read | `GET /users/me/labels` (`labels.list`) | None — the user's label set. |

`search` and `show` project Gmail's REST objects into the Gmail thread entry shape (`thread_id`, `subject`, `from`, `last_message_time`, optional `snippet` / `labels`). `search` returns an array under `threads`; `show` spreads one entry into the envelope. `labels` returns label objects (`label_id`, `name`, `type`).

> [!note] Verb names. The `bwoc gws` CLI (`BWOC-74`) invokes `search` / `show` / `labels`. The EPIC-13 brief's conceptual names — *threads* (`search`) and *message* (`show`) — are accepted as aliases so direct invocation works either way. `search` already resolves each thread's latest-message metadata, so a separate per-message verb is unnecessary for the read-mostly surface.

## How it runs

The `bwoc gws` CLI (`BWOC-74`) spawns `gws.sh` from this directory:

| Channel | What it carries |
|---|---|
| `BWOC_GWS_OPERATION` (env) | `search` \| `show` \| `labels` — fallback for `.operation` when stdin is empty. |
| `BWOC_WORKSPACE` (env) | Absolute workspace root (token file resolution, via the sibling). |
| `BWOC_PLUGIN_DIR` (env) | Absolute path to this plugin's directory — used to find `../gws-auth/gws.sh`. |
| `BWOC_GWS_TOKEN` (env) | The OAuth2 access token — **secret**, consumed by the sibling helpers. |
| stdin | One-line JSON request — see the contract examples below. |

```jsonc
{"operation":"search"}
{"operation":"search","query":"from:me is:unread","max":25}
{"operation":"show","thread_id":"18ab12cd34ef5678"}
{"operation":"labels"}
```

`.query` is a Gmail [search query](https://developers.google.com/gmail/api/guides/filtering) passed as the `q` parameter; `.max` caps the number of threads returned (default `100`).

On success: exit `0`, one JSON object on stdout. On error: human diagnostic on stderr + non-zero exit.

## Authentication & scope

This plugin holds **no credential code**. It sources `gws-auth/gws.sh` and calls `gws_curl`, which resolves the token (env → token file), refreshes it if expired, sets `Authorization: Bearer`, and handles rate limits. See [[../gws-auth/SPEC|gws-auth]] for the token model.

It requires the **`https://www.googleapis.com/auth/gmail.readonly`** scope. Because OAuth scopes are per-service and consent-bound, a token granted only Drive or Calendar scope returns HTTP 403 here — surfaced as `token lacks the required scope for Gmail threads`, not a bare failure.

> [!danger] **Sila — Adinnādāna.** The token never enters this plugin's output. It is handed to curl by the sibling helper as a request header only; the plugin projects Gmail's JSON response — never the credential — into the thread entries.

## Output shapes

### `search`

```json
{
  "ok": true,
  "plugin": "gws-gmail",
  "operation": "search",
  "total": 2,
  "threads": [
    {
      "thread_id": "18ab12cd34ef5678",
      "subject": "Sprint 13 review",
      "from": "jisoo@example.com",
      "snippet": "Closing EPIC-13 — last impl story…",
      "labels": ["INBOX", "IMPORTANT"],
      "last_message_time": "2026-05-28T09:00:00Z"
    },
    {
      "thread_id": "18ab99887766aabb",
      "subject": "Re: gws-auth helpers",
      "from": "lisa@example.com",
      "last_message_time": "2026-05-27T14:00:00Z"
    }
  ]
}
```

### `show`

```json
{
  "ok": true,
  "plugin": "gws-gmail",
  "operation": "show",
  "thread_id": "18ab12cd34ef5678",
  "subject": "Sprint 13 review",
  "from": "jisoo@example.com",
  "snippet": "Closing EPIC-13 — last impl story…",
  "labels": ["INBOX", "IMPORTANT"],
  "last_message_time": "2026-05-28T09:00:00Z"
}
```

### `labels`

```json
{
  "ok": true,
  "plugin": "gws-gmail",
  "operation": "labels",
  "total": 2,
  "labels": [
    { "label_id": "INBOX", "name": "INBOX", "type": "system" },
    { "label_id": "Label_42", "name": "BWOC", "type": "user" }
  ]
}
```

Optional fields (`snippet`, `labels`) are **omitted when absent**, never `null` — per the framework resource-schema convention.

## Pagination & rate limits

`search` paginates under the hood: it requests pages of up to 100 threads, follows Gmail's `nextPageToken`, and stops as soon as `--max` is reached (or the listing is exhausted). Each collected thread is then enriched with a `threads.get` (metadata only — Subject/From/Date headers) to fill the required `subject` / `from` / `last_message_time` fields. It returns a single bounded envelope so an agent never pulls an unbounded mailbox. Rate limiting (HTTP 429) is handled by the sibling's `gws_curl` — it honors `Retry-After` with a squared fallback, up to four attempts, before surfacing a retryable error. A single thread that 404s between `list` and `get` (deleted mid-search) is skipped; systemic auth/rate errors still abort.

## Error classes

Inherited from the sibling `gws_classify_status` (so all `gws-*` plugins map HTTP identically):

| Exit | Class | Meaning |
|---|---|---|
| `0` | success | One JSON object on stdout. |
| `1` | dependency | `jq` or `curl` missing from PATH. |
| `2` | usage / no-token | Unknown / missing operation, missing `.thread_id`, an invalid `thread_id`, or no resolvable token. |
| `3` | auth / scope | HTTP 401 (token invalid) or 403 (lacks `gmail.readonly`). |
| `4` | rate-limited | HTTP 429 after the backoff budget. |
| `5` | not-found | HTTP 404 (no such thread). |
| `6` | transport / unexpected | Network failure or an unmapped HTTP status. |

A crafted `thread_id` cannot inject into the request URL — `show` rejects any id outside `[A-Za-z0-9_-]` before issuing the call.

## Configuration

```toml
# workspace.toml
[plugins.gws-gmail]
enabled = true
```

No `[config.schema]` — the plugin holds no config of its own; credentials resolve through the `gws-auth` foundation.

## Lifecycle mapping

Per [[../../../docs/en/PLUGINS.en#Lifecycle|PLUGINS.en.md §Lifecycle]], the `gws` kind's owner is the **agent** calling out via the `bwoc gws` CLI. `init`/`teardown` are per-invocation around `invoke`. The plugin holds no local state.

| Phase | What this plugin does |
|---|---|
| `init` | Implicit per invocation; verifies `jq` + `curl` on PATH and the sibling helpers are present. |
| `invoke` | Reads the request, calls Gmail via the sibling `gws_curl`, projects the response into thread / label entries. |
| `teardown` | Implicit; no state to release. |

## Idempotency

All three verbs are read-only and order-stable across replays. `search` is deterministic for a fixed mailbox state + query + `max`; pagination + per-thread enrichment are internal and never partially mutate anything.

## Maturity

Declared **L1** — first runnable `gws/gws-gmail` reference plugin; all three verbs functional. Bumps to **L2** once the `bwoc check` extension (`BWOC-77`) and smoke tests exercise it end-to-end against an operator OAuth token with `gmail.readonly`.

> [!warning] Live-test gap. Live verification (a real `gmail.readonly` token reading real threads) gates on an operator-provided OAuth token (design note §Status). v0.1.0 is verified by: `bash -n gws.sh`, the missing-dependency + missing-token + bad-`thread_id` paths erroring cleanly, and `bwoc check` accepting the manifest.

## Neutrality

Manifest values name no LLM backend or model. `kind = "gws"` is the framework's own enum value. "Google Gmail" / "Google Workspace" appear only in `description` (where integration-target names are tolerated) and in this SPEC's prose — never in `kind`, `entry`, or config keys. Satisfies **Samānattatā**.

## See Also

- [[../gws-auth/SPEC|gws-auth SPEC]] — the OAuth credential foundation this plugin sources.
- [[../gws-drive/SPEC|gws-drive SPEC]] — the sibling Drive plugin (same family shape).
- [[../../../notes/2026-05-28_google-workspace-plugin-architecture|BWOC-72 design note]] — full EPIC-13 framing (decisions 1–5).
- [[../../../docs/en/PLUGINS.en#Workspace Resource Schema|PLUGINS.en.md §Workspace Resource Schema]] — the normative Gmail thread shape this plugin emits.
- [[SPEC.th|SPEC.th.md]] — Thai counterpart (bilingual parity).
