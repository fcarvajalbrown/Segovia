# Rust Open-Source Neuroscience Project — Research Dossier

> **Purpose:** Decide the best 1-year+, full-time, solo, ~$0-budget open-source project for a developer
> with **strong Rust / no neuroscience background**, goal = **broad open-source impact**
> (democratizing high-performance neural data computing).
>
> **Date compiled:** 2026-06-08
> **Method:** Deep multi-agent web research (105 agents, 23 primary sources, 104 claims extracted,
> 25 adversarially fact-checked → 23 confirmed / 2 killed) + two focused follow-up investigations
> (prior-art duplication sweep; language/CPU-vs-GPU reality check).

---

## TL;DR — The Recommendation

**Build a lazy-evaluated, chunked, concurrent Rust compute engine for massive multi-channel
electrophysiology time-series, exposed to Python via PyO3, integrating as a SpikeInterface-compatible
backend over SpikeGLX / Zarr / NWB.**

- It's a **systems problem** (Rust's home turf), not a neuroscience-algorithm problem (which the profile lacks).
- It attacks **documented, reproducible pain** (float64 memory blow-ups, no out-of-core/HPC support).
- It **reuses solved storage primitives** (`zarrs`, `hdf5-metno`, `arrow-rs`) instead of reinventing them.
- The **adoption path** is a PyO3 backend plugged into the dominant Python ecosystem — researchers stay in Python.
- **The niche is genuinely open** — no shipped Rust crate/project/paper/product does this.

**Two honest reframings from the follow-up fact-check (important — the original idea overstated these):**
1. **Target CPU, not GPU; the value is bounded-memory IO-bound streaming, NOT SIMD throughput.**
2. **Rust's real edge is true GIL-released shared-memory threading**, which beats SpikeInterface's
   process-pool model — *not* raw inner-loop speed.

---

## Candidate Ranking (for THIS profile)

| Rank | Idea | Verdict |
|---|---|---|
| 🥇 | **#1 Lazy chunked compute engine (PyO3)** | Best fit. Systems problem, real pain, clear adoption path, no Rust competitor at this layer. |
| 🥈 | **#3 egui/wgpu telemetry dashboard** | Viable but weaker. Conceptual ground already broken (`nigui`, `nwbview`), narrower impact, needs live hardware. |
| 🥉 | **#2 Real-time SIMD spike sorter** | Highest risk. The hard part is neuroscience algorithm IP, not speed; real-world speedups are distributed/GPU, not single-machine SIMD. |
| ❌ | #5 `no_std` implant firmware | Ruled out — needs hardware, medical-device domain, regulatory access; impossible at $0 solo. |
| ❌ | #4 Rayon connectomics pipeline | Ruled out — petabyte EM data needs storage/compute budget not available. |

---

## Part 1 — Core Evidence (all survived 3-0 adversarial fact-check unless noted)

### 1.1 The pain is real and terabyte-scale
- A single 384-channel Neuropixels 1.0 probe at 30 kHz produces **>80 GB/hour**; high-density rigs hit
  **~3.5 TB/hour**.
- Real SpikeInterface GitHub issues show Python/NumPy float64 materialization blowing up memory:
  - **26.2 GiB MemoryError** on a *modest* 32-channel 1-hour `export_to_phy`.
  - **102 GiB** failure during motion-correction interpolation.
- SyNCoPy (Frontiers in Neuroinformatics 2024) states standard connectivity analysis "can become
  impossible to carry out on laptops" and names **lack of HPC integration and absence of
  distributed-computing support** as core gaps in existing Python tools.

Sources:
- https://iopscience.iop.org/article/10.1088/1741-2552/acf5a4
- https://github.com/SpikeInterface/spikeinterface/issues/979
- https://github.com/SpikeInterface/spikeinterface/issues/3489
- https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11614769/

### 1.2 The "format zoo" is a field-acknowledged pain point
- Neo paper (2014): proprietary vendor formats with "little in the way of common data formats."
- SpikeInterface paper: a "fragmented software ecosystem which challenges reproducibility,
  benchmarking, and collaboration."
- NWB ecosystem paper (2022): lab-specific formats are "a major impediment to sharing data";
  NWB adopted by 50+ labs.
- **NWB and Zarr are the de facto integration targets** any new Rust ephys tool must support.

Sources:
- https://www.ncbi.nlm.nih.gov/pmc/articles/PMC3930095/
- https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7704107/
- https://pmc.ncbi.nlm.nih.gov/articles/PMC9531949/

