---
date: 2026-05-23
session: dashboard team-task visibility (Saṅgha Phase B-lite)
tags:
  - phase/3
  - type/note
  - module/bwoc-cli
---

# 2026-05-23 — Dashboard Team-Task Visibility

First slice of Saṅgha Phase B: make the shared task list visible where the human lead actually looks — the dashboard detail pane. Read-only; no daemon coupling.

## What changed

- **`crates/bwoc-cli/src/livecheck.rs`** — new `agent_team_summaries(root, agent_id) -> Vec<AgentTeamSummary>`. Scans `.bwoc/teams/*.toml`, filters to teams the agent is a member of, and for each reads `tasks.jsonl` to count `claimed_by_me` (in_progress claimed by this agent), `available` (pending + all deps completed), and `total`. Read-only, skips missing/malformed files. 1 unit test (temp workspace, mine/available/total + non-member sees nothing).
- **`crates/bwoc-cli/src/dashboard.rs`** — detail pane renders a `team <id>: N mine, M avail / T total` row per team the selected agent belongs to (after the refusal block). `avail` is green when > 0 (work waiting), dim otherwise. Shown only when the agent is on ≥1 team.

## Decisions

- **Read-only, dashboard-only.** The human is the implicit lead and the dashboard is their primary view. Surfacing team standing there (vs a daemon announce nobody reads) is the highest-value, lowest-risk Phase B slice. No per-agent-daemon ↔ workspace-team coupling decision needed.
- **Helper in `livecheck`, not `sangha`.** `livecheck` is the home for per-agent filesystem probes (inbox_count, refusal_summary) the dashboard already calls every draw. The team summary is the same shape; keeping it there avoids making `sangha`'s internals pub.
- **Per-draw read is fine.** Teams are few, task files small — same reasoning as the existing inbox-count / refusal probes that already run on every 2s draw.

## Status / deferred (rest of Phase B/C)

- **Daemon task-watch + `task-created`/`task-completed` hooks** — still deferred; needs the per-agent-daemon ↔ workspace-team membership-scan design.
- **Plan approval (Pavāraṇā)** — Phase C.
- **Interactive task pane** (claim/complete from the dashboard) — deferred; would mirror the `s`/`x` shell-out pattern but needs a task-selection sub-mode in the TUI.

## Test summary

- `cargo build -p bwoc-cli`, clippy, `cargo test -p bwoc-cli` — clean (88 cli tests incl. the new `agent_team_summaries` test; 133 workspace total).
- Dashboard render itself is not unit-testable (raw-mode loop); the helper that feeds it is.

## Related

- Builds on: [`2026-05-23_sangha-v1-phase-a.md`](2026-05-23_sangha-v1-phase-a.md)
- `crates/bwoc-cli/src/livecheck.rs` — `agent_team_summaries`
- `crates/bwoc-cli/src/dashboard.rs` — team row in detail pane
- Spec: `modules/agent-template/interconnect/sangha.md`
