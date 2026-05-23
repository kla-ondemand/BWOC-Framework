---
date: 2026-05-23
session: `bwoc chat --ghostty` + dashboard `g` hotkey
tags:
  - phase/3
  - type/note
  - module/bwoc-cli
---

# 2026-05-23 — Ghostty Launcher for `bwoc chat`

`bwoc chat` and the dashboard already had `--tmux` / `t` to open the
backend CLI in a fresh tmux window. This iter adds the same shape for
Ghostty: a `--ghostty` flag on `bwoc chat` and a `g` hotkey in the
dashboard. macOS-only; non-macOS exits 2 with a hint.

## What changed

- **`crates/bwoc-cli/src/chat.rs`** — `ChatArgs` gains `ghostty: bool`;
  new `open_in_ghostty(agent_id, agent_path, backend)` runs
  `open -na Ghostty.app --args --working-directory=<p> -e bwoc spawn --path <p> --backend <b>`.
  Returns exit 2 on non-macOS without invoking `open`.
- **`crates/bwoc-cli/src/main.rs`** — clap `ChatArgs` gains
  `--ghostty` flag with `conflicts_with = "tmux"` so the two launchers
  are mutex at the parser level.
- **`crates/bwoc-cli/src/dashboard.rs`** — `g` hotkey added next to
  `t`/`l`/`i`; new `open_in_ghostty(app)` mirrors `open_in_tmux`
  (selection guard → workspace guard → macOS guard → exec). Help
  overlay row added below the existing tmux rows.

## Decisions

- **Mirror `--tmux`, don't generalize.** Considered a `--window=<tmux|ghostty|terminal>`
  enum to anticipate Terminal.app / iTerm2 / Alacritty. Rejected for
  this iter — Mattaññutā says ship what's asked. The two-flag shape
  costs less than a config enum that nobody else has yet asked for;
  promote to enum when a third launcher gets requested.
- **macOS-only without `#[cfg]` gating.** Runtime `cfg!(target_os = "macos")`
  check instead of compile-time `#[cfg(target_os = "macos")]` so the
  binary still compiles on Linux/Windows and emits a useful hint
  rather than a silent missing-symbol. The hint redirects to the
  manual `ghostty -e` invocation that does exist on those platforms.
- **`open -na Ghostty.app` over `/Applications/Ghostty.app/Contents/MacOS/ghostty`.**
  Ghostty's own `--help` says explicitly: "On macOS, launching the
  terminal emulator from the CLI is not supported. Use `open -na
  Ghostty.app`." Following the upstream-documented path means we
  inherit their compatibility guarantee instead of taking on the
  responsibility of pointing at the macOS-internal binary path.
- **`-n` (always new instance) over `-a` (reuse).** `-n` forces a
  new Ghostty window even when one is already open. Without `-n`,
  `open` would reuse the front window which would obliterate whatever
  the operator was doing there. The cost is a heavier launch; the
  benefit is no accidental clobber.
- **`g` for the hotkey, not `G`.** Stays consistent with `t`/`l`/`i`
  (single lowercase keys). The Shift-modified alternates are still
  free for future use (e.g., `G` for "ghostty + log -f" if anyone
  asks).
- **Recursive use of `bwoc spawn`, not `bwoc chat`.** The new window
  runs `bwoc spawn --path <p> --backend <b>` directly rather than
  `bwoc chat <agent>`. Reason: `chat` already resolved the path and
  backend from the registry, so passing them through to `spawn`
  saves the new window's `bwoc chat` from re-reading the registry.
  Also avoids any "loop into another launcher" risk.

## Alternatives considered

- **Single `--window=<tmux|ghostty>` enum.** Rejected — see above
  (Mattaññutā). The two-flag shape is what `bwoc chat` already has;
  changing it would be a breaking interface shift for `--tmux` users.
- **`Ghostty.app` path discovery.** Considered walking `/Applications`
  + `~/Applications` to find Ghostty before invoking. Rejected —
  `open -na` already does this discovery for free via LaunchServices
  and emits a clear error if the app isn't found. Re-implementing
  that lookup adds code without value.
- **Dashboard hotkey as Shift+T instead of `g`.** Considered keeping
  "T = tmux, Shift-T = Ghostty" as a "same action, different
  launcher" pairing. Rejected — Shift-modified keys conflict with
  ratatui's KeyModifiers and would need explicit modifier handling
  in the match arm. Lowercase `g` is simpler and the mnemonic is
  better.
- **Open log / inbox in Ghostty too (`L` / `I` capitalized?).**
  Deferred. The original ask was "open agent" (chat). If operators
  ask for ghostty-flavored log / inbox we add `L` / `I` (or
  `g`-prefixed two-key chord). Not building on speculation.

## Bugs surfaced and fixed

- **None.** Clean run: `cargo build`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo test --workspace` (115
  passing). Live verified: `bwoc chat --ghostty agent-pi` opens a
  fresh Ghostty window in `agents/agent-pi/` running `claude` (the
  backend resolved from the agent's registry entry).

## Status / deferred

- **Linux/Windows path.** When Ghostty's Linux build matures, the
  `cfg!(target_os = "macos")` guard can be relaxed to call
  `ghostty -e` directly. Today's stub error message already names
  that escape hatch.
- **Backend-aware extra args.** `bwoc spawn` accepts `-- <args>...`
  to pass arguments to the backend CLI. The Ghostty launcher doesn't
  currently surface that — the call hardcodes `--path` + `--backend`
  only. Add when an operator actually needs to pass backend args
  through a fresh window.
- **iTerm2 / Terminal.app / Alacritty parity.** Same shape as the
  Ghostty addition would work, but Mattaññutā says wait for the
  request. The decision to use two flags instead of a `--window=`
  enum kept this option open without committing to it.

## Test summary

- bwoc-cli: 15 + 81 + 1 = 97 tests passing (no new tests added;
  existing pattern in `chat.rs` and `dashboard.rs` has no unit
  tests because these functions hit the OS — clap mutex enforcement
  verified live via `bwoc chat --tmux --ghostty agent-pi` which
  fails with the expected parser error).
- bwoc-core: 18 tests passing (unchanged).
- bwoc-agent: 15 tests passing (unchanged).
- Workspace total: 115 tests, 0 failures. Clippy clean.

Live verification:
1. `bwoc chat --help` shows `--ghostty` alongside `--tmux`.
2. `bwoc chat --tmux --ghostty agent-pi` → exit 2 (clap mutex).
3. `bwoc chat --ghostty agent-pi` → new Ghostty window, `bwoc
   spawn --path agents/agent-pi --backend claude` runs inside,
   stderr in the calling shell: `Opened Ghostty window for
   'agent-pi' (backend: claude)`.
4. `bwoc dashboard` + `g` hotkey: new Ghostty window opens with
   the selected agent (verified `agent-pi` and `agent-oracle`);
   `last_action` footer shows `→ Ghostty window for 'agent-pi'
   opened (backend: claude)`.

## Related

- [`crates/bwoc-cli/src/chat.rs`](../crates/bwoc-cli/src/chat.rs) — new `open_in_ghostty`.
- [`crates/bwoc-cli/src/dashboard.rs`](../crates/bwoc-cli/src/dashboard.rs) — `g` hotkey + dashboard `open_in_ghostty`.
- [`crates/bwoc-cli/src/main.rs`](../crates/bwoc-cli/src/main.rs) — clap `--ghostty` flag.
- Ghostty docs: <https://ghostty.org/docs>
- Prior parallel launcher: `--tmux` (Phase 2 shipped table — `bwoc chat`).
