---
title: genesis under bbg dimension version 3 — both burials commit
tags: cyber, soft3, audit, genesis, bostrom, pussy, chain
crystal-type: audit
crystal-domain: cyber
---

# genesis under bbg dimension version 3 — both burials commit

Date: 2026-10-16. Contracts: `specs/chain.md` §3–4, `specs/genesis.md`,
bbg `specs/state.md` § dimension versions. Code: soft3 `feat/bbg-nonce-on`
(soft3 main `c79706a` — #193's pool record in the genesis params — + the nonce path on) over bbg `feat/dimension-v3`
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
| particle (chain id) | `1bb7d6aaf85330e77ba68044b7f9b683ac8a0a5936709cffa3820ce9714826a7` (moved by #193's pool record, not by bbg) | `d984145f3a4975eb4ebba55a35aa73c6641dfcf0bc5cebdb55d36a0578ea5901` (idem) |
| file | 3,249,657 bytes, sha256 `71747fcf…` | 330,841,520 bytes, sha256 `3b562479…` |
| **state root** | `93e4660f0ef0f5216d14554188cb8aded53d598d8c80d87e4e1a811949e61fe8` (was `f2296834…`) | **`ce3052f4f2e100a2d2e8edb8a7d38183e2fd0a5ca809ade693f34238df3da8d6`** (was blocked) |
| **block-0 id** | `5669e96ec2f93a600ebe8f3fdfbfaf6a2187a9626d550c3988cf1b4b6c92edd1` (was `40f267b5…` on main) | `be6f8900ca6394a491e735947106513eeb1f35e30900d88453e2fb6d942a1565` |
| build wall | 2.0 s (root 1 s) | 204–268 s (state load + root 147 s; 189 s user on the loaded machine) |
| peak memory | 67 MB | 7.5–7.6 GB |
| `verify --root` | reproduces | reproduces, 157 s, 4.9 GB |

The state roots do not depend on the genesis file's params, so they are the
same with and without #193's pool record (checked: both bostrom builds,
before and after merging #193, give `ce3052f4…`).

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

`soft3 conformance bless chain --genesis <pussy.tade>` — 5 of 30 entries
moved, all roots:

| entry | before (main `c79706a`) | after |
|---|---|---|
| empty state root | `65ff6132…4b2445` | `515dbb7e…ec375d` |
| conformance genesis state_root | `91ea5ef7…cdb3d7c` | `c7945550…08b601a75c` |
| conformance block0 id | `aa2120ec…53ae2237` | `b9527adf…cc294417` |
| pussy state_root | `f2296834…cbeb5827` | `93e4660f…49e61fe8` |
| pussy block0_id | `40f267b5…6da12d68` | `5669e96e…6c92edd1` |

The other 25 entries (signed txs, #193's pool formats, the pussy particle)
do not move.

`soft3 conformance check chain` is green against the rebuilt pussy file;
`cargo test --release --locked` (lib 215, wallet 12, mine e2e, adversary)
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
