# ADR 0003 — Reuse storage primitives; do not rebuild

**Status:** Accepted

## Context

A from-scratch Rust Zarr or HDF5/NWB reader is tempting but would duplicate mature, actively
maintained work. The prior-art sweep confirmed `zarrs` (Zarr V3, 75+ releases) and `hdf5-metno`
(live HDF5 fork) already exist, and `nwbview` proves Rust HDF5/NWB reading works.

## Decision

Build **on** existing storage crates (`zarrs`, `hdf5-metno`, `arrow-rs`, `memmap2`). The only
storage code Segovia writes itself is a thin **SpikeGLX `.meta` parser** (small, well-specified,
no suitable crate).

## Consequences

- The differentiating layer is **lazy/chunked concurrent COMPUTE**, not storage — focus effort there.
- Scope is achievable solo in 12 months because the IO layer is mostly assembled, not invented.
- Dependency risk: `hdf5-metno` is a volunteer fork wrapping the HDF5 C library; cross-platform
  linking pain is inherited (see `tech-stack.md`). Mitigated by deferring HDF5-NWB (see ADR 0005).
- `zarrs` becomes a load-bearing dependency; track its releases.
