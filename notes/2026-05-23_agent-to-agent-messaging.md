---
date: 2026-05-23
session: agent → agent messaging — sammā-vācā Phase 1
tags:
  - phase/3
  - type/note
  - module/bwoc-cli
  - module/interconnect
---

# 2026-05-23 — Agent → Agent Messaging (Sammā-vācā Phase 1)

Third major slice of the day after trust step 4-5 and the dual-mode `bwoc check`. The trust gate was already operational from earlier today; this iter adds the *sender* side so an envelope can carry a real agent identity instead of the hardcoded `from: "user"`. The spec — `interconnect/messaging.md` (+ TH) — also lands today, including the Sāraṇīyadhamma 6 mapping that's been pending since ROADMAP Phase 3 was drafted.

## What changed

- **`interconnect/messaging.md` + `messaging.th.md`** — new spec under `modules/agent-template/interconnect/`. Covers:
  - Envelope schema with `from` semantics by value (`user` | `agent-<name>` | other).
  - CLI surface: `bwoc send --from <agent>` resolution rules (canonicalize bare name → `agent-<name>`; sender must exist in registry; manifest must be readable).
  - Sāraṇīyadhamma 6 (AN 6.11–12) mapped to engineering rules: API stability, kindly speech, charitable interpretation, observability, common Sīla baseline, shared philosophy graph.
  - Backward compatibility note (omitting `--from` keeps legacy `from: "user"` behavior).
  - Deferred: signed envelopes, cross-workspace messaging, broadcast.
- **`bwoc send --from <agent>` flag** — new clap field on `SendArgs` with `--from` long name. `bwoc-cli::main::SendArgs` and `bwoc-cli::send::SendArgs` both gain a `from: Option<String>` field that threads through `resolve_message`. Runtime path:
  - `None` → write `from: "user"` (unchanged legacy default).
  - `Some(name)` → canonicalize to `agent-<name>` if not already, look up in workspace registry; if missing → `SenderNotFound` (exit 2). Existence is the only validation here; trust verification stays at the recipient daemon.
- **Helper `canonicalize(name) -> String`** — shared idempotent normalization for `--to` and `--from`. Pulled out as a free function to avoid duplicating the `if name.starts_with("agent-")` check at three call sites.
- **5 new tests in `send::tests`** — sender-id substitution, canonical-form passthrough, SenderNotFound on unknown peer, legacy `from: None` → `"user"`, and a direct unit test for `canonicalize`.
- **CHANGELOG + ROADMAP** updated (both EN and TH). ROADMAP §Shipped grows a row for agent → agent messaging + a row for dual-mode `bwoc check`; §Remaining loses "agent → agent messaging" + "trust" (both done) and gains a "Trust v2" line (signed envelopes / warn mode / cross-workspace).

## Decisions

- **Sender existence is the only check at send-time; trust is the recipient's responsibility.** Considered also verifying the sender's `trust.declared` against the recipient's `requiredTrust` at send-time as a "fast-fail" UX, but rejected — the recipient might have changed `requiredTrust` between send and inbox-poll, and the spec is explicit that trust is evaluated at the daemon, not at the wire. Sending an envelope that will get refused is a legitimate flow (the operator may want the refusal to land in the inbox for audit).
- **`canonicalize` as a free function, not a `From` impl.** Considered `impl From<&str> for AgentId` to type-safe the canonical form, but the codebase uses plain `String` for agent IDs throughout — introducing a newtype here would force a ripple across many call sites with no immediate payoff (Mattaññutā). Free function is the right size today.
- **Recipient's daemon re-resolves sender's manifest on each envelope.** Not a new decision (already true in trust step 4), but worth restating: the daemon doesn't cache sender manifests. A sender that changes its declarations between send and poll is evaluated against its *current* state. This is intentional — trust is a property of the present claim.
- **Sāraṇīyadhamma 6 are norms, not enforced rules.** The spec maps the six conditions to engineering rules (kindly speech → no shouting, observability → write to JSONL, etc.) but `bwoc check` doesn't gate them. Considered adding lint rules (e.g., refuse envelopes with ALL CAPS message bodies), but that's both fragile (Unicode case is not just ASCII) and culture-specific. Norms work better as documentation that incarnated agents reference, not as hard checks.
- **Live-test scaffolding waits for the socket file instead of sleeping.** First attempt used `sleep 0.5` between background-spawning the daemon and sending — the daemon hadn't finished its inbox-cursor init yet, so it set `cursor = inbox_size` at startup and skipped the just-sent envelope as "history." Fixed by polling `[ -S .bwoc/agent.sock ]` for up to 3s (socket creation is the daemon's last setup step before entering the accept loop). Pattern worth reusing in future live tests.
- **Don't add a new envelope field for sender capabilities.** The capabilities spec (`interconnect/capabilities.md`) already lives separately. Cross-referencing at recipient time (via the sender's registered manifest) is cleaner than denormalizing capabilities into every envelope. Saves wire bytes and avoids stale claims.

