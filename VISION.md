# Vision

## Purpose

BWOC exists so that anyone can build AI coding agents on a **principled, backend-neutral foundation** — one that treats state impermanence, failure tracing, lifecycle, inter-agent trust, and threat modeling as first-class concerns, not afterthoughts.

## What BWOC Models

BWOC does not model "code being written." It models the **entire arc of a coding agent** as a conditioned phenomenon — what arises, persists, and ceases.

This arc has three phases, named with the canonical Sutta triad from **AN 3.47 (Saṅkhata Sutta)**:

| Phase | Pali | What the agent does |
|---|---|---|
| Arising | **uppāda** | Identity is created. The agent is incarnated from the template; persona is set; capabilities are declared; manifest is resolved. |
| Persisting (with change) | **ṭhiti** | The agent operates. It plans by Ariyasacca 4, acts by Magga 8, remembers by Sammā-sati, communicates by Brahmavihāra 4 — all within the bounded life of a task or session. |
| Passing-away | **vaya** | The action ends. Worktree is cleaned, branch released, memory pruned, task closed. Anattā — no clinging. |

This arc is the architectural shape of every BWOC artifact: a single task, a single session, a single agent's whole lifespan. The 22 frameworks in [`modules/agent-template/docs/en/PHILOSOPHY.en.md`](modules/agent-template/docs/en/PHILOSOPHY.en.md) all operate inside one of these three phases.

## The Gap

Western engineering frameworks (DDD, Clean Architecture, SOLID, Hexagonal) are thorough on structure and dependency. They are thin on:

- **What a system *is* over time** when state is constantly mutating.
- **Why** a failure happened, traced backward to its conditions.
- **How** independent agents trust and coordinate without a central authority.
- **What** an agent should refuse to do, by default, with no rule-author present.

Agent systems fail in exactly these dimensions. BWOC adopts Buddhist frameworks because they are unusually precise about impermanence, dependent origination, intent, and discipline — the same concerns, in a different vocabulary.

## Approach

- **Pali terms are section names. Content is technical.** No religious interpretation. A reader who has never heard of Buddhism can read the docs and ship code.
- **One specification, many backends.** `AGENTS.md` is the single source of truth; Claude, Antigravity, Codex, Kimi, and any future LLM read it via symlinks. No backend is favored.
- **One repository, one agent.** Each agent is a self-contained, forkable repo cloned from the template. No central runtime.
- **Documents before implementation.** The framework is fully specified in Markdown before any runtime code is written. Code follows doctrine, not the reverse.

## Success in One Year

- A public contributor can clone the template, fill `config.manifest.json`, and have a working agent profile in under 30 minutes.
- At least three reference agents exist in the wild, built by maintainers outside the original authors.
- The 22 framework mappings have been stress-tested against real agent incidents, not just designed at a whiteboard.
- Translations exist in at least three human languages beyond English, using the language-agnostic `docs/<lang>/` structure.

## Success in Three Years

- BWOC vocabulary (Yoniso manasikāra checks, Mattaññutā caps, Sīla baselines, Kalyāṇamitta trust scores) appears in agent codebases that have no formal affiliation with this project.
- A cross-vendor fleet governance pattern (Aparihāniya-dhamma 7) is in production at more than one organization.
- The framework has survived its own first major refactor without breaking the doctrine layer.

## Self-Hosted and Provider-Neutral Agents

BWOC treats **self-hosted LLMs as a first-class deployment target**, not an afterthought. The `bwoc-harness` crate adds an OpenAI-compatible model-API client and agentic loop runtime, enabling agents that run entirely on local or private infrastructure (Ollama, vLLM, LM Studio, llama.cpp server, or any OpenAI-compatible endpoint).

### The Orchestrator-vs-Runtime Split

The default `bwoc` path remains a **zero-dep CLI orchestrator**: it exec's a vendor agentic CLI (Claude, Antigravity, Codex, Kimi) and makes no model-API calls. Users who only work with cloud backends never pull a runtime.

The optional `bwoc-harness` path adds a **model-API client and agent runtime** with production-grade safety (guardrails, sandbox, permission system), telemetry, and an offline eval framework. Its heavy dependencies (tokio, reqwest, keyring) are **quarantined inside `crates/bwoc-harness`** and do not affect the default crate graph. Adding a self-hosted backend is one symlink: `ln -s AGENTS.md OLLAMA.md`.

This is a deliberate expansion from "pure orchestrator" to "orchestrator + optional runtime," justified because self-hosted operation requires control over safety, telemetry, and evaluation that a third-party agentic CLI cannot provide. The smaller spec still wins everywhere the expansion is not load-bearing.

## Non-Goals

- **Not a religion.** Not a meditation guide. Not a vehicle for any teacher, lineage, or tradition.
- **Not a replacement for DDD, Clean Architecture, or SOLID.** BWOC extends those frameworks into dimensions they were never designed to address.
- **Not a locked-in runtime.** The default `bwoc` path is still a CLI orchestrator with no model-API dependency. The `bwoc-harness` runtime is opt-in, not mandatory.
- **Not vendor-aligned.** No backend, hosting provider, vector store, or tool gets preferential treatment in the core documents. Self-hosted parity with cloud backends is a design goal, not an afterthought.
- **Not a productivity framework.** BWOC optimizes for *principled, auditable, recoverable* agents — not for the fastest possible time-to-first-token.

## Principles That Govern Hard Tradeoffs

When two good ideas conflict, these decide:

1. **Samānattatā** — equal treatment of all backends beats any per-vendor convenience.
2. **Mattaññutā** — the smaller specification beats the more complete one, unless completeness is load-bearing.
3. **Yoniso manasikāra** — verified against current state beats remembered from past state.
4. **Sīla** — communal safety baseline beats individual flexibility.
5. **Anattā** — releasing stale state beats preserving it "in case."

When in doubt, choose the option a public contributor with no prior context can adopt without asking permission.
