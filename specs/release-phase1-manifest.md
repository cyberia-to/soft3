---
title: phase-1 sibling-pin manifest and drift gate
tags: cyber, soft3, spec, release
crystal-type: spec
crystal-domain: cyber
alias: phase1.toml, sibling-pin manifest, clean-checkout gate
---

# phase-1 sibling-pin manifest and drift gate

`release/phase1.toml` is the file [[cyberia/dev]] § release train and the
root `CLAUDE.md` already name: it pins the git revision of every phase-1
component the release train closes over when cutting candidates for cyb,
cyber and soft3. A sibling whose checked-out `origin/<branch>` HEAD
diverges from its pin is a red gate; the fix is a bump pull request that
updates the pin, never a path edit in a working tree.
`scripts/check-phase1-drift.nu` is the gate that reads the manifest and
proves that comparison.

## scope

The manifest pins libraries phase 1's three release-train binaries (cyb,
cyber, soft3) depend on transitively by Cargo path dependency, per the
dependency closure `cyber/launch.md` computes in its component table. It
does not pin cyb, cyber or soft3 themselves — those three pin each other
directly, by path and version, in their own `Cargo.toml`s, and are versioned
by the bump-PR rule in the release train. It does not pin non-code owners
(bostrom's burial scripts, cybernode's ops) or components with no default
branch to track (`fs`, spec-only).

## format

```toml
phase = 1
updated = "<YYYY-MM-DD>"

[[sibling]]
name = "<component>"     # matches cyber/launch.md's component table
repo = "cyberia-to/<repo>"
branch = "<default branch>"
rev = "<40-char git sha of origin/<branch> HEAD as of `updated`>"
```

An entry may carry `note` in place of `rev` when a component has no pin
yet — today only `nu`, whose checkout has no git remote (`cyber/launch.md`
decisions log, 2026-09-18); wiring it needs a remote before it needs a pin.

## the drift gate

`scripts/check-phase1-drift.nu [--root <dir>] [--manifest <path>]` reads
the manifest and, for every entry that carries a `rev`, resolves
`origin/<branch>` inside `<root>/<name>` (default root `~/cyber`, the
layout every sibling is checked out under on the release machine) and
compares it to the pin. It prints a name/pin/live/status table and exits
non-zero if any pinned sibling has drifted or its checkout is missing or
unresolvable; entries carrying `note` instead of `rev` are reported as
skipped and never fail the gate. A future CI job that clones each sibling
fresh at its own origin tip and runs this script against that clone
layout is the full release-train clean-checkout gate; run against local
checkouts today it already proves the comparison the gate performs.

## updating

One bump touches this file's `rev` (and `updated`) for the sibling that
moved, alongside that sibling's own version bump elsewhere, in the same
pull request — the same rule the release train applies to cyb/cyber/soft3
cross-pins.
