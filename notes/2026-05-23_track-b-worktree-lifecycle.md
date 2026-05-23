# 2026-05-23 — Track B: worktree lifecycle (B1 git util layer + B2 task-claimed hook)

Phase 3 Track B implementation: git utility layer and the `task-claimed` Saṅgha hook.
Sequenced by agent-oracle (msg-20260523T103740Z). No code touches the Saṅgha `Task`
struct or `retire.rs` Step 3 — both are out of scope per the decision in
`notes/2026-05-23_phase3-remaining-sequencing.md`.

## What changed

- **`crates/bwoc-cli/src/git_worktree.rs`** (new) — B1 git util layer.
  Shell-out to `git worktree` and `git branch` via `std::process::Command`.
  No `git2`/`gitoxide` dependency — matches the existing process-spawn style.
  Public API: `worktree_add`, `worktree_list`, `worktree_remove`,
  `branch_list_glob`, `branch_delete`, `worktree_path` (convention helper),
  `worktree_branch` (convention helper). Internal: `parse_worktree_list`
  (pure, tested). `#[allow(dead_code)]` module-level — the live git functions
  have no caller yet; consumer is `bwoc retire` Step 3 (converge step).
  8 unit tests covering the parser and convention helpers.

- **`crates/bwoc-cli/src/sangha.rs`** (modified) — B2 `task-claimed` hook.
  - Added `use bwoc_core::manifest::Manifest` and `use bwoc_core::workspace::AgentsRegistry`.
  - In `mutate_task`: updated the comment (removed "claim has no hook in Phase B+"),
    added a `task-claimed` hook block when `verb == "claim"`, fired **before**
    `save_tasks` so a failing hook aborts the claim and leaves the task `pending`.
  - Hook env: `BWOC_TASK_EVENT=task-claimed`, `BWOC_TEAM`, `BWOC_TASK_ID`,
    `BWOC_AGENT`, `BWOC_WORKTREE_BASE` (resolved from the agent's
    `config.manifest.json::worktreeBase`; falls back to `/tmp` if manifest is
    absent or field is unset — non-fatal).
  - 1 new test: `task_claimed_hook_receives_env_and_blocks` — verifies env var
    delivery, exit-0 pass, and exit-1 block with stderr surfaced.

- **`crates/bwoc-cli/src/main.rs`** (modified) — added `mod git_worktree;`.

## Decisions

1. **Shell hook, not Rust worktree_add call.** The `task-claimed` hook fires
   `run_task_hook` (same as `task-created`/`task-completed`). The actual
   `git worktree add` is the operator's hook script responsibility, using the
   `BWOC_WORKTREE_BASE/$BWOC_AGENT/$BWOC_TASK_ID` env vars. This is
   backend-neutral (Samānattatā) and matches the existing hook convention.

2. **`BWOC_WORKTREE_BASE` resolved in Rust, passed to the hook.** The Rust
   side loads the agent's manifest to resolve `worktreeBase` before firing the
   hook. This keeps the shell script simple (no JSON parsing in bash) while
   preserving manifest authority. Fall-through to `/tmp` is non-fatal — the
   hook still runs, and an operator can override by setting the env var
   explicitly in a wrapper.

3. **`#[allow(dead_code)]` on the git_worktree module.** The live git functions
   are infrastructure for retire Step 3, not yet called from production code.
   The allow is documented with an explicit note to remove it when Step 3 lands.
   Annotating individual items with `#[expect]` would scatter the suppression;
   a module-level allow with a removal comment is cleaner for infrastructure.

4. **`parse_worktree_list` and convention helpers are `fn`/`pub fn`, not methods.**
   They are pure transformations with no state. The tests cover them directly.
   Live git functions wrap them and are tested by integration (cargo build + live
   workspace exercise).

## Alternatives considered

- **Call `worktree_add` directly from `mutate_task` in Rust instead of via hook.**
  Rejected: violates the existing hook convention; couples the Saṅgha CLI to git
  in a way that's hard to disable per-workspace. Hook = opt-in (Mattaññutā).
- **Pass worktreeBase as a hook arg instead of env var.** Args have quoting hazards;
  env vars match the existing convention for all BWOC hooks.
- **Put `git_worktree` in `bwoc-core`.** Core must stay extension-agnostic; git is
  a CLI-layer concern. Correct placement is `bwoc-cli`.

## Bugs surfaced and fixed

None. Clippy clean on first pass (after adding `#[allow(dead_code)]`).

## Status / deferred

- Track B B1+B2: **complete**. Gates: fmt ✓ clippy ✓ test ✓ (153 total, all pass) build ✓.
- Retire Step 3 (worktree cleanup + branch release) — deferred; needs Track A to converge.
- Track A (interconnect routing, `send.rs`) — out of scope for this task.

## Related

- `notes/2026-05-23_phase3-remaining-sequencing.md` — sequencing decision
- `notes/2026-05-23_sangha-task-hooks.md` — hook architecture this builds on
- `agents/agent-pi/.bwoc/inbox.jsonl` — Oracle's task brief (msg-20260523T103740Z)
- `crates/bwoc-cli/src/sangha.rs:run_task_hook` — hook runner (sangha.rs:340)
- `crates/bwoc-core/src/manifest.rs:41` — `worktree_base: Option<String>` field
