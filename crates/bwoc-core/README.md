# bwoc-core

Shared types for the [BWOC framework](../../README.md) — manifest, identity, lifecycle phases.

No I/O. Pure data types and small helpers consumed by [`bwoc-cli`](../bwoc-cli/) and [`bwoc-agent`](../bwoc-agent/).

## Scope

- **`manifest`** — `config.manifest.json` schema (Phase 1: stub).
- **`identity`** — `AgentId`, `Capability` (Phase 1: stub).
- **`lifecycle`** — `LifecyclePhase { Uppada, Thiti, Vaya }`. The three phases of the BWOC arc, named per AN 3.47 Saṅkhata Sutta. See [`PHILOSOPHY.en.md` §0.1](../../modules/agent-template/docs/en/PHILOSOPHY.en.md#01-the-arc--uppāda--ṭhiti--vaya).
- **`error`** — shared error types (Phase 1: stub).

## Usage

In another crate within the workspace:

```toml
[dependencies]
bwoc-core = { workspace = true }
```

```rust
use bwoc_core::lifecycle::LifecyclePhase;

let phase = LifecyclePhase::Uppada;
```

## Status

**Phase 1 v2.0 — scaffold.** Module surface is declared; implementations land as the CLI and runtime are built out.

## License

[MIT](../../LICENSE).
