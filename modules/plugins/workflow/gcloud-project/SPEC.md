---
title: gcloud-project — Google Cloud Project Context
aliases:
  - gcloud-project
tags:
  - group/framework-plugins
  - type/plugin
  - kind/workflow
  - domain/integration
  - integration/gcloud
maturity: L1
---

# gcloud-project — Google Cloud Project Context

> [!abstract] The second of two reference `workflow` plugins for the GCP foundation (`BWOC-EPIC-8`). It owns **project context** — listing accessible projects, describing one project, and setting the local `gcloud` default project. Verbs: `list`, `show`, `set-default` (the only write verb in the foundation; **operator-confirm gated in the CLI**, and it touches only the local `gcloud` config DB — no remote API mutation). Sources credential helpers from the sibling [[../gcloud-auth/SPEC|`gcloud-auth`]] plugin. Full framing: [[../../../notes/2026-05-28_gcloud-workflow-plugin-architecture|BWOC-51 design note]].

## Why two plugins, why the `workflow` kind

`gcloud-project` ships separately from `gcloud-auth` so future GCP slices (`gcloud-compute`, `gcloud-storage`, …) can depend on the auth foundation without inheriting project-management verbs. The shared helpers live in `gcloud-auth/gcloud.sh` and are sourced here at startup — credential resolution is defined exactly once. The `workflow` kind (not a new `gcp` kind) is the right shape because the framework does not own the lifecycle: agents call out, gcloud's output is surfaced. Full rationale: design note decisions 1 + 2.

## Verbs

| Operation | Direction | Auth | HTTP / side effect | Gate |
|---|---|---|---|---|
| `list` | read | required | `gcloud projects list` (paged under the hood, single envelope surfaced) | none |
| `show` | read | required | `gcloud projects describe <id>` (defaults to `gcloud config get-value project` when `.project` is omitted) | none |
| `set-default` | **write (local)** | required | `gcloud config set project <id>` — mutates `~/.config/gcloud/configurations/...` only; no remote API call | **operator confirmation** (in the `bwoc gcloud` CLI — BWOC-52) |

`set-default` is the only write verb in the EPIC-8 foundation. Risk is local to the operator's machine; reversibility is trivial (`gcloud config set project <previous>`). The CLI confirmation gate stays on because a wrong default silently routes subsequent agent verbs at the wrong project — the kind of footgun this slice explicitly tries to avoid (design note §Decision 4).

## How it runs

The `bwoc gcloud` CLI (`BWOC-52`) spawns `gcloud.sh` from this directory:

| Channel | What it carries |
|---|---|
| `BWOC_GCLOUD_OPERATION` (env) | `list` \| `show` \| `set-default` — fallback for `.operation` when stdin is empty. |
| `BWOC_WORKSPACE` (env) | Absolute workspace root; the sibling SA JSON path resolves under it. |
| `BWOC_PLUGIN_DIR` (env) | Absolute path to this plugin's directory; used to find the sibling helpers at `../gcloud-auth/gcloud.sh`. |
| stdin | One-line JSON, e.g. `{"operation":"list"}`, `{"operation":"show","project":"my-proj"}`, `{"operation":"set-default","project":"my-proj"}`. |

On success: exit `0`, one JSON object on stdout. On error: human diagnostic on stderr + non-zero exit.

## Authentication

