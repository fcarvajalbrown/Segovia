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

## Project state (2026-06-09)

> **START HERE NEXT SESSION (2026-06-22): unresolved gating question.** The "BPCells of
> electrophysiology" re-angle was deep-researched and **rejected** (SI's OOMs are config errors not
> architectural gaps; ephys is preprocessed once not revisited like single-cell matrices; not
> CZI-fundable at v0.1.0/solo). The one decisive question left is **whether the binding bottleneck has
> moved to GPU spike sorting (Kilosort4), which would kill any CPU-preprocessing angle regardless of
> language.** A focused deep-research (`wf_bd0ea473-f2e`) was launched to settle it and then **stopped
> before completion — it is UNANSWERED.** Resolve this before pursuing ANY preprocessing direction.
> Full context in the auto-memory: `gpu-bottleneck-gating-question` and `segovia-no-competitive-moat`.

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
- **Next up:** M0–2 remaining — the **Zarr (`zarrs`) reader** and a realistic **full-1-hour
  bounded-memory run** (IBL data is `mtscomp`-compressed `.cbin`, so it needs a decompression path).
  Then M2–4: the MVP **bandpass → CMR → whiten** chain + the SC1 benchmark go/no-go gate.

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
implementation-level choices do not need one. (Latest: ADR 0010.)

**Save every deep-research report to `docs/research/`.** Whenever a deep-research run completes, its
report MUST be written as a dated markdown file (`docs/research/YYYY-MM-DD-<slug>.md`) and committed —
never left only in the workflow temp output or a chat summary. `docs/research/` is the durable home
for the project's research dossiers. Transcribe only verified findings (verdict, evidence, sources,
refuted claims, caveats, open questions); no invented facts.
