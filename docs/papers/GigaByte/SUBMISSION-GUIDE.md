# GigaByte — Submission Guide

Source: gigabytejournal.com (fetched 2026-06-30). Verify APC and article types at
gigabytejournal.com/open-access-and-apc and gigabytejournal.com/for-authors before submitting.

## Cost

APC: $535 USD (2025 rate; verify current rate before submitting). Waivers available for
authors without funding — contact editorial@gigabytejournal.com before submitting to request
one. No VAT added (Hong Kong-based). Payment by bank transfer or credit card.

## Article types

Two types currently published:

- **Data Release** — focused on datasets.
- **Technical Release** — software and computational workflows. This is the correct type for
  Segovia.

GigaByte expects iterative research objects (code, release, fork, update, repeat). Segovia's
versioned crates.io + PyPI releases fit this model.

## Evaluation criteria

- Is the information usable to both broad and specialist communities?
- Is the work scientifically sound?
- Are all associated research objects open, accessible, and FAIR?

## Format and platform

Correction (verified 2026-07-07 against gigabytejournal.com/information-for-authors and
/technical-release-description): GigaByte **accepts LaTeX, Word (DOC/DOCX), and PDF** — the earlier
"XML-first, no template" note was wrong. There is a Technical Release **Overleaf template** using the
`oup-contemporary` document class (`\documentclass[a4paper,num-refs,gigabyte]{oup-contemporary}`); the
`.cls` is not viewable without an Overleaf login. If submitting LaTeX, upload the PDF plus the `.tex`,
`.cls`, `.bib` files and figures as a zip. Submission is via the online system at
https://gigabyte-review.rivervalleytechnologies.com/. Peer review is questionnaire-style with optional
open commenting; post-publication review via Hypothes.is; reviewers receive DOI credit.

**Required Technical Release structure (in order):** Title/authors → Abstract (~150 words, with an
"Availability and Implementation" subsection) → Research Area and Classifications → Statement of Need →
Implementation → Availability of Supporting Source Code and Requirements (mandatory metadata block:
Project name, Project home page, Operating system(s), Programming language, Other requirements, License,
RRID, bio.tools ID) → Data Availability Statement → List of Abbreviations → Declarations → Endnotes →
References (FORCE11 software-citation style). Register the software at SciCrunch.org for an RRID.

## Data requirements

GigaByte requires data to be deposited in GigaDB (1 TB included in APC). For a Technical
Release of a software library, clarify with the editorial team what constitutes the required
data deposit — for Segovia this may be the benchmark datasets or the synthetic recordings.

## Scope fit

GigaByte covers life-science computational workflows and big-data research tools. The
neuroinformatics + Neuropixels angle fits the "life sciences" mandate. The software-release
paradigm (versioned, reproducible, open) fits exactly. The IFC cross-domain angle adds
breadth.

## Turnaround

Not confirmed from the fetched pages. GigaByte was founded on the premise of fast publication
("at the speed of research") — estimate 4–8 weeks from submission to decision.

## Key notes

- Scope is narrower than JOSS: life sciences computational tools, not general scientific
  computing. Confirm scope fit before investing significant writing time.
- LaTeX/Word/PDF are all accepted; a Technical Release Overleaf template (`oup-contemporary` class)
  exists but its `.cls` needs an Overleaf login to retrieve.
- Waiver is not guaranteed; contact the editors first.
