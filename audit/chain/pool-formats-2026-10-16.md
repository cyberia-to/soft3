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

Against hemera 0.3 (`d5f0a09`); to be re-blessed when hemera 0.4 (profile
#15, the profile the chain freezes) lands — every row below moves then,
except the state roots.

| item | before (2026-10-15) | after |
|---|---|---|
| pussy genesis particle (chain id) | `2a83e54fe395ffbd924fc74a54a5c805eeec5396f97bf4c73fced8631a9e61b5` | `1bb7d6aaf85330e77ba68044b7f9b683ac8a0a5936709cffa3820ce9714826a7` |
| pussy state root | `f2296834490e79d855673b18b0f9eb5c8242e4f48761e85b645d60a8cbeb5827` | unchanged — params do not enter the state |
| pussy block-0 id | `b863ce2877524e718bb51ea6dc7c5aba9540860f08d70162348d8e84b3eba304` | `40f267b58818ef051fb3c1dfe7f9e673e36c85974d17f565cbcbb3556da12d68` |
| pussy file | 3,249,582 bytes | 3,249,657 bytes, sha256 `71747fcfd2be17991e801864187e595ee62f37bd16f114c6544aeeb511e962ce` |
| bostrom genesis particle | `50d829b8d4a9237386645652336269b4e6755ffd53f6c4bdf32f2783f4dc75db` | `d984145f3a4975eb4ebba55a35aa73c6641dfcf0bc5cebdb55d36a0578ea5901` (330,841,520 bytes); root still blocked by the lens limit |
| conformance genesis file / block-0 id | `40cccd00…` / `96c30f7a…` | `f27c1519…` / `aa2120ec…`; its state root unchanged |
| pool tags digest (64 B) | — | `d4ea8f2b…816b` |
| empty tree root `E_40` (64 B) | — | `9a47bbbc…3aa5` |
| `chain.snap` | 9 entries | 30 entries (21 `chain::pool@v1`) |

The pool digests are 64 bytes, two squeezes of hemera's extendable output,
after the reconciled proposal (main `0c010a3`, §1.3: at least 48 bytes) and
the coordinator's choice of the hemera API over a hand-built node; the tree
node is `W(D_NODE ‖ L ‖ R)`, four permutations at hemera 0.3. Two drafts of
this section existed on the branch for a few hours (32-byte digests; a
full-rate squeeze with a two-permutation element node) and were never
merged. A later change of bbg's roots (DIMENSION_VERSION 3) moves the state
roots and block-0 ids again, not the particles.
