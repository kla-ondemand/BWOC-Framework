//! Telemetry — per-turn metrics and session-level observability.
//!
//! P3 component. Collects tokens-in/out, latency, tool-call counts, gate
//! pass/fail counts, and appends records to `session-metrics.jsonl`
//! (schema defined in AGENTS.md §8b).  Optional OpenTelemetry export behind
//! a feature flag.
//!
//! TODO: P3 — implement metric collection, JSONL append, OTEL export feature flag.
