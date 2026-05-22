# `modules/cli/` — DEPRECATED

This module is being replaced as part of **Phase 1 v2.0**. The native Rust CLI now lives at:

- [`crates/bwoc-cli/`](../../crates/bwoc-cli/) — the `bwoc` binary (macOS · Linux · Windows; localized to TH and EN).
- [`crates/bwoc-core/`](../../crates/bwoc-core/) — shared types (manifest, identity, lifecycle).
- [`crates/bwoc-agent/`](../../crates/bwoc-agent/) — minimal runtime shipped with each incarnated agent.

This directory is kept as a deprecation stub. It will be removed once nothing in the framework references `modules/cli/`.

See the root [`README.md`](../../README.md#tech-stack) for the current tech stack.
