# ADR 0016 — IFC simulator leg (impedance flow cytometry), the dual-domain generality vehicle

**Status:** Accepted

## Context

ADR 0014 defined the project's near-term goal as a publishable streaming-architecture systems paper and
named a conceptual contribution: **cross-domain generality** — the same chunked, GIL-released,
bounded-memory streaming pattern applied to a second class of multichannel electrical-signal
preprocessing beyond extracellular electrophysiology. ADR 0015 shipped the ephys leg
(`segovia.SyntheticEphysReader`) and explicitly deferred the impedance-flow-cytometry (IFC) leg as
future `sim/ifc` work (YAGNI). With the ephys leg, the replay-latency harness, and the SpikeInterface
online-latency comparison all in place, the IFC leg is now the remaining piece that turns the paper from
"a streaming ephys tool" into "a *dual-domain* streaming pattern".

IFC is the honest namesake vehicle (the leukemia / single-cell arc, ADR 0008): it is **synthetic and
conceptual only** — no wet-lab, no biological or clinical claim. Its job is to demonstrate that the
engine's streaming/bounded-memory machinery is domain-neutral by consuming a second, physically distinct
signal through the *exact same* `ChunkSource` + `preprocess(...)` contract. IFC was previously validated
as **no engineering fit for a product** (ADR 0014: data ~15× smaller than Neuropixels, the real-time
path is FPGA-owned, no open raw-signal corpus) — which is precisely why the leg is a *generality vehicle*,
not a performance target.

## Decision

- **Add `segovia.SyntheticIfcReader` as a second `sim` vertical (`src/sim/ifc.rs`).** It implements the
  same `ChunkSource` trait (ADR 0010) and emits `i16 (samples, channels)` chunks, so it drops into the
  existing `preprocess(...)` pipeline with no pipeline changes — the same bandpass → CMR → whiten chain
  and the same harness run on IFC data unmodified. This shared contract *is* the generality claim.

- **Bipolar-Gaussian pulse model with distinct particle populations.** A particle transiting the
  differential coplanar electrodes of an impedance cytometer produces a **bipolar pulse** — a positive
  lobe then a negative lobe as the particle enters and leaves the differential sensing volume:
  `p(t) = exp(−½((t − c₁)/σ)²) − exp(−½((t − c₂)/σ)²)`, with the two lobes separated by the
  electrode-transit time and `σ` the single-lobe width. `n_populations` particle populations each carry a
  characteristic amplitude (∝ particle size / normalized impedance change), single-lobe width, and
  lobe separation; each channel is a measurement channel (e.g. excitation frequency) with a per-population
  gain. Particles arrive as a single homogeneous **Poisson** process (`event_rate`, particles/s); each
  arrival draws a population and a per-event amplitude jitter. Additive Gaussian noise per sample-channel;
  the signal is quantized by a configurable `lsb` and clamped to `i16`.

- **IFC-appropriate physical defaults.** `sample_rate = 100 kHz`, `n_channels = 2`, single-lobe width
  ~30–80 µs, lobe separation ~150–350 µs, `n_populations = 3`. The systems metrics depend on data shape
  and scale, not exact biology, but the defaults are chosen to be a credible IFC signal rather than reused
  ephys numbers.

- **Same determinism and bounded-memory guarantees as the ephys leg.** The pure-Rust dependency-free
  SplitMix64 + xoshiro256++ RNG (ADR 0015) is reused: population parameters are seeded from
  `mix(seed, population)`, arrivals from a dedicated stream, per-event population/jitter **per event
  index**, and noise **per absolute sample index**. Output is therefore chunk-size-independent and
  bit-identical across platforms; resident memory is the current chunk plus the per-population parameter
  tables plus the event list, independent of recording duration.

- **Ground truth is a first-class output.** `ground_truth()` returns `(sample, population, amplitude)`
  arrays — the population id supports a classification/generality story and the quantized amplitude is the
  particle-size label.

- **A separate `IfcError` type** rather than overloading the ephys `SimError`, with a one-line
  `From<IfcError> for PyErr` in `lib.rs`, keeping the two verticals decoupled.

## Consequences

- The paper can make the dual-domain generality claim concretely: one engine, one streaming/bounded-memory
  contract, two physically distinct synthetic signal classes, evaluated by the same harness — with **zero
  wet-lab dependency**.
- The IFC leg's claimed novelty, like the ephys leg, is **streaming / bounded-memory generation** plus the
  dual-domain coverage itself; positioned against the 2025 *Sensors* bipolar-Gaussian / Poisson IFC
  signal framework it parametrically follows.
- **Honest limitations (stated in the paper):** the pulse model is a parametric bipolar-Gaussian
  approximation, not a finite-element electric-field simulation; noise is additive Gaussian, not a
  measured impedance-noise spectrum; there is no real IFC corpus to supply external validity (unlike the
  retained real IBL run for ephys), so the IFC leg is explicitly a *conceptual generality demonstration*,
  not an empirically validated IFC model. No biological or clinical claim is made.
- No new crate dependency and no C-linking are added to the wheel matrix.
- Tested by Rust unit tests (`src/sim/ifc.rs`) and `tests/test_ifc_simulator.py`: shape/metadata,
  full-length reconstruction, chunk-size independence, seed determinism, ground-truth validity, the
  bipolar (positive-and-negative) pulse shape, config validation, and the `preprocess(...)` integration.
