---
title: Phase-one source manifest
tags: soft3, release, spec
---

# Phase-one source manifest

`release/phase1.toml` pins each shared component by checkout name, GitHub
repository, default branch and full Git revision. Checkout names preserve sibling
Cargo paths; `tok` maps to `cyberia-to/plumb`. Build tools and forks are inputs too.
A missing revision or inaccessible origin remains an explicit red gate.

The three products are captured separately from origin when cutting a candidate.
Their revisions and manifest versions are recorded in `candidate.json`.
Consumers pin soft3 in `release/soft3.toml`; the shared engine checks this pin
against the captured soft3 revision and version. See [[specs/releases]].

Pins are changed through coordinated bump PRs. Release jobs never repair drift
by rewriting local manifests or silently choosing feature branches.
