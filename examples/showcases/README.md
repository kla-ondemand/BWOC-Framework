# Showcases

Full reference agents — incarnated, with persona / memories / skills slots filled in — that demonstrate concrete BWOC patterns.

## Status: planned

This directory is currently a placeholder. Showcase agents are bigger artifacts than how-to recipes — each is essentially a complete agent tree (~50 files) checked into the framework as living example.

## What we plan to ship here

Each showcase will be a self-contained directory under `examples/showcases/<agent-name>/` containing the full incarnated tree:

- `AGENTS.md` (with placeholders filled in — not the template's `{{agentId}}` form)
- `config.manifest.json` (concrete values for one declared backend)
- `persona/README.md` — identity, domains, boundaries
- `memories/MEMORY.md` + a few example memory entries
- `skills/` and `mindsets/` with at least one populated `SKILL.md` each
- A top-level `README.md` explaining what this showcase is for and how it was built

Planned showcases (none shipped yet):

| Agent | Domain | Demonstrates |
|---|---|---|
| `documentor-agent/` | Documentation writing | PHILOSOPHY-grounded persona, write-then-review workflow, central memory references |
| `code-reviewer-agent/` | Code review | Multi-mindset approach, strict neutrality across all 4 backends, deep-memory tooling |
| `onboarding-agent/` | New-hire orientation | Long-form persona, indexed memory, "session continuity" pattern |
| `migration-agent/` | Database/data migrations | Multi-phase agent (one per migration step), interconnect handoff (Phase 3) |

## Why nothing's here yet

Showcases are content, not framework. They need:

- A real use case (so the agent's persona/memories aren't fictional)
- Reviewed by humans for backend-neutrality (no Anthropic-specific phrasing in the showcase, etc.)
- Bilingual EN/TH parity if added to the framework proper (per the bilingual HARD RULE)

Until those constraints are met, we leave the placeholder rather than ship something thin.

## How to contribute a showcase

1. Build the agent in your own workspace
2. Verify it passes `bwoc check` against all 4 backends
3. Strip any personal/proprietary content from persona and memories
4. Submit as a PR adding `examples/showcases/<your-agent>/` with the full tree + a `README.md` explaining the use case and decisions

See `CONTRIBUTING.md` for the workflow.

## See instead

- [`examples/howto/`](../howto/) — runnable recipes (active)
- [`modules/agent-template/`](../../modules/agent-template/) — the template every agent starts from
