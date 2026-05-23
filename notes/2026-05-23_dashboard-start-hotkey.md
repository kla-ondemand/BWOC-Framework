---
date: 2026-05-23
session: dashboard `s` hotkey — run (start) selected agent
tags:
  - phase/3
  - type/note
  - module/bwoc-cli
---

# 2026-05-23 — Dashboard `s` Hotkey: Run an Agent

The dashboard could chat (`t`/`g`), tail logs (`l`), and watch inbox (`i`) for the selected agent, but it couldn't **run** one — starting a daemon meant dropping to the shell for `bwoc start`. This iter adds `s` = start (run) the selected agent from inside the TUI.

## What changed

- **`crates/bwoc-cli/src/dashboard.rs`** — new `s` hotkey + `start_selected_agent(app)`. Shells out to `bwoc start <id> --yes --json --workspace <root>` with output **captured** (`.output()`, not `.status()`) so the daemon's stdout/stderr never corrupts the alt-screen TUI. Parses the JSON (`already_running`, `daemon_pid`) for a precise `last_action` footer message, then calls `app.refresh()` so the row status + ●/○ runtime indicator flip immediately.
- Help overlay (`?`) gains an `s` row; footer legend gains `s start` (between `i inbox` and `r refresh`).

## Decisions

- **Shell out, don't call `start::run` in-process.** `start::run` prints to stdout and would corrupt the TUI; capturing in a subprocess is the same pattern `open_in_tmux`/`open_in_ghostty` already use. The daemon is spawned **detached** by `bwoc start` (it `Command::spawn`s `bwoc-agent --serve` with null stdin/stdout + log-file stderr and never waits), so it survives the short-lived captured child.
- **`--json` for a clean message.** `bwoc start --json` (requires `--yes`) emits `{ agent, already_running, daemon_pid, daemon_spawned, registry_updated, workspace }`. Parsing it gives the operator "started 'agent-x' (daemon pid N)" or "already running" instead of a generic blip. Falls back to a generic message if parsing fails.
- **`s` = start only, not start+stop.** The literal ask was "run agent-* in dashboard" → start. Stop (`x`) is the symmetric pair and a trivial mirror, but per the over-engineering rule (finish what's asked, surface what's next) it's deferred to an explicit request rather than chained in.
- **Captured output over a new window.** Unlike chat/log/inbox (which open *interactive* sessions in new tmux/Ghostty windows), starting a daemon is a fire-and-forget action with no interactive surface — capturing the result inline + refreshing the dashboard is the right UX, no new window.

## Alternatives considered

- **`x` (stop) + `s` (start) together** — symmetric and cheap, but unasked. Deferred.
- **In-process `start::run`** — rejected (TUI corruption via stdout; would need to redirect/restore the terminal around the call).
- **Spawn into a tmux window like `t`** — rejected; the daemon has no interactive surface to attach to. The log is tailable separately via `l`.

## Status / deferred

- **`x` = stop the selected agent** — obvious follow-up; same shell-out shape (`bwoc stop <id> --yes`). Add on request.
- **`bwoc start` idempotency quirk surfaced (not introduced here)** — calling `bwoc start <id> --yes --json` twice in rapid succession spawned a *second* daemon (new pid) instead of reporting `already_running: true` the second time. Likely a race: the first daemon writes `.bwoc/agent.pid` slightly after `Command::spawn` returns, so the second start's liveness check (`signal_zero_alive` reading the pid file) doesn't see it yet. Out of scope for the dashboard hotkey — the hotkey faithfully shells out to whatever idempotency `bwoc start` provides. Worth a dedicated fix: have `spawn_daemon` write the PID file synchronously before returning, or have `bwoc start` poll briefly for the pid file. Tracked here; not fixed this iter.
- **`--all` from dashboard** — "run agent-*" could be read as mass-start. Not implemented; the dashboard operates on the *selected* agent by convention (matches `t`/`l`/`i`/`g`). Mass-start stays a CLI op (`bwoc start --all --yes`).

## Test summary

- `cargo build -p bwoc-cli`, `cargo clippy -p bwoc-cli --all-targets -- -D warnings`, `cargo test -p bwoc-cli` — all clean (88 tests).
- Live-verified the exact shell-out the hotkey runs: in a scratch workspace, `bwoc start agent-beta --yes --json --workspace .` emits `{"agent":"agent-beta","already_running":false,"daemon_pid":12308,"daemon_spawned":true,...}` — the shape `start_selected_agent` parses. Scratch daemons cleaned up; no orphans.
- TUI hotkey path itself is not unit-testable (raw-mode event loop); verified by code trace + the underlying command's live output.

## Related

- `crates/bwoc-cli/src/dashboard.rs` — `start_selected_agent`, hotkey dispatch, help overlay, footer
- `crates/bwoc-cli/src/start.rs` — `bwoc start` + `spawn_daemon` (the detached daemon spawn)
- Prior dashboard launchers: `2026-05-23_chat-ghostty-launcher.md` (`g` hotkey)
