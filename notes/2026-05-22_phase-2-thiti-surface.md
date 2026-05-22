---
title: Phase 2 ṭhiti surface — backfill note
aliases:
  - Phase 2 backfill
tags:
  - group/framework
  - type/note
  - meta/log
---

# 2026-05-22 — Phase 2 ṭhiti surface

Backfill note for 12 commits between `1dea3e4` and `10e2172` that shipped substantial Phase 2 + Phase 3 features without paired `notes/` entries. Closes [gap-analysis](2026-05-22_gap-analysis-remediation.md) item 3. Grouped by cluster (init · daemon spawn · doctor/livecheck · dashboard · help/install · chat) — one paragraph each, focused on **what** + **why** + **decision**.

## What changed

### Init writes .gitignore (`1dea3e4`)

`bwoc init` now emits a `.gitignore` at the workspace root that excludes the three daemon-ephemeral files `bwoc-agent --serve` regenerates each launch: `agents/*/.bwoc/agent.pid`, `agents/*/.bwoc/agent.sock`, `agents/*/.bwoc/inbox.cursor`. `inbox.jsonl` is intentionally kept tracked by default (audit trail), with an opt-out comment for teams who don't want it. The .gitignore template lives inline in `init.rs` so future contributors changing it land in the right file.

### `bwoc start` spawns the daemon (`944aa58`)

Closed a Phase 3 UX hole: `bwoc start <name>` previously only flipped the registry status, leaving users to `cd` into the agent dir and run `bwoc-agent --serve` themselves. The verb now does both — idempotent across all four `(status × daemon-alive)` state combinations. `--no-daemon` opts out of the spawn for users who want only the registry flip.

### Doctor inbox-cursor sweep (`4f20f2a`)

