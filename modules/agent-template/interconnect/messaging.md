---
title: Agent-to-Agent Messaging
aliases:
  - Messaging
  - Sāraṇīyadhamma 6
tags:
  - group/agents
  - type/design
  - meta/template
status: draft (v2026.5.23 — sender-identity surface + Sāraṇīyadhamma 6 norms)
canonical-source: AN 6.11–12 (Sāraṇīyadhamma Sutta) — Aṅguttara Nikāya 6.11–12
---

# Agent-to-Agent Messaging

> [!abstract] Inter-agent messaging extends the user→agent inbox channel ([`send.rs`](../../../crates/bwoc-cli/src/send.rs)) with a verified sender identity. Recipients can refuse based on the Kalyāṇamitta-7 [[trust|trust profile]] of the sender (see [`trust.md`](trust.md)). The conventions for cordial speech come from Sāraṇīyadhamma 6 (AN 6.11–12) — six conditions of conciliation rendered as engineering rules.

## Motivation

`bwoc send` ships envelopes from a single hard-coded sender: `"from": "user"`. That's correct for human → agent communication, but inter-agent coordination needs each sender to identify itself so the recipient can:

1. Apply the Kalyāṇamitta-7 trust gate (already implemented in `bwoc-agent --serve`; see [`trust.md` §Refusal Semantics](trust.md)).
2. Render meaningful inbox history (`bwoc inbox` shows the actual sender, not a flattened "user").
3. Audit policy fire: refusals can name a real peer, not a placeholder.

Sender identity also lets the framework enforce **Sāraṇīyadhamma 6** — the six conditions of conciliation from the Mahāparinibbāna-sutta and AN 6.11. These are the canonical Buddhist guidance for living harmoniously in community. Mapped to inter-agent messaging:

| Pali | Literal | In the System |
|---|---|---|
| Mettā-kāya-kamma | Kindly bodily action | API stability — don't break envelope schema mid-flight |
| Mettā-vacī-kamma | Kindly speech | Factual, non-shouting `message` body; no profanity, no insult |
| Mettā-mano-kamma | Kindly thought | Interpret peer envelopes charitably — malformed ≠ malicious |
| Sādhāraṇa-bhogī | Sharing what is rightly gained | State visibility — write to JSONL inbox, no hidden channels |
| Sīla-sāmaññatā | Common virtue | Same Sīla 5 baseline + manifest schema on both ends |
| Diṭṭhi-sāmaññatā | Common right view | Reference the same `PHILOSOPHY.en.md` graph when justifying claims |

Three design constraints for v1:

1. **Sender identity is asserted, not proven.** Step 4 of trust shipped with the v1 simplification that senders self-declare in their manifest. Signed envelopes (HMAC over a workspace-local secret, etc.) are deferred to v2.
2. **Trust gating is opt-in at recipient.** A recipient with `requiredTrust = []` accepts every well-formed envelope from any sender — agent or user. Strict-by-default would break every existing flow on rollout.
3. **No new file shape.** The envelope schema gains the `from` field semantics but the on-disk JSONL stays identical.

## Envelope Schema

The on-disk envelope is one JSONL line per message in `<recipient>/.bwoc/inbox.jsonl`. Schema:

```json
{
  "ts":      "<ISO 8601 UTC>",
  "from":    "user" | "agent-<sender-name>",
  "to":      "agent-<recipient-name>",
  "message": "<UTF-8 text>"
}
```

`from` semantics by value:

| `from` value | Meaning | Trust gate |
|---|---|---|
| `"user"` | Human operator (via `bwoc send` default) | Always passes (recipient cannot refuse the user) |
| `"agent-<name>"` | Another agent in the same workspace | Subject to recipient's `requiredTrust` if gating is on |
| anything else | Reserved for future identity sources (signed external senders, etc.) | Refused with `reason: "unknown_sender"` |

The runtime side (the daemon poll + refusal logic) already handles all three cases as of trust step 4. This spec just names the contract.

## CLI Surface

```
bwoc send <to> <message>                    # from=user (default)
bwoc send <to> <message> --from <agent>     # from=agent-<name>
```

Resolution rules for `--from`:
- The argument is the agent's `name` (or full `agentId`); `agent-` prefix is added if absent. Mirrors `--to` resolution.
- The named sender MUST exist in the enclosing workspace's `agents.toml`. Unknown sender → exit 2 with a clear error.
- The sender's own `config.manifest.json` MUST be readable. Unreadable → exit 1.

The recipient daemon's refusal logic (already implemented) re-resolves the sender's manifest at envelope-arrival time to read its `trust.declared` block — so a sender that has changed its declarations between `send` time and `inbox-poll` time is evaluated against its *current* state. This is intentional: trust is a property of the sender's present claim, not of when the message was sent.

## Sāraṇīyadhamma 6 — Engineering Rules

The six conditions are not enforced by the framework today; they are **norms** that the agent template's `AGENTS.md` §3 (Communication / Sammā-vācā) should reflect. The intent is for an incarnated agent to internalize them as guidance, not for `bwoc check` to gate them.

### 1. Mettā-kāya-kamma — API stability

> Kindly bodily action: don't shift the ground under a peer.

