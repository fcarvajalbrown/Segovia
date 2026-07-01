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

## Reporting

Both runs' verified results are written to a dated
`docs/research/YYYY-MM-DD-<slug>.md` and committed only after Felipe reviews (per
the standing "commit only when asked" rule). No invented numbers — transcribe only
what the harness prints.
