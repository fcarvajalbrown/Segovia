# Segovia: a chunked, GIL-released, bounded-memory streaming engine for near-real-time multichannel electrophysiology preprocessing

**Felipe Carvajal Brown** — Independent Researcher — fcarvajalbrown@gmail.com — ORCID 0000-0002-8300-7587

Preprint. Submitted to PCI Neuroscience for open peer review.

---

## Abstract

High-density extracellular electrophysiology probes (Neuropixels) acquire 384–768 channels at
30 kHz, generating data at ~22 MB/s per probe. Preprocessing this stream for near-real-time
applications—closed-loop stimulation, online brain-machine interface decoding—requires that each
chunk of incoming samples be processed within its acquisition period while keeping peak memory
bounded and independent of recording length. Batch-oriented tools driven one chunk at a time
re-read filter-margin neighbourhoods on every call and do not bound memory analytically. We
describe Segovia, an open-source Rust library (AGPL-3.0) with Python bindings (PyO3) that
addresses this gap. Segovia's core abstraction is the `ChunkSource` trait: an iterator over
fixed-size `i16` chunk buffers that is consumed by a Rayon-parallelized bandpass-filter →
common-median-reference → ZCA-whitening chain. The Python Global Interpreter Lock is released
for the duration of each Rust computation. Peak memory is analytically bounded by chunk size,
not recording length. We evaluate Segovia on a real International Brain Laboratory Neuropixels
AP-band recording (385 channels, mtscomp-compressed) and on a built-in streaming synthetic
simulator (`SyntheticEphysReader`), using a replay-at-acquisition-rate harness that measures
per-chunk latency and deadline adherence without requiring hardware. At a 300 ms chunk budget on
real data, Segovia achieves 100% real-time deadline adherence at 0.28 GB peak RSS; an equivalent
SpikeInterface online-streaming configuration achieves 69.5% adherence at 0.52 GB. Segovia is
available at https://github.com/fcarvajalbrown/Segovia and via `pip install segovia`.

---

## 1. Introduction

Neuropixels probes [@Jun2017] have made large-scale, high-density extracellular electrophysiology
routine: a single probe acquires 384 channels at 30 kHz, generating ~22 MB/s, and multi-probe
configurations multiply this by 4–8×. A session of 1–2 hours produces tens of gigabytes of raw
data. Before spike sorting can begin, this stream must pass through a preprocessing chain:
bandpass filtering to isolate action-potential frequencies (300–6000 Hz), common-median
referencing (CMR) to suppress common-mode noise, and whitening to decorrelate channels. In the
standard offline pipeline (SpikeInterface [@Buccino2020], MountainSort), this step runs after
recording is complete, optimized for throughput on a fixed dataset.

Near-real-time applications impose different constraints. Closed-loop optogenetic stimulation
must detect a neural event and respond within tens to hundreds of milliseconds—within the same
chunk of samples that the event occurred in. Online brain-machine interface decoders must
continuously emit decoded signals with bounded latency. Hardware-in-the-loop experiments must
synchronize preprocessing output with external devices at acquisition rate. These applications
require two guarantees that batch-oriented pipelines do not provide:

1. **Real-time deadline adherence.** Each chunk of `N` samples must be fully preprocessed within
   `N / fs` seconds (the "chunk period"). Exceeding this deadline stalls the application.
2. **Bounded, file-size-independent peak memory.** A closed-loop system may run for hours or
   days; it cannot exhaust RAM as the recording grows.

When a batch-oriented tool such as SpikeInterface is driven one chunk at a time via sequential
`get_traces` calls—the natural online adaptation—each call re-reads and re-decodes the
filter-margin neighbourhood (the look-ahead and look-behind samples needed by the filter), and
whitening-matrix estimation may be repeated or anchored to cached state in ways not designed for
streaming. Memory usage grows with the number of processed chunks or with the mapping of the
full recording, not with chunk size alone.

We present Segovia, a purpose-built streaming preprocessing engine that satisfies both
constraints by construction.

## 2. Related work

**Batch preprocessing frameworks.** SpikeInterface [@Buccino2020] provides a unified Python API
for spike sorting that includes the standard preprocessing chain. Its `ChunkRecordingExecutor`
parallelizes processing across chunks using a thread or process pool, optimized for bulk
throughput. It does not target per-chunk latency or analytical memory bounds. MountainSort
[@Chung2017] follows a similar offline model. Kilosort4 [@Pachitariu2024], the current
state-of-the-art GPU spike sorter, operates on preprocessed recordings rather than raw streams.
None of these tools are designed for the online, one-chunk-at-a-time regime.

