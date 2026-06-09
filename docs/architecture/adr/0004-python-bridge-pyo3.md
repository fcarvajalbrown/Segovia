# ADR 0004 — Python bridge: PyO3 + GIL release, zero-copy

**Status:** Accepted

## Context

Adoption requires that neuroscientists stay in Python (goal G3). The incumbent (SpikeInterface)
escapes the GIL via **process pools** — which on Windows/macOS pay `spawn` cost: fresh
interpreters, pickle serialization, and full per-process memory copies. This is precisely the gap
Segovia exploits, and the maintainer's machine is Windows.

## Decision

Bridge to Python with **`pyo3` + `maturin`**, returning **zero-copy** NumPy via `rust-numpy`
(and Arrow via `arrow-rs` where better). Release the GIL with **`Python::allow_threads`** around
all heavy Rust compute, parallelizing with `rayon` over shared memory.

## Consequences

- **This is the core differentiator** (G2): true shared-memory threads with the GIL released
  sidestep pickling, process-spawn cost, and per-process RAM duplication — a concrete, measurable
  win on Windows/macOS, validated at the M2–4 gate.
- Proven shippable pattern: Polars, tokenizers, pydantic-core, cryptography, MaskOps all ship
  Rust-in-Python this way.
- **Constraint:** `pyo3` and `pyo3-polars` (if used) must be version-locked together (MaskOps lesson).
- **Constraint:** zero-copy requires careful lifetime/ownership at the FFI boundary — the only
  place audited `unsafe` is permitted (NFR5).
- PyO3 0.28+ supports free-threaded Python 3.14, a future tailwind.
