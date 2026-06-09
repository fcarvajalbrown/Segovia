# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
