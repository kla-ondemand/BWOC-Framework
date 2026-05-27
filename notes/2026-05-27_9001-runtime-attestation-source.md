---
title: 9001 Runtime — Attestation Source Mechanism
date: 2026-05-27
agent: agent-lisa
story: BWOC-28
related:
  - "[[2026-05-27_iso-runtime-evidence-model]]"
  - "[[../docs/en/PLUGINS.en|PLUGINS.en.md]]"
tags:
  - epic-3
  - audit-iso-9001
  - design-note
---

# 9001 Runtime — Attestation Source Mechanism (BWOC-28)

Captures the implementation decisions for replacing `audit-iso-9001/audit.sh` stub with a runnable that emits the `attestation` evidence kind per the [BWOC-26 evidence model](2026-05-27_iso-runtime-evidence-model.md) and the [BWOC-27 schema bump](../docs/en/PLUGINS.en.md#audit-findings-schema). Two design questions are settled here: where the runtime reads attestation data from, and what it does when an operator-provided attestation is missing. A third, unplanned, discovery is also captured: the dispatcher (`crates/bwoc-cli/src/audit.rs`) needs a companion schema extension for any of this to land end-to-end.

## Question 1 — Where does the runtime read attestation data from?

Three options were on the table:

| Option | Where | Pro | Con |
|---|---|---|---|
| **A. workspace.toml block** | `.bwoc/workspace.toml [[plugins.audit-iso-9001.attestations]]` | Co-located with `[plugins.audit-iso-9001].enabled`. No new convention. `bwoc check` already walks this file. Commit-able / diff-able. | Plugin-specific schema lives inside the universal workspace config. |
| **B. Separate `attestations/9001.toml`** | New file under workspace root or `attestations/` subdir | Single-purpose, easy to scope per-plugin. | New file convention; another path operator must remember; ergonomic only if multiple stds adopt it. |
| **C. `BWOC_AUDIT_ATTESTATIONS` env path** | Env points to a file the runtime reads | Most flexible (CI secret, mounted vault). | Path-based contract isn't commit-able by default; new env contract; hidden from `bwoc check`. |

### Decision — **Option A: workspace.toml block**

Rationale:

- **Smallest blast radius.** Uses an existing file. No new path convention. No new env contract. Operator already touches `workspace.toml` to enable the plugin (`[plugins.audit-iso-9001].enabled = true`); declaring attestations right next to that toggle is natural.
- **Diff-able audit trail.** `workspace.toml` is committed into the workspace; every attestation change shows up in `git log`. That is itself a meta-attestation that the QMS has someone signing things off over time.
- **Discoverable.** `bwoc check`, `bwoc plugin list`, and the future `bwoc audit report --expired` (BWOC-29 territory) all already walk `workspace.toml`. No new discovery code.
- **Per-plugin scoping comes free.** Other EPIC-3 runtimes (`audit-iso-27001`, `audit-iso-20000-1`) can adopt the same `[[plugins.<name>.attestations]]` shape under their own namespace without collision.

Rejected — **B** adds a file the operator must learn and risks drift between `workspace.toml` enable + separate-file declare. **C** moves audit evidence out of the workspace into ambient state — the opposite of what `bwoc audit run`'s reproducibility guarantee asks for (PLUGINS.en.md §Schema rules: "evidence MUST be reproducible").

### Schema in workspace.toml

```toml
[plugins.audit-iso-9001]
enabled = true

[[plugins.audit-iso-9001.attestations]]
criterion_id  = "9001-management-review"
statement     = "Management review held 2026-04-15 covering Q1 QMS performance, customer feedback, internal audit results, improvement opportunities."
signer        = "Quality Manager: Tonkla K."
signed_at     = "2026-04-15"
valid_through = "2027-04-15"   # optional
```

- `[[plugins.audit-iso-9001.attestations]]` is an **array of tables** — one entry per criterion the operator has attested to.
- `criterion_id` MUST match an entry in `criteria.toml` (otherwise the runtime emits a framework-error finding — see §Validation below).
- `statement`, `signer`, `signed_at` are required (mirroring the BWOC-27 schema's required sub-fields for `evidence.kind = "attestation"`).
- `valid_through` is optional (mirroring the BWOC-27 optional time-bounded field).

Multiple attestations for the same `criterion_id` are not currently meaningful — first match wins; future tooling may flag duplicates as a workspace error.

## Question 2 — What happens when a criterion has no operator-provided attestation?

The criterion is declared in `criteria.toml` (stable id, severity, clause), but the operator hasn't supplied an attestation block in `workspace.toml`. Three choices:

| Status | Semantics | Verdict |
|---|---|---|
| `not_implemented` | "Runtime doesn't exist yet." | **Wrong** — the runtime DOES exist now (v0.2.0). Using this would falsify the maturity declaration. |
| `not_applicable` | "This criterion doesn't apply to this workspace's profile." | **Wrong** — the criterion applies to every QMS-conformant workspace; the workspace just hasn't supplied evidence. |
| `fail` | "We checked. There's no attestation. Here's the remedy." | **Correct** — actionable, honest, drives the operator toward providing the attestation. |

### Decision — emit `status = "fail"`

Remedy text:

```
Provide a signed attestation in .bwoc/workspace.toml under
[[plugins.audit-iso-9001.attestations]] with criterion_id="<id>",
statement, signer, and signed_at (ISO 8601 date) to satisfy this criterion.
```

`evidence.kind = "attestation"` with `value` = empty string is **not** the move — schema rule (PLUGINS.en.md line 78) forbids empty `evidence.value` unless `kind = "none"`. For a missing attestation finding, the runtime uses `evidence.kind = "none"` and `evidence.value = ""`, which is permitted with `status = "fail"` only if the dispatcher rule is relaxed.

Re-checking PLUGINS.en.md §Schema rules: `kind = "none"` MUST NOT appear with `status = pass` or `fail` — so `none + fail` is also forbidden. The right pattern is `evidence.kind = "attestation"` with `evidence.value` carrying a synthetic "no attestation provided" sentinel — but that mixes signal with evidence.

**Cleanest legal shape:** `evidence.kind = "file"`, `evidence.value = ".bwoc/workspace.toml"` (the file the operator should edit), `status = "fail"`. The evidence is the file the operator should look at; the remedy says what to add to it. This satisfies the schema's reproducibility rule (operator can `cat .bwoc/workspace.toml` and verify what's there / what's missing), and emits a single coherent finding.

```json
{
  "criterion_id": "9001-management-review",
  "severity":     "high",
  "status":       "fail",
  "evidence":     { "kind": "file", "value": ".bwoc/workspace.toml" },
  "remedy":       "Provide a signed attestation in .bwoc/workspace.toml under [[plugins.audit-iso-9001.attestations]] with criterion_id=\"9001-management-review\", statement, signer, and signed_at (ISO 8601 date) to satisfy this criterion."
}
```

Findings WITH an attestation get the full `attestation` shape with all required sub-fields:

```json
{
  "criterion_id": "9001-management-review",
  "severity":     "high",
  "status":       "pass",
  "evidence": {
    "kind":          "attestation",
    "value":         "Management review held 2026-04-15 covering Q1 QMS performance, customer feedback, internal audit results, improvement opportunities.",
    "signer":        "Quality Manager: Tonkla K.",
    "signed_at":     "2026-04-15",
    "valid_through": "2027-04-15"
  }
}
```

## Question 3 (discovered) — Dispatcher schema validator gap

`crates/bwoc-cli/src/audit.rs:227` declares the closed-enum:

```rust
const EVIDENCE_KINDS: &[&str] = &["file", "content", "command", "none"];
```

`Finding` (line 232) carries only `evidence_kind` + `evidence_value` + `remedy`. `parse_finding` rejects any `kind` outside the v1 enum. `to_json` (line 244) re-serializes only `kind` + `value`, dropping any extra sub-fields the plugin produces.

**Consequence:** without an `audit.rs` companion change, this runtime's `attestation` findings are rejected by the dispatcher with a "kind not in closed enum" framework error (exit 255). The smoke gate as specified in the operator's brief cannot pass.

BWOC-27 (the doc-side schema bump) explicitly scoped its diff to `docs/en/PLUGINS.en.md` and `docs/th/PLUGINS.th.md`. No Rust dispatcher work landed with it. BWOC-29 is "`bwoc check` extension for new evidence kinds" — the static contract validator in `bwoc check`, not the runtime dispatcher in `bwoc audit run`.

### Decision — extend dispatcher in this BWOC-28 change

This crosses agent-lisa's normal lane (Rust core/daemon) into agent-jennie's lane (`bwoc-cli`). The justification:

1. **Atomicity.** Plugin + dispatcher land together so the smoke gate truly passes on a single commit. Splitting into two PRs creates a window where one half is half-broken.
2. **Same schema, same author.** The plugin emits the shape; the dispatcher validates it. Coupling them in one change set keeps the contract honest at the diff boundary.
3. **Minimum scope.** The dispatcher edit is surgical — enum extension + optional sub-field pass-through + per-kind required-field validation + two new tests. ~80 LOC, no behavior change to existing v1 findings.
4. **Cross-lane notification.** `bwoc send agent-jennie …` after landing surfaces the cross-lane reach so jennie isn't surprised next time she edits `audit.rs`.

Changes to `audit.rs`:

- `EVIDENCE_KINDS` grows to `&["file", "content", "command", "attestation", "sample", "none"]`.
- `Finding` carries optional sub-field map (or explicit fields — see implementation): `signer`, `signed_at`, `sampled_count`, `sampled_of`, `window`, `as_of`, `valid_through`.
- `parse_finding` enforces per-kind required sub-fields (kind=`attestation` ⇒ `signer` + `signed_at` required; kind=`sample` ⇒ `sampled_count` + `sampled_of` required).
- `to_json` passes through any present optional sub-field.
- New tests: happy-path attestation + sample, rejection on missing `signer`/`signed_at`/`sampled_count`/`sampled_of`.

`bwoc check`-style criterion-level validation (BWOC-29's domain: enforcing `expected_evidence_kind` in `criteria.toml`) is **not** in scope for this commit. This is purely the runtime-side schema enforcement at finding-emission time.

## How the runtime works (target shape)

```bash
bwoc audit run --plugin audit-iso-9001 --json
```

1. Dispatcher resolves workspace, spawns `audit.sh` with `BWOC_WORKSPACE`, `BWOC_PLUGIN_DIR`, `BWOC_AUDIT_OPERATION`.
2. `audit.sh` reads `criteria.toml` for the declared criteria (8 ids — stable, unchanged from v0.1.0).
3. `audit.sh` reads `.bwoc/workspace.toml`, walks `[[plugins.audit-iso-9001.attestations]]`, builds a map `criterion_id → {statement, signer, signed_at, valid_through?}`.
4. For each criterion in declaration order:
   - If attestation present → emit `status="pass"`, `evidence.kind="attestation"` with all required sub-fields.
   - If attestation absent → emit `status="fail"`, `evidence.kind="file"` pointing at `workspace.toml`, remedy explains the fix.
5. Dispatcher validates findings against the extended schema, emits canonical envelope.

Exit codes follow BWOC-12: zero if all attestations present, N = fail count otherwise.

## Stability

All 8 declared `criterion_id`s in `criteria.toml` carry over unchanged from v0.1.0 — the runtime only changes how each criterion's finding is computed, not which criteria exist. Per PLUGINS.en.md §Stability, criterion_id renames would be a major version bump; this is a minor bump (`0.1.0 → 0.2.0`) because the contract grows (new evidence kind on findings) but does not rename anything.

## Open follow-ups (out of scope for BWOC-28)

- **BWOC-29** — `bwoc check` extension for new evidence kinds; should also validate `[[plugins.audit-iso-9001.attestations]]` entries in `workspace.toml` (`criterion_id` matches `criteria.toml`, ISO 8601 date shape, no duplicates).
- **`valid_through` expiry semantics.** Per the BWOC-26 design note open question #3, the dispatcher stamps but doesn't degrade severity. A future `bwoc audit report --expired` flag would surface expired attestations. Not built here.
- **20000-1 / 27001 runtimes (S5, EPIC-3 continued).** They reuse the same `[[plugins.<name>.attestations]]` shape; the `sample` kind needs its own data source which is not workspace.toml-shaped (it's rate-over-window). That's its own design note when 20000-1 runtime starts.

## Related

- [`notes/2026-05-27_iso-runtime-evidence-model.md`](2026-05-27_iso-runtime-evidence-model.md) — BWOC-26 design note (the evidence model this implements).
- [`docs/en/PLUGINS.en.md`](../docs/en/PLUGINS.en.md) §Audit Findings Schema — BWOC-27 schema bump.
- [`modules/plugins/audit-iso-9001/SPEC.md`](../modules/plugins/audit-iso-9001/SPEC.md) — plugin spec (updated as part of BWOC-28).
- [`crates/bwoc-cli/src/audit.rs`](../crates/bwoc-cli/src/audit.rs) — dispatcher (extended as part of BWOC-28; cross-lane reach into jennie's surface).
- `.scrum/backlog.json` BWOC-28 — this story.
