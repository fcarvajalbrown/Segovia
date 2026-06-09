# ADR 0009 — Single-source versioning & automated dual-registry release

**Status:** Accepted — implemented and proven on v0.1.0 (2026-06-09)

## Context

Segovia ships as a Rust crate (crates.io) and a PyO3 wheel (PyPI) from one repository. A version
number therefore appears in several places — `Cargo.toml`, `pyproject.toml`, the git tag,
`CITATION.cff`, `ROADMAP.md`, `__version__`. On a prior project (MaskOps) a hand-maintained
`pyproject` version silently drifted from `Cargo.toml`. The release itself (bump → changelog → tag →
publish to both registries) was also manual and error-prone (the first crate publish was done by hand).

## Decision

Make `Cargo.toml` `version` the single machine source of truth and derive everything else:

- `pyproject.toml` carries no static version (`dynamic = ["version"]`); `maturin` reads the version
  from `Cargo.toml`, so the wheel/PyPI version cannot diverge from the crate.
- `__version__` comes from `env!("CARGO_PKG_VERSION")`.
- `cargo-release` (config in `release.toml`) bumps `Cargo.toml`, rewrites the doc surfaces
  (`CHANGELOG.md`, `CITATION.cff`, `ROADMAP.md`) via `pre-release-replacements`, commits, tags
  `vX.Y.Z`, and pushes — with `publish = false` (it does not publish).
- Publishing rides on a deliberate GitHub release: `release.yml` triggers on release-published,
  asserts the tag equals the `Cargo.toml` version (`verify-version`, which fails the publish on
  mismatch), then runs `cargo publish` (crates.io, token secret) + maturin wheels/sdist → PyPI via
  Trusted Publishing (OIDC, no stored token).
- Wheels are built once as abi3 (`pyo3` `abi3-py38`): a single `cp38-abi3` wheel per platform spans
  Python ≥ 3.8.

## Consequences

- The MaskOps dual-version drift is structurally impossible: only one number exists; the rest derive.
- A release is one command (`cargo release <level> --execute`) plus a GitHub release; both registries
  publish from the same number, CI-verified.
- crates.io tokens must carry **publish-update** scope — a publish-new-only token 403s on an existing
  crate (observed on the first v0.1.0 attempt; rotating the token fixed it).
- A local `/release` skill and a pre-tag version-sync guard hook mirror the CI checks before pushing.
- Packaging *shape* remains governed by ADR 0007; this ADR governs the version flow and release pipeline.
