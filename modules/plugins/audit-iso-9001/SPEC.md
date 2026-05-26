---
title: ISO 9001 Quality Management System Audit (stub)
aliases:
  - audit-iso-9001
tags:
  - group/framework-plugins
  - type/plugin
  - kind/audit
  - domain/compliance
  - standard/iso-9001
  - status/stub
maturity: L0
---

# ISO 9001 Quality Management System Audit (stub)

> [!abstract] **Stub plugin.** Declares the QMS criteria the future runtime will check; emits `status = "not_implemented"` for every criterion with the uniform remedy `"Runtime deferred to BWOC-EPIC-3."` Schema conformance only — no real audit logic. The full runtime lands in [[../../notes/2026-05-26_iso-compliance-plugins|BWOC-EPIC-3]].

## Why This Is a Stub

[[../../notes/2026-05-26_iso-compliance-plugins|The EPIC-2 framing note]] explains the layered depth (full / stub / stub / stub) for the four ISO plugins. The short version:

- ISO 9001 describes **organizational practices** — quality policy, management review, internal audit, corrective action — that do not reduce to file-existence on a workspace. Inferring "this organization has a documented quality policy" from "this repo contains `POLICY.md`" would falsify the audit (Musāvāda — see [PHILOSOPHY.en.md](../../docs/en/PHILOSOPHY.en.md) §Sila 5).
- Runtime for 9001-class plugins needs a richer evidence model (operator attestations, time-bounded evidence, sampling) that is not present in v1 of the [PLUGINS.en.md §Audit Findings Schema](../../docs/en/PLUGINS.en.md#audit-findings-schema). Building that is EPIC-3 work.
- Operators running `bwoc plugin list --kind audit` after Sprint 3 see all four ISO frameworks. The absence of 9001 would imply "BWOC has no opinion on QMS"; the stub form says "BWOC has a placeholder, runtime is on the roadmap." That is the honest signal.

## Criteria (v0.1.0)

Eight headline QMS criteria, drawn from the main clauses of ISO 9001:2015[^iso-9001-2015]. Declaration order in [[criteria]] is the report order (PLUGINS.en.md line 84). `criterion_id` values are stable across releases (PLUGINS.en.md §Stability); renames are a major version bump.

| `criterion_id` | Clause | Title | Severity |
|---|---|---|---|
| `9001-context-of-organization` | 4 | Context of the organization | high |
| `9001-leadership-and-policy` | 5.2 | Leadership and quality policy | high |
| `9001-risks-and-opportunities` | 6.1 | Actions to address risks and opportunities | high |
| `9001-competence-and-awareness` | 7.2 | Competence and awareness | medium |
| `9001-documented-information` | 7.5 | Documented information | medium |
| `9001-internal-audit` | 9.2 | Internal audit | high |
| `9001-management-review` | 9.3 | Management review | high |
| `9001-corrective-action` | 10.2 | Nonconformity and corrective action | medium |

The eight criteria cover one practice per major clause (4 through 10) that every QMS-conformant organization is expected to operate. The residual sub-clauses (resources, infrastructure, monitoring & measurement of operational processes, customer satisfaction surveys, etc.) are deferred to v0.2.0 once the EPIC-3 runtime exists and can support them.

## How It Runs (Today)

```bash
bwoc audit run --plugin audit-iso-9001 --json
```

The dispatcher spawns `audit.sh` (per `BWOC-12`) with the standard env contract: `BWOC_WORKSPACE`, `BWOC_PLUGIN_DIR`, `BWOC_AUDIT_OPERATION`. The stub:

1. Reads `criteria.toml` from `BWOC_PLUGIN_DIR`.
2. For each criterion in declaration order, emits a finding with `status = "not_implemented"`, `evidence = { kind: "none", value: "" }`, and `remedy = "Runtime deferred to BWOC-EPIC-3."`
3. Exits `0` on success.

`BWOC_WORKSPACE` is intentionally **unread**. The stub does not inspect the workspace, and pretending to would falsify the audit. This is the Musāvāda guard at the plugin layer.

## Sample Output

```json
[
  {
    "criterion_id": "9001-context-of-organization",
    "severity": "high",
    "status": "not_implemented",
    "evidence": { "kind": "none", "value": "" },
    "remedy": "Runtime deferred to BWOC-EPIC-3."
  },
  {
    "criterion_id": "9001-leadership-and-policy",
    "severity": "high",
    "status": "not_implemented",
    "evidence": { "kind": "none", "value": "" },
    "remedy": "Runtime deferred to BWOC-EPIC-3."
  }
]
```

`bwoc audit run` wraps this into its canonical envelope `{ workspace, runs: [{ plugin, version, started_at, finished_at, findings: [...] }, ...], summary }`. The stub conforms to the same envelope as the runnable `audit-iso-29110` plugin — operators who learn one learn both.

## Configuration

```toml
# workspace.toml
[plugins.audit-iso-9001]
enabled = true
```

The plugin declares no `[config.schema]` in its manifest — the only workspace-level surface is the universal `enabled` key. A `profile` or `scope` key may be added once the EPIC-3 runtime exists.

## What EPIC-3 Will Add

The EPIC-3 runtime needs at minimum:

- **Attestation evidence.** Operator-provided statements ("our last management review was 2026-04-15, signed off by X") that the framework can record and stamp with provenance.
- **Time-bounded evidence.** A management review from three years ago does not satisfy the same clause as one from last quarter. The schema needs a "valid through" or "as of" dimension.
- **Sampling.** Internal audit conformance is about coverage across the QMS, not a single artifact. The runtime needs a way to declare "sampled N of M processes" without inflating finding count.

Until those land, every criterion in this file emits `not_implemented`. The `criterion_id`s are stable — when EPIC-3 lands the runtime, the IDs do not change; only the `status` / `evidence` / `remedy` values do.

## Maturity

Declared **L0** — stub, no runtime, schema conformance only. Bumps to **L1** once EPIC-3 lands the runtime and at least one criterion emits non-`not_implemented` findings against a real workspace.

## Neutrality

Manifest values name no LLM backend, vendor, or model. `kind = "audit"` is the framework's own enum (`BWOC-10`). The plugin references "ISO 9001" exclusively in `description`, in this SPEC's prose, and in the standardized `9001-*` criterion-id namespace — never in `criteria.toml` keys or in finding values beyond that namespace. Satisfies the **Samānattatā** rule.

## Sources

The QMS criteria are drawn from the published ISO 9001:2015 main-body clauses (4 through 10):

- ISO 9001:2015 — *Quality management systems — Requirements.* International Organization for Standardization. ISO catalogue entry: <https://www.iso.org/standard/62085.html>. [^iso-9001-2015]
- ISO/TC 176/SC 2 — *Quality management and quality assurance — Quality systems.* Public landing page: <https://www.iso.org/committee/53896.html>. The technical committee responsible for the ISO 9000 family.
- ISO — *Quality management principles* (publicly available brochure summarising the seven QMS principles that underpin the 2015 revision): <https://www.iso.org/publication/PUB100080.html>.

[^iso-9001-2015]: ISO 9001:2015 is structured around the Annex SL high-level structure (clauses 4 Context, 5 Leadership, 6 Planning, 7 Support, 8 Operation, 9 Performance evaluation, 10 Improvement). The eight criteria here cover one headline practice per main clause that every QMS-conformant organization is expected to operate; the residual sub-clauses (7.1 Resources, 8.5 Production and service provision, 9.1.2 Customer satisfaction, etc.) are deferred to a v0.2.0 expansion once the EPIC-3 runtime exists.

## See Also

- [[../../docs/en/PLUGINS.en|PLUGINS.en.md]] — the plugin spec; `audit` kind row (BWOC-10), Audit Findings Schema (BWOC-11), stub-status example.
- [[../../notes/2026-05-26_iso-compliance-plugins|2026-05-26_iso-compliance-plugins.md]] — EPIC-2 framing note (why stubs, why deferred to EPIC-3).
- [[../audit-iso-29110/SPEC|audit-iso-29110]] — the runnable reference audit plugin from BWOC-13.
- [[../audit-iso-20000-1/SPEC|audit-iso-20000-1]], [[../audit-iso-27001/SPEC|audit-iso-27001]] — sibling stub plugins shipping in the same sprint.
