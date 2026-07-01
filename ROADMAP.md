# Segovia — Roadmap

This file is the **single source of truth for version and scope.** The detailed, milestone-level
architecture plan lives in [`docs/architecture/roadmap.md`](docs/architecture/roadmap.md); this file
is the authoritative summary that release and version decisions are made against.

## Current status

- **Version:** `0.4.0` — three chunked, memory-bounded readers (SpikeGLX + Zarr + mtscomp `.cbin`) behind a shared `ChunkSource` trait, plus the streaming **bandpass → CMR → whiten** preprocessing chain (`reader.preprocess(...)`), live on crates.io + PyPI.
- **Phase:** M2–4 (prove the win) **measured and resolved** — the MVP **bandpass → CMR → whiten**
  chain (Candidate D: eager Rayon over time-chunks, GIL released) is built, validated byte-faithful
  against a whole-signal scipy reference, and benchmarked on a real 1-hour IBL Neuropixels AP
  recording against `spikeinterface(n_jobs=N)`. **SC1 outcome: the memory criterion passes
  decisively; the speed criterion does not and was dropped after a dedicated optimisation round** —
  see *the one gate* below and ADR 0013.
- **M0–2 readers (done):** the day-1 maturin/zero-copy NumPy toolchain spike, the **SpikeGLX**
  reader (`segovia.SpikeGlxReader`), the **Zarr** reader (`segovia.ZarrReader`, `zarrs`,
  gzip/zstd/blosc, ADR 0011), and the **native mtscomp `.cbin`** reader (`segovia.CbinReader`,
  `flate2` zlib + `i16` delta reversal, positioned per-chunk reads, ADR 0012) — all three validated
  byte-identical against the real `Noise4Sam_g0` recording, all streaming the same
  `(samples, channels)` `int16` chunks. The realistic-scale bounded-memory run is done: a real
  46-minute, 385-channel IBL LF recording (1.6 GB `.cbin`, 5.32 GB decompressed) streamed end to end
  in **186 MB peak RSS** — far under the 2 GB bound, file-size-independent. Remaining in M0–2:
  reading `direct-neural-biasing` source to confirm the niche is still open.

## The one gate that decided everything (SC1) — resolved

SC1 asked: on a real 1-hour Neuropixels recording, does the Rust **bandpass + CMR + whiten** chain
run in **< 2 GB peak memory** AND **faster than the equivalent `spikeinterface(n_jobs=N)`** on
Windows? Measured on a real IBL AP recording (first 10 min, 385 ch, 30 kHz; 8-core / 7.8 GB-RAM
Windows; matched parallelism):

- **Memory — decisive PASS.** Segovia holds **0.99 GB**, file-size-independent, vs SpikeInterface's
  1.75 GB (thread pool) / 2.84 GB (process pool); the SI process pool *breaches* the 2 GB bound and
  OOMs at `n_jobs = 8`.
- **Speed — not met, and judged not achievable on this workload.** Segovia is ~0.84× SI's default
  thread pool. The deferred optimisation round was run and profiled: serial `.cbin` decode is 33 % of
  the wall, but parallelising it yields no net gain because decode is **memory-bandwidth bound**
  (1.66× across 16 cores) — a ceiling SI shares — and SI 0.102 already uses a *thread* pool (shared
  memory, no pickle) with faster C/MKL kernels. The original premise (shared-memory threading beating
  SI's worker pools) does not hold against that default.

**Resolution:** SC1 is kept as the **bounded-memory gate** it decisively passes; the project's
differentiation is **bounded-memory streaming** (true regardless of file size), and the "faster than
SpikeInterface" claim is dropped. Full measurement, the optimisation round, and the reasoning are in
ADR 0013. Growth into a lazy graph / op library continues on that honest footing.

## Milestones

| Phase | Months | Focus | Exit criterion |
|---|---|---|---|
| Learn + de-risk | 0–2 | Domain + SpikeGLX/Zarr readers + day-1 maturin wheel spike | Bounded-memory chunk reader |
| **Prove the win** | **2–4** | MVP chain + benchmark (the SC1 gate) | **Memory gate passed; speed reframed (ADR 0013)** |
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
