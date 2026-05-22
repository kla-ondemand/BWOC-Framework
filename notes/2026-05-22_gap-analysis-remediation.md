# 2026-05-22 — Gap analysis remediation plan

Forward-looking plan. Captures gaps found in the session-end audit and orders remediation by ROI. Six items, all trim-or-sync — no new dependencies, no new doctrine docs. Mattaññutā enforced: each item earns its place or is cut.

## Gaps found (audit summary)

Three layers of drift between **declared state** and **observed state**:

1. **Self-contradictions** — `README.md` Status vs `docs/en/ROADMAP.en.md:38`; `CHANGELOG.md` "Known Issues" still lists items already fixed in the same release (git-init, ๑๔→๒๒, bilingual-reminder root-level coverage).
2. **Roadmap lags reality** — Phase 2 (`status`, `send`) and Phase 3 (`stop`, `retire`, daemon control socket) commands all shipped in `crates/bwoc-cli/src/`, but ROADMAP still marks Phase 1 v2.0 as the active phase. Yoniso Manasikāra failure — docs don't reflect current state.
3. **Spec promises vs proof** — README claims `macOS · Linux · Windows` + "signed binaries"; CI is ubuntu-only, zero release tags, zero crates.io publish. Samānattatā claim (4 backends) has no cross-backend exec evidence.

Plus: zero integration tests (74 `#[test]` are all inline unit), uneven root-level bilingual policy, four un-noted significant commits since the last `notes/` entry.

## Remediation — six items, ordered by ROI

### 1. Sync ROADMAP and README Status to reality

**Files:** `docs/en/ROADMAP.en.md`, `docs/th/ROADMAP.th.md`, `README.md:282-294`.

**Action:** Mark Phase 1 v2.0 DoD met; open Phase 2 as active; move shipped `status`/`send`/`stop`/`retire`/`dashboard`/`chat`/`doctor`/`inbox`/`ping`/`livecheck` into a "Shipped in Phase 2" table; leave only **cross-backend validation**, **CI matrix + release pipeline**, **process supervision (restart-on-crash)**, and **per-workspace memory** as outstanding Phase 2 items.

**Acceptance:** No reader can land on README and ROADMAP and get conflicting phase status.

**Principle:** Yoniso Manasikāra.

### 2. Trim CHANGELOG "Known Issues"

**Files:** `CHANGELOG.md` (Known Issues section).

**Action:** Delete three stale lines — git-init claim, PHILOSOPHY count mismatch (already fixed in Changed), bilingual-reminder root-level coverage (already shipped per the same release).

**Acceptance:** No item in "Known Issues" is contradicted by the "Changed" section of the same release.

**Principle:** Mattaññutā — trim over expand.

### 3. Backfill one consolidated note for un-noted commits

**Files:** new `notes/2026-05-22_phase-2-ṭhiti-surface.md`.

**Action:** Single note covering commits `1dea3e4`, `944aa58`, `4f20f2a`, `d2a6a5c`, `70ac3f8`, `9f4e2aa`, `6850e30`, `7434414`, `743a714`, `a14c706`, `6850e30`, `a4b91d2`, `10e2172`. Section per cluster (init, daemon spawn, doctor/livecheck, dashboard, chat) — one paragraph each, what + why + decision.

**Acceptance:** `git log --oneline` since `1dea3e4` has a paired note covering every `feat(`/`fix(` commit.

**Principle:** Sīla — HARD RULE compliance is non-optional.

### 4. CLI integration smoke test

**Files:** new `crates/bwoc-cli/tests/smoke.rs` (single file, one test fn).

**Action:** End-to-end against a `tempfile::tempdir()` workspace — `bwoc init → bwoc new alpha --backend claude → bwoc list` and assert `agents.toml` contains `alpha` + the workspace.toml is valid. No daemon, no LLM exec. Single golden-path test.

**Acceptance:** `cargo test --workspace` runs the smoke test; failure surfaces in CI.

