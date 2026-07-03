# segovia: A Python Package for Bounded-Memory, Near-Real-Time Preprocessing of Electrophysiology Time Series

**Felipe Carvajal Brown** — Independent Researcher — fcarvajalbrown@gmail.com — ORCID 0000-0002-8300-7587

Submission type: **Application article**

Note: JSS requires a LaTeX manuscript using the JSS LaTeX template (.cls, .bst files available
at jstatsoft.org/about/submissions). This Markdown draft must be rewritten in LaTeX before
submission. The JSS LaTeX template was unreachable at time of setup — fetch it before starting
the conversion. Scope note: JSS targets statistical computing. This paper's primary framing here
emphasizes the statistical signal processing operations (ZCA whitening, CMR, Butterworth
filtering as statistical estimators); reviewers may still flag scope mismatch. Treat JSS as a
fallback target only.

---

## Abstract

We introduce `segovia`, an open-source Python package (AGPL-3.0) with a Rust backend for
streaming, bounded-memory preprocessing of high-density multichannel electrophysiology recordings.
The package implements three statistical signal processing operations, 5th-order Butterworth
bandpass filtering, common median referencing (CMR), and global zero-phase component analysis
(ZCA) whitening, as a composable pipeline that operates one fixed-size chunk at a time. Memory
consumption is bounded by chunk size and does not grow with recording length, making the package
suitable for processing arbitrarily long recordings on memory-constrained systems and for
near-real-time applications. We describe the software design, the statistical operations
implemented, computational performance on real Neuropixels data, and a built-in streaming data
simulator for ground-truth evaluation. The package is available on PyPI (`pip install segovia`)
and on crates.io.

---

## 1. Introduction

Large-scale extracellular electrophysiology recordings from Neuropixels probes [@Jun2017] produce
multi-channel time series with 384–768 channels sampled at 30 kHz, generating ~22 MB/s per
probe. Before downstream analysis (spike sorting, local field potential analysis), this signal
must be preprocessed: high-frequency noise is suppressed by bandpass filtering, common-mode
artefacts are removed by referencing, and channel correlations are reduced by whitening. Standard
Python tools for this task (SpikeInterface [@Buccino2020]) are designed for offline batch
processing of completed recordings and do not target the constraints of memory-limited or
near-real-time settings.

`segovia` addresses these settings by implementing the same preprocessing chain as a chunked
streaming pipeline in Rust, exposed to Python via PyO3. The Python Global Interpreter Lock (GIL)
is released for each Rust computation, so concurrent Python threads are not blocked. Peak memory
is proportional to chunk size rather than recording length.

## 2. Statistical methods implemented

### 2.1 Bandpass filtering

`segovia` applies a 5th-order Butterworth bandpass filter using the second-order sections (SOS)
representation for numerical stability. The default passband is 300–6000 Hz, isolating the
action-potential frequency range. The filter is applied with zero-phase response (`sosfiltfilt`:
forward pass followed by a backward pass on the reversed signal), which is equivalent to
squaring the frequency-domain magnitude response and eliminating all phase distortion. Zero-phase
filtering requires a look-ahead `margin` of future samples to seed the backward pass; `segovia`
fetches these from the next chunk boundary and discards them from the output, maintaining correct
time alignment.

The filter coefficients are computed once at initialisation using the bilinear transform of the
Butterworth prototype. The implementation uses SOS form rather than transfer-function form to
avoid numerical instability for high-order filters.

### 2.2 Common median referencing (CMR)

After bandpass filtering, the spatial median across all channels is subtracted from each sample.
Formally, for a matrix `X` of shape `(T × C)` (samples × channels):

```
X_CMR[t, :] = X[t, :] - median(X[t, :])
```

This is a robust alternative to average referencing that is insensitive to high-amplitude
artefacts on individual channels (such as movement spikes), which would bias an arithmetic mean.
CMR is applied independently to each chunk.

### 2.3 Global ZCA whitening

Whitening decorrelates the channels by applying the inverse square root of the empirical
covariance matrix. Zero-phase component analysis (ZCA) whitening additionally minimizes the
mean squared distance between the original and whitened signals, which is preferable to PCA
whitening when the original channel ordering carries spatial meaning (as in probe recordings).

The whitening matrix is estimated from the first `n_calib` samples:

```
Sigma = (1 / n_calib) * X_calib^T * X_calib
W = (Sigma + lambda * I)^{-1/2}
```

