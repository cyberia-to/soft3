---
tags: soft3, chain, audit, zheng, proven-present, pussy
crystal-type: audit
crystal-domain: crypto
date: 2026-11-01
serves: release/director/plan.md row P8
---
# the block relation for rule set 1 — counted on the native node, 2026-11-01

Plan row P8: the block relation (proven-present S1) for rule set 1, written
against `specs/chain.md` §5, and what a block proof costs with and without
the economy boundary. This page counts what the relation has to prove. It
does not build the relation and does not measure a proof. Every figure is
labelled measured (a counter the node ran), derived (computed from a counter
and a formula in source) or extrapolated.

## 1. what was measured

- Run [41311666610](https://github.com/cyberia-to/soft3/actions/runs/41311666610),
  workflow "Diagnostic P8 census" (`.github/workflows/diag-p8-census.yml`),
  branch `diag/p8-block-census`, commit `afdfcbc` (never merged, never in the
  stand), green. Log `census.log`, sha256
  `1ec1f6e7c268645aa9b97e7f1704edd2442e71dd2fe8ed1a676c219fd2656f31`, the
  digest the run printed. 1,389 lines.
- Command in the run: `cargo test --locked --release --features opcount -p
  soft3 --lib census -- --nocapture --test-threads=2`, on `ubuntu-24.04`.
  Test `chain::tests::census::census` (`crate/src/chain/tests/census.rs`).
- Stand (`snapshot/sources.json`, candidate `candidate-20261101.1`,
  `rehearsal: true`): soft3 `afdfcbc`, bbg `2d7873c`, hemera `8559b52`, mudra
  `33f8029`, nox `41ba24f`, zheng `166a14e`, lens `d6dcc3f`, tru `464104c`,
  foculus `0551e3b`, cybergraph `fe115b3`, plus file, rune, radio, vault,
  simulation, tok (16 pins).
- The scenario is **not the live pussy-rc chain.** It is a deterministic range
  mined on the test harness (`Net::with_funds`, splitmix64 seed 8): 24
  accounts, 24 particles, 400 heights with 0–16 generated transactions per
  block (mix: transfers, bare and staked links, payouts), one block at
  `MAX_BLOCK_TXS` (height 150), the spec's clock (epoch 240, settle delay
  120), so the first boundary is height 240 and the first settlement 360.
  A second scenario, a short clock (L = 10, D = 5, 45 heights, 3 accounts),
  reaches kinds 3, 6 and 7 and four settlements. Every block is validated by
  `accept_block` on a second fresh chain with a cold signature cache
  (`clear_verified_cache`, census.rs:256), so counts are a validator's.
- 445 blocks: 400 main (397 ordinary, 3 special: 150 max, 240 boundary, 360
  settlement) and 45 short-clock. Native validation time on the runner:
  median 4 ms, max 229 ms (the 1,024-transaction block), 1,848 ms for the 400
  main blocks.
- Scripts: `/tmp/p8-scripts/{load,a1,a2,a3,a4,a5}.py` on the director host,
  run as `python3 -I aN.py` (stdlib). They parse the 445 `P8` lines, 70
  `P8-JOB`, 5 `P8-OUTCOME`, `P8-MAX`, `P8-SUMMARY`. Not in the repo. sha256
  prefixes: load `783ce7fa`, a1 `a629956e`, a2 `d8181a1f`, a3 `9b5b6f85`, a4
  `889af94f`, a5 `e2513cbe`. a1 is stats and fit, a2 by purpose and special
  blocks, a3 jobs and outcomes, a4 boundary comparison, per kind and the
  mismatch check, a5 split fits and the adjacency bracket.

What a counter means (`crate/src/chain/opcount.rs`):

- `hash.<purpose>.perms` is hemera permutations of every hash soft3 computes,
  `len/56 + 1` per input (opcount.rs:63, replayed against `hemera::hash` for
  lengths 0..400 by the test at opcount.rs:176). `calls` counts hashes.
  `uniq_*` counts once per distinct `(purpose, input)` inside one block's
  validation (opcount.rs:19-38). Unique is therefore the count if a relation
  shares every repeated hash; executed is the count if it recomputes them.
- `journal` hashes are undo before-images, not validity (opcount.rs:26); the
  unique totals exclude them. `sig_aux` is derived: one 33-byte hemera of
  `neuron_of(pubkey)` per secp256k1 verification actually run (sigs.rs:61-69).
