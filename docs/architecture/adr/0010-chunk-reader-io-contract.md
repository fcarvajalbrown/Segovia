# ADR 0010 — Chunk-reader I/O contract

**Status:** Accepted — implemented in the v0.1.0 SpikeGLX reader

## Context

The M0–2 deliverable is a chunked, memory-bounded reader that streams a multi-GB recording to Python.
The reader's output shape, memory model, and how it reconciles on-disk reality with the sidecar
metadata are decisions the rest of the engine (the compute chain, future Zarr/NWB readers) builds on,
so they are fixed once, here, rather than per format.

## Decision

- **Memory model:** memory-map the flat `int16` `.bin` (`memmap2`); OS paging keeps resident memory
  bounded regardless of file size. Each chunk is copied into one owned `(rows, channels)` array — at
  most one chunk resident on the Rust/Python side at a time. The GIL is released (`Python::detach`)
  around the copy.
- **Chunk orientation:** chunks are `(samples, channels)` — matching the on-disk SpikeGLX layout and
  SpikeInterface's `get_traces()` convention, so no transpose is needed and outputs are
  ecosystem-compatible.
- **Authority of the bytes:** the actual `.bin` size is authoritative. `n_samples` is derived from
  the real file size (`bytes / (channels · 2)`) and validated for frame alignment; the meta's declared
  `fileSizeBytes` is advisory — a stale or truncated value is tolerated and surfaced via
  `declared_file_size_bytes`, never a hard error. (Real test corpora ship a truncated `.bin` alongside
  the original full-length meta.)
- **Core stays PyO3-free:** the reader and metadata types live in pure Rust (`ephys::`); the PyO3
  wrapper is a thin layer, preserving ADR 0008's domain-neutral core seam.

## Consequences

- Bounded application memory regardless of file size (the SC1 premise), proven byte-for-byte against a
  real `Noise4Sam_g0` recording.
- `(samples, channels)` becomes the engine-wide chunk contract; future readers (Zarr, NWB) and the
  compute chain conform to it.
- Trusting actual bytes over the meta means a corrupt/over-long file is read up to its frame-aligned
  length rather than rejected — the frame-alignment check, not the declared size, is the guard.
