---
tags: cyber, soft3, proposal, privacy, zheng, mudra, bbg, post-quantum, launch
crystal-type: process
crystal-domain: cyber
alias: frozen privacy, the shielded pool, privacy proposal, P1 P2 P3 frozen
status: proposed
date: 2026-10-16
---
# privacy, frozen — one pool, one proof, one material

the question (owner, 2026-10-16): a privacy scheme that is as efficient as the harness allows today, quantum-safe, lets anyone compute custom things over its outputs, serves consensus, and above all is the machine everyone can speak into where nobody learns who said what and everyone learns what was said. and it must be frozen at launch like the hardware of a satellite: the currency becomes a material, like gold, that nobody can change afterwards.

this page gives the decisions, the relation, what freezes at genesis, what has to be researched before the pool can be switched on, and the acceptance gates. it is written against the harness as it stands on 2026-10-16: [[hemera]] over Goldilocks, [[nox]], [[zheng]] profiles 1, 2 (veil) and 6, [[lens]] WHIR, [[bbg]], [[mudra]], the [[chain|phase-1 root chain]] spec, and the measured figures of `audit/proof-latency-2026-10.md` and `zheng/audit/*-2026-10.md`.

## 0. the answer in one paragraph

the money is a single shielded pool of records committed under hemera in one Merkle tree; spending a record is proving, in zero knowledge with a hash-only zheng proof, that you know its secret and that the values balance; speaking is spending: a cyberlink is a public output of a spend, so the edge and its weight are public and the author is the spent record's key, which nobody sees; programs are data carried as predicate particles on records, and consensus verifies one fixed relation forever; blocks carry one folded proof, so a node checks a block in milliseconds whatever the transactions cost to prove; the transparent ledger keeps a counter of what entered and left the pool, so even a broken proof system cannot inflate the supply. consensus rests on one material, hemera, and on nothing else. note delivery may use a lattice KEM, but consensus never reads it, and its failure is disclosure, never forgery.

fundamental research is needed, and it is two items, not ten: hash-only zero-knowledge proofs of ≤ 64 KB (today 165–419 KB), and verifying those proofs inside the block proof so that a block stays one proof. the third gate is not research but time: cryptanalysis of hemera, the material itself.

## 1. the twelve decisions

each is final for this design; the alternatives are named once and not reopened.

1. one material. consensus soundness — no counterfeit, no double spend, no forged authorship — rests on hemera over Goldilocks and on the hash-only zheng proofs built from it (Spartan over CCS, Reed–Solomon + WHIR, Fiat–Shamir). no lattice, isogeny, pairing or homomorphic scheme is ever a consensus assumption. the only conjectures are the ones every hash-based proof system shares: the random-oracle model for Fiat–Shamir and mutual correlated agreement for Reed–Solomon codes at the shipped rates (`zheng/specs/soundness.md`).
2. confidentiality may add a key encapsulation. a record's secret note (value, randomness, data) is delivered to its recipient encrypted under ML-KEM-768 (FIPS 203, mudra `seal`). consensus never parses a ciphertext. if the lattice falls, past notes are disclosed and nothing is forged; the KEM is versioned by wallets and changes without a fork. the isogeny scheme (mudra `stealth`) is out: slower, younger, and it buys nothing a KEM does not.
3. one pool, one record. a record is `(token, value, owner, rho, r, data, predicate)`: token id, a u64 value, `owner = hemera(D_OWNER ‖ sk)`, a 32-byte uniqueness nonce `rho`, 32 bytes of commitment randomness `r`, a data particle, and a predicate particle (the nox program that must accept any spend; zero means none). the commitment is `hemera(D_CM ‖ record)`. commitments live in one hemera Merkle tree of depth 40 (2^40 records; the node jet costs one permutation per level, `zheng/audit/fast-path-2-2026-10.md`). the nullifier is `hemera(D_NULL ‖ sk ‖ rho)`. a spend names an anchor: any tree root among the last 1,000 blocks.
4. no signatures in the pool. spend authorization is knowledge of `sk` inside the proof. hash-only, quantum-safe, and nothing to rotate.
5. the turnstile is the gold. per token, the chain keeps a transparent counter `pool = Σ deposits − Σ withdrawals`. a withdrawal that would make it negative is invalid regardless of its proof. the pool's true content can never exceed what entered it, so a break of the proof system or of the hash caps counterfeit at the pool's balance and can never raise the supply. the supply schedule of the root chain (1 %/yr, `specs/chain.md` §3) is frozen with it.
6. speaking is spending. a cyberlink is a public output of a spend: `(p, q, weight, payout)` where `p`, `q` are particles, `weight` is a u64, and `payout` is an owner key for whatever the link earns. the fee comes from the spent record. the chain learns the edge and its weight; it never learns which record spoke. one record, one voice per nullifier: reuse is a double spend. this folds P1 into P2 and makes P3 empty: there is no attribution to hide, φ* and every aggregate are computed over anonymous edges. a neuron who wants a reputation proves later, selectively, that it owns a set of links (a zheng proof over its own records); nothing forces it.
7. programs are data. a record's predicate particle names a nox program; a spend must present, inside the same proof, that every input's predicate and every output's predicate accepted the transaction (the birth and death predicates of ZEXE, on nox). consensus verifies one relation forever; what people compute changes without consensus. public computation over public outputs (φ*, markets, indexes) is indexing and needs no protocol. homomorphic encryption (mudra `veil`, TFHE) and threshold computation (mudra `quorum`) stay off-chain services; neither enters consensus.
8. one proof per block. the miner folds every transaction's proof into the block proof (the constant-size profile of `zheng/specs/recursion.md`: 64,213 B worst case, verify 2.5 ms on 16 threads). nodes verify the block proof and never the transactions' proofs. raw transaction proofs travel only in the mempool, where size costs bandwidth, not consensus. the block budget of 256 KiB therefore holds whatever a transaction proof weighs.
9. transparent only at the edges. genesis balances, coinbase and legacy-key claims (secp256k1 through the mudra bridge, not quantum-safe) exist on the transparent side to enter the pool. a transparent balance that never enters is not quantum-safe, and the wallets say so. the cores' shadow indexers read the public outputs; nothing of the cores needs the transparent balances.
10. what freezes at genesis. the material and the formats: hemera, Goldilocks, nox, the record, commitment and nullifier encodings, the domain tags, tree depth, anchor window, the turnstile accounting, the block-proof statement shape, the supply schedule. these are in the genesis rule set. the pool itself, with its verifying keys, is rule set 2 and switches on when the three gates of §5 pass. that is not changing the satellite; it is powering on an instrument that was on board at launch. a verifying key set, once in a rule set, is never changed.
11. the network layer is not frozen. who broadcasts a transaction is hidden at the relay layer (a Dandelion-style stem over [[radio]], wallets choosing their first hop); it is a wallet and node matter, versioned freely.
12. consensus weight is frozen too. anything that touches consensus — a stake amplifier, private staking — is a rule set and needs a fork; so stake privacy is designed before any proof-of-stake activation, as records with a stake predicate, and never bolted on.

