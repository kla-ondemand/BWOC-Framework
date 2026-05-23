---
date: 2026-05-23
session: Saṅgha Phase C — plan approval (Pavāraṇā)
tags:
  - phase/3
  - type/note
  - module/bwoc-core
  - module/bwoc-cli
---

# 2026-05-23 — Saṅgha Phase C: Plan Approval (Pavāraṇā)

The last Saṅgha phase. A task can require the lead's sign-off on a plan before it completes — mapping **Pavāraṇā** (the monk's end-of-Vassa invitation for the Saṅgha to point out his faults: submitting oneself for review before proceeding). With this, Saṅgha is feature-complete.

## What changed

- **`bwoc-core::team`** — `Task` gains three additive fields: `requires_plan: bool` (skip-serialized when false via an `is_false` helper, so simple tasks stay one-line), `plan: Option<String>`, `plan_approved: Option<bool>` (None = pending, Some(true)/Some(false) = approved/rejected).
  - `submit_plan(tasks, id, agent, plan)` — task must be `InProgress` + claimed by `agent`; sets `plan`, resets `plan_approved` to None (a resubmission awaits fresh review).
  - `review_plan(tasks, id, approved)` — requires a plan on file; sets the verdict. No agent (the lead is the human).
  - `complete_task` gains the gate: `requires_plan && plan_approved != Some(true)` → `PlanNotApproved { plan_state }` where plan_state ∈ not submitted / pending review / rejected.
  - New errors: `NotClaimedForPlan`, `NoPlanSubmitted`, `PlanNotApproved`. 5 new unit tests.
- **`bwoc-cli::sangha`** — `run_task_plan` (dual: submit when `--plan`/`--plan-file` given, requires `--as`; else show the plan + verdict) and `run_task_review` (approve/reject, locked, no member guard — the lead is the human). `run_task_add` gains `requires_plan`.
- **`bwoc-cli::main`** — `--requires-plan` on `task add`; new `task plan` / `task approve` / `task reject` subcommands. `--plan-file` body is read in the dispatch arm (trailing newline trimmed); `--plan`/`--plan-file` are clap-mutex.
- **Spec** `sangha.md` + `.th.md` — plan approval moved to a shipped section; abstract + deferred list updated. Saṅgha now lists A / B / B+ / hooks / C as shipped.

## Decisions

- **The gate lives in `complete_task` (core), not the CLI.** Putting the `requires_plan` check in the core transition means it holds no matter what triggers completion — the `bwoc task complete` CLI, a future daemon path, anything. A CLI-only gate would be bypassable. Yoniso manasikāra: enforce at the invariant, not at one caller.
- **Lead = human, no `--as` on approve/reject.** Consistent with the Phase A decision (human is the implicit lead). `submit_plan` is the agent's act (`--as`, claimant-guarded); `review_plan` is the lead's act (no agent). The asymmetry encodes who-does-what.
- **Resubmit clears the verdict.** A rejected plan, when resubmitted, returns to `pending` rather than staying `rejected` — the lead reviews the *new* plan fresh. Models the Pavāraṇā back-and-forth.
- **`requires_plan` is opt-in per task, default false, skip-serialized.** Existing tasks and the common case are unaffected and stay one-line in `tasks.jsonl`. The plan workflow is pure addition — `non_plan_task_completes_without_plan` test guards this.
- **`task plan` is dual-mode (submit vs show)** rather than a separate `task show`. `--plan` present → submit; absent → show. Keeps the surface to one verb for the plan, and the lead reviewing a plan uses the same command the agent used to submit it.

## Alternatives considered

- **Envelope-kind extension on messaging** (the original spec hint: plan-request/approve/reject as message kinds) — rejected. Plan approval is a property of the *task*, and gating it in the task state machine is simpler + more robust than threading approval through the inbox. Messaging stays about communication; the task list owns task state.
- **Separate `requires_plan` from a `plan_required_by_default` team setting** — rejected for v1; per-task opt-in is enough. A team-wide default can be added later if every task ends up wanting it.
- **A dedicated `task show` command** — folded into `task plan`'s show mode (Mattaññutā — one verb).
- **CLI unit test for the plan handlers** — skipped; the 5 core transition tests + the thorough live verification cover the logic, and the CLI handlers are thin (reuse `mutate_task` / a locked load-mutate-save). Adding a filesystem CLI test is marginal.

## Bugs surfaced and fixed

- **None.** Clean build + clippy + 145 workspace tests.

## Test summary

- **bwoc-core**: 34 tests (was 29; +5 plan: gate-blocks-until-approved, reject→resubmit, can't-submit-for-unclaimed/others, can't-review-unsubmitted, non-plan-completes-normally).
- **Workspace total**: 145 tests, 0 failures; clippy clean.
- **Live end-to-end** (scratch workspace): `task add --requires-plan` → `claim` → `complete` refused (plan not submitted) → `task plan --plan …` → `complete` refused (pending review) → `task plan` (show) prints the plan → `reject` → resubmit → `approve` → `complete` succeeds. Every gate fired as designed.

## Status

- **Saṅgha is feature-complete**: Phase A (CLI teams/tasks) · Phase B (daemon task-watch) · B+ (wakeup + auto-claim) · task hooks · Phase C (plan approval). The autonomous-teamwork loop and the human-in-the-loop review gate both exist.
- **Remaining (not Saṅgha-core)**: team-aware dashboard task pane; designated lead-agent (only if human-implicit-lead proves limiting). Neither blocks use.

## Related

- `crates/bwoc-core/src/team.rs` — plan fields + `submit_plan` / `review_plan` + the completion gate
- `crates/bwoc-cli/src/sangha.rs` — `run_task_plan`, `run_task_review`
- Spec: `modules/agent-template/interconnect/sangha.md` §"Plan approval — Pavāraṇā"
- Prior Saṅgha notes: [[2026-05-23_sangha-v1-phase-a]], [[2026-05-23_sangha-phase-b-daemon-task-watch]], [[2026-05-23_sangha-task-hooks]]
