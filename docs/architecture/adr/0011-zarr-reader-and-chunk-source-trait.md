# ADR 0011 — Zarr reader and the `ChunkSource` trait

**Status:** Accepted — implemented in the M0–2 Zarr reader

## Context

ADR 0010 fixed the chunk-reader I/O contract on the SpikeGLX reader: stream a recording as owned
`(samples, channels)` `int16` chunks, one resident at a time, GIL released around the copy. The
M0–2 deliverable also calls for a second reader over the Zarr format so the engine is not bound to a
single on-disk layout. Adding it raises two decisions: which Rust Zarr implementation to depend on,
and whether the two readers should share an explicit contract or stay independent.

Zarr stores in the ephys ecosystem are produced by `zarr-python` (default compressor: **zstd**) and
by SpikeInterface's `ZarrRecordingExtractor` (default compressor: **blosc**). A reader that only
handled uncompressed or gzip stores would fail on the stores practitioners actually have.

## Decision

- **Depend on `zarrs`** (0.23) for Zarr v2/v3 reading. The reader opens a 2-D `int16` array node
  (default `/traces`), treats dim 0 as samples and dim 1 as channels per ADR 0010, derives
  `n_samples`/`n_channels` from the array shape, and reads `sample_rate` from the array's
  `sampling_frequency` attribute (advisory — defaults to `0.0` when absent, mirroring ADR 0010's
  "metadata is advisory" stance). Each chunk is one `retrieve_array_subset::<Vec<i16>>` over a
  `(chunk_samples, channels)` region, copied into one owned `Array2<i16>`; at most one chunk is
  resident, so application memory stays bounded regardless of store size. A non-`int16` or non-2-D
  array is a hard error.
- **Codec features: `gzip` + `zstd` + `blosc`** (with `default-features = false`, plus
  `filesystem`). This reads both ecosystem producers' defaults. `zstd-sys` and `blosc-src` build C
  from source via `cc`; this is the cross-platform-wheel build risk flagged in CLAUDE.md (the
  MaskOps HDF5/`dlopen` precedent) and is accepted deliberately in exchange for real-world
  compatibility. Codecs that link additional C libraries beyond these are **not** enabled.
- **Extract a `ChunkSource` trait** into the domain-neutral `core` module:
  `n_channels` / `n_samples` / `sample_rate` / `chunks`, with an associated `Chunks:
  Iterator<Item = Array2<i16>>`. Both `SpikeGlxReader` and `ZarrReader` implement it; the trait is
  the engine-wide producer contract the future compute chain will consume generically, so the chain
  is written once against `ChunkSource` rather than per reader. Iterators own their backing handle
  (an `Arc` of the mmap or the Zarr array), so the associated type needs no borrow of the reader and
  no GAT.
- **Core stays PyO3-free** (ADR 0008): `ChunkSource`, `ZarrReader`, and `ZarrChunkIter` are pure
  Rust; the `PyZarrReader` / `PyZarrChunks` wrappers are a thin layer that releases the GIL
  (`Python::detach`) around each chunk retrieval, exactly as the SpikeGLX wrapper does.

## Consequences

- `(samples, channels)` `int16` is now produced by two independent on-disk formats behind one trait,
  validated on real data: the `ZarrReader` reads byte-identical chunks to the `SpikeGlxReader` for
  the real `Noise4Sam_g0` recording re-encoded through gzip, zstd, and blosc.
- The C-codec dependencies (`zstd-sys`, `blosc-src`) are a watch item for the maturin wheel matrix on
  Windows/macOS/Linux; if a runner lacks the C toolchain (e.g. `cmake`) the wheel build, not the
  Rust tests, is where it surfaces.
- The compute chain (M2–4) targets `ChunkSource`, not a concrete reader, so it gains Zarr input for
  free and any future reader (NWB) only needs to implement the trait.
- v2 stores are readable via `zarrs`, but validation so far is against zarr-python v3 stores; the
  exact SpikeInterface store layout (array node naming, segment grouping) is not yet exercised
  end-to-end and remains open.
