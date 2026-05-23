# 2026-05-23 — CI back to green (fmt drift + Windows compile)

The `CI` workflow had been red on every recent push. Two independent causes, both fixed this session.

## What changed

- **fmt gate** — ran `cargo fmt --all`. Six source files had drifted (`task_watch.rs`, `dashboard.rs`, `main.rs`, `sangha.rs`, `start.rs`, `stop.rs`, `team.rs`). Pure formatting, no behavior change.
- **Windows build** — `crates/bwoc-cli/src/check.rs` had an ungated `std::os::unix::fs::symlink` in the `write_temp_agent` test helper, breaking the `build + test (windows-latest)` matrix leg (E0433). Gated the helper and its three incarnation-audit tests behind `#[cfg(unix)]`.

## Decisions

- **Incarnation-audit tests are Unix-only by design.** They exercise real backend symlinks → `AGENTS.md`. Production Windows symlink support is explicitly deferred to Phase 2 (`new.rs::create_symlinks` returns an error there), so faking symlinks on Windows would test a path that doesn't exist. `#[cfg(unix)]` matches reality rather than reindenting into a copy-based fallback. (Yoniso Manasikāra — gate to what's actually true.)
- Verified the Windows leg locally via `cargo check --target x86_64-pc-windows-msvc -p bwoc-cli --all-targets` rather than waiting on CI round-trips.

## Bugs surfaced and fixed

- The fmt drift is recurring: the auto-version hook rewrites `.rs` files on edit but nothing reformats them, so commits land unformatted and red-line the fmt gate. Worth a pre-commit `cargo fmt` if it keeps recurring — not added this session (Mattaññutā; no user ask).
- The `fail-fast: false` matrix meant the Windows compile error was masked by attention on the fmt failure — both were red the whole time.

## Status / deferred

- fmt + clippy + 91 unit tests green locally; Windows cross-check compiles clean.
- Concurrent foreign edits to `CONTRIBUTING.md` / `AGENTS.md` / `conventions.md` (trunk-based branching standard) were present in the tree and left untouched for their owner.