- The JSONL envelope schema is **append-only**: new optional fields may be added; existing fields keep their semantics; required fields are never removed.
- File paths exposed in this spec (`.bwoc/inbox.jsonl`, `.bwoc/inbox.refusals.jsonl`) are part of the contract — moving them is a breaking change.
- Protocol changes to the daemon's Unix socket (`PING`/`STATUS`/`STOP`) follow the same discipline.

### 2. Mettā-vacī-kamma — Kindly speech

> Verbal action with loving-kindness: the `message` body should read like collegial direction, not a shout.

- Prefer declarative phrasing over imperatives when possible ("please run X" beats "RUN X NOW").
- Don't use ALL CAPS, profanity, or pejoratives in the `message` body. The framework does not enforce this; reviewers and operators do.
- An honest "I can't do this" is kinder than a misleading "OK" — see Vattā in the trust spec.

### 3. Mettā-mano-kamma — Kindly thought

> Mental action with loving-kindness: interpret peer envelopes charitably.

- Malformed JSON ≠ malicious — the daemon's inbox poll already handles parse failures by warning and continuing, not by suspecting attack.
- A missing optional field ≠ peer noncompliance — default values per spec.
- An unfamiliar sender (`from: agent-x` not in registry) gets a structured refusal with `reason: "unknown_sender"`, not silent drop.

### 4. Sādhāraṇa-bhogī — Sharing what is gained

> Sharing rightly-gained resources: state must be observable.

- All inbox traffic lives in `inbox.jsonl` (versionable, greppable, replayable).
- Refusals live in `inbox.refusals.jsonl` (auditable — never deleted; merged at read time).
- No agent stashes messages in a private channel the workspace can't see.

### 5. Sīla-sāmaññatā — Common virtue

> Same precepts on both ends.

- Both sender and recipient declare conformance to Sīla 5 ([`AGENTS.md` §9](../AGENTS.md)).
- Both pass `bwoc check` before participating in inter-agent flows.
- Both run a compatible manifest schema version (mismatched `schemaVersion` is a refusal reason in trust v2 — currently lenient in v1).

### 6. Diṭṭhi-sāmaññatā — Common right view

> Aligned goals: share the same philosophical reference.

- Claims made in a `message` that reference a Buddhist framework SHOULD link to the canonical entry in `PHILOSOPHY.en.md`. This is a convention, not a wire-format requirement.
- Cross-agent specs (this file, `trust.md`, `capabilities.md`) live under `interconnect/` so every agent template ships them at the same path.

## Backward Compatibility

- Existing `bwoc send <to> <message>` invocations continue to write `from: "user"` envelopes verbatim — no behavior change.
- `--from` defaults to `user` when omitted. Scripts that don't pass it are unaffected.
- Old envelopes (pre-spec) with `from: "user"` deserialize identically. The recipient daemon's existing user-passthrough remains the codepath.

## Implementation Order

1. ✓ `bwoc-agent --serve` daemon-side refusal for non-`user` senders — **shipped in trust step 4**. The runtime side is already done; this spec just documents the contract.
2. `bwoc send --from <agent>` — sender-identity flag in `bwoc-cli`. (This iter.)
3. Tests + live verification of agent → agent flow with the trust gate. (This iter.)
4. CHANGELOG + ROADMAP cross-reference. (This iter.)
5. **Deferred (v2):** signed envelopes, sender-identity proof, cross-workspace messaging, broadcast (`bwoc send --all`).

## What This Spec Does NOT Cover

- **Signed envelopes / identity proof.** A workspace-local secret HMAC over the envelope JSON is the obvious v2 path. Today's threat model accepts that a malicious clone could write `from: agent-bob` despite being a different agent — trust verification today still operates against the sender's *manifest*, which is the per-agent file on disk.
- **Cross-workspace messaging.** Trust is per-workspace ([`trust.md` §What This Spec Does NOT Cover](trust.md)). An envelope addressed to an agent in another workspace is undefined behavior in v1.
- **Broadcast / fan-out.** `bwoc send --all <message>` is a useful operator surface but not a sender-identity concern. Queued as separate work.
- **Routing through intermediaries.** All messaging is point-to-point. An agent that wants to relay must explicitly read from its own inbox and re-send.

## Spec Revision History

- **v1 / 2026-05-23 (initial draft):** Envelope schema + `--from <agent>` CLI surface + Sāraṇīyadhamma 6 mapping to engineering rules. Trust gate integration already operational from trust step 4 shipped earlier today.

## Cross-References

- [`trust.md`](trust.md) — Kalyāṇamitta-7 trust model; the refusal gate operates on the `from` field this spec defines.
- [`capabilities.md`](capabilities.md) — capability declaration; a peer with the right skill AND the required trust AND a clean Sāraṇīyadhamma posture is the full picture.
- [`AGENTS.md` §3 (Communication)](../AGENTS.md) — Sammā-vācā principles applied to user-facing speech; the same rules apply peer-to-peer.
- [`PHILOSOPHY.en.md` #13. Sāraṇīyadhamma 6](../docs/en/PHILOSOPHY.en.md) — canonical reference for the six conditions.
- AN 6.11–12 — canonical source ([SuttaCentral AN 6.11](https://suttacentral.net/an6.11), [AN 6.12](https://suttacentral.net/an6.12)).