- `derived.*` are the permutations bbg performs inside its tries, computed in
  `crate/src/chain/bbg_state/census.rs` from the touched keys and bbg
  `2d7873c` source: leaf = `2 + ceil(limbs/4)` per dirty leaf, branch = 2 per
  dirty branch, root fold 14, stats commit 3 (census.rs:89-138). Derived, not
  run inside bbg.

## 2. the relation, written against chain.md §5.5

| §5.5 step | what the relation proves | primitives (counter) |
|---|---|---|
| 1 syntax (§1.1, §1.3) | 168 bytes, version, chain, `bits` canonical | compact-target decode 5, encode 6 per block (`u256.target_decode`, `u256.target_encode`, header.rs:162-181), integer |
| 2 parent | `prev` known, height + 1 | none hashed here; the parent's own proof is S2, not S1 |
| 3 time (§1.4) | MTP rule, drift rule | integer compares, no counter; MTP needs 11 ancestor times as witness (open) |
| 4 target (§1.5) | `bits` equals ASERT(prev) | 1 ASERT evaluation (`u256.asert`, daa.rs:50): integer, cubic factor, i128 |
| 5 work (§1.2) | `block_id < target` | `hash.header`: 4 permutations unique (1 hash); the node computes it 6 times (24 executed); 1 compare, 1 work, 1 chainwork |
| 6 body (§1.6, §1.7) | counts, sizes, `tx_root` recomputed, coinbase position | `hash.tx_root`: leaf per tx plus node per pair (calls = 2n − 1) |
| 7 transactions (§2.7, §3.4) | decode, `txid`, NSIG1 over the statement, nonce, balances, kind rules, coinbase = subsidy + fees | `hash.txid`, `hash.sig_statement`, `sig.verify` (secp256k1), `hash.sig_aux`, `hash.key` (balance key, axon id, card id, stake key, neuron id); state reads and writes per dimension |
| 8 state (§3.3) | apply on the parent state, `state_root` equal | dirty leaves and branches per bbg dimension (balances, neurons, cards, time, coins, axons_out, axons_in), root fold 14, stats commit 3 |
| economy (chain-economy §2.4, §5.7) | snapshot at the boundary, settlement at `settle(E)` | section 4 |

Per validity step the transaction kinds differ little. Per transaction,
measured over all 400 main blocks (`per_kind_all_blocks`, unique permutations):

| kind | txs | bytes | txid | tx_root | sig_statement | key | sig verifies |
|---|---|---|---|---|---|---|---|
| 0 coinbase | 400 | 82 | 3 | 2 | 0 | 3 | 0 |
| 1 transfer | 2,507 | 224 | 5 | 5 | 4 | 1.89 | 1 |
| 2 cyberlink | 1,502 | 257 | 6 | 5 | 4 | 4.95 | 1 |
| 7 payout | 282 | 248 | 6 | 5 | 4 | 4.38 | 1 |
| 3 claim | 1 | 140 | 4 | 3 | 2 | 10 | 1 |

Kind 6 (unlock) is in the short-clock run only (2 txs, 4 sig_statement and 5
tx_root each). Kind 3 has 1 sample in each run: thin.

## 3. an ordinary block (397 blocks, measured)

Ordinary means not 150, 240 or 360. Permutations are unique unless marked.

| primitive | min | median | mean | max |
|---|---|---|---|---|
| transactions (incl. coinbase) | 1 | 9 | 9.2 | 18 |
| body bytes | 86 | 2,010 | 2,052.9 | 4,074 |
| NSIG1 verifications (`sig.verify`) | 0 | 8 | 8.2 | 17 |
| header | 4 | 4 | 4.0 | 4 |
| txid | 3 | 47 | 48.1 | 95 |
| tx_root | 2 | 58 | 59.5 | 119 |
| sig_statement | 0 | 32 | 32.9 | 66 |
| sig_aux | 0 | 7 | 6.7 | 16 |
| key | 3 | 36 | 35.8 | 80 |
| trie leaf (derived, all dimensions) | 6 | 110 | 110.8 | 243 |
| trie branch (derived, incl. adjacency inner) | 16 | 264 | 263.7 | 572 |
| root fold + stats commit (derived) | 17 | 17 | 17.0 | 17 |
| bbg axon id, member digest, card stake key (derived) | 0 | 20 | 21.3 | 63 |
| state, derived total | 39 | 406 | 412.8 | 873 |
| total, unique, no journal | 51 | 601 | 599.9 | 1,213 |
| total, executed (incl. journal) | 71 | 672 | 670.1 | 1,337 |
| ASERT / compare / work / chainwork | 1 each | 1 | 1 | 1 |

