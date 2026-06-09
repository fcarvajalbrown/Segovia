<p align="center">
  <img src="assets/segovia-cover.png" alt="Segovia — a fast, memory-bounded Rust engine for electrophysiology signal processing, Neuropixels-scale, callable from Python" width="820"/>
</p>

<p align="center">
  <a href="https://github.com/fcarvajalbrown/Segovia/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/fcarvajalbrown/Segovia/ci.yml?style=flat-square&label=CI&color=CE422B" alt="CI"/></a>
  <a href="https://crates.io/crates/segovia"><img src="https://img.shields.io/crates/v/segovia?style=flat-square&color=CE422B&logo=rust&logoColor=white" alt="crates.io"/></a>
  <a href="https://pypi.org/project/segovia/"><img src="https://img.shields.io/pypi/v/segovia?style=flat-square&color=CE422B&logo=pypi&logoColor=white" alt="PyPI"/></a>
  <a href="https://docs.rs/segovia"><img src="https://img.shields.io/docsrs/segovia?style=flat-square&color=CE422B" alt="docs.rs"/></a>
  <a href="#license"><img src="https://img.shields.io/badge/license-AGPL--3.0-CE422B?style=flat-square" alt="License: AGPL-3.0-or-later"/></a>
  <a href="#status"><img src="https://img.shields.io/badge/status-pre--release%20(pre--MVP)-CE422B?style=flat-square" alt="Status: pre-release"/></a>
  <a href="CONTRIBUTING.md"><img src="https://img.shields.io/badge/PRs-welcome-CE422B?style=flat-square" alt="PRs welcome"/></a>
</p>

> A fast, chunked, **memory-bounded Rust engine for electrophysiology** signal processing — Neuropixels-scale, callable from Python.

