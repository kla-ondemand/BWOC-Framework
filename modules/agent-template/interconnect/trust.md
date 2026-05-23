---
title: Inter-Agent Trust
aliases:
  - Trust Model
  - Kalyāṇamitta 7
tags:
  - group/agents
  - type/design
  - meta/template
status: draft (v2026.5.23 — spec only; no code yet)
canonical-source: AN 7.36 (Mitta Sutta) — Aṅguttara Nikāya 7.36
---

# Inter-Agent Trust

> [!abstract] Each agent declares 7 booleans drawn from the Kalyāṇamitta-7 ("seven qualities of a good friend") canonical list in AN 7.36. Other agents read those booleans and refuse messages from senders who lack qualities the recipient requires. Self-declared at incarnation, verified by `bwoc check`, never auto-elevated at runtime.

## Motivation

Agent → agent messaging (Sammā-vācā Phase 1) needs a way for one agent to refuse messages from peers that don't meet a baseline of trustworthiness. BWOC's foundation is Buddhist principles, so the trust model uses an existing canonical list rather than inventing a new schema — **Kalyāṇamitta 7** from the *Mitta Sutta* (AN 7.36), already referenced in [`PHILOSOPHY.en.md#15. Kalyāṇamitta 7`](../docs/en/PHILOSOPHY.en.md#15-kaly%C4%81%E1%B9%87amitta-7--inter-agent-trust-new).

Three design constraints chosen for v1:

1. **Self-declared booleans, not earned scores.** Each agent claims which of the 7 qualities it satisfies. No runtime telemetry that "upgrades" an agent's profile (which is easy to game). Downgrade-only hybrid models are deferred to v2.
2. **`bwoc check` verifies evidence.** Each boolean has a documented "what counts as evidence" rule. A declaration without evidence is a check violation.
3. **Recipient-driven refusal.** An agent that wants strict peers sets `requiredTrust: [...]` in its own manifest. Messages from senders missing any required quality are refused at the inbox layer.

This is **the simplest model that supports principled refusal** while staying auditable. It is not the final word — agent-to-agent trust will accrete observable behavior into the picture as Phase 3 matures.

## The 7 Qualities (AN 7.36)

The Mitta Sutta lists seven qualities of a good friend (Pali → English gloss → System-level meaning). The third column is BWOC's adaptation as recorded in [`PHILOSOPHY.en.md`](../docs/en/PHILOSOPHY.en.md).

| Pali | Literal | In the System | Manifest key |
|---|---|---|---|
| Piyo | Likeable / endearing | Pleasant to delegate to | `piyo` |
| Garu | Respected / weighty | Respectable in capability | `garu` |
| Bhāvanīyo | Admirable / cultivating | Helps us improve | `bhavaniyo` |
| Vattā | One who speaks | Speaks beneficial truth | `vatta` |
| Vacanakkhamo | Patient listener | Can take feedback | `vacanakkhamo` |
| Gambhīrañca kathaṃ kattā | Speaker of profound things | Can explain depth | `gambhira` |
| No caṭṭhāne niyojaye | Does not urge to unworthy ground | Does not lead astray | `noCatthana` |

Manifest keys are **camelCase** for compatibility with the existing `config.manifest.json` style. No diacritics in keys (avoids encoding hazards across backends).

## Manifest Schema

A new top-level `trust` block in `config.manifest.json`. Both halves are optional; absent block ≡ all-false (no qualities declared, no qualities required).

```json
{
  "agentId": "agent-{{name}}",
  "role": "{{agentRole}}",
  "trust": {
    "declared": {
      "piyo": true,
      "garu": false,
      "bhavaniyo": true,
      "vatta": true,
      "vacanakkhamo": true,
      "gambhira": false,
      "noCatthana": true
    },
    "requiredTrust": ["vatta", "vacanakkhamo", "noCatthana"]
  }
}
```

> [!note] `declared` describes **this agent's claim about itself**; `requiredTrust` describes **what this agent demands from peers who want to message it**. They are independent — an agent can require qualities it doesn't itself claim, and that's legitimate (recipients are entitled to their own bar).

## Evidence Rules (what `bwoc check` verifies)

A declared quality is **only valid if** the corresponding evidence exists. `bwoc check` reads the manifest, then validates each `true` declaration against the rule below. A `true` without evidence → check **violation** (exit 1). A `false` is always valid (no evidence needed).

| Quality | Evidence rule (what `bwoc check` looks for) |
|---|---|
| `piyo` | Persona scope is non-empty AND describes a concrete delegate-able task (`persona/README.md` Section "Scope" is filled in). Delegation needs a clear handle to feel pleasant. |
| `garu` | At least one skill or mindset stub exists under `skills/` or `mindsets/`. Respectability requires *some* demonstrated competency surface. |
| `bhavaniyo` | `mindsets/` contains an entry whose name or content references improvement / verification / right-amount (Yoniso Manasikāra / Mattaññutā tags). Helping peers improve presumes an explicit improvement frame. |
| `vatta` | The persona's out-of-scope (anti-scope) is non-empty. Speaking beneficial truth requires being honest about what you DON'T do. Empty anti-scope ≡ no commitment to truthful refusal. |
| `vacanakkhamo` | An inbox flow has been exercised at least once (`.bwoc/inbox.jsonl` exists and is non-empty, OR `interconnect/feedback.md` documents how the agent handles feedback). |
| `gambhira` | At least one skill or doc file under the agent root is ≥ 50 lines AND mentions philosophy linkage (Pali term OR philosophical framework name). Profundity needs concrete depth, not just claims. |
| `noCatthana` | `persona/README.md` Section "Anti-scope" exists AND includes at least one explicit "will refuse" entry. Refusing inappropriate requests is the foundation of not leading astray. |

These rules are deliberately mechanical. They don't measure *actual* trustworthiness — they measure whether the agent has the structural pieces in place to even attempt the quality. Honest claims still depend on the human operator. The framework's role is to **catch obvious lies** (claim `gambhira` with no docs), not to certify virtue.

## Read API

```
bwoc trust <agent>              # human table: 7 booleans + requiredTrust list
bwoc trust <agent> --json       # { "declared": {…}, "requiredTrust": […] }
```

Status: command **not yet implemented**. Spec only.

Reading another agent's trust profile from a script:
```bash
bwoc trust agent-beta --json | jq -r '.declared.vatta'
# → true | false
```

## Refusal Semantics

When `bwoc send <recipient> <message>` (or future agent-originated send) appends a JSONL envelope:

1. The recipient's daemon reads the envelope on its next poll.
2. If recipient has a non-empty `trust.requiredTrust` array, the daemon resolves the **sender's** manifest and reads `trust.declared`.
3. If **any** required quality is missing or `false` in the sender's declaration, the daemon:
   - Marks the envelope as `refused` (does NOT delete it — auditability matters).
   - Writes a `refused: { reason: "missing_trust", missing: [qualities] }` field on the envelope.
   - Continues processing later envelopes normally.
4. The sender does NOT get an automatic notification of refusal. They can `bwoc inbox <recipient> --json | jq '.[] | select(.refused)'` if they're interested.

Sender == `user` is a special case: user-originated messages always pass (the user is by definition above the trust gate). Trust gates govern agent→agent messaging only.

Default behavior — `trust.requiredTrust` empty or absent — is **no gating**. The framework ships permissive by default; recipients opt in to refusal.

## What This Spec Does NOT Cover

- **Runtime adjustments.** v1 is strictly declared; no telemetry-driven score changes. Hybrid model deferred to v2.
- **Signing / proof of identity.** A malicious clone could lie in its `config.manifest.json`. Identity proofing (signed manifests, etc.) is a separate Phase 3 work item — this spec assumes honest declarations from agents within a workspace boundary.
- **Reputation across workspaces.** Trust is per-workspace. A trusted agent in workspace A is a stranger in workspace B until incarnated there.
- **Notification of refusal back to sender.** Deliberately omitted — refusal is the recipient's prerogative, not a contract to inform the sender. Listening is the sender's responsibility.

## Implementation Order (when code work begins)

1. `bwoc-core::Manifest`: deserialize the `trust` block. Backward-compatible: missing block ≡ defaults.
2. `bwoc check`: add 7 verification checks per `evidence-rules` above. Surface as PASS / WARN / FAIL per quality.
3. `bwoc trust <agent>` read command: table + `--json` output.
4. `bwoc-agent --serve`: on inbox poll, resolve sender's `trust.declared`, compare against own `requiredTrust`, mark refused envelopes.
5. CHANGELOG row + ROADMAP cross-reference + bilingual TH parity (`trust.th.md` mirrors this file).

Each step is mergeable independently. Step 4 is the only one with runtime risk and should ship behind a `BWOC_TRUST_GATING=1` env opt-in initially.

## Cross-References

- [`PHILOSOPHY.en.md` #15. Kalyāṇamitta 7](../docs/en/PHILOSOPHY.en.md) — the philosophical mapping this spec implements.
- [`capabilities.md`](capabilities.md) — capability declaration (skill registry); trust composes with capabilities (a peer with the right skill AND the required trust).
- `interconnect/feedback.md` (proposed, not yet drafted) — how `vacanakkhamo` evidence is structured.
- AN 7.36 Mitta Sutta — canonical source ([SuttaCentral](https://suttacentral.net/an7.36)).