### 1.3 The incumbent ecosystem (defines the adoption path)
- **SpikeInterface** is the dominant Python unifier: reads 30–43+ formats (SpikeGLX, Open Ephys,
  Intan, Neuralynx, NWB, Zarr) and wraps **10 spike sorters: 6 Python + 4 MATLAB, ZERO Rust**.
- Implication: the adoption path for a Rust tool is a **PyO3 backend/extractor plugged into
  SpikeInterface**, NOT a competing standalone app — and there is no Rust competitor at the
  compute-engine layer.

Sources:
- https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7704107/
- https://elifesciences.org/reviewed-preprints/110170

### 1.4 Why spike sorting (idea #2) is the riskiest bet
- Six Neuropixels 2.0 Quad Base probes take >1 week serially → ~10 hours (>20×) via
  **distributed/cloud parallelization (Nextflow)**, NOT single-machine SIMD.
- All incumbent sorters are Python/MATLAB algorithmic research code; building a novel *correct*
  real-time sorter needs deep neuroscience domain knowledge the profile lacks.
- Rust's value here is credible as an *acceleration backend*, not as a new sorting algorithm.

### 1.5 Don't rebuild storage — it already exists in Rust
- **`zarrs`** — mature, actively maintained native-Rust Zarr V3 (75+ releases, Zarr 3.1 conformant,
  FFI + Python bindings; `zarrs-python` reportedly outperforms zarr-python/tensorstore).
- **`nwbview`** — Rust + egui/HDF5 NWB *viewer* (proves Rust HDF5→GUI pipeline; last release 2023, stale).
- **`nigui`** — Rust + egui real-time EEG *dashboard* (proves the viz space; ~2 stars, embryonic).
- => The genuinely-missing layer is **lazy/chunked concurrent COMPUTE** over these formats, with a
  Python bridge — NOT storage, which is largely solved.

Sources:
- https://github.com/zarrs/zarrs_tools
- https://github.com/brainhack-ch/nwbview
- https://github.com/mikelma/nigui

### Killed claims (fact-checker refuted these — do NOT rely on them)
- **REFUTED 0-3:** "NWB's Zarr backend exists only as a prototype (2022)" — Zarr support is more mature
  than this implies.
- **REFUTED 0-3:** "Issue #3489's 102 GiB blow-up directly proves the lazy-engine gap" — it was partly
  user-triggered by full-resolution resampling. Justify the project on the *broad* memory/out-of-core
  evidence, not this single issue.

---

## Part 2 — Prior-Art Duplication Sweep (Is the niche taken?)

**Method:** crates.io + lib.rs API queries (electrophysiology, neuropixels, spike sorting, neuroscience,
ephys, nwb, lfp, spikeglx, open ephys, zarr), WebSearch for academic/startup work, DSP/streaming
ecosystem, SpikeInterface GitHub. crates.io returns **0 results** for "neuropixels", "spikeglx", "open ephys".

### Rust ephys/neuro crates found
| Crate | What it does | Maturity | Class |
|---|---|---|---|
| `direct-neural-biasing` | Low-latency **real-time closed-loop** neuro feedback; PyO3 + C++ bindings | ~36k dl, active | **B — closest; but online/real-time, not out-of-core batch** |
| `ruv-neural-signal` | Filtering, spectral, artifact rejection | v0.1.0, 149 dl, Mar 2026 | **B — embryonic single-shot DSP** |
| `ruv-neural-sensor` | Sensor acquisition (NV diamond, OPM, EEG) | 65 dl | C |
| `intan_importer` | Read Intan RHS files | 3,083 dl | C (I/O only) |
| `rust_abf` | Read Axon Binary Format | 17k dl | C (I/O only) |
| `european-data-format` / `fiff` / `openbci` | EDF / MEG-EEG FIFF / OpenBCI drivers | low-mid | C (I/O) |
| `nwbview` | NWB viewer | 7,479 dl, stale 2023 | C (viz only) |

No crate does spike sorting, whitening, cross-channel connectivity, or out-of-core multichannel batch
compute. None mention Neuropixels/SpikeGLX/Open Ephys/SpikeInterface.

### General Rust streaming/compute engines (none substitute)
- **Polars** — lazy + streaming, but a *dataframe/relational* engine. No DSP (no IIR/bandpass, FFT,
  resample, whiten). Dependency candidate, not substitute. (Class C)
