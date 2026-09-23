---
title: phase-1 sibling-pin manifest
tags: cyber, soft3, spec, release
crystal-type: spec
crystal-domain: cyber
alias: phase1.toml, sibling-pin manifest
---

# phase-1 sibling-pin manifest

`release/phase1.toml` is the file [[cyberia/dev]] § release train and the
root `CLAUDE.md` already name: it pins the git revision of every phase-1
component the release train closes over when cutting candidates for cyb,
cyber and soft3. A sibling whose checked-out `origin/<branch>` HEAD
diverges from its pin is a red gate; the fix is a bump pull request that
updates the pin, never a path edit in a working tree.

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

## drift

The release-train gate reads this file and compares each `rev` against
the sibling's `origin/<branch>` HEAD at cut time. A mismatch fails the
gate; a component's own default-branch progress does not reach a
candidate until its pin here is bumped. The gate itself (the CI step that
performs this comparison) is a separate, later slice — this manifest is
its input.

## updating

One bump touches this file's `rev` (and `updated`) for the sibling that
moved, alongside that sibling's own version bump elsewhere, in the same
pull request — the same rule the release train applies to cyb/cyber/soft3
cross-pins.
