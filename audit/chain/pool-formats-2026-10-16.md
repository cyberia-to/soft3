---
title: chain — pool formats frozen at genesis, re-blessed 2026-10-16
tags: cyber, soft3, audit, chain, conformance, genesis, privacy
crystal-type: audit
crystal-domain: cyber
date: 2026-10-16
---

# chain — pool formats frozen at genesis, re-blessed 2026-10-16

Receipt of the change that writes chain.md §11 (the pool, rule set 2,
formats frozen at genesis), appends the 74-byte pool record to the genesis
`p` frame, and adds chain-conformance §12. Requirements stay in the specs;
this page holds the observed values and the commands.

## stand

`~/cyber/.stands/P11`: soft3 `specs/pool-formats` over main `0c010a3`;
siblings at the integration base main is locked against — bbg `1d39975`
(DIMENSION_VERSION 2, no `NeuronRecord.nonce`), cybergraph
`feat/durable-applications` `0eae7ae`, lens `release/0.2`, honeycrisp
`main`, mudra `master` `186c0f2`, hemera `main` `d5f0a09`, strata `main`,
tade `main`. bbg `master` (`ecaebc1`) and cybergraph `master` do not build
with soft3 main (no vendored fjall; no `local-storage` feature), so the
siblings were not moved to their default branches.

## commands

```
cd soft3/crate
cargo test                                         # lib, bin and doc tests
soft3 genesis build --chain pussy   --snapshot ~/cyber/space-pussy/snapshot --cache ~/cyber/.cache/snapshots/pussy   --out ~/cyber/.cache/genesis/pussy.tade
soft3 genesis build --chain bostrom --snapshot ~/cyber/bostrom/snapshot     --cache ~/cyber/.cache/snapshots/bostrom --out ~/cyber/.cache/genesis/bostrom.tade
soft3 genesis verify ~/cyber/.cache/genesis/pussy.tade --root
soft3 conformance bless chain                      # 30 entries (9 before)
```

## observed

| item | before (2026-10-15) | after |
|---|---|---|
| pussy genesis particle (chain id) | `2a83e54fe395ffbd924fc74a54a5c805eeec5396f97bf4c73fced8631a9e61b5` | `b7740bd23b559f4016fd26b71113107bda3133be19efbacf728328498398108c` |
| pussy state root | `f2296834490e79d855673b18b0f9eb5c8242e4f48761e85b645d60a8cbeb5827` | unchanged — params do not enter the state |
| pussy block-0 id | `b863ce2877524e718bb51ea6dc7c5aba9540860f08d70162348d8e84b3eba304` | `50267e59ee8fc6ad1d342b6bb48de14f6ecda8a85785914a8b6c3016cb7abf78` |
| pussy file | 3,249,582 bytes | 3,249,657 bytes, sha256 `374ffa24c91b897a39d11b71bced75c211517e0752a09cfbfafe25f834bf26d6` |
| bostrom genesis particle | `50d829b8d4a9237386645652336269b4e6755ffd53f6c4bdf32f2783f4dc75db` | `e1af8603782407cb2444a1549892f61faa6697c93534f193a70e78d7f0ec122a` (330,841,520 bytes); root still blocked by the lens limit |
| conformance genesis file / block-0 id | `40cccd00…` / `96c30f7a…` | `2624d641…` / `9e96eec5…`; its state root unchanged |
| pool tags digest (64 B) | — | `9ecbb70a…34d0` |
| empty tree root `E_40` (64 B) | — | `80331b2b…3bdc` |

The pool digests are wide (64 bytes, the full rate of hemera's finalized
sponge) after the reconciled proposal (main `0c010a3`, §1.3: at least 48
bytes); a 32-byte draft of the same section existed on this branch for a
few hours and was never merged. A later change of bbg's roots
(DIMENSION_VERSION 3) moves the state roots and block-0 ids again, not the
particles; a change of hemera's profile (hemera#15) moves everything.
