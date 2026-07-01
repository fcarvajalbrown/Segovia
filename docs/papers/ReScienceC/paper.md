# [STUB] ReScience C — paper.md

This file is a stub. ReScience C publishes REPLICATIONS ONLY (see SUBMISSION-GUIDE.md).
A proper ReScience C submission requires identifying a specific prior computational study to
replicate and demonstrating that the replication succeeds (or fails and why).

Segovia as a new tool does not fit this venue's scope. This stub exists in case a replication
use case is identified. Two scenarios where this would become a real submission:

**Scenario A — Replicate a SpikeInterface benchmark.**
Identify a published study that benchmarked SpikeInterface's preprocessing chain (memory,
speed, correctness) and replicate those results using Segovia as an independent implementation
of the same operations. Document any differences in outcome.

**Scenario B — Replicate a MEArec simulation.**
Identify a published study that used MEArec to generate synthetic ground-truth recordings and
replicate those simulations using Segovia's `SyntheticEphysReader` as the independent
streaming simulator. Show agreement or divergence.

---

If a replication target is chosen, the paper.md for ReScience C must follow the LaTeX template
at github.com/ReScience/template and be submitted via
https://github.com/ReScience/submissions/issues.

Required YAML metadata (LaTeX front matter, not Markdown):
- Title with [Re] prefix: "[Re] Replication of: <original title>"
- Authors with ORCID
- Original paper: full citation of the study being replicated
- Keywords
- Code repository (with Zenodo DOI)
- Data availability

Do not convert this stub to a real paper without first identifying the replication target
and confirming with the ReScience C editorial board that the scope is appropriate.
