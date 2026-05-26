---
title: BWOC-12 audit-run exit code — reconcile note
date: 2026-05-27
owner: agent-jennie
status: proposal
relates_to: BWOC-12, S2-retro #2, S3 polish
---

# `bwoc audit run` exit-code reconcile

## Context

Sprint 2 retro item #2 (escalated as a Sprint 3 polish item) asked us to
reconcile the `bwoc audit run` process exit code with the convention
documented in `BWOC-12`. The handover brief I picked up reported the
behavior as "currently always exits 0".

When I read the code on `feat/sprint-3-epic-2-closure` (commit `da62a83`,
landed 2026-05-26 20:16 ICT) the actual behavior is already richer than
the handover described — see "Findings on current behavior" below. The
true gap is **not the implementation**, it is the **spec**: the
convention lives only inside the `audit.rs` module-doc comment, which
itself admits it is "proposed; PLUGINS.en.md does not pin it down"
(`crates/bwoc-cli/src/audit.rs:32`).

## Findings on current behavior

| Exit code | Trigger | Source |
|---|---|---|
| `0` | No `fail` findings across selected plugins (or no plugins selected). | `audit.rs:857` (`fail_count.min(254) as i32`), `audit.rs:782` (empty selection) |
| `1..=254` | Count of `fail` findings across all selected plugins, clamped to 254. | `audit.rs:857` |
| `255` | Framework error: discovery failure, manifest parse error, plugin spawn failure, non-JSON stdout, schema-violating finding, JSON serialization error. | `audit.rs:701` (`EXIT_FRAMEWORK_ERROR`), threaded through `:716 :724 :770 :840 :854` |
| `2` | Operator/usage error: workspace not found, or `--plugin <name>` does not match an audit-kind plugin. | `audit.rs:708 :743 :776` |

The `i32` returned from `audit::run()` is propagated to the process exit
code at `main.rs:2217` via `ExitCode::from(u8::try_from(code).unwrap_or(1))`.

So the impl already matches the convention jisoo asked for (0 / 1..=254 /
255). The `2`-for-usage-errors arm is a real fourth code that exists in
code but is undocumented.

## Decision

**Update spec + add unit tests. Do _not_ rewrite the impl.**

One-line rationale: the binary already exits non-zero on findings and
255 on framework errors; the actual debt is normative — PLUGINS.en.md
never pins the convention, so a CI consumer reading the spec cannot
rely on the code's behavior being stable.

Operator/jisoo confirmation gate: the operator (พี่ต้นกล้า) reviews this
note before I touch the spec or the tests. If she prefers a different
split — e.g. rename "framework error" to a different code, or drop the
exit-2 arm and fold usage errors into 255 — I revise this note and re-ask
before implementing.

## Diff plan (gated by sign-off)

### 1. `docs/en/PLUGINS.en.md` — add a normative §"Exit codes" subsection under "Audit kind"

New subsection placed after the existing §"Findings Schema". Table form:

```markdown
### Exit codes (`bwoc audit run`)

The framework process exit code is normative and stable across releases.
Operators and CI consumers can branch on it without parsing stdout.

| Code | Meaning |
|---|---|
| `0` | No `fail` findings across the selected plugins (or no plugins were enabled / selected). |
| `1..=254` | Count of `fail` findings across all selected plugins, clamped to `254`. |
| `255` | Framework or plugin runtime error — discovery failed, manifest parsed badly, plugin failed to spawn or returned non-JSON, or a finding violated the BWOC-11 schema. The `--json` envelope's `summary.framework_error` is `true` in this case. |
| `2` | Operator/usage error — no workspace found (no `--workspace`, no `BWOC_WORKSPACE`, no ancestor `.bwoc/workspace.toml`), or `--plugin <name>` did not resolve to an installed audit-kind plugin. |

Operators who only care about pass/fail can branch on `$? -eq 0`. CI
that wants to surface the count without parsing JSON can use `$?`
directly (clamped to 254 if a single run produces ≥ 255 fails — rare;
prefer `--json` for exact counts).
```

### 2. `docs/th/PLUGINS.th.md` — Thai parity for the same subsection (BWOC-18 bilingual rule)

### 3. `crates/bwoc-cli/src/audit.rs` — drop "proposed" hedge from the module doc-comment and cite the spec

Replace lines 32-43 of the module doc-comment from:

```rust
//! ## Exit-code convention (proposed; PLUGINS.en.md does not pin it down)
```

with:

```rust
//! ## Exit-code convention (normative — PLUGINS.en.md §"Exit codes (`bwoc audit run`)")
```

… and rewrite the bullets so they match the spec table verbatim
(including the exit-2 usage-error arm, which the current module doc
omits).

### 4. `crates/bwoc-cli/src/audit.rs` — add unit tests covering the exit-code path

The existing test module covers finding parsing + summary counting, but
not the exit-code function itself. Add the four cases jisoo asked for:

- `exit_code_all_pass_returns_zero` — `Summary { fail_count: 0, .. }`,
  no `framework_error` → exit code computation returns `0`.
- `exit_code_one_fail_returns_one` — `fail_count: 1` → `1`.
- `exit_code_two_fail_returns_two` — `fail_count: 2` → `2`.
- `exit_code_framework_error_returns_255` — `framework_error: true`
  short-circuits to `255` regardless of `fail_count`.
- Bonus: `exit_code_clamps_at_254` — `fail_count: 300` → `254`.

These will exercise a small extracted helper (`compute_exit_code(summary,
framework_error) -> i32`) so the tests do not need to spawn child
processes. The existing `pub fn run(...) -> i32` body simply delegates to
the helper at its return site.

## Out of scope

- Does **not** touch daemon/runtime internals (per agent-jennie scope —
  `Does NOT modify daemon/runtime internals`).
- Does **not** rewrite the dispatch loop, manifest parsing, or finding
  schema validation — those are already correct.
- Does **not** retire `agent/agent-jennie/feat/audit-exit-codes` (the
  older S2 branch). Leaving cleanup to the operator at S3 close.

## Gates before declaring done (hard)

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo build --workspace`
- `bwoc check --all` (workspace neutrality + manifest audit)

## After landing

- Single commit on `agent/agent-jennie/feat/BWOC-12-exitcode`.
- One-line notification to `agent-jisoo` via `bwoc send` with commit hash.
- Operator reviews note + commit before merging.
