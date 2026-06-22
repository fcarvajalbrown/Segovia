# Is there real, fundable, switchable demand for a Rust closed-loop/edge ephys engine?

**Deep-research report — 2026-06-10**
Run `wf_18561c20-223`. 108 agents · 25 sources fetched · 112 claims extracted · 25 verified (11 confirmed, 14 killed) · 6 search angles.
Adversarial framing: the default hypothesis to *disprove* was "there is NOT enough switchable, fundable demand."

> ⚠️ **This run hit a session rate limit mid-verification** (resets 2:20pm America/Santiago). 14 of 25
> claims died **unadjudicated** (vote 0-0, never verified), so several whole sub-questions are
> **unevidenced, not disproven** — see "What the rate limit left unproven" below. Treat the verdict as
> "leans demand-poor for the real-time-framework fight," with named gaps still open.

## Question

Is there real, fundable, addressable demand today for a new Rust software engine in deterministic
real-time/closed-loop ephys and/or edge/embedded on-device ephys DSP — or is that niche too small,
too hardware-locked, and too satisfied with incumbents to be a viable reason for a project to exist?
(This pass concerns *demand only*; the technical "where could Rust win" was settled by the prior study.)

## Bottom line

The default adversarial hypothesis **survives**: there is NOT a clearly fundable, switchable market
opening for a standalone Rust closed-loop/edge ephys compute engine, and the evidence **leans
demand-poor** — *for the specific fight of competing as a real-time framework.* The flagship
commercial product is a ~$100M niche; the only genuine structural Rust win (deterministic sub-ms) is
already delivered by entrenched compiled incumbents; and where an open third-party slot exists, the
bottleneck is hardware transport, not software DSP.

## Confirmed findings

### 1. The commercial closed-loop market is a small niche *(high, 3-0)*
NeuroPace — leader in responsive (closed-loop) neurostimulation — reported **~$100M FY2025 total
revenue** (RNS System $81.7M). RNS is the only FDA-approved responsive neurostim system. Even the
dominant commercial closed-loop ephys product is a ~$100M-revenue niche.
*Note: the related "20–25% sustained growth" framing was refuted 0-3 — do not claim fast growth.*
Adaptive DBS (Medtronic Percept) is a separate, larger category **not captured here**.
Sources: `sec.gov/.../exhibit991_q42025earningsr.htm` · `investors.neuropace.com`

### 2. The competitor is compiled C/C++/C#, NOT Python/SpikeInterface *(high, 3-0)*
Falcon (C++11), Intan RHX (C++ 89.4% / C 10.3%, Qt), Bonsai/ONIX (C#/.NET on FPGA-timestamped
hardware). A Rust engine in this niche displaces compiled C/C++/C#/HDL — **the "we beat Python" moat
narrative does not apply.**
Sources: `iopscience.iop.org/.../aa7526` · `github.com/Intan-Technologies/Intan-RHX` · `open-ephys.github.io/onix-docs/Software Guide/`

### 3. The one structural Rust win is already delivered by incumbents *(high, 3-0 / 2-0)*
**This is the single strongest finding against a pivot.** Falcon (2017): median round-trip 0.59 ms,
software <0.5 ms, <1-in-1000 detections exceed 1 ms. Bonsai/ONIX: vendor claim of sub-ms,
**independently corroborated by Nature Methods 2024 (Newman et al.)** — "99.9% worst case closed-loop
latency <1 ms" on standard Windows 10, ~300 µs with 2× Neuropixels 2.0, ~100 µs feedback loops. The
low-jitter sub-ms niche is occupied by compiled incumbents on commodity OS — the exact target a Rust
engine would claim.
Sources: `iopscience.iop.org/.../aa7526` · `open-ephys.github.io/onix-docs/` · `nature.com/articles/s41592-024-02521-1`

### 4. The historical incumbent pain was already solved in 2017 *(high, 3-0)*
The Falcon paper documents the pain it was built to fix: "RTXI has only a single real-time thread";
NeuroRighter and Open Ephys "lack the ability of the user to have direct control over the CPU
resources … and limit the implementation of new processing elements." Falcon (C++11, GPL3,
multi-threaded, user CPU control) addressed it nine years ago — the dissatisfaction that would
motivate switching was captured by a C++ tool, not left open for a new entrant.
Source: `iopscience.iop.org/.../aa7526`

