# Future direction — aiding leukemia research (PENDING, not committed)

> **Status (2026-06-09): deep-research COMPLETE — verdict recorded below.** The multi-agent
> deep-research ran (101 agents, 19 primary sources, 25 claims adversarially verified). Outcome
> against the decision rule (**abandon only on ABSOLUTE 95%+ disconfirming evidence**): the vertical
> is **NOT abandoned**, but it is **reframed**. The out-of-core *capability* gap in single-cell is
> already closed by shipped tools, so the vertical survives only as a **differentiation play via
> path E** (interop), and the genuine Rust build-on target is **scverse/anndata-rs — not SingleRust**.
> The 90% remains Segovia (ephys). See `docs/architecture/adr/0008-domain-neutral-core-verticals.md`.

## Verdict (deep-research, 2026-06-09)

**Capability verdict — the out-of-core / bounded-memory single-cell gap is essentially closed.**
Strong, multi-source disconfirming evidence — but it does **not** reach the absolute 95% bar, so by
the maintainer's own rule the vertical proceeds, reframed from a *capability* play to a
*differentiation* play.

- **BPCells** closes the gap in C++: peer-reviewed (PMC11996304), disk-backed streaming, ~70× memory
  reduction "with little to no loss of execution speed"; normalization + PCA of a **44M-cell** dataset
  on a **32GB laptop** (PCA 6.2h; <1h on a 256GB server). Zero Rust.
- **Scarf** closes it in Python (Zarr chunks + Dask): peer-reviewed (PMC9360040); **4M cells under
  16GB** across a full pipeline where Scanpy could not process the dataset *despite 200GB of RAM*, and
  used ~40× more RAM at 1M cells. Pure Python (99.8%) — no GIL-released native threading.
- **SingleRust is in-memory — the dossier's premise was wrong on this point.** The preprint
  explicitly says "rather than implementing full out-of-core (disk-backed) solutions." Its 30M-cell
  headline is **RAM-bound on a 512GB machine** (peak HVG 463GB, PCA 487GB), a 1.3–3.0× constant-factor
  reduction, not streaming. It *does* ship an `anndata`-crate-based backed/chunked mode, but no
  benchmarked scaling result rests on it. SingleRust also shares Segovia's exact in-memory concurrency
  thesis (compiled, zero-copy, GIL-released), so it offers **no novel mechanism** to build on either.
- **rapids-singlecell** is a different hardware target (NVIDIA GPU / VRAM); its out-of-core path is
  GPU-based. Not a competitor on the CPU bounded-memory axis. (The "GPU beats CPU 15×" disconfirming
  claim was **refuted** 0-3 in verification.)

**Realization-path verdict:**

- **Path B (SingleRust-as-dependency) — REJECTED.** SingleRust is in-memory and provides no
  out-of-core capability to build on, and shares no novel mechanism. The dependency target the
  evidence actually points to is **scverse/anndata-rs**: the official scverse out-of-core AnnData
  **Rust** crate — lazily loaded, chunked, fully HDF5-backed (goes beyond Python anndata's backed
  mode). It is an IO/storage substrate, **not** a compute engine, and makes **no** GIL-released
  threading claim — so it overlaps Segovia only on the storage/lazy/chunk axis and leaves the
  compute + threading axis open.
- **Path C (native vertical) — WEAK.** Hard to justify as new capability given BPCells + Scarf.
- **Path E (interop) — the only coherent path.** Build on `anndata-rs` as the out-of-core Rust
  substrate and compete on **language + threading-model + throughput + Python ergonomics**, not on a
  missing capability.

**Residual openings that keep E alive (differentiation, not capability):**

1. **No shipped tool combines all three of:** CPU out-of-core streaming + Rust + GIL-released Rayon
   native threading. (BPCells = C++; Scarf = Python/Dask, no GIL release; SingleRust = in-memory.)
2. **BPCells' Python story is weak and experimental** — streaming primitives (normalization, PCA,
   ATAC peak/tile) are **R-only** and merely *planned* for Python; Python is read/write-only, stuck at
   `0.3.0rc2` (pre-release) for ~19 months. **Time-sensitive** — see the monitor task below.
3. **Throughput** — out-of-core PCA carries a real penalty (~6× vs best in-memory; BPCells 6.2h /
   Scarf ~10h headline runs). A faster engine could target this.

