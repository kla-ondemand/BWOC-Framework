---
date: 2026-05-23
session: Phase 4 — Aparihāniya-dhamma 7 fleet-governance spec
tags:
  - phase/4
  - type/note
  - module/docs
---

# 2026-05-23 — Phase 4: Fleet-Governance Spec

Fourth significant slice today, after trust step 4–5, dual-mode `bwoc check` / Pi+Oracle personalization, and agent → agent messaging. Phase 4 is structurally different from Phase 1–3 — it's an **ecosystem-viability phase** whose DoD ("three or more reference agents in the wild built by external maintainers"; "BWOC vocabulary observed in unaffiliated codebases") is realized by *external* adoption, not by any sprint we run. The framework's job in Phase 4 is to make adoption *possible* — provide the vocabulary, governance, and reference patterns operators need. This note ships the one Phase 4 line item the framework itself owns: the Aparihāniya-dhamma 7 governance spec.

## What changed

- **`docs/en/FLEET-GOVERNANCE.en.md`** — new framework-root operator-facing spec, 139 lines. Seven conditions from DN 16 §1.4 (the Vajjī teaching the Buddha gave before parinibbāna) mapped to workspace-level fleet operations. Each condition gets: Pali name, traditional gloss, BWOC application, observable signal (existing CLI/git query), and suggested operator practice. v1 is descriptive — signals, not gates.
- **`docs/th/FLEET-GOVERNANCE.th.md`** — 138-line TH parity. Mirrors the EN doc section-by-section.
- **PHILOSOPHY cross-references updated** — both `modules/agent-template/docs/{en,th}/PHILOSOPHY.{en,th}.md` had a dead pointer `docs/FLEET-GOVERNANCE.md`. Now `docs/{lang}/FLEET-GOVERNANCE.{lang}.md` with the date the spec landed.
- **ROADMAP Phase 4 gains "Shipped" subsection** — same in EN and TH. The "Goals (realized by external adoption)" subsection retains the existing four items but is now honest that they're external-adoption outcomes, not sprint deliverables.
- **CHANGELOG entry** — under `[Unreleased]` Added, grouped with the trust + messaging + check work as today's bundle.

## Decisions

- **Spec lives at framework root, not in `modules/agent-template/docs/`.** Considered following the philosophy-doc location pattern, but fleet governance is fundamentally about the *workspace* (multiple agents, registry, shared resources), not about an incarnated agent reading itself. Operator-facing docs at `docs/{lang}/` (ARCHITECTURE, WORKSPACE, ROADMAP) is the more honest home. The PHILOSOPHY.en.md cross-reference got updated to point here.
- **Descriptive v1, not enforcing v1.** Each of the 7 conditions gets a *signal* the framework can already read (existing `bwoc list`, `bwoc doctor`, `bwoc check`, git history) — none get a hard gate. Promoting a signal to a gate is reserved for v2 when we have telemetry showing the rigidity is justified. Premature enforcement would lock in opinions that don't yet have evidence.
- **No new CLI surface.** Considered `bwoc fleet health` as a single roll-up command. Rejected for v1 — the existing primitives compose into the same answer (`bwoc list --json`, `bwoc doctor --auto`, `bwoc check --all`, etc.), and an operator running a fleet at scale will want to wire those into their own monitoring stack rather than depend on a framework-side aggregator. v2 can add `bwoc fleet health` once the operator-facing shape is clearer.
- **Honor the "Phase 4 is external" structure.** The ROADMAP previously listed all four Phase 4 items as flat bullets. After this iter the structure distinguishes "Shipped" (the spec) from "Goals (realized by external adoption)" (the three-year vision items). Pretending external adoption is sprint work would be dishonest.
- **DN 16 §1.4 as the canonical source.** The Buddha's teaching to the Vajjī is the most direct source for "conditions of non-decline applied to a confederacy" — and a workspace of agents is structurally a confederacy. Considered also referencing the Mahāparinibbāna more broadly but kept the citation tight (§1.4 specifically).
- **No PR to the agent template's `AGENTS.md`.** The agent-facing AGENTS.md §10 (Observability) already covers what an agent should *self*-observe; fleet governance is the operator's view. Adding fleet-governance text to AGENTS.md would conflate roles. Cross-reference from PHILOSOPHY is the right level.

