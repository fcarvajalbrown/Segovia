# Segovia — session handoff (paused 2026-06-10)

Pick-up notes for the next session. Nothing irreversible was done: `main` is clean, no merge, no tag, no release.

## Where we are

The M2–4 preprocessing chain (bandpass → CMR → whiten) is built, correct, and benchmarked. The
project's **competitive premise went to the drawing board** this session, driven by real measurements
and two deep-research passes.

### SC1 benchmark — updated numbers
Same-session real IBL AP (first 10 min, 18M samples, 385 ch, 30 kHz; Windows, 8-core / 7.8 GB RAM; n_jobs = batch = 4):

| engine | wall | peak RSS |
|---|---|---|
| **Segovia (with prefetch overlap)** | **187.8 s** | **1.19 GB** |
| spikeinterface (thread) | 187.5 s | 1.75 GB — **1.00× TIE** |
| spikeinterface (process) | 247.4 s | 2.89 GB — 1.32× faster; OOMs at n_jobs=8 |

- A **prefetch-thread overlap** (background Rust thread decodes `.cbin` ahead into a bounded channel
  while the rayon pool computes) took speed from 0.84× → **1.00×** vs SI's thread pool. This change is
  **uncommitted** in `src/dsp/pipeline.rs` — a keeper (211→187 s, output verified identical).
- An earlier attempt (parallel native-chunk decode in `CbinChunkIter`) was **reverted**: decode is
  **memory-bandwidth bound** (1.66× on 16 cores), no gain, +150 MB.

### The measurement error to fix
The SC1 run only tested **10 min of the basic chain** — which is memory-light for everyone, so SI sat
at a comfy 1.75 GB and we wrongly called the memory win "marginal." The **documented blowups** (README
cites a 26 GiB filter error and a **102 GiB motion-correction** blowup) are on *different ops at full
scale* we never tested.

### In flight when paused
Running the **full ~80 GB recording** (100,529,156 samples — the whole `tests/data/...ap.cbin`) through
Segovia's chain for a real-scale metric. Background task **`bqchfrbic`** — read its `.output` file for
the number next session.

## The strategic verdict (from research)

Two full deep-research passes are saved at:
- `docs/research/2026-06-10-rust-ephys-structural-niches.md`
- `docs/research/2026-06-10-rust-ephys-demand-validation.md`

Conclusion:
- **No competitive moat vs SpikeInterface** — batch is a tie; SI 0.102 defaults to a *thread* pool
  (numpy/scipy release the GIL), so there was never a pickle tax to beat.
- The genuine **structural Rust wins** (real-time determinism, embeddability on no_std/edge/wasm) are
  already held by **compiled C++/C#/FPGA** incumbents (Falcon, ONIX, Intan) — not Python. Chasing them
  means building **alternatives to established tools**, which is **out of scope** (see below).
- Demand for a Rust closed-loop/edge engine **leans poor**, but **fundability, edge/embedded demand,
  adaptive-DBS market size, and Rust-in-neuro traction were left UNPROVEN** (the run hit a session rate
  limit; those claims died 0-0, not refuted).

## User's scope constraint (important)

> "i dont want an alternative to already established tools — that was NOT the original scope."

Segovia's scope = a component that **slots into / accelerates the existing SpikeInterface / SpikeGLX /
Zarr / NWB stack**, NOT a competitor. The roadmap endpoint was always "SpikeInterface preprocessing
backend." Evaluate every direction by: *does this augment the existing stack, or stand up a rival?*
Rival → out of scope.

## The one live, in-scope thread

Fix a **specific documented breakage** in the existing Python/SI workflow — the **102 GiB
motion-correction memory blowup** is the prime candidate: in-scope (a component inside the SI
workflow), a real pain, not a rival. Caveats: Segovia doesn't implement that op yet, and it's
**not verified** that it's still unmet. Get a **real metric at real (80 GB) scale on the op that
actually OOMs** before any verdict.

## Open decision for next session

Speed is a tie; basic-chain memory win is marginal *because we measured the easy case*. Decide:
1. Pursue the in-scope heavy-op (motion correction) memory win — measured at real scale — as a
   contribution to the SI stack; or
2. Accept the negative result and ship v0.4.0 honestly.

**Do NOT** rewrite ROADMAP/CHANGELOG/ADR/README or release until this is settled — confirm direction
and exact wording first (a PyPI/crates.io publish is irreversible).

## Parked repo state

- Branch `feat/sc1-preprocess-chain` — 7 chain commits + docs commit `2753675` ("reframe to
  bounded-memory gate" — that framing is now itself questioned; revisit before reusing). NOT merged.
- **PR #9 = DRAFT** (deliberately, to block auto-merge). CI was green. `main` untouched.
- Uncommitted keeper: prefetch overlap in `src/dsp/pipeline.rs`.
- Untracked to keep: `docs/research/2026-06-10-*.md`. Also untracked: `scripts/robust_download_ap.py`.
- Real AP data at `tests/data/_spikeglx_ephysData_g0_t0.imec0.ap.{cbin,ch,meta}` (gitignored, ~29 GB).
  SI baseline in `.venv-si` (Python 3.12). Harness: `bench/bench.py`.
