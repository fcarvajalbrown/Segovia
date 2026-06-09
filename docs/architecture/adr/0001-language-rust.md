# ADR 0001 — Language: Rust

**Status:** Accepted

## Context

The engine needs bounded-memory, concurrent streaming compute over terabyte-scale multichannel
signals, shipped into a Python ecosystem. Realistic alternatives: C++, Zig, Julia, and
Cython/Numba. The maintainer has strong Rust skills and has already shipped a Rust+PyO3+Polars
package to PyPI (MaskOps).

## Decision

Build the engine in **Rust**.

## Consequences

- **Vs C++:** same performance ceiling; Rust wins on memory/thread safety (chunk-scheduler data
  races are exactly Rust's sweet spot for a concurrent engine) and on Cargo/maturin build-and-ship
  for cross-platform wheels.
- **Vs Julia:** Julia's GC pauses are awkward for bounded-memory streaming, and its Python-interop
  / startup-latency story is weaker for a pip-installable library.
- **Vs Zig:** immature ecosystem, no PyO3 equivalent, no 1.0 — higher risk for a year-long bet.
- **Vs Cython/Numba:** these inherit the GIL/multiprocessing model and won't give clean
  shared-memory concurrency. They are the *baseline to beat*, not the foundation.
- **Honest caveat:** Rust is *fine, leaning right* — not transformative. The win is engineering
  quality (true threads, low overhead, safety), not a capability Python fundamentally lacks. The
  Rust numerical ecosystem is mature *enough* (`rustfft`, `ndarray`, `zarrs`, `rayon`,
  `rust-numpy`), with sharp edges documented in `tech-stack.md`.
- Maintainer's existing Rust+PyO3+maturin experience makes this the lowest-risk choice.