Executed against unique: header 24 against 4, key 71.4 against 35.8 (mean),
journal 13.1 (excluded). The state part is 69 % of the unique total (412.8 of
599.9 mean). Of the state part, branches are 64 %, leaves 27 %.

Fits over the 397 ordinary blocks (a1.py, a5.py, recomputed from the per-block
lines; the summary's numbers reproduce):

| quantity y against txs x | intercept | slope per tx | R squared |
|---|---|---|---|
| unique permutations, total | 74.58 | 56.96 | 0.883 |
| unique, non-state (header, txid, tx_root, sig, key) | −4.69 | 20.79 | 0.994 |
| unique, state derived | 79.28 | 36.17 | 0.776 |
| NSIG1 verifications | −1 | 1.000 | 1.000 |
| unique total with the adjacency lower bracket (section 6) | 71.40 | 49.30 | 0.924 |

The total fit matches the summary (74.58, 56.96, 397 points). R squared 0.88
says the line is a summary of a noisy cloud (largest residual 425), not a law.
Executed permutations fit 85.05 + 63.44 per tx. One NSIG1 verification per
non-coinbase transaction is exact.

## 4. the economy boundary

"Without" below is measured where the counter is attributed to a kind
(snapshot = kind 253, settlement step = kind 254, `opcount.rs:56-62`) and
otherwise taken from ordinary blocks of the same transaction count.

| block | txs | sig verifies | unique perms | executed perms | state derived | of which boundary work (hash) |
|---|---|---|---|---|---|---|
| ordinary, 1 tx (16 blocks) | 1 | 0 | median 74 (51–78) | 94 | median 62 | 0 |
| ordinary, 6 txs (23 blocks) | 6 | 5 | median 444 (283–624) | 490 | median 321 | 0 |
| ordinary median block (9 txs) | 9 | 8 | 601 | 672 | 406 | 0 |
| 240, boundary (kind 253) | 6 | 5 | **839** | 893 | 338 | 382 unique (383 executed) |
| 360, settlement (kind 254) | 1 | 0 | **1,094** | 1,165 | 286 | 796 unique (847 executed) |
| 150, max-count (section 5) | 1,024 | 1,023 | 19,992 | 29,207 | 3,018 | 0 |

Block 240 against a 6-transaction ordinary block: +395 permutations on the
median (+89 %). Its hash work minus the snapshot is 457, within the ordinary
6-transaction range. The snapshot is 383 card-key hashes (382 unique), 383 raw
balance reads, a scan of 383 cards, 383 axon-edge reads, a scan of 0 marks
(`snapshot.positions` 383, `snapshot.marks` 0). It writes nothing into a
committed dimension: the state-derived part (338) is the ordinary part.

Block 360 against a 1-transaction block: 1,094 against a median 74, **+1,020
permutations (14.8 times)**. The 796 of the settlement step are 622 unique key
hashes (673 executed) and 174 economy permutations (`hash.economy`, 86 calls:
consistent with 43 payout leaves, 42 payout nodes and one epoch record digest,
derived from `settle.rs:379`, `economy/mod.rs:164`; the split is not logged).
It reads 588 raw balances, scans 586 cards, writes 44 balances (20 distinct
keys: 60 leaf and 200 branch permutations), writes 270 marks and the one `time`
entry (4 leaf permutations). `time` and marks: `kernel.marks.write` 270 are
node-side kernel state (`bbg_state.rs:27`), not a committed dimension, so no
trie permutation is derived for them. Whether marks need authentication is
open (section 7). Outcome (`P8-OUTCOME`, height 360, epoch 1): 24 particles,
187 axons, 43 payouts of 1,315,592 from a budget of 1,369,860.

Short-clock settlements (4, same code): 20 economy permutations over 6 calls
for the one that pays 1, and 8 unique snapshot key hashes. Epochs 3 and 4 pay
0.

The settlement engine calls, measured off the block path (`P8-JOB`, 70 lines;
`opcount::solve`, opcount.rs:77; `economy/run.rs:132-165`). Per settlement,
on each of the two chains (miner and validator):

| scenario, epoch | graph | calls | solve ms (validator / miner) | job ms | hash.economy in the job |
|---|---|---|---|---|---|
| main, 1 | 24 particles, 187 edges, 383 positions, 0 prior marks | 1 focus, 4 gradient | 729 / 739 | 781 / 788 | 4 perms (φ digest) |
| short, 1–4 | 4 particles, 2 edges | 1 focus, 4 gradient | 548–579 | 551–598 | 1 perm |

Main-range calls: focus 75–82 ms, gradient 142–194 ms each. The short graph's
gradients run 118–129 ms on 2 edges, so a fixed cost dominates at this size
(48 `PHI_STEPS`); these times say nothing about pussy-scale graphs
(`economy-cost-2026-10-17.md`: 15,967 particles, 29,112 links, 24 steps, 0.82 s
for one forward solve there, one core). A settlement is 1 + 4 engine calls here
(each gradient call is one forward and one adjoint, chain-economy §5.1: 9
solves).

What a block relation would take. Proposal, not measured:

- Proven inside the block relation at 360: applying a given settlement result.
  That is the 174 economy permutations, the balance writes with their trie
  paths, the `time` record, the pool debit and conservation. About 1,094
  permutations and 0 signatures for this block.
- Taken as a separate proof: that the result is the φ* of the two snapshots.
  This is the engine work above (`focus`, 4 `gradient`) and 586 position
  reads, none of which is a hash count. The block relation takes the result's
  digest (the epoch record) as a public input and the φ* proof, a different
  relation, binds it. Whether that proof exists is not in this census.
- At 240 the relation proves that the 383 positions frozen are the cards
  under the block's `state_root` (383 reads against the root, unpriced here),
  because settlement reads nothing else (chain-economy §2.4).

## 5. the maximum block

P8-MAX, height 150: 1,024 transactions (1 coinbase, 829 transfers, 194
cyberlinks), 235,640 body bytes (under `MAX_BLOCK_BYTES` 262,144), measured
by the same run, validated in 229 ms.

| | measured at 1,024 txs | extrapolated from the ordinary fit |
|---|---|---|
| NSIG1 verifications | 1,023 | 1,023 (exact fit) |
| unique permutations | 19,992 | **58,403** (74.58 + 56.96 × 1,024) |
| of which non-state | 16,974 | 21,289 (−4.69 + 20.79 × 1,024) |
| of which state derived | 3,018 | 37,114 (79.28 + 36.17 × 1,024) |
| executed permutations | 29,207 | 65,046 (85.05 + 63.44 × 1,024) |

The extrapolation overshoots the measurement 2.9 times, and the whole error is
in the state part (12 times). The reason is the workload: 24 accounts and 24
particles, so 1,023 transactions touch the same few trie paths and keys
(unique key hashes 379 against 7,799 executed, sig_aux unique 24 against 1,023
executed). The 150 block also uses only transfers and bare links
(`simple`, census.rs:67). Read the extrapolation as the cost of a block of 1,024
transactions between distinct accounts with no shared paths (an upper reading),
and the measurement as the cost with full sharing. A mainnet-like block sits
between; this census cannot say where. The non-state part is structural:
per transaction txid + tx_root + sig_statement is 14 (transfer) to 15
permutations, and one NSIG1 verification.

## 6. pricing

P5 (`proof-auth-2026-11.md`) gives no cost per hemera permutation in rows or
constraints, and no cost of an NSIG1 verification: the secp256k1 circuit does
not exist (P5 §5: trident `secp256k1.tri` `8f5c1a7` is a stub). What P5 gives
is whole relations, none of them a block:

| P5 figure | value | status |
|---|---|---|
| one veil proof of a 4-limb hash, rows / entries | 34 / 8,633 | measured (P5 §1) |
| AUTH1 (tagged key, 5 secret atoms), rows / entries | 66 / 16,861 | measured (P5 §7) |
| proof bytes of those, linux x86_64 | 98,797 / 129,693 | measured |
| verify, cached key | 12.7–23.1 ms | measured |

So no count in this page is converted to rows. The counts stay counts:
**601 permutations and 8 NSIG1 verifications for the median ordinary block;
839 permutations and 5 verifications for 240; 1,094 and 0 for 360.** A price
needs two numbers P5 does not have: rows per hemera permutation in zheng's
relation compiler, and rows for a secp256k1 verification. Both are open
(section 8).

Measured on the native node, for scale only: 4 ms median validation, 8 NSIG1
verifications per ordinary block, 1,023 in the largest.

## 7. caveat: the adjacency counts are an upper bound

`pending_mismatch_blocks_total = 575` in the summary. Verified here (a4.py):

- 575 is the sum of the per-block, per-dimension mismatches
  (`state.pending_mismatch`, census.rs:130): 286 on `axons_out` and 289 on
  `axons_in`. The name says blocks, but the number counts (block, dimension)
  pairs: **292 distinct blocks** (291 main, 1 short-clock), 283 of them in
  both dimensions.
- In every one of them census-touched is larger than bbg's pending (286 and
  289 blocks, no block the other way).
