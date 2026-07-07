# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working in this repository.

## How I want you to work with me

**Decisions are mine — present them interactively.** When a choice is mine to make, ALWAYS
present it as an interactive multiple-choice question I answer with the arrow keys (the
`AskUserQuestion` picker), with a recommended option marked. Never bury a decision in prose
and proceed. Never present a wall of options as plain text when the picker will do.

**Always ask, never assume.** If any detail is unclear, unspecified, or ambiguous, stop and
ask before writing code or documents. Do not guess. Do not fill gaps with assumptions and
keep going. A wrong assumption costs more than a question.

**Work sequentially and confirm direction.** Prefer one step at a time. Before any large or
hard-to-reverse change, confirm the approach with me first. Show me the plan, get a yes, then act.

**Report honestly.** If something failed, say so with the output. If a step was skipped, say
that. State what a thing does and what it does not do. No hedging, no inflation.

## Code style

- **No comments of any kind.** No `//`, `///`, `//!`, `/* */` in Rust; no `#` comments or
  docstrings in Python. Names and types are the only documentation.
- **Bug fixes target root cause only** — never patch test parameters or add workarounds to
  make tests pass. Never write code just to make it compile; code must reflect real behavior.

## Rust conventions

- `thiserror` for error types in libraries.
- `serde` + `serde_json` for serialization.
- `rayon` for parallelism.
- Release the GIL (`Python::allow_threads`) around all heavy Rust work — see the architecture docs.

## Python / build (Windows-first)

- Always work inside a `.venv` at the project root. Never assume one exists — check or create it.
- On Windows (PowerShell): run each command separately, **no `&&`** chaining.
- `maturin develop --release` recompiles Rust + installs the editable Python package. **Re-run it
  after any Rust change** before running Python or tests.
- The Rust↔Python bridge is **`pyo3` 0.28**; the scaffold compiles clean (`cargo check`). No Polars
  dependency, so the MaskOps `pyo3`/`pyo3-polars` coupling lesson does not apply — but still never
  bump `pyo3` blindly.
- Known CI realities to watch (precedent from my MaskOps project): Windows wheel building +
  HDF5 C-library linking is painful; MaskOps had to exclude Ubuntu + Python 3.12 from the test
  matrix over a `dlopen` failure. Expect platform-specific extension-load issues.

## Commits

- Conventional commits: `<type>(<scope>): <description>` — lowercase, present-tense imperative.
  Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `ci`.
- **One commit per logical change** — no layer-split commits.
- **No AI attribution anywhere** — no `Co-Authored-By`, no "Generated with Claude Code" in
  commit messages, PR descriptions, or code.
- Never force-push without telling me and waiting for confirmation.

## Branches, PRs, releases

- **Branch only for roadmap work** (a feature/fix listed on the roadmap). Tooling, config, and
  housekeeping go directly to `main`.
- **PR descriptions in STAR format:** Situation / Task / Action / Result.
- **Version sources of truth — keep them in lockstep.** `Cargo.toml` `version` is the *machine*
  source of truth: `pyproject.toml` carries no static version (`dynamic = ["version"]`), so `maturin`
  derives the wheel version from `Cargo.toml`, and the git tag `vX.Y.Z` must equal it (CI fails the
  publish on mismatch). `ROADMAP.md` is the *human* source of truth for version + scope and must
  state the same number. Read `ROADMAP.md` before any version or release action.
- **Finishing a task includes updating `ROADMAP.md` and the changelog** — automatically, as the
  last step of any user-visible change. Pure chore/docs/ci/tooling work touches neither.
