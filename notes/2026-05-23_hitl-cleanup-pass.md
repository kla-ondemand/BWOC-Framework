---
date: 2026-05-23
session: HITL cleanup pass — 4 small fixes from the /investigate audit
tags:
  - phase/3
  - type/note
  - module/bwoc-cli
  - module/agent-template
---

# 2026-05-23 — HITL Cleanup Pass (Actions 2/3/5/7)

After the `/investigate BWOC human in loop workflow` report surfaced 7 recommended actions, this iter ships the 4 small isolated ones via 4 parallel sub-agents. Actions 1 (plan-approval gate) and 6 (`bwoc send --redacted`) are feature-sized and deferred; action 4 (last_action scrollback) is polish.

## What changed

- **Action 2 — Stop-hook failure surfacing** (`modules/agent-template/.claude/hooks/inbox-auto-reply.sh`). The hook's `subprocess.run(...)` invocation now captures stdout/stderr (previously discarded), and on non-zero exit appends a one-line diagnostic to `<self>/.bwoc/agent.log` with the format `<utc-ts> inbox-auto-reply: send to <sender> failed (exit N): <stderr-truncated-to-500>`. Happy path stays silent. Log-write failures are swallowed (`OSError` → pass) so the hook never blocks the agent's Stop event.
- **Action 3 — Refusal badge in dashboard** (`crates/bwoc-cli/src/livecheck.rs` + `dashboard.rs`). New `pub fn refusal_summary(root, agent) -> (usize, Option<(String, String)>)` in `livecheck` reads `<agent>/.bwoc/inbox.refusals.jsonl`, returns count + the most-recent `(reason, envelopeFrom)` by ISO-8601 ts ordering. Dashboard detail pane (below the inbox count row) renders `Refused: N refusal(s)` in yellow when N > 0, with a sub-row `last refused: <reason> from <from>`. Row omitted entirely when N == 0 (Mattaññutā — zero count adds no signal).
- **Action 5 — `bwoc status --banner`** (`crates/bwoc-cli/src/status.rs` + `main.rs` + `locales/{en,th}/cli.ftl`). New `--banner` flag on `bwoc status <agent>` replays the daemon's startup "I am alive" multi-line block from the agent's manifest — no daemon required. Mutex with `--all`. Honors `--lang`. `--banner --json` emits `{"banner": "..."}`. Path taken: copy the formatting logic into `bwoc-cli` with its own 6 FTL keys (`status-banner-alive` / `-role` / `-model` / `-fallback` / `-memory` / `-version`) rather than promote to `bwoc-core` — the i18n-bundle threading both crates do would have made the extraction cost-heavy. 3 new tests (`banner_string_en_contains_required_fields` / `_th_alive_line` / `_omits_fallback_when_none`).
- **Action 7 — `start` / `stop` non-TTY consistency** (`crates/bwoc-cli/src/start.rs` + `stop.rs`). Single-agent paths previously failed silently when run non-interactively without `--yes`. Now both add a `NotATerminal` error variant + match arm and abort with exit 2 + actionable message: `bwoc {start,stop}: not a TTY and --yes not given. Pass --yes to confirm or run from an interactive shell.` Matches `retire`'s shape, closes the inconsistency the audit flagged. Mass-action paths (`--all`) unchanged — they already enforced this.

## Decisions

