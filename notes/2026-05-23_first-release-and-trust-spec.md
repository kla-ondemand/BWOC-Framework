---
date: 2026-05-23
session: first release shipped + Phase 3 trust spec drafted
tags:
  - phase/release
  - phase/3
  - type/note
---

# 2026-05-23 — First Release + Trust Spec

The framework's first public release shipped today (`v2026.5.23-0` + `v2026.5.23-1`), and the first Phase 3 spec — Kalyāṇamitta-7 inter-agent trust — landed as a draft. Significant because both close items that had been "remaining" for months: the release pipeline went from "ready" to "exercised", and Phase 3 went from "all remaining" to "1 spec drafted, 4 remaining". Also this is the session where I (Claude Code) learned that ต้นกล้า — the human operator — is the framework's author and that `agent-pi` / `agent-oracle` are agents under `agents/`, not the user.

## What changed

- **First public release shipped.** Tag `v2026.5.23-0` triggered the release workflow; 4 of 5 matrix jobs uploaded successfully, the linux-x64 job lost a race condition on `softprops/action-gh-release@v2`. Re-ran the failed job → 10 assets total (5 binaries + 5 sha256) across `aarch64-apple-darwin`, `x86_64-apple-darwin`, `aarch64-unknown-linux-gnu`, `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`.
- **Release workflow hardened.** `v2026.5.23-1` shipped the race-condition fix: one `create-release` job (`gh release create --generate-notes`) runs once before the matrix; matrix jobs `needs: create-release` then call `gh release upload --clobber`. All 5 platforms uploaded clean on first try with no rerun.
- **Node 24 forward-compat.** Added `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true` at the workflow level to ci.yml + release.yml — 9 days ahead of the 2026-06-02 forced cutover.
- **CI bug fix.** `clippy::unnecessary_sort_by` on Rust 1.95 had been silently failing every push since `e83a35a` (the `bwoc memory list --sort` work). Fixed by rewriting two `sort_by` calls as `sort_by_key(|e| std::cmp::Reverse(...))`.
- **Status docs synced.** CHANGELOG `[Unreleased]` split into `[v2026.5.23-0]` (everything Phase 1+2 to date) + `[v2026.5.23-1]` (workflow fix). VERSION.md + README.md Status section both reference the live release URL.
- **Trust spec drafted.** New `modules/agent-template/interconnect/trust.md` + `trust.th.md` cover the Kalyāṇamitta-7 model from AN 7.36: 7 declared booleans per agent in `config.manifest.json`, optional `requiredTrust` array for refusal at the inbox layer, evidence rules `bwoc check` will verify, refusal semantics that mark envelopes (never delete — auditability). Pointer in `PHILOSOPHY.en.md` + `PHILOSOPHY.th.md` updated from the dead `docs/COORDINATION-PROTOCOL.md` to the new spec. ROADMAP rows note "Spec draft shipped 2026-05-23".

## Decisions

- **CalVer tag `v2026.5.23-0` was the right choice over a SemVer like `v0.1.0`.** The dual-versioning policy in VERSION.md is explicit: Cargo SemVer for internal dev checkpoints; CalVer Git tags for public release identity. The release workflow's trigger glob (`v[0-9][0-9][0-9][0-9].*`) refuses non-CalVer shapes by design.
- **Same-day patch `v2026.5.23-1` rather than waiting until tomorrow.** The workflow fix was urgent enough that the small ergonomic cost of two same-day tags is worth less than the cost of leaving the race condition documented as a known issue. CalVer's `-<patch>` suffix is exactly for this.
- **Trust spec: declared booleans, not earned scores.** ต้นกล้า picked the simplest of three options (7 booleans / 7 floats / single composite). Justification in the spec: runtime telemetry is easy to game, and v1 should prove the gating mechanic works at all before adding nuance. Hybrid downgrade-only model deferred to v2.
- **Trust spec: self-declared, verified by `bwoc check`.** Same logic — keep v1 honest at the boundary (the operator authoring the manifest) rather than trying to compute truth at runtime. Identity-proof / signed manifests are a separate Phase 3 work item.
- **Permissive default for trust gating.** No `requiredTrust` = no refusal. Recipients opt in. The framework ships friendly by default; strict mode is a deliberate per-agent choice.
- **`Co-Authored-By: ต้นกล้า via Claude Code` on the trust commit.** Earlier commits this session signed as "Pi (Claude Opus 4.7)" — that was wrong (Pi is `agent-pi`, an agent ต้นกล้า built; not the user). Going forward, sign with ต้นกล้า's identity where appropriate.

