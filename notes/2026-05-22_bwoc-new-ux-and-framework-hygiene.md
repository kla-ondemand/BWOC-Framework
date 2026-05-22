# 2026-05-22 — `bwoc new` UX pass + framework-repo hygiene

Late-session work after the [foundation pass](2026-05-22_phase-1-v20-foundation.md). Three commits tightening `bwoc new` + cleaning up the framework-repo's own gitignore + correcting CLAUDE.md drift. Software-Version `0.1.357 → 0.1.359`; Document-Version `1.0.112 → 1.0.114`.

## What changed

- **`3f96a4e` fix(cli):** `bwoc new` default-target priority re-ordered. Workspace-aware branch now runs FIRST; template-sibling fallback runs only when no workspace exists in any ancestor. Previously, running `bwoc new` from inside the framework root (which is itself a workspace) bypassed `agents/` entirely and registered with a wrong relative path. See commit message for the live verification.
- **`0da8c6f` feat(cli):** `bwoc new` gained three optional prompts at incarnation time — persona scope (`--scope` / `--out-of-scope`), mindsets (`--mindsets a,b,c`), skills (`--skills a,b,c`). All TTY-promptable + flag-overridable + idempotent (no clobber of existing stubs). Manifest gained `scopeDescription` + `outOfScope`; substituted into `AGENTS.md` + `persona/README.md` placeholders. Mindset/skill stubs follow `SPEC.md` shape.
- **`8dd906f` chore(gitignore):** Consolidated two drifted BWOC-runtime sections into one. Fixed two typos (one separator missing `#` prefix → unintended pattern; one "BEOC" → "BWOC"). Inline comment now documents the **opposite-policy** between the framework repo's `.gitignore` and what `bwoc init` writes for user workspaces.
- **CLAUDE.md drift correction (this session, uncommitted at write time):** "What This Repo Is" now acknowledges the Rust workspace + top-level `scripts/`; "Repo State Quirks" replaced — dropped stale claims (`Not yet git init'd`, `SECURITY.md / CODE_OF_CONDUCT.md missing`, `.claude/ empty`), kept only quirks still true (framework-repo-as-workspace gitignore policy, `.github/CODEOWNERS` missing, auto-version hook).

## Decisions

- **`bwoc new` is now workspace-first.** The framework-developer convenience (drop agent next to `modules/agent-template/`) is a fallback for fresh framework clones before `bwoc init` has been run. Once a workspace exists anywhere upstream, it wins. Rationale: registry consistency is more valuable than scaffolding ergonomics.
- **Persona/mindset/skill seeding is opt-in.** Empty input leaves placeholders raw — the pre-existing manual-edit flow still works. New flags are additive; nothing breaks for users who don't pass them.
- **Framework-repo `.gitignore` documents its inversion inline.** Future contributors reading the file will see why `.bwoc/`, `agents/`, `projects/` are ignored at this root but tracked in user-workspace defaults.

## Bugs surfaced and fixed

- `bwoc new` from inside the framework root produced wrong paths AND wrong registry entries (`3f96a4e`). Live workspace already had a stale `agents.toml` entry from a pre-fix invocation — repaired locally; `.bwoc/agents.toml` is gitignored at framework root so the fix doesn't propagate via commit.
- `.gitignore` had an effectively-no-op typo line (74 dashes with no `#` prefix — git interpreted it as a literal pattern with no matches). Harmless but signaled drift (`8dd906f`).
- CLAUDE.md "Repo State Quirks" was telling future Claude sessions the repo had no commits + no remote — both false for ~70 commits now. Misleading for any AI relying on it as ground truth.

## Status / deferred

- `IncarnationReport.print_report` doesn't yet surface the new `mindset_stubs / skill_stubs / persona_filled` fields. Tracked, not blocking — follow-up commit.
- `agents.toml` repair for any other user hitting the pre-fix symptom is manual or via `bwoc retire <name> --keep-files && bwoc new <name>`. No automatic migration shipped.
- `.github/CODEOWNERS` still referenced by `CONTRIBUTING.md` but absent. Not creating one without explicit policy direction (Sīla — community docs need authoring intent, not OSS template defaults).

## Related

- Commits: `3f96a4e`, `0da8c6f`, `8dd906f` (+ pending CLAUDE.md edit)
- Previous session note: [Phase 1 v2.0 Foundation](2026-05-22_phase-1-v20-foundation.md)
- Spec touched: [`docs/en/WORKSPACE.en.md`](../docs/en/WORKSPACE.en.md) (workspace-root resolution behavior)
- Code touched: `crates/bwoc-cli/src/new.rs`, `crates/bwoc-cli/src/main.rs`, `crates/bwoc-core/src/manifest.rs`, `crates/bwoc-agent/src/main.rs`
