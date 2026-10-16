---
title: genesis under bbg dimension version 3 — both burials commit
tags: cyber, soft3, audit, genesis, bostrom, pussy, chain
crystal-type: audit
crystal-domain: cyber
---

# genesis under bbg dimension version 3 — both burials commit

Date: 2026-10-16. Contracts: `specs/chain.md` §3–4, `specs/genesis.md`,
bbg `specs/state.md` § dimension versions. Code: soft3 `feat/bbg-nonce-on`
(soft3 main `136721a` + the nonce path on) over bbg `feat/dimension-v3`
(dimension version 3: keyed-trie root, `NeuronRecord.nonce`, no zero rows,
deferred hashing), zheng `release/0.4`, lens `release/0.2`, nox
`release/0.4`, foculus `feat/device-profile-choice`, cybergraph
`feat/durable-applications`, mudra `refactor/spell` — the BBG-GEN stand.
Machine: M4 Max, 48 GB, release build, shared with other agents' runs (load
average 30–70). Previous run: `audit/genesis-2026-10-15.md` (bbg `1d39975`,
dimension version 2 flat: pussy committed, bostrom `root_blocked`).

## results

| | pussy | bostrom |
|---|---|---|
| particle (chain id) | `2a83e54fe395ffbd924fc74a54a5c805eeec5396f97bf4c73fced8631a9e61b5` (unchanged) | `50d829b8d4a9237386645652336269b4e6755ffd53f6c4bdf32f2783f4dc75db` (unchanged) |
| file | 3,249,582 bytes, byte-identical to the 10-15 build | 330,841,445 bytes, sha256 `7b706793…` |
| **state root** | `93e4660f0ef0f5216d14554188cb8aded53d598d8c80d87e4e1a811949e61fe8` (was `f2296834…`) | **`ce3052f4f2e100a2d2e8edb8a7d38183e2fd0a5ca809ade693f34238df3da8d6`** (was blocked) |
| **block-0 id** | `ad7e5f84c8723187b3ba3417bdc205803ea021c59706a7494a62fc85e5cfb06b` (was `b863ce28…`) | `6e0d05e242e8b5017e5e57ace6fcb1975b38aab0b2e194014b821ea00aae070e` |
| build wall | 2.5 s (root 1 s) | 204 s (state load + root 147 s) |
| peak memory | 75 MB | 7.5 GB |
| `verify --root` | reproduces | reproduces, 157 s, 4.9 GB |

bbg dimensions (records): pussy — 0 particles, 6,685 `axons_out`, 15,061
`axons_in`, 26,805 axons, 547 neurons, 1,385 balances, 23 coins; bostrom —
0 particles, 1,701,082 `axons_out`, 1,772,963 `axons_in`, 2,990,817 axons,
60,452 neurons, 93,472 balances, 49 coins. The particles dimension is empty
at genesis: every genesis link has `amount = 0`, and under dimension version
3 a conviction-free link writes topology only — the content particles are
the keys of the adjacency dimensions. The two inflations of the 10-15 run
are gone: 3,195,815 zero balance rows (3,289,287 → 93,472, exactly the `b`
frames) and 4,763,780 zero particle records (axon-particles and link
targets).

Wall time before: the dimension-version-2 trie with eager hashing took
1,790 s and 8.57 GB for bostrom (bbg #41); the flat root could not commit at
all. Deferred hashing hashes every node once on a bulk load and runs the
dimensions on parallel threads.

## conformance re-blessed

`soft3 conformance bless chain --genesis <pussy.tade>` — 5 of 9 entries
moved, all roots (the chain id and the signed-tx fingerprints do not move):

| entry | before | after |
|---|---|---|
| empty state root | `65ff6132…4b2445` | `515dbb7e…ec375d` |
| conformance genesis state_root | `91ea5ef7…cdb3d7c` | `c7945550…08b601a75c` |
| conformance block0 id | `96c30f7a…bd17e5f` | `bd79d999…0508f470` |
| pussy state_root | `f2296834…cbeb5827` | `93e4660f…49e61fe8` |
| pussy block0_id | `b863ce28…b3eba304` | `ad7e5f84…e5cfb06b` |

`soft3 conformance check chain` is green against the rebuilt pussy file;
`cargo test --release --locked` (lib 208, wallet 12, mine e2e, adversary)
passes, the chain-conformance feature too.

hemera 0.4 (profile #15) moves every root once more; this table re-blesses
then.

## reproduce

```
soft3 genesis build --chain pussy   --snapshot ~/cyber/space-pussy/snapshot --out /tmp/gen/pussy.tade
soft3 genesis build --chain bostrom --snapshot ~/cyber/bostrom/snapshot     --out /tmp/gen/bostrom.tade
soft3 genesis verify /tmp/gen/bostrom.tade --root
soft3 conformance check chain --genesis /tmp/gen/pussy.tade
```
