# 2026-05-23 — Backend rename: Gemini → Antigravity (CLI `agy`)

Google deprecates Gemini CLI for Google One / unpaid tiers on 2026-06-18 and ships **Antigravity** (`agy`) as the replacement — a multi-vendor agentic CLI that routes Gemini, Claude, and GPT-OSS model families through one binary. The framework follows the actual product surface, so the `gemini` backend identity is replaced entirely with `antigravity`. Breaking change for any agent on disk that still references `GEMINI.md` or `backend = "gemini"`.

## What changed

- **Rust** (`crates/bwoc-cli`): `Backend::Gemini` → `Backend::Antigravity`; `cli_name()` → `"agy"`; model catalog now lists the 7 picker entries surfaced by `agy` (`gemini-3.5-flash-{medium,high}`, `gemini-3.1-pro-{low,high}`, `claude-{sonnet,opus}-4.6-thinking`, `gpt-oss-120b-medium`). Backend-symlink arrays in `check.rs`, `doctor.rs`, `status.rs`, `new.rs`, `dashboard.rs` swap `GEMINI.md` → `AGY.md`. `check.rs::BACKEND_PHRASES` flags `Antigravity will/can`; `HARDCODED_MODELS` gains `gemini-3` and `gpt-oss`. All affected unit tests updated (115 pass).
- **Symlinks**: `GEMINI.md` deleted in `modules/agent-template/`, `agents/agent-pi/`, `agents/agent-oracle/`; `AGY.md → AGENTS.md` created in their place.
- **Shell scripts**: `modules/agent-template/scripts/{incarnate,check-agent-neutrality}.sh` mirror the Rust audit (`AGY.md`, new `BACKEND_PHRASES`, expanded `HARDCODED_MODELS`).
- **Docs (EN + TH parity)** updated across `VISION`, `README`, `SECURITY`, `docs/{en,th}/{ARCHITECTURE,INCARNATION,WORKSPACE}`, `modules/agent-template/{AGENTS,CLAUDE,README,conventions,neutrality}.md`, `modules/agent-template/persona/README.md`, `modules/agent-template/docs/{en,th}/{OVERVIEW,SRS}`, `modules/plugins/README.md`, and `examples/howto/configure-backends.md`. All `GEMINI.md` → `AGY.md`, "Gemini CLI" → "Antigravity CLI", `backend = "gemini"` → `backend = "agy"`.
- **E2E test**: incarnated `agent-anti` (`bwoc new anti --backend antigravity --primary-model gemini-3.5-flash-high --role analysis` + Thai persona scope), then `bwoc spawn ... -- --print "..."` actually exec'd `~/.local/bin/agy` and returned a polite Thai response matching the persona register.

## Decisions

- **Replace, don't dual-track.** Choices were (a) full rename, (b) add Antigravity as a 5th backend keeping `gemini`, (c) rename with `"gemini"` config alias. Chose (a) per [Samānattatā](modules/agent-template/docs/en/PHILOSOPHY.en.md) — the framework names backends after the CLI product, not the model family. Keeping two near-identical entries would inflate the surface for ambiguous gain. Existing on-disk agents with `GEMINI.md` / `backend = "gemini"` break loudly (Anattā — no clinging to stale identifiers).
- **Binary = `agy`, symlink = `AGY.md`, enum = `Backend::Antigravity`.** Convention in the codebase is symlink-name == uppercased CLI binary. The product name "Antigravity" lives in prose; the 3-letter binary lives in code paths.
- **Model identifiers stay `gemini-*`-prefixed.** Only the CLI product renamed; the Gemini model family still exists under Antigravity routing. `check.rs::HARDCODED_MODELS` keeps the `gemini-*` detection patterns and adds `gemini-3`, `gpt-oss` so AGENTS.md neutrality stays enforced for the new families.
- **Historical notes left untouched** (`notes/2026-05-22_*`, `notes/2026-05-23_phase-4-fleet-governance.md`). Per CLAUDE.md, session notes are snapshots — retroactive edits would lie about what was true then.

## Alternatives considered

- Keep `Backend::Gemini` with `cli_name = "agy"` (cheap rename). Rejected — the enum variant is the canonical identity in code, configs, JSON, and error messages; staying named `Gemini` while binary says `agy` would mislead every contributor reading the code for the first time.
- Provide a config alias so `backend = "gemini"` in `.bwoc/agents.toml` still parses. Rejected — fewer than three known incarnated agents touched (template + pi + oracle), and a one-line `sed` migration is cheaper than carrying an alias forever.
- Wait for Antigravity's official docs to verify the canonical kebab-case form of model IDs (the docs page at `antigravity.google/docs/getting-started` was empty at edit time). Mitigated — the user supplied the picker labels directly, and the framework's `models()` list is documented as "convenience, not a whitelist", so free-text input still works.

## Bugs surfaced and fixed

- None new. The pre-existing dual-mode `bwoc check` (Step 2 of trust work, 2026-05-23) correctly flagged `agent-anti` as incarnated-but-not-personalized (16 unsubstituted `{{placeholders}}` in `AGENTS.md` prose). That is template-level scaffolding, not rename-related — `bwoc new` writes the manifest but does not substitute placeholders in the AGENTS.md body. Tracked separately in `notes/2026-05-23_check-dual-mode-and-personalize.md`.

## Status / deferred

- **Shipped**: rename across Rust + scripts + docs + symlinks; agent-anti incarnation as live verification.
- **Deferred**: a migration helper (`bwoc migrate --from gemini-to-agy`) — not worth writing for three known agents. Manual fix: `mv GEMINI.md AGY.md` and `sed -i '' 's/backend = "gemini"/backend = "agy"/' .bwoc/agents.toml`.
- **Deferred**: AGENTS.md prose placeholder substitution at `bwoc new` time — separate concern, tracked in the trust-step-4 note.

## Related (links)

- [`CHANGELOG.md`](../CHANGELOG.md) — Unreleased § Changed — BREAKING
- [`crates/bwoc-cli/src/spawn.rs`](../crates/bwoc-cli/src/spawn.rs) — `Backend::Antigravity` + multi-vendor model catalog
- [`modules/agent-template/scripts/check-agent-neutrality.sh`](../modules/agent-template/scripts/check-agent-neutrality.sh) — shell-side audit
- [`agents/agent-anti/config.manifest.json`](../agents/agent-anti/config.manifest.json) — E2E verification artifact
