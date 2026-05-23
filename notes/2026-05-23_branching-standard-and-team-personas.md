# 2026-05-23 — Single branching standard + team personas/mindsets

Consolidated three divergent branch-naming conventions into one trunk-based / GitHub Flow standard, and defined complementary team roles + character + a per-role mindset for the four live agents (oracle, pi, codx, anti).

## What changed

### Branching standard (framework — committed)
- **`modules/agent-template/AGENTS.md` §4.1/§4.2/§4.4 + §2.2 JSON** — replaced the GitFlow-flavored `feature/`·`release/`·`hotfix/` list with the trunk-based standard: `main` is the only long-lived branch; topic branches `<type>/<slug>` where `type` ∈ Conventional Commit vocabulary (`feat fix docs refactor test chore perf style ci`); multi-agent collision guard `agent/<agent-id>/<type>/<slug>`; no `release/*` or `hotfix/*` (CalVer tags cut directly on `main`); delete branch after merge.
- **`modules/agent-template/conventions.md`** — §Branch Names rewritten to match; `{{branchName}}` example updated `feature/proj-42` → `feat/PROJ-42`.
- **`CONTRIBUTING.md`** — Development Workflow now states the trunk-based principle and points to `conventions.md` as canonical; `feat/fix/docs` already aligned.
- **`docs/en/SRS.en.md` + `docs/th/SRS.th.md`** — normative requirement **FR-4.7** rewritten (bilingual parity) to the trunk-based vocabulary. Missed in the first pass: FR-4.7 still mandated `feature/`·`release/{{version}}`·`hotfix/` and contradicted the new standard. Also swept two stale examples — `AGENTS.md:379` worktree cmd and `conventions.md:172` schema comment — from `feature/` → `feat/`. Verified clean via `grep -rn "feature/\|release/\|hotfix/"` across all tiers (instructions / conventions / spec / EN+TH). Lesson: a naming-standard change isn't done until that grep is clean everywhere.

### Team personas (live agents under `agents/` — gitignored, operational)
Resolved the pi/codx role collision (both were `software engineer`) into four non-overlapping roles covering the dev lifecycle:

| Agent | Backend | Role | Character | Mindset (principle) |
|---|---|---|---|---|
| oracle | claude/opus | orchestrator & architect | calm, big-picture, decisive | `sense-the-context` (Sappurisadhamma 7) |
| pi | claude/opus | core-systems engineer | meticulous, precise, evidence-driven | `trace-to-root` (Paṭiccasamuppāda) |
| codx | codex/gpt-5.4 | QA & test engineer | polite, careful, quality-skeptical | `prove-it-green` (Padhāna 4) |
| anti | agy/gemini-flash | analyst & researcher | polite, curious, fast, broad | `fact-before-assumption` (Yoniso Manasikāra) |

Updated three surfaces per agent: `config.manifest.json` (agentRole + scope/voice), `persona/README.md` (Role, Primary Role, Scope), `AGENTS.md` §1.1, plus one new `mindsets/*.md` each. The same trunk-based branching was synced into each agent's own `AGENTS.md`.

## Decisions
- **Branch `type` = Conventional Commit vocabulary** rather than the older `feature/` long form — one vocabulary shared with commit types (Sīla coherence), lower cognitive load.
- **No long-lived `release/*` / `hotfix/*`** — clashes with Anattā (non-clinging to branches) and Mattaññutā; CalVer tagging on `main` already in use since `v2026.5.23-1`.
- **Canonical home = template `AGENTS.md` + `conventions.md`**; `CONTRIBUTING.md` references it. Existing agents are clones that re-inherit on sync.
- **Preserved the existing Thai polite-voice persona** for codx and anti; added a team *function* and *mindset* rather than overwriting character (kept the user's intent).

## Status / deferred
- All four live agent clones were synced to the trunk-based standard across their own `AGENTS.md`, `conventions.md`, and `docs/{en,th}/SRS` (the standard was duplicated per-clone). Bilingual EN/TH parity preserved. Done via two background subagents.
- **Docs/spec ownership** was the one lifecycle gap (neither oracle nor pi writes docs). Resolved by assigning it to **anti** (`analyst & docs steward`) rather than adding a 5th agent — Mattaññutā, default not to add.
- **`agent-anti` is the least-incarnated** — its `AGENTS.md` still carries non-persona placeholders (`{{moduleName}}`, `{{primaryModel}}`, `{{fallbackModel}}`, `{{lintCmd}}`, …). Persona/role/docs/branching are done; full incarnation (so it passes `bwoc check` incarnation-mode) is deferred.

## Related
- Branching model + scope chosen interactively (trunk-based; all BWOC repos).
- Prior CalVer release: `v2026.5.23-1`.
