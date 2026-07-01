# Full-scale (29 GB / 55.8 min) Segovia vs SpikeInterface benchmark — decision log

Decided 2026-07-01. Target file: the real IBL Neuropixels AP recording
`tests/data/_spikeglx_ephysData_g0_t0.imec0.ap.cbin` (29 GB on disk, 77 GB
decompressed, 385 ch @ 30 kHz, `fileTimeSecs = 3350.97` ≈ 55.8 min). Identical
chain on both engines: bandpass (300–6000 Hz, order 5) → common-median reference →
ZCA whitening. Engines: Segovia, SpikeInterface thread pool, SpikeInterface
process pool (`n_jobs = 8`). SpikeInterface runs arm's-length in `.venv-si`
(v0.102.3) per ADR 0013. Wheel rebuilt to the released v0.4.0
(`maturin develop --release`) before measuring so the numbers cite tagged code.

## Chosen order

1. **RUN NOW — batch memory/throughput** (`bench/bench.py`, no `--limit-samples`).
   As-fast-as-possible; measures wall time, throughput (MB/s), and peak tree-RSS
   with the `< 2 GB` verdict. Upgrades the paper's central memory claim (ADR 0013)
   from the earlier 10-minute slice to the full 55.8-minute recording, proving
   file-size-independent memory at full scale.

2. **QUEUED NEXT — online latency / deadline-adherence**
   (`bench/replay_latency.py` vs `bench/replay_latency_si.py`). Replays at the true
   30 kHz acquisition rate; measures per-chunk latency (mean/SD/median/p95/p99/max),
   jitter, sustained throughput, peak RSS, and deadline-adherence (% chunks meeting
   the real-time bound). Reproduces the 2026-06-30 online claim at full length.
   Cost: ~56 min *per engine* (real-time replay) ≈ ~2 h total — run as a background
   job when ready. Do NOT start until the batch run (step 1) is reviewed.

## Update 2026-07-01 — memory finding + pivot to a pinned-batch sweep

First full run (batch=0 default) gave Segovia **3.29 GB peak vs SI-thread 2.18 GB** —
apparently breaking the "0.99 GB, file-size-independent, decisive win" claim (SI-process
killed before finishing). Investigated instead of trusting it. Root cause (code +
`scratchpad/mem_probe.py` matrix, both agree):

- `src/lib.rs:54` — **`batch == 0` auto-sizes to `rayon::current_num_threads()`**; this
  box has **16 logical CPUs**, so the default became batch=16.
- Peak RSS is `~0.17 GB × batch + ~0.5 GB base`, and **flat with recording length**:
  batch-16 at 10 min = 3.264 GB vs at 30 min = 3.301 GB (**+1%** for 3× data). So the
  thesis (bounded, file-size-independent) **holds** — the 3.29 GB was the 16-thread
  buffer bound, NOT a leak, NOT data growth. The old 0.99 GB was an 8-core measurement.
- Matrix (10 min): batch1 → **0.685 GB** / 263 s; batch4 → **1.184 GB** / 146 s;
  batch16 → 3.264 GB / 195 s. **batch 4 is both faster AND 2.8× leaner than the
  auto-16 default** — the auto default oversubscribes this machine on both axes.

**Consequence for the paper:** the memory win is real but must be re-earned at a
**pinned** batch (machine-independent, reproducible), not the auto default.

**DONE (2026-07-01):** the full 29 GB sweep ran (`scratchpad/full_sweep.log`). Result —
**Segovia batch 4 beats both SI modes on both axes:** 806 s / 1.194 GB vs SI-thread
923 s / 2.176 GB and SI-process 1022 s / 4.419 GB. File-size-independence confirmed at
full scale (batch-1 full 0.691 GB vs 10-min 0.685 GB). Output-equivalence verified: a
`--no-whiten` cross-check has Segovia matching SI to 0.0035 % (bandpass+CMR equivalent);
SI thread==process bit-identical without whitening, so the whitened divergence was SI's
random-subset whitening, not a bug. Full writeup committed intent: **step 1 (batch-1
above) is COMPLETE** and written to `docs/research/2026-07-01-full-scale-si-comparison.md`.

**Still open:** (i) decide whether to change the library `batch=0` auto-default (ADR-worthy,
machine-dependent optimum) vs only pinning the harness; (ii) commit the writeup; (iii)
**step 2 — the online-latency sweep — is still QUEUED** (below).

## Reporting

Both runs' verified results are written to a dated
`docs/research/YYYY-MM-DD-<slug>.md` and committed only after Felipe reviews (per
the standing "commit only when asked" rule). No invented numbers — transcribe only
what the harness prints.