This plugin **never reads any credential value**. It sources the sibling [[../gcloud-auth/SPEC#Authentication|`gcloud-auth`]] helpers (`gcloud_assert_cli`, `gcloud_assert_authenticated`) at startup and asks `gcloud` for project state. [[auth|auth.toml]] declares the **same** auth contract (shape only) as the sibling so:

- `bwoc check` can validate the contract independently per plugin.
- An operator inspecting this plugin alone sees the full auth model without chasing the sibling.

Credential precedence is identical to [[../gcloud-auth/SPEC#Authentication|`gcloud-auth §Authentication`]]: ADC → `.bwoc/secrets/gcloud-sa.json` → `BWOC_GCLOUD_*` env. The plugin fails fast with a clear diagnostic if there is no active credential.

> [!danger] **Sila — Adinnādāna.** Same guarantee as the sibling: no token ever enters this plugin's address space. We only call `gcloud` and surface its output. `set-default` does not transmit, log, or persist credentials in any form — it writes only the project ID to `~/.config/gcloud/`.

## Output shapes

### `list`

```json
{
  "ok": true,
  "plugin": "gcloud-project",
  "operation": "list",
  "total": 3,
  "projects": [
    { "project_id": "my-proj-1", "project_number": "111111111111", "name": "My Project 1", "lifecycle_state": "ACTIVE" },
    { "project_id": "my-proj-2", "project_number": "222222222222", "name": "My Project 2", "lifecycle_state": "ACTIVE" },
    { "project_id": "archived",  "project_number": "333333333333", "name": "Archived",    "lifecycle_state": "DELETE_REQUESTED" }
  ]
}
```

`total` is the array length the local `gcloud` returns; if the operator has many projects, `gcloud` itself handles paging and returns the full set. The plugin does not re-paginate.

### `show`

```json
{
  "ok": true,
  "plugin": "gcloud-project",
  "operation": "show",
  "project_id": "my-proj-1",
  "project_number": "111111111111",
  "name": "My Project 1",
  "lifecycle_state": "ACTIVE",
  "create_time": "2024-01-15T08:00:00Z",
  "parent": { "type": "organization", "id": "1234567890" },
  "labels": { "env": "prod" }
}
```

When `.project` is omitted, the plugin uses `gcloud config get-value project`. If that is also unset, it exits `2` with `no project ...`.

### `set-default`

```json
{
  "ok": true,
  "plugin": "gcloud-project",
  "operation": "set-default",
  "previous": "my-proj-1",
  "current": "my-proj-2",
  "note": "Local gcloud config only; no remote API mutation."
}
```

`previous` is `null` when no default was set.

## Error classes

| Exit | Class | Meaning |
|---|---|---|
| `0` | success | One JSON object on stdout. |
| `1` | dependency | `jq` missing, or sibling `gcloud-auth/gcloud.sh` not installed alongside, or `gcloud` missing (per `gcloud_assert_cli`). |
| `2` | usage | Unknown operation, or `set-default` invoked without `.project`, or `show` invoked with no project and no `gcloud config` default. |
| `3` | not-authenticated | No active `gcloud` credential (per `gcloud_assert_authenticated`). |
| `6` | gcloud-error | The underlying `gcloud` command failed; the truncated diagnostic is on stderr. |

Missing `gcloud` CLI fails **gracefully**: a clear stderr message + non-zero exit; the plugin never panics.

## Configuration

```toml
# workspace.toml
[plugins.gcloud-project]
enabled = true
```

No `[config.schema]` — project context is queried live. The only workspace-level surface is the universal `enabled` key.

## Lifecycle mapping

Per [[../../../docs/en/PLUGINS.en#Lifecycle|PLUGINS.en.md §Lifecycle]], the `workflow` kind's owner is the **agent** calling out (via the `bwoc gcloud` CLI). `init`/`teardown` are per-invocation around `invoke`. The plugin holds **no local state** beyond what `gcloud` itself caches.

| Phase | What this plugin does |
|---|---|
| `init` | Implicit; source the sibling helpers; verify `jq` on PATH. |
| `invoke` | Read the request, call `gcloud projects ...` (or `gcloud config set project`), emit JSON. |
| `teardown` | Implicit; no state to release. |

## Idempotency

- `list` and `show` are read-only.
- `set-default` is idempotent: setting the project to its current value is a no-op for `gcloud` config, and the plugin's envelope reports `previous == current` in that case. Replays after a transient `gcloud` error converge.

## Maturity

Declared **L1** — first runnable `workflow/gcloud-project` reference plugin; all three verbs functional. Bumps to **L2** once the `bwoc check` extension (`BWOC-55`) and smoke tests exercise the verbs end-to-end against an authenticated local `gcloud`.

> [!warning] Live-test gap. Live end-to-end against a real GCP project is gated on an operator-provided sandbox SA JSON at `.bwoc/secrets/gcloud-sa.json` (design note §Status). v0.1.0 is verified by: `bash -n gcloud.sh`, the missing-`gcloud-auth` sibling path erroring cleanly, the unauthenticated path returning the structured `not-authenticated` diagnostic, and `bwoc check` accepting the manifest.

## Neutrality

Manifest values name no LLM backend or model. `kind = "workflow"` is the framework's own enum value. "gcloud" / "Google Cloud" appear only in `description` (where integration-target names are tolerated per [[../../../docs/en/PLUGINS.en#Neutrality constraint (HARD)|PLUGINS.en.md §Neutrality]]) and in this SPEC's prose — never in `kind`, `entry`, or config keys. Satisfies **Samānattatā**.

## See Also

- [[../../../notes/2026-05-28_gcloud-workflow-plugin-architecture|BWOC-51 design note]] — full framing (decisions 1–7).
- [[../gcloud-auth/SPEC|gcloud-auth SPEC]] — sibling plugin (credential state); helpers sourced here.
- [[../../../docs/en/PLUGINS.en|PLUGINS.en.md]] — the plugin spec; `workflow` kind row.
- [[auth|auth.toml]] — the auth contract (shape only; no values).
- [[SPEC.th|SPEC.th.md]] — Thai counterpart (bilingual parity).
