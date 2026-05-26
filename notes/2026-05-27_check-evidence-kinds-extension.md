# 2026-05-27 — `bwoc check` evidence-kinds extension (BWOC-29)

Design note pinning the static-check shape for the new evidence kinds added by [BWOC-27](../docs/en/PLUGINS.en.md#audit-findings-schema) (`attestation`, `sample`). BWOC-17 already validates `criteria.toml` well-formedness (kebab-case `criterion_id`, closed `severity` enum, non-empty `[criterion]` table). BWOC-29 layers on top: per-criterion declaration of expected evidence shape, statically enforced before any audit run.

The decision is **additive** — every existing `criteria.toml` (audit-iso-29110 + the three stubs + audit-iso-9001 runtime from BWOC-28) keeps passing unchanged.

## Problem

After BWOC-27, the dispatcher validates each finding's shape at `invoke` time: an attestation finding without `signer`/`signed_at` is a framework error (exit 255). But the dispatcher only knows the per-finding contract — not which criterion intends to emit which kind. Two failure modes that static-check should catch *before* the runtime runs:

1. **Mistyped enum** — `expected_evidence_kind = "attestion"` (typo) silently devolves to "no kind declared", and the runtime is free to emit anything. The discrepancy only surfaces under audit run, not in `bwoc check`.
2. **Contract drift** — a criterion declares `expected_evidence_kind = "attestation"` but the criteria author forgot the per-kind contract document. Without a static field listing required sub-fields, an operator reading `criteria.toml` cannot verify the runtime will satisfy the spec without running it.

The BWOC-26 design note already proposed `expected_evidence_kind` as an optional field. This note locks the **shape** for the required-sub-fields declaration.

## Proposed shape

### 1. `expected_evidence_kind` (optional, top-level)

```toml
[criterion.9001-management-review]
severity = "high"
expected_evidence_kind = "attestation"
```

- **Type:** string.
- **Closed enum:** `file` | `content` | `command` | `attestation` | `sample` | `none`. Mirrors the runtime enum in PLUGINS.en.md §"Evidence kinds".
- **Optional.** Absent → `bwoc check` does not enforce a kind for this criterion. The plugin chooses at invoke time.
- **Static check (BWOC-29):** if present, must be one of the six valid values. Unknown values are a violation with a precise remedy (the closed enum).

### 2. Per-kind required-fields subtable

When `expected_evidence_kind` declares a kind that carries spec-mandated sub-fields (`attestation`, `sample`), the criterion **MUST** declare a matching `[criterion.<id>.<kind>]` subtable with a `required` array:

```toml
[criterion.9001-management-review]
severity = "high"
expected_evidence_kind = "attestation"

[criterion.9001-management-review.attestation]
required = ["signer", "signed_at"]

[criterion.20000-1-incident-management]
severity = "high"
expected_evidence_kind = "sample"

[criterion.20000-1-incident-management.sample]
required = ["sampled_count", "sampled_of"]
window   = "2026-Q1"   # optional, free-text
```

- **Type:** `[criterion.<id>.<kind>]` subtable.
- **`required`:** array of strings — names of evidence sub-fields the criterion commits to providing.
- **`window` (sample only):** optional free-text string. Echoes what runtime would emit; documents the intended sampling window for the operator.
- **Spec floor (enforced):** `required` MUST contain at least the spec-mandated minimums per PLUGINS.en.md:
  - `attestation`: `signer`, `signed_at`
  - `sample`: `sampled_count`, `sampled_of`
- **May tighten:** `required` MAY also include optional spec fields (`valid_through`, `as_of`, `window`) to elevate them to required for *this* criterion. Useful when an audit clause demands explicit expiry tracking (e.g. 9001 management review's quarterly cadence).
- **Static check (BWOC-29):**
  - Subtable present? If `expected_evidence_kind ∈ {attestation, sample}` and the subtable is absent → violation.
  - `required` array present and array-of-strings? If not → violation.
  - All entries in `required` are valid sub-field names for that kind? Unknown names (e.g. `"frobnicator"` under attestation) → violation.
  - Spec floor satisfied? Missing `signer`/`signed_at` under attestation or `sampled_count`/`sampled_of` under sample → violation with a precise remedy.

For `file`, `content`, `command`, `none`: no subtable is needed (existing BWOC-17 behavior preserved). A subtable for these kinds is silently ignored — they have no spec-mandated sub-fields.

### 3. Why subtable, not inline-table

Considered: `attestation = { required = ["signer", "signed_at"] }` (inline-table).

Rejected for three reasons:

1. **Extensibility.** Future BWOC versions may add per-kind fields (e.g. `window_recommended` for sample, `signer_role` for attestation). Subtables grow naturally; inline tables become long and unreadable.
2. **TOML idiom.** The framework already uses subtables for `[plugin]`, `[criterion.<id>]`, `[contract]`. Per-kind grouping fits the convention.
3. **Diff-friendliness.** Subtables put one field per line. Inline tables collapse changes onto a single diff line, making git history less useful.

Trade-off accepted: subtable is verbose for the common case (just `required = [..]`). Acceptable — `criteria.toml` files are operator-facing and read in full when reviewed; verbosity here is documentation, not noise.

## Examples

### Happy path — attestation

```toml
[criterion.9001-management-review]
severity = "high"
clause   = "9.3"
expected_evidence_kind = "attestation"

[criterion.9001-management-review.attestation]
required = ["signer", "signed_at"]
```

### Happy path — sample with tightened required-list

```toml
[criterion.20000-1-incident-management]
severity = "high"
clause   = "8.6.1"
expected_evidence_kind = "sample"

[criterion.20000-1-incident-management.sample]
required = ["sampled_count", "sampled_of", "window"]  # elevated `window` to required
```

### Backward-compat — kind omitted

```toml
# audit-iso-29110 — unchanged from v0.1.0
[criterion.29110-bp-project-plan]
severity = "medium"
description = "..."
```

No `expected_evidence_kind` → no per-kind check, no subtable required. BWOC-17 checks still apply.

### Backward-compat — kind=file/content/command/none

```toml
[criterion.29110-bp-traceability-matrix]
severity = "medium"
expected_evidence_kind = "file"
description = "Traceability matrix exists at docs/en/TRACEABILITY.en.md"
```

`file` has no spec-mandated sub-fields. Static check passes without a subtable.

## Failure modes (test coverage)

| # | Fixture | Expected violation |
|---|---|---|
| F1 | `expected_evidence_kind = "attestion"` (typo) | `"expected_evidence_kind 'attestion' not in {file, content, command, attestation, sample, none}"` |
| F2 | `kind=attestation` with no `[criterion.<id>.attestation]` subtable | `"expected_evidence_kind='attestation' but no [criterion.<id>.attestation] subtable declaring required sub-fields"` |
| F3 | `kind=attestation` with subtable missing `signer` in `required` | `"criterion 'X' attestation.required must include 'signer' (spec floor)"` |
| F4 | `kind=attestation` with `required = ["signer", "frobnicator"]` | `"criterion 'X' attestation.required contains unknown sub-field 'frobnicator' — valid: signer, signed_at, valid_through, as_of"` |
| F5 | `kind=sample` with subtable missing `sampled_of` | `"criterion 'X' sample.required must include 'sampled_of' (spec floor)"` |
| F6 | `kind=sample` with `required` not an array | `"criterion 'X' sample.required has wrong type — expected array of strings"` |

Plus happy-path tests for each of the six kinds + the "no kind declared" backward-compat case + the "non-audit plugin doesn't trigger evidence-kind check" exemption.

## Implementation surface

Within `crates/bwoc-cli/src/check.rs`, the existing `audit_audit_criteria(plugin_dir, &mut report)` function is the home for the new logic. Two small additions:

1. After the existing `severity` check loop, read `expected_evidence_kind` from the criterion table. If present and ∈ enum, branch into per-kind validation.
2. For `attestation`/`sample`, look up the matching subtable under the criterion's table, validate `required` array, check spec floor + unknown sub-fields.

Two new constants:

```rust
const EVIDENCE_KINDS: &[&str] = &["file", "content", "command", "attestation", "sample", "none"];

/// Per-kind valid sub-field names. The spec floor is a subset of this list
/// (attestation: signer+signed_at minimum; sample: sampled_count+sampled_of
/// minimum). Optional fields like valid_through can be promoted to required.
const ATTESTATION_FIELDS: &[&str] = &["signer", "signed_at", "valid_through", "as_of"];
const SAMPLE_FIELDS: &[&str] = &["sampled_count", "sampled_of", "window", "valid_through", "as_of"];
```

Zero new function exports — all new logic is private to `check.rs`. Test surface stays in the existing `#[cfg(test)] mod tests` block.

## Backward compatibility

- `audit-iso-29110`'s `criteria.toml` declares no `expected_evidence_kind` — no behavior change.
- `audit-iso-9001/20000-1/27001` stubs declare no `expected_evidence_kind` — no behavior change. Authors are free to add the new fields incrementally as runtimes land per BWOC-28's pattern.
- BWOC-17 checks (kebab-case `criterion_id`, closed `severity`) run as before. The new checks layer above, not replace.

## Decision

**Adopt the proposal as-is.** Subtable shape, spec-floor enforcement, optional `expected_evidence_kind` field. Implementation lands in BWOC-29 PR off `main`, branch `agent/agent-rose/feat/BWOC-29`.

## Related

- [`docs/en/PLUGINS.en.md`](../docs/en/PLUGINS.en.md) §"Audit Findings Schema" — normative schema BWOC-27 extended.
- [`notes/2026-05-27_iso-runtime-evidence-model.md`](2026-05-27_iso-runtime-evidence-model.md) — BWOC-26 design note that first proposed `expected_evidence_kind`.
- [`modules/plugins/audit-iso-9001/`](../modules/plugins/audit-iso-9001/) — first runtime consumer (BWOC-28). The runtime emits attestation findings; this static check verifies criteria.toml could legally declare them.
- `.scrum/backlog.json` BWOC-29 — story this design note unblocks.
