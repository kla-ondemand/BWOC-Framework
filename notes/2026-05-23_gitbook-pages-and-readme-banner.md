# 2026-05-23 — GitBook-like Pages site + README banner

Reshaped the GitHub Pages docs site (`docs/`) from the single-page Cayman theme into a
GitBook-style docs site (persistent sidebar nav + client-side search), and added a banner
image to the root `README.md`. Tracked as issue [#2](https://github.com/bemindlabs/BWOC-Framework/issues/2).
Work landed on branch `docs/gitbook-pages` (uncommitted at time of note).

## What changed

- `docs/_config.yml`: `remote_theme` Cayman → `just-the-docs/just-the-docs@v0.10.0`; enabled
  `search`, `aux_links` (GitHub), `back_to_top`, `footer_content`. Kept `jekyll-remote-theme` +
  `jekyll-relative-links`; added `jekyll-seo-tag`.
- New nav grouping pages: `docs/en.md` (`title: English`) and `docs/th.md` (`title: ภาษาไทย`),
  both `has_children: true`. They form the two top-level sidebar sections.
- `docs/index.md`: now `title: Home`, `nav_order: 1`, `permalink: /`.
- Added Just-the-Docs frontmatter (`title` / `parent` / `nav_order`) to all 18 spec pages
  (`docs/en/*.en.md` + `docs/th/*.th.md`). Reading order matches the old `index.md` link list:
  Architecture(1) · Incarnation(2) · Workspace(3) · Naming(4) · Glossary(5) · Roadmap(6) ·
  Releasing(7) · Fleet-Governance(8) · FAQ(9). EN/TH pairs share the same `nav_order`.
- `README.md`: banner image at top, stored at `assets/banner.png` (new root `assets/` dir).

## Decisions

- **just-the-docs over mkdocs/Docusaurus** — it runs under the existing
  `actions/jekyll-build-pages@v1` remote-theme path (same mechanism Cayman used), so no new
  build pipeline. *Mattaññutā* — smallest change that yields the GitBook shape.
- **Two grouping pages instead of flat nav** — gives collapsible EN / ภาษาไทย sections (the
  GitBook feel) for the cost of two scaffold files.
- **Central nav spec, fan-out edits** — `nav_order` is a cross-file/global concern, so the order
  was fixed centrally and the per-file frontmatter handed to workers as exact specs; EN+TH of a
  topic were always edited by the same worker to hold *bilingual parity*.
- **Pre-existing frontmatter preserved** — `RELEASING` and `FLEET-GOVERNANCE` (en+th) kept their
  Obsidian `title`/`aliases`/`tags`; only `parent`/`nav_order` were injected.

## Alternatives considered

- mkdocs-material (closest to GitBook visually) — rejected: needs a custom build action,
  abandoning the GitHub-native `jekyll-build-pages` flow.
- Flat nav ordered purely by `nav_order` — rejected: no language sections, less GitBook-like.

## Status / deferred

- **Verify on first deploy:** confirm `jekyll-build-pages@v1` resolves `just-the-docs` via
  `remote_theme` and that search-data generates. It's the supported GitHub Pages path, but
  unverified locally. Watch the next `Pages` workflow run.
- Branch `docs/gitbook-pages` not yet committed/pushed; no PR opened.

## Related (links)

- Issue #2 · `docs/_config.yml` · `.github/workflows/pages.yml`
