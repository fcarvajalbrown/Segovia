# Journal of Statistical Software (JSS) — Submission Guide

Source: jstatsoft.org (site returned ECONNREFUSED at time of setup — verify all details at
www.jstatsoft.org/about/submissions before submitting).

## Cost

Free. Diamond OA — no APC, no subscription fee. The journal is run entirely by volunteers
and supported by UCLA Statistics, Universität Innsbruck, and Universität Zürich.

## Turnaround

~53 weeks average from submission to publication (confirmed from jstatsoft.org/authors and
DOAJ listing). This is the slowest of the five targets. Only pursue JSS if JOSS is rejected
and speed is not the primary constraint.

## Scope

Statistical computing and statistical software. The journal's scope is statistical methodology
and software implementing statistical methods. Segovia's scope (neuroinformatics streaming
preprocessing, signal processing) is adjacent but not a natural fit — reviewers expect a
primary statistical contribution, not a systems/streaming contribution. A scope mismatch
rejection is possible. Consider framing the whitening (ZCA) and CMR steps as the statistical
core if submitting here.

## Article types (verify at jstatsoft.org)

The journal publishes several article types; the most relevant for Segovia would be:
- **Application articles** — describe software that implements a method and demonstrate it on
  a real application. These have a LaTeX template and include code listings.

## Format

LaTeX template required. Template files (.cls, .bst) available at jstatsoft.org/about/submissions
(download before starting — the site was unreachable at time of setup; try again later or
email the editors). The standard article structure for an application paper is:
- Abstract
- Introduction
- Software design / Architecture
- Computational details
- Application / Evaluation
- Summary
- Acknowledgements
- References
- Appendix (code listings)

## Software requirements

JSS requires that submitted software be free, open-source, and installable. Segovia (AGPL-3.0,
on crates.io + PyPI) satisfies this.

## Submission

Via jstatsoft.org (online submission system). Reviewers typically test the software themselves
as part of the review.

## Key notes

- The ~53-week timeline makes JSS a fallback, not a primary target.
- The statistical scope makes this a stretch unless the paper is reframed around the ZCA
  whitening step or the signal processing methodology.
- LaTeX template is required; the current JOSS draft (paper.md, Markdown) would need to be
  rewritten from scratch for JSS.
