# ADR 0008 — Domain-neutral core, ephys as first vertical, single-cell as a designed future vertical

**Status:** Accepted

## Context

Segovia's value is an engine — chunked, out-of-core, bounded-memory compute over large arrays in
Zarr/HDF5, with GIL-released Rayon threading and a zero-copy PyO3 bridge. That core is not
inherently about electrophysiology; ephys is the first domain it is pointed at. A second domain —
**single-cell genomics**, which underpins modern leukemia research (clonal evolution, CAR-T) — shares
the engine's *infrastructure* even though its *operations* differ (sparse cells×genes matrices, no
temporal continuity, vs dense channels×samples time-series with cross-chunk filter state).

The maintainer wants the project to be ~90% Segovia (ephys, the winnable open niche) with a real,
cheap ~10% path that could later aid leukemia research — **aided by the tool, not a tool made for
it**. See `docs/future/leukemia-direction.md`.

## Decision

Architect Segovia as a **domain-neutral core plus thin domain verticals**:

- **`segovia-core`** — domain-agnostic: storage adapters (Zarr/HDF5), chunk scheduler,
  Rayon+GIL-release execution, lazy operation graph, zero-copy NumPy/Arrow bridge. Knows nothing
  about spikes.
- **`segovia-ephys`** — the first and only built vertical: SpikeGLX reader, bandpass/CMR/whiten,
  SpikeInterface integration.
- **Future verticals** (e.g. single-cell) are anticipated via clean seams but **not built now**.

Define the seams as traits: a `Source` (chunked data provider), an `Operation` (chunk transform),
and a scheduler decoupled from any domain semantics.

## Consequences

- ~90% of the engine (IO, scheduling, memory model, Python bridge) is reusable by a future
  single-cell vertical; the non-reusable part is mostly the operations/algorithms. Be honest that
  the shared part is plumbing, not math.
- **Guardrail against premature generalization (YAGNI):** design the seams, do NOT build a second
  vertical or speculative abstractions for it now. The abstraction would likely be wrong before the
  second domain's needs are known. Keep boundaries clean; add no sparse-matrix code until/unless a
  single-cell vertical is actually green-lit.
- Keeps option value cheap and honest: the 10% is *design + domain literacy*, not impact yet. Do not
  let it become a story that makes ephys work feel like leukemia work — it isn't, until a vertical or
  upstream contribution ships.
- Compatible with several future realizations (dependency on / contribution to / learning from
  SingleRust, or a native vertical, or an interop backend) — see `docs/future/leukemia-direction.md`.
