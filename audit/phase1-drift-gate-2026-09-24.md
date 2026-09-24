---
title: phase-1 clean-checkout drift gate — first run
tags: cyber, soft3, audit, release
crystal-type: audit
crystal-domain: cyber
---

# phase-1 clean-checkout drift gate — first run

Property #39 (`cyber/launch.md`) needs a real gate, not just the manifest
soft3#21 proposed: something that reads `release/phase1.toml` and fails
when a pinned sibling has moved past its pin. `scripts/check-phase1-drift.nu`
is that gate; this run proves it against a fresh capture of every pinned
sibling's live `origin/<branch>` HEAD.

## what ran

```
$ nu scripts/check-phase1-drift.nu --root ~/cyber
```

24 pinned siblings, all `OK`; `nu` reported `skipped: local checkout has
no git remote...` as designed, and did not count toward drift. Exit 0.

## negative-path verification

The gate was also run against a scratch manifest with one deliberately
wrong `rev` and one nonexistent sibling, to confirm it actually fails
instead of only ever printing `OK`:

```
$ nu scripts/check-phase1-drift.nu --root ~/cyber --manifest /tmp/drift-test.toml
...
FAIL — 1 drifted, 1 unresolvable, against /tmp/drift-test.toml
$ echo $?
1
```

## how the pins in this PR's `release/phase1.toml` were captured

Same method as the 2026-09-23 capture (soft3#21), re-run 2026-09-24:

```
git -C <repo> fetch -q origin
git -C <repo> rev-parse origin/<default branch>
```

Every `rev` re-checked to be exactly 40 hex characters. Every value is
byte-identical to the 2026-09-23 capture — no pinned sibling's default
branch has moved in the intervening day.

## what this does not yet do

The gate runs against this machine's `~/cyber` checkouts, not fresh
clones at each sibling's origin tip — `--root` defaults there because
that is the release machine's actual layout today. A CI job that clones
every sibling fresh before invoking this script is the remaining piece
of the full clean-checkout gate; the comparison logic itself, which is
what actually decides drift, is what this run verifies.
