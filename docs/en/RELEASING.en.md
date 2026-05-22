---
title: Releasing BWOC
aliases:
  - Release Process
tags:
  - group/framework
  - type/process
  - meta/operations
---

# Releasing BWOC

> [!abstract] How to cut a release of the bwoc toolkit (`bwoc` CLI + `bwoc-agent` daemon). Releases are tag-driven: pushing a `v*` tag triggers the cross-platform build + GitHub Release upload pipeline.

## Pre-flight

Before tagging, the maintainer should verify:

- [ ] **CI is green** on `main` for the most recent commit — see [Actions → CI](https://github.com/bemindlabs/BWOC-Framework/actions/workflows/ci.yml).
- [ ] **`CHANGELOG.md`** has a section for the version being released. The current `[Unreleased]` block can be renamed to `[X.Y.Z] — YYYY-MM-DD`; a new empty `[Unreleased]` heading goes above it.
- [ ] **`VERSION.md`** `Software-Version` matches the tag (auto-bumped by the hook on every code edit; should already track the workspace `Cargo.toml`).
- [ ] **`Cargo.toml`** workspace version matches the tag.

## Cut the tag

```bash
git tag v0.1.0            # or v0.2.0-rc1, etc.
git push origin v0.1.0
```

The tag triggers [`.github/workflows/release.yml`](../../.github/workflows/release.yml):

1. **Matrix build** — 4 release-mode targets in parallel:
   - `x86_64-unknown-linux-gnu`
   - `aarch64-apple-darwin` (macOS Apple Silicon)
   - `x86_64-apple-darwin` (macOS Intel)
   - `x86_64-pc-windows-msvc`
2. **Package** each as `bwoc-<tag>-<target>.{tar.gz|zip}` containing `bwoc`, `bwoc-agent`, `README.md`, `LICENSE`, `CHANGELOG.md`.
3. **Sidecar** a `.sha256` next to each archive.
4. **Auto-create GitHub Release** with notes generated from the commit range since the previous tag.
5. **Upload** all artifacts to the release.

## Pre-release vs final

Tags containing a hyphen are auto-marked as **prerelease**:

| Tag | Type |
|---|---|
| `v0.1.0` | final |
| `v0.1.0-rc1` | prerelease (release candidate) |
| `v0.2.0-beta.3` | prerelease (beta) |

GitHub's prerelease flag affects auto-update tooling and "Latest release" badges.

## Versioning policy

BWOC uses [SemVer 2.0.0](https://semver.org/) for the `bwoc` toolkit binaries. See [`VERSION.md`](../../VERSION.md) for the full policy and the phase-vs-version distinction (Phase 1 / Phase 2 / etc. is a roadmap concept, not a version axis).

## What's NOT in the pipeline yet

- **Code signing** — Apple notarization (macOS) and Windows Authenticode are not configured. Binaries ship unsigned; users see "untrusted developer" prompts on first launch. Adding signing requires the maintainer to provision certs and store the signing keys in GitHub Actions secrets.
- **Linux ARM / musl** — only `x86_64-unknown-linux-gnu` builds. ARM Linux (`aarch64-unknown-linux-gnu`) and musl-libc (`x86_64-unknown-linux-musl`) variants can be added to the matrix when there's user demand.
- **Homebrew formula / Scoop manifest / cargo binstall metadata** — distribution-system integrations live in their own ecosystems.

## Rolling back

If a tagged release ships broken artifacts:

1. **Don't** delete the tag — the GitHub Release retains the broken binaries, and users may already have downloaded them.
2. Cut a new patch tag with the fix (e.g. `v0.1.1` after `v0.1.0`).
3. Edit the broken release's notes to point at the replacement.

This matches the SemVer policy: patches never go backwards.

## See also

- [`.github/workflows/release.yml`](../../.github/workflows/release.yml) — the workflow this doc explains.
- [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) — the per-commit gate that should be green before tagging.
- [`CHANGELOG.md`](../../CHANGELOG.md) — what to update before tagging.
- [`VERSION.md`](../../VERSION.md) — current version + SemVer policy.
- [`ROADMAP.en.md`](ROADMAP.en.md) — phases (does not determine version).
