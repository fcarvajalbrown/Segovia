# ADR 0015 — Built-in streaming, bounded-memory data simulator (ephys first)

**Status:** Accepted

## Context

ADR 0014 set the project's near-term definition of done as a publishable systems paper and decided that
Segovia gains a **built-in, streaming, bounded-memory data simulator** as a component of that paper. The
simulator serves two roles in the single paper: (a) it supplies synthetic benchmark data for the
systems metrics (latency / jitter / throughput / peak memory), which depend on data *shape and scale*
not biological truth; and (b) with the deferred IFC leg it demonstrates dual-domain generality with no
wet-lab dependency. The evaluation method (ADR 0014, resolved) is replay-at-acquisition-rate, which
consumes a streamed data source — so the simulator is the upstream dependency for the benchmark harness.

The fidelity model was a real decision. Three options were weighed:

- **Full biophysical (NEURON/LFPy):** most faithful, but an offline, non-streaming, C/Python-heavy step
  that cannot be a live chunk source and is not bounded-memory — it breaks the paper's premise and is
  the solo-infeasible item ADR 0014 fences out. Its extra realism only buys credibility for
  *sorter-accuracy* claims, which this project does not make.
- **Load real MEArec template banks:** more faithful waveforms, but reintroduces an external
  data/format dependency and HDF5 C-linking (the Windows-wheel trap the project avoids).
- **Parametric, biophysically grounded (chosen):** pure-Rust parametric spikes whose *spatial* amplitude
  decay is derived from the extracellular point-source model and whose *temporal* shape is a standard
  biphasic/triphasic extracellular profile.

## Decision

- **Add a `sim` module with an ephys vertical (`segovia.SyntheticEphysReader`).** It implements the
  existing `ChunkSource` trait (ADR 0010), so it drops into the existing `preprocess(...)` pipeline with
  no pipeline changes, exactly like the SpikeGLX / Zarr / `.cbin` readers. IFC is a future `sim/ifc`
  vertical and is **not** built now (YAGNI, ADR 0008/0014).

- **Biophysically-grounded parametric waveforms.**
  - *Spatial decay:* channels are a 1-D linear array at a fixed `pitch`. Each unit has a soma position
    along the array and a perpendicular distance `d_perp`; per-channel amplitude follows the
    extracellular point-source potential `V(r) = A · d_perp / r` with
    `r = sqrt((ch_pos − soma_pos)² + d_perp²)` (no singularity; the peak channel is the nearest one).
  - *Temporal shape:* a Ricker / Mexican-hat profile `−(1 − x²)·exp(−x²/2)` with `x = τ/σ`, giving the
    triphasic (small-positive / large-negative / small-positive) extracellular morphology; `σ` (~0.2–0.4
    ms) varies per unit.
  - *Firing:* an independent homogeneous Poisson process per unit (exponential inter-spike intervals).
  - *Noise:* additive Gaussian per sample-channel.
  - *Quantization:* µV signal divided by a configurable `lsb_uv`, rounded and clamped to `i16` — the same
    on-the-wire dtype the real readers emit.

- **Pure-Rust, dependency-free RNG.** A small in-tree SplitMix64-seeded xoshiro256++ with a Box–Muller
  Gaussian, rather than the `rand` crate. This adds no dependency and no C-linking, and — critically for
  a reproducible-benchmark paper — guarantees **bit-identical streams across platforms and compiler
  versions** (`rand`'s `StdRng` is explicitly allowed to change). Each unit's spike train and waveform
  parameters are seeded from `mix(seed, unit)`, and noise is seeded **per absolute sample index**.

- **Output is chunk-size-independent and bounded-memory.** Spike times and per-unit parameters are
  generated once at construction; each chunk is synthesized on demand from that fixed state, and noise
  is a deterministic function of the absolute sample index. The same recording is therefore produced for
  any `chunk_samples`, and resident memory is the current chunk plus the per-unit parameter tables
  (`O(n_units × n_channels)`) plus the spike-time list (`O(events)`), independent of recording duration
  for the per-chunk working set.

- **Ground truth is a first-class output.** `ground_truth()` returns `(sample, unit, peak_channel)`
  arrays, enabling MEArec-style `get_performance` (accuracy / precision / recall) once a detector exists.

## Consequences

- The benchmark harness (next step) can stream an arbitrarily long synthetic recording at the true
  acquisition rate with no file and no wet-lab data, while the retained real IBL run (ADR 0013) supplies
  external validity.
- The simulator's claimed novelty, positioned against MEArec, is **streaming / bounded-memory generation
  of arbitrarily large recordings + (future) dual-domain coverage** — to be validated, not assumed.
- **Honest limitation (stated in the paper):** parametric synthetic noise does not perfectly reproduce
  real recording noise statistics (a documented MEArec caveat), and the waveforms are not biophysically
  simulated — adequate for systems metrics, not for any biological or sorter-accuracy claim.
- No new crate dependency and no C-linking are added to the wheel matrix.
- Tested by Rust unit tests (`src/sim/ephys.rs`) and `tests/test_simulator.py`: shape/metadata,
  full-length reconstruction, chunk-size independence, seed determinism, ground-truth validity, the
  point-source decay (noiseless energy peaks on the ground-truth channel), config validation, and the
  `preprocess(...)` integration.