## Alternatives considered

- **Tag `v2026.5.22-0` instead.** The release tag was pushed at ~2026-05-23 UTC but the timestamps the user sees might be local time of 2026-05-22 still. Chose 2026.5.23 (UTC date matches the push) because CalVer is operator-time anyway and the ROADMAP/CHANGELOG already used 2026-05-23.
- **Generate release notes manually for v2026.5.23-0.** Considered curating release notes from the CHANGELOG `[Unreleased]` section. Chose `--generate-notes` (auto from commit history) because (a) the auto-generation is fine for a first release where there's no prior tag to diff from, and (b) the curated CHANGELOG entry is preserved separately and stable across history.
- **Score representation as floats (0.0–1.0).** Rejected — invites bikeshedding on weights and gives a false sense of precision when the underlying signal is binary ("does this agent have the structural pieces in place").
- **Strict-by-default trust gating.** Rejected for v1 because it would break every existing message flow on rollout. ต้นกล้า can change the default later by editing the spec; permissive→strict is a one-way decision worth making with adoption data, not at spec-time.

## Bugs surfaced and fixed

- **CI was silently failing on every push since `e83a35a`.** I had been running `cargo build --workspace --tests 2>&1 | tail -3` locally — which masks errors when the failure happens earlier than the last 3 lines. Rust 1.95 introduced `clippy::unnecessary_sort_by` which CI on stable hit immediately; my local toolchain was older. Lesson: glance at `gh run list` after every push, not just local green.
- **Release workflow race.** Five parallel matrix jobs all called `softprops/action-gh-release@v2` with create-or-update semantics; the action's create path raced and one job lost with "Validation Failed: already_exists". Fixed by splitting into a serial `create-release` job + parallel `upload-only` jobs.
- **`bwoc send` help text shows argument order backwards.** Help advertises `<MESSAGE> <TO>` but the struct field order makes it `bwoc send <TO> <MESSAGE>`. Discovered when trying to send to agent-oracle. Not fixed yet — surfaces in the next iter or whenever ต้นกล้า prompts.

## Status / deferred

- **Trust spec → Rust implementation deferred.** 5-step implementation order documented in the spec: (1) `bwoc-core::Manifest` deserialization, (2) `bwoc check` evidence verification, (3) `bwoc trust <agent>` read command, (4) daemon-level refusal at inbox poll behind `BWOC_TRUST_GATING=1`, (5) CHANGELOG + TH parity. Each step ships independently.
- **`bwoc send` argument-order bug.** Spec/help inconsistency known.
- **Memory hygiene done.** `user_calls_me_pi.md` + `feedback_pi_personality.md` deleted (built on the wrong premise that "Pi" was the user). New memories: `user_tonklaa.md` + `project_pi_and_oracle.md` + `feedback_ask_in_thai.md`.

## Related

- Release URLs: [`v2026.5.23-0`](https://github.com/bemindlabs/BWOC-Framework/releases/tag/v2026.5.23-0) · [`v2026.5.23-1`](https://github.com/bemindlabs/BWOC-Framework/releases/tag/v2026.5.23-1)
- Trust spec: [`modules/agent-template/interconnect/trust.md`](../modules/agent-template/interconnect/trust.md) · [`trust.th.md`](../modules/agent-template/interconnect/trust.th.md)
- Workflow fix: commit `84941c7`
- CI fix (sort_by_key): commit `8c3afab`
- Node 24 opt-in: commit `7de939f`
- Status doc sync: commit `89fcba0`
- Phase 3 trust spec commit: pending (this note ships with it)
