# Version

> **Auto-maintained header.** The hook `.claude/hooks/auto-version.sh` bumps the patch number and stamps `Last-Updated` on every Claude Code edit. Software-Version is canonical in `Cargo.toml`; Document-Version is canonical here.

**Software-Version:** `0.1.405`   *(canonical in `Cargo.toml` — bumped on `.rs` / `.toml` edits)*
**Document-Version:** `1.0.120`   *(canonical here — bumped on `.md` edits)*
**Phase:** Phase 1 v2.0 — *in progress*
**Specification:** [`AGENTS.md`](modules/agent-template/AGENTS.md) v2.0
**Last-Updated:** `2026-05-22T14:10:00Z`   *(UTC, ISO 8601 — stamped on every edit)*

---

## Source of Truth

### Software-Version

Set in `Cargo.toml` at the workspace level:

```toml
[workspace.package]
version = "0.1.0"
```

All three crates inherit it via `version.workspace = true`:

- [`crates/bwoc-core`](crates/bwoc-core/Cargo.toml)
- [`crates/bwoc-cli`](crates/bwoc-cli/Cargo.toml)
- [`crates/bwoc-agent`](crates/bwoc-agent/Cargo.toml)

The hook reads from `Cargo.toml` `[workspace.package].version`, bumps the patch component on any `.rs` / `.toml` write, then mirrors the new value into the `Software-Version` line above.

### Document-Version

The `Document-Version` line above is canonical for the framework documentation set. Bumped on any `.md` write. Independent of software version — they evolve at different cadences.

Per-document frontmatter `Version` fields (e.g. `PHILOSOPHY.en.md` "Version: 2.0", `AGENTS.md` "Version: 2.0") track **specification semantic version**, which is a separate concern and is **not** touched by the hook. Those reflect breaking changes to the spec; they are bumped intentionally, not on every edit.

### Last-Updated

UTC, ISO 8601. Updated on every edit regardless of file type. Tracks last activity, not last release.

---

## Versioning Policy

BWOC follows [Semantic Versioning 2.0.0](https://semver.org/):

| Bump | When |
|---|---|
| **MAJOR** | Breaking change to the framework specification (`AGENTS.md`), `config.manifest.json` schema, or any documented public CLI surface. |
| **MINOR** | New capability that does not break existing agents — new CLI command, new optional manifest field, new specification section. |
| **PATCH** | Backward-compatible fix or clarification — auto-bumped by the hook on every edit. |

Pre-1.0 (`0.x.y`) means the public surface is not yet stable. Breaking changes may land in any minor release until `1.0.0` is tagged.

## Phase vs Version

**Phase** describes implementation milestones; **version** describes release identity. They are independent — Phase 1 v2.0 may span several SemVer releases (e.g. `0.1.0`, `0.2.0`, `0.3.0`) before yielding to Phase 2.

| Phase | Scope |
|---|---|
| Phase 1 v2.0 | Native Rust CLI (`bwoc`) + agent runtime (`bwoc-agent`). DoD: end-to-end **uppāda** (incarnate · check · spawn). |
| Phase 2 | **ṭhiti** commands — list, status, log, send, supervision. |
| Phase 3 | **vaya** + interconnect — stop, retire, inter-agent protocol. |
| Phase 4 | Reference agents, fleet dashboard. |

See the [README Status table](README.md#status) for current phase progress.

---

## Manual Bump (when needed)

The hook handles PATCH automatically. For MINOR / MAJOR, edit `Cargo.toml` `[workspace.package].version` directly and update this file's `Software-Version` line. For document MINOR / MAJOR, edit the `Document-Version` line here directly.

The hook does not undo manual edits to higher-order components; only the patch is auto-managed.
