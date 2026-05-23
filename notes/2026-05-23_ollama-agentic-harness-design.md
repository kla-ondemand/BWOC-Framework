# 2026-05-23 — Ollama production agentic harness (design)

Design for a **BWOC-native agentic harness** that lets self-hosted models (Ollama first, any OpenAI-compatible endpoint after) act as full BWOC coding agents. Today BWOC is a pure CLI orchestrator — it execs vendor agentic CLIs (`spawn.rs:116` `Command::new(backend.cli_name())`) and makes **no model API calls**. Ollama has no agentic CLI, so BWOC must supply the agentic loop itself. This is a deliberate identity expansion (see §Identity tradeoff). This note records the architecture + decisions before any code; a bilingual spec (`docs/en/HARNESS.en.md` + `.th`) follows once the shape is agreed.

## Decisions (settled with the framework author)

- **Runtime:** `tokio` async — required for streaming + concurrent tool execution + task queue + telemetry. Accepts the dep weight as the cost of production-grade.
- **Protocol:** OpenAI-compatible `/v1/chat/completions` (tools + SSE streaming), **not** Ollama-native `/api/chat` — provider-neutral (Ollama, vLLM, LM Studio, llama.cpp server, OpenAI). Ollama is the first target. Grounded in **Samānattatā** (equal treatment of providers).
- **Isolation of dep weight:** the harness is a **separate crate `crates/bwoc-harness`** with its own heavy deps. `bwoc-cli` and `bwoc-agent` stay lean — a user who never runs self-host never pulls tokio/HTTP. This preserves the "zero-dep orchestrator" promise for the default path.
- **First artifact:** this note. Bilingual spec + crate scaffold deferred until architecture is confirmed.

## Architecture

```
bwoc spawn --backend ollama  →  bwoc-harness (new binary)
  load: AGENTS.md (system prompt) + persona + manifest(model, gates) + memory
  connect: OpenAI-compat endpoint (default Ollama http://localhost:11434/v1)
  agentic loop (Iddhipāda 4 — engine of work):
    build messages (system + history + tool schemas)
    → POST /v1/chat/completions stream=true tools=[…]
    → stream token deltas (emit events) + accumulate tool_calls
    → for each tool_call: permission → guardrails → sandbox → execute → capture
    → append assistant(tool_calls) + tool results → repeat
    → stop on: no tool_calls (final) | max-iterations | cancel | context-overflow→compact
    → emit telemetry; in task mode: bwoc task complete
```

**Crate layout** (`crates/bwoc-harness/src/`): `main.rs` (entry, context load) · `provider/` (OpenAI-compat client + SSE) · `agent_loop.rs` (turns, dispatch, stop/compaction) · `tools/` (registry + impls) · `policy/` (guardrails + permission) · `sandbox.rs` · `telemetry.rs` · `queue.rs` · `eval/`.

**Tool set the model can call:** `read_file`, `write_file`, `edit_file`, `list_dir`, `grep`, `run_command` (sandboxed), `git`/worktree, `run_gates` (lint/fmt/test/build from manifest), `bwoc_task` (claim/complete), `bwoc_send` (messaging), `memory_read/write`. Every tool passes permission → guardrails → sandbox.

## The 8 production components

| Component | Design | BWOC framework | Phase |
|---|---|---|---|
| **Safety guardrails** | Hard policy engine, runs **before** permission and cannot be overridden: block `rm -rf` repo root, secret writes/commits, identity spoof, gate bypass (`--no-verify`/`--force`), undeclared side-effects. | Sīla 5 + Taṇhā 3 | P2 |
| **Sandbox execution** | Confine all tool effects to the agent's worktree; fs write allowlist (deny outside repo/worktree); `run_command` allow/deny + arg scan (`curl\|sh`, `sudo`, force-push) + env scrub; OS layer (macOS `sandbox-exec`, Linux landlock/seccomp) pluggable, added later. | Sīla 5 + Anattā (worktree isolation) | P2 |
| **Permission system** | Per-tool / per-pattern modes `allow \| ask \| deny` from manifest + `.bwoc/harness-policy.toml`. `ask` → operator on TTY; non-TTY/autonomous → policy default (deny). Denials are fed back to the model as tool results. | Taṇhā 3 (gate the cravings) | P2 |
| **Tool authentication** | Credential broker: tools declare needed creds (e.g. `gh`→`GITHUB_TOKEN`); broker injects **scoped** creds from OS keyring into the child env at exec time only — never in the prompt, never logged. | Sīla (Adinnādāna) + Kalyāṇamitta trust | P3 |
| **Task queue** | Async, bounded, cancellable work queue feeding the loop; pulls claimable tasks from the Saṅgha shared list (`bwoc-core::team`) and local submissions; one in-flight task per worktree. | Saṅgha + Padhāna 4 | P3 |
| **Streaming** | SSE token stream from the model + a structured event stream (turn start, tool start/end, gate result, done) to stdout/socket for the TUI dashboard and logs. | Sammā-vācā (transparent speech) | P1 |
| **Telemetry** | Per-turn metrics (tokens in/out, latency, tool-call + denial counts, gate pass/fail, context tokens) appended to the existing `session-metrics.jsonl`; optional OpenTelemetry export behind a feature flag. | Satipaṭṭhāna 4 | P3 |
| **Agent eval framework** | Offline harness: task fixtures (repo-state + task + rubric) → run → score (gates pass? expected diff? optional LLM-judge); regression suite in CI; feeds the Paññā 3 self-improvement triggers already wired to `session-metrics`. | Paññā 3 + Bhāvanā 4 | P4 |

