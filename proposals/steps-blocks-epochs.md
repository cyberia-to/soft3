---
tags: cyber, soft3, proposal, foculus, cybergraph, bbg, zheng, tok, tru, mining, consensus
crystal-type: process
crystal-domain: cyber
status: proposed
date: 2026-10-12
---
# steps, blocks, epochs — the consensus clock of soft3

> soft3 has proofs ([[zheng]]), a state organ ([[bbg]]), a graph ([[cybergraph]]), a compiler ([[trident]]), a mining spec ([[fold-mining]]) — and no working blockchain. the two live proof-of-work chains launched in 2026 made a set of launch-critical consensus decisions that soft3 never wrote down anywhere: quantus (QPoW, Poseidon2 over Goldilocks, 12 s) chose a simple pool-compatible work function with ZK-native reward addresses; parano1d (Poseidon2b, 30 s) chose proof-gated block production, consensus-level pruning, and state-growth fees. this page fixes the three-tier clock soft3 will run on — **step → block → epoch** — and the five child specs that must exist before the first line of chain code. target: 1 s network-wide latency at step cadence, finality at epoch cadence, hybrid pow/pos throughout.

## 1. the three tiers

| tier | what it is | produced by | proof | cadence | owner |
|---|---|---|---|---|---|
| **step** | the smallest proven state transition: one `apply` of [[cybergraph]] writes into [[bbg]] — particle insert, axon weight, focus debit, balance move | any proving node | one zheng public certificate per step (one CCS per step relation; the trace *is* the witness) | 1 s target, set from measurement (§7) | cybergraph apply · zheng · nox |
| **block** | a DAG unit: a step bundle ordered against k parents, plus the work ticket | nodes passing the sync-quorum gate | the step proofs inside it + the settlement ticket ([[fold-mining]] first lottery) | sub-second, racing | [[foculus]] |
| **epoch** | the finality checkpoint: fold decider over the accumulated steps + parameter retarget + rank recompute | decider provers + stake quorum | decider proof (second lottery of [[fold-mining]]) co-signed by staked cyberlinks | minutes | foculus · tru · tok |

the load-bearing decision: **from-genesis validity lives in the accumulator, not in each block.** parano1d re-proves history in every block (hence its 30 s); soft3 proves only the per-step `apply` at 1 s and lets the ARC fold chain steps. a step is therefore tiny — thousands of nox reductions, not millions — and the block interval budgets only the per-step proof plus propagation. ranking ([[tru]] φ*, cyberank) is **never** inside a step; it recomputes once per epoch. reads ([[axon]], attention, query) are never in the trace at all.

## 2. module map — who owns what

- **cybergraph** owns the step relation: the exact field set of an `apply` (already fixed: particle energy, axon weights, focus debit, box ownership — `cybergraph/specs/apply.md`). it never constructs proofs, it verifies them.
- **nox/trident** own the metering: one field op = one reduction, `trident build --costs` gives the static count before run. the block class is a **reduction budget** (§3c).
- **zheng** owns the per-step certificate, exactly as repaired in [[proof-system-repair]]: one field (Goldilocks), one hash ([[hemera]]), one code (Reed–Solomon via nebu NTT), Spartan over CCS. the soundness repair is a hard dependency (§6).
- **foculus** owns ordering (the DAG), the block-attempt state machine (§3a), the DAA (§3b), retention (§3d) — after [[network-planes]] this is the `sync`/`foculus`/`solong` planes.
- **tok** owns value movement inside a step and the state-growth fee rule (§3e).
- **tru** owns the epoch: rank recompute, and — with foculus — the stake-signed finality quorum (§4).

## 3. the five child specs

each becomes its own page before chain code; this section fixes what each must decide.

### 3a. block-attempt state machine (foculus)

what one attempt is, and every way it dies. required content, from parano1d's documented lifecycle (docs.parano1d.org/mining) adapted to the DAG:

