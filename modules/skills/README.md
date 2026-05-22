# `modules/skills/` — Framework Skills

**Status:** planned. No framework skills shipped yet.

## What framework skills are

A **framework skill** is a capability the framework recommends as a baseline for any agent — versus an **agent skill** (`modules/agent-template/skills/`), which an individual agent declares for itself.

Framework skills are the "standard library" of agent capabilities: well-defined behaviors any agent can opt into, with a consistent interface and verification gates.

## Examples that might fit

- **Worktree discipline** — create, isolate, cleanup per the Anattā rule.
- **Bilingual parity check** — verify EN/TH (and future languages) stay in sync inside an agent's `docs/<lang>/`.
- **Task-log audit** — verify `task-log.jsonl` is append-only and well-formed.
- **Capability declaration validation** — verify `interconnect/capabilities.md` matches what the agent actually does.

## Distinction from `.claude/skills/`

`.claude/skills/<name>/SKILL.md` are **Claude Code project skills** — slash commands available in a Claude Code session for this specific repo (e.g., `/incarnate`, `/check-neutrality`, `/check-bilingual`, `/task-log`, `/check-naming`).

Framework skills are **agent-runtime concepts** — capabilities an incarnated agent invokes during its own operation, independent of which AI tool is driving it.

## Spec status

The framework skill manifest format, invocation contract, and recommended baseline set are not yet specified. The first framework skill lands together with its spec.

## See Also

- [`modules/README.md`](../README.md)
- [`modules/agent-template/skills/SPEC.md`](../agent-template/skills/SPEC.md) — distinct: per-agent skill slot.
- [`.claude/skills/`](../../.claude/skills/) — distinct: Claude Code project skills.
