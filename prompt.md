# Start here — handoff for the next agent

You are picking up **Segovia**: a lazy-evaluated, chunked, concurrent **Rust compute engine for
massive multi-channel electrophysiology time-series** (Neuropixels-scale), exposed to Python via
PyO3, integrating with the SpikeInterface ecosystem over SpikeGLX / Zarr / NWB. The name honors
**Claudio Segovia** (a friend, in memoriam) and the Aqueduct of Segovia — a continuous stream
carried across a row of segmented stone arches, the metaphor for chunked, span-by-span streaming.

## Read these first (in order)

1. `CLAUDE.md` — how to work with this user. **Non-negotiable:** present every decision that is
   theirs as an interactive arrow-key question (AskUserQuestion) with a recommended option; always
   ask, never assume; work sequentially and confirm before large changes. Also: no code comments
   of any kind, conventional commits, no AI attribution, Windows-first (PowerShell, no `&&`).
2. `rust-neuro-research.md` — the fact-checked research dossier this project is founded on (deep
   multi-agent web research + prior-art sweep + language/CPU-vs-GPU reality check). Treat as the
   source of truth for the problem space; do not re-research from scratch.
3. `docs/architecture/` — the architecture document set:
   - `ARD.md` — requirements, NFRs, risks, decisions
   - `candidate-architectures.md` — 4 options with pros/cons + recommendation
   - `tech-stack.md` — concrete crate choices and their sharp edges
   - `roadmap.md` — 12-month plan and the M2–4 go/no-go benchmark gate
   - `adr/0001`–`0008` — Architecture Decision Records (0008 = domain-neutral core + verticals)
