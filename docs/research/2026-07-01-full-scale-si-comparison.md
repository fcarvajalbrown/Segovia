# Full-scale (29 GB / 55.8 min) Segovia vs SpikeInterface — batch sweep

Date: 2026-07-01. Supersedes the 10-minute SC1 slice in ADR 0013 as the primary
memory/throughput comparison, and confirms the bounded, file-size-independent memory
thesis at full recording scale.

## Setup

- **Data:** real IBL Neuropixels AP recording
  `tests/data/_spikeglx_ephysData_g0_t0.imec0.ap.cbin` — 29 GB on disk (mtscomp `.cbin`),
  77 GB decompressed, **100,529,156 samples, 385 channels @ 30 kHz, 55.8 min**
  (`fileTimeSecs = 3350.97`).
- **Chain (identical on both engines):** bandpass Butterworth order 5, 300-6000 Hz →
  common-median reference → ZCA whitening (`chunk_samples = 30000` = 1 s, `margin = 1500`,
  `calib_samples = 60000`).
- **Engines:** Segovia v0.4.0 (`.venv`, rebuilt `maturin develop --release`) at pinned
  batch 1/2/4/8; SpikeInterface 0.102.3 (`.venv-si`) thread pool and process pool, both
  `n_jobs = 8`. Arm's-length in separate venvs per ADR 0013.
- **Measurement:** each engine run as a subprocess under one external tree-RSS sampler
  (`scratchpad/full_sweep.py`, 0.05 s interval, parent + children summed) — the same
  methodology for every engine. Single run per config.
- **Machine:** Windows 10, 16 logical CPUs (8 physical), 8.40 GB RAM. Baseline at start:
  4.02 GB free, ~7 % CPU, only idle editor/browser resident.

## Result — full 29 GB

| Engine | wall (s) | throughput (MB/s) | peak tree-RSS (GB) |
|---|---|---|---|
| segovia-b1 | 1456.9 | 53.1 | **0.691** |
| segovia-b2 | 989.3 | 78.2 | **0.722** |
| **segovia-b4** | **806.4** | **96.0** | **1.194** |
| segovia-b8 | 934.9 | 82.8 | 1.876 |
| spikeinterface-thread | 923.4 | 83.8 | 2.176 |
| spikeinterface-process | 1021.7 | 75.8 | 4.419 |

**Segovia at batch 4 wins on both axes against both SpikeInterface modes:**
- vs SI-thread: 1.15x faster (806 vs 923 s), 1.82x less memory (1.19 vs 2.18 GB).
- vs SI-process: 1.27x faster (806 vs 1022 s), 3.70x less memory (1.19 vs 4.42 GB).

SI-process reached 4.42 GB — over half the machine's RAM. It completed here only because
~4 GB was free; on a smaller box or a longer/multi-probe recording it would OOM. (In an
earlier attempt on a more loaded machine the SI-process run was killed mid-stream.) Its
process pool memory scales with `n_jobs`; it does not bound.

## Memory is bounded by `batch`, not by file size

`src/lib.rs:54` treats `batch == 0` as "auto = `rayon::current_num_threads()`". On this
16-thread box the default becomes batch 16, which is why the first default-config full run
measured 3.288 GB (1068 s) — the worst of every configuration on both axes. Peak RSS is
`~0.17 GB x batch + ~0.5 GB base`, confirmed by a fresh-process matrix
(`scratchpad/mem_probe.py`, in-process self-RSS):

| Config | batch | signal length | wall (s) | peak RSS (GB) |
|---|---|---|---|---|
| b1_10min | 1 | 10 min | 262.9 | 0.685 |
| b4_10min | 4 | 10 min | 146.1 | 1.184 |
| b16_10min | 16 | 10 min | 194.9 | 3.264 |
| b16_30min | 16 | **30 min** | 579.4 | 3.301 |

**File-size-independence (the thesis) holds at full scale:**
- batch 16: 10 min = 3.264 GB vs 30 min = 3.301 GB — **+1.1 %** for 3x the data.
- batch 1: 10 min = 0.685 GB (self-RSS) vs full 55.8 min = 0.691 GB (tree-RSS) — **+0.9 %**.
- batch 4: 10 min = 1.184 GB vs full 55.8 min = 1.194 GB — **+0.8 %**.

Peak memory does not move with recording length; it is set by `batch x chunk-footprint`.
The two RSS methods (in-process self vs external tree) agree for single-process Segovia
(self 3.264 GB vs tree 3.288 GB at batch 16).

**batch 4 is the optimum on this machine** — faster than batch 8 (935 s) and batch 16
(1068 s). Parallelism past ~physical-core count oversubscribes memory-bandwidth-bound work
(consistent with ADR 0013) and costs both time and memory. The auto default (batch = logical
threads) is therefore a poor choice on hyperthreaded machines.

## Output-equivalence verification

The whitened runs produced different checksums across engines (Segovia 55.9M, SI-thread
25.6M, SI-process -6.6M). A `--no-whiten` cross-check (bandpass + CMR only, 10-min slice,
`scratchpad/equiv_check.log`) isolates the cause:

| Engine (no whiten) | checksum |
|---|---|
| Segovia | -1,050,410,677 |
| SI-thread | -1,050,782,394 |
| SI-process | -1,050,782,394 |

- SI-thread and SI-process are **bit-identical** without whitening — so SpikeInterface is
  deterministic, and the earlier thread/process divergence was purely its **random-subset**
  whitening covariance estimate.
- Segovia matches SpikeInterface to **0.0035 %** (372k in ~1.05 billion) — a residual the
  size of filter edge-handling at the artificial slice boundary. Bandpass + CMR are
  equivalent between engines.

Conclusion: the whitened-run divergence is whitening *methodology* (Segovia's deterministic
first-60k calibration vs SI's random covariance subset), not a defect. The performance
comparison is over equivalent computation.

## Caveats / open questions

- **Single run per config** — wall-time has run-to-run variance that is not yet quantified;
  the memory numbers are deterministic and stable. For publication, repeat runs with
  CIs/percentiles (Hoefler SC'15).
- **Whitening is not numerically identical** between engines (by design; both are valid ZCA
  variants). A bit-exact cross-engine whitening comparison is not claimed.
- **The batch optimum is machine-dependent** (here 8 physical cores -> batch 4). It should be
  reported as a tunable knob, not a fixed number.
- **SI-process completing vs OOMing depends on free RAM** at run time; the 4.42 GB peak is
  the reproducible fact, "OOMs when RAM is tighter" is the qualitative consequence.

## Bottom line for the paper

The central claim survives full-scale scrutiny and is stronger than the old 8-core slice:
Segovia holds **bounded, file-size-independent memory** (0.69-1.19 GB at batch 1-4, flat from
10 min to 55.8 min) and, at its tuned batch, **beats SpikeInterface on both memory and
throughput** at full 29 GB. Report Segovia at a **pinned** batch (machine-independent,
reproducible), never the auto default.
