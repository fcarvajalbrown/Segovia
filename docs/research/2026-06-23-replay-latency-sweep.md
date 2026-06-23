# Replay-at-acquisition-rate latency sweep — synthetic + real IBL (2026-06-23)

First quantitative result for the streaming-architecture systems paper (ADR 0014). Measures the
Segovia `preprocess` chain (bandpass -> CMR -> whiten) streamed at the true acquisition rate on the
built-in synthetic simulator (ADR 0015) and on the real IBL AP-band recording, across a chunk-size
sweep. Segovia-only first cut; the SpikeInterface online comparison is a separate step.

## Method

- **Harness:** `bench/replay_latency.py`, run in the project `.venv`.
- **Model:** each output chunk is timed with `perf_counter_ns`; per-chunk latency = compute time to
  produce that processed chunk. The real-time deadline is the chunk period `period = chunk_samples /
  fs`; deadline-adherence = fraction of chunks with latency <= period. The first 3 chunks are
  discarded as warm-up.
- **`batch = 1` (true online):** one chunk processed at a time, no buffering of future chunks. This is
  the conservative online configuration, not the throughput-optimised batch mode.
- **Peak RSS:** sampled in-process (single-process engine) every 20 ms.
- **Source:** `SyntheticEphysReader`, 384 channels, 60 s, 30 kHz, 20 units, 5 Hz firing, 10 uV noise,
  seed 0. Filter: 5th-order Butterworth bandpass 300-6000 Hz; margin 1500 samples; whitening on,
  calibrated on the first 60000 samples.
- **Machine:** Windows, the same 8-physical / 16-logical core, 7.8 GB-RAM host as the ADR 0013 SC1 runs.

## Results — synthetic (384 ch, 60 s)

| chunk samples | period (deadline) | latency mean | p95 | p99 | max | jitter (SD) | deadline-adherence | peak RSS | throughput |
|---|---|---|---|---|---|---|---|---|---|
| 3000 (100 ms) | 100 ms | 61.3 ms | 68.4 ms | 73.7 ms | 83.1 ms | 3.6 ms | 100% | 0.15 GB | 37.2 MB/s |
| 9000 (300 ms) | 300 ms | 147.9 ms | 163.9 ms | 167.4 ms | 178.1 ms | 8.6 ms | 100% | 0.23 GB | 45.5 MB/s |
| 30000 (1 s) | 1000 ms | 617.8 ms | 650.9 ms | 664.9 ms | 672.8 ms | 18.6 ms | 100% | 0.50 GB | 36.2 MB/s |

597 / 197 / 57 chunks measured respectively (60 s of data, warm-up discarded).

## Results — real IBL AP-band (385 ch, first 60 s of a 29.4 GB .cbin)

Source: `tests/data/_spikeglx_ephysData_g0_t0.imec0.ap.cbin` (mtscomp-compressed), 385 channels
(384 + sync), 30 kHz, first 1,800,000 samples. Same filter/whitening config as the synthetic run.

| chunk samples | period (deadline) | latency mean | p95 | p99 | max | jitter (SD) | deadline-adherence | peak RSS | throughput |
|---|---|---|---|---|---|---|---|---|---|
| 3000 (100 ms) | 100 ms | 82.0 ms | 119.4 ms | 133.6 ms | 147.6 ms | 19.0 ms | **79.1%** | 0.18 GB | 27.9 MB/s |
| 9000 (300 ms) | 300 ms | 166.0 ms | 207.9 ms | 233.9 ms | 274.4 ms | 19.9 ms | 100% | 0.28 GB | 40.8 MB/s |
| 30000 (1 s) | 1000 ms | 505.7 ms | 561.9 ms | 582.9 ms | 593.7 ms | 31.2 ms | 100% | 0.48 GB | 44.7 MB/s |

597 / 197 / 57 chunks measured respectively (60 s of data, warm-up discarded).

## Findings

- **Synthetic: 100% deadline-adherence at every chunk size, down to 100 ms chunks.** Even at the
  tightest 100 ms budget, p99 compute latency is 73.7 ms with 3.6 ms jitter — the chain sustains
  real-time online preprocessing one chunk at a time.
- **Real IBL: 100% adherence at 300 ms+ budgets, but 79.1% at the 100 ms budget.** On real
  mtscomp-compressed data, jitter rises (19-31 ms vs 3.6-18.6 ms synthetic) and p95/p99 at 100 ms
  chunks (119 / 134 ms) exceed the 100 ms deadline ~21% of the time. The cause is the **serial zlib
  decode**, which ADR 0013 already identified as memory-bandwidth-bound: the Rust *compute* meets
  real-time, but real-data *decode* is the limiting factor at the tightest budget. At 300 ms and 1 s
  budgets there is comfortable headroom.
- **Bounded memory, file-size-independent, scaling only with chunk size** on both synthetic and real
  data (synthetic 0.15 / 0.23 / 0.50 GB; real 0.18 / 0.28 / 0.48 GB), consistent with the
  `batch x (chunk + 2*margin) x channels x 4 bytes` model. This is the project's genuine
  differentiation (ADR 0013), now shown to hold in the streaming regime and to be nearly identical
  between synthetic and real sources.
- **Throughput exceeds the ~22 MB/s Neuropixels acquisition rate** at all chunk sizes (synthetic
  36-46 MB/s; real 28-45 MB/s), so the engine runs faster than real time with headroom.

## Caveats / honest limitations

- **End-to-end latency is larger than the compute latency above.** The minimum end-to-end latency to
  emit a chunk is `period` (time to accumulate the chunk's samples) + look-ahead (the zero-phase
  `sosfiltfilt` right margin, `margin / fs` = 50 ms here) + compute. At 100 ms chunks that is
  ~100 + 50 + 61 = ~210 ms. The harness reports the look-ahead component separately.
- **The bandpass is zero-phase (non-causal).** `sosfiltfilt` is forward-backward and needs a bounded
  look-ahead of `margin` future samples, so the chain is "near-real-time with bounded look-ahead", not
  strictly causal. A causal single-pass filter mode would remove the look-ahead term at the cost of
  phase distortion; not implemented.
- **Synthetic noise statistics are imperfect** (MEArec caveat, ADR 0014/0015). These systems metrics
  depend on data shape and scale, not biological truth; the real IBL run above supplies external
  validity, and notably the real-data decode bottleneck only appears on real compressed data.
- **Segovia-only.** No SpikeInterface online-latency comparison yet; SI is batch-oriented and its
  online comparison is a separate piece.

## Open / next

- Add the SpikeInterface online-latency comparison (separate `.venv-si`, arm's-length, per ADR 0013).
- Optional causal single-pass filter mode to report a strictly-causal latency number.
- Explore overlapping decode with compute (prefetch already overlaps reads) to lift the 100 ms-budget
  adherence on real `.cbin`; ADR 0013 found decode parallelism gives no net batch gain
  (memory-bandwidth-bound), but the online-latency picture at small chunks may differ.