**Streaming and closed-loop frameworks.** improv [@Draelos2025] provides a Python framework for
closed-loop calcium imaging experiments, benchmarked via replay-at-acquisition-rate on prerecorded
data rather than live hardware. BRAND [@Ali2024] is a modular platform for closed-loop
experiments with deep network models, achieving sub-600 µs latency via asynchronous Redis-based
communication. RT-Sort [@vanderMolen2024] implements real-time spike detection and sorting with
7.5 ms latency on recorded ground-truth datasets. These precedents establish that
replay-at-acquisition-rate is an accepted evaluation methodology for near-real-time systems
without requiring a live hardware rig [@Hoefler2015].

**Simulators.** MEArec [@BuccinoMEArec2020] generates synthetic extracellular recordings with
ground-truth spike labels from biophysical neuron models. Segovia's built-in simulator adapts the
MEArec biophysical template approach (Ricker waveform, point-source spatial decay) into a
bounded-memory streaming generator that emits data one chunk at a time without materializing the
full recording on disk.

**Rust in scientific computing.** The PyO3 ecosystem [@PyO3] enables Rust libraries to expose
Python APIs with GIL-released computation, combining C-level performance with Python usability.
Rayon [@Rayon] provides data-parallel iterators over chunks using a work-stealing thread pool,
with no subprocess overhead and no pickle-based inter-process communication.

## 3. Architecture

### 3.1 The `ChunkSource` trait

Segovia's central abstraction is the `ChunkSource` trait, which provides an iterator over
fixed-size `(samples × channels)` `i16` buffers. Each call to `next()` returns exactly
`chunk_samples × n_channels` samples in row-major (samples × channels) order, matching the
layout expected by downstream spike sorters. Three production implementations are provided:

- **`SpikeGlxReader`** reads SpikeGLX `.bin` + `.meta` pairs via memory-mapped I/O. Chunks are
  sliced from the memory map with zero copy; no decompression is required.
- **`ZarrReader`** reads chunked Zarr arrays (gzip, zstd, Blosc codecs) via the `zarrs` crate.
  Chunk boundaries in the Zarr array are aligned to the requested chunk size where possible.
- **`CbinReader`** reads mtscomp-compressed IBL `.cbin` recordings. Each chunk is decompressed
  independently via per-chunk positioned reads using `flate2` (zlib). This is the format used
  for the real-data benchmark in §5.

### 3.2 The preprocessing chain

A `ChunkSource` is consumed by `preprocess(chunk_source, config)`, which chains three operations:

**Bandpass filter.** A 5th-order Butterworth filter with zero-phase response (`sosfiltfilt`) is
applied to each chunk. Zero-phase filtering requires a look-ahead (`margin`) of future samples to
compute the backward pass; Segovia fetches `margin` samples beyond the current chunk boundary and
discards them from the output, so the output is time-aligned. This introduces a bounded look-ahead
latency of `margin / fs` (50 ms at the default `margin = 1500` samples, `fs = 30 000` Hz). A
causal single-pass filter mode that eliminates this look-ahead is planned but not yet implemented.

**Common median reference (CMR).** The median across all channels is subtracted sample-by-sample.
This suppresses motion artefacts and common-mode electrical noise without requiring a reference
electrode. The operation is applied after bandpass filtering on each chunk independently.

**Global ZCA whitening.** The whitening matrix is estimated from the first `n_calib` samples
(default: 60 000; 2 s at 30 kHz). The matrix is stored and reused for all subsequent chunks.
Whitening is applied in `f32` to avoid integer overflow; output is `f32`.

**Parallelism and GIL release.** The chain is Rayon-parallelized across time-chunks. When
`batch > 1`, multiple chunks are processed concurrently using Rayon's work-stealing thread pool.
Cross-chunk filter state (IIR initial conditions for the forward and backward passes) is
maintained deterministically: the state for chunk `k` is seeded from the last `margin` samples of
chunk `k−1`, so output is identical regardless of thread count. The Python GIL is released via
PyO3's `allow_threads` for the entire Rust computation, so Python threads blocked on I/O or other
work are not held.

**Memory bound.** Peak memory is bounded by:

```
M = batch × (chunk_samples + 2 × margin) × n_channels × sizeof(f32)
```

This is independent of recording length. At `batch = 1`, `chunk = 9000`, `margin = 1500`,
`n_channels = 384`, the bound is ~0.08 GB. Observed peak RSS on real data is higher (0.18–0.49 GB)
due to Python runtime, the decompressor, and the whitening matrix, but scales only with
chunk size, not recording length.

### 3.3 Built-in streaming simulator

`SyntheticEphysReader` is a `ChunkSource` that emits arbitrarily long synthetic multi-channel
ephys streams without writing to disk or reading from a file. Each chunk is generated on demand
using the following model:

