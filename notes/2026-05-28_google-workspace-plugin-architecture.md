---
title: Google Workspace Plugin — architecture framing for EPIC-13
date: 2026-05-28
sprint: BWOC sprint-13
epic: BWOC-EPIC-13
story: BWOC-72
related_stories: [BWOC-73, BWOC-74, BWOC-75, BWOC-76, BWOC-77]
---

# 2026-05-28 — Google Workspace Plugin (EPIC-13 framing)

This note frames `BWOC-EPIC-13` before code lands. The operator asked for a Google Workspace plugin covering **all three** productivity services — Drive, Gmail, Calendar — read-mostly. It answers the questions `BWOC-73` (PLUGINS spec + resource schema), `BWOC-74` (`bwoc gws` CLI), `BWOC-75` (gws-auth + gws-drive), `BWOC-76` (gws-gmail + gws-calendar), and `BWOC-77` (check ext) build against: whether `gws` earns its own kind, the OAuth model, the per-service read scopes, and why writes are deferred.

The throughline: **Google Workspace is the second Google integration, but it is NOT gcloud.** `gcloud` (EPIC-8/9) is GCP *infrastructure* — projects, VMs, IAM — reached through the local `gcloud` CLI. Workspace is *productivity apps* — files, mail, calendars — reached through Google's REST APIs with OAuth2 user scopes. They share a vendor and an auth family (Google OAuth) but nothing else; conflating them would be a category error.

## Decisions

### 1. `gws` is its own plugin kind — not `workflow`, not folded into gcloud

Per the own-kind-vs-workflow rule crystallized across jira/gcloud/figma ([BWOC-61 §Decision 1](2026-05-28_figma-plugin-architecture.md)): **own-kind when BWOC defines a normative schema over the integration; `workflow`-reuse for a passthrough with no BWOC-owned shape.** `gws` carries normative **Workspace resource schemas** (a Drive file, a Gmail thread, a Calendar event each get a durable BWOC-owned shape an agent or dashboard can rely on) + an **OAuth scope model**. That earns its own kind — the same way figma's Asset Mapping and jira's Issue Mapping did.

It is emphatically **not** part of `gcloud`: gcloud shells to the local `gcloud` CLI for GCP infra; gws calls the Workspace REST APIs (Drive/Gmail/Calendar v3) with OAuth2 user-consent scopes. Different auth (OAuth user token vs ADC/service-account), different surface (apps vs infra), different lifecycle.

### 2. Service-plugin family — `gws-auth` foundation + per-service plugins

Mirroring the gcloud-* family shape (EPIC-8 §Decision 2): a **credential foundation** plugin + per-service plugins that source it.

| Plugin | Owns | Read verbs (foundation) |
|---|---|---|
| `gws-auth` | OAuth2 credential state — token, granted scopes, account | `status` |
| `gws-drive` | Drive files | `list`, `get` (metadata), `read` (content) |
| `gws-gmail` | Gmail | `threads` (search), `message` (get), `labels` (list) |
| `gws-calendar` | Calendar | `calendars` (list), `events` (list) |

`gws-auth/gws.sh` exports credential-resolution + token-refresh helpers; the three service plugins source them (no duplicated OAuth handling). Future write slices (gws-gmail send, gws-calendar create, gws-drive upload) reuse the same auth foundation — exactly why it is a separate plugin.

### 3. OAuth2 model — operator token, per-service readonly scopes, never committed

Workspace REST authenticates with an **OAuth2 access token** (Bearer) carrying user-consented scopes. Token sources, in precedence order (the jira/gcloud/figma pattern):

1. **`BWOC_GWS_TOKEN`** env — transient / CI.
2. **`.bwoc/secrets/gws-token.json`** — workspace-local, `chmod 600`, **gitignored**. Holds the OAuth token (+ refresh token if present); `gws-auth` refreshes when expired.

`auth.toml` declares the **shape** — env var, secrets path, and the required readonly scopes per service — but **no token value**:

```
drive.readonly · gmail.readonly · calendar.readonly
```

