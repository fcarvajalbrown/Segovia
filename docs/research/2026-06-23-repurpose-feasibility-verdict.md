# Repurpose Feasibility Verdict — Single-Cell Genomics and Non-Ephys Neuro-Signals

**Date:** 2026-06-23
**Run:** `wf_3742ffbf-68a` (deep-research; 5 angles, 21 sources fetched, 94 claims extracted, 25 verified 3-vote adversarial, 22 confirmed / 3 killed, 103 agents, ~2.8M tokens)

## Question

Can Segovia's transferable core — Rust + PyO3 bindings to Python, AGPL-3.0 license, and
bounded-memory chunked streaming with GIL-released (Rayon) threading — be repurposed to a NEW
domain with a genuinely binding, documented, RECURRING out-of-core / bounded-memory / streaming-IO
pain and NO mature incumbent? Two candidate domains:

- **(A)** Single-cell genomics / the leukemia arc (scRNA-seq, scATAC-seq, h5ad/zarr/10x).
- **(B)** Non-electrophysiology neuro-signals (two-photon / widefield calcium imaging, EEG/MEG, miniscope).

Success bar (PASS only if ALL hold): pain is binding, documented, recurring, AND no mature
incumbent. Constraints: solo-maintainable; Rust+PyO3→Python; AGPL-compatible; value is
bounded-memory streaming I/O (not GPU compute, not a rival framework — must be an adopted
component/backend). Adversarial default: NO-GO unless binding unmet demand is evidenced.

## Verdict

- **Domain (A): NO-GO** — affirmatively closed. The niche is occupied by scverse itself.
- **Domain (B): NO-GO** — by adversarial default. No qualifying unmet sub-niche surfaced;
  the one concrete signal was a single localized bug, and two adjacent framings were refuted.

## Domain (A) — Single-cell genomics: NO-GO (affirmatively closed)

Segovia's exact value proposition (bounded-memory, out-of-core, lazy chunked streaming I/O;
Rust core + PyO3 bindings) is **already shipped by mature, institutionally-backed incumbents** —
and most fatally, by the exact `scverse/anndata-rs` stack the question named as the key lead.

- **anndata-rs** (3-0): scverse-co-owned (Philipp A., Kai Zhang, scverse org), out-of-core
  AnnData in Rust, PyO3 bindings (`pyanndata`, pyo3 0.27), separate `anndata-hdf5` + `anndata-zarr`
  backend crates, production inside SnapATAC2 (Nature Methods 2024, >10M cells). `anndata_rs-v0.6.0`
  released **2026-06-22** — the day before this assessment. This IS Segovia's stack, already in
  production. MIT-licensed → would be plugged *into*, not displaced. No open niche.
- **annbatch** (3-0): CZI-EOSS-funded, Lamin Labs + scverse, terabyte-scale out-of-core minibatch
  streaming dataloader; delegates performance-critical local-FS reads to Rust via `zarrs-python`.
  v0.2.0 released 2026-06-12. The streaming-loader seam is filled.
- **BPCells + Scarf** (3-0): BPCells (C++, ~70x memory reduction, 44M cells on a laptop, backend
  of Seurat v5 and monocle3); Scarf (Nature Comms 2022, 4M cells end-to-end in <16GB). Both occupy
  the bounded-memory streaming niche — Scarf even closes the streaming-*compute* seam, not just I/O.
- **TileDB-SOMA** (3-0): deployed cross-language (Python 1.0 + R) out-of-core engine under CZ
  CELLxGENE Census (65M+ cells, 900+ datasets, peer-reviewed NAR). Also covers the interop /
  format-conversion sub-niche (Seurat / AnnData-Scanpy / Bioconductor).
- **anndata 0.12** (3-0): official `read_lazy` / `read_elem_lazy` lazy out-of-core access, local +
  remote, with Rust acceleration via `obstore` and `zarrs-python` co-developed by scverse maintainers.