**Principle:** Yoniso Manasikāra — verify behaviour, not just types.

**Why not more:** Adding daemon/spawn/stop tests requires test doubles for the backend CLI subprocess. Out of scope for this pass. One test that proves the most-traveled path is the right amount.

### 5. CI honesty — either matrix or scope-down claim

**Files:** `.github/workflows/ci.yml` **or** `README.md` (Tech Stack table, line 269).

**Two-option decision pending user input:**

- **(A) Add matrix:** `strategy.matrix.os: [ubuntu-latest, macos-latest, windows-latest]`. Cost: ~3x CI minutes. Benefit: proof the claim.
- **(B) Scope claim:** README Tech Stack "Platforms" cell becomes `Linux (verified) · macOS · Windows (planned)`. Cost: trivial doc edit. Benefit: honest until release pipeline lands.

Recommend **(B) for now, (A) when release pipeline is built** (Phase 2 item per ROADMAP). Less ambitious upfront, no time-to-CI penalty.

**Acceptance:** No reader can be misled about what the project currently proves on Windows/macOS.

**Principle:** Sammā-vācā (truthful claim).

### 6. Define root-level bilingual policy

**Files:** `CLAUDE.md` (Bilingual Parity HARD RULE section).

**Action:** One paragraph stating: **doctrine docs** (`VISION`, future `PHILOSOPHY-AT-ROOT` if any) require `.th.md` pair; **mechanical OSS docs** (`README`, `CHANGELOG`, `CONTRIBUTING`, `SECURITY`, `CODE_OF_CONDUCT`, `LICENSE`) are EN-only by design — translation cost > value for short-lived process docs. Crate READMEs follow code-side convention: EN-only until the framework targets a TH developer audience.

**Acceptance:** No future contributor needs to ask whether `README.th.md` should exist.

**Principle:** Mattaññutā — translation is not free; commit only what is load-bearing.

## Alternatives considered

- **Single mega-PR landing all six.** Rejected — items 1–3 are pure-docs and ship today; item 4 needs a test design pass; item 5 is a user decision; item 6 is policy and benefits from a separate explicit commit. Three smaller PRs > one churning mega-diff.
- **Auto-generate ROADMAP from commit history.** Rejected — over-engineering. ROADMAP is doctrine, not log; it should be hand-curated.
- **Skip CHANGELOG cleanup, just append a new release.** Rejected — leaves bad-faith "Known Issues" in the historical record where future readers find it.

## Status / deferred

Not started — this note is the plan. Execution order suggested:

1. Item 2 (CHANGELOG trim) — 5 min, no dependencies
2. Item 1 (ROADMAP/README sync) — 30 min, needs care with bilingual TH pair
3. Item 3 (backfill note) — 30 min, mostly retrospective writing
4. Item 6 (bilingual policy line in CLAUDE.md) — 5 min
5. Item 4 (smoke test) — 1–2 hr, needs `tempfile` dev-dep + assertion design
6. Item 5 — user decision required first (A vs B)

Deferred entirely (not in this plan):

- **Cross-backend validation harness** — a real Phase 2 deliverable, not a gap-remediation item. Belongs in its own design pass.
- **Release pipeline + crates.io publish** — same. Out of scope for the trim/sync pass.
- **`applications/` reference agents** — Phase 4 by spec. Empty directory is honest, not a gap.

## Related

- Gap analysis source: conversation transcript 2026-05-22 (session that produced this note)
- Previous note: [bwoc-new UX + framework hygiene](2026-05-22_bwoc-new-ux-and-framework-hygiene.md)
- Spec touched (planned): `docs/en/ROADMAP.en.md`, `docs/th/ROADMAP.th.md`, `README.md`, `CHANGELOG.md`, `CLAUDE.md`
- Code touched (planned): `crates/bwoc-cli/tests/smoke.rs` (new), `.github/workflows/ci.yml` (conditional)
