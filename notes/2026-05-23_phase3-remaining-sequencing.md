# 2026-05-23 — Phase 3 remaining work: sequencing decision

Sequenced the four open Phase 3 items into a build order and resolved the worktree-creation hook point. No code written this session — this is an architecture/orchestration decision captured ahead of implementation (handed to agent-pi).

## What changed

- `docs/en/ROADMAP.en.md` + `docs/th/ROADMAP.th.md` — "Remaining for Phase 3" rewritten from a flat 4-bullet list into a sequenced plan (Track A / Track B parallel → converge on `bwoc retire`; Tier 2 + Trust v2 deferred off-DoD).

## Decisions

1. **Worktree/branch full lifecycle (create + cleanup) is in Phase 3 scope.** User chose this over the leaner "defensive cleanup only" option, accepting the ~3–4× larger Track B. retire's "cleanup" only has meaning once something creates worktrees, so creation comes into scope with it.
2. **Two parallel tracks.** Track A (interconnect routing) and Track B (worktree lifecycle) are independent; both feed `bwoc retire`. Track B is the long pole — start first.
3. **B2 hook point — `task-claimed` Saṅgha hook + path convention, not `Task` struct extension.** Verified that the Saṅgha task list (`team.rs::Task`, `.bwoc/teams/<team>/tasks.jsonl`) and the per-agent `task-log.jsonl` (no Rust touches it; written by each backend) are two deliberately separate systems. retire (Rust) cannot reliably parse agent-written logs across backends. Resolution: worktree location follows the convention `<worktreeBase>/<agentId>/<taskId>`, so cleanup is filesystem-deterministic (`git worktree list` + prefix filter) without reading any log. Keeps the two systems separate (Anattā), avoids bloating `Task` (Mattaññutā), and stays backend-neutral since the creating hook is shell (Samānattatā).
4. **git via shell-out, no `git2`/`gitoxide` dependency** — matches the existing process-spawn style in the CLI.
5. **Tier 2 memory and Trust v2 deferred** — neither is on the Phase 3 DoD ("life ends cleanly + coordinate without a central authority"). Trust v2 stays gated on v1 telemetry; its cross-workspace part also depends on Track A.

## Alternatives considered

- **Defensive-cleanup-only retire** (don't build worktree creation): leaner, met the DoD literally — rejected by user in favour of full lifecycle.
- **Extend `Task` struct with `worktree_path`/`branch_name` + `bwoc task claim` shells out git directly**: single source of truth, but couples the Saṅgha coordination layer to git and mismatches scope (tasks are team-scoped; `worktreeBase` is per-agent). Rejected for the convention-based approach.

## Status / deferred

- Sequence locked in ROADMAP (EN+TH). Implementation not started.
- Handed to agent-pi (core-systems) via `bwoc send` to begin Track B (long pole). Oracle does not write crate code.
- Pre-existing roadmap nit left as-is but flagged in ROADMAP text: "Tier 2 interface" appears under both Phase 2-remaining and Phase 3 — clarified as two distinct pieces (interface vs reference impl) rather than a duplicate.

## Related (links)

- `docs/en/ROADMAP.en.md` §"Remaining for Phase 3 — sequenced"
- `notes/2026-05-23_sangha-task-hooks.md` — the `task-created`/`task-completed` hook architecture this builds on (`task-claimed` is the "one-line add" noted there)
- `crates/bwoc-core/src/team.rs` — `Task` struct (no worktree fields, intentionally)
- `crates/bwoc-cli/src/retire.rs` — current retire (registry + filesystem only)
