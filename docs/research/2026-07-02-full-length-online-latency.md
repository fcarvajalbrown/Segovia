# Full-length online per-chunk latency at 300 ms: Segovia vs SpikeInterface (2026-07-02)

Extends the 60-second online comparison (`2026-06-30-online-latency-si-comparison.md`) to the **full
55.8-minute** real IBL AP-band recording at the 300 ms chunk budget — the paper's central online
configuration — to replace the cold-start 60-second slice with a steady-state measurement over
11,167 chunks. Same harnesses, same chain, same arm's-length SI in `.venv-si` (0.102.3).

## Method

- **Harnesses:** `bench/replay_latency.py` (Segovia, `.venv`) and `bench/replay_latency_si.py`
  (SpikeInterface, `.venv-si`, separate process), driven exactly as in the 60 s comparison —
  Segovia `batch = 1`; SI sequential `get_traces(start, end)`, `n_jobs = 1`; matched 50 ms filter
  margin; first 3 chunks discarded as warm-up.
- **Source:** real IBL `_spikeglx_ephysData_g0_t0.imec0.ap.cbin` (mtscomp-compressed, 385 ch incl.
  sync, 30 kHz), **full length** — 100,529,156 samples = 3351 s = 55.8 min = 11,167 chunks of
  9000 samples (300 ms).
- **Chunk budget:** 300 ms only (the headline). 100 ms and 1000 ms were not rerun at full length.
- **Machine:** Windows, 8 physical / 16 logical cores. Segovia leg run 2026-07-02 morning; SI leg run
  same day, launched detached via `bench/run_si_leg.ps1` (guarded: prerequisite check, CPU-contention
  guard, single-instance lock, heartbeat, completion marker). The SI leg had one ~13-minute system
  sleep mid-run; Windows QueryPerformanceCounter does not advance during S3 sleep, so per-chunk
  `perf_counter_ns` latencies exclude the suspended interval and no single chunk shows a multi-minute
  outlier (observed SI max latency 932 ms). Wall time is compute time, not counting sleep.

## Results — real IBL AP-band, 300 ms budget, full length (11,167 chunks)

| metric | Segovia | SpikeInterface online |
|---|---|---|
| deadline adherence | **99.7%** | 94.7% |
| peak RSS | **0.214 GB** | 0.411 GB |
| mean latency | **179.2 ms** | 205.3 ms |
| median latency | **164.3 ms** | 172.5 ms |
| p95 latency | **256.3 ms** | 301.4 ms |
| p99 latency | **277.0 ms** | 355.0 ms |
| max latency | **334.5 ms** | 932.0 ms |
| jitter (sd) | **38.6 ms** | 60.5 ms |
| throughput | **38.1 MB/s** | 33.3 MB/s |
| wall time | 2031 s | 2324 s |

Segovia leads on every axis. The robust, large-margin wins are **peak memory (2×), tail latency
(max 334 vs 932 ms; p99 277 vs 355 ms), and jitter (38.6 vs 60.5 ms)**. The deadline-adherence lead
is real but small at steady state: **99.7% vs 94.7%**.

## Reconciliation with the 60-second slice (important)

The 60 s comparison reported SI at **69.5%** deadline adherence and Segovia at **100%** for the same
300 ms budget. At full length SI rises to **94.7%** and Segovia holds at **99.7%**. The difference is
a **cold-start vs steady-state** effect, not a contradiction:

- The first 60 s (197 chunks) is the cold window — first file reads, cold OS page cache, and library
  warm-up beyond the 3-chunk discard. Per-chunk cost is highest there, and SI's per-chunk
  `get_traces` setup pays that cold cost more heavily than Segovia's streaming iterator.
- Over 11,167 chunks SI amortizes the warm-up and reaches steady state, closing most of the
  adherence gap. Segovia is near-100% in both regimes.

**Consequence:** the 60 s slice **overstated the deadline-adherence margin.** The full-length steady-
state result is the representative measure for a sustained-streaming claim, and on it Segovia's
defensible advantages are memory, tail latency, and jitter — not the deadline-adherence gap. Any
paper headline built on "100% vs 69.5%" must be brought down to the steady-state "99.7% vs 94.7%",
with emphasis moved to the memory and tail-latency wins.

## Caveats

- **Single run per engine, no confidence intervals**, and the two legs ran hours apart rather than
  interleaved; treat the point estimates as indicative, not statistically bounded.
- **One system sleep in the SI leg** (QPC-frozen, so per-chunk latencies are unaffected; noted for
  full disclosure).
- **100 ms and 1000 ms budgets not rerun at full length** — only the 300 ms headline. The 60 s slice
  remains the only data at those budgets and is a cold-start measurement.
- **Whitening calibration differs** (Segovia deterministic first-60k samples; SI `mode="global"`
  random subset), so checksums differ (Segovia 5.59e7, SI 5.49e6) — a methodology difference, not a
  bug, identical in kind to the batch comparison; it does not affect steady-state per-chunk latency.
- The `FAILED exit=` recorded by the guarded runner was a cosmetic exit-code read on the venv launcher
  shim; the SI process completed cleanly and produced the full result.
