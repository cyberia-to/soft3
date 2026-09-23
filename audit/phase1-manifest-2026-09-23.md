---
title: phase-1 sibling-pin manifest — first capture
tags: cyber, soft3, audit, release
crystal-type: audit
crystal-domain: cyber
---

# phase-1 sibling-pin manifest — first capture

`release/phase1.toml` did not exist before this. `cyber/launch.md`
(controls) and the root `CLAUDE.md` name it as the file that "pins the
sibling revisions the train closes over," and `cyberia/dev.md` § release
train repeats the same clause — but nothing had written it yet.

## revision

25 entries: the 24 components of `cyber/launch.md`'s dependency-closure
table (2026-09-18) that are Cargo path dependencies of cyb, cyber or
soft3, minus cyb/cyber/soft3 themselves (they pin each other directly in
their own `Cargo.toml`s, not through this file — see
`specs/release-phase1-manifest.md`), plus `nu`, recorded with a `note`
instead of a `rev` because its local checkout still has no git remote
(decisions log, 2026-09-18). `fs` (spec only), `bostrom` and `cybernode`
(non-code, no Cargo dependency) are out of scope by the same spec.

## how the pins were captured

Command run per sibling directory under `~/cyber`, 2026-09-23:

```
git -C <repo> remote get-url origin
git -C <repo> symbolic-ref refs/remotes/origin/HEAD   # → default branch
git -C <repo> rev-parse origin/<default branch>
```

Every `rev` here is that command's output, taken from each repo's local
clone under `~/cyber` after `git fetch origin` — i.e. `origin/<branch>`
HEAD at capture time, not a working-tree commit. Cross-checked: every
`rev` is 40 hex characters.

## what this does not yet do

There is no gate reading this file yet — no CI step compares a sibling's
current `origin/<branch>` HEAD against its pin here and fails on drift.
That comparison is `cyberia/dev.md`'s "clean-checkout gate in CI," still
open per the decisions log (2026-09-23) list of release-train steps left
to build. This manifest is that gate's input, not the gate.

## limitations

Captured from one machine's clones at one moment; every sibling not
already at the HEAD recorded here as of 2026-09-23 needs a bump PR before
the first candidate (2026-09-25) can treat it as pinned, not drifted.
