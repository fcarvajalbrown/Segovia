# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial repository scaffold: Rust crate + PyO3/maturin Python packaging, architecture docs,
  AGPL-3.0-or-later license, CI, and contributor/community files.
- Day-1 zero-copy bridge spike: `segovia.zeros(channels, samples)` returns an `int16` NumPy array
  backed by Rust-owned memory (no copy), plus `segovia.__version__`.

[Unreleased]: https://github.com/fcarvajalbrown/Segovia/commits/main
