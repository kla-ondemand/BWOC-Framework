---
title: gcloud Compute Lifecycle — write-verb risk matrix + confirm gates (EPIC-9)
date: 2026-05-28
sprint: BWOC sprint-12
epic: BWOC-EPIC-9
story: BWOC-66
related_stories: [BWOC-67, BWOC-68, BWOC-69, BWOC-70]
upstream: bemindlabs#96
---

# 2026-05-28 — gcloud Compute Lifecycle (EPIC-9 framing)

This note frames `BWOC-EPIC-9` — the **first write-capable gcloud slice** — before code lands. EPIC-8 shipped a read-mostly gcloud foundation (`gcloud-auth` + `gcloud-project`) and deliberately deferred every write surface ([BWOC-51 §Decision 7](2026-05-28_gcloud-workflow-plugin-architecture.md)). EPIC-9 opens that surface, starting with the lowest-blast-radius slice: compute instance lifecycle. It answers the questions `BWOC-67` (PLUGINS write-verb gate pattern), `BWOC-68` (`bwoc gcloud compute` CLI), `BWOC-69` (the `gcloud-compute` plugin), and `BWOC-70` (the `bwoc check` ext) must build against.

The throughline: **this is the framework's first agent-reachable verb that changes remote infrastructure state.** Everything before it either read (audit/okr/gcloud-read/figma), coordinated locally (council), or wrote to a tracker/local-file (jira/okr-track/gcloud-set-default). `gcloud compute start/stop` flips a real VM — it costs money and affects running workloads. The whole design is about gating that safely while keeping the read path frictionless.

## Decisions

### 1. `gcloud-compute` is a `workflow` plugin — no new kind

It extends the `gcloud-*` family established in EPIC-8 ([BWOC-51 §Decision 1](2026-05-28_gcloud-workflow-plugin-architecture.md)): a passthrough to the local `gcloud` CLI with no BWOC-owned normative schema. Same reasoning that kept `gcloud-auth`/`gcloud-project` under `workflow` rather than minting a `gcp` kind. `gcloud-compute` sources the **same credential resolution** from `gcloud-auth/gcloud.sh` (the shared-helper pattern from EPIC-8 §Decision 2) — it adds verbs, not a kind.

### 2. Write-verb risk matrix

The slice ships three verbs; the matrix sets each one's gate:

| Verb | Class | Effect | Gate | Reversible? |
|---|---|---|---|---|
| `list` | read | `gcloud compute instances list` — reads only | none (reads are free) | — |
| `start` | **write** | `gcloud compute instances start <name>` — boots a VM (incurs cost) | **operator-confirm** (in the CLI) | yes (`stop`) |
| `stop` | **write** | `gcloud compute instances stop <name>` — halts a VM (interrupts workloads) | **operator-confirm** (in the CLI) | yes (`start`) |
| ~~`delete`~~ | **destructive** | would tear down a VM + disks | **DEFERRED — out of scope** | **no** |

`delete` is **explicitly excluded** from EPIC-9. It is irreversible (loses the instance + attached state); a destructive verb deserves its own deliberate treatment (a stronger gate, a typed-confirmation, maybe a dry-run-only default) and is deferred until the safer start/stop verbs are exercised. Anattā — don't ship the dangerous verb just because it's adjacent.

### 3. Operator-confirm gate model

The write verbs gate in the **CLI** (`bwoc gcloud compute`, BWOC-68), not the plugin — same place `gcloud-project set-default` (EPIC-8) and the jira write verbs gate. The gate:

1. **Shows the exact effect before acting**: the project, zone, instance name, current state, and the literal `gcloud` command that will run (with the `--` separator per the [#92 hardening](2026-05-28_gcloud-option-injection-hardening.md)).
2. **Requires explicit operator confirmation** — a y/N prompt; default **No**. Non-interactive contexts (headless agents) must pass an explicit `--yes` flag, which the agent only sets when the operator has authorized the specific action. No silent writes.
3. **Is observed by Dhammānupassanā** — the gate state is part of the output: a refused/un-confirmed write reports "no change" with the reason, never a bare failure.

The plugin (`gcloud-compute/gcloud.sh`) executes the verb when invoked; it does **not** re-implement the gate (single gate, at the CLI boundary — no double-gating, no gate-bypass via direct plugin invoke because the plugin's write verbs check for the CLI-set confirmation marker).

### 4. Reuse, don't reinvent

- **Credential resolution**: source `gcloud-auth/gcloud.sh` helpers (ADC → SA JSON → env). `gcloud-compute` holds no auth of its own.
- **Arg hardening**: every user-supplied arg (instance name, zone, project) passes to `gcloud` after a `--` separator (the [#92 hardening](2026-05-28_gcloud-option-injection-hardening.md)) so it can never be parsed as a flag.
- **Graceful degradation**: missing `gcloud` CLI / unauthenticated / 4xx → clear actionable error, no panic (the EPIC-8 pattern).
- **No new normative schema** — compute output is `gcloud`'s JSON surfaced through; BWOC owns no Compute Mapping schema (unlike jira/figma). This keeps it a `workflow` passthrough.

### 5. CLI shape — `bwoc gcloud compute <verb>`

`compute` is a **subcommand under the existing `bwoc gcloud`** (which already has `auth` / `project` / `status` from EPIC-8), not a new top-level command:

```
bwoc gcloud compute list   [--project <p>] [--zone <z>]        # read
bwoc gcloud compute start  --instance <name> --zone <z> [--yes] # gated write
bwoc gcloud compute stop   --instance <name> --zone <z> [--yes] # gated write
```

`--json` twins throughout. This keeps the whole gcloud surface under one command tree, consistent with how `bwoc jira` groups its verbs.

## Alternatives considered

- **Ship `delete` too** — rejected (Decision 2). Irreversible; deserves its own deliberate gate design. Defer.
- **Gate in the plugin instead of the CLI** — rejected (Decision 3). The CLI is the operator boundary; gating there keeps one confirmation point and matches jira/gcloud-set-default precedent.
- **A new `gcp-compute` kind / a Compute Mapping schema** — rejected (Decision 1/4). It's a `gcloud` CLI passthrough with no BWOC-owned data shape; stays a `workflow` plugin.
- **Auto-confirm in headless mode** — rejected (Decision 3). Headless agents must pass `--yes` only on explicit operator authorization; never auto-set it.

## Status / deferred

- Decisions 1-5 frozen for EPIC-9 unless BWOC-68/69 surface a contradiction.
- `delete` deferred (Decision 2) — a future story/epic with a stronger gate.
- **Live verification** (start/stop a real instance) gates on an operator GCP project + a disposable test instance; build + unit-test (gate logic, arg construction) without it.
- The write-verb gate pattern this note pins is formalized in PLUGINS by `BWOC-67` (so EPIC-10/11/12 storage/run/iam slices inherit it) — the matrix here is the compute instance.

## Related

- EPIC-8 [gcloud-workflow note](2026-05-28_gcloud-workflow-plugin-architecture.md) — the read-mostly foundation + §7 deferral this slice opens; §Decision 1/2 (workflow-reuse + shared-helper).
- [#92 gcloud `--` hardening note](2026-05-28_gcloud-option-injection-hardening.md) — the arg-separator safety reused here.
- EPIC-6 [jira note](2026-05-27_jira-plugin-architecture.md) — the operator-confirm-on-write precedent.
- Upstream tracking issue: bemindlabs#96 (EPIC-9). The §7 roadmap (compute → storage → run → iam, lowest-blast-radius first) is this slice's place in line.
