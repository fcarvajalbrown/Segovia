# ADR 0014 — Success criterion reframed to a publishable systems paper; built-in cross-domain data simulator

**Status:** Accepted

## Context

A sequence of research verdicts has established that Segovia has **no competitive product moat**:

- **SC1 (ADR 0013):** the bounded-memory criterion passes decisively (~0.99 GB, file-size-independent)
  but the "clearly faster than SpikeInterface" criterion is **not achievable** on this workload — SI
  0.102 defaults to a *thread* pool over GIL-releasing C/MKL kernels, and the `.cbin` decode is
  memory-bandwidth bound for both engines. The differentiation was reframed to bounded-memory streaming.
- **GPU-bottleneck gating (2026-06-22):** the binding bottleneck in the offline pipeline is GPU spike
  sorting (Kilosort4) plus a host-RAM wall, not CPU preprocessing. A faster CPU preprocessing engine
  does not move the critical path.
- **Repurpose research (2026-06-23):** both candidate pivots are NO-GO — single-cell (scverse already
  ships the Rust+PyO3 out-of-core stack; AGPL cannot upstream into BSD/MIT) and non-ephys neuro-signals
  (no evidenced seam).

A focused web sweep (2026-06-23, ~24 searches, not a deep-research run) examined whether any direction
remains:

- **Online / real-time streaming ephys** was never evaluated by the prior verdicts (they covered
  *offline batch*). It is a live niche where no-GIL Rust + bounded memory has a genuine structural
  advantage over Python+Dask, and where the GPU-batch argument does not apply.
- **Impedance flow cytometry (IFC)** is a real, peer-reviewed bridge to leukemia (CD34+/ALL
  classification) and is structurally identical to spike detection (transient pulses in a continuous
  multichannel electrical stream). But validation found it is **not an engineering fit for Segovia's
  moat**: IFC data is ~15× smaller per unit time than Neuropixels and runs minutes not hours (so
  bounded-memory streaming is moot), the valued real-time path favors **FPGA** over CPU, and there is
  **no open raw-signal corpus** — a solo developer cannot build or validate against real IFC data.

The reframe that resolves all of this: **a publishable CS paper does not need a market moat.** It needs
novelty, rigor, and honest evaluation. Every "no-GO" product verdict is a sound related-work or
limitations paragraph, not a fatal result. The maintainer has set the project's near-term definition of
done accordingly.

Two grounding facts were verified before this decision:

- **Simulators publish at this tier.** MEArec — a simulator of ground-truth extracellular spiking
  activity — was published in *Neuroinformatics* (2020, 19:185–204) and is the standard testbench for
  benchmarking spike sorters. A simulator is a proven publishable artifact, not speculative.
- **An IFC simulator is grounded, not invented.** The 2025 *Sensors* derivative-based framework
  validates on *synthetic* IFC streams (bipolar Gaussian pulses, Poisson arrivals, additive white
  noise, ~115 kSa/s), and accepted physical models exist (equivalent-circuit, Maxwell mixture theory,
  FEM). Dual-domain generality can be demonstrated on synthetic data with zero wet-lab dependency.

## Decision

1. **Reframe the near-term success criterion to a publishable paper in a medium-to-high-stakes CS /
   neuroinformatics venue.** This is the project's gating definition of done. Releases and tooling
   continue under the existing roadmap and release mechanics, but the goal that gates "are we done" is
   the paper, not market adoption. Candidate venues (final choice deferred to drafting): SoftwareX,
   Journal of Parallel and Distributed Computing, Future Generation Computer Systems, Neuroinformatics,
   Frontiers in Neuroinformatics, Bioinformatics.

2. **Paper angle: a streaming-architecture systems paper.** The contribution is the chunked,
   GIL-released, prefetching, bounded-memory concurrency model (ADR 0006, 0010–0013) as a reusable
   pattern for near-real-time, bounded-memory preprocessing of massive multichannel electrical-signal
   streams. The evaluation is **memory ceiling, latency, jitter, and throughput** against the
   SpikeInterface (thread/process pool) baseline, reusing ADR 0013's arm's-length methodology and its
   existing real IBL AP-band run. The paper makes **no "faster than SpikeInterface" claim** (ADR 0013
   settled that negatively); the result is the bounded-memory, file-size-independent streaming
   architecture and its real-time behavior.

3. **Cross-domain generality is a conceptual contribution.** The same streaming event-detection
   primitive applies to ephys spikes and IFC pulses. IFC / leukemia is the project's **honest namesake
   and a generality vehicle** — not a product, and not a biological or clinical claim.

4. **Segovia gains a built-in, streaming, bounded-memory data simulator.** It emits realistic-shaped
   synthetic ephys and IFC streams, grounded in accepted models (MEArec-style biophysical templates for
   ephys; bipolar-Gaussian / Poisson / additive-noise per the 2025 *Sensors* framework for IFC). It
   serves two roles within the **single** systems paper: (a) it supplies benchmark data — synthetic
   data is fully adequate for systems metrics, which depend on data *shape and scale*, not biological
   truth; and (b) it demonstrates dual-domain generality with no wet-lab. It is a **component of the
   paper, not a separate paper and not a co-equal headline.** Its claimed novelty, positioned against
   MEArec, is **streaming / bounded-memory generation of arbitrarily-large recordings + dual-domain
   coverage** — to be validated, not assumed.

## Consequences

- The "no competitive moat" verdicts (ADR 0013, the GPU-bottleneck and repurpose memories) stop being
  fatal and become the paper's honest related-work and limitations sections. The bounded-memory
  streaming result is the publishable core.
- **Synthetic-only evaluation is adequate for the systems metrics** (memory / latency / jitter /
  throughput) but **not for any biological claim** — the paper makes none. This limitation is stated
  explicitly. The existing real IBL AP-band benchmark (ADR 0013) is retained so the paper is not
  synthetic-only.
- The paper must position against **MEArec** (prior-art ephys simulator) and **SpikeInterface+Dask**
  (preprocessing baseline). Reviewer risk that real data is demanded is mitigated by the retained IBL
  run; the IFC leg remains synthetic + conceptual by design.
- **Scope guardrail (YAGNI, per ADR 0008):** build the simulator and the streaming / online benchmark
  harness only. Do **not** build a wet-lab IFC pipeline, an FPGA path, or a second product vertical.
  The IFC leg is synthetic and conceptual.
- The arm's-length AGPL benchmarking boundary from ADR 0013 (SpikeInterface in a separate process /
  venv) is kept.
- **Open question for the evaluation-design step:** whether the "near-real-time / closed-loop" framing
  requires a latency-critical demonstration beyond throughput and jitter, or whether streaming latency
  and jitter metrics suffice. Resolve when drafting the evaluation plan, before building the harness.
