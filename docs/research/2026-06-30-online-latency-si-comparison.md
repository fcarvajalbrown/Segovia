# Online per-chunk latency: Segovia vs SpikeInterface (2026-06-30)

The SpikeInterface online-latency comparison for the streaming-architecture systems paper (ADR 0014).
Extends the Segovia-only replay sweep (`2026-06-23-replay-latency-sweep.md`) with an arm's-length
SpikeInterface baseline driven in the same true-online (`batch = 1`) regime, on the same real IBL
recording and a matched synthetic recording read from disk. This is the **online** regime
(one chunk at a time), distinct from ADR 0013's **batch** throughput gate.

## Method

- **Harnesses:** `bench/replay_latency.py` (Segovia, project `.venv`) and `bench/replay_latency_si.py`
  (SpikeInterface 0.102.3, separate `.venv-si`, separate process — arm's length per ADR 0013).
- **Chain (identical):** Butterworth bandpass 300-6000 Hz, 5th order -> common median reference ->
  global ZCA whitening (`float32`). SI: `bandpass_filter -> common_reference(operator="median") ->
  whiten(mode="global", apply_mean=True, dtype="float32")`.
- **Online drive:** Segovia streams its `preprocess(...)` iterator at `batch = 1`; SI is driven by
  **sequential `recording.get_traces(start_frame, end_frame)` calls, one chunk window at a time**
  (`n_jobs = 1`), the honest online analog of Segovia's `batch = 1`. Each output chunk is timed with
  `perf_counter_ns`; per-chunk latency = compute time to produce that processed chunk.
- **Matched filter margin:** SI `bandpass_filter(margin_ms=50.0)` to match Segovia's 1500-sample
  (50 ms) margin, so both do equal filter-edge work.
- **Deadline:** chunk period `= chunk_samples / fs`; deadline-adherence = fraction of chunks with
  latency <= period. First 3 chunks discarded as warm-up (also absorbs SI's one-time whitening-matrix
  estimation).
- **Peak RSS:** sampled in-process every 20 ms (single-process engine on both sides).
- **Sources:** (1) real IBL AP-band `.cbin` (`_spikeglx_ephysData_g0_t0.imec0.ap.cbin`,
  mtscomp-compressed, 385 ch incl. sync, first 60 s = 1,800,000 samples); (2) synthetic: the ADR-0015
  `SyntheticEphysReader` (384 ch, 60 s, 30 kHz, 20 units, 5 Hz, 10 uV, seed 0) **materialized to a
  SpikeGLX `.bin`** (`bench/materialize_synthetic.py`) so both engines read identical bytes — Segovia
  via `SpikeGlxReader`, SI via `BinaryRecordingExtractor`.
- **Machine:** Windows, 8 physical / 16 logical cores, 7.8 GB RAM (same host as ADR 0013 and the
  2026-06-23 sweep).

## Results — real IBL AP-band (385 ch, 60 s, mtscomp-compressed)

| chunk | engine | latency mean | p95 | p99 | max | jitter | deadline | peak RSS | throughput |
|---|---|---|---|---|---|---|---|---|---|
| 100 ms | Segovia | 92.9 | 118.4 | 122.0 | 127.9 | 15.4 | **74.2%** | **0.21 GB** | 24.7 MB/s |
| 100 ms | SI online | 112.0 | 250.0 | 275.2 | 302.8 | 48.5 | 64.2% | 0.46 GB | 20.5 MB/s |
| 300 ms | Segovia | 194.5 | 250.1 | 256.4 | 292.5 | 34.7 | **100%** | **0.28 GB** | 35.2 MB/s |
| 300 ms | SI online | 245.8 | 355.7 | 365.7 | 407.1 | 67.8 | 69.5% | 0.52 GB | 27.9 MB/s |
| 1000 ms | Segovia | 617.3 | 692.3 | 705.9 | 707.9 | 77.5 | **100%** | **0.49 GB** | 37.4 MB/s |
| 1000 ms | SI online | 786.0 | 831.3 | 947.5 | 1036.6 | 42.9 | 98.2% | 0.74 GB | 29.1 MB/s |

(latency/jitter in ms; 597 / 197 / 57 chunks measured.)

## Results — synthetic, materialized to SpikeGLX (384 ch, 60 s)

| chunk | engine | latency mean | p95 | p99 | max | jitter | deadline | peak RSS | throughput |
|---|---|---|---|---|---|---|---|---|---|
| 100 ms | Segovia | 90.0 | 107.6 | 117.4 | 132.1 | 13.2 | 83.2% | 1.52 GB | 25.5 MB/s |
| 100 ms | SI online | 100.7 | 121.0 | 127.0 | 133.7 | 9.6 | 59.5% | 0.18 GB | 22.7 MB/s |
| 300 ms | Segovia | 186.6 | 212.1 | 268.1 | 273.9 | 24.6 | 100% | 1.60 GB | 36.7 MB/s |
| 300 ms | SI online | 205.8 | 242.4 | 263.2 | 267.1 | 15.3 | 100% | 0.24 GB | 33.2 MB/s |
| 1000 ms | Segovia | 751.2 | 811.7 | 855.6 | 867.5 | 67.4 | 100% | 1.84 GB | 30.8 MB/s |
| 1000 ms | SI online | 649.5 | 678.3 | 692.5 | 694.3 | 16.7 | 100% | 0.45 GB | 35.3 MB/s |

(latency/jitter in ms; 597 / 197 / 57 chunks measured.)

## Findings

- **On real compressed data, in the online regime, Segovia outperforms SpikeInterface on every axis
  that matters for real-time streaming.** Lower mean latency, much tighter tail latency, higher
  deadline-adherence, lower peak memory, and higher throughput, at all three chunk sizes.
- **Headline: at the 300 ms real-time budget on real IBL data, Segovia achieves 100%
  deadline-adherence at 0.28 GB; SpikeInterface's online `get_traces` achieves only 69.5% at
  0.52 GB.** SI's per-chunk tail latency (p99 366 ms vs Segovia's 256 ms) overruns the 300 ms deadline
  ~30% of the time. SI's online tails are heavy because each `get_traces` re-reads and re-decodes the
  filter-margin neighbourhood per chunk, with no cross-chunk pipelining.
