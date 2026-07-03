## ADR 0018 — conservative OOM safety cap on the `batch == 0` auto default

**Status:** Accepted (supersedes the cap deferral in ADR 0017; the tuning-optimum question ADR 0017 raised stays open)

## Context

ADR 0017 established that resident memory of the preprocessing chain is
`~0.17 GB × batch + ~0.5 GB base` and that `src/lib.rs` treats `batch == 0` as "auto =
`rayon::current_num_threads()`" — one in-flight slab per logical thread. It pinned the benchmark
harness to an explicit `batch 4` but **deferred any change to the library default**, arguing the
*optimal* batch is machine-dependent and must not be generalized from one host.

That deferral conflated two separate concerns:

- **Tuning** — which batch is fastest on a given machine. This genuinely needs cross-machine data
  and remains deferred.
- **Safety** — the `batch == 0` default scales with the host's *logical thread count* with no upper
  bound, so memory is unbounded in the one variable a user cannot see when they accept the default.
  On the 16-logical-core development box the default became batch 16 → ~3.3 GB; on a 32- or
  64-logical-core server the same default would project to ~6–11 GB and can OOM a low-RAM host. This
  is a runaway-memory footgun, and it is independent of the tuning question — capping the default can
  only *lower* memory risk, never worsen it.

The auto default's own benchmark (ADR 0017) also showed it is the *worst* choice on both memory and
throughput on the one machine measured, so nothing is lost by capping it.

## Decision

- **Cap the `batch == 0` auto default at 4:** `src/lib.rs` computes
  `batch = rayon::current_num_threads().clamp(1, 4)` when `batch == 0`. The default now bounds
  projected resident memory to roughly `0.17 × 4 + 0.5 ≈ 1.2 GB` regardless of core count, instead of
  scaling without limit.
- **Scope: this is an OOM safety guard, not a claim of optimality.** The cap value 4 is chosen as a
  conservative bound that keeps the default's memory comfortably under ~1.5 GB on any machine; it is
  not asserted to be the throughput optimum on any particular host. The tuning-optimum question ADR
  0017 raised (whether a smarter default exists — physical-core detection, a RAM-fraction budget)
  remains open for a future ADR once cross-machine data exists.
- **Explicit `batch` values are unchanged.** Any caller passing `batch >= 1` is untouched; the cap
  applies only to the `batch == 0` auto path. Users who deliberately want wide parallelism still pass
  an explicit batch.

## Consequences

- The default configuration can no longer balloon memory on many-core / low-RAM machines — the
  headline footgun behind the aborted first full-scale run (ADR 0017 context) is removed at the
  source, not only in the benchmark harness.
- **Behavior change for callers relying on `batch == 0` for wide parallelism:** on machines with more
  than 4 logical threads the default now runs at batch 4 rather than one-slab-per-thread. This lowers
  the default's memory and, per the ADR 0017 sweep, does not cost throughput on the measured host
  (batch 4 beat batch 8 and 16 there); on a machine where more parallelism would help, callers opt in
  with an explicit batch. This is a `fix` (removes a runaway-memory defect), a patch-level change
  pre-1.0.
- No new dependency, no ABI or data-contract change; the whitening/filter math and output are
  identical. Only the auto-selected parallel width changes.
- The library default and the benchmark harness default (ADR 0017, also 4) now agree, so the
  out-of-the-box behavior matches the configuration the paper reports.
