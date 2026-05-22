---
name: check-bilingual
description: Verify EN/TH parity for BWOC docs. Reports any docs/en/*.en.md without a matching docs/th/*.th.md, and (when running inside a git repo) any EN file changed in the current diff whose TH counterpart was not also changed. Use when the user says "check bilingual", "parity check", "EN TH", or before a PR touching docs.
---

# /check-bilingual — EN/TH parity audit

Read-only. Covers both the framework root `docs/` and `modules/agent-template/docs/`.

## Steps

1. **Existence pass.** For each `docs/en/*.en.md` under the framework root and the template, check that the corresponding `docs/th/*.th.md` exists. Report missing counterparts as FAIL.

   ```bash
   for en in docs/en/*.en.md modules/agent-template/docs/en/*.en.md; do
     [ -f "$en" ] || continue
     th="${en/\/en\///th/}"; th="${th/.en.md/.th.md}"
     [ -f "$th" ] || echo "MISSING TH: $th (counterpart of $en)"
   done
   ```

2. **Diff pass — only if the repo is a git repo** (`git rev-parse --is-inside-work-tree` exits 0). For each `*.en.md` in the diff against `HEAD` or the merge base, verify the matching `*.th.md` is also in the diff. Report mismatches as FAIL.

   If the repo isn't initialized (the framework root is not yet `git init`'d at the time of writing), skip this pass and say so.

3. **Stale pass.** For each EN/TH pair, compare modification times (`stat -f %m` on macOS). If EN is newer than TH by more than the file mtime resolution, flag as WARN — TH may have drifted.

4. **Do not edit the TH files.** This skill only reports. Offer to draft the TH update only after the user confirms.

## Reporting

- Summary line: `EN/TH parity: <N> FAIL, <N> WARN, <N> OK`
- One line per finding with the file path.
- Exit early with `OK` if no findings — do not pad the output.

## Apply the principle

Name **Sīlasāmaññatā** — communal conventions. The bilingual rule is a community contract; one-sided edits silently break it.
