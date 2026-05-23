---
title: 2026-05-23 — Incarnate agent-codx (Codex backend, gpt-5.4)
tags:
  - type/note
  - group/agents
  - backend/codex
---

# 2026-05-23 — Incarnate agent-codx

Spun up a 4th agent in this workspace: `agent-codx` on the Codex backend with model `gpt-5.4`, role `software engineer`, persona ผู้หญิง น่ารัก พูดเพราะ รอบครอบ. Ran a short end-to-end smoke test through `codex exec`.

## What changed

- `agents/agent-codx/` — new incarnation via `bwoc new codx --backend codex …`
- `.bwoc/agents.toml` — appended registry entry (verified via `bwoc list` + `bwoc doctor`)
- Verification gates set to the same cargo commands as `agent-pi` since the role is software engineer

## Decisions

- **Mirrored `agent-anti`'s feminine-polite persona phrasing** for scope/out-of-scope, adapted to "เขียนโค้ด" instead of "ตอบคำถาม" since role differs (software engineer vs. analysis).
- **Did not hand-substitute the 45 remaining `{{placeholders}}` in `AGENTS.md`.** Same state as `agent-anti` right after `bwoc new` — the manifest is the source of truth; placeholder substitution into `AGENTS.md` itself is a separate manual step that only `agent-pi` has done. Honoring Mattaññutā — don't expand work beyond what was asked.

## E2E test

```bash
cd agents/agent-codx
codex exec --model "gpt-5.4" "สวัสดีค่ะ ช่วยแนะนำตัวเองสั้นๆ และ 1+1=?"
```

Codex CLI v0.133.0 accepted the model string and returned a short Thai reply ending in "ค่ะ" — feminine register consistent with persona. Confirms the path: workspace registry → agent dir → backend CLI exec.

## Status / deferred

- `bwoc check agent-codx` reports 16 placeholder violations (same as `agent-anti`); not a blocker for runtime, only for "personalized" status. Defer until/unless a user-visible feature demands it.
- No git commit made; the framework's `.gitignore` excludes `agents/` from this workspace, so the new agent lives only on local disk.

## Related

- [[agents/agent-anti/AGENTS|agent-anti]] — sibling incarnation, same persona phrasing
- [[modules/agent-template/scripts/incarnate.sh|incarnate.sh]]
