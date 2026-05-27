# 2026-05-27 — EPIC-3 ISO Runtime evidence model

Design note covering the new evidence kinds the EPIC-3 runtime needs that v1 of the [BWOC-11 Audit Findings Schema](../docs/en/PLUGINS.en.md#audit-findings-schema) does not support. EPIC-2 shipped four `audit/*` plugins: one runnable (`audit-iso-29110`, file-existence) and three stubs (`audit-iso-9001`, `audit-iso-20000-1`, `audit-iso-27001`) whose every finding is `status="not_implemented"` with `remedy="Runtime deferred to BWOC-EPIC-3."` The stubs are honest placeholders — pretending to audit organisational practice from `RISKS.md`-style file existence would falsify the audit (Musāvāda, [Sila 5](../docs/en/PHILOSOPHY.en.md)). EPIC-3 builds the evidence model that makes the runtimes truthful.

The output of this note is a small enum extension (additive, not breaking) plus three optional fields. Schema bumps land in BWOC-27; this note pins the design first.

## Problem

The v1 schema has three evidence kinds: `file`, `content`, `none`. That covers file-existence (`audit-iso-29110`) and intentional-null (every stub finding today). It does **not** cover what the three EPIC-3 runtimes actually need:

- **ISO 9001 (QMS)** — clauses like 9.3 *Management review* ask whether the organisation reviewed the QMS in a defined cadence with documented inputs/outputs. The artefact is an operator-signed statement that the review happened, dated, with provenance — not a file the auditor opens.
- **ISO/IEC 20000-1 (ITSM)** — clauses like 8.6.1 *Incident management* ask whether the SMS handled N incidents within SLA. The artefact lives in a ticket system, not the workspace, and the answer is a rate over a window — "98% of last quarter's incidents resolved within SLA", not a single record.
- **ISO/IEC 27001 (ISMS)** — clauses like 6.1.3 *Statement of Applicability* and the 93 Annex A controls ask which controls are in scope and were sampled. The runtime must declare "we sampled controls A.5.15, A.5.24, A.5.29 (3 of 37 *Organizational* theme)" without inflating the finding count to 37.

These map onto three new evidence shapes plus a traceability concern. The v1 enum cannot represent them.

## New evidence kinds

The proposal adds two values to the `evidence.kind` enum and three optional fields. Existing values (`file`, `content`, `none`) are unchanged, and existing finding payloads continue to validate.

### `attestation` — operator-signed statement with provenance

> The operator (or a delegated team member) asserts that a thing happened, when it happened, and who signed it off. The runtime captures the assertion verbatim and stamps it with provenance.

Required sub-fields when `evidence.kind = "attestation"`:

| Field | Type | Required | Notes |
|---|---|---|---|
| `evidence.value` | string | yes | The free-text statement, verbatim. Multi-line allowed. |
| `evidence.signer` | string | yes | Identity that vouches for the statement (e.g. `"CISO: Suchada N."` or `"Quality Manager: Tonkla K."`). Free-text; no enum. |
| `evidence.signed_at` | string (ISO 8601 date or datetime) | yes | When the statement was signed. |

Example payload for QMS clause 9.3 management review:

```json
{
  "criterion_id": "9001-management-review",
  "severity": "high",
  "status": "pass",
  "evidence": {
    "kind": "attestation",
    "value": "Management review held 2026-04-15 covering Q1 QMS performance, customer feedback, internal audit results, and improvement opportunities. Minutes archived in Drive.",
    "signer": "Quality Manager: Tonkla K.",
    "signed_at": "2026-04-15"
  }
}
```

Used by: **9001** (most clauses), **27001** (5.2 policy, 9.2 internal audit), **20000-1** (5.2 service policy).

### `sample` — sampled-N-of-M result with rate

> The runtime sampled N items from a population of M and reports the rate. Used when the underlying signal is statistical (incidents within SLA, controls sampled from Annex A) and listing every item would inflate findings.

Required sub-fields when `evidence.kind = "sample"`:

| Field | Type | Required | Notes |
|---|---|---|---|
| `evidence.value` | string | yes | Short human summary (e.g. `"49 of 50 incidents resolved within SLA"`). |
| `evidence.sampled_count` | integer | yes | N — number actually sampled / measured. |
| `evidence.sampled_of` | integer | yes | M — population size. |
| `evidence.window` | string | optional | Time window for the sample (e.g. `"2026-Q1"`, `"last 90 days"`). Free-text. |

Example payload for ITSM clause 8.6.1 incident management:

```json
{
  "criterion_id": "20000-1-incident-management",
  "severity": "high",
  "status": "pass",
  "evidence": {
    "kind": "sample",
    "value": "49 of 50 incidents resolved within SLA",
    "sampled_count": 49,
    "sampled_of": 50,
    "window": "2026-Q1"
  }
}
```

Example payload for ISMS Annex A control sampling:

```json
{
  "criterion_id": "27001-access-control",
  "severity": "high",
  "status": "pass",
  "evidence": {
    "kind": "sample",
    "value": "Access control reviewed for 3 of 37 Organizational-theme controls",
    "sampled_count": 3,
    "sampled_of": 37,
    "window": "2026-Q1"
  }
}
```

Used by: **20000-1** (clauses 8.6, 8.3 — rate-based), **27001** (Annex A sampling, SoA scope).

## Time-bounded fields (orthogonal to kind)

Some evidence is fresh, some is stale. A 2023 management review does not satisfy clause 9.3 the same way one from last quarter does. Two **optional** fields apply to any `evidence.kind` (typically `attestation` or `sample`):

| Field | Type | Required | Notes |
|---|---|---|---|
| `evidence.as_of` | string (ISO 8601 date) | optional | When this evidence was current. |
| `evidence.valid_through` | string (ISO 8601 date) | optional | When this evidence stops being authoritative — operator-declared. After that date the runtime can emit `severity = "warn"` automatically. |

Time-bounded fields are **additive** — existing schemas don't have them, runtimes that don't care can ignore them. The bwoc audit dispatcher does not enforce semantics; it stamps the values and surfaces them in the report. Future tooling (e.g. `bwoc audit report --expired`) can use them.

## Control-to-clause traceability (deferred)

The third concern — one control supporting multiple clauses (e.g. A.5.24 incident-management feeds both clause 9 *Performance evaluation* and clause 10 *Improvement* of ISO/IEC 27001) — does **not** need a schema change in v2. The current `criterion_id`-per-finding model is fine; ISMS runtime can declare two criteria sharing the same evidence by referencing the same attestation `value`. If duplication becomes a real ergonomic pain, a `evidence.related_criteria` array can be added in v3.

This note explicitly defers it. EPIC-3 audit-iso-27001 runtime should not block on it.

## Standard ↔ evidence-kind matrix

| Standard | Main evidence kind | Time-bounded? | Sampling? | Stub today |
|---|---|---|---|---|
| **ISO/IEC 29110** | `file` (Basic-profile work products) | no | no | runnable ✓ |
| **ISO 9001** | `attestation` (most clauses) + `file` (where a published policy doc exists) | yes (signed_at + valid_through) | no | stub today |
| **ISO/IEC 20000-1** | `sample` (incidents, changes, SLAs) + `attestation` (policy) | yes (window) | yes | stub today |
| **ISO/IEC 27001** | `attestation` (5.2, 6.1.2, 6.1.3) + `sample` (Annex A controls) | yes | yes (SoA-driven) | stub today |

## Decision: additive, not breaking

**The change is purely additive.** v1 producers and consumers continue to work. Specifically:

- `evidence.kind` enum grows from `{file, content, none}` to `{file, content, attestation, sample, none}`. Existing values unchanged.
- New optional fields (`signer`, `signed_at`, `sampled_count`, `sampled_of`, `window`, `as_of`, `valid_through`) are required **only** when the corresponding `evidence.kind` is used.
- `audit-iso-29110` (v0.1.0) does not need to change at all — it emits `kind = "file"` exclusively.
- `audit-iso-9001/20000-1/27001` stubs (v0.1.0) do not need to change — they emit `kind = "none"` exclusively.
- The PLUGINS spec schema-version constant bumps by **one minor revision** to signal the addition. The major remains the same.

`bwoc check` extension (BWOC-29) enforces the per-kind required-field rule. The existing `criterion_id` kebab-case + `severity` enum checks from BWOC-17 are untouched.

## Implications

### For criteria.toml

A new optional field `expected_evidence_kind` may declare what kind a runtime intends to emit for that criterion. This lets `bwoc check` validate the static contract without invoking the plugin. Example:

```toml
[criterion.9001-management-review]
title = "Management review"
severity = "high"
expected_evidence_kind = "attestation"  # NEW, optional
```

If omitted, `bwoc check` does not enforce a specific kind — the plugin is free to choose at invoke time.

### For findings JSON shape

Already shown in the example payloads above. No structural change; just two new enum values and seven optional sub-fields.

### For `bwoc audit run` (the dispatcher)

The dispatcher validates each finding against the updated schema. If a plugin emits `kind = "attestation"` without `signer` or `signed_at`, the dispatcher fails the finding (framework error, exit code 255 per BWOC-12). No new dispatcher logic is needed beyond schema enforcement.

### For TH parity

PLUGINS.th.md mirrors the schema change in the same commit (BWOC-27). The new evidence kinds keep their English keys (`attestation`, `sample`) — only the **prose** describing them is translated. Thai SME framing per BWOC-19: emphasise that 9001 และ 27001 พึ่งพา **attestation** เป็นหลัก ซึ่งหมายถึงผู้บริหารต้อง sign-off — ไม่ใช่ alpha-bet check.

## Alternatives considered

- **Free-form `evidence.value` string only** — let plugins put anything in. Rejected: defeats the schema's purpose (typed, validatable findings). The whole BWOC-11 design is "small typed surface, big plugin flexibility".
- **Per-standard custom evidence types** — `evidence.kind = "iso-9001-attestation"`. Rejected: explodes the enum, ties the schema to specific standards, breaks `audit-iso-cmmc` if/when it shows up.
- **`evidence.kind = "attestation"` with a free-form metadata bag** — Rejected: harder to validate, harder to query, harder to translate. Explicit named sub-fields are the BWOC house style (see BWOC-11 original schema).
- **Defer everything to v3** — Rejected: EPIC-3 ships runtimes; it needs schema first. The whole point of the BWOC-EPIC-2 → BWOC-EPIC-3 split was "stubs first, schema later, runtime last."

## Status / open questions

**Status:** ready for BWOC-27 to land schema bumps.

**Open questions** (defer to BWOC-27 review):

1. Should `evidence.signed_at` accept date or datetime, or both? Proposal: both (ISO 8601 allows both forms; consumer normalises).
2. Should `evidence.window` be enum or free-text? Proposal: free-text for v2; promote to enum (`Q1`, `Q2`, ..., `last_30d`, `last_90d`, `last_365d`) in v3 if patterns emerge.
3. Should `valid_through` auto-degrade severity to `warn` past the date? Proposal: **no** for v2 — the dispatcher stamps, downstream tooling decides. Keep responsibility split clean.

## Related

- [`docs/en/PLUGINS.en.md`](../docs/en/PLUGINS.en.md) §Audit Findings Schema — current v1 schema (touched in BWOC-27).
- [`notes/2026-05-26_iso-compliance-plugins.md`](2026-05-26_iso-compliance-plugins.md) — EPIC-2 framing note (why stubs, why runtime deferred).
- [`notes/2026-05-27_audit-exit-code-reconcile.md`](2026-05-27_audit-exit-code-reconcile.md) — jennie's BWOC-12 polish (related dispatcher contract).
- `modules/plugins/audit-iso-9001/SPEC.md` — first runtime target (BWOC-28).
- `.scrum/backlog.json` BWOC-27 / BWOC-28 / BWOC-29 — downstream stories that consume this design.
