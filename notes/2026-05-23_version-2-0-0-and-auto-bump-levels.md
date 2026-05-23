---
date: 2026-05-23
session: bump app to 2.0.0 + auto-version hook gains minor/major support
tags:
  - phase/3
  - type/note
  - module/build
  - module/hooks
---

# 2026-05-23 — Version 2.0.0 + Auto-Bump Level Support

User directive: "chage app version to 2.x.x for now" + "auto-patch ต้องรองรับ pump minor, major ด้วย". Bumped the workspace from `0.1.721` → `2.0.0` and taught the auto-version hook to honor a one-shot sentinel that requests a `minor` or `major` bump on the next fire. Patch remains the default.

## What changed

- **`Cargo.toml` workspace.package version** — `0.1.721` → `2.0.0`. `cargo build` auto-synced `Cargo.lock`. All three crates (bwoc-core, bwoc-cli, bwoc-agent) inherit via `version.workspace = true`.
- **`VERSION.md` Software-Version** — `0.1.721` → `2.0.0`. Document-Version unchanged (different domain).
- **`.claude/hooks/auto-version.sh`** — gained `bump_at_level <cur> <level>` (major | minor | patch — patch is the same behavior as before) and `consume_bump_sentinel <domain>` which reads + deletes `.bwoc/next-bump.<domain>` and returns the requested level (falling back to `patch` on missing / invalid contents). Both software and document branches now compute their level via the sentinel.
- **`scripts/queue-bump.sh`** (new, 60 lines) — friendly wrapper. `./scripts/queue-bump.sh software minor` writes the sentinel; `--clear` removes it; `--status` lists pending sentinels across both domains.
- **`crates/bwoc-cli/src/send.rs`** — fixed one stale `SendArgs` test literal that a parallel session left missing the new `reply_to` and `no_wakeup` fields. The struct grew those fields elsewhere in d446655's bundle; one test site was missed.

## Decisions

- **2.0.0 directly, not 1.0.0 first.** User said "2.x.x for now"; the major jump is deliberate. The previous CalVer release `v2026.5.23-1` already shipped as a public surface, so the Cargo SemVer was always going to need a discontinuity to align with the operator-visible identity. Going straight to 2.0.0 honors that signal without an artificial 1.0.0 stop.
- **Hook keeps patch as default, sentinel opts up.** The vast majority of edits genuinely are patch-level (small fix / clarification / refactor). Forcing the operator to declare a level on every edit would be ceremony. The sentinel inverts that: the unusual case (minor / major) is the one that requires intent.
- **One-shot consume.** The sentinel is deleted after a single bump so a stale "minor" doesn't keep silently amplifying every subsequent edit. The operator must re-queue for each significant change.
- **Per-domain sentinels** (`next-bump.software`, `next-bump.document`) — not one shared file. The software and document domains evolve at different cadences; sharing a sentinel would make every Markdown edit consume an intended Rust bump (or vice versa) depending on timing. Two files = no cross-contamination.
- **Sentinels live under `.bwoc/`** — the framework's `.bwoc/` is gitignored by `.gitignore` (per `CLAUDE.md` §Repo State Quirks). In user workspaces created by `bwoc init`, `.bwoc/` is tracked but `next-bump.*` is one-shot and ephemeral; if accidentally committed, the next hook fire consumes it. Not worth a separate `.gitignore` entry yet.
- **Helper script does NOT bypass the hook.** `queue-bump.sh` only writes / clears the sentinel; the actual bump still flows through the hook on a real edit. Pairs cleanly with `scripts/bump-version.sh` (which DOES bypass the hook and bumps in place — useful for releases where you want to bump without making any code edit).

## Alternatives considered

