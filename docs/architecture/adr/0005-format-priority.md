# ADR 0005 — Format priority: SpikeGLX + Zarr first, HDF5-NWB deferred

**Status:** Accepted

## Context

The ecosystem has many formats (the "format zoo"). NWB and Zarr are the standardized integration
targets, but NWB is primarily **HDF5**-backed, and HDF5 in Rust is painful: the original `hdf5`
crate is abandoned, `hdf5-metno` wraps the C library with cross-platform linking headaches, and
there is **no schema-aware NWB reader in Rust**. SpikeGLX is a flat binary (`.bin` + `.meta`).

## Decision

Prioritize formats in this order for v1:
1. **SpikeGLX** (`.bin`/`.meta`) — flat binary, easiest, memory-mappable.
2. **Zarr** — via mature `zarrs`; Zarr-backed NWB included here.
3. **HDF5-backed NWB** — **deferred** to post-MVP (M7–10+), via `hdf5-metno`, raw HDF5 only.

## Consequences

- Fastest path to the M2–4 benchmark gate (SpikeGLX needs no heavy dependency).
- Avoids front-loading the HDF5/wheel-linking risk; it lands later when the engine is proven.
- Some NWB users are unreachable until HDF5-NWB ships — accepted trade-off for v1.
- The day-1 tooling spike (roadmap M0–2) should still touch HDF5 once to size the pain.