## 2. the relation, once

one relation `SPEND`, verified by one verifying key per rule set.

public inputs:

| field | meaning |
|---|---|
| chain | the chain id (genesis particle) |
| anchor | a commitment-tree root among the last 1,000 |
| nullifiers | `n` of them, `n ≤ 4` |
| commitments | `m` new commitments, `m ≤ 4` |
| outputs | public outputs: cyberlinks `(p, q, weight, payout)`, a transparent withdrawal `(to, token, amount)`, both or neither |
| fee | token and amount, paid to the miner |
| predicates | the predicate particles of all inputs and outputs, in order |
| notes | ciphertexts, opaque to consensus |

witness: the input records, their Merkle paths and secrets `sk`, the output records' openings, the predicate programs' witnesses.

checks inside the proof:

1. every input commitment opens to a record whose owner is `hemera(D_OWNER ‖ sk)`, and sits in the tree at the anchor;
2. every nullifier is `hemera(D_NULL ‖ sk ‖ rho)` of its input;
3. per token, Σ input values = Σ output values + fee + withdrawal, every value a u64 (two 32-bit limbs under nox's word ops);
4. every output commitment opens to its record with the committed randomness;
5. every input's and output's predicate program, run on the transaction's public inputs and the predicate's own witness, returns zero (the nox machine relation of `zheng/specs/machine.md` covers every opcode);
6. the public outputs equal what the predicates and records declare.

checks outside the proof, by every node: the anchor is recent, no nullifier is spent, the turnstile stays non-negative, the fee is at least the floor, the block proof verifies.

deposits are transparent-side transactions that add commitments without nullifiers; the turnstile counts them.

## 3. what it costs today

measured on this harness (Apple M4 Max, Metal; `audit/proof-latency-2026-10.md` §5 and `zheng/audit/fast-path-2-2026-10.md`), for the closest shapes that exist:

| statement | prove | proof |
|---|---|---|
| veil, secret leaf, depth 32, node tree | 96 ms warm, 0.18 s cold | 165–181 KB |
| P1 authorship (mudra#22), depth 20 / 32, held key | 255 / 396 ms | 292 / 419 KB |
| transparent membership, depth 32 | 0.15 s cold | 46.7 KB |
| block proof (profile 6), any length | 37–46 s, aggregator | 64,213 B worst case |

a `SPEND` with two inputs at depth 40 and two predicates is estimated at under one second on Metal and 300–500 KB of proof; it has not been measured. phones have not been measured at all. the veil proof's size comes from masked polynomials and a hiding Ligero opening; parameter changes buy at most 5 % (`fast-path-2` §b). this is why §5 has a research gate and not a tuning task.

## 4. what the alternatives would have cost

named once so they stay closed.

- transparent accounts with stealth addresses only: hides the recipient, not the sender or the amount; fails the machine.
- lattice-based zero knowledge (lattice commitments, LaBRADOR-class): a second material in consensus with its own break mode; the proofs are not smaller than the hash-only target and the arithmetic is foreign to the field.
- ring signatures and confidential transactions over a curve: not quantum-safe.
- homomorphic encryption for private computation: a third party computes on ciphertexts; it is a service pattern, not a consensus one, and TFHE parameters are the kind of thing one does not freeze.
- a separate privacy chain: splits the material and the turnstile; the pool must live where the supply lives.

## 5. the three gates, and whether this is research

yes: two of the three gates are research, with uncertain outcomes and hard deadlines; the third is time.

| gate | what passes | kind | estimate | if it fails |
|---|---|---|---|---|
| G1 · small zero knowledge | a hash-only zero-knowledge proof of `SPEND` ≤ 64 KB at ≥ 128 bits (hiding WHIR, or a zero-knowledge compiler over the succinct profile that masks only what the opening reveals), measured, reviewed like the five zheng reviews of October | research | 6–10 sessions, outcome uncertain | veil v1 as reviewed is frozen as the transaction proof; the block proof hides its size from consensus and the mempool pays the bandwidth |
| G2 · one proof per block | the block proof verifies transaction proofs in-circuit (the in-circuit WHIR verifier of `zheng/specs/recursion.md` extended to the veil opening), N transactions per block at a prover cost the aggregator can pay (target: 100 transactions per 30 s block on one M4 Max-class machine), byte-identical across backends, bit-flip scan 0 accepted | engineering on the harness | 8–12 sessions | the pool cannot launch: raw proofs in blocks allow one private transaction per block |
| G3 · the material | hemera has a published cryptanalysis campaign with a bounty, third-party algebraic analysis (Gröbner, interpolation, the partial-round structure), and the one-profile decision of `cyber/whitepaper.md` §4.4 settled | external, time | months; runs in parallel with the transparent chain, which bets on the same material | the pool waits; a hash break in a pool is undetectable counterfeit, bounded only by the turnstile (§1.5) |

the order is G2 first (it is needed whatever G1 yields), G1 in parallel, G3 from today. the transparent root chain does not wait for any of them (`cyber/launch.md`, decision 2026-10-15).

## 6. acceptance

the pool is switched on (rule set 2) when all hold:

1. `SPEND` specified with vectors in `specs/chain-conformance.md` (record, commitment, nullifier, anchor, the public-input encoding, a signed-nothing spend);
2. prove ≤ 1 s on an Apple-silicon laptop for two inputs, two outputs, depth 40, two predicates, cold; measured on one Android phone and stated;
3. G1 or its fallback decided and recorded; G2 passed; G3's campaign running for at least 90 days with no result that lowers hemera below 128 bits classical;
4. the five-review discipline of October applied to the relation (attack tests for every malleability, the ledger rows, bit-flip scans, native-vs-circuit divergence tests);
5. the turnstile tested: a forged proof accepted by a deliberately broken verifier cannot withdraw more than the pool holds;
6. wallets: shielded by default, transparent balances flagged as not quantum-safe, note encryption versioned, viewing keys optional;
7. the shadow indexers of the cores read anonymous edges and produce the same φ* on three nodes.

## 7. consequences the cores must accept

- rewards attach to links, not neurons. the payout key in the link receives what the link earns; Shapley over a cluster is over links. identity-weighted schemes become opt-in disclosures.
- sybil cost is the fee. with no identity, the only rate limit on speaking is the cost of a record; the fee floor of `specs/chain.md` is a consensus parameter and therefore frozen; the economics proposals price it.
- the nullifier set grows forever. it lives in a bbg dimension (chunked commitments, launch row 43's successor); its growth is the cost of this design and is accepted.
- anonymity is a set. at launch the pool is small and timing correlates; the relay stem (§1.11) and wallet defaults (shielded by default, delayed broadcast) are what make the set real. the protocol is not what fails here, usage is, and the page says so.

## 8. what to do now

1. `specs/chain.md`: add §11 "the pool" carrying the genesis-frozen formats of §1.10 and the `SPEND` public-input encoding, so genesis freezes them even though rule set 2 is off. this is a specification change before genesis and is the one urgent item.
2. open the three gates as tracked work: G2 in zheng (branch off `release/0.4`), G1 as a research page under `zheng/docs/explanation/` with the attempts and the measured results, G3 as `hemera/audit/campaign.md` with the bounty terms.
3. retarget mudra's P1 authorship (`mudra/specs/authorship.md`, mudra#22) onto `SPEND`: the authority set becomes the record tree, the scoped nullifier becomes the spend nullifier, and the content commitment becomes the link output. one relation, not two.
