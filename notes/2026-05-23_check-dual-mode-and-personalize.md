---
date: 2026-05-23
session: bwoc check becomes dual-mode + agent-pi/agent-oracle personalized
tags:
  - phase/3
  - type/note
  - module/bwoc-cli
  - module/agents
---

# 2026-05-23 — Dual-Mode `bwoc check` + Pi/Oracle Personalization

A latent contract bug: `bwoc check` was hardcoded for **template mode** and silently passed any incarnated-but-not-personalized agent. ต้นกล้า caught it while reviewing `agents/agent-pi/` and `agents/agent-oracle/` — both still had `{{agentId}}`, `{{primaryCapability}}`, `{{lintCmd}}`, ... in their AGENTS.md, but `bwoc check agents/agent-pi` reported 0 violations. Per AGENTS.md §5.2 the check should fail with "no placeholder is unsubstituted" — the implementation was checking the *opposite* (positive existence). Fix: dual-mode detection from `manifest.name`. Then personalized both agents to prove the new mode works against a real incarnated state.

## What changed

- **`bwoc check` dual-mode** — new `AuditMode::{Template, Incarnation}` enum + `detect_mode(manifest)` helper. The detection key is `manifest.name`: literal `{{name}}` (or missing/empty) → Template; concrete value → Incarnation.
  - **Template mode**: existing behavior. Asserts `REQUIRED_PLACEHOLDERS` are *present* (`{{agentId}}`, `{{memoryPath}}`, `{{taskId}}`, `{{deepMemoryCmd}}`). Runs hardcoded-model / hardcoded-tool / backend-phrasing neutrality checks (those rules guard the *template*, not the per-agent commitment).
  - **Incarnation mode**: asserts NO `{{xxx}}` placeholders survive in AGENTS.md except `{{taskId}}` (whitelisted as runtime per Appendix A of the template AGENTS.md — fills at task-assignment time, not at incarnation). Skips the three neutrality checks because an incarnated agent legitimately commits to a model and a backend voice.
  - New helper `extract_placeholders(content)`: scans for `{{identifier}}` patterns (ASCII alphanumeric + underscore, matches the camelCase manifest key convention), deduplicates, no regex dep.
  - 9 new unit tests: 4 for mode detection (placeholder name / real name / missing manifest / empty name), 4 for placeholder extraction (unique / non-identifier / empty / unclosed), 3 end-to-end audit checks (incarnated with unsubstituted → fail; incarnated clean → pass; template with no placeholders → warning).
- **`agents/agent-pi/` personalized** — `perl -i -pe` substitution across AGENTS.md + persona/README.md for all manifest-derived fields (agentId, agentRole, primaryModel, memoryPath, lint/format/test/build commands, version) and persona answers (`primaryCapability=Rust implementation across bwoc-* crates`, `scopeDescription=bwoc-core, bwoc-cli, bwoc-agent crates; manifest, workspace, daemon, IPC, lifecycle verbs`, `outOfScope=docs/spec doctrine, vault/oracle skills, persona of other agents`, `moduleName=bwoc-core`). Optional defaults: `fallbackModel=claude-sonnet-4-6`, `worktreeBase=/tmp`, `deepMemoryCmd=# (Tier 2 not configured)`, `maxConcurrentTasks=3`. Template-only Appendix A (Placeholder Reference) + Appendix B (Quick-Start Checklist) removed — those documents are pre-incarnation reference.
- **`agents/agent-oracle/` personalized** — same shape with Oracle's persona: `primaryCapability=Fleet coordination via inbox/messaging across agents`, `scopeDescription=inter-agent messaging, workspace state observation, multi-agent flow orchestration`, `outOfScope=writing crate code, fixing framework bugs, modifying other agents personas`, `moduleName=interconnect`.
- **`bwoc-agent::trust` test mutex** — Rust 2024 marks `std::env::set_var/remove_var` as `unsafe` because env is process-wide. Two new tests in `trust::tests` both mutate `BWOC_TRUST_GATING` and raced (1/15 flake on first parallel run). Added a module-private `static ENV_LOCK: Mutex<()>` and `let _guard = ENV_LOCK.lock().unwrap();` in both env-touching tests. 3 consecutive workspace test runs after the fix: 110 tests, 0 failures each.

## Decisions

