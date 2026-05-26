# Proposal — `bwoc doctor` cargo / rustup probe (read-only)

| | |
|---|---|
| **Author** | `agent-rose` |
| **Date** | 2026-05-27 |
| **Status** | Proposal — operator review required before impl |
| **Origin** | S1 retro action item #1, restated in S3 kickoff brief from `agent-jisoo` (msg `20260526T133549Z-aa720`) |
| **Sprint** | S3 polish (no story points; parallel to BWOC-17) |
| **Replaces** | Inline proposal lost to `bwoc run` stdout truncation (S3 retro item #3 — file-persisted is the new pattern) |

---

## 1. Why

`bwoc doctor` is the one-stop diagnostic for environment + workspace health. Today it covers:

- `~/.bwoc/` home, workspace TOML, agents TOML
- Vendor backend CLIs on `PATH` (`claude` / `agy` / `codex` / `kimi`)
- Ollama-specific probes (`bwoc-harness` binary, `localhost:11434` reachability)
- Per-agent symlinks, stale PIDs / sockets, inbox cursors, log/inbox bloat

It does **not** currently probe the Rust toolchain. That gap matters because:

- **Contributors hit it first.** Every BWOC contributor needs `cargo` to run `cargo build/test/clippy/fmt` (the four mandatory gates in every agent's section 6). A missing or stale toolchain manifests as confusing build errors halfway through a session, not as a clear diagnostic up front.
- **`bwoc-harness` install** (the BWOC-25 unblocker) requires `cargo install --path crates/bwoc-harness`. Today `check_ollama()` already hints that path in its WARN message, but never verifies the prerequisite `cargo` is present. A user following the hint hits a second confusing failure.
- **Sprint 3 retro item #1** flagged this explicitly — Rust toolchain belongs in the same diagnostic surface as backend CLIs.

The fix is small, the upside is "one less foot-gun for new contributors and for agent-lisa landing BWOC-25". Asymmetric ROI.

---

## 2. Proposed CLI Surface

**No new subcommand. No new flag.** Extend `bwoc doctor` (and `bwoc doctor --auto`, `bwoc doctor --json`) with two additional read-only checks, surfaced in the existing report and JSON shape.

```
$ bwoc doctor
BWOC Doctor
===========
  PASS   ~/.bwoc/
  PASS   backends on PATH
  PASS   rustup                      ← new
  PASS   cargo                       ← new
  PASS   bwoc-harness binary
  PASS   ollama endpoint (localhost:11434)
  ...
```

`--json` adds two more entries to `results[]` with names `"rustup"` and `"cargo"`, summary counters tick accordingly, exit code semantics unchanged (`2` iff any `fail`).

**Why not a separate `bwoc doctor toolchain` subcommand:** the existing surface is a single flat list of checks (no grouping by category). Adding a subcommand would diverge from that pattern for a check that's ~30 LoC. Grouping is a separate refactor (out of scope for this proposal).

---

## 3. Probe Semantics

Both probes use the existing `which()` helper at `crates/bwoc-cli/src/doctor.rs:797` — same pattern as `check_backends()` and the Ollama harness probe.

### 3.1 `rustup` check

| State | Status | Detail message |
|---|---|---|
| `rustup --version` succeeds, parses semver | `Pass` | (none) |
| `rustup` not on `PATH` | `Warn` | `"rustup not found on PATH. Contributors need it for `cargo build/test/clippy/fmt`. Install: https://rustup.rs"` |
| `rustup --version` runs but fails or output unparseable | `Warn` | `"rustup present but `--version` returned non-zero / unparseable output: {stderr or stdout snippet}"` |

**Never `Fail`.** Many BWOC users only ever run `bwoc spawn` against `claude` / `agy` / `codex` — they don't need the Rust toolchain. WARN matches the policy used for `backends on PATH` (no claude CLI = WARN, not FAIL) and the Ollama probes.

### 3.2 `cargo` check

| State | Status | Detail message |
|---|---|---|
| `cargo --version` succeeds | `Pass` | (none) |
| `cargo` not on `PATH` | `Warn` | `"cargo not found on PATH. Required to build BWOC from source or `cargo install bwoc-harness`. Install: https://rustup.rs"` |
| `cargo --version` runs but fails | `Warn` | `"cargo present but `--version` returned non-zero: {stderr snippet}"` |

### 3.3 Version freshness — explicitly NOT in scope for v1

The probe **does not** parse the toolchain version and compare against a minimum. Reasons:

- BWOC's `rust-toolchain.toml` (if it ever lands — currently absent) is the canonical floor. The doctor probe shouldn't duplicate or contradict that.
- A "too old" check requires deciding what "too old" means per crate. That's a separate spec question for jisoo, not a doctor change.

The version string returned by `--version` **is** recorded in the WARN detail when the probe falls through (failure modes only) for diagnostic value — but parsed-and-compared logic is deferred to a follow-up.

### 3.4 Read-only invariant

The probe issues exactly two commands:

```
rustup --version
cargo --version
```

Both are pure read operations documented as side-effect-free by `rustup`'s own help (`rustup --help`). The probe never:

- Runs `rustup install`, `rustup update`, `rustup default`, or any `rustup toolchain ...`
- Runs `cargo install`, `cargo update`, or any cargo subcommand that mutates `~/.cargo/`
- Writes to `~/.cargo/`, `~/.rustup/`, or the workspace
- Modifies `PATH` or environment

This invariant is enforced by code review + a unit test that pattern-asserts the only `Command::new` calls in the new functions are `rustup` / `cargo` with arg `--version`.

---

## 4. Code Touch Points

Single file:

```
projects/bwoc/crates/bwoc-cli/src/doctor.rs   (+ ~60 LoC)
```

Specific edits:

1. **New function `check_rust_toolchain() -> Vec<CheckResult>`** — returns two `CheckResult` entries (`rustup`, `cargo`). Place it next to `check_backends()` (line 212) and `check_ollama()` (line 240). Same return-type pattern as `check_ollama()`.

2. **Wire into `run()`** — insert one `for r in check_rust_toolchain() { results.push(r); }` block between current `check_backends()` (line 53) and `check_ollama()` (line 58), in the "external programs on PATH" group.

3. **No changes to `emit_json()`** — the JSON shape is generic over `CheckResult`; the new entries flow through unchanged.

4. **No changes to `main.rs`** — `DoctorArgs` / `bwoc doctor` CLI surface is unchanged.

5. **No new dependencies.** Reuse the existing `Command::new` + `which()` helpers. No `which` crate, no `semver` crate.

**Estimated diff size:** +60 / -0 LoC in one file. The associated tests (section 5) add another ~40 LoC in the same file's `#[cfg(test)] mod tests`.

---

## 5. Test Plan

All tests in `doctor.rs`'s existing `mod tests` (line 879), following the same pattern as `missing_scaffold_dirs_reported_when_no_auto`:

### 5.1 Unit tests

| Test | Asserts |
|---|---|
| `rust_toolchain_probe_returns_two_results` | `check_rust_toolchain().len() == 2`, names are exactly `["rustup", "cargo"]` in order |
| `rust_toolchain_warns_when_rustup_missing` | With a stubbed `PATH=/tmp/empty`, `rustup` result is `Status::Warn(_)` and detail contains `"rustup not found"` and `"rustup.rs"` |
| `rust_toolchain_warns_when_cargo_missing` | Same idea, asserts the second entry warns |
| `rust_toolchain_passes_on_dev_host` | On a host with `rustup` + `cargo` present, both results are `Status::Pass` — gated by `#[cfg(...)]` or `if which("cargo").is_some()` skip-pattern to avoid CI hosts where neither is installed |

The "PATH stubbing" pattern: tests use `std::env::set_var("PATH", ...)` inside a `serial_test::serial` block, or use a process-local helper that takes a `PATH` argument. **Decision deferred to impl** — `doctor.rs` has no `serial_test` dep today, so the simpler path is to factor `check_rust_toolchain()` to accept a `which: impl Fn(&str) -> Option<PathBuf>` injectable. That keeps the test pure.

### 5.2 Integration touch

`bwoc doctor --json` smoke test (manual; no test infra today): run on the operator's machine, confirm:

- `results[]` contains entries with `name: "rustup"` and `name: "cargo"`
- Total entries increased by exactly 2 vs pre-change baseline
- Exit code unchanged (still 0 on a healthy machine)

If `bwoc check --all` total grew from 62 → 64 (or whatever pre-change baseline) after this change lands, that's the expected delta — call it out in the commit message.

### 5.3 Read-only verification

A grep-test in CI (or operator-run sanity check):

```bash
# Confirm the new check functions invoke no mutating rustup/cargo subcommands.
! rg -n 'Command::new\("rustup"\)\.arg\("(install|update|default|toolchain)"\)' \
    crates/bwoc-cli/src/doctor.rs
! rg -n 'Command::new\("cargo"\)\.arg\("(install|update)"\)' \
    crates/bwoc-cli/src/doctor.rs
```

This is belt-and-suspenders against future drift.

---

## 6. Out of Scope (explicit non-goals)

The following are **deliberately excluded** from this proposal and any subsequent impl. If a follow-up wants them, that's a new proposal.

| Out of scope | Why |
|---|---|
| **Installing `rustup` or `cargo`** | Doctor never installs anything. Even `--auto` only fixes safe things with one obvious correct answer (symlinks, scaffold dirs). Toolchain install is OS-specific, network-bound, and consent-required. |
| **`rustup update` or `cargo update`** | Mutates user state. Violates the read-only invariant (3.4). |
| **Setting a default toolchain (`rustup default ...`)** | Same as above. The probe must not change `~/.rustup/settings.toml`. |
| **Adding components (`rustup component add clippy`)** | Same as above. |
| **Version-floor enforcement** | Deferred (3.3). Needs a spec decision in jisoo's lane. |
| **Toolchain channel detection (stable/beta/nightly)** | Deferred. Adds complexity for no current user need. |
| **`rust-toolchain.toml` validation** | The workspace doesn't have one. If/when it does, that's a separate doctor check, not part of this one. |
| **New `bwoc doctor toolchain` subcommand** | The current doctor surface is a flat list (section 2). Subcommand grouping is a separate refactor. |
| **CI integration / matrix expansion** | The CI matrix is BWOC-17 / rose's lane. This proposal does not pre-empt that work. |

---

## 7. Risks & Mitigations

| Risk | Mitigation |
|---|---|
| `rustup --version` is slow on a cold cache (rare) | The probe is one process per command, both bounded by `Command::new(...).output()`. No additional timeout needed — `which()` already pays the same cost for vendor backends. |
| User has `cargo` from Homebrew (not `rustup`-managed) | Both probes are independent. `cargo` PASS + `rustup` WARN is a valid state. WARN detail message stays neutral about install method (points at `rustup.rs` but doesn't insist). |
| Windows users without `rustup` | The probe uses `Command::new` + `which()` — same path that already handles cross-platform vendor backends. No Windows-specific code path. |
| Future drift to a mutating subcommand | Section 5.3 grep-test catches it. Code review + reviewer awareness (this note in the repo) is the social mitigation. |

---

## 8. Sign-off Checklist

- [ ] Operator (พี่ต้นกล้า) reviews this proposal.
- [ ] `agent-jisoo` confirms the read-only invariant aligns with the neutrality/PHILOSOPHY constraints.
- [ ] `agent-lisa` confirms the touch point estimate (single file, ~60 LoC) is reasonable.
- [ ] If approved: a follow-up story (likely `BWOC-26` or similar) is added to the backlog with owner `agent-rose`, sprint TBD, estimate 1pt.
- [ ] After impl lands: this note moves from "proposal" to "design record" — status field updated, link from the implementing commit message back here.
