# `modules/`

Framework-level modules. Each subdirectory is a distinct concern with its own lifecycle.

| Module | Purpose | Status |
|---|---|---|
| [`agent-template/`](agent-template/) | The canonical blueprint copied into every new agent. Single source of truth for agent shape. | Ready |
| [`plugins/`](plugins/) | Pluggable framework extensions — Tier 2 memory backends, additional LLM-backend integrations beyond the four declared. | Planned |
| [`skills/`](skills/) | Framework-level skills — capabilities the framework recommends as a baseline for any agent. | Planned |
| [`cli/`](cli/) | Deprecated stub. Replaced by [`crates/bwoc-cli/`](../crates/bwoc-cli/). | Deprecated |

For the implementation runtime (Rust crates), see [`crates/`](../crates/). For specification documents, see [`docs/`](../docs/) and [`modules/agent-template/docs/`](agent-template/docs/).

## Adding a new module

A new top-level module is a strategic decision — it adds a long-term concern the framework commits to. Open an issue or RFC before adding one. Each module needs:

- A `README.md` describing purpose, scope, and status.
- A clear boundary with adjacent modules.
- A statement of how it composes with `agent-template/` (which all agents inherit from).
