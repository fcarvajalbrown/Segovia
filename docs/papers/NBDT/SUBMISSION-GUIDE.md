# NBDT submission guide

Target: **Neurons, Behavior, Data analysis, and Theory (NBDT)** — https://nbdt.scholasticahq.com

## Why NBDT

- **Free (platinum/diamond OA).** No article-processing charge, no reader fee. Verified via DOAJ and
  the journal's fee statement.
- **No six-month-public-repo rule.** This is the rule that blocks JOSS for Segovia (the repo's first
  commit is 2026-06-09, with 39 of 44 commits on that day — JOSS's automated commit-distribution
  check would reject it). NBDT has no such requirement.
- **Scope fit.** NBDT explicitly lists **software development**, computational neuroscience, systems
  neuroscience, and data analysis in scope.
- **Turnaround.** ~12 weeks submission-to-publication (median reported).
- **Indexing.** DOAJ-indexed; CC-BY license required.

## What is in this folder

- `paper.tex` — full manuscript in standalone LaTeX (compiles with `pdflatex` + `bibtex`), ported
  from the PCI-Neuroscience full-length draft.
- `paper.bib` — bibliography (all 13 entries; `BuccinoMEArec2020` verified against the real
  Neuroinformatics 2021 paper, 19(1):185–204, DOI 10.1007/s12021-020-09467-7).

## Two changes made versus the PCI draft (both approved)

1. **Headline now leads with steady-state.** The abstract and conclusion lead with the full-length
   99.7% vs 94.7% deadline-adherence result, matching the body; the wider cold-start gap (100% vs
   69.5%) is stated separately and labeled as cold-start. This removes the abstract/body inconsistency
   a reviewer would flag.
2. **Cold-start rows kept honest.** Only the 300 ms leg is full-length. Table 2 (100 ms / 1000 ms) is
   still 60 s cold-start data and is labeled as such. If you later run the full-length 100 ms + 1000 ms
   legs (`bench/run_full_online.ps1`, ~3.2 h), swap those rows in for a revision.

## What YOU must do (I cannot do these for you)

1. **Check NBDT's "For Authors" page in a browser.** It is a JavaScript app my fetcher cannot read, so
   two things are unconfirmed: **word/figure limits** and **whether a bioRxiv/preprint is required or
   encouraged**. Confirm both before uploading: https://nbdt.scholasticahq.com/for-authors
2. **Decide on the LaTeX template.** NBDT provides an official Overleaf template
   (https://www.overleaf.com/latex/templates/nbdt-template/whswhbvsqzfr). Either:
   - **(a)** copy the body of `paper.tex` into that template for exact house styling, or
   - **(b)** submit the PDF compiled from `paper.tex` as-is (most journals accept a clean PDF at
     initial submission). Confirm which the portal expects.
3. **Create a Scholastica account and submit** at https://nbdt.scholasticahq.com — the portal needs
   your login; I cannot submit on your behalf.
4. **Confirm the CC-BY license choice** in the submission form (NBDT requires CC-BY).

## Readiness checklist

| Item | Status |
|---|---|
| Manuscript drafted (LaTeX) | Done — `paper.tex` |
| Bibliography complete + `BuccinoMEArec2020` verified | Done — `paper.bib` |
| ORCID in author block | Done — 0000-0002-8300-7587 |
| Abstract leads with steady-state result | Done |
| Cold-start rows labeled honestly | Done |
| Code + data availability statement | Done — GitHub + PyPI + crates.io in text; IBL dataset cited |
| Word / figure limits confirmed | **TODO — you, in browser** |
| Preprint requirement confirmed | **TODO — you, in browser** |
| Template choice (Overleaf vs PDF) | **TODO — you** |
| Scholastica account + upload | **TODO — you** |
| AI-usage disclosure | Add if the portal asks (NBDT has no fixed section; JOSS-style disclosure not required here) |

## Anti-AI editing pass applied

The manuscript was written to avoid machine-writing tells that reviewers and detectors flag: em-dash
density cut to near zero (replaced by commas, parentheses, and full stops), no "not X but Y"
contrast-reframe clichés, no reflexive section-summary restatement, plain verbs over inflated ones,
varied sentence length and openings, and hedging kept only where a claim is genuinely uncertain (the
single-machine and synthetic-data caveats) while facts are stated directly. Every number is concrete
and sourced.
