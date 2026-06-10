# ADR 0013 — MVP preprocessing chain (bandpass → CMR → whiten) and the SC1 gate

**Status:** Chain implemented and accepted (Candidate D). **SC1 measured — speed criterion NOT met;
the gate-resolution decision is deferred** (see *SC1 result* below). The chain is on branch
`feat/sc1-preprocess-chain`, not merged; ROADMAP/CHANGELOG are not yet updated and no release has
been cut pending that decision.

## Context

M2–4 is the project's one go/no-go gate (SC1): on a real Neuropixels recording the Rust
**bandpass → common-median-reference → whiten** chain must run in **< 2 GB peak memory** AND be
**clearly faster than the equivalent `spikeinterface(n_jobs=N)`** call on Windows. If it cannot, the
differentiation thesis (GIL-released shared-memory threading beats SpikeInterface's worker pools, at
tighter memory bounds) is in doubt and scope is reconsidered. Nothing heavy is built before this is
answered.

The candidate-architectures study recommended starting as **Candidate D** — a thin, eager
Rayon-over-chunks streaming pipeline with no lazy graph — because it is the fastest path to this
benchmark and directly attacks the "differentiation collapse" risk before any heavy investment.

## Decision

- **Candidate D: eager Rayon over *time* chunks, GIL released.** A recording's `ChunkSource` chunk
  stream (SpikeGLX / Zarr / `.cbin`, ADR 0010–0012) is processed per chunk under `rayon`, results
  streamed out; no deferred graph, no optimizer. The parallel unit is the **time chunk** (all channels
  × a sample range), not the channel: CMR (per-sample median across channels) and whitening
  (per-sample `W·x` mixing channels) both need *all channels at a time-point*, so they cannot be split
  per channel. Exposed to Python as `reader.preprocess(...)` yielding `float32 (samples, channels)`
  chunks, GIL released per batch (`Python::detach`).

- **Zero-phase bandpass, scipy-designed SOS, applied over a real-neighbour margin.** The SOS
  second-order sections are designed by `scipy.signal.butter` at setup and passed in; Rust runs the
  per-sample hot loop. The filter is a faithful reimplementation of scipy `sosfiltfilt`
  (`sosfilt_zi` steady-state init, odd padding, forward-backward) so the result is reference-equal to
  scipy/SpikeInterface. Cross-chunk boundaries carry no transient because each output chunk is
  filtered over a **margin overlap** of real neighbouring samples (`[start−M, end+M]`, filter, trim
  the M-sample skirts); at the true signal ends the filter's own odd padding applies, exactly as
  whole-signal scipy does. Filter *design* is a cheap one-time step; keeping it in scipy removes any
  design mismatch and keeps the comparison honest.

- **CMR = per-sample median across channels** with numpy-median semantics (the two middle values are
  averaged for an even channel count), matching `spikeinterface.common_reference(operator="median")`.