1. sync-quorum gate: no attempt without an authenticated peer set and a current view — never mine an isolated local view.
2. mempool admission checks *before* inclusion: authorization, spendability, limits, fees.
3. freeze all semantic fields of the candidate; compute slot writes and post-state root.
4. prove the step(s); freeze the immutable template.
5. nonce/ticket search; on a new accepted tip, **all in-flight attempts become stale and are discarded — never re-parent a proof**.
6. atomic commit of `{block, proofs}`; peers check parents, proofs, work, and every consensus commitment before accepting.

open questions: template expiry under a 1 s cadence (parano1d: 30 s; ours is template-TTL in the low hundreds of ms); orphan accounting for *proved* work (§4).

### 3b. DAA over proving latency (foculus)

the interval target must price the **whole** attempt: proving + search + propagation. parano1d's ASERT adjusts against the complete 30 s; quantus's per-block Homestead retarget reacts inside one block. decisions: ASERT-style reference window and half-life; how difficulty interacts with **two** lotteries (settlement ticket rate vs fold rate) and what happens to issuance when proving latency halves on better hardware (the DAA must absorb it, or the interval drifts); measured-proving oracle — the DAA can only see intervals, so proving latency enters indirectly; GPU provers arriving must raise difficulty, not break the clock.

### 3c. block classes as reduction budgets (nox + trident + foculus)

parano1d ships two jointly-authenticated classes (Small m23: 63 pages / 504 inputs; Large m24: 206 pages) chosen by fee pressure; every node verifies both. ours: a class is a **reduction budget** — `apply(k cyberlinks + n transfers)` counted by `trident build --costs`, classes admitted by measured proving cost on commodity hardware (§7), rejection at admission when the budget does not fit. the meter already exists; the spec must fix the budget table, the class-selection rule, and the both-classes-verify rule.

### 3d. retention (foculus + bbg)

consensus-level, not node policy: body retention window, portable receipts for payments, state-transfer path that scales with live data. parano1d: 42-block body retention, 18-block hard finality, receipts stay verifiable after pruning. soft3 must fix its windows before genesis; retrofitting is a fork. depends on the bbg sync design (state transfer = live 10-dim presence, not history replay).

### 3e. state-growth fees (tok)

parano1d: net-new live slots raise the fee, the growth component is **burned**, consolidation frees slots and avoids the fee. soft3's 10-dimensional state needs the same rule or state growth is free and the chain inherits permanent bloat. fix: which dims are occupancy-priced, the growth/burn split, and the consolidation refund.

## 4. finality — the hybrid

pow orders (settlement tickets pick blocks; folds aggregate), pos finalizes: the epoch decider proof is co-signed by a quorum of **staked cyberlinks** (the staking primitive soft3 already has — signed staked links are the validator set) at ⅔ weight, with slashing on double-finality. this is Casper-FFG-shaped but the stake is the graph's own. two consequences fall out:

