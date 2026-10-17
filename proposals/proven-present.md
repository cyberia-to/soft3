---
tags: cyber, soft3, proposal, zheng, bbg, chain, sync, recursion, bootstrap, light client
crystal-type: process
crystal-domain: cyber
alias: proven present, the present proves the past, state proof, history proof, O(1) bootstrap, proof of the present
status: proposed
date: 2026-10-17
---
# the proven present — every state carries the proof of its past

the property, fixed by the owner on 2026-10-17 as a base property of the system, cross-cutting and necessary: **a soft3 state is checked by one proof of constant size, not by replaying the history that produced it.** a neuron that holds the current state, its proof and a short suffix of recent blocks is a full validator — in the first month and in the tenth year, by the same procedure, at the same cost.

the question that prompted it: the zheng specs say O(n) and [parano1d](https://parano1d.org) says O(1) — did we lose by design? no. the two numbers measure different things. zheng's O(n) is the prover's time, which is optimal: no prover beats reading its own trace. parano1d's O(1) is the verification of history in the height of the chain: "the present proves the past", a terminal proof whose size and verification work do not grow with age. zheng's design has that property too ([recursion](../../zheng/specs/recursion.md) on `release/0.4`: "a proof of any number of steps is the last step's proof, the state it started from and one decider — its size does not depend on the number of steps"), and its proof is fifteen times smaller than parano1d's. what soft3 does not have is the relation that makes the chain itself a recursive computation. the [phase-1 root chain](../specs/chain.md) validates natively, keeps every body, and makes a new node replay from genesis; zheng proofs are "optional and later" (§7.4). parano1d ships the property on a live mainnet since 2026-08-21. this page makes it ours, names what every component owes it, and orders the work.

it depends on [[proof-system-repair]] (the constant-size profile), [[privacy-frozen]] (one material; the block proof, G2), [[steps-blocks-epochs]] (the clock) and the [[chain|root chain spec]]; it corrects one claim of [[goldilocks-parano1d]] (§3).

## 0. the property, stated once

for a chain `c` with pinned genesis `G_c`, at height `h`:

```
Π_c(h) proves:  there is a sequence of blocks B_1 … B_h such that
                  B_1.prev = id(G_c), B_i.prev = id(B_{i−1}),
                  every B_i is valid under the rule set active at its height
                  (header, work, timestamps, difficulty, body, every transaction),
                  applying B_1 … B_h to the genesis state yields bbg root R_h,
                  and chainwork(B_h) = W_h
public:         (c, h, id(B_h), R_h, W_h, rule set at h)
size:           constant in h — ≤ 64 KiB (the profile-6 bound)
verify:         constant in h — goal ≤ 1 ms
```

what it does **not** prove, said once so nobody reads it in:

| not proven | why, and who carries it |
|---|---|
| that the state's bytes are available | a proof of a root is not the data. whoever bootstraps needs someone to serve the snapshot; the root authenticates it (§4, sync) |
| that this chain is the canonical one | two valid forks have two valid proofs; the work `W_h` in the statement lets a newcomer choose (§2.5) |
| liveness, censorship-resistance | no proof can (chain §8) |
| anything before genesis | the genesis is the axiom: the burial snapshot, pinned by id (chain §4) |

and the rule it imposes on everyone: **no consensus state without a proof path.** a dimension, a transaction kind or a rule set that the history relation cannot express does not activate (§2.10).

## 1. why it is base

without it, five things that soft3 promises are hollow:

1. **bootstrap grows with age.** a phase-1 node replays every block from genesis (chain §3.6). at 2,880 blocks a day the replay is cheap in year one and a barrier in year ten; every replaying chain ends with a few archive operators and everyone else trusting them.
2. **checkpoints are trust with an expiry date.** until `CHECKPOINT_SUNSET` (525,600 blocks, about six months) a foundation key anchors newcomers (chain §5.3). after the sunset there is nothing but the reorg limit. the proven present is what replaces the key — the sunset is its deadline.
3. **a phone cannot be a validator.** cyb on a phone today trusts the server it syncs from. with `Π` it checks the state itself and trusts the server only for bytes.
4. **pruning without trust is impossible.** a node may drop old bodies (chain §3.6, `--prune`) only by trusting whoever kept them. with `Π` the bodies become history, not authority — state follows live use, not chain age.
5. **proven answers over an unproven state are a contradiction.** "remember" — a proven answer is a file with a hash, checked with 32 bytes — is only as good as the root it is read against. provable reads ([[inf]] over [[bbg]]) need a proven `R_h`.

## 2. decisions

each is final for this design; alternatives are named once.

1. **the native validator stays the reference.** the history relation is defined as the conjunction of the native rules of the active rule set, nothing more and nothing less. a conformance corpus — every block of pussy and bostrom from genesis, plus generated invalid blocks — must get the same verdict from the native validator and from the relation. a disagreement is a bug in the relation, never a new rule.
2. **one material.** the proof is hash-only over [[hemera]] and Goldilocks, the profile-6 family of [[zheng]] (IVC with the verifier inside the step, WHIR openings, wrap chain). nothing in it may introduce a second assumption ([[privacy-frozen]] §1.1).
3. **the shape is recursion, not accumulation alone.** each step proves its own segment and that the previous step's verifier accepted; only the last step's openings travel (zheng `specs/recursion.md`; `audit/accumulation-2026-10.md` §6). this corrects [[goldilocks-parano1d]] §2, which assumed accumulation alone would give constant size without a verifier in the step: it does not. soft3 pays the recursion tax too, but in a field-native circuit, and the result is 64 KB, not 1 MB.
4. **cadence: per checkpoint interval first.** `Π` is produced every `CHECKPOINT_INTERVAL` (120 blocks, one hour). the blocks after the proven height — at most `FINALITY_DEPTH + CHECKPOINT_INTERVAL` — are validated natively by the joining node. per-block proofs ("prove first, mine second") come only when proving a block fits in `BLOCK_TIME` on a miner's hardware (§5, S5).
5. **the proof carries the work.** `W_h` is public. a node with no history picks, among valid proofs for its chain id, the one with the greatest `W_h`; the reorg limit (chain §5.2) binds nodes that already follow a tip, not newcomers. the interplay — a newcomer choosing across a fork deeper than `FINALITY_DEPTH` — is written into chain §5 before S4 (§6.6).
6. **not consensus until it is ready.** in phase 1 the proof is a shadow product, published on `shadow/history` like the cores' outputs (chain §7.3); a node without the verifier runs identically. it becomes consensus in two steps: as the checkpoint proof that replaces the foundation signature (S4), then, optionally, as a per-block requirement (S5).
7. **rule set 1 is proven once, afterwards.** the transparent canary period (rule set 1, secp256k1 signatures) is proven retroactively by one long batch: the proof stays constant whatever it cost to produce, and that cost is paid once.
8. **authorization moves to hash-only.** secp256k1 ECDSA (mudra NSIG1, chain §2.7) is non-native arithmetic in a Goldilocks circuit and is expected to dominate the relation's cost; its cost is measured first (S1). going forward, authorization inside the pool is knowledge of a secret proved in the proof ([[privacy-frozen]] §1.4), which is cheap; ECDSA remains at the transparent edge, which is proven at its price.
9. **the bodies' retention does not change in phase 1.** phase-1 nodes stay archival (chain §3.6) because the replay and the shadow indexers read the whole stream. pruning becomes a validator's option, not a trust decision, from S3.
10. **every rule set extends the relation before it activates.** rule set 2 (the pool, settlement, fold, foculus) does not activate until the history relation covers its kinds, dimensions and block-proof statement. this is the cross-cutting rule, and its owner is whoever writes the rule set.

## 3. where we stand

measured on Apple M4 Max with hemera 0.3; every figure is re-measured after the hemera 0.4 sweep (zheng `feat/hemera-0.4`).

| piece | design | today | missing |
|---|---|---|---|
| constant-size proof of a long computation (zheng profile 6, `release/0.4`) | IVC + wrap chain, ≤ 64 KiB, verify ≤ 1 ms | final proof 63,865 B; verify 5.3–5.7 ms on one thread, min 2.8 ms on 16; verifying key 8,978,744 B; IVC step 1.12 s, wrap chain 18.9 s on Metal; prover peak 19.5 GB (`audit/wrap-fastverify-2026-10.md`, `audit/ivc-speed-2-2026-10.md`) | verify ≤ 1 ms (×5 short); the key's size for phones |
| one proof per block (zheng PR #70, G2) | the block's transactions folded into one proof | draft: N = 100 transactions, 61,708 B, verify 3.04 ms, prove 1,752.7 s on Metal | ≈ 58× from 100 tx / 30 s; receipt unfinished |
| the block-validity relation | header, work, ASERT, coinbase, transfer, cyberlink, claim, nonces, fees, conservation over bbg | none: validation is native Rust (chain §0 "nox programs are not required") | all of it (S1) |
| the state root in a circuit | bbg root = hemera fold over 14 leaves; dimensions are keyed hemera tries (bbg `rs/src/root.rs`, dimension version 3) | native only; the hash is already the circuit's own | trie updates as a relation (S1) |
| a slot for the proof | bbg `Checkpoint { root, acc: Option<CheckpointProof>, height }`; chain §7.4 | slot exists, always `None` | filling it (S2, S4) |
| bootstrap from a proof | state snapshot + proof + suffix | replay from genesis, or a foundation checkpoint | the sync messages and the node mode (S3) |
| parano1d, for scale | HistoryStep per block, fixed size | terminal 913 KB – 1.08 MB, prove 17.6 s, verify 0.83 s, suffix 18 blocks ([[goldilocks-parano1d]] §1) | — |

a note on sources: the zheng specs on `master` describe the retired Brakedown line — openings that grow with the trace, and a constant-size opening that stayed blocked on `lens` issue #6. the live line is `release/0.4` (WHIR, profile 6); a reader comparing zheng with anything should read that branch. bringing `master` level is part of the coordinated release of [[proof-system-repair]] §9.

## 4. what each component owes

| component | owes the proven present | phase |
|---|---|---|
| [[soft3]] chain (`crate/src/chain`) | the reference validator and the conformance corpus; `Π` production as a shadow service; `--from-proof` node mode; checkpoint proofs in the rule set of S4 | S0, S2–S4 |
| [[bbg]] | trie insert/update/delete and the 14-leaf fold as relations; snapshots of every dimension at a height, authenticated by `R_h`; the filled `Checkpoint` slot | S1, S3 |
| [[zheng]] | the chain step relation and its IVC over blocks (or segments); verify ≤ 1 ms; a verifying key a phone can hold | S1–S2, S6 |
| [[nox]] / [[trident]] | the block rules as bounded programs, so the step relation is a nox trace with a cost known before it runs; ASERT and the subsidy schedule are integer-only already (chain §1.5) | S1 |
| [[mudra]] | the measured cost of NSIG1 verification in the relation; the hash-only authorization of the pool | S1 |
| [[joy]] | proving `Π` on CPU, Metal and WebGPU, same bytes; off a neuron's device path ([[proof-system-repair]] §0 gate) | S2 |
| [[radio]] / sync | serving `(snapshot at h, Π_c(h), suffix)` and the `shadow/history` topic; refusing nothing a root authenticates | S3 |
| [[cyb]] | a light client that checks `Π` and reads through provable reads against `R_h` | S6 |
| [[foculus]] / [[tru]] | the same property for focus: φ* at an epoch with its proof (the fold of settlement, rewards §7), so the ranking a neuron sees is checkable like its balance | S6 |
| [[inf]] / [[cybergraph]] | provable reads against a proven root, the "remember" verb made whole | S6 |
| whoever writes a rule set | the relation for every new kind and dimension before the activation height (§2.10) | always |

## 5. the plan

each phase closes on its last merge, with a receipt in `audit/`.

| phase | what | acceptance |
|---|---|---|
| **S0 — genesis (now → 2026-11-05)** | nothing in consensus changes. chain §7.4 links this page. the node starts writing the conformance corpus from height 0: every block with its native verdict and its `R_h`, plus the generator of invalid blocks | corpus job running on pussy from genesis; chain spec links here |
| **S1 — the block relation** | header (hemera work under the compact target), timestamps (MTP, drift), ASERT, coinbase and subsidy, the three transaction kinds of rule set 1, nonces, fees, conservation, bbg trie updates and the root fold — as one step relation; NSIG1 cost measured and, if it dominates, isolated as its own sub-relation | relation verdict = native verdict on the whole corpus, including every generated invalid block and a bit-flip scan; rows and prove time per block and per signature in the receipt |
| **S2 — the chain proof** | IVC over blocks with state `(c, h, id(B_h), R_h, W_h, rule set)`, profile-6 wrap; `Π` published every 120 blocks on `shadow/history` and at `/proof/history` | a proof of pussy from genesis to a height ≥ 10,000, ≤ 64 KiB, its verify time, and the prove time per 120 blocks on Metal; two independent nodes produce byte-identical public inputs |
| **S3 — bootstrap from the proof** | sync messages for `(snapshot, Π, suffix)`; `soft3 node --from-proof`; pruning becomes a validator's option | a fresh node joins pussy from a proof and a suffix, reaches the same tip and the same `R` as an archival node, and does it in the same time at two heights a month apart; fleet test green |
| **S4 — proofs replace the foundation key** | a rule set in which a checkpoint carrying a valid `Π` is accepted without `checkpoint_key`; the newcomer's fork choice by `W_h` written into chain §5 | activated before `CHECKPOINT_SUNSET` (height 525,600 of each chain — about 2027-05-06 for pussy); after it, no chain state depends on a key |
| **S5 — prove first, mine second (optional)** | per-block `Π`, the block proof of G2 folded into the history step | only when proving one block fits in `BLOCK_TIME` on commodity miner hardware, measured; until then S4 stands |
| **S6 — beyond the chain** | the light client in cyb; φ* with its proof per epoch; provable reads against `R_h`; verify ≤ 1 ms and a phone-sized verifying key | cyb on Android checks `Π` and a read against it, end to end, with the time on the receipt |

the order is not negotiable in one place: S1 before anything that claims the property. S2–S6 may overlap.

## 6. open, and who closes it

1. **who pays the prover.** candidates: the tier-2 fold subsidy of settlement mining (tru `specs/rewards.md` §7–8), fees, or the miner in S5. until decided, the foundation runs the prover in shadow mode. owner decision before S4.
2. **verify ≤ 1 ms.** today ×5 short on one thread (zheng `audit/wrap-fastverify-2026-10.md`). zheng.
3. **the verifying key on a phone.** 8.98 MB today. zheng, with cyb, before S6.
4. **NSIG1 in the relation.** cost unknown until S1 measures it; the fallback is decision 7 (prove the transparent period once, offline). mudra, zheng.
5. **snapshot availability.** a proof makes the data checkable, not present. how many neurons must serve a snapshot, and what they earn, is a sync question. radio/sync, before S3.
6. **newcomer fork choice across a deep fork.** heaviest proven work against the reorg limit; to be written into chain §5 before S4. soft3 chain.
7. **bostrom.** the same property for the second chain, after its own genesis (launch tracker); nothing here is pussy-specific.

## 7. sources

- zheng `release/0.4`: `specs/recursion.md`, `audit/wrap-fastverify-2026-10.md`, `audit/ivc-speed-2-2026-10.md`, `audit/accumulation-2026-10.md`; PR #70 (block aggregation, draft).
- bbg `master`: `rs/src/root.rs`, `rs/src/checkpoint.rs`, dimension version 3 (PR #43).
- soft3: [[chain]] §0, §2.7, §3.3, §3.6, §5, §7; [[proof-system-repair]]; [[privacy-frozen]]; [[goldilocks-parano1d]]; [[steps-blocks-epochs]].
- tru `specs/rewards.md` §7–8 (the fold and its subsidy).
- parano1d: [parano1d.org](https://parano1d.org) sections 02, 03, 09 ("the present proves the past", bootstrap from proof, recursive history), read 2026-10-17; workspace analysis in [[goldilocks-parano1d]].
