# BWOC Framework — Buddhist Way of Coding

A framework for building AI coding agents grounded in Buddhist philosophy as an engineering discipline.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Platforms](https://img.shields.io/badge/platforms-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg)](#tech-stack)
[![Docs](https://img.shields.io/badge/docs-EN%20%7C%20TH-blue.svg)](modules/agent-template/docs/)
[![Status](https://img.shields.io/badge/status-Phase%201%20v2.0-yellow.svg)](#status)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

Buddhist principles are used here as **engineering thinking aids** — not religious interpretation. Pali terms are section names; the content is technical.

> Conceptual core: [`PHILOSOPHY.en.md`](modules/agent-template/docs/en/PHILOSOPHY.en.md) · Vision: [`VISION.md`](VISION.md) · Contributing: [`CONTRIBUTING.md`](CONTRIBUTING.md)

---

## Contents

- [What It Is](#what-it-is)
- [Why Buddhist Frameworks](#why-buddhist-frameworks)
- [The 22 Frameworks](#the-22-frameworks)
- [Stack Diagram](#stack-diagram)
- [Five Principles to Know First](#five-principles-to-know-first)
- [Repository Structure](#repository-structure)
- [Getting Started](#getting-started)
- [Reading Paths](#reading-paths)
- [Security Rules (Sīla 5)](#security-rules-sīla-5)
- [FAQ](#faq)
- [Tech Stack](#tech-stack)
- [Status](#status)
- [Contributing](#contributing)
- [Security](#security)
- [Code of Conduct](#code-of-conduct)
- [License](#license)

---

## What It Is

BWOC provides a **template and doctrine** for creating AI coding agents with a consistent, principled foundation:

- **One repo, one agent** — each agent lives in its own repository cloned from the template
- **Backend-neutral** — runs on Claude, Gemini, Codex, Kimi, or any LLM
- **Persistent memory** — accumulates knowledge across sessions with impermanence-aware pruning
- **Multi-agent safe** — multiple agents co-operate in the same repo without collision

---

## Why Buddhist Frameworks

Buddhist thinking addresses areas where Western engineering frameworks (DDD, Clean Architecture, SOLID) are thin: **state impermanence, failure tracing, lifecycle, inter-agent trust, and threat modeling**.

| Engineering Problem | Buddhist Framework |
|---|---|
| Problem solving | Ariyasacca 4 (Four Noble Truths) |
| Functional requirements | Magga 8 (Noble Eightfold Path) |
| System architecture | Khandha 5 (Five Aggregates) |
| State & impermanence | Tilakkhaṇa (Three Marks) |
| Failure analysis | Paṭiccasamuppāda (Dependent Origination) |
| Audit logging | Kamma 3 (Three Doors of Action) |
| Observability | Satipaṭṭhāna 4 (Four Foundations) |
| Agent lifecycle | Bhāvanā 4 (Four Cultivations) |
| Self-improvement | Paññā 3 (Three Roots of Wisdom) |
| Capability maturity | Ariya-dhana 7 (Seven Noble Treasures) |
| Error UX | Brahmavihāra 4 (Four Divine Abidings) |
| Inter-agent trust | Kalyāṇamitta 7 (Seven Qualities of a Good Friend) |
| Threat modeling | Taṇhā 3 (Three Cravings) |
| Baseline security | Sīla 5 (Five Precepts) |
| Fleet governance | Aparihāniya-dhamma 7 (Seven Non-Decline Principles) |

---

## The 22 Frameworks

Organized into six groups — see [`PHILOSOPHY.en.md`](modules/agent-template/docs/en/PHILOSOPHY.en.md) for full mappings.

### A — Process
Ariyasacca 4 · Magga 8 · Khandha 5

### B — State
Tilakkhaṇa · Paṭiccasamuppāda · Kamma 3

### C — Growth
Iddhipāda 4 · Bhāvanā 4 · Paññā 3 · Ariya-dhana 7

### D — Relational
Sappurisadhamma 7 · Saṅgahavatthu 4 · Sāraṇīyadhamma 6 · Brahmavihāra 4 · Kalyāṇamitta 7

### E — Discipline
Yoniso Manasikāra · Acinteyya 4 · Satipaṭṭhāna 4 · Padhāna 4

### F — Governance
Aparihāniya-dhamma 7 · Taṇhā 3 · Sīla 5

---

## Stack Diagram

```
┌──────────────────────────────────────────────────────┐
│  Aparihāniya-dhamma (Fleet Governance)               │ ← Org level
├──────────────────────────────────────────────────────┤
│  Taṇhā 3 (Threat Model) + Sīla 5 (Baseline)          │ ← Security
├──────────────────────────────────────────────────────┤
│  Bhāvanā 4 (Lifecycle) + Paññā 3 (Improvement)       │ ← Agent growth
├──────────────────────────────────────────────────────┤
│  Sāraṇīyadhamma + Kalyāṇamitta (Inter-agent)         │ ← Interconnect
├──────────────────────────────────────────────────────┤
│  Saṅgahavatthu + Brahmavihāra (UX)                   │ ← User layer
├──────────────────────────────────────────────────────┤
│  Magga 8 (Functional requirements)                   │ ← SRS
├──────────────────────────────────────────────────────┤
│  Khandha 5 (Architecture)                            │ ← Components
├──────────────────────────────────────────────────────┤
│  Satipaṭṭhāna 4 (Observability)                      │ ← Cross-cutting
├──────────────────────────────────────────────────────┤
│  Iddhipāda 4 (Engine of work)                        │ ← Runtime
├──────────────────────────────────────────────────────┤
│  Tilakkhaṇa + Kamma 3 (State & Audit)                │ ← Foundation
├──────────────────────────────────────────────────────┤
│  Paṭiccasamuppāda (Failure analysis)                 │ ← When broken
├──────────────────────────────────────────────────────┤
│  Yoniso manasikāra + Acinteyya (Method)              │ ← Thinking
└──────────────────────────────────────────────────────┘
       Ariyasacca 4 (Problem-solving cycle, end-to-end)
       Sappurisadhamma 7 (Context sensing, end-to-end)
```

---

## Five Principles to Know First

### 1. Yoniso Manasikāra — Verify Before Act
Memory is a past claim. Verify against present state before acting on it.

### 2. Mattaññutā — Right Amount
`MEMORY.md` ≤ 200 lines. Forces selection of what actually matters.

### 3. Anattā — Non-Clinging
Task done → cleanup worktree → delete branch. No attachment to past state.

### 4. Samānattatā — Equal Treatment
All backends receive equal treatment. No vendor favoritism in tooling.

### 5. Sīla-sāmaññatā — Communal Convention
All agents run under the same rules via `conventions.md` and a neutrality check.

---

## Repository Structure

```
bwoc-framwork/
└── modules/
    └── agent-template/          ← Core template (clone to create an agent)
        ├── docs/
        │   ├── en/              ← English documentation
        │   │   ├── PHILOSOPHY.en.md     ← Conceptual core (read first)
        │   │   ├── OVERVIEW.en.md       ← Entry door (5-min orientation)
        │   │   ├── PRD.en.md            ← Product (Ariyasacca 4)
        │   │   ├── SRS.en.md            ← Requirements (Magga 8)
        │   │   ├── SELF-IMPROVEMENT.en.md
        │   │   └── THREAT-MODEL.en.md
        │   └── th/              ← Thai documentation (bilingual pair)
        ├── project-example.md
        ├── reference-example.md
        └── task-log.example.jsonl
```

---

## Getting Started

### Install the CLI (one command)

```bash
./scripts/install.sh
```

Equivalent to `cargo install --path crates/bwoc-cli --locked`. Requires a [Rust toolchain](https://rustup.rs/) on PATH. Installs the `bwoc` binary to `~/.cargo/bin/`.

### As an Agent Author

Either path — both produce the same canonical structure:

```bash
# Today (shell script, manual manifest edit)
cd modules/agent-template && ./scripts/incarnate.sh <agent-name>

# Or (Rust CLI, manifest inputs as flags — Phase 1 v2.0 in progress)
bwoc new <agent-name> --role "..." --primary-model "..." \
  --lint-cmd "..." --format-cmd "..." --test-cmd "..." --build-cmd "..."
```

**Target: from clone to first configured commit in under 30 minutes.**

Full walkthrough — including placeholder resolution, persona definition, multilingual setup, and the verification checklist — is in [`docs/en/INCARNATION.en.md`](docs/en/INCARNATION.en.md) (Thai: [`docs/th/INCARNATION.th.md`](docs/th/INCARNATION.th.md)).

### Reading Paths

**30 min** — `OVERVIEW.en.md` → workflow examples

**2 hours** — `OVERVIEW` → `PHILOSOPHY` (groups A–F) → `PRD` → `SRS`

**Full depth** — read every file in `docs/` in order

---

## Security Rules (Sīla 5)

These are non-negotiable baseline rules derived directly from the Five Precepts:

- No `rm -rf` of repo root
- No committing secrets
- No spoofing agent identity
- No bypassing verification gates
- No undeclared side-effects

---

## FAQ

The three most-asked questions in summary; full FAQ in [`docs/en/FAQ.en.md`](docs/en/FAQ.en.md) (Thai: [`docs/th/FAQ.th.md`](docs/th/FAQ.th.md)).

**Do I need to know Buddhism?**
No. Pali terms are labels; content is purely technical.

**Does this conflict with DDD / Clean Architecture / SOLID?**
No. BWOC extends them into areas they don't cover: state impermanence, failure tracing, inter-agent trust.

**Can I use this without the Buddhist framing?**
Yes — keep the technical skeleton. You lose the unified "why" behind design decisions.

---

## Tech Stack

BWOC is specification-first. The reference implementation is a native, cross-platform Rust toolchain.

| Surface | Stack | Platforms |
|---|---|---|
| Specification | Markdown (two-tier: plain for `AGENTS.md`, Obsidian-flavored elsewhere) | — |
| `bwoc` CLI | Rust, single static binary | **macOS · Linux · Windows** |
| `bwoc-agent` runtime (ships with each incarnated agent) | Rust, single static binary | **macOS · Linux · Windows** |
| CLI i18n (output strings) | Project Fluent (`.ftl` per locale) | **Ships with TH · EN**; pluggable for any future language |
| Backend integration | Subprocess of the LLM's own CLI — Claude Code, Gemini CLI, Codex CLI, Kimi CLI | Whatever the backend supports |
| Distribution | `cargo install bwoc-cli` + GitHub Release binaries (signed) | — |
| License | MIT (see [`LICENSE`](LICENSE)) | — |

The CLI has zero runtime dependencies beyond `libc` / `Win32`. No JVM, no Node, no Docker required to incarnate or run an agent.

---

## Status

**Current phase:** Phase 1 v2.0 — *uppāda foundation*, in progress.

| Area | Status |
|---|---|
| Specification (Philosophy, PRD, SRS, Threat) | Ready |
| Lifecycle, Observability, Failure, Improvement | Ready |
| Coordination, Governance | Ready |
| `bwoc` CLI (Rust, macOS · Linux · Windows) | **Phase 1 v2.0 — in progress** |
| `bwoc-agent` runtime (Rust) | **Phase 1 v2.0 — in progress** |
| Reference agents | Phase 4 |
| Fleet dashboard | Phase 4 |

For the full phase-by-phase plan with completed / in-progress / remaining items, see [`docs/en/ROADMAP.en.md`](docs/en/ROADMAP.en.md) (Thai: [`docs/th/ROADMAP.th.md`](docs/th/ROADMAP.th.md)).

---

## Contributing

We welcome contributions. See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the workflow, commit style, and PR checklist. New to the project? Start with [`VISION.md`](VISION.md), then [`PHILOSOPHY.en.md`](modules/agent-template/docs/en/PHILOSOPHY.en.md).

## Security

Found a vulnerability? **Do not open a public issue.** Email **info@bemind.tech** as described in [`SECURITY.md`](SECURITY.md). The full threat model lives in [`THREAT-MODEL.en.md`](modules/agent-template/docs/en/THREAT-MODEL.en.md).

## Code of Conduct

This project follows a [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) grounded in Sīla 5 (prohibited conduct) and Brahmavihāra 4 (expected disposition). Pali terms are section names; content is technical and non-sectarian.

## License

[MIT](LICENSE) — see the full license text.
