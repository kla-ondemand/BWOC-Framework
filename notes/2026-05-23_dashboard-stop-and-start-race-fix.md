---
date: 2026-05-23
session: dashboard `x` stop hotkey + bwoc start daemon-spawn race fix
tags:
  - phase/3
  - type/note
  - module/bwoc-cli
---

# 2026-05-23 — Dashboard `x` (Stop) + `bwoc start` Race Fix

Follow-up to [`2026-05-23_dashboard-start-hotkey.md`](2026-05-23_dashboard-start-hotkey.md), which shipped `s` (start) and explicitly deferred the symmetric `x` (stop) plus surfaced a `bwoc start` daemon-spawn race. This iter closes both.

## What changed

- **`crates/bwoc-cli/src/dashboard.rs`** — new `x` hotkey + `stop_selected_agent(app)`, mirroring `start_selected_agent`. Shells out to `bwoc stop <id> --yes --json --workspace <root>` with captured output, parses the real `daemon_outcome` field (`not_running` | `socket_ok` | `sigterm` | `sigkill` | `could_not_kill`) into a precise `last_action` message, then refreshes so the row status + ●/○ flip. Help overlay + footer legend gain the `x` binding.
- **`crates/bwoc-cli/src/start.rs`** (`spawn_daemon`) — after `Command::spawn` returns the child pid, the parent now writes `.bwoc/agent.pid` with that pid immediately, instead of leaving the pid file to the daemon's own startup. Closes the rapid-double-start race.

## Decisions

- **Parent pre-writes the pid file with `child.id()`.** The daemon writes `.bwoc/agent.pid` from its own `std::process::id()` during `--serve` startup (bwoc-agent/src/main.rs:142), which lags `spawn()` by milliseconds. A second `bwoc start` in that window reads no pid file → "not running" → duplicate daemon. `child.id()` *is* the daemon's pid, so the parent writing it is identical to what the daemon writes later — idempotent, no conflict. Best-effort (`let _ =`): a write failure just reopens the race, it doesn't break the spawn, so the error isn't propagated.
- **Parse `daemon_outcome`, not an invented `was_running`.** First draft of `stop_selected_agent` guessed a `was_running` boolean; the actual `bwoc stop --json` shape (stop.rs:262) is `{ workspace, agent, daemon_outcome, registry_updated }`. Read the source, matched the real five-value enum, gave each a distinct footer message (incl. a `⚠` for `could_not_kill`). Yoniso manasikāra — verified against the code, not assumed.
- **`x` mirrors `s` exactly.** Same selection/workspace guards, same captured-shell-out pattern, same refresh-on-success. Symmetry keeps the two functions trivially reviewable side by side.

## Alternatives considered

- **Poll for the pid file in `spawn_daemon`** (sleep-loop up to ~500ms) instead of pre-writing — rejected: adds latency to every start and is still racy at the margin. Pre-writing the known pid is deterministic and instant.
- **Fix the race in `bwoc-agent` (write pid file earlier in startup)** — would shrink but not eliminate the window (there's always *some* gap between fork and the daemon's first write). The parent-side write closes it fully because the parent has the pid before the child runs any code.
- **`x` opens a confirm dialog in the TUI** — rejected; `bwoc stop --yes` already encodes the confirmation, and a modal would be heavier than the action warrants. The footer message is the feedback.

## Bugs surfaced and fixed

- **`bwoc start` duplicate-daemon race** — FIXED. Live-verified: in a scratch workspace, `bwoc start agent-gamma --yes --json` twice in immediate succession now returns `already_running: true` on the second call (previously spawned a second daemon with a fresh pid). Stop afterward returns `daemon_outcome: socket_ok` (graceful STOP via the control socket), confirming only one daemon existed.

## Status / deferred

- **No new gaps.** The dashboard now covers the full single-agent lifecycle the operator needs live: chat (`t`/`g`), log (`l`), inbox (`i`), start (`s`), stop (`x`), refresh (`r`).
- **Mass start/stop from the dashboard** (`--all`) still stays a CLI op by design — the dashboard operates on the selected agent, consistent with every other hotkey.

## Test summary

- `cargo build -p bwoc-cli`, `cargo clippy -p bwoc-cli --all-targets -- -D warnings`, `cargo test -p bwoc-cli` — all clean (88 tests).
- Live verification (scratch workspace, `agent-gamma`):
  1. `start #1` → `pid 33436, spawned true, already false`
  2. `start #2` (immediate) → `pid null, spawned false, already true` ← race fixed
  3. `stop --json` → `daemon_outcome: socket_ok`
  - No orphan daemons after cleanup.

## Related

- Prior: [`2026-05-23_dashboard-start-hotkey.md`](2026-05-23_dashboard-start-hotkey.md) (the `s` hotkey + where this race was first surfaced)
- `crates/bwoc-cli/src/dashboard.rs` — `stop_selected_agent`, hotkey dispatch, help, footer
- `crates/bwoc-cli/src/start.rs` — `spawn_daemon` pid pre-write
- `crates/bwoc-cli/src/stop.rs` — `daemon_outcome` JSON shape
- `crates/bwoc-agent/src/main.rs:142` — the daemon's own pid-file write