**Decisive license clash:** scverse/AnnData is BSD; anndata-rs / anndata-zarr are MIT. An AGPL
Segovia component is **structurally non-upstreamable** into the incumbent stack — it could only ever
be an external AGPL dependency, which fails the "adoptable as a component inside an existing tool"
requirement at the root.

## Domain (B) — Non-ephys neuro-signals: NO-GO (by adversarial default)

The search surfaced **no qualifying unmet sub-niche**. The only concrete signal:

- **suite2p PCA-denoise allocation** (bug location confirmed 3-0): `denoise.py` allocates a single
  dense `block_re = np.zeros((nblocks, nframes, Lyb*Lxb))` scaling with frame count instead of
  streaming. **But** this is one optional code path (`denoise=True`), not a binding, recurring,
  cross-tool out-of-core pain.

Two adjacent (B) framings were **refuted**:

- The "binding failure on a realistic movie / 106 GiB demand" framing — killed **1-2**.
- "suite2p doesn't persist/reuse the registered binary movie" — killed **0-3** (it DOES persist and
  reuse a `.bin`).

suite2p/CaImAn memmap and MNE `preload=False` were not shown to leave a binding gap. Under the
adversarial default, absence of evidenced demand → NO-GO. (This is weaker than (A)'s affirmatively-
closed verdict: the search simply did not find a qualifying seam, not that one provably cannot exist.)

## Caveats

- **Time-sensitivity cuts against Segovia.** The fastest-moving evidence (anndata-rs v0.6.0,
  anndata-zarr, annbatch) is all dated 2026-06-12 to 2026-06-22 — incumbents are not just present
  but *accelerating* in exactly Segovia's space at the moment of assessment.
- Two of the strongest (A) sources (BPCells bioRxiv preprint, TileDB vendor blog) are not
  journal-peer-reviewed, but every load-bearing fact was independently corroborated by shipped,
  maintained software (and a peer-reviewed NAR paper for TileDB), so (A)'s NO-GO does not rest on
  weak sources.
- The AGPL-vs-BSD/MIT clash is decisive and under-explored in the raw claims; it disqualifies the
  component path in the only domain where the need would otherwise exist.

## Open questions (not affirmatively answered)

- (B): do suite2p/CaImAn memmap, MNE `preload=False`, and existing Zarr/Dask loaders leave ANY
  specific bounded-memory streaming seam unserved (e.g. miniscope long-recording concatenation,
  multi-session widefield)? Not affirmatively answered.
- Given the license clash, is there ANY viable form in which a Segovia-derived core could be adopted
  by scverse-ecosystem tools, or does AGPL alone disqualify the only domain where the need exists?
- Does the prior ephys conclusion (binding bottleneck = GPU compute + host-RAM wall) have an
  analogue in (A)/(B) — has the field's true constraint already moved past CPU streaming I/O?
- Is there demonstrated standalone demand for anndata-rs *outside* SnapATAC2 (general scRNA-seq)?

## Sources

- BPCells (bioRxiv): https://www.biorxiv.org/content/10.1101/2025.03.27.645853v1.full
- Scarf (Nature Comms / PMC): https://pmc.ncbi.nlm.nih.gov/articles/PMC9360040/
- TileDB-SOMA launch: https://www.tiledb.com/blog/tiledb-launches-soma-apis-for-single-cell-data
- CZ CELLxGENE Census (NAR / PMC): https://pmc.ncbi.nlm.nih.gov/articles/PMC11701654/
- anndata 0.12: https://scverse.org/blog/2025-anndata-012/
- anndata-rs: https://github.com/scverse/anndata-rs
- anndata-zarr (crates.io): https://crates.io/crates/anndata-zarr
- annbatch: https://github.com/scverse/annbatch
- suite2p denoise issue #832: https://github.com/MouseLand/suite2p/issues/832
- suite2p binary-reuse issue #131: https://github.com/MouseLand/suite2p/issues/131
- TileDB-SOMA repo: https://github.com/single-cell-data/TileDB-SOMA
- SingleRust: https://github.com/SingleRust/SingleRust
- CaImAn mmapping: https://github.com/flatironinstitute/CaImAn/blob/main/caiman/mmapping.py
