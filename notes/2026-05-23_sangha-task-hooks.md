---
date: 2026-05-23
session: Saṅgha task hooks — task-created / task-completed (blocking)
tags:
  - phase/3
  - type/note
  - module/bwoc-cli
---

# 2026-05-23 — Saṅgha Task Hooks

Workspace-level shell hooks on the task lifecycle, mirroring Claude Agent Teams' `TaskCreated` / `TaskCompleted`. The defining behavior: a non-zero exit **blocks** the operation, so hooks act as quality gates.

## What changed

- **`crates/bwoc-cli/src/sangha.rs`** — new `run_task_hook(workspace, event, env)`:
  - Looks for `<ws>/.bwoc/hooks/<event>` (`task-created` / `task-completed`).
  - Missing file → `Ok(())` (no-op). On Unix, present-but-not-executable → also `Ok(())` (treated as disabled).
  - Runs the hook with the task context as env vars, cwd = workspace. Exit 0 → `Ok`; non-zero → `Err("blocked by <event> hook (exit N): <first stderr line>")`.
  - Wired into `run_task_add` (fires `task-created` after the in-memory add, **before** `save_tasks` — so a block leaves the file untouched) and `mutate_task` (fires `task-completed` for the `complete` verb only, after the transition succeeds, before save).
  - `mutate_task` gained a `task_id` param so the hook can pass `BWOC_TASK_ID`.
  - 1 unit test: missing = ok, non-executable = ok, exit 0 = ok, exit 2 = err (stderr surfaced).

## Decisions

- **Block-before-persist.** The hook runs after the mutation is computed in memory but before `save_tasks`. A non-zero exit returns exit 2 and never writes — matching Claude's "TaskCreated can prevent creation". The file is the commit point; gating before it is clean and atomic.
- **CLI is the hook site, not the daemon.** Hooks fire where the mutation happens (`bwoc task add` / `complete`), so the block semantics are meaningful. The daemon's task-watch only *observes*; it has nothing to block.
- **Env vars, not args.** Matches the auto-version / Claude-Code hook convention (context via environment), keeps the hook signature stable as fields grow, and avoids quoting issues with task titles.
- **Non-executable = disabled, not error.** Lets an operator stage a hook script without arming it (chmod -x to pause). A missing hook is likewise silent — hooks are strictly opt-in (Mattaññutā: zero cost when unused).
- **`task-completed` only for `complete`, not `claim`.** Claim is an intra-team coordination act, not a deliverable checkpoint; gating it would surprise. If a `task-claimed` hook is ever wanted it's a one-line add, deferred until asked.

## Alternatives considered

- **Fire hooks from the daemon task-watch** — rejected; the watch can't block a CLI mutation it only observes after the fact. Quality gates belong at the mutation point.
- **Run hook before computing the mutation** — for `task-created` it's equivalent (we have id+title pre-add). For `task-completed`, running after the in-memory transition means the hook only fires for a *valid* completion (right state, right claimant) — a hook never sees a completion that would have failed anyway. Better signal.
- **Pass a JSON payload on stdin** (like Claude Code hooks) — heavier; env vars suffice for the handful of fields. Revisit if hooks need structured/nested data.

## Bugs surfaced and fixed

- **None.** Clean build + clippy + tests.

## Test summary

- `cargo test -p bwoc-cli` — 91 tests (was 90; +1 `task_hook_missing_is_noop_blocking_is_err`). Clippy clean.
- **Live-verified** in a scratch workspace:
  - `task-created` pass → hook logged the env (`task-created squad/t1 first`), task added.
  - `task-created` exit 2 → `blocked by task-created hook (exit 2): no new tasks allowed right now`, exit 2, blocked task absent from `task list`.
  - `task-completed` exit 2 → blocked, task stayed `in_progress`.
  - `task-completed` pass → completed, hook logged `completed: t1 by agent-pi`.

## Status / deferred

- **Auto-claim** (daemon claims itself, behind opt-in) — the last deferred Saṅgha B+ item; gated separately for the autonomous-mutation risk.
- **`task-claimed` hook** — not built; one-line add if wanted.
- **Plan approval (Pavāraṇā)** — Phase C.

## Related

- `crates/bwoc-cli/src/sangha.rs` — `run_task_hook`, wired into add + complete
- Spec: `modules/agent-template/interconnect/sangha.md` §"Task hooks (shipped)"
- Builds on Phase A/B: [[2026-05-23_sangha-v1-phase-a]], [[2026-05-23_sangha-phase-b-daemon-task-watch]]
