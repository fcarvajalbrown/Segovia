# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
