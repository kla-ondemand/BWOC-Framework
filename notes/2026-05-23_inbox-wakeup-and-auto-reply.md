---
title: "Inbox tmux wakeup + Stop-hook auto-reply"
date: 2026-05-23
tags:
  - type/dev-log
  - group/cli
  - group/agent-template
  - meta/messaging
---

# 2026-05-23 — Inbox tmux wakeup + Stop-hook auto-reply

Ported the bus-wakeup + auto-reply mechanism from `/Users/lps/Learn/it-app-workspace/bin` (single-bus + tmux + Stop hook) onto BWOC's per-agent inbox model. The motivation is the same as the upstream: shorten latency between *envelope arrived* and *agent responding* for two cases — a peer waiting on an interactive answer, and a multi-agent flow where the orchestrator wants its prompt to land inside the next assistant turn rather than after the daemon's poll interval.

## What changed

| Surface | Change |
|---|---|
| `crates/bwoc-cli/src/send.rs` | Envelope JSON gains `messageId` (always) + optional `replyTo`. New `notify_tmux()` does best-effort `tmux send-keys` after successful inbox append. New `generate_message_id()` builds `msg-<utc-slug>-<5hex>` without pulling in `rand` (uses sub-second nanos). Three new tests cover id format, replyTo round-trip, and slug shape. |
| `crates/bwoc-cli/src/main.rs` | `bwoc send` gains `--reply-to <msg-id>` and `--no-wakeup` flags; `SendArgs` extended in both clap struct + runtime struct. |
| `modules/agent-template/.claude/hooks/inbox-auto-reply.sh` (new) | Claude Code Stop hook. Resolves self codename via walked-up `config.manifest.json`, parses transcript, detects the marker, shells out to `bwoc send --from <self> --reply-to <id> --no-wakeup <sender> "<reply>"`. Silent on missing marker, sender=user, manifest absent, or empty assistant text. |
| `modules/agent-template/.claude/settings.json` (new) | Wires the hook into the template's `hooks.Stop` array. |
| `modules/agent-template/interconnect/messaging.md` + `.th.md` | Bilingual parity. §Envelope Schema gains a field table (the change-set lives here, not in the prose). §CLI Surface adds `--reply-to` / `--no-wakeup`. New §Wakeup & Auto-Reply explains the two halves and the per-backend deferral matrix. |
| `CHANGELOG.md` | Unreleased entry under §Added. |

## Decisions

- **Two halves, one protocol.** The native side (Rust in `bwoc send`) and the hook side (bash Stop hook) are coupled only by the marker format `[bwoc inbox <msg-id> from <sender>]`. Anything that can write that marker (tmux send-keys, a direct CLI invocation, a future Antigravity hook) plugs in without bwoc-cli changes. Samānattatā by interface, not by implementation.
- **`messageId` always; `replyTo` optional.** Required `messageId` means recipients can always thread without reading-then-writing back. Optional `replyTo` means first-turn envelopes don't lie about being responses. Backward compat is preserved because the daemon parses envelopes as `serde_json::Value` and only reads `from` for trust evaluation.
- **No `rand` dependency.** Generating the 5-hex suffix from `SystemTime::now().subsec_nanos() & 0xF_FFFF` avoids a new crate. Two sends inside the same wallclock second still get distinct ids unless they land on the exact same nanosecond — acceptable for a workspace-scoped audit id, and `cargo` dependency creep stays at zero. Mattaññutā.
- **`--no-wakeup` and `BWOC_DISABLE_TMUX_WAKEUP` both.** The flag is for the principled case (the auto-reply hook explicitly suppresses wakeup on its replies); the env is for the lazy case (CI, tests). Test suite uses the env so the existing tests in `send.rs` don't try to spawn tmux.
- **Hook walks for `config.manifest.json`, not `state.json`.** Upstream `bus-reply-hook.sh` reads `state.json`. BWOC doesn't have a state.json — the manifest is the canonical place for `agentId`. Walked-up search lets the hook work regardless of where in the agent tree Claude Code was launched.
- **Sender `user` ⇒ skip auto-reply.** Humans read the inbox/logs directly; mirroring their own assistant turn back to them would be noise and could create a loop if a user prompt happened to contain the marker text literally.
- **Hook in `modules/agent-template/.claude/`, not framework-root `.claude/`.** This is per-agent-on-incarnation behavior, not framework-author tooling. The framework root's `.claude/` continues to hold `auto-version.sh` and `bilingual-reminder.sh` — author-time hooks, not agent-runtime hooks.

## Alternatives considered

- **Single shared `agents/_shared/messages.jsonl` like upstream.** Rejected — BWOC's per-agent `inbox.jsonl` is already the established contract (trust gating, refusal sidecar, daemon cursor all key off it). Re-introducing a shared bus would fork the format.
- **`bwoc reply` as a dedicated verb.** Considered. Cut because `bwoc send --reply-to <id>` covers every reply case without growing the verb surface. Mattaññutā again.
- **Tmux wakeup as a separate `bwoc notify` verb.** Rejected — the wakeup is conceptually tied to envelope delivery, and splitting it would create a race window where the envelope landed but the wakeup hadn't fired yet. Inline + best-effort keeps the two ordered.
- **Generate `messageId` in `bwoc-core` as a typed `MessageId` newtype.** Rejected for now — the envelope is still an inline JSON object, no typed Envelope struct exists yet. Lifting the id into core can wait until we have a second consumer (likely when the v2 signed-envelope work needs a stable id surface).
- **Use timestamp (`ts`) as the id.** Rejected — sub-second collisions are too easy on a busy bus, and `ts` isn't intended to be a primary key.

## Bugs surfaced and fixed

- Initial draft of the `Sent to ...` print had a positional/named-arg mismatch — fixed in the same change before compilation.

## Status / deferred

- Manual tmux smoke test (spawn two agents in tmux, send between them, observe wake + reply) is **not yet run** — gates passed `cargo build` but the live two-process verification needs operator hands. Captured here so we don't forget. Next operator session should run it before tagging.
- Antigravity / Codex / Kimi hook equivalents: deferred until each backend's hook surface is identified. Protocol contract is documented in messaging.md so when those land, no Rust changes are required.
- `bwoc spawn` does not currently wrap the backend CLI in tmux. Operators who want the wakeup feature must wrap manually (e.g. `tmux new-session -s pi 'bwoc spawn pi'`). Auto-wrapping is a separate operator-ergonomics decision — flagged as a future follow-up but not blocked on.

## Related

- Upstream pattern: `/Users/lps/Learn/it-app-workspace/bin/{agent-send,agent-reply,agent-ls,bus-reply-hook.sh}`.
- BWOC spec touched: `modules/agent-template/interconnect/messaging.md` + `.th.md`.
- Trust integration (unchanged): `modules/agent-template/interconnect/trust.md` — the new fields don't alter refusal logic.
- Philosophy: Mattaññutā (additive schema, no new deps), Samānattatā (protocol shared across backends), Yoniso Manasikāra (verified the upstream pattern before porting).