**Segovia** is a lazy-evaluated, chunked, concurrent compute engine for massive multi-channel
electrophysiology time-series (Neuropixels-scale: 30 kHz × thousands of channels). It is written in
**Rust**, exposed to **Python** via [PyO3](https://github.com/PyO3/pyo3), and built to slot into the
existing neuroscience stack — **SpikeInterface**, **SpikeGLX**, **Zarr**, and **NWB** — rather than
replace it. The aim is **out-of-core, bounded-memory streaming preprocessing** (bandpass filtering,
common-median referencing, whitening) with **GIL-released shared-memory threads** instead of the
process-pool / pickle / per-process-copy model that makes Python spike-sorting pipelines run out of
memory.

## Status

**Early development — pre-MVP (v0.1.0).** The first functional piece has shipped: a chunked,
memory-bounded **SpikeGLX `.meta`/`.bin` reader** (`segovia.SpikeGlxReader`), published to
[crates.io](https://crates.io/crates/segovia) and [PyPI](https://pypi.org/project/segovia/) at
v0.1.0 — `pip install segovia` works. The compute engine (the **bandpass → CMR → whiten** chain) is
**not built yet**, so parts of the quickstart below still describe the **target** API. The whole
premise rests on one make-or-break benchmark — see [The benchmark gate](#the-benchmark-gate). Follow
the [roadmap](ROADMAP.md) for progress.

## Contents

- [Why Segovia](#why-segovia)
- [How it works](#how-it-works)
- [Install](#install)
- [Quickstart](#quickstart)
- [The benchmark gate](#the-benchmark-gate)
- [Architecture](#architecture)
- [Roadmap](#roadmap)
- [Why the name](#why-the-name)
- [Contributing](#contributing)
- [Citation](#citation)
- [License](#license)

## Why Segovia

A neuroscience lab can record a brain faster than its software can read it back. A single
high-density Neuropixels probe writes roughly **80 GB/hour (~22 MB/s)**; standard Python pipelines
load that at double size and then copy it wholesale into every worker process. Documented failures
include a **26 GiB** memory error filtering a modest recording and a **102 GiB** blow-up during
motion correction. The data is fine — the plumbing leaks.

Segovia targets that plumbing. It is **CPU-first** (the workload is IO/memory-bound, so a GPU would
spend more time waiting on the PCIe bus than computing), reuses mature Rust storage crates
(`zarrs`, `hdf5-metno`, `arrow-rs`) instead of reinventing them, and earns its keep through one
concrete advantage: **true shared-memory threading in Rust with the GIL released**. This is
**out-of-core spike-sorting preprocessing** — bounded memory regardless of recording length, real-time
capable, and callable from the Python tools researchers already use.

## How it works

```mermaid
flowchart LR
    A["Storage<br/>SpikeGLX .bin · Zarr · NWB/HDF5"] --> B["Chunked source<br/>channels × samples tiles"]
    B --> C["Op chain<br/>bandpass → CMR → whiten → detect"]
    C --> D["Sink<br/>zero-copy NumPy / Arrow"]
    C -. "Rayon over chunks, GIL released" .-> C
    style A fill:#0B1020,stroke:#5A6B8C,color:#F5F7FA
    style B fill:#0B1020,stroke:#5A6B8C,color:#F5F7FA
    style C fill:#0B1020,stroke:#CE422B,color:#F5F7FA
    style D fill:#0B1020,stroke:#DEA584,color:#F5F7FA
```

Data is read in **chunks** (spans of channels × samples), streamed through an operation chain, and
returned to Python **zero-copy**. Only a bounded window is ever resident in memory — the metaphor is
the **Aqueduct of Segovia**, a continuous stream carried span-by-span across a row of stone arches.

## Install

> Not yet published — this is the planned install once the first release ships.

```bash
pip install segovia
```

```bash
cargo add segovia
```

## Quickstart

> Target API (illustrative, not yet shipped). Read a SpikeGLX recording, run the
> bandpass → common-median-reference → whiten chain in bounded memory, and get a zero-copy NumPy result.

```python
import segovia

recording = segovia.read_spikeglx("data/probe0.imec0.ap.bin")

filtered = (
    recording
    .bandpass(low=300, high=6000)
    .common_median_reference()
    .whiten()
)

chunk = filtered.to_numpy(start=0, end=30_000)
```

## The benchmark gate

Segovia's existence hinges on one measurable claim (call it **SC1**): on a real 1-hour Neuropixels
recording, the Rust **bandpass + CMR + whiten** chain must run in **under 2 GB of peak memory** *and*
be **faster than the equivalent `spikeinterface(n_jobs=N)`** call on Windows and macOS. If that cannot
be shown, the premise is wrong and the project says so. This benchmark is built **first**, not last.
**Result: pending** — see the [roadmap](ROADMAP.md).

## Architecture

The full architecture document set lives in [`docs/architecture/`](docs/architecture/):

- [`ARD.md`](docs/architecture/ARD.md) — requirements, NFRs, risks, decisions.
- [`candidate-architectures.md`](docs/architecture/candidate-architectures.md) — four options, trade-offs, recommendation.
- [`tech-stack.md`](docs/architecture/tech-stack.md) — concrete crate choices and their sharp edges.
- [`roadmap.md`](docs/architecture/roadmap.md) — the milestone-level plan.
- [`adr/`](docs/architecture/adr/) — Architecture Decision Records.

## Roadmap

[`ROADMAP.md`](ROADMAP.md) is the single source of truth for version and scope. In short: learn the
domain and de-risk the toolchain (M0–2), **prove the benchmark win** (M2–4, the go/no-go gate), grow
into a real engine with a Python API (M4–7), add breadth and correctness (M7–10), and ship as a
SpikeInterface preprocessing backend (M10–12). A deferred, gated single-cell vertical sits beyond
that — see [`docs/future/leukemia-direction.md`](docs/future/leukemia-direction.md).

## Why the name

Segovia is named for **Claudio Segovia**, a friend who died of leukemia at 26. The name also evokes
the **Aqueduct of Segovia** — a continuous stream carried across a long row of segmented stone arches,
which is exactly this engine's chunked, span-by-span streaming model.

The connection is honest, not a marketing claim. An electrophysiology engine does not cure cancer, and
saying otherwise would be dishonest. But the underlying computational problem — data too large for
memory, and a Python layer that copies it until it chokes — is shared with **single-cell genomics**,
the computational backbone of modern leukemia research (clonal evolution, drug resistance, CAR-T).
Segovia's core is kept **domain-neutral** so the same machinery could one day help with that work too:
**aided by the tool, not a tool made for it.** That direction is deliberately deferred and gated —
the honest details, including disconfirming evidence, are in
[`docs/future/leukemia-direction.md`](docs/future/leukemia-direction.md).

## Contributing

Contributions are welcome — see [`CONTRIBUTING.md`](CONTRIBUTING.md). The project is Windows-first,
uses a Rust + PyO3 + maturin toolchain, conventional commits, and STAR-format PRs.

## Citation

If you use Segovia in your research, please cite it via [`CITATION.cff`](CITATION.cff) (GitHub shows a
"Cite this repository" button). A DOI will be added on the first archived release.

## License

Segovia is licensed under the **GNU Affero General Public License v3.0 or later**
([AGPL-3.0-or-later](LICENSE)).

This is deliberate: Segovia is free for everyone — researchers, individuals, and non-profits — and the
copyleft terms keep it that way. Anyone who distributes Segovia, or runs a modified version as a
network service, must release their complete corresponding source under the same license, so the
project cannot be taken closed-source or proprietary.

Unless you explicitly state otherwise, any contribution you submit for inclusion is licensed under
AGPL-3.0-or-later, without any additional terms or conditions.
