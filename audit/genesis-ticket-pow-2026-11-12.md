---
title: genesis under the settlement ticket — the three blessed identities of 2026-11-12
tags: cyber, soft3, audit, genesis, chain, mining, pussy, bostrom
crystal-type: audit
crystal-domain: cyber
---

# genesis under the settlement ticket — the three blessed identities of 2026-11-12

Date: 2026-11-12. Contracts: `specs/chain.md` §0/§1.1 (amended 2026-11-04),
tru `specs/rewards.md` §7, cybernode `tasks/ticket-pow.md`. This page is the
receipt `conformance/chain-genesis.toml` and `conformance/chain.snap` are
blessed against in soft3#682; it is a record assembled from the two cybernode
receipts named below and from the pinned siblings, not a new measurement on
this desk. Every row says what confirms it.

## revisions

| | |
|---|---|
| soft3 | `feat/ticket-pow` `8dd579b` (this pull request's head) |
| cybernode | `feat/ticket-pow` `f42382d` (review round 1 head, the dev-chain receipt) and `ad13862` (cybernode#18 head, R18-17, the alpha restart) — the two revisions that built these files |
| tru | `feat/coalition-marginal` `3cd4e45` (tru#46, the coalition game and the marginal) |
| bbg | `be5c5db` (master, #51: DIMENSION_VERSION 4) — unchanged by this blessing |
| hemera | `7237a23` (release/0.4, profile #15) — unchanged |
| lens | `release/0.2` — unchanged |

Machine: build-a (x86_64 Linux, `cargo build --release`). Command:
`cybernode genesis build --chain <c> --snapshot <burial> --out <file>`, the
burial snapshots unchanged from 2026-10-17. The receipt inputs (the rebuilt
`.tade` files and their `.json` reports) stay on build-a under
`/root/.cache/cyber/genesis/`.

Sources: cybernode `audit/chain/ticket-pow-2026-11.md` § revisions (the
two-node dev receipt, under `f42382d`) and cybernode
`audit/chain/alpha-tickets-2026-11-12.md` § revisions, § genesis (the alpha
restart, under `ad13862`).

## what moved the identity, and what did not

The genesis file's `params` record already carried a `bits u32`
(cybernode `rs/src/genesis/file.rs` line 6, the params layout, and the
`bits` field). Before this change every file was written with the single
constant `GENESIS_BITS = 0x1e010000`; now the field carries the chain's own
value — pussy `0x20020000` (2^249), bostrom the floor `0x20100000` (2^252),
any other name pussy's (`genesis_bits(chain)` in cybernode
`rs/src/chain/params.rs`). **The field is not new, so the file's length does
not move**: `file_bytes` is unchanged for all three chains while the
`file_sha256` and the particle (the chain id, derived from the params) do.

Block 0 grew: `block0_header` went from 168 to 200 bytes, the last 32 being
the sample, zero because block 0 is not mined (cybernode
`rs/src/genesis/file.rs`, `block0_header`). So `block0_id` moves for every
chain.

The state roots do not depend on the params, and none of them moved.

## the three identities

| | pussy | bostrom | pussy-rc |
|---|---|---|---|
| bits | `0x20020000` (2^249) | `0x20100000` (2^252, the floor) | `0x20020000` (2^249) |
| particle | `a94ee10120f2d24009f091eb5f620de0e65f67ef2619073988766d50be566d7f` | `1f1ef0f66166bdbb3fe63f90ce8d08c51598e92a86f58201d3aaed5e9abadd05` | `c1950dee724b600bafed258c7932c0d62248b87d0d5f59f3bcef8cb4760be143` |
| file sha256 | `640cf34d19742dc74c20be2c7811ab6a1d0952f313deec34dbdf50d2a1244dbd` | `25b8163aae54210d6391fc8f36ef6a6db1b595f80bf728524520fa080fba384a` | `2923fcd3a76abcf3b1906c5830c39556dc08cde2707f017dad7854d070fc6388` |
| file bytes | 3,249,657 (unchanged) | 330,841,520 (unchanged) | 3,249,660 (unchanged) |
| block 0 | `b5aca6a0446c3fd3080dc8cac4a03c3b0bafc41fe9847b9779f19a6faaeb6e06` | `48d5f86c9b82c11d7f0267e19b1961eff0eb3b4682eec10199deb7179f57439b` | `ff4b56ea65f8b81f86c8e58b7d3c84d1fe4e48676358efe720fa21f8ebf213b3` |
| state root | `d48fee47…130371` (unchanged) | `fa0c2c94…d7508d` (unchanged) | `d48fee47…130371` (pussy's) |
| root build | — | 252 s on build-a | 2 s on build-a |

What confirms each, beyond this blessing:

- **pussy** — the same four values are compiled into cybernode's own test
  `pussy_genesis_matches_the_blessed_vectors_when_cached`
  (`rs/src/chain/genesis.rs`), which decodes the cached `pussy.tade` and
  asserts the chain id, `params().bits == 0x20020000`, the state root and
  the block-0 id. The release gate `stack-chain-genesis` runs the same
  comparison against a freshly built file.
- **pussy-rc** — built twice from the same snapshot, byte-identical: once at
  cybernode `f42382d` (cybernode `audit/chain/ticket-pow-2026-11.md`
  § revisions) and once at `ad13862` for the alpha restart (cybernode
  `audit/chain/alpha-tickets-2026-11-12.md` § genesis, which records the
  same particle, sha256, 3,249,660 bytes and block 0). Both alpha hosts
  checked `sha256sum` before start, and edge-b's `/status` after the restart
  reads genesis `c1950dee…` with tip `ff4b56ea…`.
- **bostrom** — **not confirmed outside this blessing.** The file was rebuilt
  on build-a in the same run (252 s for the root; the `.json` report is in
  `/root/.cache/cyber/genesis/` on build-a), but no second build and no test
  pins these four values. bostrom is not a launch chain of phase 1 and has
  no stand; its identity is re-derived whenever `stack-chain-genesis` is run
  against a bostrom burial.

## the rehearsal stand restarted

Changing `[pussy-rc]` restarts the stand from block 0. It did: the alpha
rehearsal network restarted on both production hosts (edge-b, agent-a) at
2026-11-12 16:29 UTC on this genesis, the grind chain of 32,519 blocks kept
beside it as `pussy-rc.grind-20261112` (cybernode
`audit/chain/alpha-tickets-2026-11-12.md`). The stand's previous genesis
`5614749e…b713` — the one `audit/release-2026-11-02/director.md` records it
booted at — is superseded by `c1950dee…e143`. The stand's genesis **time**
(1793523600, 2026-11-01T09:00:00Z) is unchanged, which leaves the ASERT
anchor eleven days behind the clock: the chain mines at the floor
`0x20100000` from height 3 until the schedule catches up.

## what moved in `conformance/chain.snap`

Three groups moved, and only one of them is the genesis identity:

1. `chain::genesis@v1` "pussy particle" and "pussy block0_id" — the identity
   above. The pussy weights, φ*_0 and state root rows did not move.
2. `chain::state@v1` "conformance genesis file" and "conformance block0 id" —
   the conformance net's own genesis is written with `bits: GENESIS_BITS`
   (cybernode `rs/src/chain/economy/vectors.rs`, the `genesis()` params), so
   its bytes move with the constant and its block 0 with the 200-byte header.
   "conformance genesis state_root" did not move, as expected.
3. `chain::economy@v1` — eleven entries moved (ten distinct digests: "two
   epochs: epoch 1 record" and "reorg above the boundary: epoch 1 record" are
   the same record and carry the same `h264dbc6f…`), one is new ("two epochs:
   state root at 35") and one is gone ("two epochs: state root at 25", below).
   **The tru epoch
   engine pin is unchanged and correct**: `tru-epoch-e660d1e` is still the
   id cybernode's `rs/src/chain/economy/focus.rs` returns, and the alpha
   receipt records `/economy` reading `tru-epoch-e660d1e` on the built
   binary. The economy entries moved for cybernode's reasons, not tru's:
   - settlement moved from `settle(E) = boundary(E) + D` to
     `settle(E) = boundary(E + 1) + D` — an epoch's game is sampled over the
     *next* epoch's blocks (rewards.md §7). The scenarios' settlements
     therefore sit at heights 25 and 35 where they sat at 15 and 25, which
     is why "two epochs: state root at 25" is replaced by "state root at 35"
     and every epoch record and reorganization vector is taken at a new
     height;
   - the scenarios' `MIN_LOCK` moved 15 → 25 (≥ 2L + D) to stay consistent
     with that;
   - settlement now takes the window's lottery as an input.

### the one line removed by hand: "two epochs: state root at 25"

The bless run does not delete a vector the engine stopped producing. `bless`
writes `merged(committed, current)`, and `merged` only *replaces* committed
entries of the same `(key, scenario)` and **keeps the rest** (cybernode
`rs/src/chain/conformance.rs:220-226`); `drift`, and with it `check` and the
`committed_chain_snap_matches_this_build` test, ranges over `current` alone
(same file, 229-239), so a committed row no current entry regenerates is
never compared and never reported. "two epochs: state root at 25" is such a
row: at cybernode `ad13862` the economy vector set pushes
`("two epochs: state root at 35", d.roots[35])` and nothing else named
"state root at" in that scenario (`rs/src/chain/economy/vectors.rs:279`), so
the height-25 row survived the 2026-11-12 run untouched, at the digest the
pre-ticket engine gave it. Under the ticket, height 25 is where epoch 1
settles and the scenario's locks moved with `MIN_LOCK`, so that digest is a
state this engine does not produce. The line is deleted here and nothing
re-adds it.

The listing this leaves is exact: the vector set of `ad13862` emits twenty
`chain::economy@v1` entries — the three signed transactions, the four
conformance genesis weights and φ*_0 rows, the six "two epochs" epoch rows,
"two epochs: state root at 35", the two reorganization records, the two
"new high" rows and the two "random 2,000 blocks" rows — and the file now
carries exactly those twenty. The other sections are unchanged in shape:
two `chain::tx@v1`, four `chain::state@v1`, twenty-one `chain::pool@v1`,
seven `chain::genesis@v1` (pussy, from the pinned file). Re-blessing on this
file is therefore a no-op apart from the date in line 1.

## why the payout digests are empty

Under the ticket, an epoch pays only when it created value: when the clipped
share sum and `v(N)` — the value of the grand coalition,
`J(φ*(A_E)) − J(φ*(A_{E−1}))` — are both positive the shares are rescaled to
`v(N)`; **otherwise nothing is paid and the pool carries** (cybernode
`rs/src/chain/economy/settle.rs`, the payout rule in the module comment and
the rescale). The old rule divided the budget among the earners with no such
gate, so epoch 1 of "two epochs" paid its whole budget.

The conformance economy net is three particles in a cycle — `boot → pussy`,
`pussy → cyber`, `pussy → boot`, `cyber → boot` (cybernode
`rs/src/chain/economy/vectors.rs`, `genesis()`). It is not a sink net, and
the three locks of the "two epochs" scenario (one `+1`, one valence `0`, one
`−1`) do not leave that epoch's game with a positive value, so epoch 1 now
falls on the carry branch: budget 2,183,216, paid 0, pool after 8,732,353.
Epoch 2 of the same scenario already paid nothing before this change (it has
no stake change, hence no players). This is the same effect cybernode hit in
its own suite: two payout tests had to be redrawn onto a sink graph
"where every lock points into the sink so each epoch's v(N) > 0", because
the small net "could not create value" (cybernode `f42382d`, R18-6).

The blessed suite has **not** stopped covering payment. `chain::economy@v1 ::
"new high: first lock and relock paid"` is `u64s([first, again])` and reads
`7cb81d00…` — first = 1,947,260, a non-zero payout at the epoch-1 settlement
of the new-high scenario (one lock, `boot → pussy`, which does create value);
the second amount, the relock's, is 0. A staked settlement with a non-zero
payout across epochs is exercised in cybernode by
`multi_epoch_settlement_end_to_end` and
`settlement_pays_by_the_sampled_mean_at_k_min`.

## what this page does not have

- bostrom's build report (particle, sha256, bytes, block 0 re-read from the
  file) from a second machine or a gate run.
- a train or build log id: these three files were built by hand on build-a,
  not by a train leg. The train reproduces `pussy-rc.tade` on every stack leg
  (`stack-chain-genesis`), so the first green stack leg after the cybernode
  pin moves is the independent check of `[pussy-rc]`; pussy's is
  `stack-pussy-identity`.
