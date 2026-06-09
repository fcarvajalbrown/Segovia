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
- **`ROADMAP.md` is the single source of truth** for version and scope. Read it before any
  version or release action.
- **Finishing a task includes updating `ROADMAP.md` and the changelog** — automatically, as the
  last step of any user-visible change. Pure chore/docs/ci/tooling work touches neither.
- **A release is a deliberate roadmap event, never a side effect of a commit.** Creating a `v*`
  tag is a production action — require my explicit approval before tagging.
- **Announce releases via Discussions natively:** cut releases with
  `gh release create <tag> --discussion-category "Announcements"` so GitHub auto-posts the release
  notes as an Announcements discussion. No standing Action — it rides on the deliberate release step.

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

- **Repo:** https://github.com/fcarvajalbrown/Segovia (`origin`). **First runnable code landed** — the
  day-1 zero-copy NumPy spike (`segovia.zeros`, `segovia.__version__`), merged via PR #2 with CI green
  on Windows/macOS/Linux. The `segovia` crate is **published on crates.io at v0.0.0**
  (AGPL-3.0-or-later, published manually 2026-06-09). **PyPI not yet.**
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
- **Next up (next session):** (1) **automate package publishing** — see the TODO below; (2) M0–2
  roadmap work — the SpikeGLX `.meta`/`.bin` + Zarr chunked, memory-bounded reader.

## TODO (next session) — automate package publishing ("set it and forget it")

The crates.io publish was manual. Next session, wire publishing into CI so a deliberate release ships
both packages automatically:

- **Rotate keys.** Revoke the crates.io token used 2026-06-09 (it was pasted in chat) and create a
  **new** crates.io token. For PyPI, prefer **Trusted Publishing (OIDC)** so no token is stored at all;
  otherwise create a PyPI API token.
- **Store them as GitHub Actions *secrets*** (encrypted) — **not** plaintext repo "variables": e.g.
  `CARGO_REGISTRY_TOKEN` (and `PYPI_API_TOKEN` if not using Trusted Publishing).
- **Add `.github/workflows/release.yml`** triggered on a `v*` tag / published GitHub Release: run
  `cargo publish` for the crate and build + upload PyPI wheels via `PyO3/maturin-action`. Then a
  release publishes both automatically — no manual `cargo publish`.
- Keep a release deliberate (tag = approved event), and link each release to an Announcements
  discussion (`gh release create <tag> --discussion-category "Announcements"`).

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
