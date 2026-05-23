---
date: 2026-05-23
session: Saṅgha Phase B — daemon task-watch (announce-only)
tags:
  - phase/3
  - type/note
  - module/bwoc-agent
---

# 2026-05-23 — Saṅgha Phase B: Daemon Task-Watch

Makes teams *live*: a running `bwoc-agent --serve` watches the shared task lists of every team its agent belongs to and announces newly-claimable tasks to stderr — the same shape as the inbox watch. Announce-only; auto-claim + wakeup are a deliberate follow-up.

## What changed

- **`crates/bwoc-agent/src/task_watch.rs`** (new, `#[cfg(unix)]`) — `TaskWatch`:
  - `build(agent_id, workspace_root)` — snapshots currently-claimable tasks into `seen` WITHOUT announcing (no startup replay, mirroring the inbox cursor starting at EOF). Inert when `workspace_root` is `None`.
  - `poll()` — rescans member teams, announces tasks newly in the claimable set (`bwoc-agent: task available ← <team>/<task>: <title>`), and drops from `seen` any task that left the claimable set so a re-open re-announces.
  - `scan_claimable()` — walks `.bwoc/teams/*.toml`, keeps teams the agent is a member of, parses each `tasks.jsonl` (via the pure `bwoc_core::team` functions), collects `pending` tasks with all deps `completed`.
  - 4 unit tests: new-after-build announce, non-member team ignored, blocked task not claimable, inert without workspace.
- **`crates/bwoc-agent/src/main.rs`** — `mod task_watch` (unix); builds the watcher after the trust context (reusing `trust_ctx.workspace_root` + `manifest.agent_id`), polls it in the serve loop's idle branch on a 2s cadence (vs the 100ms inbox tick — tasks change rarely).
- **`interconnect/sangha.md` + `.th.md`** — moved daemon task-watch from "Deferred" to a "Phase B (shipped, announce-only)" section; split the remaining deferred items into task-hooks, auto-claim+wakeup, and plan-approval.

## Decisions

- **Announce-only, not auto-claim.** Mattaññutā — prove the watch + announce before wiring the daemon to claim work and wake the agent. Auto-claim couples the daemon to mutation (lock acquisition, claim-as-self) and the tmux wakeup; that's a bigger surface that deserves its own iter once the announce is observed useful.
- **Reuse `trust_ctx.workspace_root`.** The trust context already walks ancestors for `.bwoc/workspace.toml` once at startup. The task watch needs the same root, so it borrows it instead of walking again — one ancestor walk per daemon lifetime.
- **2s poll cadence, not 100ms.** The inbox ticks at 100ms because messages are latency-sensitive. Tasks change on human/agent action (rare); re-reading every team file 10×/second would be wasteful. 2s is live enough and cheap.
- **Snapshot-at-startup (no replay).** Same posture as the inbox cursor: a daemon starting up shouldn't dump every already-open task as "new". `build()` pre-seeds `seen` with the current claimable set so only tasks appearing *after* startup announce.
- **`scan_claimable` in bwoc-agent, not bwoc-core.** `bwoc-core::team` is deliberately IO-free (pure model + transitions). The filesystem walk lives in the agent (like `bwoc-cli::livecheck::agent_team_summaries` lives in the CLI). Both use core's pure `parse_tasks` / `Team::from_toml` / `has_member`. The scan logic is duplicated across cli + agent — acceptable per the codebase's per-crate-helper convention; promote to a shared crate only if a third consumer appears.
- **Drop-from-seen on leave.** When a task is claimed/completed it leaves the claimable set; `poll()` removes it from `seen`. If it ever returns to `pending` (e.g., a future "unclaim") it re-announces. Keeps `seen` bounded to the *currently* open set, not unbounded history.

## Alternatives considered

- **inotify / fs events** instead of polling — rejected; adds a dep + platform complexity. The daemon already polls the inbox in the same loop; piggybacking a 2s task poll is simpler and matches the existing design.
- **Auto-claim in Phase B** — rejected (see above; deferred to B+).
- **Put the scan in `bwoc-core::team`** — rejected; would break the module's IO-free purity that Phase A established.
- **Persist a task cursor** like the inbox — unnecessary. The claimable set is recomputed from the (small) team files each poll; there's no byte-offset to track, and snapshot-at-startup already handles "don't replay".

## Bugs surfaced and fixed

- **Mid-edit placement bug (caught before build).** First inserted the `TaskWatch::build` call right after `let start = Instant::now()`, but `trust_ctx` (whose `workspace_root` it borrows) is constructed later in the function. Moved the build to just before the accept loop, after the trust posture block. Compile would have failed; caught by reading the surrounding scope.

## Test summary

- **bwoc-agent**: 19 tests (was 15; +4 task_watch). Workspace total **139** (was 135). Clippy clean.
- **Live end-to-end**: scratch workspace, `agent-pi` on team `squad` with one pre-existing task. Started `bwoc-agent --serve` in the agent dir → log showed `watching Saṅgha tasks for member 'agent-pi'` and did NOT announce the pre-existing task. Added a new task via `bwoc task add` while the daemon ran → within the poll window the log showed `bwoc-agent: task available ← squad/t3: fresh task after daemon start`. Daemon killed + scratch cleaned; no orphans.

## Addendum — opt-in tmux wakeup (same session)

Added the wakeup half right after the announce-only landing, gated opt-in so the default stays announce-only.

- **`BWOC_TASK_WAKEUP=1`** — on a newly-claimable task, `TaskWatch::poll` also calls `wake_session(agent_id, team, task, title)`: `agent-<x>` → tmux session `<x>` → two-step send-keys of `[bwoc task <team>/<id>] <title>` (mirrors `send.rs::notify_tmux`; reimplemented in the agent since cli's is a private fn in a bin crate). A live Claude session running the agent sees the marker and can `bwoc task claim` it.
- **Agent stays in control — no auto-claim.** The daemon pings but never mutates the task list. This sidesteps the stranding risk (claim a task, then fail to wake → orphaned `in_progress`). Auto-claim (daemon claims itself) is the riskier follow-up, gated separately, deferred.
- **Startup log** names the posture: `watching Saṅgha tasks for member 'agent-pi' (BWOC_TASK_WAKEUP=1 — will ping tmux session on new tasks)`.
- **Live-verified**: tmux session `pi` created, daemon started with `BWOC_TASK_WAKEUP=1`, `bwoc task add` → the pane received `[bwoc task squad/t1] wake me up` within the poll window. (Note: zsh aliases `tmux` to the oh-my-zsh plugin wrapper; the test used the real `/opt/homebrew/bin/tmux` binary directly.)

## Status / deferred (Phase B+ / C)

- **Task hooks** — `task-created` / `task-completed` workspace-level shell hooks (mirrors Claude Agent Teams' TaskCreated/TaskCompleted).
- **Auto-claim** — daemon claims the next available task itself (behind its own opt-in). Higher risk than wakeup (autonomous mutation + stranding); deferred until the wakeup→agent-claims loop is proven.
- **Plan approval (Pavāraṇā)** — Phase C; envelope-kind extension on messaging.

## Related

- `crates/bwoc-agent/src/task_watch.rs` · serve loop in `crates/bwoc-agent/src/main.rs`
- Builds on: [`2026-05-23_sangha-v1-phase-a.md`](2026-05-23_sangha-v1-phase-a.md), [`2026-05-23_dashboard-team-visibility.md`](2026-05-23_dashboard-team-visibility.md)
- Spec: `modules/agent-template/interconnect/sangha.md` §Phase B