- **Commit-message-driven bumps** (e.g., `feat:` → minor, `BREAKING CHANGE:` → major). Rejected — the hook is PostToolUse, not pre-commit. By the time a commit message exists, the version has already been bumped many times during the session. Sentinel is the natural fit for tool-time decisions.
- **Detect breakage by diff analysis.** Considered semantic analysis (Rust public API changes → minor, etc.). Rejected — far too much complexity for the marginal value; even cargo-semver-checks needs a known prior version to compare against.
- **One unified sentinel `.bwoc/next-bump` with `software=minor` format.** Rejected — two files is simpler than a key-value parser in bash; and either-domain semantics is what we actually want.
- **No sentinel deletion — sticky levels.** Considered keeping the level until manually cleared (so a "minor" cycle stays minor for the whole feature branch). Rejected for v1 — too easy to forget; one-shot's explicitness matches the auto-patch default better. Promote to sticky-with-counter (`minor:5` meaning next 5 bumps are minor) if operators ask.
- **Major jump via two `major` script invocations** (0 → 1 → 2). Rejected — required edits to `scripts/bump-version.sh` (it does X→X+1, not X→explicit-target) which is more invasive than just editing the version literal directly.

## Bugs surfaced and fixed

- **Stale `SendArgs` test literal** — `crates/bwoc-cli/src/send.rs:318` was missing the `reply_to` and `no_wakeup` fields that the rest of the file already includes. Likely missed by the d446655 bundle commit's mass-update. Fixed by adding both fields with the same defaults (`None`, `true`) the other test sites use.
- **Parallel-session interference observed.** Multiple Claude Code sessions are firing the auto-version hook against the same `Cargo.toml` / `VERSION.md` simultaneously, so the version is moving (2.0.0 → 2.0.3 → 2.0.11) faster than this session's edits alone would explain. Not a defect — the hook is doing its job; the operator needs to be aware that any cross-session "I'll set 2.0.0 cleanly" can drift before the commit lands. Mitigation: bump-version.sh + a commit immediately after to lock in the intended starting point. Not addressing in code this iter — surfaced as a known operator pattern.

## Status / deferred

- **`scripts/bump-version.sh --target <X.Y.Z>`** — would let the operator set an explicit version (e.g., directly to 2.0.0) without two `major` invocations. Worth adding the next time bump-version.sh is touched. Today's bump used a manual `Cargo.toml` edit because the script only supports +1 increments.
- **Sticky-level sentinel** (`echo "minor:5" > .bwoc/next-bump.software` = minor for next 5 fires). Defer until an operator actually asks for it.
- **Bilingual translation of VERSION.md** — currently English-only. Not high value (operator doc, mostly mechanical). Defer.
- **`bwoc bump` CLI surface** — currently the manual bump and queue-bump are both shell scripts. Promoting to a `bwoc bump <domain> <level>` subcommand would centralize the surface and pick up the lang/workspace plumbing already in the CLI. Defer until the operator pattern stabilizes.

## Test summary

- **Workspace**: 118 tests passing (15 + 84 + 1 + 18 + 0). Clippy clean (`cargo clippy --workspace --all-targets -- -D warnings`). The +3 vs the morning's 115 = the parallel session's additions plus the `SendArgs` fix here.
- **Hook pipe-test**: queued `document minor` → simulated `.md` write → hook emitted `auto-version: document:minor → 1.1.0`. Sentinel auto-deleted after consume. Tested `software major` similarly — `Cargo.toml` and `VERSION.md Software-Version` both went 2.0.0 → 3.0.0 in one shot (then reverted as part of this iter's bump-to-2.0.0 sequence).
- **queue-bump.sh self-test**: `--status` (empty + populated), queue + clear cycle, invalid domain → exit 2 with usage, invalid level → exit 2 with usage.

## Related

- Hook: [`.claude/hooks/auto-version.sh`](../.claude/hooks/auto-version.sh)
- Helper: [`scripts/queue-bump.sh`](../scripts/queue-bump.sh)
- Manual immediate bump (separate role): [`scripts/bump-version.sh`](../scripts/bump-version.sh)
- Version source of truth: [`VERSION.md`](../VERSION.md) — Software-Version + Document-Version + Last-Updated
- Workspace dep: [`Cargo.toml`](../Cargo.toml) `[workspace.package].version`