### 5. An open third-party DSP plugin slot DOES exist *(high, 3-0)*
**This cuts AGAINST the hardware-lock thesis.** Open Ephys GUI is "agnostic to the origin of the
incoming data" (Source plugins interface with "virtually any hardware"), and the signal chain uses
"modules which can be written, compiled, and distributed separately from the main host application."
Supported sources span Intan, Neuropixels PXI, NI-DAQmx, LSL, etc. **The market is not structurally
closed — there is a real plugin slot.** (Claims that Intan RHX is hardware-locked were refuted/split.)
Scope caveat: the plugin API is C++; the slot's existence says nothing about third-party usage at
scale, Rust-plugin feasibility, or fundable demand.
Sources: `iopscience.iop.org/.../aa5eea` · `open-ephys.github.io/gui-docs/`

### 6. …but where that slot exists, software DSP is not the bottleneck *(high, 3-0)*
In Open Ephys, even with a 5 ms software buffer, mean closed-loop latency stays ~10 ms, explicitly
attributed to "delays inherent in the USB communication protocol" (official docs: 20–30 ms baseline).
Decisive natural experiment: the Open Ephys PCIe prototype dropped latency to **69 µs** mean once USB
was removed, with the residual bottleneck being ADC sampling, **not software DSP**. The vendor's own
remedy was a hardware transport swap, not faster software. So a sub-ms CPU DSP engine offers little
marginal value in the one stack with an open slot.
Sources: `iopscience.iop.org/.../aa5eea` · `open-ephys.github.io/gui-docs/Tutorials/Closed-Loop-Latency.html`

### 7. The modern-hardware real-time slot is already filled by first-party tools *(high, 3-0)*
ONIX docs recommend exactly two platforms — "Bonsai Package OpenEphys.Onix1" and "Open Ephys GUI
Plugin ONIX Source" — both maintained within the Open Ephys ecosystem, with Bonsai achieving sub-ms.
A third-party Rust engine would have to displace first-party tooling that already meets the bar.
Source: `open-ephys.github.io/onix-docs/Software Guide/`

## What the rate limit left UNPROVEN (unevidenced, NOT disproven)

These sub-questions had **all** their candidate claims die at 0-0 (never adjudicated):

- **Fundability** — *zero* surviving claims on whether NIH BRAIN / EU / VC money flows to open-source
  real-time/edge neuro **infrastructure software** (vs hardware/therapeutics). Completely unevidenced.
- **Edge/embedded on-device DSP demand** — claims about Spartan-6 spike sorters, 55nm closed-loop
  ASICs, and in-house lab builds all died 0-0. **The embeddability angle's demand was never tested.**
- **Adaptive DBS market size** (Medtronic Percept — the *larger* closed-loop category) — never quantified.
- **Rust adoption traction in neuro** — the JetBrains State-of-Rust claim died 0-0.

## Caveats

- Time-sensitivity: NeuroPace figures are current (FY2025). Falcon (2017) and Open Ephys USB-latency
  (2016/17) evidence is older, but staleness *strengthens* the adversarial case — the capability has
  not regressed and the vendor remedy was hardware, not better software DSP.
- Source quality is strong: every load-bearing finding rests on primary peer-reviewed papers
  (J. Neural Eng., Nature Methods), official vendor docs, or SEC/IR filings. No blog-only load-bearing claims.
- The hardware-lock adversarial point is genuinely **weaker** than the prompt assumed: Open Ephys is
  provably hardware-agnostic with an open plugin slot. The market is not structurally closed — it is
  already-served and transport-bottlenecked.
- The growth-rate claim was explicitly refuted 0-3.

## Open questions (the gaps to close once the rate limit resets)

1. Actual market size/growth of **adaptive DBS** (Medtronic Percept and competitors) — the larger
   closed-loop category, never quantified here.