- **Spike templates:** Each unit has a Ricker (Mexican-hat) temporal waveform of amplitude `A`
  convolved with a point-source spatial decay `V(r) = A × d_perp / r`, where `r` is the
  probe-to-source distance and `d_perp` is the perpendicular component. Waveforms are pre-computed
  at initialisation and stamped into the chunk buffer at spike times.
- **Spike times:** Each unit fires independently following a Poisson process with rate `lambda`.
- **Noise:** Additive white Gaussian noise with standard deviation `sigma` is added to all
  channels.
- **PRNG:** A SplitMix64 seed generator feeds per-unit xoshiro256++ PRNGs. The PRNG is
  deterministic across platforms and chunk sizes: splitting the recording into different chunk
  sizes yields bit-identical output.

The simulator's memory footprint equals one chunk buffer plus the template bank (bounded, small).
It is used in two roles: as a standalone benchmark source for systems metrics (§5.2), and as the
basis for `ground_truth()`, which returns `(sample_index, unit_id, peak_channel)` tuples for
MEArec-style spike-sorter accuracy evaluation.

## 4. Evaluation methodology

We follow the replay-at-acquisition-rate methodology: prerecorded or synthetic data are streamed
from disk (or generated) at the true 30 kHz sampling rate; per-chunk end-to-end latency is
measured with nanosecond-precision monotonic clocks (`perf_counter_ns`). A chunk meets its
real-time deadline when its compute latency ≤ the chunk period (`chunk_samples / fs`). Deadline
adherence is the fraction of chunks satisfying this bound. The first 3 chunks of each run are
discarded as warm-up.

**Metrics reported:** per-chunk latency mean, standard deviation (jitter), p95, p99, max;
deadline adherence; peak whole-process RSS (sampled every 20 ms in-process); sustained throughput
(total output bytes ÷ total elapsed wall time).

**SpikeInterface baseline.** SI is run in a separate venv (`.venv-si`, `spikeinterface==0.102.3`)
in the same process, driven by sequential `get_traces(start_frame, end_frame)` calls with
`n_jobs = 1`—the online analog of Segovia's `batch = 1`. Filter margin is matched
(`bandpass_filter(margin_ms=50.0)`). Whitening uses `mode="global"` with random calibration
chunks (SI default); Segovia uses the first 60 000 samples. Both differences affect only warm-up
and are excluded by the 3-chunk discard.

**Machine:** Windows, 8 physical / 16 logical cores, 7.8 GB RAM.

## 5. Results

### 5.1 Real IBL AP-band recording

Source: `_spikeglx_ephysData_g0_t0.imec0.ap.cbin` from the IBL reproducible ephys dataset
[@IBL2025] (mtscomp-compressed, 385 channels including sync, 30 kHz, first 60 s = 1 800 000
samples).

| Chunk | Engine | Mean (ms) | p95 (ms) | p99 (ms) | Max (ms) | Jitter (ms) | Adherence | Peak RSS | Throughput |
|---|---|---|---|---|---|---|---|---|---|
| 100 ms | Segovia | 92.9 | 118.4 | 122.0 | 127.9 | 15.4 | **74.2%** | **0.21 GB** | 24.7 MB/s |
| 100 ms | SI online | 112.0 | 250.0 | 275.2 | 302.8 | 48.5 | 64.2% | 0.46 GB | 20.5 MB/s |
| 300 ms | Segovia | 194.5 | 250.1 | 256.4 | 292.5 | 34.7 | **100%** | **0.28 GB** | 35.2 MB/s |
| 300 ms | SI online | 245.8 | 355.7 | 365.7 | 407.1 | 67.8 | 69.5% | 0.52 GB | 27.9 MB/s |
| 1000 ms | Segovia | 617.3 | 692.3 | 705.9 | 707.9 | 77.5 | **100%** | **0.49 GB** | 37.4 MB/s |
| 1000 ms | SI online | 786.0 | 831.3 | 947.5 | 1036.6 | 42.9 | 98.2% | 0.74 GB | 29.1 MB/s |

(597 / 197 / 57 chunks measured per row.)

At the 300 ms budget, Segovia achieves 100% deadline adherence at 0.28 GB; SpikeInterface online
achieves 69.5% at 0.52 GB. SI's per-chunk tail latency is high (p99 366 ms vs. Segovia's 256 ms)
because each `get_traces` call re-reads and re-decodes the 50 ms filter-margin neighbourhood, with
no cross-chunk pipelining. At 100 ms, both engines fall short of 100% adherence because the
mtscomp zlib decode is memory-bandwidth bound (established in ADR 0013): the Rust compute meets
the deadline but the decode does not.

