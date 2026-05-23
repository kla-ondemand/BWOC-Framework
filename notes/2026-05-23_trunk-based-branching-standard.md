# 2026-05-23 — Trunk-based branching standard

Establishes one branching standard across every BWOC repo: `main` is the only long-lived branch (always releasable); all work happens on short-lived topic branches deleted after merge. Branch `type` reuses the Conventional Commit vocabulary.

## What changed

- `modules/agent-template/AGENTS.md` §4.2 — rewrote Branch Naming: `<type>/{{taskId}}` with `<type>` ∈ `feat fix docs refactor test chore perf style ci`; multi-agent collision guard `agent/{{agentId}}/<type>/{{taskId}}`; dropped `release/*` and `hotfix/*`. Tier-1 doc — stayed placeholder-only (neutrality audit: 0 violations).
- `modules/agent-template/conventions.md` — branch-names section + placeholder table aligned to the same vocabulary.
- `CONTRIBUTING.md` — Development Workflow now states the trunk-based standard and links `conventions.md` as canonical.
- `docs/en/SRS.en.md` + `docs/th/SRS.th.md` — **FR-4.7 rewritten** (bilingual parity) to the new vocabulary. This was the load-bearing fix: the requirement still mandated the old `feature/`/`release/`/`hotfix/` names and would have contradicted the new standard.

## Decisions

- **No `release/*` or `hotfix/*`.** Releases are CalVer tags (`v<YYYY>.<M>.<D>-<patch>`) cut directly on `main`, so a long-lived release branch buys nothing. (Anattā — nothing to cling to between releases.)
- **Branch `type` = commit `type`.** One vocabulary for both keeps the mental model single (Sīlasāmaññatā).

## Bugs surfaced and fixed

Verification before commit caught four stale references the standard's author missed — the change was internally contradictory as staged:
- `AGENTS.md:379` worktree example `-b feature/{{taskId}}` → `feat/`.
- `conventions.md:172` schema comment `# "feature/proj-42"` → `feat/`.
- `SRS.en.md:78` / `SRS.th.md:78` FR-4.7 still listed the entire old vocabulary (incl. `release/{{version}}`, `hotfix/{{taskId}}`).

Lesson: a naming-standard change isn't done until `grep -rn "feature/\|release/\|hotfix/"` is clean across all tiers (instructions, conventions, spec, EN+TH).

## Status / deferred

- Neutrality audit passes; no stale branch refs repo-wide (the only remaining mentions are the prohibition statements themselves).
- The original AGENTS.md/conventions.md/CONTRIBUTING.md edits arrived uncommitted in the working tree from a parallel session; completed and committed here on user direction.
