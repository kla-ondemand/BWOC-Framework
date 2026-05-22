# 2026-05-22 — Phase 1 v2.0 Foundation

Single-session foundation pass: open-source hygiene + bilingual spec layer + Rust workspace scaffold + auto-versioning + CI + over-engineering protection. Software-Version went `0.1.0 → 0.1.4`; Document-Version went `1.0.0 → 1.0.31`.

## What changed

- **Open-source hygiene:** `VISION.md` (+ TH), `SECURITY.md`, `CODE_OF_CONDUCT.md` (BWOC-native, Sīla 5 + Brahmavihāra 4 — not Contributor Covenant), `CHANGELOG.md` (Keep a Changelog 1.1.0), `VERSION.md` with `Software-Version` + `Document-Version` + `Last-Updated`, README badges/TOC/footer.
- **Specification layer (all bilingual EN/TH):** `PHILOSOPHY` §0.1 *The Arc* (uppāda · ṭhiti · vaya from AN 3.47 Saṅkhata Sutta) · `GLOSSARY` · `ARCHITECTURE` · `INCARNATION` · `WORKSPACE` · `NAMING` · `ROADMAP` · `FAQ`. Plus `modules/agent-template/conventions.md` updated to reference NAMING.
- **Rust workspace scaffold:** Cargo workspace at root, edition 2024, MSRV 1.85, three crates (`bwoc-core`, `bwoc-cli`, `bwoc-agent`). `LifecyclePhase { Uppada, Thiti, Vaya }` declared in `bwoc-core::lifecycle`. CLI `--lang` flag with TH + EN locale skeletons (Project Fluent). All gates pass: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo build`, `cargo test`.
- **Claude Code tooling:** 5 project skills (`/incarnate`, `/check-neutrality`, `/check-bilingual`, `/task-log`, `/check-naming`); 2 PostToolUse hooks (`bilingual-reminder`, `auto-version`); CLAUDE.md + CLAUDE.local.md.
- **Continuous integration:** `.github/workflows/ci.yml` — minimal, single-OS (ubuntu-latest), four standard Rust gates. Multi-OS matrix + release pipeline deferred to Phase 2.
- **Over-engineering protection:** new hard-rule section in `CLAUDE.md`; feedback memory persisted across sessions; the 3-minute /doc cron loop that drove 14 iterations was cancelled.

## Decisions (the ones a future contributor needs to know)

- **AGENTS.md stays plain Markdown; everything else is Obsidian-flavored.** Two-tier rule enforced. Backend symlinks (`CLAUDE.md`/`GEMINI.md`/`CODEX.md`/`KIMI.md` → `AGENTS.md`) plus `CLAUDE.md` exception when the agent wants its own Claude-specific guidance.
- **Bilingual = EN canonical + per-language translation.** `docs/<lang>/<NAME>.<lang>.md` inside the template; `FILENAME.md` (EN) + `FILENAME.<lang>.md` (translation) at repo root. CLI strings same pattern at `locales/<lang>/cli.ftl`.
- **CoC is BWOC-native, not Contributor Covenant.** Sīla 5 for prohibitions, Brahmavihāra 4 for expected disposition. Explicitly non-sectarian framing to avoid ความแตกแยกเชิงศาสนา.
- **Software and Document versions are independent.** Auto-bumped on every Claude Code edit; software for `.rs` / `.toml` / `crates/*`, document for `.md`. UTC ISO 8601 Last-Updated stamp.
- **Workspace marker is `.bwoc/`.** Operational commands refuse to run until `.bwoc/workspace.toml` + `.bwoc/agents.toml` validate. Central per-user memory at `~/.bwoc/` parallel to per-agent and per-workspace memory scopes.
- **CLI is a thin orchestrator.** `bwoc spawn` exec's the configured backend's CLI; no embedded LLM client. `bwoc-agent` is a small runtime that ships with each incarnated agent.
- **One workflow, single OS, four gates.** Multi-OS matrix and release pipeline are Phase 2 — explicitly deferred to avoid over-engineering at scaffold time.

## Alternatives considered

- **Generic Contributor Covenant for CoC** → rejected. Imported identity-political framing inconsistent with non-religious technical stance.
- **Auto-bump every commit via git hook** → rejected. Repo isn't `git init`'d yet; would create release cadence tied to commit cadence, not edit cadence.
- **Multi-OS CI matrix at Phase 1** → deferred to Phase 2. Single OS proves the code compiles; multi-OS earns its place when release pipeline lands.
- **Merging CHANGELOG and implementation logs** → kept separate. CHANGELOG is release-oriented ("what shipped"); notes are development-oriented ("how it was built, what was decided"). Both serve.

## Bugs surfaced and fixed mid-session

- `auto-version.sh` BSD sed incompatibility — GNU-only `0,/regex/s||...|` silently failed on macOS Cargo.toml bumps. Replaced with portable `s|^version = "X.Y.Z"$|version = "X.Y.Z"|`.
- `auto-version.sh` path-scoping bug — out-of-repo `.md` edits (e.g., `~/.claude/projects/.../memory/*.md`) wrongly bumped Document-Version. Added early-exit `case "$rel" in /*) exit 0 ;; esac`.
- Stale TH PHILOSOPHY count — `๑๔ ประการ` was wrong; corrected to `๒๒ ประการ` (verified by counting groups A–F in EN).
- Bilingual-reminder hook race condition — when EN/TH are written in parallel tool calls, the EN hook fires before the sibling TH write completes and falsely reports TH missing. Informational only; not fixed.

## Status / deferred

| Area | Status |
|---|---|
| Doc layer | Substantially complete |
| Rust scaffold | Compiles; commands stubbed (`bwoc --lang th` works) |
| CI | Live (single-OS) |
| Auto-version | Live + verified |
| `bwoc check` | ✓ Implemented (full parity with `check-agent-neutrality.sh`; 15 PASS against agent-template; 2 unit tests) |
| `bwoc new` | ✓ Implemented (flags + interactive TTY prompts; non-TTY fail-fast with full missing-field list; live end-to-end with `bwoc check` returns 15 PASS) |
| `bwoc spawn` | ✓ Implemented (minimal exec, `--path` + `--backend` flags + `-- <extra>` passthrough; live-verified launching kimi in agent template) |
| `bwoc init` | ✓ Implemented (creates `.bwoc/workspace.toml` + `.bwoc/agents.toml` + `agents/`; refuses existing without `--force`; live-verified all three scenarios) |
| `bwoc workspace info` | ✓ Implemented (dumps resolved workspace + config + agent count from registry) |
| `bwoc workspace validate` | ✓ Implemented (5 rules from WORKSPACE.en.md; exit 0/2; live-verified 3 degradation scenarios) |
| `bwoc-agent` real runtime | ✓ Implemented (reads `config.manifest.json` of cwd; prints `I am alive: <agentId>` + role + model + fallback + memory + version; exit 2 if not in an agent dir) |
| `bwoc new` workspace auto-registration | ✓ Implemented (ancestor-walks for `.bwoc/`, appends `AgentEntry` to `agents.toml`; `--backend` flag; duplicate-id refusal; best-effort; live-verified inside-workspace and outside-workspace scenarios) |
| `~/.bwoc/` directory creation | ✓ Implemented (Phase 1 min: directory + empty `config.toml` on first CLI invocation; best-effort; cross-platform `$HOME` lookup with no `dirs` dep) |
| `/check-naming` in CI | ✓ Implemented (`.github/workflows/docs.yml`: 3 gates per NAMING §Audit; mindepth-2 fix + `.local` exemption to avoid false positives; NAMING + SKILL kept in sync with CI) |
| bilingual-reminder hook coverage | ✓ Extended — reverse direction for `docs/<lang>/`; root-level `FILENAME.md` ↔ `FILENAME.th.md` (canonical→translation only fires if translation exists; out-of-repo silent) |
| Non-policy issue + PR templates | ✓ `.github/ISSUE_TEMPLATE/{bug_report,feature_request}.md` + `.github/PULL_REQUEST_TEMPLATE.md`. BWOC-specific fields (backend, arc phase, principle alignment); PR checklist mirrors CONTRIBUTING.md + adds CI/parity/manifest-schema gates |
| `bwoc list` | ✓ Implemented (reads workspace `agents.toml`; full WORKSPACE.en.md resolution: explicit > env > ancestor walk > cwd > fail; live-verified 4 scenarios; same logic should be promoted to info/validate later) |
| `--lang` Fluent wiring | ✓ Infrastructure live (i18n module; embedded FTL; bundle_for + t helpers; 4 unit tests). One message proof-of-concept (`default-help-hint`). Full string conversion across other commands is a follow-up. |
| Workspace resolution in info/validate | ✓ Promoted to full WORKSPACE.en.md chain (was cwd-only). Backward compatible; new ancestor-walk from any subdir. |
| Fluent conversion: `bwoc init` | ✓ 7 message keys (EN+TH); `t_with` helper for `$path` interpolation; lang threaded via InitArgs. Caught + fixed Fluent dot-in-identifier gotcha. Known cosmetic: Fluent strips leading WS — minor flatter formatting. |
| Fluent conversion: `bwoc list` | ✓ 5 message keys (empty msg + 4 column labels); `lang` threaded via `ListArgs::into_runtime`. Known cosmetic: Thai `สถานะ` column alignment slightly off (byte-count padding); fix needs `unicode-width` dep. |
| Fluent conversion: `bwoc spawn` | ✓ 1 message key (`spawn-exec-status` with $backend + $path); `lang` threaded via `SpawnArgs::into_runtime`. Live-verified with real codex CLI. |
| Fluent conversion: `bwoc workspace info` | ✓ 9 message keys (header + 7 field labels + agent row); `lang` threaded via `InfoArgs`. Same TH alignment cosmetic as `list`. |
| Fluent conversion: `bwoc workspace validate` | ✓ 5 message keys (header + PASS/FAIL labels + 2 summary lines); `lang` threaded via `ValidateArgs`. Finding descriptions stay English (rule-specific; would balloon .ftl). Live-verified 3 scenarios. |
| Fluent conversion: `bwoc check` | ✓ 9 message keys (header + target + PASS/WARN/FAIL labels + 2 summary + 2 tail). `run()` signature now `(&Path, &str)`. Finding descriptions stay English. Live-verified EN+TH against agent-template. |
| Fluent conversion: `bwoc new` | ✓ 10 message keys (report header + reg status + next-steps + 4 numbered + prompt format). All major fns thread `&FluentBundle`. Symlink lines stay literal. Live-verified EN+TH. |
| Fluent conversion: `bwoc-agent` | ✓ Duplicated i18n module + own locales/ (not yet promoted to bwoc-core, per Mattaññutā). 7 keys (6 liveness + 1 error). `BWOC_LANG`/`LANG`/`en` resolution chain. Live-verified EN + TH from inside an incarnated agent. **All 8 Fluent surfaces complete.** |
| `bwoc-core::workspace` types | ✓ Implemented (Workspace/AgentsRegistry with TOML serde; 3 unit tests) |
| Runtime works from any directory | ✓ `include_dir!` embedded template + `BWOC_TEMPLATE` env + `~/.bwoc/template/` cache + extract-to-tmp fallback. Live-verified `bwoc new` from `/tmp` with no framework in ancestors. |
| Manual major/minor version bumps | ✓ `scripts/bump-version.sh` (patch still auto via hook). Edits via shell to avoid hook re-fire. |
| `scripts/install.sh` upgrade-in-place | ✓ `--force` flag + existing-install detection + version printout. |
| Phase 1 v2.0 ROADMAP cleanup | ✓ EN + TH "Remaining for ship" → "Shipped in Phase 1 v2.0" with ✓ marks. |
| Policy-bearing items (CODEOWNERS, ISSUE_TEMPLATE/config.yml) | 🔒 HELD — needs user policy direction |

> **Phase 1 v2.0 — DoD reached** for every non-HELD item in this table and in `docs/en/ROADMAP.en.md` §"Shipped in Phase 1 v2.0". The earlier "Spec'd, not yet implemented" rows for `bwoc init` / workspace surface, `~/.bwoc/` central memory, and `/check-naming` audit-in-CI were stale — iters 6, 7, 10, 11 implemented them; this is the cleanup. The only outstanding work is the HELD policy items and the user's release-tag decision.

## Related

- [`CHANGELOG.md`](../CHANGELOG.md) `[Unreleased]` for the full file-by-file enumeration.
- [`ROADMAP.en.md`](../docs/en/ROADMAP.en.md) for the phase plan.
- [`docs/en/`](../docs/en/) for every spec doc this session produced.
- `.claude/loop-roadmap.md` for the within-session loop iteration log (iterations 1–14, all closed).

## Addendum — `do to complete` cron loop (post-session)

Started a second cron loop `419aa20f` (every 3 min, prompt `do to complete`) to chip away at deferred items. Each fire does ONE deferred item per over-engineering protection.

### Iter 2 — `bwoc check` implemented (full parity)

Ported `modules/agent-template/scripts/check-agent-neutrality.sh` into `crates/bwoc-cli/src/check.rs`. Structured `AuditReport` (pure data) + `print_report()` (I/O) for testability. Subcommand wired into `main.rs` via clap. New dep: `serde_json` (added to workspace.dependencies; bwoc-cli inherits). Two unit tests pass; live run against `modules/agent-template` returns 15 PASS / 0 violations / 0 warnings. Cargo fmt + clippy --D warnings + build + test all clean. Software-Version 0.1.4 → 0.1.7.

### Iter 1 — modules/* audit (the original addendum)

- **Filled three empty READMEs** (`modules/README.md`, `modules/plugins/README.md`, `modules/skills/README.md`) — each had been a 0-line stub the prior survey missed.
- **Spec'd two empty slots in agent-template** (`mindsets/SPEC.md`, `skills/SPEC.md`) — Obsidian-formatted, mirror `memories/README.md` / `persona/README.md` style. User explicitly chose `SPEC.md` filename over `README.md` to remove the "is this a landing page" ambiguity. The existing `memories/README.md` and `persona/README.md` remain as-is (rename to SPEC.md is a separate Discovered item).
- **Decisions reaffirmed:** `plugins/` and `skills/` are planned framework-level features (not vestigial). Both wait for their first concrete instance before formalizing the loading/invocation contract.