Third in the daemon-debris series after stale-PID and stale-socket. Detects three failure modes for `inbox.cursor`: malformed (won't parse as `u64`), out-of-bounds (cursor > file size), orphan (cursor file present but `inbox.jsonl` missing). All three are FAILs without `--auto`; `--auto` removes the bad file and the daemon resets to current EOF on next start.

### Persistent inbox cursor in the daemon (`d2a6a5c`)

Reliability fix for the messaging loop. Before this commit, the daemon started at current EOF every launch — meaning messages received while it was offline got "consumed" into history and never announced to its stderr. Now `<agent>/.bwoc/inbox.cursor` persists the byte offset across daemon restarts. First-run still starts at EOF (no false replay); subsequent runs catch up on backlog. Save is best-effort: a failed write logs a warning instead of crashing.

### Dashboard surfaces runtime / inbox (`70ac3f8`)

Detail pane gained three lines: `runtime ● running (pid N, uptime Xs)` / `○ not running` plus `inbox N message(s)`. Sources match what `bwoc status` and `bwoc list` already use (PID file + signal-0 + STATUS socket query for uptime; inbox.jsonl line count for inbox). Brings TUI parity with the CLI for the same data; no behavior drift between surfaces.

### Refactor: shared `livecheck` module (`9f4e2aa`)

Five copies of `signal_zero_alive` + `running_pid` + `query_uptime` + `format_uptime` + `inbox_count` had accumulated across `status.rs`, `doctor.rs`, `workspace.rs`, `dashboard.rs`, `start.rs`. Consolidated into `crate::livecheck`. Net effect: -31 LOC, +6 unit tests for the new module, **zero behavior change**. The duplications were byte-identical (just copy-paste between iters), so consolidation was mechanical. Decision: extract on the 5th caller, not the 3rd — earlier extractions would have been premature when the surface was still evolving.

### Dashboard auto-refresh (`6850e30`)

Manual `r` key kept the dashboard from feeling live. Added `AUTO_REFRESH_INTERVAL = 2s` const + `last_refresh_at: Instant` tracking. The accept loop already polled events every 200ms; now it also checks elapsed-since-refresh and calls `app.refresh()` past the threshold. 2s chosen to balance liveness against disk thrash for large registries; `r` still resets the timer for instant refresh.

### README Getting Started refresh (`7434414`)

The README claimed `cargo install --path crates/bwoc-cli` was equivalent to `./scripts/install.sh`, but the script had just been extended to install `bwoc-agent` too — leaving any user following the README with a broken `bwoc start` (no daemon binary). Rewrote Getting Started: full toolkit install vs CLI-only one-liner, interactive `bwoc new` quickstart, lifecycle commands shown.

### `bwoc help` 3 new topics (`743a714`)

The 5 original topics (`getting-started`/`backends`/`workspace`/`manifest`/`arc`) covered Phase 1. Phase 2/3 commands (~20) were undocumented in the in-binary help. Added `lifecycle` (state machine), `daemon` (bwoc-agent --serve internals), and `messaging` (inbox flow). Each topic ends with `See:` cross-references forming a navigable triangle.

### install.sh installs both binaries (`a14c706`)

Companion to `944aa58`: now that `bwoc start` spawns the daemon, install.sh needs to land both binaries. Split into `[1/2] cargo install bwoc-cli` + `[2/2] cargo install bwoc-agent`. Added pre-flight PATH warning (was after-the-fact), `--help` flag, and quickstart in the tail output.

### Dashboard `t` hotkey opens tmux (`a4b91d2`)

User-requested: from the TUI dashboard, press `t` to open a chat session with the selected agent in a new tmux window. Resolves the backend from the registry, the model from the manifest, and `exec`s `bwoc spawn` in the agent's directory via `tmux new-window`. Requires `$TMUX` (the dashboard must itself be running inside tmux). Footer momentarily replaces the hotkey legend with action feedback (success / not-in-tmux / no-agent-selected / exec-failure).

### `bwoc chat <name>` CLI equivalent (`10e2172`)

Wraps the dashboard's `t` hotkey logic into a CLI command for users who don't want to launch the TUI first. Same auto-resolve flow. `--tmux` flag opens in a new tmux window; default execs in the current shell.

## Decisions

- **Idempotent start/stop, both running and registry side effects** — easier to teach than separate verbs. Both run when needed, no-op when not.
- **Persist inbox cursor on disk, not in-memory** — daemon restart should not skip messages. Cost: one file write per consumed batch. Acceptable.
- **Extract `livecheck` at the 5th caller, not the 3rd** — earlier extractions would have ossified an unstable shape. Mattaññutā.
- **Dashboard `t` requires `$TMUX`** — no in-process terminal emulator; integrate with the user's existing tmux session instead of reinventing.
- **`bwoc chat` has a `--tmux` flag instead of a separate verb** — both shapes (exec-in-place vs new-window) are common needs; a flag is cheaper than a second command.

## Alternatives considered

- **One commit per cluster vs one big "Phase 2 surface" commit** — went with one commit per logical change for `git log` clarity. CHANGELOG groups them.
- **Promote `signal_zero_alive` to `bwoc-core`** — rejected. Liveness checking is a CLI concern, not a core type. Per-crate placement matches dependency direction (CLI may use core; core must not depend on CLI semantics).
- **TUI dashboard runs an in-process PTY for chat** — rejected. Reimplementing a terminal emulator is large surface area. tmux integration shells out to a battle-tested tool.

## Status / deferred

All 12 commits are now backfilled. Closes [gap-analysis](2026-05-22_gap-analysis-remediation.md) item 3.

Deferred to follow-up:

- CLI integration smoke test (gap-analysis item 4).
- TH localization of lifecycle command messages (stop/start/retire/ping/send/inbox/chat use English literals; not yet Fluent-wired).
- Real process supervision (restart-on-crash, health-check loop) — currently the daemon exits cleanly on signal; auto-respawn is Phase 2 remaining work.
- Code signing for release binaries — needs Apple Developer + Windows Authenticode certs from the user.

## Related

- Plan: [gap-analysis remediation](2026-05-22_gap-analysis-remediation.md)
- Prior note: [bwoc-new UX + framework hygiene](2026-05-22_bwoc-new-ux-and-framework-hygiene.md)
- Roadmap: [`docs/en/ROADMAP.en.md`](../docs/en/ROADMAP.en.md) §"Shipped in Phase 2" + "Shipped in Phase 3"
- Changelog: [`CHANGELOG.md`](../CHANGELOG.md)