2. Is grant (NIH BRAIN / EU) or VC money flowing to open-source real-time/edge neurotech
   **infrastructure software** — as opposed to hardware/therapeutics? Unevidenced.
3. For on-device/implantable DSP (NeuroPace RNS, Neuralink, Paradromics): custom ASIC/FPGA/C firmware
   built in-house, and would regulated safety-critical teams ever adopt an external open-source Rust
   DSP library? All candidate claims failed (0-0).
4. Is there any measurable Rust adoption traction in neuro tooling, and what are the concrete
   switching barriers (ecosystem maturity, hiring, trust, C++ plugin-API friction for a Rust author)?

## Notable refuted claims

- "Closed-loop neurostim market growing ~20–25%/yr, NeuroPace 2026 guidance 20–22%." — 0-3.
- "Open Ephys GUI cannot achieve sub-ms closed-loop precision." — 1-0 (insufficient support).
- "Intan RHX is tightly coupled / hardware-locked to Intan hardware." — 1-0 / 0-1 (refuted/split).
- (Killed unadjudicated, 0-0, due to rate limit) FPGA Spartan-6 spike sorters; 55nm closed-loop ASIC;
  in-house lab builds; pyNeurode 160 ms; ONIX 300 µs; SpikeInterface-is-offline-only; Rust-adoption-thin.

## Sources

| URL | Quality | Angle |
|---|---|---|
| sec.gov/.../exhibit991_q42025earningsr.htm (NeuroPace 10-K) | primary | Market size & growth |
| mobilityforesights.com/.../adaptive-closed-loop-deep-brain-stimulation-systems-market | secondary | Market size & growth |
| nature.com/articles/d41586-025-03849-0 | secondary | Market size & growth |
| iopscience.iop.org/article/10.1088/1741-2552/aa7526 (Falcon) | primary | Incumbent pain points |
| iopscience.iop.org/article/10.1088/1741-2552/aa5eea (Open Ephys) | primary | Hardware bundling / lock-in |
| open-ephys.github.io/gui-docs/Tutorials/Closed-Loop-Latency.html | primary | Hardware bundling / lock-in |
| open-ephys.github.io/onix-docs/Software Guide/ | primary | Hardware bundling / lock-in |
| github.com/Intan-Technologies/Intan-RHX | primary | Hardware bundling / lock-in |
| blog.jetbrains.com/rust/2026/02/11/state-of-rust-2025/ | primary | Rust-in-neuro adoption |
| ncbi.nlm.nih.gov/pmc/articles/PMC12250937/ | primary | Rust-in-neuro adoption |
| biorxiv.org/content/10.1101/2022.01.18.476764v1.full (pyNeurode) | primary | Rust-in-neuro adoption |
| open-ephys.org/onix-system | primary | Rust-in-neuro adoption |
| elifesciences.org/articles/61834 (SpikeInterface) | primary | Rust-in-neuro adoption |
| rustfoundation.org/.../2025-technology-report | primary | Rust-in-neuro adoption |
| pmc.ncbi.nlm.nih.gov/articles/PMC10977078/ (55nm ASIC) | primary | On-device DSP & build-vs-buy |
| nature.com/articles/s44172-025-00504-4 | primary | On-device DSP & build-vs-buy |
| medium.com/@neuronic_img/a-deeper-look-at-neuralinks-n1-chip | blog | On-device DSP & build-vs-buy |
| arxiv.org/html/2601.01772 | primary | On-device DSP & build-vs-buy |
| arxiv.org/html/2311.05063v2 | primary | On-device DSP & build-vs-buy |
| thetransmitter.org/.../neurophysiology-data-sharing-system-faces-funding-cliff | secondary | Fundability of infra software |
| pmc.ncbi.nlm.nih.gov/articles/PMC12319967/ | primary | Fundability of infra software |
| braininitiative.nih.gov/research/dissemination/u24-program | primary | Fundability of infra software |
| thetransmitter.org/.../278-million-cut-in-brain-initiative-funding | secondary | Fundability of infra software |
| neurotechnology.substack.com/p/2024-funding-snapshot | secondary | Fundability of infra software |
