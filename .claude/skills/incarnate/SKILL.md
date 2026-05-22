---
name: incarnate
description: Scaffold a new BWOC agent by running the template's incarnate.sh, then guide the user through filling in the {{camelCase}} placeholders in AGENTS.md and config.manifest.json. Use when the user says "incarnate", "new agent", "clone the template", or names a new agent to create.
disable-model-invocation: true
---

# /incarnate — clone the BWOC agent template into a new agent

This skill wraps `modules/agent-template/scripts/incarnate.sh`. It is user-triggered (has side effects: copies files, runs `git init`, creates a commit).

## Arguments

`$ARGUMENTS` — the agent name (required), optionally followed by a target path.

## Steps

1. **Validate the name.** Lowercase, hyphen-separated, no spaces. Confirm with the user if it's the first time seeing this name.
2. **Run the script** from the template directory:
   ```bash
   cd modules/agent-template && ./scripts/incarnate.sh <name> [target-path]
   ```
   Default target is `../agent-<name>/` relative to the template.
3. **Report what the script created**: target path, symlinks, git commit hash. Quote the script's "Next steps" block verbatim — do not paraphrase.
4. **Offer to do step 2** of the script's next-steps (filling `AGENTS.md` Section 1 placeholders). If accepted, read the manifest schema in `modules/agent-template/config.manifest.json` and `modules/agent-template/conventions.md`, then propose the placeholder values before editing.

## What this skill does NOT do

- Does not push to any remote.
- Does not delete or overwrite an existing target directory — the script exits 1 if the path exists; respect that.
- Does not modify the template itself.

## Apply the principle

Name **Yoniso manasikāra** in the report: verify the resulting agent directory by listing its contents and confirming `./scripts/check-agent-neutrality.sh` exits 0.
