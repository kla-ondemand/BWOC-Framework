---
name: check-naming
description: Audit every *.md file in the repo against the unified naming standard in docs/en/NAMING.en.md. Reports deviations by category (root-level OSS standard, spec docs, notes, etc.). Use when the user says "check naming", "audit md naming", "find naming violations", or before a release. Read-only.
---

# /check-naming — Markdown naming audit

Read-only audit. Runs the grep checks documented in [`docs/en/NAMING.en.md` §Audit](../../../docs/en/NAMING.en.md#audit), categorizes findings, and proposes fixes without applying them.

## Arguments

`$ARGUMENTS` — optional path scope. Defaults to the framework root. Useful for auditing a single incarnated agent: `/check-naming /path/to/agent-foo`.

## Steps

1. **Resolve the scope.** Default = current working directory (assumed to be a framework root or workspace). Verify it contains either `docs/` or `modules/agent-template/` so the audit is meaningful.

2. **Run the three audit greps from NAMING.en.md §Audit verbatim** (the same checks `.github/workflows/docs.yml` runs in CI). Report each as PASS / N FINDINGS with the offending paths:

   ```bash
   # A) Root-level — allow UPPERCASE.md, UPPERCASE.<lang>.md, CLAUDE.local.md
   find . -maxdepth 1 -name '*.md' \
     | grep -vE '^\./(README|LICENSE|CHANGELOG|CONTRIBUTING|CODE_OF_CONDUCT|SECURITY|VISION|VERSION|CLAUDE|AGENTS)(\.local|\.[a-z]{2,3})?\.md$'

   # B) docs/<lang>/ — UPPERCASE.<lang>.md (mindepth 2 skips slot-level examples)
   find docs modules/agent-template/docs -mindepth 2 -type f -name '*.md' 2>/dev/null \
     | grep -vE '/[A-Z]+(-[A-Z]+)*\.(en|th|[a-z]{2,3})\.md$' \
     | grep -v '/README'

   # C) Notes — YYYY-MM-DD_<title>.md
   find . -path '*/notes/*.md' 2>/dev/null \
     | grep -vE '/[0-9]{4}-[0-9]{2}-[0-9]{2}_[a-z0-9-]+\.md$'
   ```

3. **Categorize findings** by which NAMING category was violated:
   - Root-level metadata expects `UPPERCASE.md` or `UPPERCASE.<lang>.md`.
   - `docs/<lang>/` files expect `UPPERCASE.<lang>.md`.
   - Notes expect `YYYY-MM-DD_<title>.md`.
   - Sub-slot READMEs (`memories/README.md`, `persona/README.md`, etc.) are exempt — they're Obsidian spec files, not OSS landings.
   - Crate READMEs (`crates/<x>/README.md`) are exempt — Rust convention.

4. **Propose fixes**, do not apply them:
   - For each finding, name the expected pattern and the suggested rename.
   - Flag if a finding might be intentional (e.g., `*.bad.md` for anti-pattern examples) before suggesting a rename.

5. **Summary line**: `naming audit: <PASS|N findings> across <X> files scanned`.

## Hard rules

- **Read-only.** Never `mv`, `git mv`, or modify files.
- Always quote `find` paths to handle spaces (though they shouldn't exist per NAMING).
- Skip `node_modules/`, `target/`, `.git/`, `.claude/cache/`, `.claude/.local/`.
- The Obsidian spec files in `modules/agent-template/<slot>/README.md` are exempt — surface them in the report as "exempt by category 5 in NAMING.en.md", not as violations.

## Apply the principle

Name **Sīlasāmaññatā** (communal convention) in the summary — the naming standard exists because shared convention beats individual preference. The audit reports deviations as opportunities to align with the community.
