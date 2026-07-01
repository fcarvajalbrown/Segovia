# GPU-bottleneck gating question — verdict (2026-06-22)

**Run:** deep-research `wf_bf0b999d-cc5` (re-run of the stopped `wf_bd0ea473-f2e`).
**Method:** 5 search angles, 17 sources fetched, 71 falsifiable claims extracted, top 25
verified by 3-vote adversarial verification (kill on >=2/3 refutes). 13 confirmed, 12 killed.

## The question

Has the binding memory AND wall-clock bottleneck in the modern Neuropixels pipeline moved so
decisively to GPU spike sorting (Kilosort4 / GPU-dependent sorters) that a CPU-targeted,
bounded-memory streaming **preprocessing** engine (bandpass -> CMR -> whiten; ties SpikeInterface
on throughput; no GPU sorting) would be optimizing a step that is no longer the binding constraint?

## Verdict: YES — preprocessing is not the binding step

The binding constraints of the modern Neuropixels pipeline — both the GPU-compute load and the
memory wall that scales with recording length — live in **spike sorting (Kilosort4)**, not in
CPU preprocessing. A CPU-targeted bounded-memory preprocessing engine optimizes a step that is
not the binding constraint. **No surviving counterevidence** put preprocessing back on the
critical path: every "preprocessing still binds" niche claim was refuted in verification.

This settles the gating question. Per [[segovia-no-competitive-moat]] and
[[segovia-scope-is-component-not-alternative]], any preprocessing-centric direction for Segovia
is optimizing a non-binding step regardless of language.

## Important honesty caveat — the magnitudes did NOT survive verification

The adversarial pass killed **every precise time-budget figure**, including ones that would have
*supported* the verdict. So the answer is directional, not quantified:

- KILLED (0-3): "preprocessing 0.38 h/probe vs sorting/postproc dominating" (eLife 110170).
- KILLED (1-2): "sorting ~60% / postproc ~23% / preprocessing ~8%" (bioRxiv 2025.11.12.687966).
- KILLED (0-3): "Kilosort4 run times within 2x recording duration" (Nature Methods 2024).
- KILLED (0-3): "8 h to sort a 4-shank NP2.0 on an RTX 4080"; "160 h template recalculation"
  (Kilosort issue #631).

Do **not** cite a specific preprocessing-vs-sorting time fraction as fact. The headline summary
the harness emitted ("preprocessing is one-seventh of sorting", "0.38h vs 2.7h") rests on claims
that were themselves killed — it is not supported by the verified set.

## What actually survived verification (the confirmed skeleton)

1. **Kilosort4 is fundamentally GPU-dependent.** It is implemented on the GPU; the CPU backend
   is testing-only / not intended for production. (3-0; Nature Methods 2024, Kilosort docs.)
2. **GPU hardware is required only for the sorting step**, not for preprocessing. (2-1.)
3. **Kilosort4 requires an NVIDIA GPU with >=12 GB VRAM.** (3-0; Kilosort hardware docs.)
4. **The memory wall that scales with recording length is HOST RAM inside the sorter**, not GPU
   VRAM and not preprocessing. (3-0.) Even the memory-pressure story points at sorting, not at
   the bounded-memory preprocessing Segovia targets.
5. **The field has standardized on SpikeInterface + Kilosort.** The Allen Institute deprecated
   its standalone `ecephys_spike_sorting` pipeline. (3-0.)

## Sub-question answers

- **Q1 TIME BUDGET:** Directionally, sorting + postprocessing dominate and preprocessing is a
  small fraction — but no specific split survived verification. Treat the exact fraction as
  unknown; treat "preprocessing is minor" as directionally supported and uncontested.
- **Q2 MEMORY/HARDWARE GATE:** Two gates, both in sorting. (a) GPU VRAM (>=12 GB NVIDIA) is a
  hard gate for Kilosort4 and a CPU preprocessor cannot relieve it. (b) Host RAM is the
  length-scaling memory wall, and it binds in the sorter's clustering, not in preprocessing.
- **Q3 CPU-ONLY SORTERS:** GPU Kilosort is the production default (Allen, IBL via SpikeInterface).
  CPU-only sorters (Mountainsort5, SpyKING CIRCUS, Tridesclous) still exist and are the only
  GPU-free path, but that is a *sorter* concern, not a preprocessing one — it does not create a
  preprocessing niche.
- **Q4 PREPROCESSING-AS-BOTTLENECK COUNTEREVIDENCE:** None survived. The 102 GiB motion-correction
  OOM (SI #3489) was killed (0-3) — consistent with the prior finding that it was user error
  (upsampled motion grid). "Drift correction is the CPU-bound binding step" was killed (0-3).
  The niches rest on refuted single-issue reports.
- **Q5 WHERE TIME ACTUALLY GOES:** Verified evidence points to the sorting stage (GPU compute +
  host-RAM clustering). No verified evidence located the bottleneck in filter/CMR/whiten.
- **Q6 VERDICT:** A CPU preprocessing engine optimizes a non-binding step. The narrow niches that
  could rescue it (long/concatenated recordings, repeated re-filtering, no-GPU labs, real-time)
  produced **zero surviving counterevidence** in this pass — they remain asserted, never
  demonstrated. Not enough to justify a preprocessing-centric project.

## Implication for Segovia

The preprocessing premise is settled negative. This does not by itself pick a new direction; it
removes preprocessing (BPCells-style, component backend, re-angles) from the table. Any future
direction must target the actually-binding steps (GPU sorting compute, host-RAM clustering at
scale, sorting-adjacent IO/format) or accept retirement of the compute-engine framing.

## Sources (verified set)

- Nature Methods 2024 — Kilosort4 (s41592-024-02232-7) [primary]
- Kilosort4 hardware docs — kilosort.readthedocs.io/en/latest/hardware.html [primary]
- Kilosort issue #631, #686 — VRAM / runtime reports [primary/forum]
- Allen Institute ecephys_spike_sorting (deprecated) [primary]
- eLife reviewed-preprint 110170; bioRxiv 2025.11.12.687966 — time-budget (claims killed) [primary]
- SpikeInterface issues #3489, #3321, #3591 — OOM reports (preprocessing-niche claims killed) [primary/forum]
- eNeuro ENEURO.0229-23.2023; PMC11093732; PMC5469488 [primary]
