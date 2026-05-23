# Changelog

All notable changes to BWOC are documented in this file.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning 2.0.0](https://semver.org/). See [`VERSION.md`](VERSION.md) for the current version and phase status.

## [Unreleased]

Phase 3 work in progress beyond the v2026.5.23-1 release. Items shipped here will land in the next CalVer tag.

## [v2026.5.23-1] — 2026-05-23

### Fixed

- **Release workflow race condition** — five parallel matrix jobs each called `softprops/action-gh-release@v2` with create-or-update semantics; one created the release first, then the next-arriving job raced and failed with "Validation Failed: already_exists". Refactored into one `create-release` job (`gh release create --generate-notes`) + per-target matrix jobs that only `gh release upload --clobber`. `v2026.5.23-1` shipped all 10 assets (5 binaries + 5 sha256) on the first run, no rerun needed.

## [v2026.5.23-0] — 2026-05-23

First public release of the BWOC framework. CalVer scheme: `v<YYYY>.<M>.<D>-<patch>`.

### Added

Everything documented under the prior `[Unreleased]` "Phase 1 v2.0 work in progress" rollup is included in this release. Highlights:

**Open-source project hygiene**

- `VISION.md` + `VISION.th.md` — project purpose, the arc BWOC models, success criteria, non-goals, tradeoff principles. Bilingual (EN canonical, TH translation).
- `SECURITY.md` — coordinated disclosure process; scope; links to the existing threat model.
- `CODE_OF_CONDUCT.md` — BWOC-native (Sīla 5 prohibitions + Brahmavihāra 4 dispositions); explicitly non-sectarian.
- `VERSION.md` — current version mirror, source-of-truth pointer to `Cargo.toml`, SemVer policy, phase-vs-version distinction.
- Root `README.md` Tech Stack section, badges (License · Rust · platforms · languages · status), table of contents, and footer (Contributing · Security · CoC · License).

### Added

**Open-source project hygiene**

- `VISION.md` + `VISION.th.md` — project purpose, the arc BWOC models, success criteria, non-goals, tradeoff principles. Bilingual (EN canonical, TH translation).
- `SECURITY.md` — coordinated disclosure process; scope; links to the existing threat model.
- `CODE_OF_CONDUCT.md` — BWOC-native (Sīla 5 prohibitions + Brahmavihāra 4 dispositions); explicitly non-sectarian.
- `VERSION.md` — current version mirror, source-of-truth pointer to `Cargo.toml`, SemVer policy, phase-vs-version distinction.
- Root `README.md` Tech Stack section, badges (License · Rust · platforms · languages · status), table of contents, and footer (Contributing · Security · CoC · License).

**Specification**

- `PHILOSOPHY.en.md` + `PHILOSOPHY.th.md` §0.1 *"The Arc"* — establishes **uppāda · ṭhiti · vaya** (AN 3.47 Saṅkhata Sutta) as the architectural shape underlying all 22 frameworks.

**Implementation — Phase 1 v2.0 (Rust)**

- Cargo workspace at the repo root: edition 2024, resolver 3, MSRV 1.85.
- `crates/bwoc-core` — shared types; declares `LifecyclePhase { Uppada, Thiti, Vaya }`.
- `crates/bwoc-cli` — `bwoc` binary with `--lang` flag (precedence: `--lang` flag > `BWOC_LANG` env > `$LANG` env > `en` fallback) and clap subcommand surface.
- `crates/bwoc-cli` — **`bwoc check [path]`** implemented. Full feature parity with `modules/agent-template/scripts/check-agent-neutrality.sh`: AGENTS.md existence, backend symlink validation (GEMINI/CODEX/KIMI → AGENTS.md), CLAUDE.md handling (symlink or standalone), `config.manifest.json` JSON validation, required placeholders, no YAML frontmatter, no wikilinks, no hardcoded model IDs/tool names, no backend-specific phrasing. Read-only; exit 0 = pass, 1 = violations. Pure-data `audit()` + `print_report()` for testability; two unit tests cover wikilink detection and missing-target case.
- `crates/bwoc-cli` — **`bwoc new <name> --role ... --primary-model ... --lint-cmd ... --format-cmd ... --test-cmd ... --build-cmd ...`** implemented. Ports `incarnate.sh` plus the manifest-input spec from `INCARNATION.en.md` §"Setting the Manifest". Recursively copies template (skips `.git/`, `*.example.*`), creates backend symlinks (Unix only; Windows deferred), writes a flat resolved manifest. Kebab-case name validation. Refuses if target exists. Auto-detects template by walking up cwd ancestors. Live end-to-end verified: `bwoc new` then `bwoc check` returns 15 PASS / 0 violations.
- `crates/bwoc-cli` — **`bwoc new` interactive TTY prompts** for missing required fields. Uses `std::io::IsTerminal` (no new dep). On TTY: prompts each missing field with `{key} ({description}): ` where description comes from the template's `config.manifest.json` `requiredConfig.<field>.description`. On non-TTY: collects ALL missing fields in one pass and fails fast with exit code 2 and a comma-separated list — no partial blocking on stdin in CI. Empty prompt response is treated as missing. Two new unit tests cover the fail-fast path and template-description loading.
- `crates/bwoc-cli` — **`bwoc spawn [--path <agent>] [--backend <claude\|gemini\|codex\|kimi>] [-- <args>...]`** implemented. Validates the path is a BWOC agent (has `AGENTS.md`), then exec's the backend CLI in the agent's directory via `std::process::Command::status()` (cross-platform; propagates exit code). Default backend is `claude`. Backend-not-found returns actionable "backend CLI 'X' not found on PATH" error. Extra args after `--` pass verbatim to the backend. Four new unit tests cover backend CLI mapping, missing-path rejection, non-agent-dir rejection, and template acceptance. Live verification: `bwoc spawn --path modules/agent-template --backend kimi` successfully launched Kimi Code CLI in the agent directory.

**Phase 1 v2.0 uppāda surface — DoD reached**

The three-command uppāda arc (`bwoc new` → `bwoc check` → `bwoc spawn`) now works end-to-end via the Rust CLI without any shell-script invocation. Software-Version 0.1.21.

- `bwoc-core::workspace::{Workspace, WorkspaceMeta, WorkspaceDefaults, AgentsRegistry, AgentEntry}` — types for `.bwoc/workspace.toml` and `.bwoc/agents.toml` with TOML serde + load/save. New workspace-level dep: `toml = "0.9"`. Three unit tests cover workspace roundtrip, empty agents.toml, and agents-with-entries roundtrip.
- `crates/bwoc-cli` — **`bwoc init [path] [--force]`** implemented. Creates `.bwoc/workspace.toml` (name auto-derived from directory; version `0.1.0`; created stamp UTC ISO 8601) + `.bwoc/agents.toml` (empty registry with a comment header) + the `agents/` directory (per `agents_dir` default). Refuses if `workspace.toml` already exists; `--force` overrides. UTC ISO 8601 stamp computed from `SystemTime` + a small proleptic-Gregorian conversion to avoid pulling in `chrono`/`time`. Four new unit tests cover creation, idempotency refusal, force-overwrite, and date-format anchors (epoch boundaries + 2024 leap day).
- `crates/bwoc-cli` — **`bwoc workspace info [path]`** + **`bwoc workspace validate [path]`** implemented. `info` dumps resolved workspace path, config (name/version/created/defaults), and agent count + per-agent rows from `agents.toml`. `validate` runs the 5 rules from `WORKSPACE.en.md` §"Validation Rules" — `.bwoc/` exists; `workspace.toml` parses + has required `name`/`created` fields; `version` is parseable SemVer (strict X.Y.Z); `agents.toml` parses; `agents_dir` exists — and exits 0 (complete) or 2 (violations). Short-circuits early on structural failures (missing `.bwoc/`, malformed `workspace.toml`). Pure-data `validate()` + `print_validation_report()` for testability; 4 new unit tests cover SemVer validation, missing `.bwoc/`, clean workspace, and bad SemVer. Live-verified against `bwoc init`'d workspace: 7 PASS / 0 violations; degraded scenario (deleted `agents/`) yields 6 PASS / 1 FAIL with the missing-dir message.
- `crates/bwoc-agent` — **real runtime, no longer a stub.** Reads `config.manifest.json` from the current directory and prints structured liveness with the agent identity (`I am alive: <agentId>` + role + model + fallback + memory + version). Exit 0 on success; exit 2 if cwd is not an incarnated agent (missing `config.manifest.json`) with an actionable message; exit 1 on manifest parse failure. Pure-data `liveness_banner(&Manifest) -> String` separated from `main` for unit testability; 2 new unit tests cover required-fields presence and optional-fallback omission. Live-verified inside an incarnated agent directory: prints all six lines correctly; non-agent dir gives "no config.manifest.json in <path>" and exits 2.
- `crates/bwoc-cli` — **`bwoc new` auto-registers the new agent in the enclosing workspace's `.bwoc/agents.toml`** when one is found. Walks ancestors from `target.parent()` for `.bwoc/workspace.toml`; if found, appends an `AgentEntry { id, path (relative to workspace root), backend, incarnated (UTC ISO 8601), status: "active" }` to the registry. New `--backend` flag (defaults `claude`) records which LLM backend the agent runs against. Best-effort: registration failures log a warning but do NOT fail the incarnation (the agent files are already valid on disk). Refuses to register a duplicate agent_id (`NewError::DuplicateRegistration` — user must `bwoc retire` first). Outside any workspace, the report says "No workspace found in ancestors — agent not registered in any agents.toml". 1 new unit test for ancestor-walk. Live-verified both scenarios.
- `crates/bwoc-cli/src/util.rs` — extracted shared `utc_now_iso8601()` + `format_iso8601(secs)` helpers (previously in `init.rs`), now consumed by both `init` and `new`. 1 unit test covers the same 4 epoch-anchor fixtures.
- `crates/bwoc-cli/src/user_home.rs` — Phase 1 minimum `~/.bwoc/` bootstrap per `WORKSPACE.en.md` §"Central Memory". `ensure_initialized()` creates `~/.bwoc/` + an empty `config.toml` (with a header pointing at the spec) if missing; idempotent and cheap when they exist. Cross-platform home-dir lookup via `$HOME` (Unix) / `%USERPROFILE%` (Windows), no `dirs` crate dep. Called from `main` at startup as best-effort — failure logs a warning but does not block commands. Memory/, workspaces.toml, logs/ are deferred to the commands that need them (Mattaññutā — don't create speculatively). 2 unit tests cover creation + idempotency-without-overwrite. Live-verified: `HOME=/tmp/fake-home bwoc` creates `.bwoc/config.toml` from scratch; `env -u HOME bwoc` prints the warning and still runs.
- `crates/bwoc-core` — **`manifest::Manifest`** type with serde camelCase keys (`agentId`, `primaryModel`, `lintCmd`, ...), `load_from_path` + `save_to_path`, `ManifestError` (thiserror) for IO + JSON failures. Two unit tests cover JSON roundtrip and camelCase serialization with `skip_serializing_if` for None options.
- `scripts/install.sh` — one-command install of the `bwoc` CLI (`./scripts/install.sh` runs `cargo install --path crates/bwoc-cli --locked` with toolchain check + PATH hint).
- `crates/bwoc-agent` — minimal "I am alive" runtime stub shipped with each incarnated agent.
- `crates/bwoc-cli/locales/{en,th}/cli.ftl` — Project Fluent locale skeletons; **TH and EN ship at launch**; any future language is a folder drop.

**Crate-level documentation**

- `crates/bwoc-core/README.md` — pure-data scope, `LifecyclePhase` arc surfacing in code.
- `crates/bwoc-cli/README.md` — install, `--lang` precedence example, command surface table organized by arc phase.
- `crates/bwoc-agent/README.md` — phase-scoped responsibility table (Phase 1 = liveness only; Phase 2 = task loop + socket; Phase 3 = interconnect + vaya).

**Framework reference**

- `docs/en/GLOSSARY.en.md` + `docs/th/GLOSSARY.th.md` — single alphabetized lookup table of every Pali term in BWOC with one-line engineering meaning. Bilingual. Designed so non-Buddhist newcomers can read framework code/specs without learning all 22 frameworks first.
- `docs/en/ARCHITECTURE.en.md` + `docs/th/ARCHITECTURE.th.md` — implementation stack (framework → template → agent → CLI → runtime), `bwoc spawn` information flow, backend-neutrality mechanism, multilingual structure across docs / root metadata / CLI locales, and trust boundary table cross-referencing `THREAT-MODEL`. Distinct from the conceptual stack in `PHILOSOPHY` and `README`.
- `docs/en/INCARNATION.en.md` + `docs/th/INCARNATION.th.md` — canonical step-by-step "how to create a new agent" doc consolidating content previously scattered across `incarnate.sh` comments, root README, and `modules/agent-template/README.md`. Covers prerequisites, six-step walkthrough, adding a backend, multilingual setup, verification checklist, and post-incarnation reading path. **Extended with**: "Setting the Manifest" section spec'ing that `bwoc new` accepts manifest fields via flags + interactive TTY prompts (non-TTY = fail-fast), driven by the `requiredConfig` schema in `config.manifest.json`; "Editing the Manifest After Incarnation" specifies direct file edit as canonical with `bwoc manifest set/get` deferred to Phase 2.

**Continuous integration**

- `.github/workflows/ci.yml` — minimal CI on ubuntu-latest: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`. Single-OS by intent (multi-OS matrix + release pipeline are Phase 2). Scaffold passes all four gates locally before CI is wired.
- `.github/workflows/docs.yml` — runs the `*.md` naming audit on every PR/push that touches markdown. Three gates per `docs/en/NAMING.en.md` §Audit: (A) root-level files must match `UPPERCASE.md`, `UPPERCASE.<lang>.md`, or the Claude Code convention `CLAUDE.local.md`; (B) files inside `docs/<lang>/` and `modules/agent-template/docs/<lang>/` (mindepth 2) must match `UPPERCASE.<lang>.md`, with slot READMEs exempt; (C) anything under `*/notes/` must match `YYYY-MM-DD_<title>.md`. Emits `::error::` GitHub annotations on violations and exits non-zero. Audit greps refined this iter (allow `.local` suffix at root; `mindepth 2` to skip the docs/ root which holds slot-level examples). `NAMING.en.md` + `NAMING.th.md` + `.claude/skills/check-naming/SKILL.md` updated to keep the documented greps identical to what CI runs.

**Workspace resolution promoted to `workspace info` / `validate`**

- `crates/bwoc-cli/src/workspace.rs` — `run_info` and `run_validate` now use the full WORKSPACE.en.md resolution chain (`find_workspace_root`: explicit path → `BWOC_WORKSPACE` env → ancestor walk → cwd → exit 2). Previously they used cwd-only fallback. Backward compatible — passing an explicit path still works. New behavior: running `bwoc workspace info` or `bwoc workspace validate` from any subdir of a workspace now finds the workspace (no need to cd to root). Non-workspace dirs get the same actionable "no workspace found ... pass a path, set BWOC_WORKSPACE, or run `bwoc init` first" message as `bwoc list`. Dropped the now-unused `resolve_root` helper. Live-verified 4 scenarios: info from subdir, validate from subdir, info from non-workspace dir (exit 2), info with explicit path.

**Phase 1 v2.0 — DoD reached**

`docs/en/ROADMAP.en.md` and `docs/th/ROADMAP.th.md` "Remaining for ship" tables renamed to "Shipped in Phase 1 v2.0" — all 8 spec'd items + 2 follow-on capabilities (runtime-works-anywhere via embedded template; manual major/minor SemVer bumps) now ✓. Stale "Spec'd, not yet implemented" rows in `notes/2026-05-22_phase-1-v20-foundation.md` cleaned up (iters 6, 7, 10, 11 had implemented them; the notes hadn't been refreshed). Only outstanding Phase 1 work: HELD policy items (CODEOWNERS, ISSUE_TEMPLATE/config.yml) and the user's release-tag decision.

**Runtime: works from any directory**

- `crates/bwoc-cli/src/new.rs` — agent template now **embedded into the binary at compile time** via `include_dir!("$CARGO_MANIFEST_DIR/../../modules/agent-template")`. `resolve_template` chain: `--template <path>` → `$BWOC_TEMPLATE` env → ancestor walk for `modules/agent-template/` → `~/.bwoc/template/` cache → **embedded fallback** (extracted to a pid-tagged tmp dir per invocation). Closes the "bwoc new must be run from inside the framework" UX wart.
- `default_target` updated to mirror the resolution: framework-dev path keeps "drop next to template"; everywhere else defaults to `cwd/agent-<name>` (was previously `template.parent().parent()/agent-<name>` which resolved to `/agent-<name>` when template was a tmp dir).
- `crates/bwoc-cli/Cargo.toml` + workspace `Cargo.toml` — add `include_dir = "0.7"` (1 new transitive dep `include_dir_macros`).
- Live-verified by running `bwoc new busaba ...` from `/tmp/learn-workspace-test/` (no framework in ancestors, no `~/.bwoc/template/` cache) → agent created cleanly with AGENTS.md + the four backend symlinks.

**Version bumping**

- `scripts/bump-version.sh <major|minor|patch> [--software|--document|--both]` — manual SemVer bumps for major/minor (patch is still auto-bumped on every Claude Code edit by `.claude/hooks/auto-version.sh`). Computes the new version, writes back to `Cargo.toml` (Software-Version, canonical) and `VERSION.md` (Software-Version mirror + Document-Version), and refreshes the `Last-Updated` UTC ISO 8601 stamp. Edits via shell — not Claude tools — so the auto-version hook doesn't re-fire and bump on top. Smoke-tested across all 3 levels × 3 targets.

**Installer upgrade**

- `scripts/install.sh` — adds `--force` to `cargo install` so re-running the script **upgrades in place** instead of erroring with "already installed". Detects existing install + phrases the message as "Upgrading bwoc in place (was: X.Y.Z)" vs first-install "Installing"; prints the new version after install. Comment header documents the embedded-template behavior + cross-references `bump-version.sh`.

**Fluent string conversion — `bwoc-agent`**

- `crates/bwoc-agent/src/i18n.rs` — new module (duplicated from `bwoc-cli/src/i18n.rs`, intentionally not extracted to bwoc-core yet — see file header). `bundle_for(lang)`, `t`, `t_with`, plus `resolve_lang()` matching bwoc-cli's chain (`BWOC_LANG` → `LANG` → `en`).
- `crates/bwoc-agent/locales/{en,th}/agent.ftl` — 7 keys: 6 liveness lines (alive, role, model, fallback, memory, version) + 1 missing-manifest error.
- `crates/bwoc-agent/Cargo.toml` — adds `fluent-bundle` + `unic-langid` from workspace deps.
- `crates/bwoc-agent/src/main.rs` — `liveness_banner(&Manifest, &FluentBundle)`; missing-manifest error path also localized. Parse-error path stays English.
- TH translation: "I am alive" → "ฉันยังมีชีวิตอยู่"; labels like "role:"/"model:" stay English (programmer-standard technical terms). 4 new i18n unit tests + 3 banner tests (was 2 — now 7 in bwoc-agent).
- Live-verified: from inside an incarnated agent dir, `bwoc-agent` prints EN banner; `BWOC_LANG=th bwoc-agent` prints TH banner.

**Phase 1 v2.0 Fluent conversion — COMPLETE across all 8 CLI/agent surfaces** (init · list · spawn · workspace info · workspace validate · check · new · bwoc-agent).

**Fluent string conversion — `bwoc new`**

- `crates/bwoc-cli/locales/{en,th}/cli.ftl` — 10 new `new-*` keys: report header lines (incarnated agent + target), workspace-registration status (registered with `$path` / not-registered), next-steps header + 4 numbered steps (cd & check, edit AGENTS.md, edit persona, git commit), and the interactive prompt format (`new-prompt-format` with `$key` + `$desc`). TH: "Incarnated agent" → "สร้าง agent"; "Target" → "เป้าหมาย"; "Next steps" → "ขั้นต่อไป"; "ตรวจสอบ neutrality" for the check sub-step, etc.
- `crates/bwoc-cli/src/new.rs` — `run()` / `incarnate()` / `resolve()` / `resolve_one()` / `print_report()` all now take or thread a `&FluentBundle<FluentResource>`. The interactive prompt format uses `new-prompt-format` instead of the hardcoded `"{key} ({desc}): "` template. Symlink lines stay literal (data, not labels). Error path stays English.
- `crates/bwoc-cli/src/main.rs` — `NewArgs::into_runtime(lang)` symmetric with init/list/spawn.
- Mid-iter fixes: missing `use crate::i18n;` import in new.rs (cascaded into 11 errors); two unit tests updated to pass `lang: "en"` in fixture args and `&bundle` into `resolve()`.
- Live-verified EN ("Incarnated agent: agent-alphaen / Target: ... / Next steps: ...") and TH ("สร้าง agent: agent-alphath / เป้าหมาย: ... / ขั้นต่อไป: ..."). 34 tests pass.

**Fluent string conversion — `bwoc check`**

- `crates/bwoc-cli/locales/{en,th}/cli.ftl` — 9 new `check-*` keys: header, target (with `$target`), 3 status labels (PASS/WARN/FAIL), success summary (with `$warnings`) + its tail line, failure summary (with `$violations`+`$warnings`) + its tail line. TH: `PASS`→`ผ่าน`, `WARN`→`เตือน`, `FAIL`→`ไม่ผ่าน`; "Neutrality check passed." → "การตรวจสอบ neutrality ผ่าน".
- `crates/bwoc-cli/src/check.rs` — `print_report()` now takes a `&FluentBundle<FluentResource>` and renders the header/labels/summary through `i18n::t`/`t_with`. `run()` signature changed to `run(target: &Path, lang: &str)` to thread the language. Finding descriptions (~10 rule-specific lines like "AGENTS.md contains {{agentId}}") stay English — translating those would balloon the .ftl by 15-20 keys for marginal benefit.
- `crates/bwoc-cli/src/main.rs` — Check dispatch passes resolved `lang` into `check::run`.
- Live-verified against `modules/agent-template`: EN ("Target: ..." / "PASS  ..." / "0 violations, 0 warning(s) / Neutrality check passed.") and TH ("เป้าหมาย: ..." / "ผ่าน  ..." / "0 ละเมิด, 0 คำเตือน / การตรวจสอบ neutrality ผ่าน"). 34 tests pass.

**Fluent string conversion — `bwoc workspace validate`**

- `crates/bwoc-cli/locales/{en,th}/cli.ftl` — 5 new keys: `validate-header` (with `$path`), `validate-label-pass`, `validate-label-fail`, `validate-summary-success` (with `$passes`), `validate-summary-failure` (with `$passes` + `$violations`). TH: `PASS` → `ผ่าน`, `FAIL` → `ไม่ผ่าน`, summary phrasings translated.
- `crates/bwoc-cli/src/workspace.rs` — `print_validation_report()` now takes the bundle and renders header + per-finding pass/fail prefix + summary through `i18n::t`/`t_with`. `run_validate` builds the bundle from `args.lang`. Finding descriptions (".bwoc/ exists", "workspace.toml parses", etc.) stay in English — translating ~10 rule-specific strings would balloon the .ftl file; deferred unless requested.
- `crates/bwoc-cli/src/main.rs` — `ValidateArgs.lang` plumbed; dispatch passes the resolved lang in.
- Live-verified 3 scenarios: EN happy (`7 pass(es), 0 violation(s) — workspace is complete.`), TH happy (`7 ผ่าน, 0 ละเมิด — workspace ครบถ้วน`), TH degraded with deleted `agents/` (`6 ผ่าน, 1 ละเมิด — แก้ก่อนใช้งาน workspace นี้`, exit 2).

**Fluent string conversion — `bwoc workspace info`**

- `crates/bwoc-cli/locales/{en,th}/cli.ftl` — 9 new keys: `info-header` (with `$path`), 7 `info-label-*` field labels (name/version/created/backend/lang/agents-dir/agents), and `info-agent-row` (with `$id`, `$status`, `$path`).
- `crates/bwoc-cli/src/workspace.rs` — `info()` now takes a `&FluentBundle<FluentResource>` and renders header + each labeled field + per-agent rows through `i18n::t`/`t_with`. `run_info` builds the bundle from `args.lang`. Error path stays English.
- `crates/bwoc-cli/src/main.rs` — `InfoArgs` now carries `lang`; dispatch passes the resolved `lang` in.
- **Known cosmetic** (carried over from iter 18): the labels were originally hardcoded literals, so the fixed-position colon alignment worked. Now labels vary by language (`name` vs `ชื่อ`, `version` vs `เวอร์ชัน`) and have different byte widths, so alignment is uneven. Acceptable for readability; a proper fix needs Unicode-width-aware padding (`unicode-width` crate or similar).
- Live-verified EN ("Workspace: /tmp/infoi18n / name: infoi18n / version: 0.1.0 / ...") and TH ("Workspace: /tmp/infoi18n / ชื่อ: infoi18n / เวอร์ชัน: 0.1.0 / สร้างเมื่อ: ... / agent: 1").

**Fluent string conversion — `bwoc spawn`**

- `crates/bwoc-cli/locales/{en,th}/cli.ftl` — 1 new `spawn-exec-status` message key with `$backend` and `$path` args. TH uses Thai preposition `ใน` ("in").
- `crates/bwoc-cli/src/spawn.rs` — `spawn()` builds its own bundle and emits the exec-status line via `i18n::t_with`. Error path (BackendNotFound, PathMissing, NotAnAgent, Io) stays English.
- `crates/bwoc-cli/src/main.rs` — `SpawnArgs::into_runtime(lang)` symmetric with init + list.
- Live-verified by spawning the real `codex` CLI in `modules/agent-template` from both EN and TH locales; status line correctly interpolates backend name + path.

**Fluent string conversion — `bwoc list`**

- `crates/bwoc-cli/locales/{en,th}/cli.ftl` — 5 new `list-*` message keys: `list-empty` (with `$path` arg), `list-col-id`, `list-col-status`, `list-col-backend`, `list-col-path`. TH translates `STATUS` → `สถานะ`; the other column labels stay as English ASCII (`ID`/`Backend`/`Path`) since they're programmer-standard terms.
- `crates/bwoc-cli/src/workspace.rs` — `run_list` now drives the success path through `i18n::t` / `t_with`. Error path stays English (same rule as `init`).
- `crates/bwoc-cli/src/main.rs` — `ListArgs` threads `lang` to runtime via `into_runtime(lang)`. Symmetric with `InitArgs`.
- **Known cosmetic**: Rust's `{:<10}` pads by byte count not visual width, so the Thai `สถานะ` column header is slightly off-alignment. Acceptable for now; fixing would require pulling in `unicode-width` and a width-aware formatter (deferred — not blocking readability).
- Live-verified 4 scenarios: EN empty, TH empty, EN populated, TH populated.

**Fluent string conversion — `bwoc init`**

- `crates/bwoc-cli/src/i18n.rs` — added `t_with(bundle, key, &[(name, value)])` for named-arg interpolation. The slice-of-tuples shape keeps call sites ergonomic without exposing `FluentArgs` directly. 1 new unit test (`t_with_interpolates_named_args`).
- `crates/bwoc-cli/locales/{en,th}/cli.ftl` — added 7 `init-*` message keys (success title, three created-file lines, next-steps header, two next-step suggestions). **Fluent gotcha caught**: `.` is not allowed in identifier names, so keys use `init-success-title` style, not `init.success-title`. First attempt panicked at runtime ("ExpectedToken('=')"); fixed by renaming and updating callers.
- `crates/bwoc-cli/src/init.rs` — `run()` now drives the success-path output through `i18n::t` / `t_with` with `lang` threaded down via `InitArgs`. Error path remains in English (`thiserror` localization deferred).
- `crates/bwoc-cli/src/main.rs` — passes the resolved `lang` into `init::InitArgs`.
- **Known cosmetic regression**: Fluent strips leading whitespace from single-line message values, so the `"  + "` indentation in the pre-Fluent `bwoc init` output is gone (output still reads cleanly). Restorable with Fluent's `{""}` empty-string placeable trick when we touch this surface again.

**`--lang` → Project Fluent wiring**

- `crates/bwoc-cli/src/i18n.rs` — new module exposing `bundle_for(lang)` and `t(bundle, key)`. Locale files (`locales/<lang>/cli.ftl`) embedded into the binary at compile time via `include_str!`, so distributed `bwoc` doesn't need to find them on disk. Unsupported language codes fall back to `en`. Fluent's default Unicode bidirectional isolation marks disabled for clean terminal output. Missing-key lookups return a visible `«missing key: <key>»` marker rather than panicking — surfaces gaps during dev. 4 new unit tests (EN content, TH content, unknown-lang fallback, missing-key marker).
- `crates/bwoc-cli/Cargo.toml` — new deps `fluent-bundle` + `unic-langid` (both already in `[workspace.dependencies]` from iter 1's scaffold; just inheriting them now).
- `crates/bwoc-cli/locales/en/cli.ftl` + `locales/th/cli.ftl` — added `default-help-hint` message (EN: "try `bwoc --help`"; TH: "ลองใช้ `bwoc --help`").
- `crates/bwoc-cli/src/main.rs` — replaces the default-no-subcommand `println!` literal with `i18n::t(&bundle, "default-help-hint")` driven by the resolved `--lang`. **This iter wires infrastructure plus ONE message as proof; converting the remaining `println!` literals across `check`/`new`/`spawn`/`init`/`workspace`/`list`/`bwoc-agent` is a follow-up so we don't bundle all string conversions into one iter (Mattaññutā).** Live-verified: `bwoc` → EN; `bwoc --lang th` → Thai; `BWOC_LANG=th bwoc` → Thai; `bwoc --lang ja` → EN fallback.

**`bwoc list`**

- `crates/bwoc-cli` — **`bwoc list [--workspace <path>]`** implemented. Reads the enclosing workspace's `.bwoc/agents.toml` and prints an id/status/backend/path table. Workspace resolution per `WORKSPACE.en.md` §"Workspace Resolution": explicit `--workspace` → `BWOC_WORKSPACE` env → ancestor walk for `.bwoc/workspace.toml` → cwd self-check → fail with actionable exit-2 error. Empty registry prints `(no agents in workspace <path>)` and exits 0. 1 new unit test for ancestor-walk. Live-verified 4 scenarios: empty workspace, two-agent workspace via `--workspace`, ancestor walk from a workspace subdir, and non-workspace dir (exit 2 with actionable message). Same full-resolution logic should later be promoted to `workspace info` / `validate` (logged as follow-up).

**Issue and PR templates (non-policy)**

- `.github/ISSUE_TEMPLATE/bug_report.md` — structured form with BWOC-specific fields: BWOC version, OS, Rust toolchain, backend (claude/gemini/codex/kimi), surface (framework/template/CLI/runtime/hooks), and **arc phase** (uppāda/ṭhiti/vaya — where in the agent's life did this surface?). Includes a SECURITY redirect for exploitable defects.
- `.github/ISSUE_TEMPLATE/feature_request.md` — Problem/Solution/Alternatives shape grounded in Ariyasacca 4 (Dukkha → propose; Samudaya implied; Nirodha/Magga in scope section). Optional but-encouraged "Buddhist framework alignment" field referencing GLOSSARY.
- `.github/PULL_REQUEST_TEMPLATE.md` — Summary + What/Related/Checklist/Risk-and-rollback. The Checklist mirrors `CONTRIBUTING.md` §Pull Request Checklist verbatim PLUS adds bilingual-parity + naming-audit + manifest-schema gates that the CI workflows enforce.

These three are explicitly **non-policy** (mechanical forms that mirror existing CONTRIBUTING.md content). The policy-bearing items still HELD: `CODEOWNERS` (review-duty assignment) and `ISSUE_TEMPLATE/config.yml` (contact-routing URLs).

**Implementation logs (new convention)**

- `notes/` directory established with `notes/2026-05-22_phase-1-v20-foundation.md` as the starter — single session covering open-source hygiene + bilingual spec layer + Rust scaffold + auto-versioning + CI + over-engineering protection. Captures decisions, alternatives, and bugs surfaced.
- `CLAUDE.md` — "Implementation Logs (HARD RULE)" section added: every significant change gets `notes/YYYY-MM-DD_<title>.md` per the pattern in `NAMING.en.md`. One note per session, not per file.

**Modules layer (filled previously-empty placeholders)**

- `modules/README.md` — top-level modules overview (`agent-template/` ready · `plugins/` planned · `skills/` planned · `cli/` deprecated). Adds "Adding a new module" guidance.
- `modules/plugins/README.md` — planned framework plugins spec. Defines what plugins are (Tier 2 memory backends, additional LLM-backend integrations, workflow integrations), what they are NOT (vendor-specific shortcuts), and that the loading mechanism lands with the first plugin.
- `modules/skills/README.md` — planned framework skills spec. Distinguishes framework skills from agent skills (per-agent slot) and from `.claude/skills/` (Claude Code project skills).
- `modules/agent-template/mindsets/SPEC.md` — agent slot spec. Mindsets = decision-making frameworks; one mindset per file; Obsidian frontmatter; "When NOT to apply" required; each anchors one Pali principle.
- `modules/agent-template/skills/SPEC.md` — agent slot spec. Skills = concrete capabilities; bounded; verifiable; cross-linked from `interconnect/capabilities.md`; maturity levels L1–L7 per Ariya-dhana 7.

**Tooling and process (Claude Code)**

- `CLAUDE.md` — framework-level guidance for Claude Code sessions.
- `.claude/skills/` — `/incarnate`, `/check-neutrality`, `/check-bilingual`, `/task-log`, `/check-naming` (project-scoped slash commands).
- `.claude/hooks/bilingual-reminder.sh` — `PostToolUse` `Write|Edit` hook reminding to update the matching TH file when an EN doc changes. **Extended** to cover (a) the **reverse direction** for `docs/<lang>/` (editing TH reminds about EN canonical) and (b) **root-level `FILENAME.md` ↔ `FILENAME.th.md`** (e.g., `VISION.md` ↔ `VISION.th.md`). Root-level canonical→translation only fires if the translation already exists, to avoid noisy reminders for unpaired files like `CHANGELOG.md`. Out-of-repo paths exit silently (matches `auto-version.sh` scoping). Pipe-tested all 8 scenarios.
- `.claude/hooks/auto-version.sh` — `PostToolUse` `Write|Edit` hook that auto-bumps SemVer PATCH on every Claude Code edit. Software domain (`.rs` / `.toml` / `crates/*`) bumps `Cargo.toml` `[workspace.package].version`; document domain (`.md`) bumps `VERSION.md` `Document-Version`. Both stamp `Last-Updated` (UTC, ISO 8601). Self-managed files are guarded against self-trigger.
- `docs/en/WORKSPACE.en.md` + `docs/th/WORKSPACE.th.md` — workspace concept spec. Defines on-disk structure (`.bwoc/workspace.toml`, `.bwoc/agents.toml`), validation rules ("complete before work"), CLI surface (`bwoc init`, `bwoc workspace info/validate`), workspace resolution precedence (`--workspace` flag → `BWOC_WORKSPACE` env → ancestor walk → cwd → refuse), central per-user memory at `~/.bwoc/` (config, memory, workspaces registry, logs), and memory scope hierarchy (per-agent → per-workspace → per-user → Tier 2).
- `docs/en/NAMING.en.md` + `docs/th/NAMING.th.md` — unified `*.md` naming standard with 12 categories, rule definitions, quick decision tree, and audit grep snippets. New note pattern `YYYY-MM-DD_<title>.md` (ISO 8601 date prefix + underscore + kebab-case title) for chronological notes; valid locations are `<repo>/notes/`, `<workspace>/.bwoc/notes/`, or `~/.bwoc/notes/`.
- `docs/en/ROADMAP.en.md` + `docs/th/ROADMAP.th.md` — phase-by-phase plan (Phase 1 v2.0 uppāda → Phase 4 fleet). Each phase has Definition of Done and links the spec doc each remaining item refers to. README Status table now points here for the full plan.
- `docs/en/FAQ.en.md` + `docs/th/FAQ.th.md` — newcomer FAQ across Conceptual, Project Mechanics, Setup, Multi-Language and Multi-Backend, Conventions, Operations, and Contributing categories. Extracts the three READMEs Qs and expands with Qs surfaced by VISION/GLOSSARY/ARCHITECTURE/INCARNATION/WORKSPACE/NAMING. README FAQ section now points here for the full set.
- `.claude/settings.json` — registers both hooks for the team.

**Phase 2 + 3 implementation arc** (theme-grouped; per-commit detail in `git log` and [`notes/2026-05-22_phase-2-thiti-surface.md`](notes/2026-05-22_phase-2-thiti-surface.md))

- **Lifecycle verbs** (Phase 3 vaya + state machine):
  - `bwoc retire <name>` (registry removal; `--keep-files` retains agent dir)
  - `bwoc stop <name>` — 3-step escalation ladder: socket `STOP` → SIGTERM → SIGKILL (~3s wait between steps); reports which step ended the daemon
  - `bwoc start <name>` — flips registry status AND spawns `bwoc-agent --serve`; `--no-daemon` opt-out; idempotent across all (status × daemon) combinations
  - `bwoc workspace prune` — reconciles phantom registry entries vs orphan agent dirs; `--apply` removes safe drift

- **Daemon + IPC** (Phase 2 ṭhiti):
  - `bwoc-agent --serve` Unix daemon: writes `.bwoc/agent.{pid,sock}`; line-text IPC protocol (`PING`/`STATUS`/`STOP`) debuggable with `nc -U`
  - Persistent inbox cursor (`.bwoc/inbox.cursor`) — daemon resumes after restart
  - `bwoc ping <agent>` — CLI client for PING
  - Stderr redirect to `<agent>/.bwoc/agent.log` for `bwoc log` to tail
  - `bwoc-agent --version` / `-V` / `--help` / `-h` flags (was: `--serve` only)
  - Windows: `--serve` is a clean cfg-gated stub (default mode + `--version`/`--help` work); named-pipe daemon path queued

- **Messaging stack** (sammā-vācā Phase 0):
  - `bwoc send <agent> <msg>` — JSONL envelope to `<agent>/.bwoc/inbox.jsonl`
  - `bwoc inbox <agent>` — `--limit` · `--json` · `--watch` · `--clear`
  - INBOX column in `bwoc list`
  - Daemon-side inbox watch: announces new envelopes to stderr as they arrive

- **Observation + UX**:
  - `bwoc list` — runtime ●/○ indicator; filters `--status` / `--backend` / `--running`
  - `bwoc status [name]` — health + identity + uptime; per-agent detail surfaces persona scope + mindset/skill/memory counts; `--json` mirrors the human shape
  - `bwoc dashboard` (TUI) — ratatui-based; agents pane + detail pane + 2s auto-refresh + `t` hotkey to spawn chat in a new tmux window + workspace-level projects/notes/memory counts in banner
  - `bwoc chat <agent>` — auto-resolves backend from registry; `--tmux` for new-window mode
  - `bwoc doctor` — env + workspace diagnostic; `--auto` sweeps stale `agent.pid` / `agent.sock` / `inbox.cursor`
  - `bwoc log <agent>` — tails daemon stderr; `-f` follow · `-n N` lines · `--clear` truncate-in-place
  - `bwoc completion <shell>` — bash/zsh/fish/powershell/elvish via clap_complete
  - `bwoc help` — 10 topical guides: `getting-started`, `backends`, `workspace`, `manifest`, `arc`, `lifecycle`, `daemon`, `messaging`, `persona`, `memory`
  - `--json` across read-only commands: `list`, `status`, `workspace info`, `workspace validate`, `check`, `inbox`, `memory list|search`
  - Banner ANSI Shadow wordmark + command index for the no-subcommand case
  - Unicode-width column padding in `bwoc list` (Thai header alignment)

- **Per-workspace memory** (`<workspace>/.bwoc/memory/`):
  - `bwoc init` scaffolds the directory with a README documenting the 4-tier scope hierarchy
  - `bwoc memory list | show | put | search` — full read/write/search CLI with path-traversal refusal, atomic write (stage-to-temp + rename), `--force` overwrite gate, case-insensitive substring search; both human and `--json` output where useful

- **Persona configuration at incarnation**:
  - `bwoc new --scope` / `--out-of-scope` — fill `{{scopeDescription}}` / `{{outOfScope}}` placeholders in AGENTS.md + persona/README.md
  - `bwoc new --mindsets a,b,c` / `--skills a,b,c` — seed stub `.md` files matching the SPEC.md scaffold
  - Manifest schema gained `scopeDescription` + `outOfScope` fields (optional)
  - IncarnationReport surfaces persona_filled + mindset_stubs + skill_stubs counts

- **CI + Release**:
  - `.github/workflows/ci.yml` — matrix build + test across `ubuntu-latest` · `macos-latest` · `windows-latest`; fmt + clippy gated on Ubuntu only (rules are OS-independent)
  - `.github/workflows/release.yml` — triggers on CalVer tag `v<YYYY>.<M>.<D>-<patch>`; 5-target release matrix (Linux x64 + Linux ARM64 + macOS Apple Silicon + macOS Intel + Windows x64); auto-creates GitHub Release with notes + SHA-256 sidecars; `fail_on_unmatched_files: true` so partial releases never ship
  - `.github/workflows/docs.yml` — naming-audit `notes/README.md` exemption added (category 5 slot READMEs)
  - `docs/en/RELEASING.en.md` + `docs/th/RELEASING.th.md` (bilingual pair) — pre-flight, tag-and-push, prerelease vs stable, rollback policy
  - `VERSION.md` "Dual Namespaces" — Cargo SemVer (auto-bumped per edit, dev checkpoint) + Release CalVer (public release identity, manual tag)

- **Refactor + hygiene**:
  - `crate::livecheck` module consolidates 5 byte-identical copies of `signal_zero_alive` / `running_pid` / `query_uptime` / `format_uptime` / `inbox_count` across status/doctor/workspace/dashboard/start
  - End-to-end smoke test at `crates/bwoc-cli/tests/smoke.rs` — `init → new → list` against a real tempdir
  - Test-friendly `cfg(unix)` gating on signal-0 / HOME-env / `/tmp`-path tests for Windows portability
  - `bwoc-agent` Windows stub: `serve_loop` + 4 helpers cfg-gated; non-Unix returns "daemon is Unix-only" exit 2

- **Docs sync**:
  - ROADMAP + README + VERSION.md + CLAUDE.md all kept current with shipped features; multiple per-iter sync commits
  - Root-level bilingual policy documented in CLAUDE.md (which docs require TH pair, which don't)
  - CHANGELOG Known Issues trimmed from 4 → 1 stale items removed
  - 4 implementation notes under `notes/`: bwoc-new UX, gap-analysis, Pages+release pipeline, Phase 2 ṭhiti surface backfill

**Late Phase 2 polish** (since the bullet block above)

- **Memory CRUD completed**:
  - `bwoc memory put <name> [--file <p>] [--force]` — write from stdin or file; atomic stage+rename
  - `bwoc memory search <query> [--json]` — case-insensitive substring across entries
  - `bwoc memory rm <name> [--yes]` — delete an entry (TTY confirm; refuses README.md and traversal)
  - `bwoc memory show --all [--json]` — print every entry concatenated with `# === <name> ===` headers (or JSON array); pairs with agent-boot context loading
  - `bwoc help memory` — topic doc covering all 4 CRUD verbs + search

- **Dashboard hotkey triad**:
  - `t` opens `bwoc spawn` in a new tmux window (chat — original)
  - `l` opens `bwoc log -f` in a new tmux window (daemon log live tail) — NEW
  - `i` opens `bwoc inbox --watch` in a new tmux window (inbox live tail) — NEW
  - Window naming `<agent-id>` / `<agent-id>-log` / `<agent-id>-inbox` so all three can coexist

- **`bwoc list` filter + ordering surface**:
  - `--inbox-pending` — filter to agents with unread envelopes
  - `--sort id | inbox | incarnated | backend` — stable sort with informative default
  - `--count` — emit just the row count (integer or `{"count": N}` with `--json`); short-circuits after filter+sort for shell-script idioms

- **`bwoc doctor`**:
  - WARN on oversized `agent.log` (10 MiB threshold; `--auto` truncates — diagnostic chatter)
  - WARN-only on oversized `inbox.jsonl` (5 MiB threshold; `--auto` explicitly refuses to discard user data — Sammā-vācā)
  - `--json` output with `{ results, summary, exit }` stable shape for CI gating
  - `bwoc help doctor` topic — full status taxonomy, all 7 checks, deliberate asymmetry on user-data handling

- **Workspace surfaces**:
  - `bwoc workspace info` text + JSON gained per-workspace `Resources` block (projects / notes / memory counts)
  - Dashboard banner shows the same counts

- **bwoc-agent**:
  - `--version` / `-V` / `--help` / `-h` flags (was: only `--serve` handled)

**Mass-action verb matrix + shell ergonomics** (latest batch)

- **Six verbs gain `--all`** for fleet-wide operations:
  - `bwoc stop --all` — signal-escalation per agent (STOP → SIGTERM → SIGKILL)
  - `bwoc start --all` — flip registry + spawn daemons (`--no-daemon` opt-out)
  - `bwoc status --all` — full detail block per agent (loop of single-agent view)
  - `bwoc check --all` — fleet-wide neutrality audit with `{ agents[], summary }` JSON
  - `bwoc ping --all` — mass liveness probe (not-running labeled but not failed)
  - (`bwoc list` is already always all-agents; `bwoc retire --all` deliberately omitted — destructive)
  - Each uses clap `ArgGroup` for the `name`/`--all` mutex; trying neither or both → parse error

- **Script-friendly read flags**:
  - `bwoc list --count` / `--names-only` — integer or bare ids for shell loops
  - `bwoc memory list --count` / `--names-only` — same on memory entries
  - `bwoc inbox <name> --count` — envelope count for `if [ $(...) -gt 0 ]`
  - `bwoc workspace info --path-only` — for `cd "$(bwoc workspace info --path-only)"`

- **List filters + sort**:
  - `--inbox-pending` (agents with unread envelopes), combinable with --running/--status/--backend
  - `--sort id | inbox | incarnated | backend` (stable; default = registry order)

- **`bwoc memory put` write modes**:
  - 3 sources: inline positional `[content]` > `--file <path>` > stdin
  - 3 write modes: create (default) / `--force` overwrite / `--append`
  - All atomic via .tmp staging + rename

- **`bwoc send`**: inline `<msg>` OR `--file <path>` (clap mutex; same UX as memory put)

- **Workspace attention summary** — `bwoc workspace info` + dashboard banner show
  total pending inbox count across all agents when > 0; cross-link to
  `bwoc list --inbox-pending` for the "what needs attention?" workflow.

- **`bwoc help` topics 10 → 11**: + `doctor` (status taxonomy + auto-fix policy)

**Process supervision + remaining UX polish** (most recent batch)

- **`bwoc supervise <agent>`** — restart-on-crash supervisor closes a
  Phase 2 "Remaining for ship" item. Spawns `bwoc-agent --serve`,
  waits, respawns on non-zero exit; rate-limited (default 10/min,
  `--max-restarts-per-min N`). Clean exit (status 0) stops the
  supervisor. SIGINT/SIGTERM via ctrlc → exit 0. Stderr → same
  `agent.log` as `bwoc start`, so `bwoc log -f` works against
  supervised daemons. Usage: `tmux new-window 'bwoc supervise alpha'`
  or inside the user's own systemd unit. New `ctrlc` dep on bwoc-cli
  (already a workspace dep for bwoc-agent).

- **`bwoc retire --keep-memory`** — third file mode between default
  (delete) and `--keep-files` (retain all). Removes everything under
  the agent dir EXCEPT `memories/`, preserving accumulated knowledge
  for future agents. clap mutex with `--keep-files`.

- **`bwoc inbox --all`** — print every agent's inbox concatenated,
  each preceded by a `=== <agent-id> (N message(s)) ===` header.
  Empty inboxes still get a header. `--clear` and `--watch` are
  refused with `--all` (mass-clear too destructive; mass-watch
  interleaves confusingly). JSON shape: `{ agents: [{ agent, total,
  shown, messages }] }`.

- **UPTIME column on every overview surface** — `bwoc list` (table)
  and `bwoc status` (table) gained UPTIME between BACKEND and INBOX/
  MODEL. `bwoc list --json` and `bwoc status --json` gained
  `running` + `uptime_seconds` (nullable). All four surfaces share
  the same `livecheck::query_uptime` + `format_uptime` data path.

- **`bwoc check --all`** — fleet-wide neutrality audit. Iterates the
  workspace registry, runs `audit()` per agent, prints per-agent
  sections + fleet summary. JSON shape: `{ workspace, agents[],
  summary }`. Exit 1 iff any agent has violations.

- **`bwoc ping --all`** — mass liveness probe across the workspace.
  Agents with no live socket get "not running" label (not a
  failure; they're just stopped). Protocol drift / connection errors
  → exit 1.

- **Memory write/sort ergonomics**:
  - `bwoc memory put <name> "inline"` — third source mode (precedence:
    inline > --file > stdin); trailing newline appended automatically
  - `bwoc memory put <name> --append` — accumulate to existing entry
    (read-modify-write staged atomically; clap mutex with `--force`)
  - `bwoc memory list --json` adds inline `count` + `total_bytes`
    aggregates
  - `bwoc memory list --sort name|size|modified` — mirror of
    `bwoc list --sort` for memory entries

- **`bwoc send <agent> --file <path>`** — second message source
  (clap mutex with inline `<msg>`); trailing newlines trimmed so
  vim/EOF newline doesn't bloat the envelope.

- **`bwoc help` topic 11 → 12**: + `script` (shell idioms for
  --count / --names-only / --json / --path-only across all read
  commands)

**Write-command JSON family + dashboard help + memory sort** (most recent)

- **JSON-everywhere completed across write commands**:
  - `bwoc new --json` — incarnation report `{ agent_id, target,
    registered_in, symlinks, mindset_stubs, skill_stubs, persona_filled }`
  - `bwoc start --json` (requires `--yes`) — `{ workspace, agent,
    daemon_spawned, daemon_pid, already_running, registry_updated }`
  - `bwoc stop --json` (requires `--yes`) — `{ workspace, agent,
    daemon_outcome, registry_updated }` (outcome: not_running /
    socket_ok / sigterm / sigkill / could_not_kill)
  - `bwoc retire --json` (requires `--yes`) — `{ workspace, agent,
    path, mode, registry_updated }` (mode: delete / keep_files /
    keep_memory)
  - `bwoc workspace prune --json` — `{ workspace, phantoms, orphans,
    applied, removed }` for CI gating
  - `bwoc supervise --json` — emits one structured event per action
    (watch_start / spawn / crash_respawn / clean_exit / rate_limit_hit /
    signal_stop / spawn_failed)
  - `bwoc inbox --watch --json` (was rejection, now streams) — one
    compact JSON envelope per line for log shippers
  - Safety guard on destructive verbs: --json requires --yes
    (scripted destructive ops without explicit ack → exit 2)

- **Dashboard `?` overlay** — centered help popup listing every
  hotkey, dismissed on any key. Footer gains a `?: help` chip.

- **`bwoc memory list --sort name|size|modified`** — mirror of
  `bwoc list --sort`. Default = name (alphabetical). Unknown field
  → exit 2 with accepted-values hint. Entry mtime captured via
  `metadata().modified()`.

- **`bwoc memory list --json` aggregates** — inline `count` +
  `total_bytes` fields so CI doesn't have to walk entries[] to
  compute totals.

- **`bwoc help --all`** — concatenated all-topics output with
  `# === <name> ===` Markdown-safe separators for offline reading
  or pipe into docs generator.

### Changed

- `modules/agent-template/README.md` — added badges, table of contents, and footer; trimmed the "Incarnating a New Agent" section to a quickstart that points at `docs/en/INCARNATION.en.md`.
- `README.md` "Getting Started > As an Agent Author" — replaced outdated manual `cp -r` recipe with the canonical `./scripts/incarnate.sh` invocation and link to `INCARNATION.en.md`.
- `README.md` FAQ — trimmed to top-3 + link to full `docs/en/FAQ.en.md`.
- `README.md` Status — trimmed to a summary table + link to `docs/en/ROADMAP.en.md` for the full phase plan.
- `VERSION.md` — restructured header to expose `Software-Version`, `Document-Version`, `Last-Updated` (UTC ISO 8601). Auto-managed by `.claude/hooks/auto-version.sh`.
- `crates/bwoc-cli/README.md` — added workspace command surface (`bwoc init`, `bwoc workspace info/validate`) and `--workspace` flag declaration.
- `modules/agent-template/conventions.md` — pointer to `docs/en/NAMING.en.md` as the comprehensive `*.md` naming standard; softened validation-checklist rule from "File names are kebab-case.md" to "Markdown file names follow NAMING.en.md (12 categories)"; renamed "Files & Directories" section to "Directories" since file naming now lives in NAMING.
- `modules/agent-template/docs/th/PHILOSOPHY.th.md` — corrected `## ๑. หลักธรรมหลัก ๑๔ ประการ` to `## ๑. หลักธรรมหลัก ๒๒ ประการ` to match the EN side (22 verified by counting groups A–F).
- `.claude/hooks/auto-version.sh` — two silent bugs fixed: (1) GNU-only sed `0,/regex/s||...|` replaced with portable `s|^version = "X.Y.Z"$|version = "X.Y.Z"|` for Cargo.toml bumps on macOS BSD sed; (2) out-of-repo file paths (e.g., `~/.claude/projects/.../memory/*.md` edits) no longer trigger Document-Version bumps — added early-exit when the file is not under the workspace root. Both verified via pipe-test.
- `modules/agent-template/AGENTS.md` reference set — unchanged (the v2.0 spec is the baseline this Phase implements).

### Deprecated

- `modules/cli/` — replaced by `crates/bwoc-cli/`. A stub README is left in place; the directory will be removed once nothing references it.

### Conventions

- **Root-level bilingual files**: `FILENAME.md` is the English canonical; `FILENAME.<lang>.md` is a translation (e.g. `VISION.md` ↔ `VISION.th.md`). Parallel to but distinct from the `docs/<lang>/` pattern used inside the agent template.

### Known Issues

- Two `CONTRIBUTING.md`-referenced policy files are HELD pending user direction: `.github/CODEOWNERS` (review-duty assignments) and `.github/ISSUE_TEMPLATE/config.yml` (Discussions URL + contact routing). The non-policy issue/PR templates (`bug_report.md`, `feature_request.md`, `PULL_REQUEST_TEMPLATE.md`) shipped earlier. See `.claude/loop-roadmap.md` for the HELD status detail.

---

## Pre-Phase-1

Framework specification existed prior to this changelog: `AGENTS.md` v2.0, the 22 Buddhist-framework mappings in `PHILOSOPHY.en.md`, the PRD (Ariyasacca 4), SRS (Magga 8), lifecycle, threat model (Taṇhā 3 + Sīla 5), and self-improvement (Paññā 3) documents.
