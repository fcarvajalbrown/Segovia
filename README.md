# Segovia

A fast, memory-bounded Rust engine for electrophysiology signal processing — Neuropixels-scale, callable from Python.

> ⚠️ **Temporary README** — placeholder content to be refined for SEO and completeness before the first public push.

## What it is

Segovia is a lazy-evaluated, chunked, concurrent Rust compute engine for massive multi-channel
electrophysiology time-series (Neuropixels-scale: 30 kHz × thousands of channels), exposed to Python
via PyO3 and built to integrate with the SpikeInterface ecosystem over SpikeGLX, Zarr, and NWB.

The differentiating goal is **bounded-memory, out-of-core streaming preprocessing** with
GIL-released shared-memory threading (Rayon) — true threads instead of SpikeInterface's
process-pool / pickle / per-process-copy model.

The name honors **Claudio Segovia**, a friend, and evokes the Aqueduct of Segovia — a continuous
stream carried across a row of segmented stone arches, the metaphor for this engine's chunked,
span-by-span streaming model.

## Status

Early architecture stage. No public release yet. See the architecture docs and roadmap:

- `docs/architecture/ARD.md` — Architecture Requirements Document
- `docs/architecture/candidate-architectures.md` — candidate architectures and recommendation
- `docs/architecture/tech-stack.md` — concrete crate choices
- `docs/architecture/roadmap.md` — 12-month milestones and the benchmark go/no-go gate
- `docs/architecture/adr/` — Architecture Decision Records

## Design at a glance

- **CPU, not GPU** — the workload is IO/memory-bound (~22 MB/s per probe).
- **Reuse storage crates** — `zarrs`, `hdf5-metno`, `arrow-rs`; SpikeGLX + Zarr first.
- **Prove the win early** — the full chain (bandpass + CMR + whiten) must run in < 2 GB memory and
  beat `spikeinterface(n_jobs=N)` on Windows/macOS — the make-or-break benchmark gate.

## License

To be finalized — dual MIT OR Apache-2.0 (Rust ecosystem convention) is planned.
