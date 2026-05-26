# 2026-05-26 — Skills & Plugins standard (initial scaffold)

Specced the two long-planned framework modules together so the "first lands with its spec" rule in both READMEs is satisfied in one scaffold pass. No crate code written this session — only spec skeletons + design rationale + scrum tracking in the workspace.

## What changed

- `docs/en/SKILLS.en.md` + `docs/th/SKILLS.th.md` — new framework spec for `modules/skills/` (nav_order 11).
- `docs/en/PLUGINS.en.md` + `docs/th/PLUGINS.th.md` — new framework spec for `modules/plugins/` (nav_order 12).
- This note — the single session record for all four docs (per the `CLAUDE.md` "one note per session" rule).
- Workspace-side scrum tracking — `BWOC-EPIC-1` plus nine stories (BWOC-1..9) in the parent workspace's `.scrum/`. Lives outside this repo on purpose: scrum is operator-state, the spec docs are the framework contract.

## Decisions

### The skill/plugin line — and why this is two specs, not one

Both modules share a substrate (TOML manifest, neutrality gate, per-workspace opt-in) but split on **who invokes them**:

- **Skill** — invoked by an *agent* during its own operation. Agents *opt in* via their `config.manifest.json`. Examples: worktree discipline, bilingual parity check, task-log audit.
- **Plugin** — loaded by the *framework runtime*. The *workspace operator* opts in via a new `[plugins]` table in `workspace.toml`. Examples: Tier 2 memory backends, additional LLM-backend integrations, workflow integrations.

The split is the line between "what an agent does" and "what the framework offers". One spec would have muddied that line; two specs let each grow on its own contract surface.

### Manifest format = TOML

Consistent with `workspace.toml` and `.bwoc/agents.toml`. JSON is reserved for per-agent runtime state (`config.manifest.json`). Human-authored declarations are TOML throughout.

### Discovery is per-workspace, not central

No global registry. Plugins listed in `workspace.toml [plugins]`. Skill opt-in stays in the per-agent `config.manifest.json`. This matches **Anattā** — the framework already refuses central authority for routing (`routes.toml` is peer-declared); skills and plugins follow the same pattern.

### First reference picks (one each)

- **Skill — `worktree-discipline`** — Phase 3 already shipped `git_worktree` util + the `task-claimed` Saṅgha hook. Codifying behavior that exists is the lowest-risk way to validate the manifest + invocation contract. The skill formalizes `claim_task` / `release_task` against the existing util.
- **Plugin — `memory-tier2-noop`** — A pass-through to Tier 1. Proves the loading mechanism without committing to a vector-store choice. Lines up with the deferred Phase 3 "Tier 2 memory" item (`docs/en/ROADMAP.en.md`) without forcing a decision on the backend.

Both are intentionally boring. The first reference exists to **shake out the spec**, not to deliver new capability.

### nav_order: 11 and 12 (append, don't reshuffle)

The existing run goes 1..10 (ARCHITECTURE → FLEET-GOVERNANCE). Inserting SKILLS/PLUGINS thematically (after HARNESS, before FAQ) would re-number FAQ and FLEET-GOVERNANCE for no reader benefit. Appending preserves stable links. **Mattaññutā.**

## Alternatives considered

- **One unified `MODULES.en.md` spec covering both.** Rejected: the skill/plugin invocation contracts are genuinely different. A unified spec would force every reader to filter for "the half that applies to me".
- **Skip the design note; let the spec docs speak for themselves.** Rejected: the *why* behind the skill/plugin line, the first-reference picks, and the discovery-is-per-workspace stance are decisions that won't survive in the spec body (which describes the *what*). Captured here so future contributors don't relitigate.
- **Spec everything first, defer reference impls to a later phase.** Rejected: both READMEs explicitly say "the first lands together with its spec." Honoring that constraint keeps the spec honest — a manifest format that no implementation has tried is theory.
- **Use JSON manifests instead of TOML.** Rejected: framework-author-facing files are TOML by convention (workspace, agents). Mixing in JSON for this one surface would create inconsistency for no gain.

## Status / deferred

- Spec docs are **skeletons + key decisions**, not finished contracts. Bodies of stories 1–3 will refine the manifest tables and lifecycle hooks once one implementor (probably `agent-lisa`) starts building the reference skill against them.
- Reference implementations (`worktree-discipline` skill, `memory-tier2-noop` plugin) are not in this session — they are stories BWOC-6 and BWOC-7.
- `bwoc skill` and `bwoc plugin` CLI surfaces are stories BWOC-4 and BWOC-5; not scaffolded yet.
- TH bodies in the spec docs are present as **structural mirrors** (same headings, same code blocks, prose translated). They will get a final translation sweep in BWOC-9 once EN content stabilizes.

## Related (links)

- `modules/skills/README.md` — what was planned
- `modules/plugins/README.md` — what was planned
- `modules/agent-template/skills/SPEC.md` — adjacent (per-agent skill slot), unchanged
- `.claude/skills/` — adjacent (Claude Code session skills), unchanged
- `docs/en/ROADMAP.en.md` — Phase 3 deferred items: "Tier 2 memory" backend
- Workspace scrum: `BWOC-EPIC-1`, stories `BWOC-1..9`
