# Segovia v0.4.0: a bounded-memory, near-real-time streaming preprocessor for Neuropixels-scale electrophysiology

**Felipe Carvajal Brown** — Independent Researcher — fcarvajalbrown@gmail.com — ORCID 0000-0002-8300-7587

Submission type: **Technical Release**

---

## Abstract

Segovia is an open-source Rust library (AGPL-3.0) with Python bindings for bounded-memory,
near-real-time preprocessing of high-density extracellular electrophysiology recordings. It
applies a bandpass-filter → common-median-reference → ZCA-whitening chain to Neuropixels-scale
(384–768 channels, 30 kHz) `i16` streams, one chunk at a time, releasing the Python Global
Interpreter Lock for each Rust computation. Peak memory is bounded by chunk size—not recording
length—and scales analytically as `batch × (chunk + 2×margin) × channels × sizeof(f32)`.
At a 300 ms chunk budget on a real International Brain Laboratory AP-band recording (385 channels,
mtscomp-compressed), Segovia achieves 100% real-time deadline adherence at 0.28 GB peak RSS.
The package ships three production readers (SpikeGLX, Zarr, mtscomp `.cbin`) and a built-in
streaming synthetic ephys simulator (`SyntheticEphysReader`). Segovia v0.4.0 is available at
https://github.com/fcarvajalbrown/Segovia, via `pip install segovia` (PyPI), and via
`cargo add segovia` (crates.io).

---

## Background

Neuropixels probes [@Jun2017] acquire 384 channels at 30 kHz (~22 MB/s per probe). Standard
preprocessing pipelines (SpikeInterface [@Buccino2020], MountainSort) run offline on completed
recordings optimized for batch throughput. Near-real-time applications—closed-loop stimulation,
online brain-machine interface decoding—require that each incoming chunk of samples be
preprocessed within its acquisition period (the real-time deadline) with peak memory independent
of recording length. Batch tools driven one chunk at a time re-read filter-margin neighbourhoods
on every call and do not bound memory analytically. Segovia was built to fill this gap.

## Implementation

### Core abstraction

Segovia's `ChunkSource` trait is an iterator over fixed-size `(samples × channels)` `i16`
buffers. Three production implementations are provided:

- `SpikeGlxReader` — memory-mapped SpikeGLX `.bin` + `.meta` (zero-copy).
- `ZarrReader` — chunked Zarr arrays (gzip, zstd, Blosc) via the `zarrs` crate.
- `CbinReader` — mtscomp-compressed IBL `.cbin`, per-chunk zlib decompression via `flate2`.

### Preprocessing chain

`preprocess(chunk_source, config)` applies a Rayon-parallelized chain: 5th-order Butterworth
bandpass filter (`sosfiltfilt`), common median reference, global ZCA whitening. The Python GIL
is released via PyO3's `allow_threads` for the Rust computation. Cross-chunk IIR filter state is
maintained deterministically regardless of thread count.

### Built-in simulator

`SyntheticEphysReader` emits arbitrarily long synthetic streams without writing to disk. Spike
templates use a Ricker temporal waveform with point-source spatial decay
(`V(r) = A × d_perp / r`); firing is Poisson per unit; noise is additive white Gaussian.
The PRNG (SplitMix64 + xoshiro256++) is platform-independent and chunk-size-independent.
`ground_truth()` returns `(sample, unit, channel)` tuples for downstream sorter evaluation.

## Results

Evaluation follows the replay-at-acquisition-rate methodology: data streamed at the true 30 kHz
rate with per-chunk compute latency measured by monotonic clock. Deadline adherence = fraction of
chunks with latency ≤ chunk period.

**Real IBL AP-band recording, full length** (385 ch, mtscomp-compressed, 55.8 min, 11,167 chunks,
300 ms budget — steady state):

| Engine | Mean | p99 | Max | Adherence | Peak RSS | Jitter |
|---|---|---|---|---|---|---|
| Segovia | 179.2 ms | 277.0 ms | **334.5 ms** | **99.7%** | **0.21 GB** | **38.6 ms** |
| SpikeInterface online | 205.3 ms | 355.0 ms | 932.0 ms | 94.7% | 0.41 GB | 60.5 ms |

At steady state Segovia leads on every axis; the decisive margins are peak memory (2×), maximum
latency (2.8×), and jitter. The per-chunk table below is the **cold-start first-60 s** window, where
SpikeInterface's warm-up cost is highest and the adherence gap is widest (100% vs 69.5% at 300 ms);
that gap narrows to the steady-state figures above over the full recording.

**Real IBL AP-band recording, cold-start first 60 s** (385 ch, mtscomp-compressed):

| Chunk | Engine | Mean latency | p99 | Adherence | Peak RSS |
|---|---|---|---|---|---|
| 100 ms | Segovia | 92.9 ms | 122.0 ms | **74.2%** | **0.21 GB** |
| 100 ms | SpikeInterface online | 112.0 ms | 275.2 ms | 64.2% | 0.46 GB |
| 300 ms | Segovia | 194.5 ms | 256.4 ms | **100%** | **0.28 GB** |
| 300 ms | SpikeInterface online | 245.8 ms | 365.7 ms | 69.5% | 0.52 GB |
| 1000 ms | Segovia | 617.3 ms | 705.9 ms | **100%** | **0.49 GB** |
| 1000 ms | SpikeInterface online | 786.0 ms | 947.5 ms | 98.2% | 0.74 GB |

**Synthetic recordings** (384 ch, `SyntheticEphysReader`, seed 0): 100% deadline adherence at
all chunk sizes (100/300/1000 ms) with jitter 3.6/8.6/18.6 ms.

Peak memory is bounded and file-size-independent on both sources; on the full 55.8-minute
(29 GB compressed) real recording the memory bound holds to within 1% of a ten-minute slice. In a
batch-throughput comparison on that full recording, Segovia at a pinned batch held peak memory below
both SpikeInterface executor modes (1.19 GB vs 2.18 GB thread-pool and 4.42 GB process-pool) and
completed in less wall time (806 s vs 923 s and 1022 s) in a single run; the memory bound is the
robust result and the wall-time margin warrants multi-run replication. Throughput exceeds the
22 MB/s Neuropixels acquisition rate at all configurations. Full tables and reproducibility
scripts are in `docs/research/` and `bench/`.

Known limitation: the zero-phase Butterworth filter introduces a 50 ms look-ahead; a causal
filter mode is not yet implemented. Benchmarks are on a single machine (Windows, 8 physical /
16 logical cores).

## Conclusion

Segovia v0.4.0 provides bounded-memory, near-real-time preprocessing for Neuropixels-scale
electrophysiology as a composable `ChunkSource` iterator with a Python-accessible API. The
100% deadline-adherence result at 300 ms budgets and the sub-0.5 GB file-size-independent RSS
make it suitable for closed-loop and online applications where batch tools fall short.

## Data availability

Benchmark raw results are in `bench/_tmp/results.jsonl` (regenerated via `bench/run_online_sweep.sh`).
The real IBL recording used is `_spikeglx_ephysData_g0_t0.imec0.ap.cbin`; obtain from the
International Brain Laboratory data portal. Synthetic data is generated on demand by
`SyntheticEphysReader` — no deposit required.

Note for GigaByte submission: data deposit in GigaDB is required. Discuss with editorial
team what constitutes the required deposit for a software Technical Release — likely the
benchmark JSON results and/or the synthetic recording (materialize via
`bench/materialize_synthetic.py`).

## References