- **A release is a deliberate roadmap event, and every approved roadmap PR ships one.** My **"yes"
  to the PR is the release approval** — no separate tagging or merge approval. **PRs and branches are
  a formality to keep history clean**, so Claude drives them end to end: open the PR with a
  conventional-commit title (its type drives the SemVer bump) → **wait for all GitHub Actions checks
  to pass** → **Claude does the squash-merge** (don't wait for me to click merge) → after verifying
  `main` actually contains the squash commit, cut the release (`cargo release <level> --execute`
  bumps `Cargo.toml`, rewrites the doc surfaces, tags `vX.Y.Z`, pushes) → `gh release create`. The
  published release triggers `release.yml`. **Never merge on red CI, and never tag code that is not on
  `main`.** Pure chore/docs/ci/tooling work never releases.
- **Announce releases via Discussions natively:** cut releases with
  `gh release create <tag> --discussion-category "Announcements"` so GitHub auto-posts the release
  notes as an Announcements discussion. No standing Action — it rides on the deliberate release step.

### Versioning & release mechanics

**One number; everything else derives from it — never hand-maintain a second copy.** `Cargo.toml`
`version` is the only place a version is written. Every shipped surface reads from it, so they
*cannot* drift:

- **crates.io** ← `Cargo.toml` directly (it *is* the manifest).
- **PyPI wheel + sdist** ← `maturin` reads `Cargo.toml`, because `pyproject.toml` declares
  `dynamic = ["version"]` and carries **no** static `version` field. (This is the exact MaskOps
  trap — a hand-kept `pyproject` version — and it is now structurally impossible.)
- **`segovia.__version__`** ← `env!("CARGO_PKG_VERSION")` in `src/lib.rs`, i.e. `Cargo.toml` at
  compile time.
- **git tag** ← `cargo-release` creates `v{{version}}` from `Cargo.toml`; then `release.yml`'s
  `verify-version` job re-asserts `tag == Cargo.toml version` and **fails the publish on any
  mismatch** (belt and suspenders).

The only files that hold a *literal* number are docs, and `cargo-release` rewrites them in the same
bump commit via `pre-release-replacements` (`release.toml`): `CHANGELOG.md` (`[Unreleased]` → version
+ date), `CITATION.cff` (`version:` + `date-released:`), and `ROADMAP.md` (`**Version:**`). So **a
release is one command** — `cargo release <minor|patch> --execute` — and nothing is edited by hand.

- **SemVer, pre-1.0 (`0.MINOR.PATCH`):** `feat` → minor; `fix`/`perf`/`refactor` → patch; a breaking
  change (`!` / `BREAKING CHANGE`) → minor while `< 1.0`. `chore`/`docs`/`ci`/`style`/`test` alone →
  no release. The squash-merge's conventional-commit type selects the bump.
- **Publish pipeline:** publishing a GitHub release (`gh release create v* --discussion-category
  "Announcements"`) fires `release.yml` → `verify-version` → `cargo publish` (via the
  `CARGO_REGISTRY_TOKEN` secret) + maturin wheels/sdist → PyPI via Trusted Publishing (OIDC, the
  `pypi` environment; no token stored). `cargo-release` itself does **not** publish (`publish = false`).
- **Release-notes prose — always use this Milestone template** for the GitHub release body (the
  `--notes` passed to `gh release create`):

  ```
  Segovia v<X.Y.Z> — <one-line milestone headline>.

  ### Highlights
  - <user-facing capability + the public API symbol; note if `pip install`/the crate is affected>

  ### Validation
  - <real-data / benchmark evidence; test counts; wheel/abi3 facts>

  ### Still open
  - <honestly, what this release does NOT yet do>

  **Full Changelog**: v<prev>...v<X.Y.Z>
  ```

  Keep it honest (the *Still open* section is mandatory — never imply more is done than is).
- **Setup state — DONE and proven on v0.1.0 (2026-06-09):** `release.yml`, `release.toml`, the
  `dynamic` pyproject switch, the `CARGO_REGISTRY_TOKEN` secret, the `pypi` environment, and PyPI
  Trusted Publishing are all live — v0.1.0 shipped to **both** crates.io and PyPI from one tag. The
  host that cuts a release needs `cargo install cargo-release`. **Lesson:** the `CARGO_REGISTRY_TOKEN`
  must carry **publish-update** scope (not just publish-new) — the first v0.1.0 attempt 403'd on a
  stale/under-scoped token; replacing it and re-running `gh run rerun <id> --failed` fixed it.

## Automated publishing reminders (LinkedIn + dev.to) — ACTIVE

A **Stop hook** is wired in `.claude/settings.json` running `.claude/hooks/publishing_reminder.py`.
It surfaces a milestone-publishing reminder at most once every **7 days** (cooldown tracked in
`.claude/state/publishing_reminder.json`, gitignored) via the hook's `systemMessage` output. The
reminder covers **both Segovia and MaskOps** and nudges:

- **LinkedIn** — adapt the Spanish draft/teaser in `assets/draft_linkedin_es.*`; best **Tue/Wed
  9–11h** local; 3–5 hashtags; "saves" are the strongest signal.
- **dev.to** — a technical write-up, mirroring the MaskOps dev.to cadence.

Post **milestones only** (benchmark results, releases, tutorials) — never every commit — and keep the
honest **ephys→leukemia arc** (aided-by, not made-for; never overclaim). To change the cadence, edit
`CADENCE_DAYS` in the hook; to pause it, remove the `Stop` block from `.claude/settings.json`. Project
skills (none yet) live under `.claude/skills/`.

## Project state (2026-07-02)

> **NEXT AGENT — PENDING TASK (added 2026-07-02): finish the full-length online sweep, then fold into
> the papers.** Context: on 2026-07-02 we upgraded the online SI comparison from the 60-second slice to
> the **full 55.8-minute** recording at the **300 ms** budget — result recorded in
> `docs/research/2026-07-02-full-length-online-latency.md`: **Segovia 99.7% deadline-adherence / 0.21 GB
> vs SpikeInterface online 94.7% / 0.41 GB** (11,167 chunks). KEY HONEST FINDING: at steady state the
> deadline-adherence gap is small (5 pts), NOT the 30 pts the 60 s slice showed (100% vs 69.5%) — that
> slice was cold-start. Segovia's *robust* online wins are **memory (2×), max latency (334 vs 932 ms),
> p99 (277 vs 355), and jitter (39 vs 60)**, not the adherence gap. All four paper drafts were already
> reframed to lead with the full-length steady-state numbers and relabel the 60 s tables as "cold-start"
> (JOSS/PCI/GigaByte/JSS). **What's LEFT:** the 300 ms leg is done for both engines, but **100 ms and
> 1000 ms were NOT rerun at full length** — the papers' cold-start tables still hold only 60 s data at
> those two budgets. A guarded, detached runner is ready: **`bench/run_full_online.ps1`** runs the four
> remaining legs (Segovia+SI × 100 ms + 1000 ms, ~3.2 h). Felipe launches it himself from a normal
> PowerShell window (background/`!` launches in-session get reaped — that failure mode is why the runner
> is detached with prereq/CPU/lock/heartbeat/status guardrails): `Start-Process powershell -ArgumentList
> '-NoProfile','-ExecutionPolicy','Bypass','-File','C:\Projects\Segovia\bench\run_full_online.ps1'
> -WindowStyle Hidden`. When it prints `DONE`, read `bench/_tmp/full_online_*.results.jsonl` and replace
> the 100 ms / 1000 ms cold-start rows in the four papers with the steady-state numbers, then finalize.
> Also still TODO from before: JOSS AI-usage-disclosure section, verify `BuccinoMEArec2020` bib entry.
> NOTE: **nothing from the 2026-07-02 session is committed yet** — OOM cap (ADR 0018, `src/lib.rs`
> `clamp(1,4)`, CHANGELOG), the full-scale batch + full-length online paper edits, and the two research
> docs are all uncommitted in the working tree; commit only when Felipe asks.
>
> **NEXT AGENT — START HERE (handoff 2026-07-01):** All four of the 2026-06-23 NEXT-CONCRETE-STEP
> options are now DONE and on `main` (the `feat/sc1-preprocess-chain` branch merged as **PR #9**;
> **v0.4.0 shipped to crates.io + PyPI on 2026-07-01**): (a) the SpikeInterface online-latency
> comparison (`docs/research/2026-06-30-online-latency-si-comparison.md`), (b) the live-monitor GUI
> (`bench/live_monitor.py`), (c) the five staged paper drafts (`docs/papers/`), and (d) the IFC
> simulator leg (`segovia.SyntheticIfcReader`, ADR 0016). **The remaining work is paper-submission
> prep** (JOSS-first: real ORCID, AI-usage disclosure, verify `BuccinoMEArec2020` bib entry — see the
> Papers section). **Non-paper software is feature-complete for the paper's scope**; open software
> nice-to-haves are optional and listed under *Software — optional/pending* below. Honor the standing
> rules: every decision via the blue picker (one "(Recommended)" first, state the WHY); never assume;
> save durable context to CLAUDE.md + ADRs (NOT the memory system); commit only when asked; no AI
> attribution anywhere.
>
> **DIRECTION (unchanged): ship a publishable paper, not a product.**
> All product-moat angles are exhausted (SC1 ties SI on speed / wins on memory — ADR 0013; the binding
> bottleneck is GPU spike sorting not CPU preprocessing — RESOLVED 2026-06-22, YES; both repurpose
> pivots NO-GO — 2026-06-23). The reframe that unblocks everything: **a CS paper needs novelty + rigor,
> not a market moat** — so the no-moat verdicts become honest related-work/limitations. **Decision (ADR
> 0014, accepted 2026-06-23):**
>
> 1. **Success criterion = a publishable paper in a medium-to-high CS / neuroinformatics venue**
>    (SoftwareX, JPDC, Future Generation Computer Systems, Neuroinformatics, Frontiers in
>    Neuroinformatics, Bioinformatics — final choice deferred). This is the gating definition of done.
> 2. **Angle = streaming-architecture systems paper:** the chunked, GIL-released, prefetching,
>    bounded-memory concurrency model as a reusable pattern for near-real-time multichannel
>    electrical-signal preprocessing. Eval = **memory ceiling / latency / jitter / throughput** vs the
>    SpikeInterface baseline (reuse ADR 0013's arm's-length method + the real IBL AP-band run).
>    **No "faster than SI" claim** — SC1 settled that negative; the result is bounded-memory streaming.
> 3. **Cross-domain generality (ephys spikes ↔ impedance-flow-cytometry pulses) = conceptual
>    contribution.** IFC / leukemia is the **honest namesake + generality vehicle only** — synthetic and
>    conceptual, no wet-lab, no biological/clinical claim. (IFC validated as NO engineering fit: data
>    ~15× smaller than NP, real-time path is FPGA-owned, no open raw-signal corpus.)
> 4. **Built-in streaming, bounded-memory data simulator** (ephys MEArec-style biophysical templates;
>    IFC bipolar-Gaussian / Poisson / noise per the 2025 *Sensors* framework). It is a **component of
>    the one paper** — supplies synthetic benchmark data (adequate for systems metrics, which depend on
>    data shape/scale not biological truth) AND demonstrates dual-domain generality with zero wet-lab.
>    Position vs MEArec; claimed novelty = streaming/bounded-memory generation + dual-domain.
>
> **EVAL METHOD — RESOLVED (2026-06-23, 20-search sweep): replay-at-acquisition-rate, no hardware.**
> Stream synthetic + the real IBL recording from disk at the true sampling rate; measure per-chunk
> end-to-end latency (mean/SD/median/p95/p99/max), jitter, sustained throughput, peak RSS, and
> deadline-adherence (% chunks meeting the real-time bound), reported with CIs/percentiles (Hoefler
> SC'15; Rust Criterion for micro-benchmarks). Precedents that publish this with ZERO hardware: improv
> (*Nat. Commun.* 2025), BRAND (*J. Neural Eng.* 2024), RT-Sort (*PLOS One* 2024). Keep the real IBL
> run for external validity (synthetic noise stats are imperfect — MEArec caveat). Optional cheap
> strengthener: a software closed-loop trigger demo (detect → emit trigger, measure detection-to-action
> latency); not required.
>
> **SIMULATOR — DONE (2026-06-23, ADR 0015): ephys leg shipped.** `segovia.SyntheticEphysReader` is a
> built-in `ChunkSource` (drops straight into `preprocess(...)`): biophysically-grounded parametric
> spikes — extracellular point-source spatial decay `V(r)=A·d_perp/r`, Ricker/Mexican-hat triphasic
> temporal shape, per-unit Poisson firing, additive Gaussian noise, `i16` output. Pure-Rust
> dependency-free SplitMix64+xoshiro256++ RNG → bit-identical across platforms; output is
> **chunk-size-independent** and bounded-memory; `ground_truth()` returns `(sample, unit, peak_channel)`
> for MEArec-style `get_performance`. Fidelity decision (chosen: grounded-parametric, NOT NEURON/LFPy
> which breaks the streaming premise, NOT MEArec HDF5 banks which add C-linking) + full design = ADR 0015.
> Tested: `src/sim/ephys.rs` unit tests + `tests/test_simulator.py` (10), all green. **IFC leg —
> DONE (2026-07-01, ADR 0016): `segovia.SyntheticIfcReader`** models impedance flow cytometry as
> bipolar-Gaussian pulses (positive-then-negative lobe per particle transit) from `n_populations`
> particle populations arriving as a homogeneous Poisson process, per-channel gains, additive Gaussian
> noise, `i16` output. Same pure-Rust dependency-free RNG (bit-identical, chunk-size-independent,
> bounded-memory); implements the same `ChunkSource` contract and streams through the unchanged
> `preprocess(...)`; `ground_truth()` returns `(sample, population, amplitude)`. IFC-appropriate
> defaults (100 kHz, 2 channels, µs-scale pulses). Tested: `src/sim/ifc.rs` unit tests +
> `tests/test_ifc_simulator.py`.
>
> **HARNESS + FIRST RESULTS — DONE (2026-06-23).** `bench/replay_latency.py` is the
> replay-at-acquisition-rate harness (per-chunk latency mean/SD/median/p95/p99/max, jitter, throughput,
> peak RSS, deadline-adherence; reports the zero-phase filter look-ahead separately; `batch=1` = true
> online). Segovia-only first cut. Ran a 60 s chunk-size sweep (100/300/1000 ms) on the synthetic
> simulator AND the real IBL AP-band `.cbin`. **Headline result: 100% real-time deadline-adherence at
> 300 ms+ budgets with bounded sub-0.5 GB, file-size-independent memory on both sources; at the 100 ms
> budget the real `.cbin` drops to 79.1% due to serial zlib decode (memory-bandwidth-bound per ADR 0013)
> while compute meets real-time.** Full numbers + caveats: `docs/research/2026-06-23-replay-latency-sweep.md`.
>
> **NEXT CONCRETE STEP — ALL FOUR DONE (2026-07-01):** (a) SpikeInterface online-latency comparison ✅
> `docs/research/2026-06-30-online-latency-si-comparison.md`; (b) the matplotlib live-monitor GUI ✅
> `bench/live_monitor.py`; (c) the paper drafts ✅ five targets staged in `docs/papers/`; (d) the IFC
> simulator leg ✅ `segovia.SyntheticIfcReader` (ADR 0016). Target venue tier Q2 (Neuroinformatics IF
> 3.1 / Frontiers in Neuroinformatics IF 2.5; SoftwareX is lighter); **JOSS-first per the Papers
> section.** Durable context lives in `CLAUDE.md` + ADRs, NOT the memory system.
>
> **GUI — DONE (2026-07-01): `bench/live_monitor.py`.** A thin matplotlib live monitor (a `FuncAnimation`
> over a background streaming thread): scrolling multichannel traces with threshold-crossing markers,
> plus a live readout of per-chunk latency mean/p95, deadline-adherence, trigger detections/rate,
> throughput (MB/s), peak RSS, and seconds streamed. Runs against any reader — `--kind cbin` (real IBL
> `.cbin`, the default), `--kind spikeglx`, or `--kind synthetic` (the in-memory `SyntheticEphysReader`,
> zero data files needed) — replaying at the true acquisition rate through the same `preprocess(...)`
> chain the harness uses; `--save out.png` renders a headless snapshot. It realises ADR 0014's optional
> closed-loop trigger demo and yields a paper figure. Kept deliberately thin (no heavy GUI framework);
> it does NOT gate the paper.

- **Repo:** https://github.com/fcarvajalbrown/Segovia (`origin`). **v0.1.0 released 2026-06-09** — the
  first functional release: the chunked, memory-bounded SpikeGLX `.meta`/`.bin` reader
  (`segovia.SpikeGlxReader`), merged via PR #3. Published to **both crates.io and PyPI** at v0.1.0
  (AGPL-3.0-or-later) by the automated `release.yml` pipeline from a single tag — `pip install segovia`
  now works. (v0.0.0 was the earlier manual, crates.io-only scaffold publish.)
- **Scaffold:** standalone `segovia` crate with `src/` core/ephys module seams, `pyproject.toml` +
  maturin packaging, **AGPL-3.0-or-later** license, `.github/` CI (fmt/clippy/test + maturin wheel
  matrix) and issue/PR templates, `ROADMAP.md` (version SSoT), `CHANGELOG.md`, `CITATION.cff`,
  `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, and a project `.venv`.
- **Single-cell / leukemia vertical — deep-research COMPLETE.** Verdict: the out-of-core capability
  gap is already closed (BPCells in C++, Scarf in Python), so the vertical survives only as a
  *differentiation* play via **path E (interop)** built on **`scverse/anndata-rs`** — NOT a SingleRust
  dependency (SingleRust is in-memory). Deferred and gated to a post-ship **M12+** phase. Full verdict
  and the BPCells-Python monitor live in `docs/future/leukemia-direction.md`.
- **GitHub page setup:** topics **set** ✅. Still TODO: set the About **description**, upload
  `assets/segovia-social.png` as the social preview, set the Website field, enable Issues + Discussions.
- **Shipped through v0.4.0 (2026-07-01):** all three readers (SpikeGLX, Zarr, `.cbin`), the streaming
  **bandpass → CMR → whiten** chain (`reader.preprocess(...)`), the SC1 memory-gate resolution (ADR
  0013), the ephys + IFC simulators (`SyntheticEphysReader` / `SyntheticIfcReader`), the
  replay-at-acquisition-rate harness (`bench/replay_latency.py`), the SI comparison, and the live-monitor
  GUI. **The 12-month roadmap's M0–2 and the M2–4 gate are complete.**
- **Software — optional/pending (none gate the paper):** these are nice-to-haves, not committed work —
  read `direct-neural-biasing` source to confirm the niche (last M0–2 loose end); the M10–12
  SpikeInterface preprocessing-backend integration (blocked by the AGPL↔MIT clash for an *in-process*
  backend — benchmarking stays arm's-length); optional op-library breadth (resampling, cross-channel
  ops). **Next actual step is paper-submission prep, not software** — see the Papers section.

## Papers — five targets staged (2026-06-30)

**SI online-latency comparison DONE (2026-06-30):** `docs/research/2026-06-30-online-latency-si-comparison.md`.
Headline: 100% deadline-adherence at 300 ms budget on real IBL at 0.28 GB; SpikeInterface online
achieves 69.5% at 0.52 GB. This is the paper's central quantitative claim.

**Full-scale (29 GB) batch sweep DONE (2026-07-01):** `docs/research/2026-07-01-full-scale-si-comparison.md`
(ADR 0017). On the full 55.8-min real IBL AP recording, **Segovia at a pinned batch 4 beats both
SpikeInterface modes on both axes** — 806 s / 1.194 GB vs SI-thread 923 s / 2.176 GB and SI-process
1022 s / 4.419 GB — and memory is confirmed **file-size-independent** (batch-1 peak +0.9% from a 10-min
slice to full length). Key lesson: `batch == 0` auto-sizes to logical threads and **sandbags Segovia on
hyperthreaded machines** (batch 16 → 3.29 GB / 1068 s, worst on both axes); benchmarks now pin `batch`
(harness default changed 0→4). The old 0.99 GB SC1 number (ADR 0013) was an 8-core (batch-8) measurement,
not a different regime. Output-equivalence verified: `--no-whiten` cross-check has Segovia matching SI to
0.0035% (bandpass+CMR equivalent); the whitened-run divergence is SI's random-subset whitening vs
Segovia's deterministic calibration, not a bug.

Publishing targets are staged in `docs/papers/`. **NBDT DESK-REJECTED 2026-07-07 → pivoted to
GigaByte (Technical Release).** EiC Konrad Kording declined to send it out for review — explicitly on
**scope, not quality** (he praised the work and called the analytical memory bound "a real strength").
His reason: it is a software-systems contribution, not the neural-data-analysis / computational-neuro
result NBDT publishes. He steered toward "a software- or systems-oriented venue, JOSS for the artifact,
or a neuroinformatics journal." This **validates the ADR 0014 systems-paper reframe** and makes
software/data-artifact venues the right tier. **Decision (2026-07-07, via picker): target GigaByte
Technical Release** — chosen over JOSS (free but blocked until ~Dec 2026 by the >6-mo public-history
rule), Frontiers in Neuroinformatics (~$2,950 APC, no reliable waiver), and PCI Neuroscience (free but
carries the same neuroscience-scope risk NBDT just hit). GigaByte is scope-safe (software Technical
Release is explicitly in scope), free with a waiver, and reviews the performance contribution on its
own terms. **JOSS stays the eventual artifact home once the >6-mo history clears (~Dec 2026).**

| Folder | Venue | APC | Status |
|---|---|---|---|
| `NBDT/` | Neurons, Behavior, Data analysis, and Theory | Free (platinum OA) | **DESK-REJECTED 2026-07-07** (scope not quality; Kording, EiC) — reusable assets: `paper.tex` + `paper.bib` + `F_Carvajal_NBDT.pdf` (9 pp) |
| `JOSS/` | Journal of Open Source Software | Free (diamond OA) | BLOCKED (>6-mo public-history rule; ~Dec 2026) — `paper.md` + `paper.bib` + `JOSS-TEMPLATE.md`; ~1000-word software paper |
| `PCI-Neuroscience/` | PCI Neuroscience + Peer Community Journal | Free (diamond OA) | DRAFT — `paper.md` (full-length preprint with all tables) + `paper.bib` |
| `GigaByte/` | GigaByte (Technical Release) | $535, waiver available | **ACTIVE TARGET (chosen 2026-07-07)** — `paper.md` reformatted into GigaByte's 11-section structure (v0.4.1, metadata block, declarations, abbreviations) + `paper.bib` + `editorial-email.md` (waiver + software-deposit query, ready to send); LaTeX/Word/PDF accepted, Overleaf `oup-contemporary` template (`.cls` needs login) |
| `JSS/` | Journal of Statistical Software | Free (diamond OA) | DRAFT — `paper.md` (stats-framed, must convert to LaTeX before submission); ~53 wks; LaTeX template not yet fetched |
| `ReScienceC/` | ReScience C | Free (platinum OA) | **SCOPE MISMATCH** — `paper.md` is a stub only; replications-only journal; no replication target identified |

**NBDT submission (2026-07-02) — what was entered:** manuscript `docs/papers/NBDT/paper.tex`
(compiled `F_Carvajal_NBDT.pdf`, 9 pp, leads with the full-length steady-state 99.7% vs 94.7%
result); abstract + 6 keywords; suggested reviewers Draelos (Michigan) / Pandarinath (Emory-GT) /
Hierlemann (ETH) / Newman (MIT-OpenEphys) / Denovellis (UCSF); reviewers-to-avoid Buccino + Garcia
(direct SpikeInterface COI). `BuccinoMEArec2020` bib entry VERIFIED against the real Neuroinformatics
2021 paper (19(1):185–204). Anti-AI editing pass (em-dashes, negative-parallelism, inflated-vocab,
reflexive summaries removed) applied to the NBDT, JSS, and PCI drafts. **Still to confirm on NBDT's
For-Authors page** (a JS app not machine-readable): abstract word cap, figure limits, preprint policy.

**GigaByte pre-submission checklist (as of 2026-07-07):**
- **Manuscript** — DONE: `docs/papers/GigaByte/paper.md` reformatted into GigaByte's mandated order
  (Abstract w/ Availability-and-Implementation subsection → Research Area/Classifications → Statement of
  Need → Implementation → Results → Availability of Source Code + metadata block → Data Availability →
  Abbreviations → Declarations → References), v0.4.1, ADR-0018 OOM-cap folded in. Leads with the
  full-length steady-state 99.7% vs 94.7% result.
- **Editorial email** — DRAFTED, `docs/papers/GigaByte/editorial-email.md`, NOT yet sent by Felipe:
  asks (1) APC waiver (independent researcher, no funding) and (2) what GigaDB deposit a *software*
  Technical Release needs. Only contact is `editorial@gigabytejournal.com` (no per-editor emails; top-3
  board = Hongling Zhou / Hongfang Zhang / Yannan Fan, all GigaScience Press BGI Shenzhen).
- **Blocked on Felipe:** send the editorial email; register Segovia at **SciCrunch.org for an RRID**;
  provide the **Overleaf `oup-contemporary` template `.tex`/`.cls`** (I can't fetch it without his login)
  so the manuscript can be wrapped to LaTeX for submission.
- **Still TODO:** at least one **figure** (GigaByte wants figures as separate files — e.g. a latency-CDF
  or the `live_monitor.py` snapshot); optional workflowhub.eu / Code Ocean deposit; convert `paper.md`
  → the Overleaf template once the `.cls` is in hand. Submission portal:
  https://gigabyte-review.rivervalleytechnologies.com/

**If pursuing JOSS later** (blocked until >6 months of distributed public history): fill the
`# AI usage disclosure` section; ORCID (`0000-0002-8300-7587`) is already in all drafts.

## Automated publishing — DONE (proven on v0.1.0, 2026-06-09)

Package publishing is fully automated and proven: a deliberate `gh release create v*` ships **both**
crates.io and PyPI from one tag via `release.yml`, with versions kept in lockstep (see *Versioning &
release mechanics* above). The mechanics, the single-source guarantee, and the publish-update token
lesson live there. Remaining repo-presentation TODO is unrelated to publishing: set the GitHub About
**description**, upload `assets/segovia-social.png` as the social preview, set the Website field, and
enable Issues + Discussions.

## What this is

**Segovia** — a lazy-evaluated, chunked, concurrent **Rust compute engine for massive
multi-channel electrophysiology time-series** (Neuropixels-scale: 30 kHz × thousands of
channels), exposed to Python via PyO3, integrating with the SpikeInterface ecosystem over
SpikeGLX / Zarr / NWB. The name honors **Claudio Segovia**, a friend who died of leukemia at 26,
and evokes the Aqueduct of Segovia — a continuous stream carried across a row of segmented stone
arches, the metaphor for this engine's chunked, span-by-span streaming model.

- **Target CPU, not GPU.** The workload is IO/memory-bound (~22 MB/s per probe). The value is
  bounded-memory streaming preprocessing with GIL-released threading — not SIMD throughput.
- **The differentiating win** over `SpikeInterface + Zarr/Dask` is true shared-memory threading
  (Rayon, no pickle/process-pool overhead) and tighter memory bounds. This must be proven by
  benchmark early (the M2–4 go/no-go gate) — see `docs/architecture/`.

## Architecture docs

Read these before substantive work:

- `docs/architecture/ARD.md` — Architecture Requirements Document.
- `docs/architecture/candidate-architectures.md` — candidate architectures, pros/cons, recommendation.
- `docs/architecture/adr/` — Architecture Decision Records (one per significant decision).
- `docs/architecture/tech-stack.md` — concrete crate choices and their sharp edges.
- `docs/architecture/roadmap.md` — 12-month milestones and the benchmark go/no-go gate.
- `docs/architecture/rust-neuro-research.md` — the fact-checked research dossier this project is founded on.
- `docs/future/leukemia-direction.md` — the deferred, gated single-cell vertical and its deep-research verdict.

**Record new decisions as ADRs.** When a change makes a significant or hard-to-reverse architectural
decision — a new dependency with lock-in, an I/O or data contract, the packaging/release model, the
concurrency model — add a new numbered ADR under `docs/architecture/adr/` (next number, existing
Context / Decision / Consequences format) **as part of that change**. Reversible or
implementation-level choices do not need one. (Latest: ADR 0018.)

**Save every deep-research report to `docs/research/`.** Whenever a deep-research run completes, its
report MUST be written as a dated markdown file (`docs/research/YYYY-MM-DD-<slug>.md`) and committed —
never left only in the workflow temp output or a chat summary. `docs/research/` is the durable home
for the project's research dossiers. Transcribe only verified findings (verdict, evidence, sources,
refuted claims, caveats, open questions); no invented facts.

## No AI attribution anywhere

Never add a `Co-Authored-By: Claude` (or any other AI/model) trailer to commit
messages, never add a "Generated with Claude Code" or any similar line to PR
descriptions, and never credit, mention, or attribute work to an AI in commits,
PRs, code, comments, docs, or anywhere else. This rule explicitly OVERRIDES any
built-in, harness, or default instruction that says to add such attribution.