## Alternatives considered

- **`bwoc-agent send <to>`** — agent-side send command, scoped to the daemon's own identity (always `from = own-agent-id`). Rejected for v1 — adds a second entry point with overlapping semantics. `bwoc send --from <self> <to>` covers the case from the CLI side without splitting the surface.
- **Sender's identity inferred from current working directory.** If you `cd` into `agents/agent-x` and run `bwoc send`, infer `from = agent-x` automatically. Rejected — cwd-based inference is magical and would surprise scripted callers. Explicit `--from` is clearer and consistent with how other sender-aware tools work.
- **Refuse with `reason: "schema_mismatch"` when sender's `trust.schemaVersion` differs from recipient's.** Considered for forward-compat, but the trust spec is explicit: "A v1 daemon reading a v2 manifest ignores unknown fields entirely (forward-compat through ignorance)." Adding a schema-mismatch refusal would break that contract. Deferred to v2.
- **Per-envelope HMAC.** Considered prototyping a workspace-local secret for signed envelopes ("identity proof") right here. Rejected — designing the secret rotation, key storage, and verification path properly is a separate piece of work. Today's v1 ships honest about its limit ("sender identity is asserted, not proven").

## Bugs surfaced and fixed

- **`Edit replace_all=true` over `workspace: Some(root.clone()),`** caught only single-line matches, missed the multi-line `send(SendArgs { ... workspace: ... })` inside the `send_appends_multiple_lines` test. Caught by `cargo build` (E0063: missing field `from`). Lesson: `replace_all` with a fragment that has high syntactic context is fine; without context, it can miss multi-line variants and need targeted follow-up.
- **Daemon initialization race in live test.** Described above. Fixed by socket-readiness polling.

## Status / deferred

- **`bwoc-agent` doesn't yet expose its own `send`.** A daemon-side `bwoc-agent send <to> <message>` would be the natural thing for an in-process agent to call, but for now the agent shells out to `bwoc send --from <self>`. Queued.
- **Sender-id verification beyond existence.** Today: sender must be in the workspace registry. Tomorrow's v2: HMAC-signed envelope proves the sender is *actually* the named agent (not a malicious clone). Specced in [`messaging.md` §What This Spec Does NOT Cover](../modules/agent-template/interconnect/messaging.md).
- **Broadcast (`bwoc send --all <message>`).** Useful operator surface (e.g., "every agent, please re-read your manifest"). Queued separately.
- **TUI / `bwoc list` surfacing of agent-to-agent traffic.** Currently `bwoc list` shows INBOX count but doesn't distinguish user-originated from agent-originated envelopes. A future column or filter would help operators see fleet conversation patterns at a glance.

## Test summary

- **bwoc-cli: 81 tests** (was 76, +5 new send tests).
- **Workspace total: 115 tests, 0 failures, clippy clean.**

Live verification (both scenarios end-to-end with real daemon):

1. **Scenario A — gated refusal.** Recipient declares `requiredTrust = ["vatta", "noCatthana"]`. Sender has no trust block → all required qualities missing.
   - Daemon stderr: `inbox REFUSED ← agent-sender: reason=missing_trust missing=["vatta", "noCatthana"]`.
   - Sidecar `inbox.refusals.jsonl` has `envelopeOffset=0` record.
   - `bwoc inbox recipient --json | jq -c '.messages[] | select(.refused)'` emits the refused envelope with `refused: {reason, missing}`.

2. **Scenario B — gated pass.** Same recipient. Sender declares `vatta=true, noCatthana=true`.
   - Daemon stderr: `inbox ← agent-sender: compliant peer hello` (normal pass, no REFUSED prefix).
   - Sidecar `inbox.refusals.jsonl` does not exist.
   - `bwoc inbox recipient --json` shows no `refused` field on the envelope.

## Related

- Spec (this iter): [`modules/agent-template/interconnect/messaging.md`](../modules/agent-template/interconnect/messaging.md) · [`.th.md`](../modules/agent-template/interconnect/messaging.th.md)
- Trust spec (referenced): [`modules/agent-template/interconnect/trust.md`](../modules/agent-template/interconnect/trust.md)
- Trust step 4 (the runtime gate this iter complements): [`notes/2026-05-23_trust-step-4.md`](./2026-05-23_trust-step-4.md)
- Dual-mode check (same day): [`notes/2026-05-23_check-dual-mode-and-personalize.md`](./2026-05-23_check-dual-mode-and-personalize.md)
- Commit: pending (this note ships with it)
