# Roadmap

Phase-by-phase plan for BWOC. **Phases** describe implementation milestones; each may span several SemVer releases. See [`VERSION.md`](../../VERSION.md) for the version-vs-phase distinction. See [`VISION.md`](../../VISION.md) for success criteria at 1-year and 3-year horizons.

---

## Current Status

**Active phase:** Phase 2 — *ṭhiti operations* — in progress. Phase 1 v2.0 DoD met.
**Software-Version:** see [`VERSION.md`](../../VERSION.md).
**Document-Version:** see [`VERSION.md`](../../VERSION.md).

---

## Phase 1 v2.0 — uppāda Foundation

**Definition of done:** end-to-end **uppāda** for one backend — incarnate · check · spawn an agent that runs.

### Completed

- Cargo workspace (`bwoc-core`, `bwoc-cli`, `bwoc-agent`) scaffold; edition 2024; MSRV 1.85.
- `VERSION.md` with `Software-Version`, `Document-Version`, and `Last-Updated`; auto-managed by `.claude/hooks/auto-version.sh`.
- Open-source hygiene: `VISION.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `CHANGELOG.md`; root `README.md` with badges, TOC, footer.
- Spec docs (all bilingual EN/TH): `PHILOSOPHY` §0.1 *The Arc*, `GLOSSARY`, `ARCHITECTURE`, `INCARNATION`, `WORKSPACE`, `NAMING`.
- Crate READMEs (`bwoc-core`, `bwoc-cli`, `bwoc-agent`).
- Claude Code tooling: 4 project skills (`/incarnate`, `/check-neutrality`, `/check-bilingual`, `/task-log`); 2 PostToolUse hooks (`bilingual-reminder`, `auto-version`).
- `incarnate.sh` and `check-agent-neutrality.sh` shell scripts in the template (work today; will be ported to Rust).

### Shipped in Phase 1 v2.0

All items below are now implemented. The phase's Definition of Done (end-to-end **uppāda** for one backend) is **met**. Only HELD policy items (`CODEOWNERS` · `ISSUE_TEMPLATE/config.yml`) remain pending user direction; the release pipeline now exists (see Phase 2).

| Item | Spec | Status |
|---|---|---|
| `bwoc init [path]` | [`WORKSPACE.en.md`](WORKSPACE.en.md#cli-surface) | ✓ |
| `bwoc workspace info` · `validate` | [`WORKSPACE.en.md`](WORKSPACE.en.md#cli-surface) | ✓ |
| `bwoc new <name>` (port of `incarnate.sh`) | [`INCARNATION.en.md`](INCARNATION.en.md) | ✓ |
| `bwoc check [path]` (port of `check-agent-neutrality.sh`) | [`crates/bwoc-cli/README.md`](../../crates/bwoc-cli/README.md) | ✓ |
| `bwoc spawn <name>` (minimal `exec`) | [`ARCHITECTURE.en.md`](ARCHITECTURE.en.md#information-flow--bwoc-spawn-agent-foo) | ✓ |
| `bwoc list` (reads `.bwoc/agents.toml`) | [`WORKSPACE.en.md`](WORKSPACE.en.md) | ✓ |
| `--lang` flag wired to Project Fluent (TH + EN locales) | [`crates/bwoc-cli/README.md`](../../crates/bwoc-cli/README.md) | ✓ all 8 surfaces (init/list/spawn/workspace info/workspace validate/check/new/bwoc-agent) |
| `/check-naming` skill (audit `*.md` against `NAMING.en.md`) | [`NAMING.en.md`](NAMING.en.md#audit) | ✓ + wired into `.github/workflows/docs.yml` |
| Runtime works from any directory | embedded `include_dir!` agent template + `BWOC_TEMPLATE` env + `~/.bwoc/template/` cache | ✓ |
| Manual major/minor version bumps | `scripts/bump-version.sh <level> [--software\|--document\|--both]` | ✓ (patch still auto-bumped by hook) |

---

## Phase 2 — ṭhiti Operations

**Definition of done:** an agent operates with a real control surface; multiple backends are exercised; releases are reproducible.

### Shipped in Phase 2

| Item | Notes |
|---|---|
| `bwoc-agent --serve` daemon | Unix-only (`.bwoc/agent.pid` + `.bwoc/agent.sock`; cfg-gated stub on Windows) |
| IPC control socket — line-text protocol | `PING`/`STATUS`/`STOP` over Unix domain socket; debuggable with `nc -U` |
| `bwoc status [name]` | Per-agent health + runtime indicator (●/○) + uptime via socket query |
| `bwoc list` | Registry view with runtime indicator + INBOX count + `--running` / `--status` / `--backend` filters |
| `bwoc send <to> <msg>` + `bwoc inbox <agent>` | JSONL inbox at `<agent>/.bwoc/inbox.jsonl`; `--watch` / `--clear` / `--limit` / `--json` |
| `bwoc doctor` | Env + workspace diagnostic; `--auto` sweeps stale `agent.pid` / `agent.sock` / `inbox.cursor` |
| `bwoc start <name>` (idempotent) | Flips registry + spawns `bwoc-agent --serve` if not running; `--no-daemon` opt-out |
| `bwoc ping <name>` | CLI client for the daemon's PING command |
| `bwoc chat <name>` (+ `--tmux`) | Auto-resolves backend from registry; exec's `bwoc spawn` |
| `bwoc dashboard` (TUI) | ratatui-based; agents pane + detail pane + 2s auto-refresh + `t` hotkey to tmux-spawn |
| Daemon-side inbox watch + cursor | Announces new envelopes to stderr; `.bwoc/inbox.cursor` survives restart |
| `--json` across read-only commands | `list`, `status`, `workspace info`, `workspace validate`, `check` |
| CI matrix | `ubuntu-latest` · `macos-latest` · `windows-latest` green on every push |
| Release pipeline (CalVer) | `release.yml` triggers on `v<YYYY>.<M>.<D>-<patch>` tag; 4 cross-platform binaries + `.sha256` to auto-created GitHub Release |
| Help system (in-binary) | 9 topics: `getting-started`, `backends`, `workspace`, `manifest`, `arc`, `lifecycle`, `daemon`, `messaging`, `persona` |
| Shell completion | `bwoc completion <bash\|zsh\|fish\|powershell\|elvish>` via clap_complete |
| `bwoc init` writes `.gitignore` | Excludes daemon ephemerals (PID/socket/cursor) for user workspaces |
| `bwoc new --scope / --out-of-scope / --mindsets / --skills` | Persona substitution + mindset/skill stub seeding at incarnation |
| Shared `livecheck` module | Consolidated 5 copies of `signal_zero_alive` / `running_pid` / `query_uptime` / `format_uptime` / `inbox_count` |
| `bwoc-agent --serve` Windows stub | Compiles + runs default mode; `--serve` returns exit 2 with "Unix-only" message |

### Remaining for ship

- **Restart-on-crash supervision** — the daemon currently exits on signal; auto-respawn / health-check loop not implemented.
- **`bwoc log <agent>`** — daemon emits to stderr currently; no log-tail IPC command.
- **Per-workspace memory** (`<workspace>/.bwoc/memory/`).
- **Cross-backend validation** — full uppāda + ṭhiti against all 4 backend CLIs in CI (proves Samānattatā).
- **Code signing** — Apple notarization + Windows Authenticode for release artifacts (user-cert authorization required).
- **Linux ARM / musl builds** — only `x86_64-unknown-linux-gnu` in the release matrix.
- **Memory mining tooling and pluggable Tier 2 backend interface.**
- **Windows named-pipe daemon path** — replace the cfg-gated stub with a real Windows implementation.

---

## Phase 3 — vaya + Interconnect

**Definition of done:** an agent's life ends cleanly; agents coordinate without a central authority.

### Shipped in Phase 3

| Item | Notes |
|---|---|
| `bwoc stop <name>` | Sends `STOP` over the socket (when daemon is alive) + flips registry status. Idempotent. |
| `bwoc retire <name>` | Removes from registry; `--keep-files` retains the agent dir. |
| `bwoc workspace prune` | Reconciles phantom registry entries vs orphan agent dirs; `--apply` removes safe drift. |
| User → agent inbox (sammā-vācā Phase 0) | `bwoc send` + `bwoc inbox` ship as JSONL envelopes; foundation for agent → agent messaging. |

### Remaining for Phase 3

- **Full vaya** for `bwoc retire` — currently registry-only with optional file delete; needs worktree cleanup + branch release + memory prune + interconnect deregistration.
- **Signal escalation** for `bwoc stop` — current behavior is socket `STOP` → exit; no SIGTERM → SIGKILL ladder if daemon ignores `STOP`.
- **Agent → agent messaging** — Sammā-vācā channel proper; Sāraṇīyadhamma 6 cordiality rules.
- **Trust scoring** — Kalyāṇamitta 7 qualities applied to capability declarations and message provenance.
- **`.bwoc/interconnect/`** per-workspace routing config.
- **Tier 2 memory backend reference implementation.**

---

## Phase 4 — Reference Agents + Fleet

**Definition of done:** ecosystem viability proven; cross-vendor production fleet governance is achievable.

- Three or more reference agents in the wild, built by maintainers outside the original authors (per [`VISION.md`](../../VISION.md) one-year success).
- Fleet dashboard — Aparihāniya-dhamma 7 governance applied to a real multi-agent installation.
- BWOC vocabulary (Yoniso manasikāra checks, Mattaññutā caps, Sīla baselines, Kalyāṇamitta trust scores) observed in codebases unaffiliated with this project (three-year success).
- Cross-vendor production fleet pattern in use at more than one organization.

---

## Cross-cutting (every phase)

- **Bilingual parity** — every spec doc has EN canonical + TH (and future languages); the bilingual-reminder hook gates this.
- **Backend neutrality** — every CLI feature works against any of the four declared backends; `/check-neutrality` gates this for `AGENTS.md`.
- **Doc-version + software-version stay consistent** — both auto-stamped on every Claude Code edit.
- **Open-source readiness** — every artifact a public contributor needs (CONTRIBUTING, SECURITY, CoC, LICENSE, VERSION, CHANGELOG, VISION, ROADMAP) is current and accurate.

---

## Non-Goals

See [`VISION.md` §Non-Goals](../../VISION.md#non-goals). Summary: BWOC is not a religion, not a runtime/SDK/LLM, not a replacement for DDD / Clean Architecture / SOLID, not vendor-aligned, and not a productivity framework.

---

## See Also

- [`VERSION.md`](../../VERSION.md) — current versions and SemVer policy.
- [`VISION.md`](../../VISION.md) — 1-year and 3-year success criteria.
- [`CHANGELOG.md`](../../CHANGELOG.md) — what shipped, when.
- [`ARCHITECTURE.en.md`](ARCHITECTURE.en.md) — how the components fit.
