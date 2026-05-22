---
title: Agent Memory System
aliases:
  - Memory
tags:
  - group/agents
  - type/memory
  - meta/template
---

# Agent Memory System

> [!abstract] Two-tier persistent memory — keeps context across sessions without re-discovery.

## Architecture

```
Tier 1 (file-based, always loaded)
  MEMORY.md              — index, ≤ 200 lines
  memories/*.md          — individual memory files

Tier 2 (deep backend, optional)
  {{deepMemoryCmd}}      — pluggable semantic search / vector store
```

## Memory Types

| Type | Purpose | When to Save |
|---|---|---|
| `user` | Who you work with | Role, preferences, expertise level |
| `feedback` | How to approach work | Corrections AND confirmations |
| `project` | What is happening | Goals, decisions, blockers, deadlines |
| `reference` | Where to find things | External URLs, dashboards, ticket boards |

## File Format

```markdown
---
name: descriptive-slug
description: one-line hook for relevance decisions
type: user | feedback | project | reference
created: 2026-05-22
updated: 2026-05-22
---

<content>

**Why:** <motivation>
**How to apply:** <when/where this applies>
```

## Rules

- **Verify before acting** — a memory naming a file or function is a past claim. Grep before trusting.
- **Save from success AND failure** — corrections are easy to notice; confirmed approaches are quieter but equally important.
- **Convert relative dates** — "Thursday" → `2026-05-22`.
- **Cap at 200 lines** — `MEMORY.md` index. Forces quality over volume.
- **Do not save** — code patterns (derivable from code), git history (use `git log`), anything in `AGENTS.md`.

## Tier 2 Interface

```bash
{{deepMemoryCmd}} wake-up                     # session start — emit context
{{deepMemoryCmd}} search "<query>"            # search past decisions
{{deepMemoryCmd}} mine <path> --mode convos   # persist at session end
```

Tier 2 is optional. Its absence does not break the agent.

## Session Lifecycle

**Start:** Load `MEMORY.md` → load relevant memories → load `task-log.jsonl` → verify claims against current code

**End:** Update `task-log.jsonl` → save discoveries → prune stale entries → optionally mine to Tier 2
