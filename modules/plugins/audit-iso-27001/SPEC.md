---
title: ISO/IEC 27001 Information Security Management System Audit (stub)
aliases:
  - audit-iso-27001
tags:
  - group/framework-plugins
  - type/plugin
  - kind/audit
  - domain/compliance
  - standard/iso-iec-27001
  - status/stub
maturity: L0
---

# ISO/IEC 27001 Information Security Management System Audit (stub)

> [!abstract] **Stub plugin.** Declares the ISMS criteria the future runtime will check; emits `status = "not_implemented"` for every criterion with the uniform remedy `"Runtime deferred to BWOC-EPIC-3."` Schema conformance only — no real audit logic. The full runtime lands in [[../../notes/2026-05-26_iso-compliance-plugins|BWOC-EPIC-3]].

## Why This Is a Stub

[[../../notes/2026-05-26_iso-compliance-plugins|The EPIC-2 framing note]] explains the layered depth (full / stub / stub / stub) for the four ISO plugins. The short version:

- ISO/IEC 27001 describes **information security management practices** — risk assessment processes, control selection (Annex A), access control policies, incident response readiness, business continuity exercises — that reduce to organisational evidence (policy documents, signed-off SoAs, incident drill records, exercise reports) rather than to repository artifacts. Inferring "this organization has run an information security risk assessment" from "this repo contains `RISKS.md`" would falsify the audit (Musāvāda — see [PHILOSOPHY.en.md](../../docs/en/PHILOSOPHY.en.md) §Sila 5).
- Runtime for 27001-class plugins needs attestation + sampling + control-mapping evidence (operator-signed statements with provenance, sampled controls from Annex A's 93 entries, traceability to the Statement of Applicability) that is not present in v1 of the [PLUGINS.en.md §Audit Findings Schema](../../docs/en/PLUGINS.en.md#audit-findings-schema). Building that is EPIC-3 work.
- Operators running `bwoc plugin list --kind audit` after Sprint 3 see all four ISO frameworks. The absence of 27001 would imply "BWOC has no opinion on InfoSec management"; the stub form says "BWOC has a placeholder, runtime is on the roadmap." That is the honest signal.

## Criteria (v0.1.0)

Eight headline ISMS criteria, drawn from the main clauses and the Annex A controls of ISO/IEC 27001:2022[^iso-27001-2022]. Declaration order in [[criteria]] is the report order (PLUGINS.en.md line 84). `criterion_id` values are stable across releases (PLUGINS.en.md §Stability); renames are a major version bump.

| `criterion_id` | Reference | Title | Severity |
|---|---|---|---|
| `27001-isms-scope` | 4.3 | Scope of the ISMS | high |
| `27001-information-security-policy` | 5.2 | Information security policy | high |
| `27001-risk-assessment` | 6.1.2 | Information security risk assessment | critical |
| `27001-statement-of-applicability` | 6.1.3 | Statement of Applicability | critical |
| `27001-access-control` | A.5.15 | Access control | high |
| `27001-incident-management` | A.5.24 | Incident management planning and preparation | high |
| `27001-business-continuity` | A.5.29 | Information security during disruption | medium |
| `27001-internal-audit` | 9.2 | Internal audit | high |

The eight criteria mix four **main-body clauses** (4.3, 5.2, 6.1.2, 6.1.3, 9.2 — the ISMS-establishment requirements every certified organization must operate) and three **Annex A controls** from the *Organizational* theme (5.15 access control, 5.24 incident preparation, 5.29 continuity). The Annex A controls were re-themed and de-duplicated from 114 (in :2013) to 93 (in :2022) across four themes — *Organizational* (37), *People* (8), *Physical* (14), *Technological* (34). The remaining 90 Annex A controls are deferred to v0.2.0 once the EPIC-3 runtime can sample them coherently.

## How It Runs (Today)

```bash
bwoc audit run --plugin audit-iso-27001 --json
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
    "criterion_id": "27001-isms-scope",
    "severity": "high",
    "status": "not_implemented",
    "evidence": { "kind": "none", "value": "" },
    "remedy": "Runtime deferred to BWOC-EPIC-3."
  },
  {
    "criterion_id": "27001-information-security-policy",
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
[plugins.audit-iso-27001]
enabled = true
```

The plugin declares no `[config.schema]` in its manifest — the only workspace-level surface is the universal `enabled` key. Control-selection keys (`soa_path`, `theme_filter`, etc.) may be added once the EPIC-3 runtime exists.

## What EPIC-3 Will Add

The EPIC-3 runtime needs at minimum:

- **Attestation evidence.** Operator-provided statements ("our last risk assessment was 2026-03-10, signed off by the CISO, scope: customer-data flows") with provenance and time stamps.
- **SoA-driven control sampling.** The Statement of Applicability declares which of the 93 Annex A controls are in scope. The runtime must sample only those, declare `not_applicable` for the rest, and surface SoA gaps.
- **Time-bounded evidence.** A risk assessment from three years ago does not satisfy 6.1.2 the same way one from last quarter does. The schema needs an "as of" or "valid through" dimension.
- **Control-to-clause traceability.** Some controls support multiple clauses (e.g. A.5.24 incident-management feeds both clause 9 and clause 10). The runtime needs a way to express this without duplicating findings.

Until those land, every criterion in this file emits `not_implemented`. The `criterion_id`s are stable — when EPIC-3 lands the runtime, the IDs do not change; only the `status` / `evidence` / `remedy` values do.

## Maturity

Declared **L0** — stub, no runtime, schema conformance only. Bumps to **L1** once EPIC-3 lands the runtime and at least one criterion emits non-`not_implemented` findings against a real workspace.

## Neutrality

Manifest values name no LLM backend, vendor, or model. `kind = "audit"` is the framework's own enum (`BWOC-10`). The plugin references "ISO/IEC 27001" exclusively in `description`, in this SPEC's prose, and in the standardized `27001-*` criterion-id namespace — never in `criteria.toml` keys or in finding values beyond that namespace. Satisfies the **Samānattatā** rule.

## Sources

The ISMS criteria are drawn from the published ISO/IEC 27001:2022 main-body clauses (4 through 10) and the Annex A control set:

- ISO/IEC 27001:2022 — *Information security, cybersecurity and privacy protection — Information security management systems — Requirements.* International Organization for Standardization. ISO catalogue entry: <https://www.iso.org/standard/27001>. [^iso-27001-2022]
- ISO/IEC 27002:2022 — *Information security, cybersecurity and privacy protection — Information security controls.* ISO catalogue entry: <https://www.iso.org/standard/75652.html>. Companion standard providing implementation guidance for the Annex A controls.
- ISO/IEC JTC 1/SC 27 — *Information security, cybersecurity and privacy protection.* Public landing page: <https://www.iso.org/committee/45306.html>. The joint technical committee responsible for the ISO/IEC 27000 family.

[^iso-27001-2022]: ISO/IEC 27001:2022 follows the Annex SL high-level structure (clauses 4 Context, 5 Leadership, 6 Planning, 7 Support, 8 Operation, 9 Performance evaluation, 10 Improvement). Annex A contains 93 information security controls organized into four themes — *Organizational* (37 controls, prefix A.5.x), *People* (8 controls, prefix A.6.x), *Physical* (14 controls, prefix A.7.x), and *Technological* (34 controls, prefix A.8.x) — restructured from the 114-control / 14-clause arrangement in ISO/IEC 27001:2013. The eight criteria here cover five main-body clauses plus three *Organizational*-theme Annex A controls that headline ISMS operations; the remaining 90 Annex A controls are deferred to a v0.2.0 expansion once the EPIC-3 runtime can sample them coherently.

## See Also

- [[../../docs/en/PLUGINS.en|PLUGINS.en.md]] — the plugin spec; `audit` kind row (BWOC-10), Audit Findings Schema (BWOC-11), stub-status example.
- [[../../notes/2026-05-26_iso-compliance-plugins|2026-05-26_iso-compliance-plugins.md]] — EPIC-2 framing note (why stubs, why deferred to EPIC-3).
- [[../audit-iso-29110/SPEC|audit-iso-29110]] — the runnable reference audit plugin from BWOC-13.
- [[../audit-iso-9001/SPEC|audit-iso-9001]], [[../audit-iso-20000-1/SPEC|audit-iso-20000-1]] — sibling stub plugins shipping in the same sprint.
