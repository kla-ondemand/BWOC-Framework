# 2026-05-22 — Runtime TUI Plan

Proposal for a Terminal User Interface over BWOC runtimes. Status:
**plan only**, no code yet. Approval requested before any iter starts.

## What "runtime" means here

Three candidate scopes, smallest first:

1. **`bwoc dashboard` (multi-agent overview)** — top-level command that
   reads the workspace's `agents.toml` and renders an interactive list:
   pick an agent, see its manifest, recent log lines, and process state.
   Read-mostly; no control verbs in v1.
2. **`bwoc-agent` interactive mode** — instead of just printing the
   liveness banner and exiting, hold the screen and show live log /
   manifest / health for **one** agent. Hot-reload on manifest change.
3. **`bwoc spawn --tui`** — wrap the spawned backend CLI (claude /
   gemini / codex / kimi) in a TUI shell with side panels showing
   agent identity, persona, recent memory. Most ambitious; needs PTY
   multiplexing and could fight the backend's own prompt UI.

Recommendation: **start with option 1 (`bwoc dashboard`)**. It's
self-contained, read-only, and proves the TUI stack works before
we put it in a hot path. Option 2 is a natural follow-up that
reuses most of option 1's primitives. Option 3 is Phase 3+ work.

## Library choice

Use `ratatui` (the actively maintained fork of `tui-rs`) with the
`crossterm` backend. Reasons:

- Crossterm is pure-Rust, works on macOS / Linux / Windows uniformly.
  Matches the framework's existing cross-platform commitment.
- Ratatui's immediate-mode model fits well with our "read manifest +
  agents.toml on each render" pattern — no retained state to
  invalidate.
- Both are MIT/Apache, no surprise deps. Adds roughly
  `ratatui` + `crossterm` + `unicode-width` (also useful for the
  byte-count alignment caveat noted in iter 18) + small transitive
  graph. Manageable.

Alternative considered: `cursive`. Rejected because it's less
actively maintained and ratatui is now the Rust-TUI canonical
choice; reaching for it later would be churn.

## MVP — `bwoc dashboard` v1

Scope: read-only, single-pane → detail view.

Surface:
- **Main pane**: list of agents from `agents.toml`. Columns:
  `id`, `status`, `backend`, `path`. Highlighted row = current
  selection.
- **Detail pane** (right or bottom, layout-dependent on width):
  selected agent's manifest fields (`agentRole`, `primaryModel`,
  `fallbackModel`, `memoryPath`, `version`) + computed status
  (does the agent dir exist? AGENTS.md present? symlinks intact?
  — same probes as `bwoc doctor`).
- **Status bar**: workspace path, agent count, framework version.
- **Footer**: hotkey legend (`q` quit, `↑↓` navigate, `r` refresh,
  `?` help).

Out of scope for MVP:
- Sending messages to agents (Phase 3 vaya/interconnect work).
- Killing processes / process supervision (Phase 2 control socket).
- Live log tail (needs a log file convention first; deferred).
- Search / filter (one-line addition once the list grows).

Acceptance:
- Runs from any workspace subdir (ancestor walk to find `.bwoc/`).
- Same workspace-resolution chain as `list` / `info` / `validate`
  (explicit > `BWOC_WORKSPACE` env > ancestor walk > error).
- Quits cleanly on `q`, `Ctrl-C`, and SIGINT/SIGTERM — restores
  terminal state. No "broken terminal" after crash.
- Non-TTY invocation (pipes, CI): refuses with an exit-2 hint
  pointing at `bwoc list` for non-interactive use.

## Phased rollout

Each phase is its own iter / commit. Stops at any boundary if the
work looks heavier than expected.

- **Phase 0** — add `ratatui`/`crossterm` workspace deps; new
  `crates/bwoc-cli/src/dashboard.rs` with a "hello, world" TUI
  that draws a single bordered block and exits on `q`. Wire as
  `bwoc dashboard` subcommand. Proves the stack, ~50 LOC.
- **Phase 1** — populate the main pane with the agents.toml table.
  Navigation, highlight, refresh. Read-only. ~120 LOC.
- **Phase 2** — detail pane with manifest fields + `doctor`-style
  status probes (reuse the existing `doctor` checks). ~80 LOC.
- **Phase 3** — i18n the labels via the existing Fluent bundle.
  Lang resolved same as elsewhere. ~30 LOC + FTL keys.
- **Phase 4** — log tail (when a log convention exists for agents)
  and "open in $EDITOR" for the manifest. Optional; cuts here
  unless requested.

## Open questions (please decide before Phase 0)

1. **Is `bwoc dashboard` the right name?** Alternatives: `bwoc tui`,
   `bwoc ui`, `bwoc watch`. "dashboard" is most descriptive;
   "tui" is jargon; "watch" implies polling.
2. **Should `bwoc` with no subcommand keep the current banner, or
   launch the dashboard if a workspace is found?** The banner is
   the safe default; auto-launch is friendlier but surprising.
3. **Color scheme?** The startup banner uses bold-yellow on dark.
   The dashboard should match for visual continuity. Anything
   specific you want (saffron tone for Buddhist motif)?
4. **Does the "TUI for runtimes" framing actually include option 2
   (`bwoc-agent` interactive)?** That's a bigger change with
   real implications (long-running process, signal handling). If
   yes, it's a separate plan doc — let me know.
5. **Cross-cron interaction**: cron `c045cc3d` is firing every
   5 min on the "improve CLI" loop. Should I pause that cron
   while the dashboard work is in flight, or keep both going
   (dashboard work as foreground, small UX picks from the cron
   in the background)?

## Estimated total

Phase 0 + 1 + 2 + 3 = roughly **300–400 LOC** + a `ratatui` dep.
A focused day's work split across 3–4 iters. Stops graceful at
any phase boundary if you want to pivot.

## Related

- ROADMAP Phase 2 §"bwoc-agent control socket" — the eventual
  bidirectional channel this TUI would talk to in option 2/3.
- `crates/bwoc-cli/src/banner.rs` — existing static styled-stdout
  rendering. The dashboard is the interactive evolution.
- `crates/bwoc-cli/src/doctor.rs` — the diagnostic checks the
  dashboard's detail pane would reuse verbatim.
- `crates/bwoc-cli/src/workspace.rs` §`find_workspace_root` — the
  ancestor walk the dashboard's workspace resolution would mirror.
