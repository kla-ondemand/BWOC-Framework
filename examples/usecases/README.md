# Use Cases

Patterns for specific real-world scenarios — when do you reach for BWOC, and how do you shape your agents to fit the work?

## Status: planned

Active how-tos are in [`examples/howto/`](../howto/). Use cases are bigger — they describe **why** you'd structure a workspace a certain way, not the literal commands.

## What we plan to ship here

| Use case | What it covers | Status |
|---|---|---|
| `solo-developer.md` | One person, one machine, multiple projects — `~/.bwoc/`, project per dir, agents share central memory | Planned |
| `team-monorepo.md` | A team's monorepo with multiple stacks (TS frontend + Rust backend + Python data) — one agent per stack, shared workspace memory | Planned |
| `multi-backend-comparison.md` | Same agent persona, four backends side-by-side, comparing outputs across claude / gemini / codex / kimi | Planned |
| `documents-first-greenfield.md` | New project starting from PRD/SRS in `notes/`, agents help translate spec to code, the [Documents-first](../../docs/en/FAQ.en.md) discipline in practice | Planned |
| `legacy-migration.md` | Bringing a legacy codebase under BWOC piece-by-piece — `bwoc check` as a gate, persona built from existing conventions | Planned |
| `oss-maintainer.md` | OSS project using BWOC — agent helps triage issues, write release notes, draft PR reviews | Planned |

## Why nothing's here yet

Use cases need to be honest: they're worthless if they describe imaginary workflows. We'll write them as real adopters bring back patterns that work, not as speculative "you could do X" pieces.

If you're using BWOC for a scenario above and would write a 1-2 page use case based on your actual experience, please open an issue or PR.

## Difference vs how-tos

| | How-to (`howto/`) | Use case (`usecases/`) |
|---|---|---|
| Question answered | "How do I X?" | "When and why would I structure things like Y?" |
| Length | 1–2 pages | 2–4 pages |
| Code-to-prose ratio | High (mostly commands + expected output) | Lower (mostly patterns, decisions, tradeoffs) |
| Audience | New users following a recipe | Decision-makers shaping a workspace |
| Lifespan | Updates with the CLI surface | Updates rarely; tied to adoption patterns |

## See instead

- [`examples/howto/`](../howto/) — runnable recipes
- [`docs/en/ARCHITECTURE.en.md`](../../docs/en/ARCHITECTURE.en.md) — formal architectural decisions
- [`docs/en/FAQ.en.md`](../../docs/en/FAQ.en.md) — Q&A across many of these scenarios in summary form