The plugin sets `Authorization: Bearer <token>` on outbound requests; never logs/echoes/serializes it (Sīla — Adinnādāna). `bwoc check` (BWOC-77) fails closed on any value-looking field in `auth.toml`, as the jira (BWOC-45) / gcloud (BWOC-55) / figma (BWOC-65) guards do.

> [!warning]
> OAuth scopes are **per-service and consent-bound**: a token granted only `drive.readonly` cannot read Gmail. `gws-auth status` reports which scopes the token carries; a service verb whose scope is absent surfaces "token lacks <scope> for <service>", never a bare 403.

### 4. Read-mostly — write verbs deferred

Every EPIC-13 verb **reads**. The obvious writes — `gmail send`, `calendar events.insert`, `drive files.create/upload` — are **deferred** to future epics, the same discipline gcloud-compute applied to `delete` (BWOC-66 §Decision 2). Sending mail / creating events / uploading files are higher-blast-radius (externally visible, some irreversible) and each deserves the operator-confirm write-gate treatment ([PLUGINS §Write verbs](../docs/en/PLUGINS.en.md), BWOC-67) in its own slice. The read-mostly foundation earns trust + the OAuth surface first.

### 5. Rate limits + pagination

Workspace APIs have per-user quotas + return `429`. `gws.sh` honors `Retry-After` on `429` (the jira/figma precedent). List verbs paginate under the hood (Drive `files.list`, Gmail `threads.list`, Calendar `events.list` all page) and surface a single bounded envelope; an explicit `--max <n>` caps results so an agent never pulls an unbounded inbox.

## Workspace resource schema (for BWOC-73)

`BWOC-73` adds normative per-service resource shapes (parallel to Audit Findings / Jira Issue Mapping / Figma Asset Mapping). One entry shape per service; `BWOC-73` finalizes fields. Sketch:

- **Drive file**: `file_id` (stable key), `name`, `mime_type`, `modified_time`, `owners`, `web_view_link`.
- **Gmail thread**: `thread_id` (stable key), `subject`, `from`, `snippet`, `labels`, `last_message_time`.
- **Calendar event**: `event_id` (stable key), `summary`, `start`, `end`, `attendees_count`, `calendar_id`.

Each keys on its stable id; the rest are mutable projections refreshed each read. Optional fields omitted (not `null`), per the framework convention.

## Alternatives considered

- **Fold into `gcloud` / reuse `workflow`** — rejected (Decision 1). Different auth, surface, and a normative schema → own kind.
- **One mega `gws` plugin** — rejected (Decision 2). Per-service plugins + a shared auth foundation let future write slices + new services (Sheets, Docs) attach without a mega-plugin; gcloud-* precedent.
- **Ship a write verb (e.g. gmail send) in the foundation** — rejected (Decision 4). Read-mostly first; writes are per-slice with confirm gates.
- **Service-account auth (like gcloud)** — rejected for the foundation. Workspace user data needs **user-consent OAuth** scopes, not a service account (a SA can't read a user's personal Gmail without domain-wide delegation, which is an admin concern out of scope here).

## Status / deferred

- Decisions 1-5 frozen for EPIC-13 unless BWOC-73/75 surface a contradiction.
- **Write verbs** (send / create / upload) deferred to future epics, each with its own confirm-gate.
- **Sheets / Docs / Slides / Admin** services deferred — the foundation does Drive/Gmail/Calendar; more services attach to the gws-auth foundation later.
- **Live verification** gates on an operator OAuth token (+ consented scopes); build + unit-test (auth shape, request construction, pagination) without it.
- Domain-wide delegation / service-account Workspace access — out of scope (admin concern).

## Related

- EPIC-8 [gcloud-workflow note](2026-05-28_gcloud-workflow-plugin-architecture.md) — the credential-foundation + per-service-plugin family shape; the Google integration `gws` is explicitly NOT.
- EPIC-7 [figma note](2026-05-28_figma-plugin-architecture.md) — the read-mostly + own-kind-for-a-schema precedent.
- [PLUGINS.en.md §Write verbs — operator-confirm gate](../docs/en/PLUGINS.en.md) — the gate the deferred write slices will inherit.
- [PLUGINS.en.md §Plugin Kinds](../docs/en/PLUGINS.en.md#plugin-kinds) — the enumeration `BWOC-73` extends.
