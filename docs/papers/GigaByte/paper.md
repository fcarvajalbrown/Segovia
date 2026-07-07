# Segovia v0.4.1: a bounded-memory, near-real-time streaming preprocessor for Neuropixels-scale electrophysiology

**Felipe Carvajal Brown** — Independent Researcher — fcarvajalbrown@gmail.com — ORCID 0000-0002-8300-7587

Corresponding author: Felipe Carvajal Brown (fcarvajalbrown@gmail.com)

Submission type: **Technical Release**

---

## Abstract

Segovia is an open-source Rust library with Python bindings for bounded-memory, near-real-time
preprocessing of high-density extracellular electrophysiology recordings. It applies a
bandpass-filter → common-median-reference → whitening chain to Neuropixels-scale (384–768 channels,
30 kHz) 16-bit integer streams one chunk at a time, releasing the Python Global Interpreter Lock for
each Rust computation. Peak memory is bounded by chunk size, not by recording length, and scales
analytically. On a full 55.8-minute International Brain Laboratory AP-band recording (385 channels,
mtscomp-compressed) at a 300 ms chunk budget, Segovia holds 99.7% real-time deadline adherence at
0.21 GB peak resident memory. The package ships three production readers (SpikeGLX, Zarr, mtscomp
`.cbin`) and a built-in streaming synthetic simulator.

### Availability and Implementation

Source code (Rust + Python, AGPL-3.0-or-later): https://github.com/fcarvajalbrown/Segovia.
Install via `pip install segovia` (PyPI) or `cargo add segovia` (crates.io). Version v0.4.1.

---

## Research Area and Classifications

Selected in the submission system. Suggested research area: **Neuroscience**. Suggested
classifications: computational neuroscience; software / bioinformatics tools; signal processing;
real-time and streaming systems. (These are entered on the GigaByte submission form and are recorded
here as the intended selections.)

---

## Statement of Need

Neuropixels probes [@Jun2017] acquire 384 channels at 30 kHz (~22 MB/s per probe). Standard
preprocessing pipelines (SpikeInterface [@Buccino2020], MountainSort) run offline on completed
recordings and are optimized for batch throughput. Near-real-time applications — closed-loop
stimulation, online brain-machine interface decoding — require that each incoming chunk of samples be
preprocessed within its acquisition period (the real-time deadline), with peak memory independent of
recording length. Batch tools driven one chunk at a time re-read filter-margin neighbourhoods on
every call and do not bound memory analytically. Segovia was built to fill this gap: a composable
streaming preprocessor whose memory ceiling is fixed by chunk size and whose per-chunk latency meets
the acquisition deadline on real recordings.

## Implementation

### Core abstraction

Segovia's `ChunkSource` trait is an iterator over fixed-size `(samples × channels)` 16-bit integer
buffers. Three production implementations are provided:

- `SpikeGlxReader` — memory-mapped SpikeGLX `.bin` + `.meta` (zero-copy).
- `ZarrReader` — chunked Zarr arrays (gzip, zstd, Blosc) via the `zarrs` crate.
- `CbinReader` — mtscomp-compressed IBL `.cbin`, per-chunk zlib decompression via `flate2`.

### Preprocessing chain

`preprocess(chunk_source, config)` applies a Rayon-parallelized chain: 5th-order Butterworth
bandpass filter (zero-phase `sosfiltfilt`), common median reference, and global ZCA whitening. The
Python GIL is released via PyO3's `allow_threads` for the Rust computation. Cross-chunk IIR filter
state is maintained deterministically regardless of thread count.

Peak resident memory scales as `batch × (chunk + 2 × margin) × channels × sizeof(f32)` and is
independent of recording length. The default (auto) batch width is capped at `min(logical_threads, 4)`
as a conservative out-of-memory guard, bounding the default footprint to ~1.2 GB on any machine;
callers may pin an explicit batch. This cap is an OOM safety guard, not a throughput-optimum claim.

### Built-in simulator

`SyntheticEphysReader` emits arbitrarily long synthetic streams without writing to disk. Spike
templates use a Ricker temporal waveform with point-source spatial decay (`V(r) = A × d_perp / r`);
firing is Poisson per unit; noise is additive white Gaussian. The pseudorandom number generator
(SplitMix64 + xoshiro256++) is platform-independent and chunk-size-independent. `ground_truth()`
returns `(sample, unit, channel)` tuples for downstream sorter evaluation.

## Results

Evaluation follows a replay-at-acquisition-rate methodology: data are streamed at the true 30 kHz
rate with per-chunk compute latency measured by a monotonic clock. Deadline adherence is the fraction
of chunks with latency at or below the chunk period.