Peak RSS scales only with chunk size on real data, consistent with the analytical bound. Segovia's
RSS is below SI's at every chunk size.

### 5.2 Synthetic recordings (Segovia-only baseline)

Source: `SyntheticEphysReader`, 384 channels, 60 s, 30 kHz, 20 units, 5 Hz Poisson firing,
10 μV noise, seed 0.

| Chunk | Mean (ms) | p95 (ms) | p99 (ms) | Max (ms) | Jitter (ms) | Adherence | Peak RSS | Throughput |
|---|---|---|---|---|---|---|---|---|
| 100 ms | 61.3 | 68.4 | 73.7 | 83.1 | 3.6 | **100%** | 0.15 GB | 37.2 MB/s |
| 300 ms | 147.9 | 163.9 | 167.4 | 178.1 | 8.6 | **100%** | 0.23 GB | 45.5 MB/s |
| 1000 ms | 617.8 | 650.9 | 664.9 | 672.8 | 18.6 | **100%** | 0.50 GB | 36.2 MB/s |

On synthetic uncompressed streams Segovia achieves 100% deadline adherence at all chunk sizes, with
markedly lower jitter than on real compressed data (3.6 ms vs 15.4 ms at 100 ms chunks). The
lower jitter reflects the absence of zlib decode variance. Throughput exceeds 22 MB/s at all
configurations.

### 5.3 Batch throughput (context only)

The online comparison above is not comparable to batch throughput. In the batch regime (SI's
`ChunkRecordingExecutor`, `n_jobs = N`), SpikeInterface's parallel C/MKL kernels run across all
cores and achieve speeds SI was designed for. Segovia ties SI on batch throughput and wins on
peak memory (0.99 GB vs SI's 1.75 GB thread-pool / 2.84 GB process-pool, the latter exceeding
the 2 GB bound). The no-faster-than-SI-in-batch conclusion (ADR 0013) stands and is not
contradicted by the online results: SI is a batch tool; Segovia is a streaming tool. Each wins
in its intended regime.

## 6. Discussion

**The online vs batch distinction.** The results establish a clear separation: batch-oriented
tools driven one chunk at a time pay per-chunk overhead that a purpose-built streaming engine
eliminates. This is not a deficiency in SpikeInterface but a natural consequence of its design
goal. The implication is that closed-loop applications requiring <300 ms preprocessing latency
with bounded memory should use a streaming-first tool; batch tools remain the right choice for
offline analysis.

**Non-causal filter and look-ahead.** The zero-phase Butterworth filter introduces a 50 ms
look-ahead (the `margin / fs` term). This means true end-to-end latency from sample acquisition
to preprocessed output is `chunk_period + 50 ms + compute_latency`, not just `compute_latency`.
At 300 ms chunks this is ~400–450 ms total. A causal single-pass filter mode would eliminate the
50 ms look-ahead at the cost of phase distortion; this is future work. All latency figures in
this paper are compute latency only; the look-ahead is reported separately by the harness.

**Synthetic data limitations.** The synthetic simulator does not reproduce the exact noise
statistics of real recordings—this is a documented limitation of the MEArec-style approach. The
benchmark metrics reported here (latency, memory, deadline adherence) are systems metrics that
depend on data shape and scale, not on biological fidelity, so the synthetic results are valid
for the claims made. The real IBL run is retained for external validity and surfaces the real
decoding bottleneck (mtscomp zlib) that synthetic uncompressed data does not expose.

**Single-machine measurements.** All benchmarks are on a single machine (Windows, 8 physical
cores, 7.8 GB RAM). Performance on other hardware configurations will differ. The analytical
memory bound `M = batch × (chunk + 2×margin) × channels × sizeof(f32)` is machine-independent;
the latency and throughput figures are not.

**Future work.** A causal filter mode; parallelized mtscomp decompression to lift the 100 ms
budget limitation on compressed data; a thin live-monitor GUI for real-time latency/throughput/RSS
visualization; the IFC cross-domain simulator leg (`sim/ifc`).

## 7. Conclusion

Segovia is a streaming, bounded-memory preprocessing engine for Neuropixels-scale electrophysiology
that satisfies two hard constraints for near-real-time applications—real-time deadline adherence
and file-size-independent memory—that batch-oriented tools do not provide in the online regime.
At a 300 ms chunk budget on a real mtscomp-compressed IBL recording, Segovia achieves 100%
deadline adherence at 0.28 GB; SpikeInterface online achieves 69.5% at 0.52 GB. The engine is
available as `pip install segovia` (Python, PyPI) and `cargo add segovia` (Rust, crates.io) under
AGPL-3.0-or-later.

## Acknowledgements

The name Segovia honors Claudio Segovia (1983–2009), a friend who died of leukemia at 26.

## References
