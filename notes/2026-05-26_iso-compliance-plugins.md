---
title: ISO compliance plugins — framing for EPIC-2
date: 2026-05-26
sprint: BWOC sprint-2
epic: BWOC-EPIC-2
story: BWOC-19
related_stories: [BWOC-10, BWOC-11, BWOC-12, BWOC-13, BWOC-14, BWOC-15, BWOC-16, BWOC-17, BWOC-18]
---

# 2026-05-26 — ISO compliance plugins (EPIC-2 framing)

This note sets the spec frame for `BWOC-EPIC-2` before any of the four ISO compliance plugins land. It answers the four design questions Sprint 2 must resolve before `BWOC-10` (kind extension), `BWOC-11` (findings schema), and `BWOC-13` (the first runnable plugin) can be drafted without churn: why `audit` is the right name for a fourth plugin kind, why ISO/IEC 29110 carries the runtime burden first, why the other three frameworks ship as stubs, and what the findings report must guarantee so every audit plugin — runnable or stub — speaks one shape.

## Decisions

### 1. `audit` becomes the 4th plugin kind, not `compliance` or `policy`

The current PLUGINS spec enumerates three kinds in [PLUGINS.en.md §Kinds](../docs/en/PLUGINS.en.md#kinds): `memory-backend`, `llm-backend`, `workflow`. Each kind is defined by **the lifecycle hook the framework calls and who owns the call site** (see "Lifecycle owner per kind", PLUGINS.en.md §Lifecycle). ISO compliance work does not fit any of the three: it is not memory, not a model call, and not an outbound integration into a third-party system. It is an **inward-facing inspection** of the workspace itself, run on operator demand (`bwoc audit run`, per `BWOC-12`).

Adopt `audit` as the name (not `compliance`, not `policy`):

- `audit` describes the **action shape** — inspect, report findings, exit with a status — and stays neutral to the **standard being audited**. The same kind covers ISO/IEC 29110 today, ISO 9001 / 20000-1 / 27001 in Sprint 3, and any operator-authored audit (license headers, doc parity, secret scans) later. `compliance` overloads to "regulatory" and biases readers away from non-ISO audits.
- `policy` reads as configuration (rules the framework enforces on every action). The plugin runs **only when invoked**; it does not gate other operations.
- `audit` already appears as the forward-looking enum value in [PLUGINS.en.md §`init` template](../docs/en/PLUGINS.en.md#bwoc-plugin-init----from-modulesplugin-template) ("Future kinds (e.g. `audit` per BWOC-EPIC-2)"). The hint was deliberate; cash it.

The `BWOC-10` edit extends `[plugin].kind` enum from `{memory-backend | llm-backend | workflow}` to `{memory-backend | llm-backend | workflow | audit}` and adds an `audit` row to both the §Kinds and §Lifecycle owner per kind tables. **Lifecycle owner for `audit`:** the `bwoc audit` CLI command (per `BWOC-12`). **Invoked when:** operator runs `bwoc audit run [--plugin <name>]`. **Frequency:** once per CLI invocation, never implicit. This keeps the kind narrowly defined and prevents `audit` plugins from drifting into ambient policy enforcement, which is a different concern with a different threat model.

### 2. ISO/IEC 29110 is the first runnable reference

ISO/IEC 29110 (Software engineering — Lifecycle profiles for Very Small Entities) is the right first reference plugin for three reasons:

1. **Scope match.** 29110's Basic profile targets organizations of 1–25 people producing software. BWOC's own working unit — one operator plus a small fleet of agents — fits the profile exactly. The other three frameworks (9001, 20000-1, 27001) describe organizational practices (QMS, ITSM, ISMS) that don't reduce cleanly to file-existence checks on a workspace.
2. **Mechanizable evidence.** 29110 Basic deliverables map well to artifacts a BWOC workspace already produces: project plan, requirements spec, design, traceability, test report, configuration management plan. Each can be reduced to a "file exists / contains pattern" check — the simplest possible audit runtime. This is what `BWOC-13` ("full") means: each criterion has a real check, not a stub returning `not_implemented`.
3. **Forcing function for the schema.** Building a real audit against a real standard surfaces schema gaps that paper specs hide. The findings shape from `BWOC-11` will be exercised by 29110 first; the three stubs in Sprint 3 then conform to a schema that has already survived one runnable consumer. This sequencing is the same Yoniso Manasikāra applied at spec scale: verify the schema works before three more plugins depend on it.

The runtime surface stays minimal: each criterion runs a file-existence or content-pattern check, returns a `Finding` (schema below), and the plugin aggregates into a report. No external tooling, no network, no shell-out. The plugin is testable purely by `cargo test` against a fixture workspace.

### 3. 9001, 20000-1, 27001 ship as stubs in Sprint 3; runtime deferred to EPIC-3

The other three frameworks ship as **manifest + SPEC + criteria list** with no runtime (`BWOC-14`, `BWOC-15`, `BWOC-16`). Each criterion is declared in the plugin's `SPEC.md` and enumerated in a `criteria.toml` (shape to be settled inside those stories), but `invoke` returns `{status: "not_implemented", ...}` for every finding, uniformly.

Why stubs, not skip:

- **Discoverability.** Operators running `bwoc plugin list --kind audit` after Sprint 3 see all four frameworks. The absence of 9001 would imply "BWOC has no opinion on QMS"; the stub form says "BWOC has a placeholder, runtime is on the roadmap." That is the honest signal.
- **TH parity is cheap now, expensive later.** Authoring `SPEC.en.md` + `SPEC.th.md` for the three stubs (covered by `BWOC-18`) costs one sprint slot now and would cost a backfill epic later. Bilingual parity is a HARD RULE; landing the TH side at the same time as the EN is the only honest path.
- **Schema validation across kinds.** Three stubs conforming to the `BWOC-11` findings schema with `status = "not_implemented"` exercises the schema's "no evidence available" path on day one. This catches "schema only works when status is pass/fail" regressions before they reach an operator.

Why **not** runtime in EPIC-2:

- 9001, 20000-1, 27001 audit **organizational practices** — management reviews, internal audits, risk registers, asset inventories — that are not present as files in a typical BWOC workspace. Reducing them to file-existence checks would falsify the audit (and BWOC would issue a passing report against a clause it cannot actually evaluate, violating **Musavada** — see [PHILOSOPHY.en.md](https://github.com/) §Sila 5).
- Runtime for these three needs a richer evidence model (operator attestations, time-bounded evidence, sampling) that does not exist in v1 of the schema. Building that is `EPIC-3` work, not EPIC-2.

`BWOC-EPIC-3` (ISO Compliance Plugins — Runtime) is the explicit successor epic. This note is the first place that name is committed; it will land in `.scrum/epics.json` when EPIC-3 is opened. Until then, "deferred to EPIC-3 runtime" is the canonical phrasing in every plugin's `SPEC.md` Status section.

### 4. Findings report schema — what `BWOC-11` must enforce

Every `audit` plugin's `invoke` returns a list of findings. The schema (spec target for `BWOC-11`) MUST enforce these five fields, normatively, with these semantics:

| Field | Type | Required | Semantics |
|---|---|---|---|
| `criterion_id` | string, kebab-case, plugin-scoped | yes | Stable identifier for the criterion being checked. MUST match an entry in the plugin's declared criteria list. Stable across releases — renaming is a breaking change. |
| `severity` | enum: `info`, `low`, `medium`, `high`, `critical` | yes | The criterion's intrinsic severity, declared in the plugin's criteria list, **not** decided per-run. A `critical` finding with `status = "pass"` is normal and means "we checked the most important thing and it's fine." |
| `status` | enum: `pass`, `fail`, `not_applicable`, `not_implemented` | yes | Outcome of this check on this workspace. `not_applicable` is for criteria that don't apply to this workspace's profile (e.g. multi-tenant clause on a solo workspace). `not_implemented` is the stub-plugin status. **Not** a free-text field. |
| `evidence` | structured: `{ kind: "file" \| "content" \| "command" \| "none", value: string }` | yes (`kind` always; `value` required unless `kind = "none"`) | Where the plugin looked. `none` is the honest answer for `not_implemented` and `not_applicable`. `file`/`content`/`command` evidence MUST be reproducible — an operator running the same check by hand finds the same artifact. This is the **Musavada** guard: no claim without a referent. |
| `remedy` | string, plain prose; optional iff `status = "pass"` | conditional | Actionable next step. MUST be present on every `fail`, `not_applicable`, and `not_implemented` finding. "Why this status, and what to do." For `pass`, omit. For `not_implemented`, the remedy is "Runtime deferred to BWOC-EPIC-3" or equivalent — uniform across the three stubs. |

Non-negotiable schema rules:

- **Closed enums, not free strings.** `severity` and `status` are validated at plugin load time and at every `invoke` boundary. An unknown value is a plugin bug, not a finding.
- **No nested findings.** A criterion either passes or fails as a unit. Sub-checks are separate criterion entries with their own `criterion_id`. This keeps the report flat and machine-parseable.
- **Stable serialization order.** Findings serialize in criterion-declaration order (the order they appear in the plugin's criteria list), not check-execution order. This makes diffs across runs meaningful.
- **JSON is the canonical wire format.** `bwoc audit run --json` (per `BWOC-12`) emits one envelope `{ plugin, version, started_at, finished_at, findings: [...] }`. Human-readable output is a renderer over the same shape; the JSON is normative.

The `BWOC-11` spec lands these as a normative table in `PLUGINS.en.md` (new subsection under §Kinds, paired with the `audit` kind row added in `BWOC-10`), and as a parallel table in `PLUGINS.th.md`. The schema is **referenced**, not duplicated, from each plugin's `SPEC.md`.

## Alternatives considered

- **Name the kind `compliance`.** Rejected — overloads to "regulatory" and biases readers away from non-ISO audits (license headers, doc parity). `audit` keeps the kind general.
- **Ship ISO 9001 first, not 29110.** Rejected — 9001 is QMS, almost none of which reduces to file-existence on a workspace. Would force the schema to support attestation evidence on day one, before the simpler file-evidence path is proven.
- **Skip the three stubs, ship only 29110 in EPIC-2.** Rejected — operators running `bwoc plugin list --kind audit` would see only one entry and conclude BWOC has no opinion on QMS/ITSM/ISMS. Stubs are the honest "placeholder, runtime on roadmap" signal, and they exercise the `not_implemented` schema path.
- **Encode `kind` in the plugin directory path (`modules/plugins/audit/iso-29110/`).** Rejected — already settled in Sprint 1 (per the inbox message from `agent-lisa` on 2026-05-26 and the BWOC-3 reconcile). Plugins stay flat at `modules/plugins/<name>/`; `kind` is declared in `manifest.toml` only. The audit kind inherits this layout — `modules/plugins/audit-iso-29110/`, not `modules/plugins/audit/iso-29110/`.
- **Free-text `severity` field.** Rejected — the report must be machine-comparable across runs and across plugins. Free text would mean "high" in one plugin and "High" in another are different severities to a diff tool.

## Status / deferred

- This note: complete. No code or spec edits land in `BWOC-19` itself — it is design framing only.
- `BWOC-10` (audit kind in PLUGINS spec, EN+TH): next in queue.
- `BWOC-11` (findings schema, EN+TH): follows BWOC-10 in the same sprint.
- `BWOC-22` (skill-template + plugin-template): independent of the audit work but next in queue per the Sprint 2 brief; lands before BWOC-10/11 because jennie's BWOC-23/24 is blocked on it.
- `BWOC-EPIC-3` (ISO Compliance Plugins — Runtime): not yet opened in `.scrum/epics.json`. Carries the runtime work for `audit-iso-9001`, `audit-iso-20000-1`, `audit-iso-27001`. Will be opened by the operator at EPIC-2 close.

## Related

- Epic: `BWOC-EPIC-2` (ISO Compliance Plugins — Foundation + ISO/IEC 29110)
- Stories framed by this note: `BWOC-10`, `BWOC-11`, `BWOC-12`, `BWOC-13`, `BWOC-14`, `BWOC-15`, `BWOC-16`, `BWOC-17`, `BWOC-18`
- Spec docs touched downstream: [`PLUGINS.en.md`](../docs/en/PLUGINS.en.md), [`PLUGINS.th.md`](../docs/th/PLUGINS.th.md)
- Sprint 1 retro path-reconcile precedent: `.scrum/retrospectives/sprint-1-retro.md` (flat plugin layout)
- Sprint 2 planning: `.scrum/planning/sprint-2-planning.md`
