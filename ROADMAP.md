# Segovia — Roadmap

This file is the **single source of truth for version and scope.** The detailed, milestone-level
architecture plan lives in [`docs/architecture/roadmap.md`](docs/architecture/roadmap.md); this file
is the authoritative summary that release and version decisions are made against.

## Current status

- **Version:** `0.3.0` — three chunked, memory-bounded readers (SpikeGLX + Zarr + mtscomp `.cbin`) behind a shared `ChunkSource` trait, live on crates.io + PyPI.
- **Phase:** M0–2 (learn + read + de-risk tooling) — nearly complete; the day-1 maturin/zero-copy
  NumPy toolchain spike is done, the **chunked, memory-bounded SpikeGLX `.meta`/`.bin` reader**
  (`segovia.SpikeGlxReader`) landed the phase deliverable, a second **chunked, memory-bounded
  Zarr reader** (`segovia.ZarrReader`, `zarrs` crate, gzip/zstd/blosc) streams the same
  `(samples, channels)` `int16` chunks behind the shared `ChunkSource` trait (ADR 0011), and a third
  **native mtscomp `.cbin` reader** (`segovia.CbinReader`, `flate2` zlib + `i16` delta reversal,
  positioned per-chunk reads) reads IBL's compressed corpus directly (ADR 0012). All three are
  validated byte-identical against the real `Noise4Sam_g0` recording. The **realistic-scale
  bounded-memory run is done**: a real 46-minute, 385-channel IBL LF recording (1.6 GB `.cbin`,
  5.32 GB decompressed) streamed end to end at ~250 MB/s in **186 MB peak RSS** — far under the 2 GB
  bound, and file-size-independent so the bound carries to a full AP-band hour. Remaining in M0–2:
  reading `direct-neural-biasing` source to confirm the niche is still open.

## The one gate that decides everything (SC1)

On a real 1-hour Neuropixels recording, the Rust **bandpass + CMR + whiten** chain must run in
**< 2 GB peak memory** AND be **faster than the equivalent `spikeinterface(n_jobs=N)`** call on
Windows/macOS. If this cannot be shown by M4, the project premise is invalid and scope/approach is
reconsidered. Build nothing heavy before this is answered.

## Milestones

| Phase | Months | Focus | Exit criterion |
|---|---|---|---|
| Learn + de-risk | 0–2 | Domain + SpikeGLX/Zarr readers + day-1 maturin wheel spike | Bounded-memory chunk reader |
| **Prove the win** | **2–4** | MVP chain + benchmark (the SC1 gate) | **SC1 passes** |
| Real engine | 4–7 | Lazy graph + Python API | `pip install` + 10-line demo |
| Breadth | 7–10 | More ops + correctness | Op library + tests |
| Ship | 10–12 | SpikeInterface backend + release | Public `v0.x` + benchmarks |
| Future vertical (gated) | 12+ | single-cell / leukemia (interop) | Post-ship; 3 entry gates |

The **single-cell / leukemia vertical is deferred and gated** — not in the 12-month scope. It would
only begin post-ship, via interop on `scverse/anndata-rs`, and only if it clears its entry gates. See
[`docs/future/leukemia-direction.md`](docs/future/leukemia-direction.md).

## Versioning

Semantic Versioning. A `v*` tag is a deliberate release event requiring explicit maintainer approval —
never a side effect of a commit.
