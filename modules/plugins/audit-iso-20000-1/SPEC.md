---
title: ISO/IEC 20000-1 IT Service Management System Audit (stub)
aliases:
  - audit-iso-20000-1
tags:
  - group/framework-plugins
  - type/plugin
  - kind/audit
  - domain/compliance
  - standard/iso-iec-20000-1
  - status/stub
maturity: L0
---

# ISO/IEC 20000-1 IT Service Management System Audit (stub)

> [!abstract] **Stub plugin.** Declares the ITSM criteria the future runtime will check; emits `status = "not_implemented"` for every criterion with the uniform remedy `"Runtime deferred to BWOC-EPIC-3."` Schema conformance only — no real audit logic. The full runtime lands in [[../../notes/2026-05-26_iso-compliance-plugins|BWOC-EPIC-3]].

## Why This Is a Stub

[[../../notes/2026-05-26_iso-compliance-plugins|The EPIC-2 framing note]] explains the layered depth (full / stub / stub / stub) for the four ISO plugins. The short version:

- ISO/IEC 20000-1 describes **service management practices** — service catalogue maintenance, SLA agreements, incident and problem management, change records — that live in service-management tooling (ITSM platforms, ticket systems), not in workspace files. Inferring "this organization runs incident management" from "this repo contains `INCIDENTS.md`" would falsify the audit (Musāvāda — see [PHILOSOPHY.en.md](../../docs/en/PHILOSOPHY.en.md) §Sila 5).
- Runtime for 20000-1-class plugins needs adapter-style evidence (ITSM-system queries, ticket samples, SLA-report parsing) that is not present in v1 of the [PLUGINS.en.md §Audit Findings Schema](../../docs/en/PLUGINS.en.md#audit-findings-schema). Building that is EPIC-3 work.
- Operators running `bwoc plugin list --kind audit` after Sprint 3 see all four ISO frameworks. The absence of 20000-1 would imply "BWOC has no opinion on ITSM"; the stub form says "BWOC has a placeholder, runtime is on the roadmap." That is the honest signal.

## Criteria (v0.1.0)

Eight headline ITSM criteria, drawn from the main clauses of ISO/IEC 20000-1:2018[^iso-20000-1-2018]. Declaration order in [[criteria]] is the report order (PLUGINS.en.md line 84). `criterion_id` values are stable across releases (PLUGINS.en.md §Stability); renames are a major version bump.

| `criterion_id` | Clause | Title | Severity |
|---|---|---|---|
| `20000-1-service-management-system-scope` | 4.3 | Scope of the service management system | high |
| `20000-1-service-policy-and-objectives` | 5.2 | Service management policy and objectives | high |
| `20000-1-service-catalogue` | 8.3.1 | Service catalogue | medium |
| `20000-1-service-level-management` | 8.3.3 | Service level management | high |
| `20000-1-change-management` | 8.5.1 | Change management | high |
| `20000-1-incident-management` | 8.6.1 | Incident management | high |
| `20000-1-problem-management` | 8.6.3 | Problem management | medium |
| `20000-1-continual-improvement` | 10.2 | Continual improvement | medium |

The eight criteria cover the headline practices a 20000-1-conformant organization is expected to operate, balanced across the **plan-the-SMS** clauses (4, 5) and the **operate-the-SMS** clauses (8 — service portfolio, service delivery, resolution). Sub-clauses 8.2 (service portfolio), 8.4 (supply & demand — capacity, demand, budgeting), and 8.7 (service assurance — availability, continuity, info security) are deferred to v0.2.0.

## How It Runs (Today)

```bash
bwoc audit run --plugin audit-iso-20000-1 --json
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
    "criterion_id": "20000-1-service-management-system-scope",
    "severity": "high",
    "status": "not_implemented",
    "evidence": { "kind": "none", "value": "" },
    "remedy": "Runtime deferred to BWOC-EPIC-3."
  },
  {
    "criterion_id": "20000-1-service-policy-and-objectives",
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
[plugins.audit-iso-20000-1]
enabled = true
```

The plugin declares no `[config.schema]` in its manifest — the only workspace-level surface is the universal `enabled` key. ITSM-tool adapter keys (`itsm_endpoint`, `ticket_query`, etc.) may be added once the EPIC-3 runtime exists.

## What EPIC-3 Will Add

The EPIC-3 runtime needs at minimum:

- **Adapter evidence.** Read-only adapters to common ITSM platforms (Jira Service Management, ServiceNow, Zendesk, plain CSV exports) that can answer "how many incidents in the last quarter, with what SLA breach rate?"
- **Sampling and rolling windows.** SLA performance is a rate over a window, not a single artifact. The runtime needs a way to declare "sampled N incidents from the last 90 days" without inflating finding count.
- **Operator attestation.** Some clauses (the service-management policy, the management commitment) reduce to "yes, we have one, signed off by X on Y." The schema needs an attestation evidence kind.

Until those land, every criterion in this file emits `not_implemented`. The `criterion_id`s are stable — when EPIC-3 lands the runtime, the IDs do not change; only the `status` / `evidence` / `remedy` values do.

## Maturity

Declared **L0** — stub, no runtime, schema conformance only. Bumps to **L1** once EPIC-3 lands the runtime and at least one criterion emits non-`not_implemented` findings against a real workspace.

## Neutrality

Manifest values name no LLM backend, vendor, or model. `kind = "audit"` is the framework's own enum (`BWOC-10`). The plugin references "ISO/IEC 20000-1" exclusively in `description`, in this SPEC's prose, and in the standardized `20000-1-*` criterion-id namespace — never in `criteria.toml` keys or in finding values beyond that namespace. Satisfies the **Samānattatā** rule.

## Sources

The ITSM criteria are drawn from the published ISO/IEC 20000-1:2018 main-body clauses (4 through 10):

- ISO/IEC 20000-1:2018 — *Information technology — Service management — Part 1: Service management system requirements.* International Organization for Standardization. ISO catalogue entry: <https://www.iso.org/standard/70636.html>. [^iso-20000-1-2018]
- ISO/IEC JTC 1/SC 40 — *IT service management and IT governance.* The joint technical committee responsible for the ISO/IEC 20000 family: <https://www.iso.org/committee/5013818.html>.
- ISO/IEC 20000-10:2018 — *Concepts and vocabulary* (publicly excerpted in ISO's online browsing platform for terminology only): useful for distinguishing "service", "SMS", and "SLA" definitions.

[^iso-20000-1-2018]: ISO/IEC 20000-1:2018 follows the Annex SL high-level structure (clauses 4 Context, 5 Leadership, 6 Planning, 7 Support, 8 Operation, 9 Performance evaluation, 10 Improvement). Clause 8 (Operation of the SMS) is the substantive ITSM-practice clause and is itself broken into seven sub-clauses (8.1 operational planning, 8.2 service portfolio, 8.3 relationship & agreement, 8.4 supply & demand, 8.5 service design build & transition, 8.6 resolution & fulfilment, 8.7 service assurance). The eight criteria here cover one headline practice from each of clauses 4, 5, 8.3, 8.5, 8.6, and 10; the residual sub-clauses (8.2, 8.4, 8.7, plus several 8.3 and 8.6 children) are deferred to a v0.2.0 expansion once the EPIC-3 runtime exists.

## See Also

- [[../../docs/en/PLUGINS.en|PLUGINS.en.md]] — the plugin spec; `audit` kind row (BWOC-10), Audit Findings Schema (BWOC-11), stub-status example.
- [[../../notes/2026-05-26_iso-compliance-plugins|2026-05-26_iso-compliance-plugins.md]] — EPIC-2 framing note (why stubs, why deferred to EPIC-3).
- [[../audit-iso-29110/SPEC|audit-iso-29110]] — the runnable reference audit plugin from BWOC-13.
- [[../audit-iso-9001/SPEC|audit-iso-9001]], [[../audit-iso-27001/SPEC|audit-iso-27001]] — sibling stub plugins shipping in the same sprint.