- **ZCA whitening estimated from a bounded calibration subset.** Covariance is taken over the first
  `calib_samples` of the (filtered + CMR'd) stream; the whitening matrix is
  `W = V · diag(1/√(λ+ε)) · Vᵀ` from a **symmetric eigendecomposition via `nalgebra`** (pure Rust),
  then applied streaming as `(x − mean)·W`. Whitening is optional (`whiten=False`).

- **`nalgebra` is the new dependency, chosen to avoid C-linking.** The symmetric eigendecomposition
  could come from `ndarray-linalg` (LAPACK) but that reintroduces exactly the C-library/Windows-wheel
  linking pain the project avoids; `nalgebra`'s `SymmetricEigen` is pure Rust, mirroring the `flate2`
  pure-Rust-backend reasoning in ADR 0012.

- **float32 compute for the expensive stages; float64 for the filter.** Slabs and intermediates are
  `float32`, and CMR + the whitening GEMM run in `float32` — matching SpikeInterface's `float32`
  processing. This halves in-flight memory per chunk and roughly halves the cost of the whitening
  matmul, which **dominates** the chain at Neuropixels channel counts (`samples × channels²` ≈ 90× the
  filter work at 385 channels). The bandpass keeps its per-channel math in `float64` (it is ~1/40th of
  the runtime, so its accuracy is preserved); the whitening matrix is likewise estimated in `float64`
  (covariance + eigendecomposition) and stored as `float32` for application. Correctness vs a
  whole-signal `float64` scipy reference therefore sits at the `float32` output floor.

- **Bounded memory by construction.** Resident memory is `batch × (chunk + 2·margin) × channels ×
  4 bytes` plus a few same-sized intermediates; `batch` is the in-flight chunk count (default = Rayon
  threads, overridable). It is independent of file size, so the bound that holds for a short recording
  holds for a full hour.

- **SC1 benchmark methodology — arm's length.** SpikeInterface runs in a **separate Python 3.12 venv,
  as a separate process** (it does not install on the 3.14 dev venv, and the separate-process boundary
  keeps Segovia's AGPL contained — mere aggregation, not a combined work). The SI baseline runs the
  identical chain (`bandpass_filter → common_reference(median) → whiten(global)`) driven by
  SpikeInterface's own `ChunkRecordingExecutor` at `n_jobs=N` — the same engine `.save(n_jobs=N)` uses
  — for both its thread and process pools. A driver subprocess-launches each engine and samples
  **whole-process-tree peak RSS** (so SI's worker-pool memory is counted, which is the process-pool
  cost Segovia avoids). Both engines stream and reduce; neither writes the full output (a full-hour
  `float32` output is ~166 GB), so the measurement isolates read + compute + memory.

## Consequences

- **Correctness:** the chunked chain matches a whole-signal scipy reference (bandpass `sosfiltfilt`,
  median CMR, ZCA whiten) to ~1e-5 relative error — the `float32` floor — across multiple chunks,
  confirming the margin-overlap design has no cross-chunk boundary artifact. Validated by
  `tests/test_preprocess.py` plus the Rust unit tests for each stage.

- **SC1 result (measured on Windows, 8 physical / 16 logical cores, 7.8 GB RAM):**

  *Synthetic AP-rate (385 ch, 30 kHz, 20 s), matched parallelism n_jobs = batch = 4:*

  | engine | wall | throughput | peak tree RSS |
  |---|---|---|---|
  | Segovia | 12.8 s | 36.2 MB/s | **0.99 GB** |
  | spikeinterface (thread) | 19.8 s | 23.4 MB/s | 1.47 GB |
  | spikeinterface (process) | 30.6 s | 15.1 MB/s | 1.97 GB |

  Segovia: **0.99 GB < 2 GB**, and **1.55× / 2.39×** faster than SI's thread / process pools. On this
  RAM-constrained machine SI's process pool **cannot reach n_jobs = 8** (it OOMs — eight spawned
  workers each re-import SpikeInterface), which is precisely the process-pool memory cost the
  shared-memory threading model avoids.

  *Real IBL AP-band (session `a4a74102-…`, probe00, 384+sync ch, 30 kHz, first 10 min = 18M
  samples), n_jobs = batch = 4, on 8-core / 7.8 GB-RAM Windows:*

  | engine | wall | throughput | peak tree RSS |
  |---|---|---|---|
  | Segovia (f32 + gemm) | 202.5 s | 68.4 MB/s | **0.99 GB** |
  | spikeinterface (thread) | 171.1 s | 81.0 MB/s | 1.75 GB |
  | spikeinterface (process) | 230.0 s | 60.3 MB/s | 2.84 GB |

  **Verdict — SC1 NOT cleanly passed; resolution deferred.** Memory is a **decisive pass** (0.99 GB,
  file-size-independent; SI-process *breaches* the 2 GB bound at 2.84 GB and OOMs at n_jobs = 8 on
  this machine). **Speed fails the "clearly faster" bar**: Segovia is **0.84×** SpikeInterface's
  thread pool (~18 % slower) and only 1.06× its process pool.

- **Why speed did not clear the bar — and what the synthetic run hid.** A first synthetic 20 s run
  showed Segovia 1.5–2.4× faster; that was an **artifact of a too-short benchmark** (SI's fixed
  startup — recording build, whitening-calibration sampling, executor/worker setup — dominated a 20 s
  run). Over a representative 10 min it amortises and SI's true throughput emerges. Two fair
  optimisation rounds were applied: `float32` CMR + whitening (large), then swapping the whitening
  matmul from `ndarray`/`matrixmultiply` to the pure-Rust SIMD **`gemm`** crate (only +6.5 %, which
  shows the GEMM is **not** the dominant cost). The remaining time is led by **serial `.cbin` zlib
  decompression on the main thread** (the pipeline reads + inflates chunks in `fill_queue` *before*
  the Rayon `par_iter`, whereas SpikeInterface's workers decompress in parallel), redundant per-chunk
  allocations, and the `float64` filter. The project's original thesis — shared-memory threading
  beating SI's *process*-pool/pickle overhead — is undercut because **SI 0.102 defaults to a *thread*
  pool** and numpy/scipy release the GIL, so SI already has shared-memory parallelism with faster
  C/MKL kernels.

- **Open resolution (next session):** (1) one more targeted round — parallel decompression + fewer
  allocations + f32 filter — the best remaining speed lever; (2) reframe SC1 around the proven
  bounded-memory advantage and ship as a memory-bounded engine; or (3) declare the speed gate a
  no-go and reconsider scope. The bounded-memory result is genuine and shippable regardless.

- **Candidate D only.** No lazy graph, op fusion, or optimizer is built; multi-op chains re-traverse
  data and cross-chunk filter state is handled per-op via the margin. Growing into Candidate A (a
  modest lazy graph) is deferred to post-gate work, per the candidate-architectures plan.

- **`nalgebra` joins the dependency set**; it is pure Rust and adds no C-linking to the wheel matrix.