## Phasing

- **P0** — this design note → bilingual spec → spike: prove the OpenAI-compat tool-loop against a tool-capable Ollama model (qwen2.5-coder / llama3.1 / mistral-nemo), edit one file end-to-end.
- **P1** — core loop + tool set + streaming, single task, inside a worktree. Dev-only (no safety).
- **P2** — **safety: guardrails + sandbox + permission system. Non-negotiable before any non-toy use.**
- **P3** — task queue (Saṅgha integration) + telemetry + tool authentication.
- **P4** — eval framework + hardening (retry, fallback model, context compaction).
- **P5** — backend wiring: `Backend::Ollama` (`spawn.rs:17-22`) launches `bwoc-harness` instead of an external CLI; add `OLLAMA.md` symlink; update `check.rs`/`doctor.rs`/`status.rs`/`banner.rs` (file:line list captured in the backend-architecture exploration); docs.

## Identity tradeoff (must resolve before P1)

This makes BWOC a **model-API client + agent runtime**, contradicting the current VISION/README identity ("pure CLI orchestrator, zero runtime deps beyond libc"). Mitigation chosen: **quarantine the heavy deps in `bwoc-harness`** so the default `bwoc`/`bwoc-agent` path stays lean and dep-free. Still, VISION should be updated to name self-host as a first-class goal and explain the orchestrator-vs-runtime split. This is an expansion, not a trim — opposite of Mattaññutā's "smaller spec wins" — justified only if self-host is strategic.

## Alternatives considered

- **Exec an Ollama-capable agentic CLI** (aider/opencode/Codex-with-base-URL) as a normal backend — cheapest, keeps the orchestrator identity intact, **no new code**. Rejected because the user wants a BWOC-native harness (control over safety/telemetry/eval, no third-party agentic CLI dependency).
- **Ollama-native `/api/chat`** — rejected for OpenAI-compat (provider neutrality).
- **sync + threads** — rejected; streaming/concurrency/queue would be awkward and not production-grade.

## P0 spike — result (verified 2026-05-23)

A throwaway Python spike (`/tmp/bwoc-harness-spike/spike.py`, stdlib only, `stream=false`) drove the full tool-loop against the local Ollama OpenAI-compat endpoint (`:11434/v1/chat/completions`) and **edited a real file end-to-end**, sandbox-confined:

- **gemma4 (8B):** turn 1 `read_file` → turn 2 `write_file` (correct Thai unicode) → turn 3 final. Output was valid Python and ran (`greet("Pi")` → `สวัสดี, Pi`). Read-before-edit emerged on its own. ✓
- **llama3.2 (3B):** the same loop fired (mechanism is model-agnostic) but the model **garbled the content** — double-escaped unicode, skipped the read, broke structure. ✗

**Conclusions:** (1) the OpenAI-compat tool-loop + sandbox guardrail is sound — no protocol surprises from Ollama. (2) Model capability is decisive; small models are unusable for code edits → P1 needs a vetted-model gate + fallback. (3) `stream=false` sufficed for the spike; P1 adds streaming. (4) Wrong model tag (`gemma4:8b` vs pulled `gemma4`) returns HTTP 404 from `/v1` — the provider layer must validate the model exists up front.

## Open questions

- Tool-calling reliability varies by model; need a vetted model list + a non-tool fallback (ReAct-style text protocol) for models without native tool support.
- Daemon relationship: does `bwoc-agent --serve` spawn `bwoc-harness`, or is the harness a peer launched by `bwoc spawn`? (Leaning: `spawn` launches it; daemon stays IPC/inbox only.)
- Context-window management strategy (summarize vs truncate vs retrieve from memory).
- OS-sandbox scope for v1 (worktree+allowlist only) vs requiring landlock/sandbox-exec from P2.

## Related

- Backend architecture (exec-CLI model, no API calls today): `crates/bwoc-cli/src/spawn.rs`.
- Saṅgha task model the queue builds on: `crates/bwoc-core/src/team.rs`.
- Telemetry sink: `session-metrics.jsonl` (schema in template `AGENTS.md` §8b).
- Shipped in v2026.5.23-3 (2.1.0); this harness is the headline of the next cycle.