- Totals over the 445 blocks: `axons_out` bbg pending 451 against census
  touched 1,242; `axons_in` 449 against 1,226. Balances (4,839), neurons
  (2,712), cards (940), time (5) and coins (445) agree exactly, block by block.

The cause is visible in code, not directly counted. `census::link` inserts the
particle into `touched.out` and `touched.inn` on every link, unconditionally
(census.rs:62-63). bbg `AxonIndex::add` returns before `touched()` when the
axon is already in the particle's set (`keyed.rs:417-421` at `2d7873c`;
`Trie::set` returns false on an equal digest, `trie.rs:290-298`), so a
repeated `(p, q)` link leaves the particle clean and pending does not count
it. The harness draws `(p, q)` from 24 particles (552 ordered pairs), so
repeats are certain. Indirect evidence: 965 adds by height 240, and 187
distinct axons in the snapshot at 240 (`P8-OUTCOME`). The census does not log
`add`'s return value, so the exact repeat count is **not measured**. The
census still charges, for a repeated link: the member digest (correct, bbg
computes it before `set`, keyed.rs:419), and the particle's dirty leaf, outer
branches and inner branches (not paid by bbg). Whether this accounts for the whole
gap (791 and 777 excess touches per dimension over 445 blocks) is not shown:
the size of the gap is open.