- **orphans must be paid.** at 1 s with proving inside the interval, a real orphan rate is unavoidable; if orphaned *proved* work goes unpaid, only large proving nodes survive the waste and the chain re-centralizes through the back door (kaspa's inclusive reward is the precedent).
- **the epoch certificate stops carrying the payout** — already decided in [[network-planes]]; here it gains the decider + quorum signature as its content.

## 5. the two lotteries — gate, race, stitch

> shapley tickets buy **correctness of computation**, the grind buys **ordering**, and the fold stitches the first into the second. three functions, three mechanisms, each doing exactly one job. quantus has only the grind; parano1d has only the proof-gate; this split is the thing soft3 adds that neither has.

- **the ticket is a gate, the grind is a race.** every epoch, a producer must hold ≥ k_min verified settlement tickets (the useful-work floor — a ticket proves a shapley marginal was correctly computed; real computation the graph needs, correctness-attached because zheng makes it cheap to prove) to be eligible for rewards. every block carries the grind nonce under ASERT. one is quota, the other is race; conflating them is where two-lottery designs break.
- **difficulty independence.** ticket "difficulty" never adjusts — it is the batch size k_min, a parameter of [[tru]] (Hoeffding precision, per [[fold-mining]]). grind difficulty absorbs all interval pressure, including proving-latency changes (§3b). two knobs, two controllers, no coupling.
- **temporal amortization.** tickets are zheng proofs, expensive relative to a grind attempt (hemera ≈ 0.3 µs per permutation) — so tickets amortize across the epoch (self-fold batches first, leaderless tree assembly), while the grind runs fresh per block. the block interval budgets only the grind plus the fold (~30 field ops + 1 hash per fold — nothing); ticket cost lands on the epoch timescale.
- **issuance split follows measured cost, not ideology.** the grind share pays for the security minimum of ordering; the ticket share pays marginal computation at cost; recalibrated per epoch from measured numbers (audit doctrine), starting shape: grind ≈ the compute of ⅓ of the interval, tickets ≈ the rest. underpriced tickets starve the quota; overpaid grind centralizes hashpower. the DAA spec (§3b) is where this is written down.
- **useful work, honestly framed.** settlement tickets are the useful half of PoW by construction — needed work that was unverifiable until proofs made it ticket-able. the grind stays a grind: its cost is the security budget, not waste. one free byproduct: schedule the grind input from state-sync segment roots that nodes must hash anyway, so the same permutations serve double duty at zero security cost.

implementation homes: `foculus/specs/fold-mining.md` carries the ticket and fold mechanics (implemented: `src/tickets.rs`, `src/ticket_proof.rs`); §3b carries grind difficulty; tok carries the issuance split. this section is the decision the child specs inherit — it closes when the split is written into `foculus/specs/parameters.md` with numbers.

## 6. hard dependencies, stated honestly

1. the soundness repair lands: ARC (or WARP) accumulation over the step relation — [[proof-system-repair]], phases as scheduled. **without a sound accumulator there is no from-genesis story and the 1 s design is void.**
2. hemera analysis: external review of the inverse-S-box profile or a return to audited Poseidon2 parameters. the step relation hashes with hemera on every row.
3. measured `apply` costs on commodity hardware (M4-class and Ryzen-class, published to `audit/`, command and revision named per report doctrine) — the block budgets (§3c) and the interval (§7) are *derived* from these numbers, never asserted.

## 7. the interval is measured, not aspirational

parano1d moved its target 15 s → 20 s → 30 s across versions because it measured. the 1 s target stands only if the measured budget closes: proving ≤ ⅓ of the interval on commodity hardware, propagation the next fraction, selection and search the rest. today's anchors (M4 Max, 2026-10-08/09, [[proof-system-repair]] §1): hemera ≈ 0.3 µs per permutation, one hash proof 15,608 B with 30 ms verify. the step budget is denominated in nox reductions; a step that does not fit the budget is not a bigger step, it is a rejected transaction.

## 8. mining NOID — the measured network, and the trustless pool design

### the network today, from one node

readings from a live v2.0.3 node on 2026-10-12 (M4-class MacBook, NEON+PMULL backend, synced at height 215,262, 9 peers). values derived from them are marked ≈ and carry the assumption that nonce search is one Poseidon2b permutation per attempt.

| reading | value |
|---|---|
| network hashrate | ~207 GH/s |
| observed mean interval | 16 s (target 30 s; difficulty ↑ 41.4 % over 10 blocks — the network runs ahead of target) |
| difficulty / expected work per block | 20.21 M / ≈ 5.30 T hashes |
| gross reward | 16 NOID (90 % to the miner during the development allocation ≈ 14.4 NOID) |
| circulating supply / live UTXOs / state level | 10.58 M / 87.7 k / 24 |

≈ 5,400 blocks/day follow; ≈ 77.8 k NOID/day goes to miners — an order-of-$14 k/day pie at the 2026-10-10 exchange print. market numbers, not protocol numbers.

### what one MacBook does

Poseidon2b on Apple Silicon is PMULL-bound: one permutation ≈ 700–1,400 carry-less multiplies (round count × state width), and at ~2 cycles/mul effective under interleaving that is ≈ 0.25–0.5 µs per permutation per core-lane. estimates:

| machine | ≈ hashrate, all cores |
|---|---|
| M1 (8 cores) | 8–15 MH/s |
| M4 Pro (12–14 cores) | 25–40 MH/s |

network size in MacBook units: 207 GH/s ≈ **5–20 k M4-equivalents**. the band is wide because the real mix includes EPYC VPS boxes at 150–250 MH/s each — the machine count sits at the low end if servers dominate, the high end if Macs do. to be measured, not asserted: run `parano1d --mode miner --cpu-threads N` on pinned cores for 60 s and record (per `audit/` doctrine, command and revision named).

solo on one M4: share ≈ 25e6 / 207e9 ≈ 0.012 % → one block per **≈ 37 hours** expected, ≈ 9.4 NOID/day in expectation with geometric variance that can run to weeks of nothing. hourly rewards solo are impossible at this difficulty; they require aggregation.

### the pool — and what a pool node is actually for

parano1d's external worker receives a single-use nonce template (30 s TTL, no body, no witness) — the proving node owns the block. a pool is therefore **one proving node + N nonce-only workers**, and the node performs exactly the proving it would do solo. capacity:

- one M4-class proving node keeps the network cadence (Small-class proving is seconds of the 16–30 s interval) while serving workers. the template is one per attempt and shared concurrently, so the worker count is bounded by RPC overhead, not proving: **hundreds to a few thousand light miners per node** is a sane design point.
- a light miner is one binary: 1–2 cores, < 200 MB, no state, no sync, no proving — against a full node's multi-GB state, continuous validation, and near-saturating proving load. the node sells the two things light machines lack (state + proving) and buys raw hash; the worker keeps ~everything except hash.

### shares are already proofs

a valid nonce on an issued template is self-authenticating: it commits to the template hash and the payout address, and anyone checks it against the PoW target. that is a lithos NISP with no new cryptography. accounting:

1. worker submits a nonce → node verifies → records a share (template, worker, nonce, target).
2. when the pool's block lands, a payout contract splits the 14.4 NOID pro-rata over verified shares (PPLNS window) — a trident formula settled on the soft3 chain. collateral providers (staked NOID, lithos-style) take their cut by contract, not by operator promise.
3. the operator's only lever against a miner is refusing to land their blocks — visible on-chain, and covered by the collateral.

### the honeycrisp lever

the stock miner already dispatches to NEON+PMULL (the node reports `BACKEND NEON+PMULL`), so the honest gain is pipeline quality, not a new instruction: 2-lane interleaved CLMUL chains, P-core pinning with the persistent pool (no ~20 µs spawn per task), a tape-allocated proving arena instead of malloc churn (`honeycrisp`: PMULL 70×+ over scalar, Tape take ~1 ns). realistic estimates: **1.5–2.5× on the nonce stage, 1.2–1.5× on proving**. one honeycrisp Mac then counts as ≈ 2–3 default Macs — a 40-Mac pool behaves like ~100, and the same node capacity serves proportionally more light miners. the deeper lever stays the one noted earlier: the first GPU/bitsliced or dedicated-tower GF(2^128) prover re-prices the whole network, because proving, not nonce search, owns the interval.

### can an M4 be paid every hour

solo: no — ≈ 37 h expected per block at the 2026-10-12 difficulty. pooled: yes, and it is a dial, not a hope — a pool at ≈ 1 % of network (≈ 2 GH/s ≈ 50–80 Macs) lands a block every ≈ 27 min on average and pays every member on every landing. share difficulty (lithos's actual insight) sets the cadence per miner: low-diff shares → hourly micro-payouts; high-diff → rare large ones; identical expectation, chosen variance.

acceptance: a trident payout contract + a template-share path in foculus, exercised against a live parano1d template feed with at least one external worker; measured hashrate of a honeycrisp miner vs the stock miner on the same host, published to `audit/`.

## 9. order

1. §6 dependencies (repair phases, hemera, measurements) — already scheduled elsewhere.
2. the five specs §3a–§3e, in that order — each is one page, each closes by review, none needs chain code.
3. chaosnet (spacepussy-test) with frozen params; rehearsal net; genesis. activation-by-height mechanics for every later rule change, from day one.
