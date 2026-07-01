# ADR 0017 — batch-sizing default for the preprocessing pipeline

**Status:** Accepted

## Context

The streaming preprocessing chain (ADR 0013) processes `batch` time-chunks per Rayon
parallel step. `src/lib.rs:54` treats `batch == 0` as a sentinel meaning "auto = 
`rayon::current_num_threads()`", i.e. one in-flight slab per logical thread. Resident memory
is `~0.17 GB × batch + ~0.5 GB base` for the 385-channel, 30 kHz, 1 s-chunk configuration —
memory scales linearly with `batch` and is otherwise independent of recording length.

The full-scale (29 GB / 55.8 min real IBL) benchmark
(`docs/research/2026-07-01-full-scale-si-comparison.md`) showed the auto default is a poor
choice on this class of machine (8 physical / 16 logical cores):

- At `batch == 0` the default became **batch 16**, giving the worst result on both axes —
  3.288 GB peak and 1068 s — because the work is memory-bandwidth-bound (ADR 0013) and
  parallelism past the physical-core count oversubscribes bandwidth while inflating memory.
- A pinned **batch 4** was the optimum: 1.194 GB and 806 s — faster than batch 8 (935 s) and
  batch 16, and using far less memory. batch 4 also beat both SpikeInterface modes on both
  memory and throughput.
- The memory ceiling was confirmed **file-size-independent** (batch-1 peak moved +0.9 % from a
  10-minute slice to the full 55.8-minute recording).

The old 0.99 GB SC1 figure (ADR 0013) was an 8-core measurement — i.e. batch 8, not a
different memory regime. The lesson is that the headline memory number is a function of
`batch`, and `batch` under the auto default is a function of the host's logical-thread count,
so an unpinned benchmark is not reproducible across machines.

## Decision

- **Pin the throughput/memory benchmark harness to an explicit batch.** `bench/bench.py` and
  `bench/segovia_chain.py` now default `--batch 4` instead of `0`. Benchmarks and any figure
  or claim in the paper always state an explicit batch; the auto default is never used to
  produce a reported number. (The online replay harness already pins `batch = 1` for true
  online operation and is unchanged.)

- **Defer changing the library default.** `src/lib.rs` keeps `batch == 0 → 
  rayon::current_num_threads()` for now. Whether a better default exists (cap at physical
  cores, a fixed small default, or `num_threads / 2`) is a genuine design question that
  depends on multi-machine data we do not yet have — the observed optimum (batch 4 on 8
  physical cores) is machine-dependent and a single-host result must not be generalized into a
  behavioral change affecting every user. This ADR records the finding and the deferral; a
  future ADR may change the library default once a small cross-machine sweep exists.

- **Document `batch` as the memory/throughput tuning knob.** It is the primary lever for the
  bounded-memory story: lower `batch` for a tighter memory ceiling, raise it toward the
  physical-core count for throughput, past which memory-bandwidth saturation makes it
  counterproductive.

## Consequences

- Benchmark numbers become reproducible across machines because the parallel width no longer
  silently tracks the host's logical-thread count.
- The paper reports Segovia at a pinned batch and presents the memory-vs-batch relationship as
  a deliberate, tunable property rather than a single fragile number — a stronger and more
  honest systems-paper claim than the earlier "always < 1 GB".
- No library behavior changes; existing users of `batch = 0` see no difference. The
  library-default question remains open and is explicitly a future decision, not silently
  resolved here.
- No new dependency, no ABI or data-contract change; this is a harness default plus a recorded
  decision.
