---
title: gcloud-compute — Google Cloud Compute Engine Lifecycle
aliases:
  - gcloud-compute
tags:
  - group/framework-plugins
  - type/plugin
  - kind/workflow
  - domain/integration
  - integration/gcloud
maturity: L1
---

# gcloud-compute — Google Cloud Compute Engine Lifecycle

> [!abstract] The first **write-capable** reference `workflow` plugin for the GCP family (`BWOC-EPIC-9`). It owns **Compute Engine instance lifecycle** — listing instances, starting one, stopping one. Verbs: `list` (read), `start` / `stop` (**write — operator-confirm gated in the CLI**, and additionally guarded here by the CLI-set confirmation marker `confirmed: true` in the request). `delete` is deliberately **not** shipped (deferred — irreversible). Sources credential helpers from the sibling [[../gcloud-auth/SPEC|`gcloud-auth`]] plugin. Full framing: [[../../../notes/2026-05-28_gcloud-compute-write-verbs|BWOC-66 design note]].

## Why a write-capable `workflow` plugin

EPIC-8 shipped a read-mostly gcloud foundation (`gcloud-auth` + `gcloud-project`) and deferred every write surface. `gcloud-compute` opens that surface with the lowest-blast-radius slice: instance start/stop. It stays a `workflow` plugin — not a new `gcp` kind — because the framework owns no normative Compute schema; the agent calls out and `gcloud`'s own JSON is surfaced through (design note §Decision 1). It sources the **same** credential resolution from `gcloud-auth/gcloud.sh` (the shared-helper pattern from EPIC-8 §Decision 2) — it adds verbs, not a kind, and holds no auth of its own.

`start` / `stop` are the framework's **first agent-reachable verbs that change remote infrastructure state** — they cost money and interrupt workloads. The whole design gates that safely while keeping the read path frictionless.

## Verbs

| Operation | Direction | Auth | `gcloud` shell-out | Gate |
|---|---|---|---|---|
| `list` | read | required | `gcloud compute instances list --format=json` (optional `--zones=` / `--project=` filters) | none |
| `start` | **write** | required | `gcloud compute instances start --zone=<z> [--project=<p>] --format=json -- <name>` — boots a VM (incurs cost) | **operator-confirm** (in the `bwoc gcloud compute` CLI — BWOC-68) + `confirmed: true` marker guard here |
| `stop` | **write** | required | `gcloud compute instances stop --zone=<z> [--project=<p>] --format=json -- <name>` — halts a VM (interrupts workloads) | **operator-confirm** + `confirmed: true` marker guard |

`start` / `stop` are reversible (each undoes the other). `delete` — irreversible (loses the instance + disks) — is **excluded** from EPIC-9 and deferred to a future slice with a stronger gate (design note §Decision 2). Read verbs carry **no** gate.

## The write gate (normative)

