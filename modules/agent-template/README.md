# Agent Base Profile — Template

The canonical template for creating BWOC-compliant AI coding agents.

[![Template](https://img.shields.io/badge/role-agent%20template-blue.svg)](../../README.md)
[![Backend-neutral](https://img.shields.io/badge/backends-Claude%20%7C%20Gemini%20%7C%20Codex%20%7C%20Kimi-purple.svg)](#backend-neutrality-sammā-ājīva)
[![Format](https://img.shields.io/badge/format-two--tier%20Markdown-lightgrey.svg)](../../CLAUDE.md)
[![Docs](https://img.shields.io/badge/docs-EN%20%7C%20TH-blue.svg)](docs/)

One clone per agent. Backend-neutral by design: Claude, Gemini, Codex, and Kimi all read the same `AGENTS.md` via symlinks.

> Framework root: [`../../README.md`](../../README.md) · Philosophy: [`docs/en/PHILOSOPHY.en.md`](docs/en/PHILOSOPHY.en.md) · The Arc: [`PHILOSOPHY.en.md §0.1`](docs/en/PHILOSOPHY.en.md#01-the-arc--uppāda--ṭhiti--vaya)

---

## Contents

- [What This Template Provides](#what-this-template-provides)
- [Backend Neutrality (Sammā-ājīva)](#backend-neutrality-sammā-ājīva)
- [Incarnating a New Agent](#incarnating-a-new-agent)
- [File Structure After Incarnation](#file-structure-after-incarnation)
- [Key Rules (Non-Negotiable)](#key-rules-non-negotiable)
- [Documentation Paths](#documentation-paths)
- [Status](#status)

---

## What This Template Provides

| Concern | File | Framework |
|---|---|---|
| Agent instructions (all backends) | `AGENTS.md` | Magga 8 |
| Product requirements | `docs/en/PRD.en.md` | Ariyasacca 4 |
| Software requirements | `docs/en/SRS.en.md` | Magga 8 |
| Philosophy reference | `docs/en/PHILOSOPHY.en.md` | 22 frameworks |
| Threat model | `docs/en/THREAT-MODEL.en.md` | Tanha 3 + Sila 5 |
| Self-improvement loop | `docs/en/SELF-IMPROVEMENT.en.md` | Panna 3 |
| Task log format | `docs/task-log.example.jsonl` | Kamma 3 |
| Project memory example | `docs/project-example.md` | Samma-sati |
| Reference memory example | `docs/reference-example.md` | Samma-sati |

---

## Backend Neutrality (Sammā-ājīva)

`AGENTS.md` is the single source of truth. Backend entry files are symlinks — no separate content per backend.

```
CLAUDE.md  ──┐
GEMINI.md  ──┤──→  AGENTS.md
CODEX.md   ──┤
KIMI.md    ──┘
```

To add a new backend:
```bash
ln -s AGENTS.md <BACKEND>.md
```

No other change required. Verify with:
```bash
./scripts/check-agent-neutrality.sh
```

---

## Incarnating a New Agent

```bash
./scripts/incarnate.sh <agent-name>
cd ../agent-<agent-name>
```

Then fill `config.manifest.json`, edit Section 1 of `AGENTS.md`, define the persona, and run `./scripts/check-agent-neutrality.sh`. Target: first commit within 30 minutes.

Full step-by-step (placeholders, persona, verification checklist, multilingual setup): [`docs/en/INCARNATION.en.md`](../../docs/en/INCARNATION.en.md) · [`docs/th/INCARNATION.th.md`](../../docs/th/INCARNATION.th.md).

---

## File Structure After Incarnation

```
agent-<name>/
├── AGENTS.md                  ← single source of truth (all backends)
├── CLAUDE.md → AGENTS.md      ← symlink
├── GEMINI.md → AGENTS.md      ← symlink
├── CODEX.md  → AGENTS.md      ← symlink
├── KIMI.md   → AGENTS.md      ← symlink
├── config.manifest.json       ← placeholders + runtime config
├── task-log.jsonl             ← append-only audit trail
├── memories/
│   └── MEMORY.md              ← index (≤ 200 lines)
├── interconnect/
│   ├── capabilities.md        ← machine-readable skill declarations
│   └── coordination.md        ← inter-agent protocol
└── docs/
    ├── en/                    ← English documentation
    └── th/                    ← Thai documentation (bilingual pair)
```

---

## Key Rules (Non-Negotiable)

**Worktree isolation** — every task in its own worktree, never in the main directory.

**No git stash** — use worktrees instead.

**Verification gates** — lint, format, test, regression, build must all pass before declaring done.

**Memory cap** — `MEMORY.md` ≤ 200 lines. Forces quality over accumulation.

**Verify before act** — treat all memory as a past claim; verify against current code before acting.

**Cleanup** — after merge, remove worktree and delete branch. No clinging.

---

## Documentation Paths

**30 min** — `docs/en/OVERVIEW.en.md`

**2 hours** — OVERVIEW → `docs/en/PHILOSOPHY.en.md` (groups A–F) → `docs/en/PRD.en.md` → `docs/en/SRS.en.md`

**Full depth** — every file in `docs/en/` in document-map order

---

## Status

| Area | Status |
|---|---|
| AGENTS.md (multi-backend) | Ready |
| PRD, SRS, Philosophy | Ready |
| Threat model, Self-improvement | Ready |
| Task log, Memory examples | Ready |
| Scripts (incarnate, check-neutrality) | Phase 1 |
| Interconnect protocol | Phase 3 |
| Fleet governance | Phase 4 |
