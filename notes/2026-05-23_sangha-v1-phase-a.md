---
date: 2026-05-23
session: Saṅgha v1 Phase A — teams + shared task list
tags:
  - phase/3
  - type/note
  - module/bwoc-core
  - module/bwoc-cli
---

# 2026-05-23 — Saṅgha v1 Phase A: Teams + Shared Task List

The agent-teams foundation. A **team** groups a subset of workspace agents under one shared task list; teammates **self-claim** pending, unblocked tasks under a file lock. Human is the implicit lead (no `lead` field). This is the CLI + on-disk layer only — daemon watch, plan approval, and dashboard pane are deferred.

Locked decisions (from the user before building):
1. **Lead model** — human is implicit lead; `team.toml` has only `members[]`.
2. **Task claim** — self-claim (any member); file-lock against the race.
3. **Phase A scope** — CLI foundation + spec only; no daemon/dashboard.

## What changed

- **`crates/bwoc-core/src/team.rs`** (new) — pure data model + transition rules, no IO locking:
  - `Team { id, members, created_at }` (TOML), `Task { id, title, state, deps, claimed_by, created_at, completed_at }` (JSONL), `TaskState { Pending, InProgress, Completed }`.
  - `add_task` (rejects dup id + unknown dep), `claim_task` (must be pending + all deps completed → in_progress + claimant), `complete_task` (must be in_progress + actor == claimant → completed + stamp), `parse_tasks`/`render_tasks` (JSONL), `ensure_member`.
  - 11 unit tests covering every transition + guard + JSONL round-trip.
- **`crates/bwoc-cli/src/sangha.rs`** (new) — IO + locking layer:
  - `bwoc team create/list/retire`, `bwoc task add/list/claim/complete`.
  - `TaskLock` — dependency-free advisory lock via `O_CREAT | O_EXCL` (`create_new`) holding the PID; stale lock (dead PID, `livecheck::signal_zero_alive`) is stolen; ~5s acquire budget; released on `Drop`.
  - Atomic task writes (tmp + rename). Member guard before claim/complete. `--json` on every command.
- **`crates/bwoc-cli/src/main.rs`** — `mod sangha` + `Team`/`Task` nested subcommand enums + dispatch.
- **`modules/agent-template/interconnect/sangha.md` + `.th.md`** (new, bilingual) — the spec: Saṅgha + Saṅgaha-vatthu 4 (norms) + Saṅghakamma (the lock-settled claim) mapping, data model, state machine, CLI surface, concurrency, exit codes, deferred phases.

## Decisions

- **Core is pure; locking lives in the CLI.** `bwoc-core` has no `libc` dep (it hand-rolls time, avoids chrono). The lock needs a signal-0 staleness probe, which `bwoc-cli::livecheck` already provides via `libc`. So the data model + transitions sit in core (unit-testable without IO), and the CLI wraps them with the lock. Clean separation, no new core dep.
- **Dependency-free O_EXCL lock, not `fs2`/`flock`.** Matches the project's dep-averse stance. `create_new` is atomic on POSIX; PID-in-lockfile + signal-0 staleness mirrors the daemon-pid pattern already in `start.rs`/`doctor.rs`. Steal-on-stale prevents a crashed claimer from wedging the team forever.
- **Rewrite-under-lock, not event-sourced JSONL.** Tasks are few and mutations (claim/complete) are state changes, not appends. Reading the whole file, mutating, and atomic-renaming under the lock is simpler than folding an event log — and the lock is the race protection the spec names. (inbox.jsonl stays append-only because messages genuinely only append.)
- **Auto-id `t<N>` where N = len+1.** Human-friendly and monotonic *because tasks are never deleted* (completed ones stay for audit). `--id` overrides for callers who want meaningful ids. Safe under concurrency because `add` holds the lock.
- **`--as <agent>` is required for claim/complete.** A CLI invocation has no ambient agent identity, so the claimer names itself. The member guard (`ensure_member`) rejects outsiders. Self-claim = any member may claim (Samānattatā — equal before the list), not lead-assignment.
- **Transition errors exit 2 (user error), IO/lock errors exit 1.** Blocked/wrong-state/non-member/non-claimant are the operator's fault and leave the file untouched; lock-timeout and write failures are environmental. Matches the rest of the CLI's 2-vs-1 convention.
- **Saṅgaha-vatthu 4 as norms, not gates.** Phase A doesn't enforce kindly titles or unselfish claims — same posture as messaging.md carrying Sāraṇīyadhamma 6. The spec names them so an incarnated agent internalizes them.