- **Mode detection key is `manifest.name`, not the directory path.** Considered detecting by checking if the target's canonical path is `modules/agent-template/`, but that's brittle (anyone vendoring BWOC into another repo would re-path it). `manifest.name` is structural: a template's name is `{{name}}` by definition; an incarnation has a real name. The check follows from the manifest itself, not from where the agent lives on disk.
- **`{{taskId}}` is the only whitelisted runtime placeholder.** Per Appendix A of the template AGENTS.md, `taskId` is "runtime — task assignment." Every other placeholder is "yes / no — user edit" or "default" — meaning incarnation-time substitution. Hardcoding the whitelist (rather than inferring from Appendix A at runtime) is acceptable because the runtime placeholder set changes only when the spec changes.
- **Skip neutrality checks in incarnation mode.** A real tension surfaced when substituting `{{primaryModel}}` → `claude-opus-4-7`: the HARDCODED_MODELS check fires (substring match on `claude-opus`). But the neutrality requirement (no hardcoded backends) is a property of the *template*, not the per-agent instance. Each agent is allowed to commit to a model. Resolution: gate HARDCODED_MODELS + HARDCODED_TOOLS + BACKEND_PHRASES checks by mode. Template gets them; incarnation doesn't. Operators who want strict neutrality run `bwoc check modules/agent-template/`.
- **Missing manifest → Template mode (not Incarnation).** Considered the opposite default ("if you have no manifest, you're probably broken"), but a half-built agent is most often *being* incarnated — landing it in Template mode means the check warns about missing recommended placeholders, which is the friendlier diagnostic. Reading a half-built dir as a "broken incarnation" with a wall of FAILs would obscure the real state ("you haven't finished `bwoc new` yet").
- **Removed Appendix A + B from incarnated agents instead of preserving them.** The post-substitution Appendix A table had values in the "Placeholder" column where placeholder names used to be — nonsensical and misleading. Considered re-escaping the placeholders (e.g., HTML entities, code-fence trickery) but that adds churn for documentation that doesn't apply to incarnated agents anyway. Cleanest: delete those two appendices from incarnations; they live canonically in the template.
- **`perl -i -pe` over the Edit tool for substitution.** ~19 unique placeholders across 2 files per agent = ~38 mechanical replacements per agent. Edit-by-Edit would be slow and noisy. Perl's `\Q...\E` quote-meta makes the placeholder syntax safe without escaping each `{`. Used for the one-off personalization only; future `bwoc new` substitution should bake this into the Rust path (queued, not done here).
- **Don't add CHANGELOG/ROADMAP automated cross-references.** The CHANGELOG entries are written manually; ROADMAP tracking happens through the spec's own implementation-order section. Adding tooling to sync these would be cute but premature (Mattaññutā — three similar lines beats a generator).

## Alternatives considered

- **Strict by default.** Make the check fail on `{{taskId}}` too unless explicitly told it's runtime. Rejected — Appendix A is authoritative; the whitelist matches it.
- **Per-agent override file.** Let an agent declare `.bwoc/check-allowlist.txt` listing placeholders it intentionally keeps literal. Rejected — invites scope creep. The runtime whitelist (1 entry) is small enough to be a constant.
- **Substitute optional `{{deepMemoryCmd}}` with `null`.** Would break the `bash` code block where the placeholder appears as a command. Used `# (Tier 2 not configured)` instead — a shell comment that doesn't execute but reads honestly.
- **Substitute via a Rust helper.** Considered porting the substitution into `bwoc-cli::personalize` and running it on the two existing agents. Rejected for this iter — one-off personalization, and `bwoc new` is the proper place for that logic going forward (separate task). `perl -i -pe` got us through today's gap.

## Bugs surfaced and fixed

- **The original bug.** `bwoc check agents/agent-pi` reported 0 violations on an agent with 19 unsubstituted placeholders. The §5.2 spec says "no placeholder is unsubstituted" — implementation checked positive presence instead. Now fixed by `AuditMode` detection.
- **clippy `if_same_then_else`.** Initial `detect_mode` had `if placeholder_form { Template } else if name.is_empty() { Template } else { Incarnation }` — same branch twice. Refactored to `if placeholder_form || name.is_empty() { Template } else { Incarnation }`. Clippy clean across the workspace after.
- **Trust-test env race.** Detailed above. Mutex closes it. 3× full-workspace `cargo test` runs all green after the fix.
- **CWD drift after `cd` in a bash one-liner.** A `cd agents/agent-pi && perl -i -pe ...` left the next `cargo run` from the wrong directory; `AGENTS.md not found` until I returned to root. Reminder: prefer absolute paths in multi-step bash blocks; `cd` only when explicitly intentional.

## Status / deferred

- **`bwoc new` should substitute placeholders automatically going forward.** Today it copies the template + writes `config.manifest.json`, but doesn't touch AGENTS.md / persona/README.md. The mechanical substitution path (manifest fields + interactive prompts for persona fields) belongs in Rust, not in `perl -i -pe`. Queued — this iter only fixed the two existing un-personalized agents.
- **Persona/README.md still uses the template's Markdown structure.** Pi/Oracle personas are technically correct after substitution but not specifically tuned to each agent's voice. ต้นกล้า can refine over time; the structural pass is enough for the check to pass.
- **`taskId` not the only future runtime placeholder.** When task records or worktree paths start interpolating session IDs, ULIDs, etc., the runtime whitelist may grow. Today's whitelist of `["{{taskId}}"]` matches Appendix A and the spec exactly.

## Test summary

- bwoc-core: 18 tests
- bwoc-agent: 15 tests (env-mutex fix gives deterministic green)
- bwoc-cli: 76 tests (+9 new check-mode tests beyond step 4's +4 refusal-merge tests)
- end-to-end: 1 test
- **Total: 110 passing, 0 failures, clippy clean.** Verified across 3 consecutive runs.

Live verification:
- `bwoc check agents/agent-pi` → 9 PASS, 0 violations.
- `bwoc check agents/agent-oracle` → 9 PASS, 0 violations.
- `bwoc check modules/agent-template` → 15 PASS, 0 violations (template mode still enforces placeholder presence + neutrality).
- `bwoc check --all` → Fleet summary `2 agent(s): 18 pass, 0 warn, 0 violation(s)`.

## Related

- Trust step 4 note: [`2026-05-23_trust-step-4.md`](./2026-05-23_trust-step-4.md)
- Previous trust note: [`2026-05-23_first-release-and-trust-spec.md`](./2026-05-23_first-release-and-trust-spec.md)
- AGENTS.md §5.2 (the rule that should have been enforced): [`modules/agent-template/AGENTS.md`](../modules/agent-template/AGENTS.md)
- AGENTS.md Appendix A (placeholder reference + runtime designation): same file, end of doc.
- Commit: pending (this note ships with it)
