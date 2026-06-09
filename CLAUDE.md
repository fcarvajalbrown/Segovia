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

## Automated publishing reminders (set up next session)

TODO for the next session: configure a **Stop hook in `.claude/settings.json`** that fires a
**periodic LinkedIn publishing reminder** (e.g. every 21 days, only printing if the interval has
elapsed) — covering **both this project (Segovia) and MaskOps**. Mirror the proven pattern from
MaskOps (`tools/social/devto_reminder.py` + the Stop hook that fires every 21 days). The reminder
should nudge a milestone-only post (benchmark results, releases, tutorials) — not every commit —
and respect a cooldown. Draft copy lives in `assets/draft_linkedin_es.md`. Use the `update-config`
skill to write the hook. Confirm cadence and channels with the user before enabling.

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
- `rust-neuro-research.md` — the fact-checked research dossier this project is founded on.
