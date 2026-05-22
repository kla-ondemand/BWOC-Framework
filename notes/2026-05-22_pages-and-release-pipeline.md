# 2026-05-22 — GitHub Pages + Release pipeline

End-of-day session wiring the public delivery channels: GitHub Pages for the docs site and a tag-driven release workflow for cross-platform binaries. Decisions on stack, signing, and version scheme were made up-front so the workflow files encode the choice rather than reflect drift.

## What changed

- **`.github/workflows/pages.yml`** — Jekyll-default Pages build. `actions/configure-pages → actions/jekyll-build-pages → actions/deploy-pages`. Source = `docs/`. Triggers on `push`/`docs/**`/`README.md`/`workflows/pages.yml` and `workflow_dispatch`. No manual `_site` build, no custom theme assets — leans entirely on the remote Cayman theme.
- **`docs/_config.yml`** — Jekyll config: title, description, `remote_theme: pages-themes/cayman@v0.2.0`, `jekyll-remote-theme` + `jekyll-relative-links` plugins (so the existing `*.en.md` ↔ `*.th.md` links inside `docs/` resolve correctly on the Pages site).
- **`docs/index.md`** — landing page for the Pages site. Lists EN docs, TH docs, links into `modules/agent-template/docs/` (still browsed on GH because the template ships with the agent), and external links to README/VISION/CHANGELOG/Releases.
- **`.github/workflows/release.yml`** — pre-existed in this session; patched two places to honor the explicit CalVer constraint:
  - Tag glob narrowed from `v*` → `v[0-9][0-9][0-9][0-9].*` (CalVer-shaped only).
  - `prerelease: ${{ contains(github.ref, '-') }}` removed — CalVer tags always contain `-<patch>`, so the heuristic would mark every release as prerelease. Default is now stable; flip manually if a build is experimental.
  - Added `fail_on_unmatched_files: true` to surface packaging mistakes early.
- **`.github/workflows/ci.yml`** — extended (this session, by a parallel edit) to split `quality` (fmt+clippy, Linux only) from `build-and-test` (matrix over ubuntu/macos/windows). Backs the README's "macOS · Linux · Windows" claim with proof. **Closes [gap-analysis-remediation](2026-05-22_gap-analysis-remediation.md) item 5.**
- **`README.md`** — Tech Stack `Distribution` row trimmed: "`cargo install bwoc-cli` + GitHub Release binaries (signed)" → "GitHub Release binaries with SHA-256 checksums; `cargo install --git` from source (crates.io publish targeted for 1.0)". Sammā-vācā — describe what currently ships, not what's aspirational.
- **`VERSION.md`** — "Versioning Policy" section rewritten to document the **dual versioning** explicitly. Cargo SemVer = development checkpoint (auto-bumped); Git tag CalVer `vYYYY.M.D-<patch>` = public release identity. Added a maintainer recipe for cutting a release.

## Decisions

- **CalVer for release tags** (user-specified). Format: `vYYYY.M.D-<patch>` (e.g. `v2026.5.22-0`). Same-day reissues bump the patch number. Cargo SemVer continues to autobump on `.rs` / `.toml` edits — the two namespaces coexist intentionally. Trying to push CalVer into Cargo.toml would fight Cargo's semver requirement; trying to enforce SemVer on tags would erase the CalVer signal users actually care about.
- **Jekyll default, zero tooling** (user-specified, recommended option). No mdBook, no Astro. Cayman remote-theme + Markdown is enough for a doctrine repo where the value is the text, not the polish.
- **Unsigned binaries + SHA-256 checksums** (user-specified, recommended option). Code signing requires per-platform certs the user hasn't authorized and storage the project doesn't have. Checksums prove integrity without taking on cert-management debt. README claim updated to match (Sammā-vācā).
- **Skip crates.io publish for now** (user-specified, recommended option). Targeted for the Cargo 1.0 milestone. Pre-1.0 publishing locks in an API the workspace isn't ready to freeze. Until then `cargo install --git` works for the determined.
- **Pages source = `docs/`, not repo root.** Keeps the Pages site focused on framework-level documentation; the cloneable agent template's doctrine lives under `modules/agent-template/docs/` and is browsed on GH (no reason to copy it into the Pages site — it travels with the template repo each agent forks).

