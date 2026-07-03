---
title: 'Segovia: a chunked, GIL-released, bounded-memory Rust engine for near-real-time electrophysiology preprocessing'
tags:
  - neuroscience
  - electrophysiology
  - neuropixels
  - real-time
  - streaming
  - rust
  - python
  - signal-processing
authors:
  - name: Felipe Carvajal Brown
    orcid: 0000-0002-8300-7587
    corresponding: true
    affiliation: 1
affiliations:
  - name: Independent Researcher
    index: 1
date: 30 June 2026
bibliography: paper.bib
---

# Summary

Segovia is an open-source Rust library (AGPL-3.0) with Python bindings via PyO3 for streaming,
bounded-memory preprocessing of high-density extracellular electrophysiology recordings. It applies
a bandpass-filter → common-median-reference → whitening chain to multi-channel `i16` time-series
in true-online mode—one chunk at a time—holding peak memory proportional to chunk size rather than
recording length. The Python Global Interpreter Lock (GIL) is released for the duration of each
Rust computation so that concurrent Python threads are not blocked. Three production readers are
provided (SpikeGLX `.bin`, Zarr, mtscomp `.cbin`) alongside a built-in streaming synthetic ephys
simulator. Segovia targets researchers building near-real-time closed-loop experiments and
developers who need a memory-predictable preprocessing component that integrates with the
SpikeInterface ecosystem [@Buccino2020].

# Statement of need

High-density probes such as Neuropixels [@Jun2017] acquire 384–768 channels at 30 kHz, producing
roughly 22 MB/s per probe and tens of gigabytes per session. Offline batch preprocessing pipelines
(SpikeInterface, MountainSort) are optimised for throughput on completed recordings and are not
designed for per-chunk latency constraints. When a batch-oriented tool is driven one chunk at a
time in the online regime—the only option in a closed-loop experiment that has not yet finished
recording—each call re-reads and re-decodes the filter-margin neighbourhood for that chunk and may
repeat whitening-matrix estimation; memory usage can grow with recording length rather than with
chunk size.

Near-real-time applications—closed-loop optogenetic stimulation, online brain-machine interface
decoding, hardware-in-the-loop experiments—impose two hard constraints that batch-oriented tools
do not guarantee: (1) each incoming chunk of samples must be preprocessed within the chunk's
acquisition period (the "real-time deadline"), and (2) peak memory must remain bounded and
file-size-independent over arbitrarily long recordings. No existing Python-accessible library
simultaneously satisfies these two constraints with a pure-threading concurrency model—no
subprocess overhead, no GIL-holding kernels—for Neuropixels-scale workloads.

# Architecture

Segovia's central abstraction is the `ChunkSource` trait: an iterator over fixed-size
`(samples × channels)` `i16` buffers. Three production implementations are provided:

- `SpikeGlxReader` — memory-mapped SpikeGLX `.bin` + `.meta` pairs (zero-copy reads).
- `ZarrReader` — chunked Zarr arrays via the `zarrs` crate (gzip, zstd, Blosc codecs).
- `CbinReader` — mtscomp-compressed IBL `.cbin` recordings, per-chunk positioned zlib
  decompression via `flate2`.

A `ChunkSource` is consumed by `preprocess(chunk_source, config)`, which applies a
Rayon-parallelized chain across time-chunks: 5th-order Butterworth bandpass filter
(`sosfiltfilt`), common median reference, and global ZCA whitening. Cross-chunk filter state
(IIR initial conditions) is maintained deterministically regardless of thread count. The GIL is
released for the Rust computation via PyO3's `allow_threads`. Peak memory is bounded by
`batch × (chunk_samples + 2 × margin) × channels × sizeof(f32)` and does not grow with recording
length.

# Built-in streaming simulator

`SyntheticEphysReader` is a `ChunkSource` that emits arbitrarily long synthetic multi-channel
ephys streams without writing to disk. Spike templates combine a Ricker (Mexican-hat) temporal
waveform with a point-source spatial decay (`V(r) = A × d_perp / r`); per-unit firing follows an
independent Poisson process; channel noise is additive white Gaussian. A reproducible,
platform-independent PRNG (SplitMix64 seeding xoshiro256++) guarantees bit-identical output
across operating systems and chunk sizes. Memory footprint is bounded by chunk size, not recording
length, matching the production readers. The `ground_truth()` method returns `(sample, unit,
channel)` spike events for MEArec-style [@BuccinoMEArec2020] evaluation of downstream spike
sorters.

# Evaluation

Evaluation uses a replay-at-acquisition-rate harness (`bench/replay_latency.py`): prerecorded and
synthetic data are streamed from disk at the true 30 kHz sampling rate; per-chunk latency is
measured with nanosecond-precision monotonic clocks. A chunk meets its real-time deadline when
compute latency ≤ chunk period (`chunk_samples / fs`); deadline adherence is the fraction of
chunks satisfying this bound.

On the full 55.8-minute International Brain Laboratory Neuropixels AP-band recording (385 channels,
mtscomp-compressed, 11,167 chunks) at a 300 ms chunk budget, Segovia sustains 99.7% deadline
adherence at 0.21 GB peak RSS; an equivalent SpikeInterface online configuration—sequential
`get_traces` calls, `n_jobs = 1`—reaches 94.7% at 0.41 GB. At steady state the deadline-adherence
margin is modest, but Segovia's advantage is decisive on the axes that bound worst-case real-time
behaviour: half the peak memory, a 2.8× lower maximum latency (334 ms vs 932 ms), lower p99
(277 ms vs 355 ms), and lower jitter (39 ms vs 60 ms), because each `get_traces` call re-decodes the
filter-margin neighbourhood with no cross-chunk pipelining. A cold-start first-60 s window shows a
wider adherence gap (100% vs 69.5%) that narrows as SpikeInterface amortises warm-up over the run;
the full-length steady-state figures above are the representative measure. Throughput exceeds the
22 MB/s Neuropixels acquisition rate.

Peak memory is bounded and file-size-independent on both synthetic and real recordings: in the
online regime peak RSS remains below 0.5 GB across all chunk sizes on the real `.cbin` regardless of
recording length, and on the full 55.8-minute (29 GB compressed) recording the bound holds to within
1% of a ten-minute slice, consistent with the analytical model. In a batch-throughput comparison on
that full recording, Segovia at a pinned batch held peak memory well below both of SpikeInterface's
parallel executors—1.19 GB versus 2.18 GB (thread pool) and 4.42 GB (process pool)—while completing
in less wall time (806 s versus 923 s and 1022 s) in a single run. The bounded-memory result is the
robust, deterministic advantage; the wall-time margin is a single-machine, single-run measurement
that warrants replication with confidence intervals. Full batch tables are in the repository.

A known limitation: the default bandpass filter is zero-phase (`sosfiltfilt`) and introduces a
bounded look-ahead of `margin / fs` (50 ms at the default setting); the chain is therefore
"near-real-time with bounded look-ahead," not strictly causal. A single-pass causal filter mode is
not yet implemented.

Full benchmark tables, the evaluation harness scripts, and reproducibility instructions are in
`docs/research/` and `bench/`.

# AI usage disclosure

[To be completed by the author before submission.]

# Acknowledgements

The name Segovia honors Claudio Segovia (1983–2009), a friend who died of leukemia at 26, and
evokes the Aqueduct of Segovia—a continuous stream carried across a row of segmented stone arches—
as a metaphor for the chunked, span-by-span streaming model at the core of the library.
