---
title: TASK-002 Track A — Inter-workspace routing (peer send)
date: 2026-05-23
author: agent-pi
---

# 2026-05-23 — TASK-002: Inter-Workspace Routing (Track A)

Implementation of Phase 3 Track A: peer `send` routing per
`modules/agent-template/interconnect/routing.md` (spec v1 / 2026-05-23).

Branched off `main` at `a971084` (Track B already merged).

## What changed

### `crates/bwoc-core/src/routing.rs` (new)

`Routes` type: loads and validates `.bwoc/interconnect/routes.toml`. Public surface:
- `Routes::load(workspace_root: &Path) -> Result<Routes, RoutingError>` — absent file → empty (no error).
- `Routes::resolve(recipient_id: &str) -> Option<&Path>` — returns peer workspace root.
- `Route { workspace: PathBuf, kind: RouteKind }`, `RouteKind::Agent(String) | Namespace(String)`.
- `RoutingError`, `RouteValidationError::BothKeys | NeitherKey`.
- 14 unit tests in-module covering absent/empty file, agent route, namespace route, both-keys error, neither-key error, resolution order (exact wins over namespace, longest namespace wins), no-match.

### `crates/bwoc-core/src/lib.rs`

Added `pub mod routing;`.

### `crates/bwoc-cli/src/send.rs`

Step 2 insertion: on a local registry miss, loads `Routes` and retargets
`(resolved_workspace, entry)` to the peer. Local-hit path is byte-for-byte
unchanged. Sender (`from`) resolution stays against the local registry —
intentional Trust-v2 seam (bare id, no workspace qualification).

New `SendError::Routing(#[from] RoutingError)` variant.

Added 7 integration tests covering all 6 spec cases:
1. Local hit unchanged.
2. Exact-agent peer route → envelope in peer inbox.
3. Namespace prefix route → envelope in peer inbox.
4a. Both-keys route → `Routing` error.
4b. Neither-key route → `Routing` error.
5. No match → `NotFound` unchanged.
6. Trust-gated peer send → envelope delivers with bare `from` id (trust gate is
   recipient-side; sender is `unknown_sender` in the peer registry — the safe
   v1 default).

## Decisions

- **`validate_route` fails eagerly per entry** — a single bad route in routes.toml
  aborts `send` with a `Routing` error rather than silently skipping. The spec
  says "reject a route with both/neither key"; aborting is the strictest
  interpretation. Operators can silence this only by fixing the file.
- **Stale route (peer registry doesn't contain the agent)** → `NotFound`.
  The routing table pointed somewhere that doesn't hold the agent. Safer than
  delivering to the wrong inbox.
- **`clippy::is_none_or`** — replaced `map_or(true, …)` in `resolve` to satisfy
  `-D warnings` at the Rust 1.85 MSRV.

## Alternatives considered

- Store `Routes` in `AgentsRegistry` to avoid double file load on a peer hit.
  Rejected: keeps the two concerns separate; extra load is one small TOML read,
  not hot path.
- Return `(workspace, entry)` from a dedicated free function instead of inline
  block. Inline block is equivalent and avoids an extra function boundary for
  what amounts to a control-flow join.

## Bugs surfaced and fixed

None — clean first pass. Clippy caught `map_or(true, …)` antipattern; fixed
before first commit.

## Status / deferred

- Step 4 (CHANGELOG row, ROADMAP cross-reference, `routing.th.md` bilingual
  parity) is explicitly out of scope for this task per operator instruction.
  Needs to be scheduled separately.
- Network transport (ssh/http) deferred to Trust v2.
- Loop/cycle protection deferred (v1 is single-hop only).

## Related

- Spec: `modules/agent-template/interconnect/routing.md`
- Track B: commit `a971084` on `main`
- agent-codx test branch: `agent/agent-codx/test/routing-qa`
