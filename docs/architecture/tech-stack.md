# Segovia — Technology Stack

Concrete dependency choices with one-line justifications and known sharp edges. All choices favor
mature, actively-maintained crates and reuse storage primitives rather than rebuilding them.

## Core compute & concurrency

| Crate | Role | Justification |
|---|---|---|
| `rayon` | Data parallelism over chunks | Work-stealing threads, no process-pool/pickle overhead — the core differentiator vs SpikeInterface. |
| `ndarray` | In-memory channels × samples arrays | NumPy analog; ergonomic views/slices for chunk tiles. |
| `rustfft` + `realfft` | FFT primitives (spectral ops, FFT-based filtering) | Mature, SIMD-accelerated (AVX/SSE/Neon); `realfft` for real signals. |
| `thiserror` | Library error types | House convention; clean error enums. |
| `serde` + `serde_json` | Config / metadata serialization | House convention. |

## Storage / IO (reuse — do not rebuild)

| Crate | Role | Justification |
|---|---|---|
| `zarrs` | Zarr V3/V2 read (and write) | Mature, actively maintained, Zarr 3.1 conformant; the strongest dependency. **Reuse, do not reimplement.** |
| `memmap2` | Memory-mapped SpikeGLX `.bin` access | Bounded-memory streaming of flat binary without full load. |
| (custom) SpikeGLX `.meta` parser | Parse SpikeGLX sidecar metadata | Small, well-specified format; no suitable crate — write a thin parser. |
| `hdf5-metno` | HDF5-backed NWB read (deferred) | The original `hdf5` crate is abandoned (last release 0.8.1, Nov 2021); `hdf5-metno` is the live community fork. **Deferred to post-MVP.** |
| `arrow-rs` | Arrow interop for zero-copy exchange | Columnar exchange with Python/Polars when Arrow is the better bridge than raw NumPy. |

## Python bridge

| Crate / tool | Role | Justification |
|---|---|---|
| `pyo3` | Rust ↔ Python FFI | Industry standard; `Python::allow_threads` releases the GIL around heavy Rust work (essential — see `adr/0004`). |
| `rust-numpy` | Zero-copy NumPy views | Return results without copying; `PyReadonlyArray` for inputs. |
| `maturin` | Build + wheel packaging | Proven on MaskOps; builds cdylib + installs editable package; produces PyPI wheels. |
| `pyo3-polars` | (optional) Polars plugin path | Only if the optional Polars-plugin packaging (Candidate B) is pursued. Pin in sync with `pyo3`. |

## Sharp edges (flagged from the dossier — do not get surprised)

- **`std::simd` (portable-simd) is nightly-only** with no stabilization timeline. The workload is
  IO-bound, so hand-SIMD is rarely needed; if it is, use `wide` or `pulp` on stable. **Do not
  architect around portable-simd.**
- **HDF5 is the ecosystem wound.** `hdf5-metno` wraps the HDF5 C library — cross-platform linking
  (especially Windows wheels) is a known headache, and there is **no schema-aware NWB reader in
  Rust** (you read raw HDF5 and interpret NWB structure yourself). **Lead with SpikeGLX + Zarr;
  treat HDF5-NWB as a later, painful add.**
- **`pyo3` / `pyo3-polars` version coupling** (MaskOps lesson): never bump `pyo3` independently of
  `pyo3-polars`. Pin both together.
- **Windows wheel + extension-load issues** (MaskOps precedent): a real dependency caused an
  Ubuntu + Python 3.12 `dlopen` exclusion from the test matrix. Expect platform-specific
  extension-load surprises; test the wheel matrix early.
- **`ndarray` has no built-in lazy evaluation / out-of-core** — chunking and the lazy graph are
  Segovia's responsibility (see candidate architectures).

## Prior art to consult before building (not dependencies)

- `direct-neural-biasing` — Rust + PyO3 real-time neuro DSP; closest existing project. Read its
  source to confirm it stays in the real-time/online niche, not Segovia's offline out-of-core one.
- `zarrs_tools`, `nwbview`, `nigui` — storage/viz tools confirming what NOT to rebuild.

## Dev / CI

| Tool | Role |
|---|---|
| `pytest` | Python integration tests against fixtures + sample datasets. |
| `cargo test` + `criterion` | Rust unit tests + benchmarks (the SC1 gate lives here). |
| GitHub Actions | Multi-platform wheel build matrix (Windows-first), benchmark runs. |
| `.venv` + `maturin develop --release` | Local dev loop (re-run after any Rust change). |
