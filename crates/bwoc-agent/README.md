# bwoc-agent

Minimal runtime shipped with each incarnated [BWOC](../../README.md) agent.

Native single binary for **macOS · Linux · Windows**. Spawned by [`bwoc spawn`](../bwoc-cli/) from inside an incarnated agent directory.

## Scope by phase

| Phase | Responsibility |
|---|---|
| **1 v2.0 (current)** | Read `config.manifest.json` in the current directory; print `I am alive: <agentId>`; exit 0. Proves the binary distribution pipeline. |
| **2** | Open a control socket for `bwoc send` / `bwoc status`. Run the Ariyasacca-4 task loop (Dukkha → Samudaya → Nirodha → Magga) and append entries to `task-log.jsonl`. |
| **3** | Inter-agent messaging (Sammā-vācā). Lifecycle release on `bwoc retire` (vaya). |

## Usage

Typically invoked by `bwoc spawn`, not directly. For local development inside an incarnated agent directory:

```bash
cd path/to/agent-<name>
bwoc-agent
```

Phase 1 output:

```
bwoc-agent (Phase 1 v2.0 scaffold) — runtime stub
```

## Status

**Phase 1 v2.0 — scaffold.** Real responsibilities (manifest read, task loop, control socket) arrive in follow-up iterations.

## License

[MIT](../../LICENSE).
