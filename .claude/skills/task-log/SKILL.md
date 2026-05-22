---
name: task-log
description: Append a Four-Noble-Truths-shaped task entry to task-log.jsonl (Dukkha → Samudaya → Nirodha → Magga). Use when the user says "log task", "task-log", "record this task", "append to task log", or at the start of any non-trivial task per the AGENTS.md protocol.
disable-model-invocation: true
---

# /task-log — append a task entry to task-log.jsonl

User-triggered. Side effect: appends one line to `task-log.jsonl` (append-only — never rewrite the file).

## Arguments

`$ARGUMENTS` — free text describing the task. The skill extracts or asks for the four fields below.

## Steps

1. **Locate `task-log.jsonl`.** Default to the current working directory's root (a real incarnated agent). If absent and we're inside the framework root, ask the user whether to create one here (the framework itself does not normally log tasks) or to point at a specific agent path.
2. **Gather the four truths.** Ask the user inline for any missing field. Keep questions to one line each.
   - **Dukkha** — what is the concrete problem? what breaks or is missing?
   - **Samudaya** — root cause (trace backward).
   - **Nirodha** — measurable success state.
   - **Magga** — minimal path (steps, gates, cleanup).
3. **Build the JSON entry** matching `modules/agent-template/AGENTS.md` §2.2:
   ```json
   {
     "taskId": "TASK-NNN",
     "moduleName": "...",
     "branchName": "feature/TASK-NNN",
     "worktreePath": "/tmp/<agentId>/TASK-NNN",
     "status": "in_progress",
     "startedAt": "<ISO-8601 UTC>",
     "lastAction": "logged via /task-log",
     "completedAt": null,
     "blockedReason": null,
     "dukkha": "...",
     "samudaya": "...",
     "nirodha": "...",
     "magga": "..."
   }
   ```
   - `taskId`: increment from the last line of the file (`tail -n1 task-log.jsonl | jq -r .taskId`). If the file is empty, start at `TASK-001`.
   - `startedAt`: `date -u +%Y-%m-%dT%H:%M:%SZ`.
   - `moduleName`, `worktreePath`, `agentId`: read from `config.manifest.json` if present; otherwise ask.
4. **Append** as a single line. Validate JSON via `jq -c .` before writing to ensure no malformed entry corrupts the file.
   ```bash
   echo "$ENTRY" | jq -c . >> task-log.jsonl
   ```
5. **Confirm** by printing the new line and its `taskId`. Do not echo the whole file.

## Hard rules

- Append-only. Never `sed -i`, `>`, or otherwise rewrite existing lines.
- One JSON object per line — JSONL, not a JSON array.
- The framework root has no `task-log.jsonl` by design; this skill is primarily for incarnated agents.

## Apply the principle

Name **Sammā-sankappa** (Right Intention) — the four-truth structure forces intent to be made explicit before action.
