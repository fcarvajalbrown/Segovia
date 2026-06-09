# Segovia — Architecture Requirements Document (ARD)

| | |
|---|---|
| **Project** | Segovia — a chunked, lazy, concurrent Rust compute engine for electrophysiology time-series |
| **Status** | Draft v0.1 — pre-implementation |
| **Author** | Software architect (working doc) |
| **Founded on** | `rust-neuro-research.md` (fact-checked research dossier) |
| **Audience** | The solo maintainer; future contributors; potential SpikeInterface integrators |

---

## 1. Problem statement

Modern extracellular electrophysiology records from thousands of channels at 30 kHz. A single
384-channel Neuropixels 1.0 probe produces **>80 GB/hour (~22 MB/s)**; high-density rigs reach
**~3.5 TB/hour**. The dominant tooling (SpikeInterface, MNE, Neo) is Python-based and, despite
being lazy/chunked, escapes the GIL only through **process pools** — which on Windows/macOS pay
`spawn` cost: fresh interpreters, pickle serialization, and **full per-process memory copies**.
Real, documented failures include a **26.2 GiB MemoryError** on a modest 32-channel 1-hour
export and a **102 GiB** blow-up during motion correction.

The opportunity, confirmed by a prior-art sweep, is a **genuinely open niche**: there is no Rust
compute engine in this space. Storage is already solved in Rust (`zarrs`, `hdf5-metno`); the
missing layer is **lazy, chunked, concurrent compute** (filtering, resampling, whitening,
detection) over standard formats, with a zero-copy Python bridge.

## 2. Goals

- **G1.** Process Neuropixels-scale recordings in **bounded, predictable memory** regardless of
  recording length (out-of-core / streaming).
- **G2.** Beat the Python incumbent on the operations that matter, primarily by using **true
  shared-memory threads (Rayon) with the GIL released** instead of process pools.
- **G3.** Be **adoptable from Python** — researchers keep writing Python; Segovia is a fast
  backend, not a new platform to learn.
- **G4.** Interoperate with the existing ecosystem (SpikeInterface, NWB/Zarr) rather than
  competing with it.
- **G5.** Ship cross-platform wheels (Windows-first) installable with `pip install segovia`.

## 3. Non-goals (explicitly out of scope)

- **N1. GPU compute.** The preprocessing workload is IO/memory-bound; PCIe transfer dominates any
  GPU filter offload. GPU only wins inside a GPU-resident spike sorter — a different project.
- **N2. A novel spike-sorting algorithm.** That is entrenched Python/MATLAB algorithmic IP and
  requires neuroscience domain depth the maintainer does not have. Segovia may *feed* sorters and
  do threshold-based *detection*, but does not invent sorting.
- **N3. Reimplementing storage formats.** Build on `zarrs` / `hdf5-metno` / `arrow-rs`; do not
  write a new Zarr or HDF5 stack.
- **N4. A GUI / dashboard.** Visualization (`nigui`, `nwbview`) is adjacent and already broken
  ground; out of scope for v1.

## 4. Stakeholders

| Stakeholder | Interest |
|---|---|
| Solo maintainer | Achievable 12-month solo scope; leverages strong Rust + existing PyO3/Polars/maturin experience (MaskOps). |
| Systems neuroscientists | Faster, memory-safe preprocessing callable from existing Python pipelines. |
| SpikeInterface community | A high-performance backend/extractor that plugs in without disruption. |

## 5. Functional requirements

| ID | Requirement | Priority |
|---|---|---|
| FR1 | Read SpikeGLX `.bin`/`.meta` recordings as a chunked, lazily-iterable signal source. | Must |
| FR2 | Read Zarr-backed arrays via `zarrs` as a signal source. | Must |
| FR3 | Streaming **bandpass filter** (IIR/FIR) producing bounded-memory output. | Must (MVP) |
| FR4 | Common-median / common-average referencing (CMR/CAR). | Should |
| FR5 | Whitening. | Should |
| FR6 | Resampling / decimation. | Should |
| FR7 | Threshold-based spike **detection** (not sorting). | Could |
| FR8 | A **lazy operation graph** — operations compose without materializing intermediates. | Must |
| FR9 | Python API returning **zero-copy** NumPy/Arrow where possible. | Must |
| FR10 | Read HDF5-backed NWB via `hdf5-metno` (raw HDF5, no schema awareness). | Could (deferred) |
| FR11 | Expose a SpikeInterface-compatible recording/preprocessing interface. | Should |

## 6. Non-functional requirements

