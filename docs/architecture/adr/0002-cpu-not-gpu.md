# ADR 0002 — Target CPU, not GPU

**Status:** Accepted

## Context

The original idea framed value as "SIMD parallelism" and the field's famous speedups (Kilosort,
>20×) are GPU-based. It is tempting to target GPU. But those speedups are in **spike sorting**
(iterative template-matching/clustering), not streaming preprocessing.

## Decision

Segovia targets **CPU**. The preprocessing engine does not use GPU compute (non-goal N1).

## Consequences

- **The workload is IO/memory-bound, not compute-bound.** One probe is ~80 GB/hour ≈ **~22 MB/s**;
  NVMe does thousands of MB/s and one CPU core filters far faster than 22 MB/s. CPU decompression
  already runs 12–71× real-time.
- **GPU offload of streaming filters loses to PCIe transfer** — "the maximum sampling rate is
  limited by PCIe bandwidth rather than the computations on the GPU." Shipping raw bytes across
  PCIe for a cheap filter and back is net-negative.
- GPU only wins when data stays resident across many ops — the *sorter's* regime (Kilosort4 does
  preprocessing per-batch on the GPU it's already sorting on). That is a different project.
- **Implication:** optimize for IO scheduling, decompression overlap, cache-friendly chunking, and
  avoiding copies — not AVX-512 microkernels.
- Re-opening this requires fusing preprocessing into a GPU-resident sorter — out of scope for v1.
