---
date: 2026-05-23
session: BWOC 2.0 release + Homebrew formula
tags:
  - phase/3
  - type/note
  - module/release
  - module/install
---

# 2026-05-23 — BWOC 2.0 Release + Homebrew Formula

First major version of the BWOC framework. CalVer tag `v2026.5.23-2` cut at 05:42 UTC after the paperwork seal commit `f3ad0c1`. `release.yml` ran clean — all 5 binaries (Linux x64/arm64, macOS x64/arm64, Windows x64) + 5 `.sha256` sidecars uploaded to GitHub Release on the first run. Homebrew formula added the same iter so non-Rust users can `brew install bwoc`.

## What changed

- **CalVer tag `v2026.5.23-2`** — annotated with "BWOC 2.0 — first major version" message. release.yml triggered, build matrix ran ~3 minutes, all 10 assets shipped.
- **`Formula/bwoc.rb`** (new) — Homebrew formula targeting the 4 Unix binaries from the release. Cross-platform via `on_macos`/`on_linux` + `on_arm`/`on_intel` blocks. SHA256s read from the published `.sha256` sidecars. Installs `bwoc` + `bwoc-agent` to the formula's `bin/`; tests via `--version`. Windows is intentionally NOT covered (no Homebrew on Windows; users grab the `.zip` from Releases directly).
- **`README.md` Getting Started** — Homebrew install promoted to the first option. Tap command + per-platform coverage. From-source `./scripts/install.sh` retained as the second option, `cargo install` as the third. Honors VERSION.md's policy that Cargo SemVer is dev-side and CalVer is release-side: users on the binary path never see the `2.0.X` Cargo version.

## Decisions

- **`Formula/bwoc.rb` in this repo** vs separate `homebrew-bwoc` repo — chose single-repo on user's call. Tap install line is slightly longer (`brew tap bemindlabs/bwoc <url>` instead of `brew tap bemindlabs/bwoc`) but the maintenance cost is one repo instead of two. If adoption climbs, a separate `homebrew-bwoc` repo can be a follow-up move.
- **macOS + Linux, not Windows** — Homebrew on Windows isn't a real install path; the existing `.zip` asset already serves Windows users. Adding it would mean documenting a path nobody uses.
- **Binary-only formula, no source build** — could have added a `build.rs` path that calls `cargo build --release`, but that requires every brew user to have a Rust toolchain. For a 2.0 release where the headline is "easy install", a pre-built binary formula is the right call. The CHANGELOG and migration sections already cover the from-source path for developers who want it.
- **SHA256s frozen in the formula** — every CalVer release will edit `Formula/bwoc.rb` to bump the tag fragments + sha256 hashes. Could automate via `release.yml` post-step, but that's a Phase 4 polish — the manual edit is small.
- **Test stanza uses `--version`, not the literal `2.0.0`** — the binary returns the Cargo SemVer (auto-bumped, today shipped as 2.0.29 due to session edit churn). Asserting literal `2.0.0` would lie; asserting "bwoc" in the output validates the binary actually runs.

## Alternatives considered

- **Separate `homebrew-bwoc` repo** — cleaner tap UX (`brew tap bemindlabs/bwoc` with no URL), but doubles the maintenance surface. Defer until either adoption demands it or this repo's Formula directory starts colliding with other artifacts.
- **homebrew-core submission** — the official central tap, no `tap` step needed. Requires the project to be "notable" (high GitHub stars, well-established, etc.) and the formula to pass strict audits. BWOC v2.0 is too young; revisit at v3 or once external adoption picks up.
- **Build-from-source formula** (`url ... "/refs/tags/v2026.5.23-2.tar.gz"` with `depends_on "rust"`) — eliminates the need to ship pre-built binaries through Homebrew. Rejected because (a) the release.yml already builds and uploads binaries; routing brew through source would duplicate the build at every user's machine, and (b) Apple-Silicon → Intel native builds would force every user to install a Rust toolchain to run `bwoc`. Pre-built path is strictly better.
- **`brew bottle` integration** — would let us host bottles (compressed pre-built archives) at the homebrew-core CDN. Same issue as homebrew-core: requires acceptance into the central tap.