## Alternatives considered

- **`actions/deploy-pages` with a custom static-site generator.** Rejected per the user's recommended-option pick. Jekyll default earns its place by being zero-config; a custom build is only worth it once the docs structure outgrows Jekyll's limits, which hasn't happened.
- **Tag glob `v*.*.*` instead of `v[0-9][0-9][0-9][0-9].*`.** Rejected — too lax. The narrower glob refuses ad-hoc legacy SemVer tags like `v0.1.0`, which would otherwise trigger a release that doesn't match the declared format. Sīla — encode the policy in the workflow.
- **Auto-detect prerelease from a secondary suffix (e.g. `v2026.5.22-0-rc1`).** Considered, rejected. Adds shape complexity to the CalVer scheme for a case (experimental release) the maintainer can express with one click in the GH UI. Mattaññutā.
- **Aggregate `SHA256SUMS.txt` for the whole release.** Considered, dropped. Per-archive `.sha256` sidecars give users what they need; one combined file would duplicate without adding signal.

## Bugs surfaced and fixed

- `release.yml` prerelease heuristic (`contains(github.ref, '-')`) would have marked every CalVer tag as prerelease — `v2026.5.22-0` always contains `-`. Patched.
- `release.yml` tag glob `v*` accepted any tag — not specific to CalVer. Tightened to `v[0-9][0-9][0-9][0-9].*`.
- README `Distribution` row asserted "signed" binaries and `cargo install bwoc-cli` from crates.io; neither was true. Updated to current ground truth.

## Status / deferred

Done in this session:

- Pages workflow + Jekyll config + landing page
- Release workflow patched for CalVer
- CI matrix (parallel edit by another session); README & VERSION.md updated to match

Deferred (out of scope for this pass — captured for follow-up):

- **Enable Pages in repo settings** — one-time manual click in `bemindlabs/BWOC-Framework → Settings → Pages → Source: GitHub Actions`. The workflow cannot do this for itself.
- **First CalVer tag** — `v2026.5.22-0` not yet cut. Waiting on maintainer authorization (the gap-remediation plan deferred crates.io publish + release-pipeline cutover; the pipeline is now in place but un-tagged).
- **Code signing** — Apple Developer cert + Windows Authenticode setup. Requires user-supplied secrets and a documented cert-rotation policy. No timeline yet.
- **macOS Universal binary** — currently `aarch64-apple-darwin` only on macos-latest. Adding `x86_64-apple-darwin` (Intel) is a one-line matrix addition; deferred until there is evidence of demand. (Note: the user's release.yml as written already includes both — this matrix is wider than my original plan.)
- **`cargo install bwoc-cli` from crates.io** — requires Cargo 1.0 milestone first.

## Related

- Spec touched: `VERSION.md` (Versioning Policy rewrite), `README.md` (Tech Stack `Distribution` row)
- Workflow added: `.github/workflows/pages.yml`
- Workflow patched: `.github/workflows/release.yml` (tag glob, prerelease, fail_on_unmatched_files)
- Workflow extended (parallel edit): `.github/workflows/ci.yml`
- Doc added: `docs/_config.yml`, `docs/index.md`
- Plan reference: [gap-analysis-remediation](2026-05-22_gap-analysis-remediation.md) — item 5 closed (CI matrix), item 1 still open (ROADMAP/README sync), item 2 still open (CHANGELOG trim)
- Prior note: [bwoc-new UX + framework hygiene](2026-05-22_bwoc-new-ux-and-framework-hygiene.md)