## Alternatives considered

- **Ship a third reference agent (e.g., `agent-scribe`) instead of / alongside the spec.** Considered as the more "concrete" Phase 4 work. Rejected for this iter — a third reference agent is useful but doesn't move Phase 4 forward conceptually (we have two already; a third doesn't change the picture much). The governance spec is the bigger lever because it gives every *future* operator (us or external) a coherent vocabulary.
- **Cross-vendor live verification (build same agent on Gemini, Codex, Kimi).** Considered as the verification side of Phase 4. Deferred — `bwoc check`'s neutrality rules already gate the structural pieces; live-running on each backend is a meaningful test but takes hours of session time per backend. Queue for a dedicated session.
- **Aparihāniya-dhamma-as-Rust-checks.** Considered prototyping `bwoc fleet health` as code today. Rejected — the spec needs to settle first; otherwise the implementation would lock in interpretations that the operator community hasn't had a chance to push back on. Spec first, code later when patterns emerge.
- **Renaming "Phase 4" to "Vision Phase" or "Adoption Phase" to make its nature clearer.** Considered. Decided to keep "Phase 4" for continuity with VISION.md and existing references, but the ROADMAP structure now makes the framing explicit ("Goals realized by external adoption").

## Bugs surfaced and fixed

- **PHILOSOPHY.en.md / .th.md pointed at `docs/FLEET-GOVERNANCE.md`** with no language directory — would 404 under the bilingual convention. Fixed both files to point to `docs/{lang}/FLEET-GOVERNANCE.{lang}.md`.
- **Bilingual-parity hook is noisy on intentional pairs.** Editing EN-then-TH (or vice versa) of a known pair fires the parity warning on every Edit, even when both files are being updated in the same conversation. The hook can't tell "Claude is mid-pairing" from "Claude only updated one side." Not a bug per se — the hook's pessimism is correct in general — but I'm tracking it as a notable workflow friction. Could be addressed by the hook short-circuiting if both files' mtimes are within ~60s.

## Status / deferred

- **`bwoc fleet health` as a roll-up command.** v2 territory. Combines the queries from the spec's "Observable Fleet Health" table into a single command with `--json` output for monitoring integration.
- **Schema-version floor in `workspace.toml`.** Spec mentions this as the discipline for condition #3 (process-bound convention change), but the field doesn't exist yet. Add to `bwoc-core::workspace::Workspace` when implementing.
- **Aparihāniya-dhamma 7 dashboard view.** `bwoc dashboard` could grow a fleet-health pane showing the 7 signals at a glance. Phase 2 dashboard infrastructure is in place; this is a Phase 4+ extension.
- **Spec validation against a real multi-agent fleet.** The framework has two agents (Pi, Oracle); the spec is grounded in those examples but hasn't been pressure-tested against a fleet of 5–10 agents. External adoption is where the spec earns its keep — early adopters will find the rough edges.

## Test summary

No code changes this iter — spec-only. Workspace test count unchanged (115 tests). `bwoc check`, `bwoc list`, `bwoc doctor` still clean against the framework workspace.

The two agents (`agents/agent-pi`, `agents/agent-oracle`) provide a 2-agent fleet today; operator can already run all 7 health signals against the framework's own workspace as the first "real" validation of the spec.

## Related

- Spec (this iter): [`docs/en/FLEET-GOVERNANCE.en.md`](../docs/en/FLEET-GOVERNANCE.en.md) · [`docs/th/FLEET-GOVERNANCE.th.md`](../docs/th/FLEET-GOVERNANCE.th.md)
- Philosophy mapping: [`PHILOSOPHY.en.md` #20. Aparihāniya-dhamma 7](../modules/agent-template/docs/en/PHILOSOPHY.en.md)
- Today's earlier slices: [`trust-step-4`](./2026-05-23_trust-step-4.md), [`check-dual-mode-and-personalize`](./2026-05-23_check-dual-mode-and-personalize.md), [`agent-to-agent-messaging`](./2026-05-23_agent-to-agent-messaging.md)
- ROADMAP Phase 4: [`docs/en/ROADMAP.en.md` §Phase 4](../docs/en/ROADMAP.en.md)
- DN 16 §1.4 canonical source: [SuttaCentral DN 16](https://suttacentral.net/dn16)
- Commit: pending (this note ships with it)
