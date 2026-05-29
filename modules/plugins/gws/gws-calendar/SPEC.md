---
title: gws-calendar — Google Calendar Calendars & Events (Read-Mostly)
aliases:
  - gws-calendar
tags:
  - group/framework-plugins
  - type/plugin
  - kind/gws
  - domain/integration
  - integration/google-workspace
maturity: L1
---

# gws-calendar — Google Calendar Calendars & Events (Read-Mostly)

> [!abstract] A per-service plugin of the `gws` kind (`BWOC-EPIC-13`). It reads **Google Calendar** — `calendars` (Calendar `calendarList.list`) and `events` (Calendar `events.list`) — and projects each event into the normative [[../../../docs/en/PLUGINS.en#Workspace Resource Schema|Calendar event shape]]. It **never** creates or modifies events (read-mostly by design — write slices like `events.insert` are deferred, design note §Decision 4). It sources the OAuth credential helpers from the [[../gws-auth/SPEC|`gws-auth`]] foundation, so it carries no auth code of its own. Requires the `calendar.readonly` scope. Full framing: [[../../../notes/2026-05-28_google-workspace-plugin-architecture|BWOC-72 design note]].

## Verbs

| Operation | Aliases | Direction | Calendar endpoint | Side effect |
|---|---|---|---|---|
| `calendars` | `list` | read | `GET /users/me/calendarList` (`calendarList.list`) | None — paginates under the hood. |
| `events` | — | read | `GET /calendars/{calendarId}/events` (`events.list`) | None — paginates under the hood; `--max` caps total results. |

`calendars` returns the calendars the token can see (`calendar_id`, `summary`, optional `primary` / `access_role`). `events` projects each event into the Calendar event entry shape (`event_id`, `calendar_id`, `summary`, `start`, `end`, optional `attendees_count`) and returns an array under `events`.

> [!note] Verb names. The `bwoc gws` CLI (`BWOC-74`) invokes the `calendars` operation behind its `calendar list` subcommand; `list` is accepted as an alias so direct invocation works either way.

## How it runs

The `bwoc gws` CLI (`BWOC-74`) spawns `gws.sh` from this directory:

| Channel | What it carries |
|---|---|
| `BWOC_GWS_OPERATION` (env) | `calendars` \| `events` — fallback for `.operation` when stdin is empty. |
| `BWOC_WORKSPACE` (env) | Absolute workspace root (token file resolution, via the sibling). |
| `BWOC_PLUGIN_DIR` (env) | Absolute path to this plugin's directory — used to find `../gws-auth/gws.sh`. |
| `BWOC_GWS_TOKEN` (env) | The OAuth2 access token — **secret**, consumed by the sibling helpers. |
| stdin | One-line JSON request — see the contract examples below. |

```jsonc
{"operation":"calendars"}
{"operation":"events"}
{"operation":"events","calendar_id":"primary","max":50}
{"operation":"events","calendar_id":"team@group.calendar.google.com"}
```

`.calendar_id` selects which calendar to read events from (default `primary`); `.max` caps the number of events returned (default `100`).

On success: exit `0`, one JSON object on stdout. On error: human diagnostic on stderr + non-zero exit.

## Authentication & scope

This plugin holds **no credential code**. It sources `gws-auth/gws.sh` and calls `gws_curl`, which resolves the token (env → token file), refreshes it if expired, sets `Authorization: Bearer`, and handles rate limits. See [[../gws-auth/SPEC|gws-auth]] for the token model.

It requires the **`https://www.googleapis.com/auth/calendar.readonly`** scope. Because OAuth scopes are per-service and consent-bound, a token granted only Drive or Gmail scope returns HTTP 403 here — surfaced as `token lacks the required scope for calendar 'primary' events`, not a bare failure.

> [!danger] **Sila — Adinnādāna.** The token never enters this plugin's output. It is handed to curl by the sibling helper as a request header only; the plugin projects Calendar's JSON response — never the credential — into the event entries.

## Output shapes

### `calendars`

```json
{
  "ok": true,
  "plugin": "gws-calendar",
  "operation": "calendars",
  "total": 2,
  "calendars": [
    { "calendar_id": "primary", "summary": "me@example.com", "primary": true, "access_role": "owner" },
    { "calendar_id": "team@group.calendar.google.com", "summary": "BWOC Team", "access_role": "reader" }
  ]
}
```

### `events`

```json
{
  "ok": true,
  "plugin": "gws-calendar",
  "operation": "events",
  "total": 2,
  "events": [
    {
      "event_id": "abc123def456",
      "calendar_id": "primary",
      "summary": "Sprint 13 review",
      "start": "2026-05-28T09:00:00Z",
      "end": "2026-05-28T10:00:00Z",
      "attendees_count": 4
    },
    {
      "event_id": "ghi789jkl012",
      "calendar_id": "primary",
      "summary": "All-day offsite",
      "start": "2026-06-01",
      "end": "2026-06-02"
    }
  ]
}
```

Optional fields (`primary`, `access_role`, `attendees_count`) are **omitted when absent**, never `null` — per the framework resource-schema convention. `start` / `end` carry a date-time for timed events and a date for all-day events.

## Pagination & rate limits

Both verbs paginate under the hood: `calendars` follows `calendarList.list`'s `nextPageToken`; `events` requests pages of up to 100 events (`singleEvents=true`, `orderBy=startTime` for a deterministic, recurrence-expanded ordering), follows `nextPageToken`, and stops as soon as `--max` is reached (or the listing is exhausted). Each returns a single bounded envelope so an agent never pulls an unbounded calendar. Rate limiting (HTTP 429) is handled by the sibling's `gws_curl` — it honors `Retry-After` with a squared fallback, up to four attempts, before surfacing a retryable error.

## Error classes

Inherited from the sibling `gws_classify_status` (so all `gws-*` plugins map HTTP identically):

| Exit | Class | Meaning |
|---|---|---|
| `0` | success | One JSON object on stdout. |
| `1` | dependency | `jq` or `curl` missing from PATH. |
| `2` | usage / no-token | Unknown / missing operation, an invalid `calendar_id`, or no resolvable token. |
| `3` | auth / scope | HTTP 401 (token invalid) or 403 (lacks `calendar.readonly`). |
| `4` | rate-limited | HTTP 429 after the backoff budget. |
| `5` | not-found | HTTP 404 (no such calendar). |
| `6` | transport / unexpected | Network failure or an unmapped HTTP status. |

A crafted `calendar_id` cannot inject into the request URL — `events` rejects any id outside `[A-Za-z0-9_.@-]` and percent-encodes it into the path before issuing the call.

## Configuration

```toml
# workspace.toml
[plugins.gws-calendar]
enabled = true
```

No `[config.schema]` — the plugin holds no config of its own; credentials resolve through the `gws-auth` foundation.

## Lifecycle mapping

Per [[../../../docs/en/PLUGINS.en#Lifecycle|PLUGINS.en.md §Lifecycle]], the `gws` kind's owner is the **agent** calling out via the `bwoc gws` CLI. `init`/`teardown` are per-invocation around `invoke`. The plugin holds no local state.

| Phase | What this plugin does |
|---|---|
| `init` | Implicit per invocation; verifies `jq` + `curl` on PATH and the sibling helpers are present. |
| `invoke` | Reads the request, calls Calendar via the sibling `gws_curl`, projects the response into calendar / event entries. |
| `teardown` | Implicit; no state to release. |

## Idempotency

Both verbs are read-only and order-stable across replays. `events` is deterministic for a fixed calendar state + `calendar_id` + `max`; pagination is internal and never partially mutates anything.

## Maturity

Declared **L1** — first runnable `gws/gws-calendar` reference plugin; both verbs functional. Bumps to **L2** once the `bwoc check` extension (`BWOC-77`) and smoke tests exercise it end-to-end against an operator OAuth token with `calendar.readonly`.

> [!warning] Live-test gap. Live verification (a real `calendar.readonly` token reading real events) gates on an operator-provided OAuth token (design note §Status). v0.1.0 is verified by: `bash -n gws.sh`, the missing-dependency + missing-token + bad-`calendar_id` paths erroring cleanly, and `bwoc check` accepting the manifest.

## Neutrality

Manifest values name no LLM backend or model. `kind = "gws"` is the framework's own enum value. "Google Calendar" / "Google Workspace" appear only in `description` (where integration-target names are tolerated) and in this SPEC's prose — never in `kind`, `entry`, or config keys. Satisfies **Samānattatā**.

## See Also

- [[../gws-auth/SPEC|gws-auth SPEC]] — the OAuth credential foundation this plugin sources.
- [[../gws-gmail/SPEC|gws-gmail SPEC]] — the sibling Gmail plugin (same family shape).
- [[../../../notes/2026-05-28_google-workspace-plugin-architecture|BWOC-72 design note]] — full EPIC-13 framing (decisions 1–5).
- [[../../../docs/en/PLUGINS.en#Workspace Resource Schema|PLUGINS.en.md §Workspace Resource Schema]] — the normative Calendar event shape this plugin emits.
- [[SPEC.th|SPEC.th.md]] — Thai counterpart (bilingual parity).