4. `docs/future/leukemia-direction.md` — the single-cell (leukemia-relevant) vertical on the
   domain-neutral core (the "90/10"). **Status update (2026-06-09):** the maintainer is running the
   multi-agent deep-research **tonight** to justify the vertical and choose the realization path
   (B SingleRust-as-dependency vs C native vertical vs E interop — to be decided by the findings, NOT
   pre-committed; SingleRust's in-memory/disk-backed model may clash with the out-of-core core).
   The maintainer intends to build the vertical and **will only abandon it on ABSOLUTE 95%+
   disconfirming evidence**. Keep `segovia-core` domain-neutral; keep the deep-research honest (let it
   find disconfirming evidence — do not rig it to confirm).

## Confirmed decisions (do not relitigate without the user)

- **Packaging:** standalone `segovia` crate + thin PyO3 wheel. A Polars plugin is optional/later. (ADR-0007)
- **M2–4 benchmark gate (make-or-break):** the full chain **bandpass + CMR + whiten together** must
  run in **< 2 GB memory** AND beat `spikeinterface(n_jobs=N)` on Windows/macOS. If it fails, the
  project premise is invalid — reconsider. Protect this experiment; build nothing heavy before it.
- **CPU, not GPU** (workload is IO/memory-bound, ~22 MB/s per probe). (ADR-0002)
- **Reuse storage** (`zarrs`, `hdf5-metno`, `arrow-rs`); SpikeGLX + Zarr first, HDF5-NWB deferred. (ADR-0003, 0005)
- **The Rust win** is GIL-released shared-memory threads (Rayon) vs SpikeInterface's
  process-pool/pickle/per-process-copy model. Biggest risk = "differentiation collapse" — prove the
  win early. (ADR-0004, 0006)

## Still open (ask the user via the picker before the relevant milestone)

- **OD3** — lazy graph design: eager-chunked iterator vs deferred operation graph (currently phased:
  eager first, modest graph in M4–7).
- **OD4** — public API naming and the SpikeInterface integration surface.

## ⭐ PRIORITY #1 — run the leukemia deep-search (do this before scaffolding)

This is the maintainer's explicit top priority. Before any code or repo scaffolding, run the
multi-agent deep-research to confirm — to a **95% bar** — whether the single-cell (leukemia-relevant)
vertical is justified or whether the maintainer is wrong about it. The decision rule is binary and
honest: **only abandon the vertical on ABSOLUTE 95%+ disconfirming evidence**; otherwise it proceeds.
Do not rig the search to confirm — let it find disconfirming evidence.

**Scoped research question to run:**
> Where, if anywhere, is there an unmet high-performance / out-of-core gap in leukemia-relevant
> single-cell computing (scRNA-seq, clonal-evolution / CAR-T analysis pipelines) that a domain-neutral
> `segovia-core` could fill? Specifically: is **SingleRust** a dependency to build on, a model to learn
> from, or a competitor that already closes the gap? Assess SingleRust / Scarf / BPCells /
> rapids-singlecell / scanpy+AnnData against the out-of-core, bounded-memory, GIL-released-threading
> thesis. Return the 95%-confidence verdict: is the vertical justified, and via which path
> (B = SingleRust-as-dependency, C = native vertical, E = interop)?

Run it via the **deep-research** workflow (`Workflow({ name: "deep-research", args: <the question above> })`).
The path (B/C/E) is decided by the findings, NOT pre-committed — SingleRust's in-memory/disk-backed
model may clash with the out-of-core core. Full options + prior-art table in
`docs/future/leukemia-direction.md`. Write the verdict back into that file when done.

## Then — suggested next steps (confirm with the user first)

1. **Scaffold the repo** wired to the standalone-crate + PyO3 shape: `ROADMAP.md` (the single source
   of truth for version/scope), `Cargo.toml`, `pyproject.toml`, `maturin` config, `src/` skeleton,
   `.gitignore`, `.venv`. No logic yet.
2. **`git init` + first commit** (no remote needed) so the architecture docs are versioned from day one.
3. Then **M0–2** of the roadmap: domain learning, SpikeGLX `.bin`/`.meta` + Zarr readers, and a
   day-1 maturin wheel spike on Windows to size the HDF5/wheel pain.

## Note on memory

A project memory was saved under the old folder path (`C:\Projects\cyber\memory\`). Because the
folder was renamed, that memory may not auto-load here. The durable record is these in-repo files —
trust them. The user has prior experience shipping this exact stack (Rust + PyO3 + maturin + PyPI)
from their MaskOps project; the toolchain is familiar territory for them.

---

## Brand & launch assets (already created)

In `assets/` and `docs/brand/`:
- `segovia-cover.svg/.png` (1200×630) — blog/LinkedIn article cover.
- `segovia-feed.svg/.png` (1080×1080) — square social feed image.
- `segovia-social.png` (1280×640) — **GitHub repo social preview** (Settings → General → Social preview).
- `logo_render.py` — Pillow renderer for all sizes (this machine has no cairo, so SVG→PNG converters
  fail; use this). Copy a `render(...)` call and scale coords for a new size.
- `docs/brand/visual-identity.md` — the brand rule book (palette, motif, typography, a regeneration prompt).
- `assets/draft_linkedin_es.md` — launch article + teaser (Spanish), with the LinkedIn **SEO title**
  and **SEO description** fields filled in.

## Prepare the GitHub page — SEO-first, from day one (do NOT defer this)

The user wants the repository discoverable and polished from the first commit — badges, README SEO,
metadata — not bolted on later. When scaffolding, set all of this up.

### Repository metadata (GitHub Settings + first push)
- **Name:** `segovia`.
- **Description (keyword-rich, ~1 line):** e.g. *"Segovia — a fast, chunked, memory-bounded Rust
  engine for electrophysiology (Neuropixels) signal processing, with Python bindings."*
- **Topics/tags** (GitHub "About" → topics; these drive GitHub search): `rust`, `python`, `pyo3`,
  `neuroscience`, `electrophysiology`, `neuropixels`, `spike-sorting`, `spikeinterface`,
  `signal-processing`, `time-series`, `zarr`, `nwb`, `dsp`, `open-source`, `scientific-computing`.
- **Social preview image:** upload `assets/segovia-social.png`.
- **Website** field: point to the docs site / crates.io / PyPI once they exist.
- Enable Issues + Discussions; add a few "good first issue"s after MVP.

### README.md — SEO-friendly structure
GitHub READMEs are indexed by Google and by GitHub search, so weave the keywords into prose (not just
tags). Suggested order:
1. **H1 + one-line tagline** with primary keywords: `# Segovia` / *"A fast, memory-bounded Rust engine
   for electrophysiology signal processing — Neuropixels-scale, callable from Python."*
2. **Badges row** (see below).
3. **The social image** (`assets/segovia-cover.png`) right under the title.
4. **What it is / why** — 2–3 sentences naming Rust, electrophysiology, Neuropixels, SpikeInterface,
   bounded memory, the GIL-released-threads advantage.
5. **Install:** `pip install segovia` (and `cargo add segovia`).
6. **Quickstart** — the 10-line Python example (read SpikeGLX → bandpass+CMR+whiten → NumPy).
7. **Features**, **Benchmarks** (the SC1 gate result once it exists), **Roadmap** (link
   `docs/architecture/roadmap.md`), **Architecture** (link `docs/architecture/`), **Contributing**,
   **Citation**, **License**.
8. Natural-language keyword coverage: "spike sorting preprocessing", "out-of-core", "Zarr/NWB",
   "Python bindings", "real-time-capable".

### Badges (shields.io — add as services come online; use placeholders until then)
- CI status (GitHub Actions), `crates.io` version, `docs.rs`, **PyPI version**, **PyPI downloads**,
  **license (dual MIT/Apache-2.0)**, Rust edition/MSRV, "PRs welcome". Optional later: codecov,
  a JOSS DOI badge if a paper is published.

### Files to create for a credible OSS + scientific project
- **`LICENSE-MIT` + `LICENSE-APACHE`** — dual-license (Rust ecosystem convention); note it in README + `Cargo.toml`.
- **`CITATION.cff`** — *important for a neuroscience tool*: makes the repo citable (GitHub shows a
  "Cite this repository" button), which drives academic adoption and backlinks. Fill author + title +
  keywords; add a DOI later (Zenodo release archiving is free).
- **`README.md`**, **`ROADMAP.md`** (single source of truth for version/scope), **`CHANGELOG.md`**
  (Keep-a-Changelog style), **`CONTRIBUTING.md`**, **`CODE_OF_CONDUCT.md`**, **`SECURITY.md`**.
- **`.github/`**: CI workflow (cargo test + clippy + fmt), a maturin wheel-build matrix
  (Windows-first; remember the MaskOps Ubuntu+3.12 `dlopen` caution), issue/PR templates.
- **Keywords in package manifests** (these surface on crates.io / PyPI search):
  - `Cargo.toml`: `description`, `keywords = ["neuroscience","electrophysiology","neuropixels","dsp","signal-processing"]`,
    `categories`, `repository`, `license = "MIT OR Apache-2.0"`.
  - `pyproject.toml`: `description`, `keywords`, PyPI `classifiers` (Topic :: Scientific/Engineering,
    Intended Audience :: Science/Research), `project.urls`.
- Reserve the names early: claim `segovia` on **crates.io** and **PyPI** with a 0.0.x placeholder so
  the name isn't taken (confirm with the user first — publishing is a production action).

### Discoverability beyond GitHub (low effort, high payoff)
- A short entry in the SpikeInterface ecosystem / "awesome-neuroscience"-style lists once there's an MVP.
- Consider a JOSS (Journal of Open Source Software) paper after v1 for academic citations.
- Keep the README keywords aligned with the LinkedIn SEO title/description in `assets/draft_linkedin_es.md`.

> Remember the working style (CLAUDE.md): present these as interactive decisions where the user has a
> real choice (name reservation, license, badge set), recommend a default, and never assume.