Per [[../../../docs/en/PLUGINS.en#Write verbs — the operator-confirm gate (normative)|PLUGINS.en.md §Write verbs]], the operator-confirm gate lives at the **CLI boundary** (`bwoc gcloud compute`, BWOC-68) — one confirmation point, shown before acting (target, zone, current state, the literal `gcloud` command), default **No**, `--yes` for non-interactive contexts. This plugin does **not** re-prompt.

It **does** carry a defense-in-depth guard (design note §Decision 3): the write verbs refuse to execute unless the CLI-set confirmation marker `confirmed: true` is present in the stdin request (the `bwoc gcloud compute` CLI adds it only after the operator-confirm gate passes — BWOC-68 `compute_write_request`). So a direct plugin invocation — bypassing the CLI gate — is refused with a structured "no change" envelope (`ok: false`, `changed: false`, `reason: "unconfirmed-write"`) rather than a silent write or a bare failure (Dhammānupassanā). The marker is the only coupling between the gate and the plugin; the manifest's `[[verb]]` table declares which verbs are writes so both `bwoc check` (BWOC-70) and the CLI gate can see the classification.

## How it runs

The `bwoc gcloud compute` CLI (`BWOC-68`) spawns `gcloud.sh` from this directory:

| Channel | What it carries |
|---|---|
| `BWOC_GCLOUD_OPERATION` (env) | `list` \| `start` \| `stop` — fallback for `.operation` when stdin is empty. |
| `BWOC_WORKSPACE` (env) | Absolute workspace root; the sibling SA JSON path resolves under it. |
| `BWOC_PLUGIN_DIR` (env) | Absolute path to this plugin's directory; used to find the sibling helpers at `../gcloud-auth/gcloud.sh`. |
| stdin | One-line JSON, e.g. `{"operation":"list"}`, `{"operation":"start","instance":"vm-1","zone":"us-central1-a","confirmed":true}`, `{"operation":"stop","instance":"vm-1","zone":"us-central1-a","project":"my-proj","confirmed":true}`. |
| `.confirmed` (in the stdin JSON) | Write-confirmation marker — set to `true` by the CLI **after** the operator confirms a write. Read verbs ignore it; `start` / `stop` refuse without it. |

On success: exit `0`, one JSON object on stdout. On error: human diagnostic on stderr + non-zero exit.

### Argument hardening (#92)

Every user-supplied **positional** (the instance name) is passed to `gcloud` after a `--` end-of-options separator; every user-supplied **flag value** (zone, project) is bound with `=` in a single argv token. Neither can be parsed by `gcloud` as a flag — a `-`-leading instance name or zone is neutralized. Mirrors the [[../../../notes/2026-05-28_gcloud-option-injection-hardening|#91/#92 hardening]] the sibling plugins use.

## Authentication

This plugin **never reads any credential value**. It sources the sibling [[../gcloud-auth/SPEC#Authentication|`gcloud-auth`]] helpers (`gcloud_assert_cli`, `gcloud_assert_authenticated`) at startup and asks `gcloud` to act. Credential precedence is identical to [[../gcloud-auth/SPEC#Authentication|`gcloud-auth §Authentication`]]: ADC → `.bwoc/secrets/gcloud-sa.json` → `BWOC_GCLOUD_*` env. The plugin fails fast with a clear diagnostic if there is no active credential.

Unlike the sibling plugins, `gcloud-compute` ships **no** `auth.toml` — it holds no auth of its own (design note §Decision 4); the credential contract is the sibling's. `bwoc check` does not audit a workflow plugin that ships no `auth.toml`.

> [!danger] **Sila — Adinnādāna.** No token ever enters this plugin's address space. We only call `gcloud` and surface its output. `start` / `stop` transmit, log, and persist no credential in any form.

## Output shapes

### `list`

```json
{
  "ok": true,
  "plugin": "gcloud-compute",
  "operation": "list",
  "total": 2,
  "instances": [
    { "name": "vm-1", "zone": "us-central1-a", "machine_type": "e2-medium", "status": "RUNNING",     "internal_ip": "10.128.0.2", "creation_timestamp": "2026-05-01T08:00:00.000-07:00" },
    { "name": "vm-2", "zone": "us-central1-b", "machine_type": "e2-small",  "status": "TERMINATED",  "internal_ip": "10.128.0.3", "creation_timestamp": "2026-05-02T09:00:00.000-07:00" }
  ]
}
```

`zone` and `machine_type` are shortened from the full `gcloud` resource URLs to their last path segment. `total` is the array length `gcloud` returns; the plugin does not re-paginate.

### `start` / `stop`

```json
{
  "ok": true,
  "plugin": "gcloud-compute",
  "operation": "start",
  "instance": "vm-1",
  "zone": "us-central1-a",
  "changed": true,
  "result": { "...": "the raw gcloud --format=json result" }
}
```

`result` carries `gcloud`'s own JSON output verbatim (workflow passthrough — BWOC owns no Compute Mapping shape). `stop` returns the same envelope with `"operation": "stop"`.

### Refused write (no confirmation marker)

```json
{
  "ok": false,
  "plugin": "gcloud-compute",
  "operation": "start",
  "changed": false,
  "reason": "unconfirmed-write",
  "message": "write verb 'start' requires operator confirmation; the bwoc gcloud compute CLI sets the confirmation marker (\"confirmed\": true) in the request after a y/N prompt. Direct plugin invocation of a write verb is refused — no instance was changed."
}
```

Emitted (exit `5`) when a write verb is invoked without `confirmed: true` in the request. In the normal CLI path the marker is always set after the operator confirms, so this only fires on a direct-invoke bypass.

## Error classes

| Exit | Class | Meaning |
|---|---|---|
| `0` | success | One JSON object on stdout. |
| `1` | dependency | `jq` missing, or sibling `gcloud-auth/gcloud.sh` not installed alongside, or `gcloud` missing (per `gcloud_assert_cli`). |
| `2` | usage | Unknown operation, or `start` / `stop` invoked without `.instance` or `.zone`. |
| `3` | not-authenticated | No active `gcloud` credential (per `gcloud_assert_authenticated`). |
| `5` | unconfirmed-write | A write verb (`start` / `stop`) invoked without the `confirmed: true` marker — refused, no change made. |
| `6` | gcloud-error | The underlying `gcloud compute` command failed (e.g. instance not found, permission denied, 4xx); the truncated diagnostic is on stderr. |

Missing `gcloud` CLI fails **gracefully**: a clear stderr message + non-zero exit; the plugin never panics.

## Configuration

```toml
# workspace.toml
[plugins.gcloud-compute]
enabled = true
```

No `[config.schema]` — compute state is read live from `gcloud`. The only workspace-level surface is the universal `enabled` key. The `[[verb]]` tables in `manifest.toml` declare the write classification (consumed by `bwoc check` BWOC-70 + the CLI gate BWOC-68), not operator config.

## Lifecycle mapping

Per [[../../../docs/en/PLUGINS.en#Lifecycle|PLUGINS.en.md §Lifecycle]], the `workflow` kind's owner is the **agent** calling out (via the `bwoc gcloud compute` CLI). `init`/`teardown` are per-invocation around `invoke`. The plugin holds **no local state** beyond what `gcloud` itself caches.

| Phase | What this plugin does |
|---|---|
| `init` | Implicit; source the sibling helpers; verify `jq` on PATH. |
| `invoke` | Read the request, (for writes) check the confirmation marker, call `gcloud compute instances ...`, emit JSON. |
| `teardown` | Implicit; no state to release. |

## Idempotency

- `list` is read-only.
- `start` is idempotent: starting an already-`RUNNING` instance is a no-op for `gcloud` and returns success. `stop` likewise on an already-`TERMINATED` instance. Replays after a transient `gcloud` error converge to the requested lifecycle state.

## Maturity

Declared **L1** — first runnable `workflow/gcloud-compute` reference plugin; all three verbs functional, the write gate wired. Bumps to **L2** once the `bwoc check` extension (`BWOC-70`) and smoke tests exercise the verbs end-to-end against a real instance.

> [!warning] Live-test gap. Live end-to-end (actually starting/stopping a real VM) is gated on an operator-provided GCP project + a disposable test instance (design note §Status). v0.1.0 is verified by: `bash -n gcloud.sh`, the missing-`gcloud-auth` sibling path erroring cleanly, the unauthenticated path returning the structured `not-authenticated` diagnostic, the unconfirmed-write path returning the structured `unconfirmed-write` refusal, and `bwoc check` accepting the manifest.

## Neutrality

Manifest values name no LLM backend or model. `kind = "workflow"` is the framework's own enum value. "gcloud" / "Google Cloud" / "Compute Engine" appear only in `description` (where integration-target names are tolerated per [[../../../docs/en/PLUGINS.en#Neutrality constraint (HARD)|PLUGINS.en.md §Neutrality]]) and in this SPEC's prose — never in `kind`, `entry`, or config keys. Satisfies **Samānattatā**.

## See Also

- [[../../../notes/2026-05-28_gcloud-compute-write-verbs|BWOC-66 design note]] — the write-verb risk matrix + confirm-gate framing (decisions 1–5).
- [[../gcloud-auth/SPEC|gcloud-auth SPEC]] — sibling plugin (credential state); helpers sourced here.
- [[../gcloud-project/SPEC|gcloud-project SPEC]] — sibling plugin (project context); the same shared-helper + write-gate precedent.
- [[../../../docs/en/PLUGINS.en|PLUGINS.en.md]] — the plugin spec; `workflow` kind row + §Write verbs gate.
- [[SPEC.th|SPEC.th.md]] — Thai counterpart (bilingual parity).
