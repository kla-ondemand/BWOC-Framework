---
date: 2026-05-23
session: "What's New" surface — banner section + once-per-version notice
tags:
  - phase/3
  - type/note
  - module/bwoc-cli
---

# 2026-05-23 — "What's New" Surface

User: "ใส่ what's new? เมื่อ run cli เสมอ" — show what's new whenever the CLI runs. Implemented as two complementary surfaces so it's "always there" without spamming every command.

## What changed

- **`crates/bwoc-cli/src/whats_new.rs`** (new) — single source for the release headline + highlight bullets (`HEADLINE`, `HIGHLIGHTS`). Plus `notify_if_updated()`: a once-per-`MAJOR.MINOR` upgrade notice.
- **`crates/bwoc-cli/src/banner.rs`** — bare-`bwoc` banner gains a `✨ What's New` section that renders `HEADLINE` + the highlight bullets (imported from `whats_new`, not duplicated).
- **`crates/bwoc-cli/src/main.rs`** — `mod whats_new`; calls `whats_new::notify_if_updated()` once at startup *for subcommands only* (the bare banner already shows the full block).

## Decisions

- **Two surfaces, not one.** "เสมอ / always" pulls two ways: the bare-`bwoc` banner is the always-visible home for the full What's New; the once-per-version notice covers the "I ran a subcommand and didn't know I'd upgraded" case. Together they satisfy "always" without printing release notes on every `bwoc list`.
- **Key the notice on `MAJOR.MINOR`, not the full version.** The auto-version hook bumps the patch on every `.rs`/`.toml` edit (the build is at 2.0.52 right now), so keying on the full version would fire the notice constantly in development. `MAJOR.MINOR` only changes on a release-significant bump, so the notice fires once per real version step and stays quiet through patch churn.
- **Notice → stderr, gated on stdout-TTY.** Printing to stderr keeps it out of `--json` / piped stdout that consumers parse. Additionally gating on `stdout().is_terminal()` means scripts and CI never see it at all — only an interactive operator does. Verified: `bwoc list | …` (piped) emits no notice.
- **Record-before-print.** `notify_if_updated` writes `~/.bwoc/last-seen-version` *before* printing, so a later write failure can't loop the notice forever (worst case: the operator sees it once extra, never repeatedly).
- **`BWOC_NO_WHATSNEW=1` escape hatch.** An operator who finds the notice noisy can hush it without losing the banner section.
- **Highlights as a baked-in constant, not CHANGELOG parsing.** Reading + parsing CHANGELOG.md at runtime is fragile (format drift, file may not ship with the binary). A small `HIGHLIGHTS: &[&str]` updated per release is reliable and ships in the binary. The release checklist gains one line: "bump `whats_new::HEADLINE` + `HIGHLIGHTS`."

## Alternatives considered

- **Show the full What's New on every command** — rejected; `bwoc list` printing six bullets every run is hostile. The once-per-version one-liner is the npm/homebrew-style answer to "always check, show when there's something new."
- **Track last-seen as the full patch version** — rejected (dev churn, see above).
- **Store the marker in the workspace** — rejected; "what's new" is per-user/per-machine (you upgraded the binary), not per-workspace. `~/.bwoc/last-seen-version` is the right home.
- **A `bwoc whatsnew` subcommand** — possible future nicety; not built (Mattaññutā). The banner already serves the on-demand full view.

## Bugs surfaced and fixed

- **None.** Clean build + clippy + tests. 2 unit tests in `whats_new` (major.minor shape; highlights stay lean ≤6 single-line bullets).

## Test summary

- `cargo build -p bwoc-cli`, clippy, `cargo test -p bwoc-cli` — clean.
- Live: bare `bwoc` shows the `✨ What's New` section with all six highlights; `bwoc list` piped (non-TTY) emits no notice (gating verified). The TTY notice path is exercised by construction + the `major_minor` unit test (a true PTY isn't scriptable here).

## Status / deferred

- **Release checklist** now includes bumping `whats_new::HEADLINE` + `HIGHLIGHTS` alongside the CHANGELOG seal. Worth adding to the `release.yml` / VERSION.md maintainer recipe next time either is touched.
- **`bwoc whatsnew` subcommand** — deferred; banner covers on-demand viewing.

## Related

- `crates/bwoc-cli/src/whats_new.rs` — headline + highlights + notice
- `crates/bwoc-cli/src/banner.rs` — What's New section
- `crates/bwoc-cli/src/main.rs` — subcommand notice call
- Marker: `~/.bwoc/last-seen-version`