Brackets (a5.py). The upper bound is the table in section 3: adjacency
leaf, branch and inner-branch permutations as derived from census touched. The
lower bracket scales those three parts, per block and per dimension, by
bbg pending over census touched; member digests and axon ids stay (bbg pays
them). Scaling is proportional, branches do not shrink linearly, so the lower
figure is a bracket by proportion, not a proven bound.

| | upper (census touched) | lower (scaled by bbg pending) |
|---|---|---|
| adjacency permutations, 445 blocks | 45,004 (axons_out 22,452, axons_in 22,552) | 15,487 (axons_out 7,709, axons_in 7,778) |
| unique total, 397 ordinary blocks (sum) | 238,145 | 208,852 (−12.3 %) |
| unique total, all 445 blocks | 262,662 | 233,145 (−11.2 %) |
| ordinary median block | 601 | 541 |
| ordinary fit | 74.58 + 56.96 per tx | 71.40 + 49.30 per tx |
| block 240 | 839 | 721 |
| block 360 | 1,094 | 1,094 (no adjacency touched) |
| block 150 | 19,992 | 19,902 |

Adjacency is 17.1 % of the
upper total over the 445 blocks. Everything else in this page does not depend
on the bracket.

## 8. what remains

1. Row cost per hemera permutation and per secp256k1 verification: unmeasured.
   A relation of N chained permutations in zheng (rows, prove ms, proof bytes)
   would price sections 3–5. P5 §5 says the NSIG1 circuit needs to be written
   first.
2. The census counts the recomputation of dirty leaves and branches. A
   relation also needs the reads: balances, neurons and cards read per block
   (`state.balances.read` median 8, `neurons.read` 8, `cards.read` 4) and the
   old values under the parent root, as authentication paths. Not counted;
   they add to every figure here.
3. The adjacency gap (section 7): log the return value of `add`, rerun, and
   replace the bracket by one number. Check that `axons_*` outer and inner
   work for repeated links is what bbg really hashes.
4. The snapshot at 240 (383 positions and 0 marks here) and the 270 marks
   written at 360 are kernel state outside the 14 committed leaves. Whether
   the block relation proves them, and against which root, is a spec question
   (proven-present S1 asks every dimension authenticated by `R`).
5. The φ* relation (focus, gradient) is not in P8. At pussy scale one forward
   solve is 0.82 s on one core; a settlement is 9 solves per chain-economy
   §5.1. Its proof is the load behind "with the economy boundary".
6. Workload realism: 24 accounts and 24 particles saturate the tries; the max
   block shows it (section 5). A second run with many accounts, or with the
   mined pussy-rc range, would separate the extrapolated and measured
   columns. Kinds 3 and 6 have 1 and 2 samples.
7. MTP and drift (step 3) need 11 ancestor times as a witness; not counted.
8. Per-transaction NSIG1 in the relation is the dominant unpriced term (1,023
   in a full block). P5's proof-authorized kind would replace it with a
   96–131 KB envelope per transaction, so aggregation is this relation's job
   (P5 §4, option 2).