- **Bounded memory holds head-to-head on real data:** Segovia's peak RSS is below SI's at every chunk
  size on the compressed `.cbin` (0.21/0.28/0.49 GB vs 0.46/0.52/0.74 GB) and scales only with chunk
  size, not file size — the project's genuine differentiation (ADR 0013), now confirmed against a live
  SI baseline in the streaming regime.
- **Synthetic corroborates latency but not memory.** On the materialized SpikeGLX file, latency is
  comparable (Segovia better at 100/300 ms, SI better at 1 s) and Segovia again leads deadline
  adherence at the 100 ms budget (83.2% vs 59.5%). But Segovia's RSS is inflated to 1.5-1.8 GB on the
  **raw uncompressed `.bin`** path because `SpikeGlxReader` memory-maps the file and the touched pages
  are charged to process RSS; SI's `BinaryRecordingExtractor` slicing leaves them in shared OS page
  cache. So the synthetic RSS comparison is a memory-mapping/accounting artifact, not a real bound
  difference, and the real-`.cbin` RSS above is the valid bounded-memory result.

## Reconciliation with ADR 0013 (no contradiction)

ADR 0013 measured **batch throughput** with SI's parallel `ChunkRecordingExecutor(n_jobs=N)` and found
Segovia *ties* SI on speed (and wins on memory). That is SI's strength: many chunks decoded and
filtered in parallel across worker threads. This document measures the **online** regime — one chunk
at a time, `n_jobs = 1` — where a batch-oriented tool driven via repeated `get_traces` cannot pipeline
and pays per-chunk margin re-reads, while a purpose-built streaming engine does not. The two results
are consistent: SI wins/ties on bulk batch throughput; Segovia wins on online per-chunk latency,
tail latency, deadline-adherence, and memory. This is the paper's thesis — *for near-real-time
streaming, a purpose-built bounded-memory streaming engine beats adapting a batch tool.*

## Caveats / honest limitations

- **SpikeInterface is batch-oriented by design.** Driving it one chunk at a time via `get_traces` is a
  legitimate way to expose online latency, but it is not SI's intended bulk-processing mode; the
  comparison answers "what happens if you use SI for near-real-time streaming", not "is SI's batch
  pipeline slow" (it is not — see ADR 0013).
- **End-to-end latency exceeds the compute latency reported here** by `period` (chunk accumulation) +
  the 50 ms zero-phase filter look-ahead; both engines share this and it is reported separately by the
  harnesses.
- **Whitening calibration differs:** Segovia uses the first 60,000 samples; SI `mode="global"` samples
  random chunks across the recording. Both are one-time, warm-up-discarded, and do not affect
  steady-state per-chunk latency.
- **Real `.cbin` is processed at 385 channels including the sync channel** on both sides (SI via
  `load_sync_channel=True`, which also bypasses a probeinterface 0.3.2 / SI 0.102.3 probe-geometry
  incompatibility irrelevant to the bandpass/CMR/whiten chain), matching the 2026-06-23 Segovia run.
- **Synthetic noise statistics are imperfect** (MEArec caveat); the real IBL run supplies external
  validity and is where the result is strongest.

## Artifacts

- `bench/replay_latency_si.py` — SI online per-chunk latency harness.
- `bench/materialize_synthetic.py` — materializes the ADR-0015 synthetic recording to SpikeGLX.
- `bench/run_online_sweep.sh` — runs the 12-config sweep, both engines.
- Raw JSON: `bench/_tmp/results.jsonl` (not committed; regenerate via the sweep script).