Figure 1 summarizes the full-length steady-state comparison.

![Full-length steady-state comparison on the real IBL AP-band recording (385 channels, 55.8 min, 300 ms budget). (a) Per-chunk latency (mean, p99, maximum) against the 300 ms real-time deadline; Segovia's 334.5 ms maximum yields 99.7% deadline adherence, whereas SpikeInterface online reaches 932 ms. (b) Peak resident memory, 0.21 GB versus 0.41 GB.](template/fig_latency_memory.png)

**Real IBL AP-band recording, full length** (385 ch, mtscomp-compressed, 55.8 min, 11,167 chunks,
300 ms budget, steady state):

| Engine | Mean | p99 | Max | Adherence | Peak RSS | Jitter |
|---|---|---|---|---|---|---|
| Segovia | 179.2 ms | 277.0 ms | **334.5 ms** | **99.7%** | **0.21 GB** | **38.6 ms** |
| SpikeInterface online | 205.3 ms | 355.0 ms | 932.0 ms | 94.7% | 0.41 GB | 60.5 ms |

At steady state Segovia leads on every axis; the decisive margins are peak memory (2×), maximum
latency (2.8×), and jitter. The per-chunk table below is the cold-start first-60 s window, where
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

**Synthetic recordings** (384 ch, `SyntheticEphysReader`, seed 0): 100% deadline adherence at all
chunk sizes (100/300/1000 ms) with jitter 3.6/8.6/18.6 ms.

Peak memory is bounded and file-size-independent on both sources; on the full 55.8-minute (29 GB
compressed) real recording the memory bound holds to within 1% of a ten-minute slice. In a
batch-throughput comparison on that full recording, Segovia at a pinned batch held peak memory below
both SpikeInterface executor modes (1.19 GB vs 2.18 GB thread-pool and 4.42 GB process-pool) and
completed in less wall time (806 s vs 923 s and 1022 s) in a single run; the memory bound is the
robust result and the wall-time margin warrants multi-run replication. Throughput exceeds the
22 MB/s Neuropixels acquisition rate at all configurations. Full tables and reproducibility scripts
are in `docs/research/` and `bench/`.

Known limitation: the zero-phase Butterworth filter introduces a 50 ms look-ahead; a causal filter
mode is not yet implemented. Benchmarks are on a single machine (Windows, 8 physical / 16 logical
cores).

## Availability of Supporting Source Code and Requirements

- **Project name:** Segovia
- **Project home page:** https://github.com/fcarvajalbrown/Segovia
- **Operating system(s):** Linux, macOS, Windows (platform independent)
- **Programming language:** Rust (core, edition 2021, MSRV 1.74); Python bindings via PyO3
- **Other requirements:** Python ≥ 3.8 for the bindings (prebuilt abi3 wheels on PyPI); a Rust
  toolchain and `maturin` to build from source
- **License:** AGPL-3.0-or-later (OSI-approved)
- **RRID:** to be registered at SciCrunch.org prior to publication
- **bio.tools ID:** not yet registered
- **Version reported:** v0.4.1

## Data Availability Statement

Segovia is a software library; no new experimental dataset was generated. The real recording used
for evaluation is a publicly released International Brain Laboratory AP-band Neuropixels recording
(`_spikeglx_ephysData_g0_t0.imec0.ap.cbin`), obtainable from the International Brain Laboratory data
portal. Benchmark raw results are in the repository (`bench/_tmp/`, regenerated via the harness in
`bench/`). Synthetic recordings are generated deterministically on demand by `SyntheticEphysReader`
(seed-fixed, no deposit required) and can be materialized via `bench/materialize_synthetic.py`. The
form of the required GigaDB deposit for a software Technical Release is being confirmed with the
editorial team.

## List of Abbreviations

- **AGPL:** GNU Affero General Public License
- **AP:** action potential (band)
- **CMR:** common median reference
- **GIL:** Global Interpreter Lock
- **IBL:** International Brain Laboratory
- **IIR:** infinite impulse response
- **MSRV:** minimum supported Rust version
- **RSS:** resident set size
- **SOS:** second-order sections
- **ZCA:** zero-phase component analysis (whitening)

## Declarations

**Ethics approval and consent to participate:** Not applicable. No human or animal data were
collected for this work; the recording used for evaluation is a publicly released dataset.

**Competing interests:** The author declares no competing interests.

**Funding:** No funding was received for this work.

**Authors' contributions:** F.C.B. is the sole author and performed all work (CRediT:
conceptualization, software, methodology, investigation, writing).

**Acknowledgements:** The author thanks the International Brain Laboratory for the publicly available
recording used in evaluation. The software is named in memory of Claudio Segovia.

## References