## Bugs surfaced and fixed

- **`brew audit` blocked by stale Xcode CLT** — local audit failed with "Your Command Line Tools are too outdated". Not a formula bug; environmental. Verified the formula syntax via direct visual inspection + cross-check against the [Homebrew formula cookbook](https://docs.brew.sh/Formula-Cookbook). Real audit needs the user's machine to update Xcode CLT before running.
- **`brew tap` from local path clones at HEAD, doesn't see uncommitted Formula** — first test attempt failed with "No available formula" because the tap was a snapshot of the previous commit. Fix: tap from the published GitHub remote AFTER the formula commit lands. Or for local testing, commit + `brew untap && brew tap` cycle.

## Status / deferred

- **Auto-update formula SHA256 in release.yml** — every CalVer release should rewrite `Formula/bwoc.rb` with the new tag fragment + 4 fresh SHAs. The bash glue is straightforward (`sed` + the sidecars are already produced). Add it the next time release.yml is touched. Estimate: 30-line CI step.
- **Cask for the dashboard binary** — `bwoc` ships both the CLI + `bwoc-agent` daemon as terminal binaries. A Homebrew cask isn't needed (casks are for GUI apps). Skip.
- **`brew livecheck` block** — would auto-detect new releases from the GitHub Releases API. Worth adding when the release cadence stabilizes. Out of scope today.
- **Crates.io publish (`cargo install bwoc-cli`)** — third install path. Per VERSION.md, targeted for the Cargo `1.0.0` milestone — but with the SemVer baseline now at 2.0.0, that milestone is functionally "always". Re-decide: should we publish to crates.io as part of the v2.0 surface? Adds visibility but requires every published crate (`bwoc-core`, `bwoc-cli`, `bwoc-agent`) to be in shape. Deferred — needs its own session.
- **Windows MSI / WinGet** — explicit install path beyond the manual `.zip` download. Out of scope for 2.0; revisit when Windows adoption shows up in operator feedback.
- **`bwoc --version` output stability** — the binary returns the Cargo SemVer which auto-bumps on every edit. Users see version numbers like `2.0.29` that don't match the release tag `v2026.5.23-2`. Per VERSION.md the policy is "dual-namespace and intentional", but the brew formula test stanza had to dodge the issue (matches "bwoc" not a literal). Consider adding a release-tag-stamping pass to release.yml that pins `--version` output to the CalVer tag for release binaries. Not a blocker; surface noted.

## Test summary

- **release.yml** — 5 matrix builds + create-release job, all green. 10 assets uploaded. Run id `26324900411`.
- **SHA256s captured** from the 4 Unix tarball sidecars:
  - `aarch64-apple-darwin`: `a0224f7e…259c96af`
  - `x86_64-apple-darwin`: `9b04c8d5…71fef11`
  - `aarch64-unknown-linux-gnu`: `c15fd79d…36f9a56`
  - `x86_64-unknown-linux-gnu`: `456e23c0…941bc460`
- **Local `brew install` from this commit** — pending until formula lands on the remote tap (see "Bugs surfaced" above).
- **Workspace cargo gates** — 121 tests pass, clippy clean (final check before the tag push at f3ad0c1).

## Related

- Release: https://github.com/bemindlabs/BWOC-Framework/releases/tag/v2026.5.23-2
- CalVer policy: [`VERSION.md`](../VERSION.md) §"Versioning Policy — Dual Namespaces"
- Release pipeline: [`.github/workflows/release.yml`](../.github/workflows/release.yml)
- Migration guide: [`CHANGELOG.md`](../CHANGELOG.md) §v2026.5.23-2 → "Migration from v2026.5.23-1"
- Tap install: `brew tap bemindlabs/bwoc https://github.com/bemindlabs/BWOC-Framework && brew install bwoc`