| ID | Requirement | Target |
|---|---|---|
| NFR1 | **Bounded memory** | Peak RSS < 2 GB for a 1-hour 384-ch recording at default chunking, independent of total length. |
| NFR2 | **Throughput** | Sustain ≥ real-time per probe (≥ 22 MB/s) for the full MVP chain on a laptop; target multiple-× real-time multi-core. |
| NFR3 | **Speed vs incumbent** | Measurably faster than `spikeinterface(n_jobs=N)` for the MVP chain on Windows/macOS (see §8). |
| NFR4 | **Platforms** | Windows (primary), Linux, macOS. cp38–cp313 wheels. |
| NFR5 | **Safety** | No `unsafe` in the public compute path except audited zero-copy FFI boundaries. No data races (compiler-enforced). |
| NFR6 | **Install** | `pip install segovia` with no Rust toolchain required by the user. |
| NFR7 | **Determinism** | Identical output across thread counts (filtering must not depend on chunk scheduling). |

## 7. Constraints & assumptions

- **C1.** Solo developer, full-time, ~12 months, ~$0 budget — scope must reach a useful MVP and
  a defensible benchmark within that envelope.
- **C2.** Strong Rust; **no neuroscience background** — favor systems problems over algorithmic
  ones; budget M0–2 for domain learning.
- **C3.** **Windows-first** development machine — the incumbent's `spawn`/pickle penalty is
  largest here, which is also where Segovia's advantage is largest.
- **C4.** Prior experience shipping the *exact* stack (Rust cdylib + `pyo3` + `pyo3-polars` +
  `maturin` + PyPI) on MaskOps — adoption-path tooling risk is low.
- **A1.** Free public datasets are sufficient for development: DANDI (NWB), IBL Brain-Wide Map
  (AWS open data), Allen, plus SpikeGLX/Open Ephys sample files.

## 8. Quality attributes & success criteria

The project's existence hinges on one measurable claim. **Success criteria:**

- **SC1 (the gate).** On a real 1-hour Neuropixels recording, the Rust bandpass + CMR + whiten
  chain runs in **< 2 GB peak memory** AND is **faster than the equivalent
  `spikeinterface(n_jobs=N)` call on Windows/macOS**. *If this cannot be shown by M4, the project
  premise is invalid and scope/approach must be reconsidered* (see `roadmap.md` go/no-go gate).
- **SC2.** A neuroscientist can `pip install segovia`, point it at a SpikeGLX file, and get a
  filtered, zero-copy NumPy result in < 10 lines of Python.
- **SC3.** Reproduce the 26.2 GiB / 102 GiB documented failures running instead in bounded memory.
- **SC4.** Segovia is usable as a SpikeInterface preprocessing backend without forking SI.

## 9. Key risks (carried from the dossier)

| Risk | Severity | Mitigation |
|---|---|---|
| **Differentiation collapse** — SI + Zarr/Dask already do lazy/chunked/parallel/bounded preprocessing; the win may be only ~1.5–3×. | **High** | Prototype and benchmark the narrow win FIRST (M2–4 gate), on Windows/macOS where `spawn` hurts most, before committing the year. |
| HDF5/NWB linking pain; no schema-aware NWB reader in Rust. | Medium | Lead with SpikeGLX + Zarr; defer HDF5-NWB; use `hdf5-metno`. |
| `std::simd` is nightly-only. | Low | Workload is IO-bound; avoid hand-SIMD; use `wide`/`pulp` on stable if needed. |
| `direct-neural-biasing` (real-time Rust neuro DSP) expands into this niche. | Low–Med | Read its source early; stay focused on offline out-of-core, a distinct niche. |
| Adoption friction (labs adding a compiled wheel). | Medium | Leverage proven MaskOps wheel/CI patterns; test packaging in month 1. |

## 10. Decisions

Resolved with the maintainer:

- **OD1 — RESOLVED.** Packaging: **standalone `segovia` crate + thin PyO3 wheel** (Candidate A).
  A Polars plugin is optional/additive only — see `adr/0007` (Accepted).
- **OD2 — RESOLVED.** The M2–4 benchmark gate is anchored on the **full MVP chain (bandpass +
  CMR + whiten) together**, not a single op — the most realistic go/no-go test. See `roadmap.md`.
- **Architecture posture — RESOLVED.** Domain-neutral `segovia-core` + thin `segovia-ephys` vertical;
  a single-cell (leukemia-relevant) vertical is a *designed-for but uncommitted* future direction
  (the "90/10"). See `adr/0008` and `docs/future/leukemia-direction.md`. Build no second vertical now.

Still open (resolve before the relevant milestone):

- **OD3.** Lazy graph design: eager-chunked iterator **vs** a true deferred operation graph —
  phased per `adr/0006` (eager first, modest graph in M4–7).
- **OD4.** Public API naming and the SpikeInterface integration surface (FR11).
