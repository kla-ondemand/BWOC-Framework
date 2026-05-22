# How-To: Diagnose and Fix Common Issues

## Goal

Use `bwoc doctor` to find what's wrong with your environment or workspace, and let `--auto` fix the safe issues.

## Prerequisites

- `bwoc` CLI installed
- Optionally: an existing workspace to scan

## The command

```bash
bwoc doctor                # diagnose only
bwoc doctor --auto         # diagnose + fix safe issues in place
bwoc doctor --workspace P  # explicit workspace path
```

Exit codes:

- `0` — nothing failed (or everything that failed got fixed)
- `2` — at least one FAIL remains after the (optional) auto-fix pass

## What it checks

| # | Check | Auto-fixable? |
|---|---|---|
| 1 | `~/.bwoc/` directory exists | ✓ creates the dir + empty `config.toml` |
| 2 | At least one backend CLI on PATH (`claude` / `gemini` / `codex` / `kimi`) | ✗ informational only — you install them |
| 3 | `.bwoc/workspace.toml` parses cleanly | ✗ needs manual edit — won't silently rewrite |
| 4 | `.bwoc/agents.toml` parses cleanly | ✗ same |
| 5 | Scaffold dirs present (`agents_dir`, `projects/`, `notes/`) | ✓ `mkdir -p` |
| 6 | Each registered agent has its 4 backend symlinks (`CLAUDE.md` / `GEMINI.md` / `CODEX.md` / `KIMI.md` → `AGENTS.md`) | ✓ recreates the missing symlinks |

The policy: auto-fix only when there's **one obvious correct answer**. Anything that needs judgment (malformed config, missing `AGENTS.md`, missing backend CLI) is reported, never silently rewritten.

## Common scenarios

### "I deleted a scaffold dir by accident"

```bash
$ rm -rf projects/
$ bwoc doctor
…
  FAIL   scaffold dirs — missing: projects (rerun with --auto to create)
$ bwoc doctor --auto
…
  FIXED  scaffold dirs — created: projects
```

### "An agent's backend symlink is broken"

```bash
$ rm agents/my-agent/KIMI.md
$ bwoc doctor
…
  FAIL   agent: agent-my-agent — missing backend symlinks: KIMI.md (rerun with --auto to recreate)
$ bwoc doctor --auto
…
  FIXED  agent: agent-my-agent — recreated symlinks: KIMI.md
```

### "I'm not in a workspace"

```bash
$ cd /tmp && bwoc doctor
…
  WARN   workspace — not inside a BWOC workspace — workspace-level checks skipped. Pass a path or run `bwoc init` first.
```

The env-level checks (`~/.bwoc/`, backends on PATH) still run; workspace-level checks are skipped with a single WARN.

### "I uninstalled a backend CLI"

```bash
$ bwoc doctor
…
  WARN   backends on PATH — no backend CLI on PATH (claude/gemini/codex/kimi). `bwoc spawn` will fail.
```

WARN, not FAIL — you may have other workflows that don't need `bwoc spawn`. Install at least one backend if you want to run agents.

## What doctor will NOT do

- Edit `workspace.toml` or `agents.toml` if they're malformed (you'd lose information). Doctor reports the parse error; you fix the file.
- Recreate a missing `AGENTS.md` in an agent dir (we don't know what content belonged there). Doctor reports it; you decide whether to `bwoc retire` and re-incarnate.
- Install missing backend CLIs. Doctor only checks PATH.
- Touch anything outside `.bwoc/`, `agents_dir/`, and the standard scaffold dirs.

## What's next

- [`workspace-layout.md`](workspace-layout.md) — what each dir is for
- `crates/bwoc-cli/src/doctor.rs` — the check definitions (source of truth)
- [`docs/en/WORKSPACE.en.md` §Validation Rules](../../docs/en/WORKSPACE.en.md) — the formal spec doctor implements
