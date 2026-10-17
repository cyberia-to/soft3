---
tags: cyber, soft3, proposal, zheng, bbg, chain, foculus, sync, recursion, pcd, bootstrap, light client
crystal-type: process
crystal-domain: cyber
alias: proven present, the present proves the past, prove first mine second, state proof, history proof, block proof, O(1) bootstrap
status: proposed
date: 2026-10-17
revised: 2026-10-17 — every block, not every checkpoint interval; no shadow stage; no "when it fits" (owner)
---
# the proven present — the present proves the past

the property, fixed by the owner on 2026-10-17 as a base property of the system, cross-cutting and necessary:

**every block is a proven transition, and its proof verifies the proofs of its parents.** so every state, at every height, is verified recursively back to genesis by one proof of constant size — the present proves the past. a block is proved first and mined second: work orders transitions that are already proven; it never stands in for a proof. a neuron that holds the state and the proof of the tip is a full validator, in the first month and in the tenth year, by the same procedure, at the same cost.

this is how the system is designed, and the system has no meaning until it works this way. there is no cadence (every block, not every hundred), no shadow stage, no "once proving fits in the block time". the measured distance to the target is a list of work to close (§3, §5), never a reason to lower it.

the question that prompted the page: the zheng specs say O(n) and [parano1d](https://parano1d.org) says O(1) — did we lose by design? no. zheng's O(n) is the prover's time, which is optimal: no prover beats reading its own trace. parano1d's O(1) is the verification of history in the height of the chain: a terminal proof whose size and verification work do not grow with age. zheng's design has the same property ([recursion](../../zheng/specs/recursion.md) on `release/0.4`: "a proof of any number of steps is the last step's proof, the state it started from and one decider — its size does not depend on the number of steps"), with a proof about fifteen times smaller than parano1d's. what soft3 lacks is the chain as a recursive computation: the [phase-1 root chain](../specs/chain.md) validates natively, keeps every body, makes a newcomer replay from genesis, and calls zheng proofs "optional and later" (§7.4). this page makes the property ours, names what every component owes it, and orders the work.

it depends on [[proof-system-repair]] (the constant-size profile), [[privacy-frozen]] (one material; the block proof, G2), [[steps-blocks-epochs]] (the DAG and the clock) and the [[chain|root chain spec]]. it reverses one sentence of [[steps-blocks-epochs]] §1 and corrects one claim of [[goldilocks-parano1d]] §2 (§2.3, §2.4).

## 0. the property, stated once

blocks form a DAG ([[steps-blocks-epochs]] §1: a block is a bundle of steps ordered against `k` parents; the phase-1 chain is the case `k = 1`). for a block `B` of chain `c` with pinned genesis `G_c`:

```
π(B) proves:   B's parents P_1 … P_k each carry a proof π(P_j) that verifies
                 (the recursion: the verifier of π(P_j) runs inside π(B)),
               every step in B is valid — its authorization, its apply over bbg —
               B is valid under the rule set active at its height
                 (header, work, timestamps, difficulty, budgets, fees, conservation),
               applying B to its parents' state yields bbg root R(B),
               and chainwork(B) = W(B)
base:          π(G_c) is the genesis axiom: id(G_c), R(G_c), W = 0
public:        (c, height, id(B), R(B), W(B), rule set)
size:          constant in height and in the DAG's width — ≤ 64 KiB (the profile-6 bound)
verify:        constant — goal ≤ 1 ms
order:         π(B) is complete before the work on B is searched (prove first, mine second)
```

it is proof-carrying data over the DAG: every node of the DAG carries a proof that its whole past is valid, and verifying one tip verifies everything beneath it.

what it does **not** prove, said once so nobody reads it in:

| not proven | why, and who carries it |
|---|---|
| that the state's bytes are available | a proof of a root is not the data; whoever bootstraps needs someone to serve the snapshot, which the root authenticates (§4, sync) |
| that this branch is the canonical one | two valid branches carry two valid proofs; `W(B)` in the statement lets anyone compare them (§2.6) |
| liveness, censorship-resistance | no proof can (chain §8) |
| anything before genesis | genesis is the axiom: the burial snapshot, pinned by id (chain §4) |

and the rule it imposes on everyone: **no consensus state without a proof path.** a dimension, a transaction kind or a rule set that the block relation cannot express does not activate (§2.11).

## 1. why it is base

without it, five things soft3 promises are hollow:

1. **bootstrap grows with age.** a replaying chain is cheap in year one and a barrier in year ten; it ends with a few archive operators and everyone else trusting them.
2. **checkpoints are trust with an expiry date.** until `CHECKPOINT_SUNSET` (525,600 blocks, about six months) a foundation key anchors newcomers (chain §5.3). a proven tip anchors them without a key.
3. **a phone cannot be a validator.** with the tip's proof, cyb checks the state itself and trusts a server only for bytes.
4. **pruning without trust is impossible.** with every block proven, old bodies are history, not authority — state follows live use, not chain age.
5. **proven answers over an unproven state are a contradiction.** "remember" — a proven answer is a file with a hash, checked with 32 bytes — is only as good as the root it is read against; provable reads ([[inf]] over [[bbg]]) need a proven `R`.

## 2. decisions

each is final for this design; alternatives are named once.

1. **every block, from genesis.** every block of every chain has a proof that verifies its parents' proofs. no interval, no sampling, no "checkpoint proofs". blocks produced before the block proof becomes a validity condition are proven after the fact (§2.8), so the recursion still starts at genesis and no state in the chain's history is left without one.
2. **prove first, mine second.** a miner assembles the block, completes `π(B)`, freezes the template, and only then searches the work. a block whose proof does not verify is invalid however much work it carries. work chooses the order among proven transitions and nothing else.
3. **recursion over the DAG, not accumulation alone.** each block proof runs the verifiers of its parents' proofs inside its own circuit; only the last proofs travel (zheng `specs/recursion.md`; `audit/accumulation-2026-10.md` §6: without a verifier inside the next step the proof grows with the run). this corrects [[goldilocks-parano1d]] §2, which assumed accumulation alone gives constant size. soft3 pays the recursion, but in a field-native circuit, and its proof is 64 KB, not parano1d's 1 MB.
4. **this reverses [[steps-blocks-epochs]] §1** ("from-genesis validity lives in the accumulator, not in each block"): from-genesis validity lives in each block. it costs one constant-size merge per block, not a re-proof of history — the parents' proofs are verified, their pasts are never recomputed. the epoch remains the tier of finality and of ranking, not of validity.
5. **the work is spread over the DAG.** the proving of a block is never one machine's sequential job. each step is proved where its witness lives: the neuron that authored it proves its authorization and its apply (as parano1d's wallet proves authorization and its miner the transition); aggregators fold step proofs in a tree (the leaf/node/top programs of zheng's block aggregation, G2); the block producer runs one merge: its parents' proofs, the folded steps, the block rules. the block's latency is the depth of that tree times one merge, not the number of its transactions times one proof. parallel branches of the DAG are proven concurrently; joining them is a merge.
6. **the proof carries the work.** `W(B)` is public. a newcomer picks, among valid tip proofs for its chain, the greatest `W`; the reorg limit (chain §5.2) binds nodes that already follow a tip. how a newcomer chooses across a fork deeper than `FINALITY_DEPTH` is written into chain §5 with the activation (§6.5).
7. **one material.** the proof is hash-only over [[hemera]] and Goldilocks, the profile-6 family of [[zheng]]. nothing in it may introduce a second assumption ([[privacy-frozen]] §1.1).
8. **the canary blocks are proven after the fact.** the blocks produced before the block proof becomes a validity condition (rule set 1 from 2026-11-05) are proven by catch-up provers in order, from genesis to the tip, until the proving keeps pace with the chain; from the activation height every block carries its own. a node may verify the catch-up proofs before the activation; it must verify the block proof from the activation.
9. **authorization is hash-only.** a neuron authorizes a step by proving, inside the step's proof, knowledge of the secret its neuron id commits to — no public key and no signature on the wire, as in the pool ([[privacy-frozen]] §1.4) and in parano1d. this collides with the root chain spec as it stands: chain §2.7 authorizes every transfer, cyberlink and claim with mudra NSIG1, a secp256k1 ECDSA signature (`mudra/specs/neuron-auth.md`, "native secp256k1 statement authentication"), which is non-native arithmetic in a Goldilocks circuit and the most expensive thing a block relation could contain. the resolution, for the owner before genesis (§6.1): authorization in the chain becomes hash-only; secp256k1 remains only where a legacy key must be proven once — the claim of a burial balance (chain §2.8, mudra legacy bridge) — and is proven at its price, once per account, never per transaction.
10. **the bodies' retention follows the proofs.** while blocks without proofs are final (before the activation), nodes stay archival (chain §3.6). from the activation, a node that keeps the state and the tip's proof is a full validator, and keeping bodies is a service, not a duty.
11. **every rule set extends the relation before it activates.** rule set 2 (the pool, settlement, fold, foculus) does not activate until the block relation covers its kinds, dimensions and block-proof statement. this is the cross-cutting rule, and its owner is whoever writes the rule set.

## 3. where we stand

measured on Apple M4 Max with hemera 0.3; every figure is re-measured after the hemera 0.4 sweep (zheng `feat/hemera-0.4`).

| piece | design | today | the distance |
|---|---|---|---|
| constant-size recursive proof (zheng profile 6, `release/0.4`) | IVC with the verifier in the step + wrap chain, ≤ 64 KiB, verify ≤ 1 ms | final proof 63,865 B; verify 5.3–5.7 ms on one thread, min 2.8 ms on 16; verifying key 8,978,744 B; IVC step 1.12 s, wrap chain 18.9 s on Metal; prover peak 19.5 GB (`audit/wrap-fastverify-2026-10.md`, `audit/ivc-speed-2-2026-10.md`) | verify ×5; the key's size for phones; a merge of `k` parents, not one |
| one proof per block (zheng PR #70, G2) | the block's step proofs folded into one | draft: 100 transactions → 61,708 B, verify 3.04 ms, prove 1,752.7 s on one Metal machine | ≈ 58× from 100 tx per 30 s on one machine; the structural lever named in its receipt is aggregation by accumulation; the distributed tree of §2.5 is not built |
| the block relation | header, work, ASERT, coinbase, steps, nonces, fees, conservation over bbg | none: validation is native Rust (chain §0, "nox programs are not required") | all of it (S1) |
| authorization | hash-only, inside the step proof | NSIG1, secp256k1 ECDSA on every transaction (chain §2.7) | a pre-genesis decision (§6.1) |
| the state root in a circuit | bbg root = hemera fold over 14 leaves, dimensions as keyed hemera tries (bbg `rs/src/root.rs`, dimension version 3) | native only, but the hash is already the circuit's own | trie updates as a relation (S1) |
| a place for the proof | the block carries `π(B)`; bbg `Checkpoint { root, acc: Option<CheckpointProof>, height }` | the header has no proof field; the slot is always `None` | the rule set that adds it (S4) |
| bootstrap from the tip | state snapshot + tip proof | replay from genesis, or a foundation checkpoint | sync messages and the node mode (S5) |
| parano1d, for scale | a recursive proof in every block, fixed size | terminal 913 KB – 1.08 MB, prove 17.6 s, verify 0.83 s, 30 s blocks ([[goldilocks-parano1d]] §1) | it ships; we close the distance |

the arithmetic of the target, not a measurement: with the step proofs made by their authors and folded in a 4-ary tree, a block of 1,024 steps is five merge levels; at one merge of the order of the IVC step (1.12 s today on Metal) plus the block merge with its parents, a block proof completes in seconds of wall-clock across machines, inside 30 s. S3 measures it.

a note on sources: the zheng specs on `master` describe the retired Brakedown line — openings that grow with the trace, and a constant-size opening that stayed blocked on `lens` issue #6. the live line is `release/0.4` (WHIR, profile 6); read it when comparing zheng with anything. bringing `master` level is part of the coordinated release of [[proof-system-repair]] §9.

## 4. what each component owes

| component | owes the proven present | phase |
|---|---|---|
| [[soft3]] chain (`crate/src/chain`) | the reference validator and the conformance corpus; the template lifecycle "prove, freeze, mine"; the proof field in the block; the activation rule set; the newcomer mode | S0, S2, S4, S5 |
| [[foculus]] | the DAG with `k` parents, the block-attempt state machine around the proof (never re-parent a proof, [[steps-blocks-epochs]] §3a), the DAA pricing the whole attempt including proving (§3b) | S2–S4 |
| [[bbg]] | trie insert/update/delete and the 14-leaf fold as relations; snapshots of every dimension authenticated by `R`; the filled `Checkpoint` slot | S1, S5 |
| [[zheng]] | the step relation, the merge of `k` parent proofs with folded steps, the distributed aggregation tree; verify ≤ 1 ms; a verifying key a phone can hold | S1–S3, S6 |
| [[nox]] / [[trident]] | the block rules and every step kind as bounded programs with a cost known before they run; ASERT and the subsidy are integer-only already (chain §1.5) | S1 |
| [[mudra]] | hash-only neuron authorization proved inside a step; secp256k1 confined to the one-time legacy claim, with its proof | S0–S1 |
| [[joy]] | proving steps on the author's device (CPU, Metal, WebGPU, same bytes, ≤ 1 s, [[proof-system-repair]] §0) and merges on aggregators | S1–S3 |
| [[radio]] / sync | carrying step proofs to aggregators and block proofs with blocks; serving `(snapshot, tip proof)` to newcomers | S3, S5 |
| [[cyb]] | proving its own steps; a light client that checks the tip's proof and reads against `R` | S3, S6 |
| [[tru]] | φ* at an epoch with its proof (the fold of settlement, rewards §7), so the ranking a neuron sees is checkable like its balance | S6 |
| [[inf]] / [[cybergraph]] | the step relation of `apply`; provable reads against a proven root | S1, S6 |
| whoever writes a rule set | the relation for every new kind and dimension before the activation height (§2.11) | always |

## 5. the plan

each phase closes on its last merge, with a receipt in `audit/`. the order of S1 → S2 → S4 is fixed; S3 runs beside S2 and must close before S4; S5 and S6 follow.

| phase | what | acceptance |
|---|---|---|
| **S0 — before genesis (now → 2026-11-05)** | the owner's decision on authorization (§6.1) and its landing in chain §2.7 and mudra; the node writes the conformance corpus from height 0 — every block with its native verdict and `R`, plus a generator of invalid blocks; chain §7.4 points here | decision recorded in chain §2.7; corpus job running on pussy from genesis |
| **S1 — the block relation** | the steps of rule set 1 with hash-only authorization, header (hemera work under the compact target), timestamps (MTP, drift), ASERT, coinbase and subsidy, nonces, fees, conservation, bbg trie updates and the root fold — one relation; the legacy claim's secp256k1 proof as its own sub-relation | relation verdict = native verdict on the whole corpus, including every generated invalid block and a bit-flip scan; rows and prove time per step kind in the receipt |
| **S2 — the recursive block proof** | `π(B)` verifying its parents' proofs and its folded steps; catch-up proving from genesis to the tip (§2.8); every new block published with its proof as soon as the catch-up reaches the tip | a proof for every block of pussy from genesis to the current tip, each ≤ 64 KiB, each verifying its parent; two independent provers give byte-identical public inputs |
| **S3 — the proof inside the block time** | the distributed tree of §2.5: authors prove steps, aggregators fold, the producer merges; GPU paths; the levers of zheng PR #70 §4 (aggregation by accumulation) | over a day of real load on pussy, every block's proof is complete before its work is searched, within `BLOCK_TIME`, on hardware a miner owns; the latency distribution in the receipt |
| **S4 — activation: a block without its proof is invalid** | the rule set that adds the proof field, makes `π(B)` a validity condition and "prove first, mine second" the template rule; checkpoints by key end | activated before `CHECKPOINT_SUNSET` (height 525,600 — about 2027-05-06 for pussy); from it, no state of the chain depends on a key or on a replay |
| **S5 — the newcomer from the tip** | sync messages for `(snapshot, tip proof)`; `soft3 node --from-proof`; pruning as an option | a fresh node joins from the tip's proof, reaches the same tip and the same `R` as an archival node, in the same time at two heights a month apart; fleet green |
| **S6 — beyond the chain** | the light client in cyb; φ* with its proof per epoch; provable reads against `R`; verify ≤ 1 ms and a phone-sized key | cyb on Android checks the tip's proof and a read against it, end to end, timed on the receipt |

## 6. open, and who closes it

1. **authorization before genesis.** chain §2.7 and the chain kernel authorize with secp256k1 ECDSA today. the formats freeze at genesis; changing them afterwards is a rule set. the owner decides before 2026-11-05 whether rule set 1 already authorizes hash-only, or whether the canary carries ECDSA until the activation rule set replaces it (and the catch-up proves those signatures once, at their price). soft3 chain, mudra.
2. **who pays the provers.** the step author proves its own step; aggregators and the producer's merge need a reward — the tier-2 fold subsidy of settlement mining (tru `specs/rewards.md` §7–8), fees, or the block subsidy. owner decision before S4.
3. **verify ≤ 1 ms.** today ×5 short on one thread. zheng.
4. **the verifying key on a phone.** 8.98 MB today. zheng with cyb, before S6.
5. **newcomer fork choice across a deep fork.** heaviest proven work against the reorg limit; written into chain §5 with S4. soft3 chain.
6. **snapshot availability.** a proof makes the data checkable, not present; who serves snapshots and what they earn is a sync question, before S5. radio/sync.
7. **bostrom.** the same property for the second chain from its own genesis (launch tracker); nothing here is pussy-specific.

## 7. sources

- zheng `release/0.4`: `specs/recursion.md`, `audit/wrap-fastverify-2026-10.md`, `audit/ivc-speed-2-2026-10.md`, `audit/accumulation-2026-10.md`; PR #70 (block aggregation, draft).
- bbg `master`: `rs/src/root.rs`, `rs/src/checkpoint.rs`, dimension version 3 (PR #43).
- mudra: `specs/neuron-auth.md` (NSIG1).
- soft3: [[chain]] §0, §2.7, §2.8, §3.3, §3.6, §5, §7; [[proof-system-repair]]; [[privacy-frozen]]; [[steps-blocks-epochs]]; [[goldilocks-parano1d]].
- tru `specs/rewards.md` §7–8 (the fold and its subsidy).
- parano1d: [parano1d.org](https://parano1d.org) sections 02, 03, 04, 09, 14 ("the present proves the past", bootstrap from proof, proofs built where the information lives, recursive history, prove first, mine second), read 2026-10-17; workspace analysis in [[goldilocks-parano1d]].