## Alternatives considered

- **Designated lead agent** (Claude Agent Teams style) — rejected for v1 per the user's decision. Human-implicit-lead is simpler and matches BWOC's existing "user is the coordinator" pattern. A `lead` field is an additive v2 move if the model proves limiting.
- **Lead-assigns-tasks** instead of self-claim — rejected; self-claim maximizes parallelism and maps to Samānattatā. The lock makes self-claim safe.
- **`fs2` crate for locking** — rejected (new dep). O_EXCL is enough.
- **Event-sourced task log** (append claim/complete events, fold to state) — rejected for Phase A; rewrite-under-lock is simpler and the audit trail (claimed_by, completed_at, completed-tasks-retained) is already preserved. Revisit if cross-process tailing of task events is needed (Phase B daemon watch).
- **Adding a `bwoc help sangha` topic** — deferred (Mattaññutā). The spec doc + `--help` cover Phase A; add a help topic when the surface stabilizes.

## Bugs surfaced and fixed

- **None.** Clean build + clippy + 132 workspace tests (was 121; +11 core team tests). The borrow-checker pushed one design refinement: `claim_task` snapshots `(id, state)` pairs before the mutable `find` to check dependency completion without a double mutable borrow — documented inline.

## Test summary

- **bwoc-core**: 29 tests (was 18; +11 team). All transitions, guards, dep-block/unblock, JSONL + TOML round-trips.
- **Workspace total**: 132 tests, 0 failures; clippy clean.
- **Live end-to-end** (scratch workspace, agent-pi + agent-oracle):
  - `team create squad --members agent-pi,agent-oracle` → toml written; `team list` shows counts.
  - `task add` ×2 (t1, t2 with `--deps t1`) → both pending.
  - `task claim squad t2 --as agent-pi` while t1 pending → exit 2 (blocked by dependency). ✓
  - `task claim squad t1 --as agent-pi` → ok; outsider `--as agent-ghost` → member-guard exit 2. ✓
  - `task complete squad t1 --as agent-oracle` → not-claimant exit 2; `--as agent-pi` → ok. ✓
  - dep cascade: after t1 completed, `task claim squad t2 --as agent-oracle` → ok. ✓
  - **concurrent claim race**: two `bwoc task claim` on the same fresh task in parallel → exactly one won (`in_progress` by one agent), the other got `only pending tasks can be claimed`. Lock works. ✓
  - `team retire squad --yes` → membership + task list removed; `team list` empty. ✓

## Status / deferred (Phase B/C)

- **Daemon task-watch + `task-created`/`task-completed` hooks** (Phase B) — a running `bwoc-agent --serve` reacts to task-list changes.
- **Plan approval / Pavāraṇā** (Phase C) — teammate submits plan → lead approves/rejects via an envelope-kind extension on messaging.md.
- **Team-aware dashboard task pane** (Phase B+).
- **Designated lead agent** — only if human-implicit-lead proves limiting.

## Related

- Spec: [`modules/agent-template/interconnect/sangha.md`](../modules/agent-template/interconnect/sangha.md) (+ `.th.md`)
- Core: `crates/bwoc-core/src/team.rs` · CLI: `crates/bwoc-cli/src/sangha.rs`
- Builds on: [[trust]] (Kalyāṇamitta 7), [[messaging]] (Sāraṇīyadhamma 6)
- Fleet-level counterpart: `docs/en/FLEET-GOVERNANCE.en.md` (Aparihāniya-dhamma 7)
