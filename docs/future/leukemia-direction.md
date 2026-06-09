# Future direction — aiding leukemia research (PENDING, not committed)

> **Status (2026-06-09): ACTIVE — deep-research running tonight.** The maintainer is committing to a
> single-cell (leukemia-relevant) vertical and running the multi-agent deep-research tonight to
> *justify* it and choose the realization path. Decision rule: **only abandon the vertical on
> ABSOLUTE 95%+ disconfirming evidence.** The realization approach (B SingleRust-as-dependency vs
> C native vertical vs E interop) is **not** pre-committed — the research decides. The 90% remains
> Segovia (ephys). Keep the search honest (let it find disconfirming evidence).
> See `docs/architecture/adr/0008-domain-neutral-core-verticals.md`.

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

## What the lean research found (so we don't fool ourselves)

The "obvious" single-cell pivots are already occupied or filling fast — a structural reality of a
large, well-funded field:

- **SingleRust** — a Rust single-cell toolkit (bioRxiv Aug 2025): AnnData-compatible, 2.3–24.6× faster
  than Scanpy, supports in-memory *and* disk-backed ops, and explicitly names streaming / memory-
  constrained algorithms as a direction it is pursuing. Plays directly to Rust skills.
  (https://github.com/SingleRust/SingleRust , https://www.biorxiv.org/content/10.1101/2025.08.04.668429v2.full)
- **Scarf** — Python, memory-efficient out-of-core analysis of millions of cells *on a laptop*.
  (https://www.ncbi.nlm.nih.gov/pmc/articles/PMC9360040/)
- **BPCells** — high-performance disk-backed single-cell (2025), built for commodity hardware.
  (https://www.biorxiv.org/content/10.1101/2025.03.27.645853.full.pdf)
- **scanpy** — now advertises >100M cells via Dask/backed mode (imperfect; documented memory issues).
  (https://github.com/scverse/scanpy)
- **rapids-singlecell** — GPU acceleration, pushing toward billion-cell.
  (https://developer.nvidia.com/blog/driving-toward-billion-cell-analysis-and-biological-breakthroughs-with-rapids-singlecell/)

Conclusion: a from-scratch "Rust single-cell engine" would duplicate SingleRust and enter a crowded,
GPU-pressured space. The honest, high-impact moves are interop / contribution / a narrow niche — not a
competing flagship.

## Options for realizing the 10% (choose later)

| # | Option | Pros | Cons |
|---|---|---|---|
| A | **Pending deep-research (multi-agent)** before any single-cell build — full workflow scoped to leukemia-relevant single-cell gaps + how `segovia-core` could extend. | Rigorous; avoids another duplication; same method that caught SingleRust. | Token cost; do it only when ready to seriously consider the vertical. |
| B | **Use SingleRust as a dependency** — call it for single-cell ops; interoperate via AnnData/Zarr. | No rebuild; leverages their algorithms; fast. | Coupling to their API/maturity; their memory model differs from Segovia's out-of-core core. |
| C | **Learn from SingleRust, build a `segovia-singlecell` vertical** on `segovia-core` (sparse support, AnnData reader). | Own coherent engine; full control; reuses the 90% core. | Duplicates effort; enters the crowded space; needs sparse-matrix work. |
| D | **Contribute to SingleRust directly** (out-of-core / streaming, which it is asking for). | Highest *real* leukemia impact for a solo no-bio dev; lowest risk; learn the domain for free. | Not "your own" project. |
| E | **Interop backend** — make `segovia-core` speak AnnData/Zarr so single-cell tools (incl. SingleRust, scanpy) can use Segovia as a fast IO/compute backend. | Bridge not compete; reuses core; broad reach. | Indirect; value depends on others adopting it. |

## Recommended sequence (cheap → committed)

1. **Now (free):** design the `segovia-core` seams so a vertical is *possible* (ADR 0008). Build nothing single-cell.
2. **Ongoing (cheap):** keep domain literacy by making small contributions to **SingleRust** (Option D) — this doubles as learning and as real leukemia-adjacent impact today.
3. **Later (gated):** when/if seriously considering a vertical, run the **pending multi-agent deep-research** (Option A), then choose between B / C / E with real evidence.

## Guardrails

- Do **not** build a single-cell vertical or speculative abstractions now (YAGNI; ADR 0008).
- Do **not** let the 10% become a comfort story — ephys work does not fight leukemia until a vertical
  or upstream contribution actually ships.
- Re-verify incumbents before committing — this space moves fast; these findings are a 2025–2026 snapshot.

## Pending tasks

- [ ] (Option A) Run multi-agent deep-research on leukemia-relevant single-cell computational gaps + Segovia-core extension — **running 2026-06-09 (tonight)**; output decides realization (B/C/E) and must clear the 95% bar to change course.
- [ ] (Option D) Make a first small contribution to SingleRust to learn the domain.
- [ ] Keep `segovia-core` domain-neutral as the engine evolves (revisit at each milestone).
