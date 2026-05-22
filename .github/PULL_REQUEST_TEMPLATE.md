## Summary

One or two sentences on *why* this change is needed.

## What changed

- Bullet list of the concrete edits, grouped by surface (spec docs / Rust crates / hooks / skills / CI).

## Related

- Closes #
- Refs #
- Spec doc(s) touched:

## Checklist

(Per [`CONTRIBUTING.md` §Pull Request Checklist](../CONTRIBUTING.md#pull-request-checklist).)

- [ ] Branch is up to date with `main`
- [ ] PR title follows commit style (`type(scope): subject` per Conventional Commits)
- [ ] Description explains *why* the change is needed (not just *what*)
- [ ] Related issues are linked (`Closes #123`)
- [ ] Documentation is updated where applicable
- [ ] **Bilingual EN/TH pair preserved** (if `docs/<lang>/*.<lang>.md` or root `FILENAME.md` ↔ `FILENAME.th.md` changed)
- [ ] **No secrets, credentials, or personal paths committed**
- [ ] CI is green (`cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, naming audit)
- [ ] If this changes the manifest schema, `bwoc check` still passes against the agent template
- [ ] If this is a policy-bearing change (CoC, SECURITY SLAs, governance), the policy was discussed in an issue first

## Risk and rollback

How would a reviewer roll this back if it breaks something? (Single commit revert / staged feature-flag / spec-only — no rollback needed.)
