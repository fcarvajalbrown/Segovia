# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.1] - 2026-07-03

### Changed
- **Conservative OOM safety cap on the `batch == 0` auto default.** `reader.preprocess(...)` with the
  default (auto) batch now caps parallel width at `min(logical_threads, 4)` instead of one in-flight
  slab per logical thread. Resident memory is `~0.17 GB × batch + ~0.5 GB`, so the previous unbounded
  auto default projected to ~3.3 GB on a 16-thread box and ~6 GB+ on larger servers — a runaway-memory
  footgun that could OOM low-RAM hosts. The cap bounds the default to ~1.2 GB on any machine. Callers
  passing an explicit `batch >= 1` are unaffected. This is an OOM safety guard, not a throughput-optimum
  claim; the optimal-default question stays open. See ADR 0018.

## [0.4.0] - 2026-07-01

### Added
- **IFC simulator leg** (`segovia.SyntheticIfcReader`): a second `sim` vertical modelling impedance
  flow cytometry as **bipolar-Gaussian pulses** (a positive then negative lobe as a particle transits
  the differential electrodes) from `n_populations` distinct particle populations arriving as a
  homogeneous Poisson process, with per-channel gains, additive Gaussian noise, and `i16` output. It
  implements the same `ChunkSource` contract and streams through the unchanged `preprocess(...)` chain,
  demonstrating the engine's dual-domain generality with no wet-lab dependency. Same pure-Rust
  dependency-free RNG as the ephys leg, so output is chunk-size-independent, bounded-memory, and
  bit-identical across platforms; `ground_truth()` returns `(sample, population, amplitude)`. IFC-
  appropriate defaults (100 kHz, 2 channels, µs-scale pulses). Tested by `src/sim/ifc.rs` unit tests
  and `tests/test_ifc_simulator.py`. See ADR 0016.
- Streaming **bandpass → common-median-reference → whiten** preprocessing chain, exposed as
  `reader.preprocess(sos, chunk_samples, margin, calib_samples, ...)` on all three readers and
  yielding `float32 (samples, channels)` chunks with the GIL released per batch (`Preprocessor`
  iterator). Architecture is **Candidate D**: eager Rayon over time-chunks (the parallel unit is the
  time chunk, since CMR's per-sample median and whitening's `W·x` both mix all channels). The
  bandpass is a faithful reimplementation of scipy `sosfiltfilt` (scipy-designed SOS passed in;
  `sosfilt_zi` steady-state init, odd padding, forward-backward) applied over a real-neighbour
  **margin overlap**, so chunked output is reference-equal to whole-signal scipy with no cross-chunk
  boundary artifact; CMR is the per-sample median across channels (numpy-median semantics); whitening
  is ZCA from a bounded calibration subset (`W = V·diag(1/√(λ+ε))·Vᵀ` via a pure-Rust `nalgebra`
  symmetric eigendecomposition), applied with the `gemm` crate. CMR and the whitening GEMM compute in
  `float32`; the filter stays `float64`. Resident memory is `batch × (chunk + 2·margin) × channels`,
  independent of file size. Validated to the `float32` floor against a whole-signal scipy reference
  (`tests/test_preprocess.py`). See ADR 0013.

### Changed
- **SC1 gate resolved as a bounded-memory gate.** On a real 1-hour IBL Neuropixels AP recording the
  chain holds **0.99 GB peak, file-size-independent** (vs SpikeInterface 1.75 GB thread / 2.84 GB
  process, which breaches 2 GB and OOMs at `n_jobs = 8`) — a decisive memory win. The speed criterion
  ("faster than SpikeInterface") was measured (~0.84× SI's thread pool) and, after a profiled
  optimisation round, judged not achievable on this **memory-bandwidth-bound** workload against SI's
  default thread pool + C/MKL kernels; it is dropped. Segovia's stated differentiation is now
  bounded-memory streaming. See ADR 0013 and `ROADMAP.md`.

## [0.3.0] - 2026-06-09

### Added
- Chunked, memory-bounded mtscomp `.cbin` reader (`segovia.CbinReader`): opens an IBL/SpikeGLX
  `.cbin` + `.ch` file pair and streams it in the same `(samples, channels)` `int16` chunks as the
  SpikeGLX and Zarr readers behind the `ChunkSource` trait. Reads only each chunk's compressed bytes
  via positioned reads (so peak memory is independent of file size, not a whole-file mmap); each
  ~1-second mtscomp native chunk is `zlib`-inflated (`flate2`) and its time-delta reversed by an
  `i16` wrapping cumulative sum; native chunks are re-chunked to the caller's `chunk_samples` with
  resident memory bounded by `max(native_chunk, chunk_samples)` rows, and the GIL released during
  each chunk. Honors `chunk_order` `F`/`C`; rejects spatial-diff, non-`int16`, and non-`zlib` files
  with typed errors. Validated byte-identical against the real `Noise4Sam_g0` recording round-tripped
  through the `mtscomp` compressor, and streamed a real 46-minute 385-channel IBL LF recording
  (5.32 GB decompressed) in 186 MB peak RSS. See ADR 0012.

## [0.2.0] - 2026-06-09

### Added
- Chunked, memory-bounded Zarr reader (`segovia.ZarrReader`): opens a 2-D `int16` array node
  (default `/traces`) in a Zarr v2/v3 store via the `zarrs` crate and streams it in the same
  `(samples, channels)` `int16` chunks as the SpikeGLX reader (ADR 0010), retrieving one
  `(chunk_samples, channels)` region at a time so application memory stays bounded regardless of
  store size, with the GIL released during each chunk copy. `sample_rate` is read from the array's
  `sampling_frequency` attribute. Reads gzip-, zstd-, and blosc-compressed stores — covering both
  `zarr-python`'s and SpikeInterface's default compressors. Validated against the real `Noise4Sam_g0`
  recording: the Zarr reader yields byte-identical chunks to `SpikeGlxReader` for the recording
  re-encoded through all three codecs.
- `ChunkSource` trait (`n_channels` / `n_samples` / `sample_rate` / `chunks`) shared by
  `SpikeGlxReader` and `ZarrReader` as the engine-wide chunk-producer contract. See ADR 0011.

## [0.1.0] - 2026-06-09

### Added
- Initial repository scaffold: Rust crate + PyO3/maturin Python packaging, architecture docs,
  AGPL-3.0-or-later license, CI, and contributor/community files.
- Day-1 zero-copy bridge spike: `segovia.zeros(channels, samples)` returns an `int16` NumPy array
  backed by Rust-owned memory (no copy), plus `segovia.__version__`.
- Chunked, memory-bounded SpikeGLX reader (`segovia.SpikeGlxReader`): parses the `.meta` sidecar
  (channel count, sample rate, stream type, declared file size, raw fields), memory-maps the `.bin`,
  and streams it in fixed-size `(samples, channels)` `int16` chunks via `reader.chunks(chunk_samples)`
  with the GIL released during each chunk copy. Sample count is derived from the **actual** `.bin`
  size and validated for frame alignment; a stale or truncated meta `fileSizeBytes` is tolerated and
  surfaced via the `declared_file_size_bytes` property. Validated byte-for-byte against the real
  `Noise4Sam_g0` Neuropixels recording from the NEO `ephy_testing_data` corpus.

[Unreleased]: https://github.com/fcarvajalbrown/Segovia/commits/main
