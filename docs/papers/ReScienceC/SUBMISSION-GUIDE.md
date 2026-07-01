# ReScience C — Submission Guide

Source: rescience.github.io (fetched 2026-06-30).

## SCOPE WARNING

ReScience C publishes REPLICATIONS AND REPRODUCTIONS ONLY. It is not a venue for new software
or new tools. Every article in ReScience C replicates the results of a previously published
computational study, using independently written, free, open-source software.

Segovia as a new tool does NOT fit ReScience C's scope.

## When this folder would become relevant

This venue only becomes relevant if one of these scenarios applies:
- You replicate the results of a published preprocessing study (e.g., reproduce a SpikeInterface
  or MountainSort benchmark) using Segovia as the implementation vehicle.
- You reproduce a published simulator (e.g., MEArec) and demonstrate it with Segovia's
  built-in streaming synthetic reader as the independent implementation.

If neither applies, do not invest writing time in this folder.

## Cost

Free. Platinum/diamond OA — no APC, no subscription. Volunteer-run via GitHub.

## Turnaround

Fast: weeks to months. Reviews are open and conducted via GitHub Issues.

## Format

LaTeX template available at github.com/ReScience/template. The required YAML metadata goes at
the top of the LaTeX source. Required metadata fields include:
- Title (must include "[Re]" prefix)
- Authors with ORCID
- Original paper reference being replicated
- Keywords
- Code repository (required — must be publicly accessible with a Zenodo DOI)
- Data availability

## Submission process

1. Write the replication article using the LaTeX template.
2. Upload code to a public repository (GitHub). Upload data to Zenodo.
3. Submit via GitHub Issues at github.com/ReScience/submissions.
4. Peer review is conducted publicly in the issue thread.
5. Upon acceptance, upload finalized code to Zenodo for a DOI.

## Key notes

- The [Re] prefix in the title is mandatory and signals this is a replication.
- Reviewers run the code and verify the replication independently.
- The article documents obstacles encountered, not just positive results — honesty about
  divergences from the original is required.
- ReScience C is indexed in DOAJ and citable, but does not have an impact factor.