- **arrow-rs / DataFusion / Arroyo / SeaStreamer** — columnar/SQL/stream, no signal semantics. (C)
- **ndarray** — NumPy analog, no built-in lazy eval (open issue #1101), no out-of-core. Building block. (C)
- **rustfft / realfft / fundsp / dasp / sci-rs** — `fundsp`/`dasp` are audio-oriented; `rustfft`/`realfft`
  are FFT primitives; `sci-rs` is a SciPy port. None handle thousands of channels or ephys formats. (B/C)
- **zarrs / icechunk** — storage only, not compute. (C)

### SpikeInterface / MNE / Neo Rust accelerator? — None
SpikeInterface GitHub has no Rust/PyO3 backend issues ("rust" hits are `ruff` linting). SI's own perf
strategy is Python + memmap + numpy + joblib. No MNE/Neo Rust accelerator found.

### Academic / startup? — None
No paper or company for "Rust electrophysiology / Neuropixels / spike sorting compute engine." 2025–2026
large-scale ephys pipeline papers (Power Pixels, eLife reproducible pipelines, SimSort, MEDiCINe, Allen
ecephys) are all Python/MATLAB/CUDA on SpikeInterface/Kilosort.

### ✅ Verdict: NICHE GENUINELY OPEN (two partial-overlap flags)
No DIRECT DUPLICATE found. Strongest evidence: 0 crates.io results for the formats; SpikeInterface has no
Rust backend nor discussion of one; 2025–26 literature treats ephys compute cost as an open bottleneck
solved with Python+GPU.

**Caveats (absence-of-evidence, not proof-of-absence):**
- **Read `direct-neural-biasing` source before committing** — closest existing thing; validates the
  approach, occupies the real-time online niche, potential competitor if it expands scope.
- `ruv-neural-signal` shows others starting to circle the space.
- Closed-source/internal lab tools and very new unindexed repos can't be ruled out by web search.

Key URLs:
- https://crates.io/crates/direct-neural-biasing
- https://crates.io/crates/ruv-neural-signal
- https://crates.io/crates/zarrs
- https://crates.io/crates/sci-rs
- https://github.com/SpikeInterface/spikeinterface
- https://docs.pola.rs/user-guide/concepts/streaming/

---

## Part 3 — Language & CPU-vs-GPU Reality Check (Does it HAVE to be Rust?)

**Bottom line:** Rust is a *fine, leaning-right* choice for the specific niche of
"memory-bounded, IO-bound, parallel CPU streaming preprocessing that escapes Python's GIL" —
**NOT** "raw numerical throughput that beats GPU."

### 3.1 The decisive number: ~22 MB/s per probe
- 30 kHz × 384 ch × 2 bytes ≈ **80 GB/hour ≈ ~22 MB/s sustained per probe**.
- NVMe SSD does 3,000–7,000 MB/s; one CPU core filters far faster than 22 MB/s.
- Decompression already runs **12–71× real-time** on CPU (WavPack ~12.8×, blosc-zstd ~71×).
- SpikeInterface docs concede: "saving preprocessed recordings to disk isn't always optimal, as the
  writing to disk can be slower than recomputing the preprocessing chain on-the-fly."
- **=> The workload is overwhelmingly IO/memory-bound, not compute-bound.** Optimize for IO scheduling,
  decompression overlap, cache-friendly chunking, avoiding copies — NOT AVX-512 microkernels.

### 3.2 CPU vs GPU — GPU is mostly a trap for the *preprocessing* stage
- Kilosort's famous >20× speedups are in **spike SORTING** (iterative template-match/cluster), not
  streaming filter/whiten preprocessing.
- **PCIe kills streaming filter offload:** for GPU FIR/polyphase filtering, "the maximum sampling rate
  that can be handled is limited by PCIe bandwidth rather than the computations on the GPU."
- GPU only wins when data already lives in GPU memory and stays there for many ops (the sorter's regime;
  Kilosort4 does preprocessing per-batch on the GPU it's already sorting on).
- **Verdict: CPU is the right target for this engine.** A GPU rewrite only makes sense fused into a
  GPU-resident sorter — a different project (Kilosort's territory).

### 3.3 Rust's GENUINE win over the Python incumbent
- SpikeInterface is *already* lazy, chunked, parallel, bounded-memory (filtering via `scipy.signal`).
- To escape the GIL it uses `ChunkRecordingExecutor` + `ProcessPoolExecutor`:
  - `"fork"` only safe on Linux; **Windows/macOS fall back to `"spawn"`** → fresh interpreter + pickle
    overhead, and on **Windows, full memory copies per process**.
- A Rust engine with **Rayon + shared-memory threads + `Python::allow_threads` (GIL release)** sidesteps
  pickling, process-spawn cost, and per-process RAM duplication. **On Windows this is a concrete,
  measurable win** (and the user is on Windows). PyO3 0.28+ also supports free-threaded Python 3.14.
- **This — not inner-loop speed — is the honest, durable advantage.**

### 3.4 Language alternatives scored
- **C++** — only serious rival. Same perf ceiling, more mature FFT/BLAS/HDF5. Rust wins on
  safety-under-concurrency (chunk-scheduler data races are Rust's sweet spot) and Cargo/maturin
  build-and-ship. Reasonable Rust win.
- **Julia** — sci-computing native, ~C perf, but GC pauses awkward for bounded-memory streaming, and
  Python-interop/startup-latency weaker for a pip-installable lib. Disfavored by the deploy-into-SI requirement.
- **Zig** — fast, great C interop, but immature ecosystem, no PyO3 equivalent, no 1.0. Higher risk. Not recommended.
- **Cython / Numba / NumPy** — Numba can hit C speed for kernels but inherits GIL/multiprocessing model.
  Legitimate *baseline you must beat*.
- **Dask / Zarr+Dask** — the real "do you even need a new engine?" threat: already gives lazy, chunked,
  out-of-core, parallel arrays, and SI layers lazy preprocessing on top. **You must articulate what your
  engine does that `SpikeInterface + Zarr + Dask` does not** — honest answer: lower per-chunk overhead,
  true threads vs process pools, tighter memory bounds, fused multi-op pipelines. Real but **narrow** gap.

### 3.5 Rust numerical ecosystem — mature *enough*, with sharp edges
**Solid:** `rustfft` (SIMD AVX/SSE/Neon/WASM) + `realfft`; `ndarray` + `rayon` + `rust-numpy`
(zero-copy NumPy via `PyReadonlyArray`); `zarrs` (excellent, actively maintained — your strongest dep);
`arrow-rs` / Polars internals. Polars, tokenizers, pydantic-core, cryptography, ruff all ship
Rust-in-Python via PyO3/maturin — proven precedent.

**Sharp edges (real, survivable):**
- **`std::simd` (portable-simd) is nightly-only**, no stabilization timeline. Mitigation: you mostly don't
  need hand-SIMD (IO-bound); `wide`/`pulp` work on stable. Don't architect around portable-simd.
- **HDF5 is the real wound:** the original `hdf5` crate is abandoned (last release 0.8.1, Nov 2021).
  Use the live community fork **`hdf5-metno`** — thin volunteer wrapper over the C lib, with cross-platform
  C-linking pain (a known wheel-building headache). **NWB is HDF5-based, and there is NO schema-aware Rust
  NWB reader** — you'd use raw HDF5 and ignore schema semantics. **Lead with SpikeGLX (flat binary) +
  Zarr-backed NWB; treat HDF5-backed NWB as a later, painful add.**

### Final Verdict
- **(a) Language:** Rust is **fine, leaning right** for this niche. Stack: Rust + PyO3/maturin + Rayon +
  zarrs + rust-numpy + rustfft. Edge over Python = GIL-released shared-memory threading; edge over C++ =
  safety-under-concurrency + packaging. Not overkill, but not transformative — you're out-engineering an
  existing capability, not creating a missing one.
- **(b) CPU or GPU:** **CPU, decisively**, for the preprocessing engine (IO/memory-bound, ~22 MB/s/probe;
  PCIe dominates any GPU filter offload). GPU only inside a GPU-resident sorter = different project.
- **(c) Biggest risk:** **NOT the language — differentiation collapse.** SpikeInterface + Zarr/Dask already
  do lazy/chunked/out-of-core/parallel/bounded-memory preprocessing. If the measured win over
  `spikeinterface(n_jobs=N)` is only ~1.5–3× on an already-fast-enough workload, adoption is hard.
  **Mitigation: prototype the narrow win FIRST** — benchmark a Rust Rayon+GIL-release bandpass+CMR+whiten
  chunk pipeline vs SI process-pool preprocessing on real Neuropixels data, **on Windows/macOS where spawn
  hurts most**, before committing the year.

### Key sources
- IO-bound / data rate: https://iopscience.iop.org/article/10.1088/1741-2552/acf5a4
- SI incumbent preprocessing: https://spikeinterface.readthedocs.io/en/stable/modules/preprocessing.html
- GPU PCIe-bound filtering: https://arxiv.org/pdf/1511.03599 , https://arxiv.org/pdf/2303.09886
- Kilosort GPU = sorting: https://www.nature.com/articles/s41592-024-02232-7
- PyO3/maturin + GIL release: https://www.nandann.com/blog/rust-pyo3-python-extensions-guide
- zarrs: https://github.com/zarrs/zarrs , RustFFT: https://github.com/ejmahler/RustFFT
- portable-simd status: https://shnatsel.medium.com/the-state-of-simd-in-rust-in-2025-32c263e5f53d
- abandoned hdf5 vs fork: https://docs.rs/crate/hdf5/latest , https://github.com/metno/hdf5-rust
- Julia comparison: https://discourse.julialang.org/t/comparison-of-rust-to-julia-for-scientific-computing/78508

---

## Part 4 — Free Datasets to Develop Against (all $0)

| Source | Format | URL |
|---|---|---|
| DANDI Archive | NWB | https://about.dandiarchive.org/ |
| IBL Brain-Wide Map | Neuropixels (AWS open data) | https://registry.opendata.aws/ibl-brain-wide-map/ |
| IBL 2025 release docs | — | https://docs.internationalbrainlab.org/notebooks_external/2025_data_release_brainwidemap.html |
| Allen Neural Dynamics | NWB/Zarr | https://allenneuraldynamics.github.io/data.html |
| SpikeGLX (`.bin`/`.meta` spec) | flat binary | https://billkarsh.github.io/SpikeGLX/ |
| Open Ephys (Neuropixels-PXI) | Open Ephys | https://open-ephys.github.io/gui-docs/User-Manual/Plugins/Neuropixels-PXI.html |

---

## Part 5 — Draft 12-Month Roadmap (refine in the PRD)

**MVP north star:** *Open a 1-hour Neuropixels SpikeGLX/NWB recording, apply a bandpass filter in
constant (<2 GB) memory, callable from Python, faster than the equivalent SpikeInterface call —
benchmarked on Windows/macOS where SI's process-pool spawn hurts most.*

- **M0–2 · Learn + read.** Ephys fundamentals (spike/LFP/channel). Parse SpikeGLX `.bin`/`.meta`;
  read Zarr via `zarrs`. Validate against free IBL/DANDI data.
  **Also: read `direct-neural-biasing` source; prototype the PyO3/maturin wheel build on Windows day 1.**
- **M2–4 · Core MVP + prove the win.** Chunked, memory-bounded reader + bandpass + CMR + whiten chunk
  pipeline with Rayon + GIL release. **Benchmark vs `spikeinterface(n_jobs=N)` on Windows/macOS; prove
  constant memory AND a real speed/overhead win.** ← go/no-go gate for the whole year.
- **M4–7 · Python bridge.** PyO3 bindings returning NumPy/Arrow zero-copy; lazy operation graph
  (map/filter/resample).
- **M7–10 · Concurrency + detection.** Threshold-based spike *detection* (not full sorting),
  cross-channel ops with Rayon.
- **M10–12 · Ship for adoption.** Package as a SpikeInterface preprocessing backend/extractor; publish
  crate + PyPI wheels; write a benchmark post reproducing the 26.2/102 GiB failures now in bounded memory.
  Lead formats: SpikeGLX + Zarr; HDF5-NWB later via `hdf5-metno`.

---

## Part 6 — Open Questions to Resolve in the PRD

1. **Which single MVP operation yields the highest adoption pull and clearest benchmark win?**
   (filtering vs motion correction vs whitening vs connectivity vs resampling) — pick this as the M2–4 op.
2. **Will labs actually add a compiled Rust wheel** to their Python pipeline? Test cross-platform wheel +
   HDF5 linking friction in month 1, not month 11.
3. **Is the measured win over `SpikeInterface + Zarr/Dask` big enough** to justify a new engine? (the
   differentiation-collapse risk). Validate at the M2–4 gate.
4. **Does `direct-neural-biasing` (or any unindexed/internal tool) overlap** more than the web search
   could see? Read its source directly.

---

## Appendix — Research Provenance

- Deep-research workflow: 105 agents, 23 sources fetched, 104 claims extracted, 25 verified
  (3-vote adversarial, need 2/3 to confirm), 23 confirmed / 2 killed, 7 findings after synthesis.
- Follow-up agent A: prior-art duplication sweep (crates.io/lib.rs/GitHub/PyPI + academic).
- Follow-up agent B: language + CPU-vs-GPU reality check.
- All structural facts (formats, data volumes, memory errors, existing crates) backed by peer-reviewed
  papers or primary GitHub/registry sources. The final recommendation is a synthesis/judgment built on
  those facts, not itself a verified empirical claim.
