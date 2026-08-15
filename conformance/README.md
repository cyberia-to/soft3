---
title: conformance
tags: cyber, soft3, conformance
crystal-type: spec
crystal-domain: cyber
alias: conformance harness, stability snapshot, conformance crate
---
# conformance

stability harness of [[soft3]] — a release of the stack IS a conformance snapshot. one [[hemera]] fingerprint per canonical encoding, one per mechanism output. drift surfaces at commit time.

```
canonical(value)        →  hemera fingerprint  →  conformance/encoding.snap
nox(scenario, input)    →  hemera fingerprint  →  conformance/mechanism.snap
union(all snapshots)    →  hemera root         →  protocol stability root
```

each soft3 repo carries its own `conformance/` directory of `.snap` files. CI compares; drift fails. promotion through five tiers (alpha → epsilon) determines how hard a snapshot is to move.

## layout

| dir | purpose |
|-----|---------|
| [docs/](docs/) | why this crate exists |
| [specs/](specs/) | how snapshots are produced, compared, blessed |
| [rs/](rs/) | `cyber-conformance` — trait, tier enum, harness types |

## position in the stack

conformance sits beneath every other [[soft3]] component except [[hemera]] itself. [[bbg]], [[lens]], [[zheng]], [[radio]], [[foculus]], [[cybergraph]], [[nox]], [[wysm]], [[trident]], [[mudra]], [[mir]] each register their stable surface area as `Conformant` types and named mechanism scenarios.

## status

scaffold. trait surface drafted. snapshot harness lands after [[hemera]] reaches stable output. `cargo conformance` subcommand follows.

see [[soft3]] for the surrounding stack and [[hemera]] for the underlying fingerprint primitive.
