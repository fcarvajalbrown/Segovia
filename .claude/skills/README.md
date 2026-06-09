# Project skills

Project-scoped Claude Code skills live here, one directory per skill, each with a `SKILL.md`.
They are invoked with `/<skill-name>` and are available to anyone working in this repository.

Nothing is defined yet. Likely future candidates for Segovia:

- a benchmark-runner skill (set up the SC1 gate run and collect memory/throughput numbers),
- a release skill (changelog + version bump + tag, gated on explicit approval),
- a dataset-fetch skill (pull free IBL/DANDI/SpikeGLX fixtures for tests).

Hooks (automated, non-invokable behaviors) live in `../hooks/` and are wired in `../settings.json`.
