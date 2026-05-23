//! Policy / permission system and safety guardrails.
//!
//! P2 component. Two sub-layers:
//!
//! **Guardrails** — hard policy engine that runs *before* permission checks
//! and cannot be overridden by the model or operator. Blocks `rm -rf` of
//! the repo root, secret commits, identity spoof, gate bypass flags
//! (`--no-verify`, `--force`, `-f`), and undeclared side-effects outside
//! task scope. Grounded in Sīla 5 + Taṇhā 3.
//!
//! **Permission system** — per-tool / per-pattern modes `allow | ask | deny`
//! loaded from `config.manifest.json` and `.bwoc/harness-policy.toml`.
//! `ask` mode prompts the operator on TTY; in non-TTY / autonomous mode
//! falls back to the policy default (deny). Denials are fed back to the
//! model as tool results so it can adapt.
//!
//! TODO: P2 — implement guardrails engine, permission loader, TTY prompt.
