# ADR 0007 — Packaging: standalone crate + PyO3 wheel (Polars plugin optional)

**Status:** Accepted — confirmed by maintainer (OD1 resolved)

## Context

Three packaging shapes are viable (see `candidate-architectures.md`):
- **A** standalone Rust crate + thin PyO3 wheel,
- **B** a Polars expression plugin (the exact pattern shipped on MaskOps),
- both.

The maintainer has proven experience shipping the Polars-plugin pattern, which lowers tooling risk
for B. But Polars is a columnar/relational engine; dense (channels × samples) signal kernels with
cross-chunk filter state fit it awkwardly, and "Polars is fast" dilutes Segovia's differentiation.

## Decision (proposed)

Ship Segovia as a **standalone `segovia` crate + thin `segovia-py` PyO3 wheel** (Candidate A).
Treat a **Polars plugin as optional, additive distribution** later — not the core engine.

## Consequences

- Full control over multichannel/cross-chunk semantics and memory bounds; reusable crates.io artifact.
- Clean SpikeInterface integration as a separate adapter (FR11) rather than via Polars.
- Forgoes Polars' free lazy engine / out-of-core — Segovia owns chunking/scheduling (accepted; see ADR 0006).
- If the maintainer prefers to lead with the familiar Polars-plugin path for speed, that is the
  main alternative — **this ADR is the one to confirm interactively before M4.**

## Status note

Confirmed by the maintainer: **A — standalone `segovia` crate + thin PyO3 wheel.** A Polars
plugin remains an optional, additive distribution layer to revisit post-MVP if there's demand.
