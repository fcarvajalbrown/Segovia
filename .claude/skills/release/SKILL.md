---
name: release
description: Cut a deliberate Segovia release — squash-merge the approved roadmap PR, bump the version with cargo-release, tag, and publish to crates.io + PyPI. Invoke only when the maintainer has approved shipping a roadmap PR as a release.
disable-model-invocation: true
allowed-tools: Bash(git *), Bash(cargo *), Bash(gh *), Bash(python *), Read, Edit
---

# Release Segovia

Cut a deliberate release. Versioning is single-source: `Cargo.toml` `version` is the only number;
the wheel, `__version__`, and the tag all derive from it. See CLAUDE.md → *Versioning & release
mechanics* for the full guarantee.

## Preconditions — verify first, never skip
1. The roadmap PR is open and **every GitHub Actions check is green**: `gh pr checks <n>`. Never
   release on red CI.
2. Decide the bump from the squash-merge commit type: `feat` → `minor`; `fix`/`perf`/`refactor` →
   `patch`; breaking → `minor` while `< 1.0`.

## Steps
1. **Squash-merge** the approved PR with a conventional-commit subject (it drives the bump):
   `gh pr merge <n> --squash --delete-branch --subject "feat(scope): ..."`.
2. `git checkout main` then `git pull --ff-only`. Confirm the squash commit is on `main` —
   **never tag code that is not on `main`**.
3. **Pre-flight version-sync check** (must pass or abort):
   `python "${CLAUDE_SKILL_DIR}/scripts/version_sync_check.py"`.
4. **Dry-run**: `cargo release <level>` (no `--execute`). Review the printed diff.
5. **Execute**: `cargo release <level> --execute --no-confirm`. Bumps `Cargo.toml`, rewrites
   `CHANGELOG.md`/`CITATION.cff`/`ROADMAP.md`, commits `chore(release): vX.Y.Z`, tags `vX.Y.Z`,
   and pushes.
6. **Publish**: write the body from [release-notes-template.md](release-notes-template.md), then
   `gh release create vX.Y.Z --discussion-category "Announcements" --title "vX.Y.Z" --notes "<body>"`.
   This fires `release.yml` → crates.io + PyPI.
7. **Watch**: `gh run watch <run-id> --exit-status`. If `publish crate (crates.io)` fails with
   `403`, the `CARGO_REGISTRY_TOKEN` secret needs a **publish-update**-scoped token; once the
   maintainer fixes it, `gh run rerun <run-id> --failed`.
8. **Verify** both registries carry the new version, then report honestly.

## Release-notes prose
Always use the Milestone template in [release-notes-template.md](release-notes-template.md). The
**Still open** section is mandatory — never imply more is done than is.