**The biggest open question — is single-cell even the right vertical?** Segovia's core domain is
Neuropixels-scale electrophysiology: **dense, continuous 30kHz signal**, not **sparse single-cell
count matrices**. The single-cell out-of-core conclusion may not transfer at all, and ephys out-of-core
may be the genuinely under-served niche these sparse-optimized tools do not touch. Resolve this before
any single-cell commitment.

## Importance (flagged by the maintainer)

This direction is marked **important on two axes**, even though it is uncommitted:
- **Architecture north-star** — keep `segovia-core` domain-neutral so this vertical stays possible
  without a rebuild (ADR 0008). Weigh it in every structural decision.
- **LinkedIn narrative** — future articles/posts should carry the honest ephys→leukemia arc (see
  `assets/draft_linkedin_es.md`). Aided-by, not made-for; never overclaim.

## The motivation

Personal: a friend died of leukemia at 26. The goal is for the year's work to have a real, honest
path toward fighting leukemia — without pretending the ephys work itself does so.

## The principle (90/10)

Segovia is **aided toward** leukemia, **not made for** it. Build the domain-neutral engine (~90% of
the value) for ephys, where the niche is genuinely open and winnable. Keep a cheap, real ~10% path
open so the same engine — or the maintainer's skills — can later serve single-cell genomics, which is
the computational backbone of modern leukemia research (clonal evolution, drug resistance, CAR-T).

## Options for realizing the 10% (verdict applied)

| # | Option | Status after deep-research |
|---|--------|----------------------------|
| A | Run the multi-agent deep-research before any single-cell build. | **DONE (2026-06-09)** — verdict above. |
| B | Use **SingleRust** as a dependency for single-cell ops. | **REJECTED** — SingleRust is in-memory; no out-of-core capability and no novel mechanism to build on. Dependency target redirected to `anndata-rs`. |
| C | Build a native `segovia-singlecell` vertical on `segovia-core`. | **WEAK** — duplicates closed capability (BPCells/Scarf); justified only by the threading/throughput differentiation, which is unproven. |
| D | Contribute upstream (now `anndata-rs` / SingleRust) for domain learning + real near-term impact. | **Still valid** — cheapest real leukemia-adjacent impact for a solo dev. |
| E | **Interop** — build on `anndata-rs`; compete on Rust + GIL-released threading + throughput + Python ergonomics. | **CHOSEN PATH** — the only coherent way the vertical clears the bar. Build nothing now. |

## Recommended sequence (cheap → committed)

1. **Now (free):** keep the `segovia-core` seams domain-neutral so a vertical stays *possible*
   (ADR 0008). Build nothing single-cell.
2. **Before any single-cell commitment:** answer the "is single-cell even the right vertical?"
   question — confirm whether ephys-native out-of-core is the stronger, less-crowded niche.
3. **If/when committing:** realize via **path E** on `anndata-rs`; never via B; C only with a proven
   throughput/threading win. Re-verify incumbents first — this space moves fast.

## Guardrails

- Do **not** build a single-cell vertical or speculative abstractions now (YAGNI; ADR 0008).
- Do **not** let the 10% become a comfort story — ephys work does not fight leukemia until a vertical
  or upstream contribution actually ships.
- Re-verify incumbents before committing — this space moves fast; these findings are a 2025–2026
  snapshot. The BPCells Python gap especially is closing.
- Frame the vertical honestly as **differentiation, not novel capability** — the capability exists.

## Pending tasks

- [x] (Option A) Multi-agent deep-research on leukemia-relevant single-cell gaps + Segovia-core
  extension — **DONE 2026-06-09**; verdict recorded above (path E; build on `anndata-rs`, not
  SingleRust).
- [ ] **MONITOR — BPCells Python releases (`>=0.4.0`).** The most time-sensitive opening is BPCells'
  experimental, R-only-streaming Python interface. Re-check `https://pypi.org/project/bpcells/` and
  `https://bnprks.github.io/BPCells/news/index.html` at the next milestone (and roughly quarterly):
  if Python gains streaming normalization / PCA / ATAC primitives and leaves pre-release, the Python
  ergonomics opening **closes** — re-evaluate path E before any single-cell investment.
- [ ] Resolve the prior question: is single-cell the right vertical at all, or is **ephys-native
  out-of-core** the stronger niche? (Sparse-matrix tools do not address dense continuous signal.)
- [ ] (Option D) Make a first small contribution to `anndata-rs` (or SingleRust) to learn the domain.
- [ ] Keep `segovia-core` domain-neutral as the engine evolves (revisit at each milestone).