- **Batch all 4 in one commit.** They're a coordinated review-pass cleanup; splitting into 4 commits would inflate the log without aiding bisect. The commit message lists each so blame-walks land cleanly.
- **Defer Actions 1 + 6.** Plan-approval (#1) is a feature with envelope-kind extension + new CLI subcommand + bilingual spec — belongs in the Saṅgha v1 spec discussion the user is already exploring. Tombstone-redact (#6) needs schema + reader semantics decisions that haven't been made. Note them in the release plan, don't ship blind.
- **Skip Action 4** (`last_action` scrollback). Low priority polish; not worth the bytes in a 2.0 release.
- **Spawn sub-agents instead of doing sequentially.** Independent files, no cross-cutting logic — parallel is 4× faster. Each sub-agent reported back with verification result + LOC + edge-case notes; consolidated their reports here.

## Alternatives considered

- **Promote `liveness_banner` to `bwoc-core`** (Action 5) — would have shared one code path between daemon and `bwoc status --banner`. Rejected because the Fluent bundle is per-crate (separate locales/ dirs) and threading would require restructuring the i18n module. Two formatters with parallel FTL keys is fine until either side actually drifts.
- **`Refused: 0` shown in dim** (Action 3) — would mirror the inbox row's behavior (`0 message(s)` in DarkGray). Rejected because the inbox row is always relevant (inbox is the primary channel); the refusal sidecar is a secondary surface that only matters when populated. Doubling the dim-zero noise costs scan-time.
- **Make Action 7 a single shared helper** in `bwoc-cli/src/util.rs` — would deduplicate the ~10 lines per file. Rejected for now: three callers (`start`, `stop`, future) is below the "rule of three" threshold for extraction, and the message text is slightly different between commands.

## Bugs surfaced and fixed

- **Pre-existing `main.rs` compile error during Action 3's verification window.** Sub-agent 3 noted that `cargo build` failed mid-iter due to `StatusArgs` field mismatch from a concurrent session's edit; Action 5's sub-agent fixed it as part of adding the `--banner` flag (the StatusArgs struct gained the new field, which silenced the error). Not really a bug we introduced — parallel-session interference. Final workspace build is clean: 121 tests pass, clippy clean.
- **`utcnow()` deprecation in Action 2's bash/python heredoc.** Python 3.12+ deprecates `datetime.datetime.utcnow()`. Sub-agent kept the deprecated form for now since the script's minimum runtime is whatever ships with macOS. Promote to `datetime.now(datetime.UTC)` next time the Python minimum is bumped. Noted, not fixed.

## Status / deferred

- **Action 1 — plan-approval gate** (Pavāraṇā). Bundle into the Saṅgha v1 spec session. Will need a new envelope kind (`plan-request`/`plan-approve`/`plan-reject`), CLI subcommands (`bwoc plan submit/approve/reject`), and bilingual spec doc.
- **Action 6 — `bwoc send --redacted <msg-id>`**. Append-only tombstone for the audit-trail / typo-recovery case. Deferred until Saṅgha v1 needs it (task-claim retraction is a similar shape).
- **Action 4 — dashboard `last_action` scrollback / Ctrl-L history dump**. Low priority polish. Skip until an operator actually asks.
- **`bwoc-agent::i18n::t` dead-code allow** — Sub-agent 5 noted that `bwoc-agent/src/i18n.rs` has `#[allow(dead_code)]` on `pub fn t()` while `bwoc-cli/src/i18n.rs` doesn't. Neither is broken; minor inconsistency. Cleanup sweep when i18n coverage expands.

## Test summary

- **Workspace**: 121 tests passing (15 + 87 + 1 + 18 + 0). Up from 118 — 3 new `banner_string` tests in `status::tests`.
- **Clippy**: clean (`cargo clippy --workspace --all-targets -- -D warnings`).
- **Live verification per sub-agent**:
  - Action 2: fail path → diagnostic line in `agent.log`; happy path → no log addition.
  - Action 3: populated sidecar → row renders yellow with last-refusal sub-line; empty sidecar → row absent.
  - Action 5: `bwoc status agent-pi --banner` → matches daemon "I am alive" block; `--lang th` → Thai; `--json` → `{"banner": "..."}`; `--banner --all` → clap exit 2.
  - Action 7: `echo | bwoc start agent-pi` → exit 2 + actionable message; `--yes` path unaffected. Same for `stop`.

## Related

- Investigation source: `/investigate BWOC human in loop workflow` (this session)
- Sub-agents that did the work: 4 parallel `software-engineering` agent invocations (ids: `a2e1ebaf…`, `a21da987…`, `aae141ea…`, `aa0ff83b…`)
- Files: `crates/bwoc-cli/src/{dashboard,livecheck,main,start,status,stop}.rs`; `crates/bwoc-cli/locales/{en,th}/cli.ftl`; `modules/agent-template/.claude/hooks/inbox-auto-reply.sh`
