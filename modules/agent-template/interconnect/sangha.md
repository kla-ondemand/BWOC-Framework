---
title: Saṅgha — Teams & Shared Task List
aliases:
  - Saṅgha
  - Agent Teams
  - Saṅgaha-vatthu 4
tags:
  - group/agents
  - type/design
  - meta/template
status: draft (v2026.5.23 — Phase A: membership + shared task list + self-claim)
canonical-source: Saṅgaha-vatthu (AN 4.32, DN 31) · Saṅghakamma (Vinaya, Mahāvagga)
---

# Saṅgha — Teams & Shared Task List

> [!abstract] A **team** (saṅgha) groups a subset of a workspace's agents under one shared task list. The human operator is the implicit lead. Teammates **self-claim** pending, unblocked tasks; an advisory file lock makes each claim a Saṅghakamma — a communal act settled by exactly one member. This is Phase A: the CLI + on-disk foundation. Daemon task-watch, plan approval (Pavāraṇā), and team-aware dashboard are later phases.

## Motivation

BWOC already has the pieces for agents to *talk* — the inbox ([`send.rs`](../../../crates/bwoc-cli/src/send.rs)), verified sender identity + the Kalyāṇamitta-7 trust gate ([[trust]]), and Sāraṇīyadhamma-6 cordiality norms ([[messaging]]). What it lacked was a place for agents to *coordinate work*: a shared list of tasks they can claim, complete, and gate on each other.

Compared to one-shot subagents (which only report back), a team shares a task list and lets members pick up work independently — the same distinction Claude Agent Teams draws between subagents and teammates. BWOC's twist: the coordination state is plain files in the workspace (`.bwoc/teams/`), transport-agnostic and readable by any backend running through `bwoc spawn`.

## Buddhist grounding

| Concept | Source | BWOC application |
|---|---|---|
| **Saṅgha** | the community of practitioners | A team — a named, bounded set of agents working a common list. ≥1 member; the human is lead, never a member. |
| **Saṅgaha-vatthu 4** | AN 4.32 — four bases of social cohesion | The norms a team runs on: **dāna** (share findings, don't hoard), **peyyavajja** (kindly task titles + messages — see [[messaging]]), **atthacariyā** (claim work that helps the team, not just yourself), **samānattatā** (every member equal before the task list — no privileged claimer). |
| **Saṅghakamma** | Vinaya, Mahāvagga — formal communal acts | The claim protocol: a task transitions to one member under a lock, so two members never claim the same item. The lock *is* the quorum that makes the act valid. |

> [!note] Saṅgaha-vatthu 4 are **norms, not gates** in Phase A — `bwoc` does not enforce kindly titles or unselfish claims. They live here so an incarnated agent can internalize them, the same way [[messaging]] carries Sāraṇīyadhamma 6.

## Data model

A team is membership + a task list.

```toml
# .bwoc/teams/<team-id>.toml
id = "squad"
members = ["agent-pi", "agent-oracle"]
created_at = "2026-05-23T06:47:15Z"
```

```jsonl
# .bwoc/teams/<team-id>/tasks.jsonl  — one task per line
{"id":"t1","title":"design schema","state":"completed","created_at":"…","claimed_by":"agent-pi","completed_at":"…"}
{"id":"t2","title":"implement","state":"in_progress","deps":["t1"],"created_at":"…","claimed_by":"agent-oracle"}
```

- **No `lead` field.** The human operator is the implicit lead — they create the team, add tasks, and synthesize results. (A designated lead *agent* is a possible v2 extension, not v1.)
- **`deps`** is a list of task ids that must be `completed` before a task is claimable. Omitted when empty.
- **`claimed_by`** is set on claim and kept through completion (audit trail of who did the work).

### Task state machine

```
pending ──claim──▶ in_progress ──complete──▶ completed
```

- **claim**: task must be `pending` AND every dependency `completed`. Sets `in_progress` + `claimed_by`.
- **complete**: task must be `in_progress` AND the actor must be the claimant. Sets `completed` + `completed_at`.
- Tasks are never deleted (completed tasks stay for the audit trail), so auto-ids (`t1`, `t2`, …) are monotonic.

## CLI surface

```bash
# Teams
bwoc team create <id> --members a,b,c     # define a team
bwoc team list                            # teams + member/task counts
bwoc team retire <id> [--yes]             # remove membership + task list (destructive)

# Tasks (operate on one team)
bwoc task add <team> "<title>" [--deps t1,t2] [--id <custom>]
bwoc task list <team>                     # id · state · claimant · title
bwoc task claim <team> <task> --as <agent>      # self-claim (member only)
bwoc task complete <team> <task> --as <agent>   # claimant only
```

All commands take `--workspace` (standard resolution: flag → `BWOC_WORKSPACE` → ancestor walk → cwd) and `--json` for structured output.

> [!example] An agent running inside `bwoc spawn` self-claims with its own id:
> ```bash
> bwoc task claim squad t2 --as agent-oracle
> ```
> Claiming a blocked or already-claimed task exits `2` with an actionable message; the task file is left untouched.

## Concurrency — the lock

`bwoc task add/claim/complete` acquire an advisory lock (`.bwoc/teams/<id>/tasks.lock`) before the read-modify-write. The lock is a dependency-free `O_CREAT | O_EXCL` file holding the holder's PID; a stale lock (PID dead, signal-0 probe) is stolen. Two agents racing to claim the same task serialize: one wins (`in_progress`), the other reads the now-`in_progress` task and is refused. Verified live with two concurrent `bwoc task claim` processes.

## Refusal & exit codes

| Situation | Exit | Message shape |
|---|---|---|
| Blocked by dependency | 2 | `task 't2' is blocked: dependency 't1' is not completed` |
| Already claimed / wrong state | 2 | `task 't1' is in_progress — only pending tasks can be claimed` |
| Non-member claims | 2 | `agent 'x' is not a member of this team (members: …)` |
| Complete by non-claimant | 2 | `task 't1' is claimed by 'agent-pi', not 'agent-oracle'` |
| Lock contention timeout | 1 | `could not acquire task lock (… remove tasks.lock if stale)` |

## Deferred (later phases)

- **Daemon task-watch + hooks** — `task-created` / `task-completed` events so a running `bwoc-agent --serve` reacts to list changes. (Phase B.)
- **Plan approval (Pavāraṇā)** — a teammate submits a plan, the lead approves/rejects before implementation. Maps to an envelope-kind extension on [[messaging]]. (Phase C.)
- **Team-aware dashboard** — a task pane in `bwoc dashboard`. (Phase B+.)
- **Designated lead agent** — a `lead` field + lead-only operations. Only if the human-implicit-lead model proves limiting.

## Related

- [[trust]] — Kalyāṇamitta-7; gates *who* a teammate accepts messages from.
- [[messaging]] — Sāraṇīyadhamma 6; the channel teammates talk over.
- [`crates/bwoc-core/src/team.rs`](../../../crates/bwoc-core/src/team.rs) — data model + transition rules.
- [`crates/bwoc-cli/src/sangha.rs`](../../../crates/bwoc-cli/src/sangha.rs) — CLI + lock.
- Fleet-level governance (operator view of many agents): `docs/en/FLEET-GOVERNANCE.en.md` (Aparihāniya-dhamma 7).
