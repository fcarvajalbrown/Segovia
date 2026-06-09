# Segovia — 12-Month Architecture Roadmap

Adapted from the dossier roadmap. The pivotal event is the **M2–4 benchmark go/no-go gate**:
everything before it exists to answer one question — *is Segovia measurably better than the
Python incumbent on the operations that matter?* Do not build the full engine before answering it.

> `ROADMAP.md` (repo root) is the single source of truth for version/scope once the project
> starts. This file is the architecture-level plan that seeds it.

---

## M0–2 · Learn + read + de-risk tooling

- Learn ephys fundamentals (spike / LFP / channel / referencing / whitening — enough to build correctly).
- Parse SpikeGLX `.bin`/`.meta`; read Zarr via `zarrs`. Validate against **free** IBL / DANDI data.
- **Read `direct-neural-biasing` source** to confirm the niche is still open.
- **Day-1 tooling spike:** stand up a `maturin` wheel build on Windows returning a zero-copy NumPy
  array from Rust. Surface the HDF5/wheel pain now, not in month 11.
- **Deliverable:** a chunked, memory-bounded reader that streams a 1-hour SpikeGLX recording in
  bounded memory and hands chunks to Python.

## M2–4 · Core MVP + prove the win  ← GO/NO-GO GATE

- Build **Candidate D** (thin Rayon-over-chunks pipeline) for the MVP chain:
  **bandpass filter → CMR → whiten**, with the GIL released.
- Benchmark vs `spikeinterface(n_jobs=N)` **on Windows/macOS** (where `spawn`/pickle hurts most).
- **Gate (SC1):** must show **< 2 GB peak memory** AND a **clear speed/overhead win**.
  - ✅ Pass → proceed to M4–7.
  - ❌ Fail → STOP and reconsider: the differentiation-collapse risk has materialized. Re-scope
    (different op? GPU-resident sorter? abandon?). **Do not sink the year on faith.**
- **Deliverable:** reproducible benchmark + the MVP north star: *open a 1-hour Neuropixels
  SpikeGLX recording, bandpass-filter it in constant (<2 GB) memory, callable from Python, faster
  than the equivalent SpikeInterface call.*

## M4–7 · Refactor to a real engine + Python API

- Refactor D → **Candidate A**: standalone `segovia` crate + thin `segovia-py` PyO3 layer.
- Add a **modest lazy operation graph** (FR8): compose ops without materializing intermediates.
  No full optimizer yet (defer Candidate C).
- Zero-copy NumPy/Arrow outputs (FR9).
- **Deliverable:** `pip install`-able package; a 10-line Python script does read → filter → result.

## M7–10 · Breadth: ops + concurrency hardening

- Add resampling/decimation, threshold-based spike **detection** (FR7 — detection, not sorting),
  cross-channel ops.
- Deterministic output across thread counts (NFR7); cross-chunk filter-state correctness.
- Optional: begin HDF5-NWB read via `hdf5-metno` (FR10) if demand/time allows.
- **Deliverable:** a small but real op library with correctness + memory tests on free datasets.

## M10–12 · Ship for adoption

- Package as a **SpikeInterface preprocessing backend / extractor** (FR11) — integrate, don't compete.
- Publish the crate (crates.io) + PyPI wheels (Windows/Linux/macOS, cp38–cp313).
- Write the launch artifact: a benchmark post reproducing the **26.2 GiB / 102 GiB** documented
  failures now running in bounded memory (SC3).
- **Deliverable:** v0.x public release; reproducible benchmarks; SpikeInterface integration demo.

---

## Milestone summary

| Phase | Months | Focus | Architecture | Exit criterion |
|---|---|---|---|---|
| Learn + de-risk | 0–2 | Domain + readers + tooling | (readers only) | Bounded-memory chunk reader |
| **Prove the win** | **2–4** | **MVP chain + benchmark** | **D (thin Rayon)** | **SC1 gate passes** |
| Real engine | 4–7 | Lazy graph + Python API | A (crate + PyO3) | `pip install` + 10-line demo |
| Breadth | 7–10 | More ops + correctness | A | Op library + tests |
| Ship | 10–12 | SI backend + release | A (+ optional B) | Public release + benchmarks |

## The one rule

**Nothing downstream of M4 matters if the M2–4 gate fails.** Protect that experiment: keep the
MVP chain small, benchmark honestly, and be willing to stop.

---

## Future verticals (PENDING — not in the 12-month scope)

Segovia is built as a domain-neutral `segovia-core` + the `segovia-ephys` vertical (ADR 0008). A
**single-cell genomics vertical** — the computational backbone of modern leukemia research — is an
anticipated but **uncommitted** future direction (the "10%"). It is *not* part of this 12-month plan;
only the core's seams are kept clean to make it possible later. Findings, options, and pending tasks
live in `docs/future/leukemia-direction.md`. Guardrail: build no second vertical or speculative
abstraction now (YAGNI).
