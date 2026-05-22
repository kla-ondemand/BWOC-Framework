# Roadmap

Phase-by-phase plan for BWOC. **Phases** describe implementation milestones; each may span several SemVer releases. See [`VERSION.md`](../../VERSION.md) for the version-vs-phase distinction. See [`VISION.md`](../../VISION.md) for success criteria at 1-year and 3-year horizons.

---

## Current Status

**Active phase:** Phase 1 v2.0 — *uppāda foundation* — in progress.
**Software-Version:** see [`VERSION.md`](../../VERSION.md).
**Document-Version:** see [`VERSION.md`](../../VERSION.md).

---

## Phase 1 v2.0 — uppāda Foundation

**Definition of done:** end-to-end **uppāda** for one backend — incarnate · check · spawn an agent that runs.

### Completed

- Cargo workspace (`bwoc-core`, `bwoc-cli`, `bwoc-agent`) scaffold; edition 2024; MSRV 1.85.
- `VERSION.md` with `Software-Version`, `Document-Version`, and `Last-Updated`; auto-managed by `.claude/hooks/auto-version.sh`.
- Open-source hygiene: `VISION.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `CHANGELOG.md`; root `README.md` with badges, TOC, footer.
- Spec docs (all bilingual EN/TH): `PHILOSOPHY` §0.1 *The Arc*, `GLOSSARY`, `ARCHITECTURE`, `INCARNATION`, `WORKSPACE`, `NAMING`.
- Crate READMEs (`bwoc-core`, `bwoc-cli`, `bwoc-agent`).
- Claude Code tooling: 4 project skills (`/incarnate`, `/check-neutrality`, `/check-bilingual`, `/task-log`); 2 PostToolUse hooks (`bilingual-reminder`, `auto-version`).
- `incarnate.sh` and `check-agent-neutrality.sh` shell scripts in the template (work today; will be ported to Rust).

### In Progress

- `bwoc-cli` command implementations.
- `bwoc-agent` runtime (Phase 1 DoD: "I am alive" reading `config.manifest.json`).
- Workspace structure on disk (`.bwoc/workspace.toml`, `.bwoc/agents.toml`).
- Central memory directory `~/.bwoc/`.

### Shipped in Phase 1 v2.0

All items below are now implemented. The phase's Definition of Done (end-to-end **uppāda** for one backend) is met; only HELD policy items (CODEOWNERS · ISSUE_TEMPLATE/config.yml) and the user's release-tag decision remain.

| Item | Spec | Status |
|---|---|---|
| `bwoc init [path]` | [`WORKSPACE.en.md`](WORKSPACE.en.md#cli-surface) | ✓ |
| `bwoc workspace info` · `validate` | [`WORKSPACE.en.md`](WORKSPACE.en.md#cli-surface) | ✓ |
| `bwoc new <name>` (port of `incarnate.sh`) | [`INCARNATION.en.md`](INCARNATION.en.md) | ✓ |
| `bwoc check [path]` (port of `check-agent-neutrality.sh`) | [`crates/bwoc-cli/README.md`](../../crates/bwoc-cli/README.md) | ✓ |
| `bwoc spawn <name>` (minimal `exec`) | [`ARCHITECTURE.en.md`](ARCHITECTURE.en.md#information-flow--bwoc-spawn-agent-foo) | ✓ |
| `bwoc list` (reads `.bwoc/agents.toml`) | [`WORKSPACE.en.md`](WORKSPACE.en.md) | ✓ |
| `--lang` flag wired to Project Fluent (TH + EN locales) | [`crates/bwoc-cli/README.md`](../../crates/bwoc-cli/README.md) | ✓ all 8 surfaces (init/list/spawn/workspace info/workspace validate/check/new/bwoc-agent) |
| `/check-naming` skill (audit `*.md` against `NAMING.en.md`) | [`NAMING.en.md`](NAMING.en.md#audit) | ✓ + wired into `.github/workflows/docs.yml` |
| Runtime works from any directory | embedded `include_dir!` agent template + `BWOC_TEMPLATE` env + `~/.bwoc/template/` cache | ✓ |
| Manual major/minor version bumps | `scripts/bump-version.sh <level> [--software\|--document\|--both]` | ✓ (patch still auto-bumped by hook) |

---

## Phase 2 — ṭhiti Operations

**Definition of done:** an agent operates with a real control surface; multiple backends are exercised; releases are reproducible.

- `bwoc-agent` control socket — exposes `status`, `log`, `send` to the CLI.
- `bwoc status` · `log` · `send` commands.
- Real process supervision: signal handling, restart-on-crash, health checks.
- Per-workspace memory (`<workspace>/.bwoc/memory/`).
- Cross-backend validation: full uppāda + ṭhiti against Claude, Gemini, Codex, and Kimi CLIs (Samānattatā in practice).
- GitHub Actions release pipeline: matrix build for macOS · Linux · Windows; signed binaries; checksums; GitHub Release.
- Memory mining tooling and pluggable Tier 2 backend interface.

---

## Phase 3 — vaya + Interconnect

**Definition of done:** an agent's life ends cleanly; agents coordinate without a central authority.

- `bwoc stop <name>` — graceful stop with signal escalation.
- `bwoc retire <name>` — full vaya: worktree cleanup, branch release, memory prune, registry removal.
- `bwoc workspace prune` — reclaim orphaned agent entries.
- Inter-agent messaging — Sammā-vācā channel; Sāraṇīyadhamma 6 cordiality rules.
- Trust scoring — Kalyāṇamitta 7 qualities applied to capability declarations and message provenance.
- `.bwoc/interconnect/` per-workspace routing config.
- Tier 2 memory backend reference implementation.

---

## Phase 4 — Reference Agents + Fleet

**Definition of done:** ecosystem viability proven; cross-vendor production fleet governance is achievable.

- Three or more reference agents in the wild, built by maintainers outside the original authors (per [`VISION.md`](../../VISION.md) one-year success).
- Fleet dashboard — Aparihāniya-dhamma 7 governance applied to a real multi-agent installation.
- BWOC vocabulary (Yoniso manasikāra checks, Mattaññutā caps, Sīla baselines, Kalyāṇamitta trust scores) observed in codebases unaffiliated with this project (three-year success).
- Cross-vendor production fleet pattern in use at more than one organization.

---

## Cross-cutting (every phase)

- **Bilingual parity** — every spec doc has EN canonical + TH (and future languages); the bilingual-reminder hook gates this.
- **Backend neutrality** — every CLI feature works against any of the four declared backends; `/check-neutrality` gates this for `AGENTS.md`.
- **Doc-version + software-version stay consistent** — both auto-stamped on every Claude Code edit.
- **Open-source readiness** — every artifact a public contributor needs (CONTRIBUTING, SECURITY, CoC, LICENSE, VERSION, CHANGELOG, VISION, ROADMAP) is current and accurate.

---

## Non-Goals

See [`VISION.md` §Non-Goals](../../VISION.md#non-goals). Summary: BWOC is not a religion, not a runtime/SDK/LLM, not a replacement for DDD / Clean Architecture / SOLID, not vendor-aligned, and not a productivity framework.

---

## See Also

- [`VERSION.md`](../../VERSION.md) — current versions and SemVer policy.
- [`VISION.md`](../../VISION.md) — 1-year and 3-year success criteria.
- [`CHANGELOG.md`](../../CHANGELOG.md) — what shipped, when.
- [`ARCHITECTURE.en.md`](ARCHITECTURE.en.md) — how the components fit.