where `lambda` is a regularisation constant. `W` is computed once and stored; all subsequent
chunks are whitened by `X_white = X @ W`. Whitening is computed in `f32`.

## 3. Software design

### 3.1 The `ChunkSource` trait

The central Rust abstraction is the `ChunkSource` trait, an iterator over fixed-size
`(samples × channels)` `i16` buffers. Three implementations are provided:

- **`SpikeGlxReader`** reads memory-mapped SpikeGLX `.bin` + `.meta` files.
- **`ZarrReader`** reads chunked Zarr arrays (gzip, zstd, Blosc) through the `zarrs` crate.
- **`CbinReader`** reads mtscomp-compressed IBL `.cbin` files through per-chunk zlib decompression.

A `ChunkSource` is consumed by `preprocess(chunk_source, config)`, which runs the bandpass →
CMR → whitening chain using Rayon's data-parallel thread pool. Cross-chunk filter state (IIR
initial conditions) is propagated deterministically.

### 3.2 Built-in simulator

`SyntheticEphysReader` is a `ChunkSource` that generates synthetic multi-channel signals without
a file. Each unit's spike train is a Poisson process; each spike template is a Ricker (Mexican-hat)
waveform spatially attenuated by point-source decay. Additive Gaussian noise is added. The PRNG
(SplitMix64 + xoshiro256++) is platform-independent and chunk-size-independent, ensuring
reproducible benchmarks. `ground_truth()` returns spike ground-truth labels for evaluating
downstream spike sorters.

## 4. Computational performance

Performance is evaluated by the replay-at-acquisition-rate method: data are streamed at the true
30 kHz sampling rate and per-chunk compute latency is measured by monotonic clock. A chunk meets
its real-time deadline when compute latency ≤ chunk period (`chunk_samples / fs`).

On the full 55.8-minute real International Brain Laboratory Neuropixels AP-band recording (385
channels, mtscomp-compressed, 11,167 chunks, Windows, 8 physical / 16 logical cores) at the 300 ms
budget (steady state), Segovia sustains 99.7% deadline adherence at 0.21 GB peak RSS versus
SpikeInterface online (`get_traces`, `n_jobs=1`) at 94.7% and 0.41 GB, with markedly lower tail
latency (max 334 vs 932 ms) and jitter (39 vs 60 ms). The cold-start first-60 s per-chunk sweep,
where SpikeInterface's warm-up cost is highest and the adherence gap widest, is:

| Chunk | Mean (ms) | p99 (ms) | Deadline adherence (cold 60 s) | Peak RSS |
|---|---|---|---|---|
| 100 ms | 92.9 | 122.0 | 74.2% | 0.21 GB |
| 300 ms | 194.5 | 256.4 | 100% | 0.28 GB |
| 1000 ms | 617.3 | 705.9 | 100% | 0.49 GB |

At 300 ms the 60 s slice shows Segovia 100% vs SpikeInterface 69.5%; that gap narrows to the
steady-state 99.7% vs 94.7% above over the full recording. On synthetic uncompressed data,
`segovia` achieves 100% adherence at all chunk sizes with jitter < 20 ms.

Peak memory scales with chunk size and not recording length on all inputs. On the full 55.8-minute
(29 GB compressed) recording the memory bound holds to within 1% of a ten-minute slice; in a
batch-throughput comparison there, Segovia at a pinned batch used less peak memory than both of
SpikeInterface's parallel executor modes (1.19 GB vs 2.18 / 4.42 GB) and completed in less wall time
(806 s vs 923 / 1022 s) in a single run.

## 5. Usage

```python
import segovia
import numpy as np

reader = segovia.SpikeGlxReader("path/to/recording.bin")
config = segovia.PreprocessConfig(
    bandpass=segovia.BandpassConfig(low_hz=300.0, high_hz=6000.0),
    cmr=True,
    whiten=segovia.WhitenConfig(n_calib=60000),
)
for chunk in reader.preprocess(config):
    arr = np.asarray(chunk)
```

Full API documentation and additional examples are in the repository README and `docs/`.

## 6. Summary

`segovia` implements bandpass filtering, CMR, and ZCA whitening as a streaming, bounded-memory
Rust pipeline exposed to Python via PyO3. It is the first Python-accessible library to provide
all three standard electrophysiology preprocessing operations in a single-chunk, GIL-released,
memory-bounded pipeline suitable for near-real-time and memory-constrained settings. Source code,
tests, and documentation are at https://github.com/fcarvajalbrown/Segovia (AGPL-3.0-or-later).

## References
