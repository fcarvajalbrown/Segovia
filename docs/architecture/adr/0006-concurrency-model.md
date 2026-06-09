# ADR 0006 — Concurrency & evaluation model: Rayon-over-chunks, lazy graph deferred

**Status:** Accepted (phased)

## Context

The engine must process out-of-core in bounded memory with high CPU utilization. Options range
from a minimal eager chunk pipeline to a full deferred operation DAG with an optimizing scheduler
(see `candidate-architectures.md`, candidates D and C).

## Decision

Phase the concurrency/evaluation model:

1. **Phase 1 (M2–4):** eager **Rayon-over-chunks** streaming pipeline (Candidate D) — minimal,
   just enough to clear the SC1 benchmark gate.
2. **Phase 2 (M4–7):** add a **modest lazy operation graph** (Candidate A) — compose ops without
   materializing intermediates — but **no full optimizer**.
3. **Deferred:** a custom optimizing scheduler with fusion/pushdown (Candidate C) is adopted only
   where a concrete benchmark proves the simple pipeline insufficient (YAGNI).

## Consequences

- Time-to-first-benchmark is minimized; the project-killing question (SC1) is answered first.
- **Cross-chunk filter state** (IIR filters carry state across chunk boundaries) must be handled
  explicitly and made **deterministic across thread counts** (NFR7) — a first-class correctness
  concern from Phase 1, not an afterthought.
- Memory is bounded by chunk size × in-flight chunk count; the scheduler caps in-flight buffers.
- Avoids over-engineering an optimizer before value is proven; keeps a clean upgrade path D → A → C.
