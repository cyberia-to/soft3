---
tags: cyber, soft3, proposal, privacy, zheng, mudra, bbg, post-quantum, launch
crystal-type: process
crystal-domain: cyber
alias: frozen privacy, the shielded pool, privacy proposal, P1 P2 P3 frozen, privacy research, anonymous machine
status: proposed
depends-on: soft3/proposals/self
date: 2026-10-16
reconciled: 2026-10-16 with the second draft of the same day (soft3 PR #190), §9
evidence: audit/privacy-2026-10-16/
---
# privacy, frozen — one pool, one proof, one material

hemera note (2026-10-16): the hemera#15 decision this page waits for is taken — hemera 0.4 (x⁷ in every round, 6 + 6 + 48 rounds, 32-byte digest kept) is the frozen material on hemera `release/0.4`; the G3 campaign runs on it (`hemera/audit/campaign.md`).

the question (owner, 2026-10-16): a privacy scheme that is as efficient as the harness allows today, quantum-safe, lets anyone compute custom things over its outputs, serves consensus, and above all is the machine everyone can speak into where nobody learns who said what and everyone learns what was said. and it must be frozen at launch like the hardware of a satellite: the currency becomes a material, like gold, that nobody can change afterwards.

this page gives the decisions, the relation, what freezes at genesis and what freezes at rule set 2, what has to be researched before the pool can be switched on, and the acceptance gates. it is written against the harness as it stands on 2026-10-16: [[hemera]] over Goldilocks, [[nox]], [[zheng]] profiles 1, 2 (veil) and 6, [[lens]] WHIR, [[bbg]], [[mudra]], the [[chain|phase-1 root chain]] spec, and the measured figures of `audit/proof-latency-2026-10.md` and `zheng/audit/*-2026-10.md`.

a second draft written the same hour by another session (soft3 PR #190) argued the opposite launch verdict: defer the irreversible genesis until the evidence closes, and, in its revision, make encrypted collective computation over hidden contributions a genesis requirement. it is reconciled here, in §9; its evidence, sources and reproducible counterexamples are in [the research receipt](../audit/privacy-2026-10-16/README.md) and are cited where they bind.

## 0. the answer in one paragraph

the money is a single shielded pool of records committed under hemera in one Merkle tree; spending a record is proving, in zero knowledge with a hash-only zheng proof, that you know its secret and that the values balance; speaking is spending: a cyberlink is a public output of a spend, so the edge and its weight are public and the author is the spent record's key, which nobody sees; programs are data bound to records as predicate digests, and consensus verifies one fixed relation forever; blocks carry one folded proof, so a node checks a block in milliseconds whatever the transactions cost to prove; the transparent ledger keeps a counter of what entered and left the pool, so even a broken proof system cannot inflate the supply. consensus rests on one material, hemera, and on nothing else. note delivery may use a lattice KEM, but consensus never reads it, and its failure is disclosure, never forgery.

two moments freeze, and they freeze different things. the genesis of 2026-11-05 freezes the material, the supply and the pool's formats; the root chain is transparent, and everything done on it is public forever. the activation height of rule set 2 freezes the pool's relation and verifying keys, and from that height the privacy promise holds. privacy is not retroactive, and the page does not pretend it is (§1.13).

fundamental research is needed, and its core is two items, not ten: hash-only zero-knowledge proofs of ≤ 64 KB (today 165–419 KB), and verifying those proofs inside the block proof so that a block stays one proof. the third gate is not research but time: cryptanalysis of hemera, the material itself. the reconciliation adds two gates the pool cannot switch on without (a written leakage model, and the lifetime security argument of the composition) and two that gate other things (settlement over secret inputs held by many neurons; anonymity against a global network observer), §5.

<a id="privacy-self-dependency"></a>

## 0.1 mutual dependency with self

The complete private protocol depends on [self's private state-machine core](self.md#1-why-privacy-needs-self-and-self-needs-privacy).
Its first instance runs the network's privacy service: it admits encrypted
contributions, fixes the input set and permitted function, carries key and
state generations through executor replacement, and releases only the result
authorized by the accepted history. The machine remembers previous disclosures,
remaining allowances and pending obligations. These are protocol state with a
fixed transition law, required to make joint computation and controlled release
work over time.

The dependency runs both ways. Privacy supplies the cryptographic mechanisms
that protect self's state and operations; self supplies the persistent use and
release rules that make those mechanisms the complete private service. The
ownership boundary is mechanisms and leakage/security arguments in privacy,
state-machine law and lifecycle in self, with one shared interface and
conformance receipt. Privacy's service instantiates that core with a fixed
program. Agent models and objectives are further applications of the core.

G6 and [self S0](self.md#6-conformance-to-privacy-and-joint-freeze) close together:
bootstrap, accepted state, input/function/result binding, disclosure accounting,
finality, secret succession, recovery, resource bounds and failure behavior.
G4 and G5 qualify the corresponding leakage and security composition. A
threshold compromise can disclose protected state despite the fixed law; the
construction retains its stated corruption and erasure assumptions.

The pool's owner-held spend path has the separate scope and activation gates
below. The complete private machine requested here additionally includes joint
computation and requires the shared self core before its own immutable launch.
A pool activation qualifies that pool; the complete private-protocol claim
requires S0/G6 and every other gate of its declared functionality. The root
chain's existing activation schedule remains the one in §1.13.

## 1. the thirteen decisions

each is final for this design; the alternatives are named once and not reopened.

1. one material. consensus soundness — no counterfeit, no double spend, no forged authorship — rests on hemera over Goldilocks and on the hash-only zheng proofs built from it (Spartan over CCS, Reed–Solomon + WHIR, Fiat–Shamir). no lattice, isogeny, pairing or homomorphic scheme is ever a consensus assumption; the lattice-norm conservation sketch of `bbg/specs/privacy.md` is refuted by a reproduced counterexample (a nonzero imbalance passes its norm test, [receipt](../audit/privacy-2026-10-16/README.md#reproduced-counterexamples)). the only conjectures are the ones every hash-based proof system shares: the random-oracle model for Fiat–Shamir (its quantum version is gate G5) and mutual correlated agreement for Reed–Solomon codes at the shipped rates (`zheng/specs/soundness.md`). the material is hemera as pinned at genesis: the profile decision of [hemera#15](https://github.com/cyberia-to/hemera/pull/15) (S-box, round counts, digest width) is due before 2026-11-05, because after it every choice is a fork.
2. confidentiality may add a key encapsulation. a record's secret note (value, randomness, data) is delivered to its recipient encrypted under ML-KEM-1024 (FIPS 203 as standardized, mudra `seal`, category 5, 1,568-byte ciphertext) inside a reviewed KEM–KDF–AEAD envelope bound to its output commitment. consensus never parses a ciphertext. if the lattice falls, past notes are disclosed and nothing is forged; the KEM is versioned by wallets and changes without a fork. because disclosure of past notes is permanent, the wallet default takes the largest standard parameter set. IND-CCA is not recipient anonymity: until the envelope's key-privacy argument closes (G7), wallets do not claim private recipient delivery. the isogeny scheme (mudra `stealth`) and any native-field KEM are out: slower or younger, and they buy nothing a standard KEM does not.
3. one pool, one record. a record is `(token, value, owner, rho, r, data, predicate)`: token id, a u64 value, `owner = hemera(D_OWNER ‖ sk)`, a uniqueness nonce `rho`, commitment randomness `r`, a data particle, and a predicate binding (the full-width digest of the nox program that must accept any spend; zero means none). every consensus-critical digest of the pool — owner, rho, commitment, nullifier, predicate binding, tree node — is a full-width hemera digest of at least 48 bytes (six Goldilocks elements, ≥ 128-bit generic quantum collision resistance by Brassard–Høyer–Tapp, `2^(n/3)`), squeezed from the same permutation; the 32-byte particle stays a locator for data, never an authority over value or program, because a 32-byte digest gives about 85 bits of quantum collision resistance (`hemera/specs/README.md`) and a format frozen at genesis can never be widened. the commitment is `hemera(D_CM ‖ record)`. commitments live in one hemera Merkle tree of depth 40 (2^40 records). the nullifier is `hemera(D_NULL ‖ chain ‖ sk ‖ rho)`. the rho of output `j` is derived inside the proof, `hemera(D_RHO ‖ nf_0 ‖ j)` from the transaction's first nullifier (for a deposit, from the deposit's transaction id), so no sender can give two records the same nullifier and make one unspendable. every encoding is canonical and every limb is checked `< p`: a single field element would alias distinct identifiers (0 and p are the same element, reproduced in the receipt). a spend names an anchor: any tree root among the last 1,000 blocks.
4. no signatures in the pool. spend authorization is knowledge of `sk` inside the proof. hash-only, quantum-safe, and nothing to rotate. spend, view and transport keys are separate derivations of the spell under distinct domains: a sender who knows the delivery secret never learns the recipient's `sk`.
5. the turnstile is the gold. per token, the chain keeps a transparent counter `pool = Σ deposits − Σ withdrawals`. a withdrawal that would make it negative is invalid regardless of its proof. the pool's true content can never exceed what entered it, so a break of the proof system or of the hash caps counterfeit at the pool's balance and can never raise the supply. the supply schedule of the root chain (1 %/yr, `specs/chain.md` §3) is frozen with it.
6. speaking is spending. a cyberlink is a public output of a spend: `(p, q, weight, payout)` where `p`, `q` are particles, `weight` is a u64, and `payout` is an owner key for whatever the link earns. the fee comes from the spent record. the chain learns the edge and its weight; it never learns which record spoke. one record, one voice per nullifier: reuse is a double spend. this folds P1 into P2. it does not make P3 empty: φ* over public anonymous edges has no attribution to hide, but a settlement over a neuron's own ego-net is a proof over records only that neuron holds (an owner-held witness, §1.7), and today's settlement tickets publish the full marginal vector (`foculus/src/marginal_cert.rs`, launch row 12), which rule set 2 must replace. a neuron who wants a reputation proves later, selectively, that it owns a set of links (a zheng proof over its own records); nothing forces it. no protocol hides an author from the content of the message itself, from unique values, timing or outside facts; the promise is conditioned on those voluntary disclosures. the public weight is one number, never its factors: in tru's attention (`tru/specs/attention.md`) each summand of the effective adjacency is `a(ℓ) · κ(ν(ℓ)) · f(m(ℓ))`, and karma `κ` and the will share inside `a` belong to the neuron, so a published factor, or a product whose other factors are public, is a fingerprint of the author. the spender proves the product inside `SPEND` from its karma and will held as private credentials (owner-held witnesses) and publishes only the product; G4 states what the distribution of products still reveals. whether per-link products are public at all, or hidden with only the summed adjacency released (the revised second draft), is decided by the rule set that activates weights: rule set 1 has none (a cyberlink's amount must be 0, `specs/chain.md` §10 item 2), and the hidden path needs G6. exact sums over overlapping sets reveal individual terms by subtraction (reproduced in the receipt), whichever path is taken.
7. programs are data. a record's predicate binding names a nox program; a spend must present, inside the same proof, that every input's predicate and every output's predicate accepted the transaction (the birth and death predicates of ZEXE, on nox). predicates only add constraints: no predicate relaxes checks 1–4 of §2, and native conservation and issuance are enforced by `SPEND` itself, outside any program's discretion. consensus verifies one relation forever; what people compute changes without consensus. computation splits by who holds the witness:

   | computation | who executes it |
   |---|---|
   | a program over public outputs (φ*, markets, indexes) | anyone; it is indexing and needs no protocol |
   | a program over one neuron's own hidden records (owner-held witness) | that neuron, locally, proving the result with a zheng proof; verifiers learn what the program declares public |
   | a program over hidden inputs held by several independent neurons (joint secret input: valence positions and serum reports, BTS surprise against the crowd, contributor-grouped Shapley settlement, hidden weights) | collective computation: each neuron proves and encrypts its own contribution, the network evaluates over the closed input set, a threshold release discloses only the authorized result; threshold homomorphic encryption (additive for sums after local preprocessing, bounded exact evaluation beyond) as the baseline, malicious-secure MPC as the comparison; or the function is redefined over public or owner-held inputs; self's state-machine core enforces the persistent input/release lifecycle; joint gate G6/S0 |

   no homomorphic or threshold scheme enters consensus soundness. a later rule set may adopt a threshold encryption layer for the confidentiality of contributions only: every input-encryption, evaluation and decryption step is checked by a zheng proof over hemera, so a lattice break or a colluding key committee discloses contributions and never forges a result, the same failure class as the KEM of §1.2. its other failure, a committee too small to release, halts the dependent settlement; the rule set names that outcome, and it is never a plaintext fallback. mudra's native-field `veil` (TFHE) remains unqualified and is not that layer. a central prover that receives everyone's secret inputs is not a solution to the third row even if its output proof is zero knowledge.
8. one proof per block. the miner folds every transaction's proof into the block proof (the constant-size profile of `zheng/specs/recursion.md`: 64,213 B worst case, verify 2.5 ms on 16 threads). nodes verify the block proof and never the transactions' proofs. raw transaction proofs travel only in the mempool, where size costs bandwidth, not consensus. the block budget of 256 KiB therefore holds whatever a transaction proof weighs. the reference semantics stays the per-transaction relation: the block-proof statement is defined as the conjunction of the transactions' `SPEND` statements, and a reference validator that checks raw proofs one by one must reach the same verdict on every block of the conformance corpus (G2). the aggregator learns no spend secrets; it folds proofs, not witnesses.
9. transparent only at the edges. genesis balances, coinbase and legacy-key claims (secp256k1 through the mudra bridge, not quantum-safe) exist on the transparent side to enter the pool. a transparent balance that never enters is not quantum-safe, and the wallets say so. the post-quantum claim covers the pool and nothing else; no proof wrapper makes an exposed classical key quantum-safe. the transparent side is not frozen: before rule set 2 activates, the owner decides whether it narrows transparent-to-transparent transfers (every transparent move thins the anonymity set) and when classical authorization ends for anything but entry into the pool (§6.8). a later rule set may narrow the edge; none may widen it to minting. the cores' shadow indexers read the public outputs; nothing of the cores needs the transparent balances.
10. what freezes at genesis. the material and the formats: hemera as pinned by hemera#15, Goldilocks, nox, the record, commitment, rho and nullifier encodings at full width, the domain tags, tree depth, anchor window, integer-limb conservation, the lifetime nullifier set, the turnstile accounting, the block-proof statement shape, the supply schedule. these are in the genesis rule set. the pool itself, with its verifying keys, is rule set 2 and switches on when the blocking gates of §5 pass. that is not changing the satellite; it is powering on an instrument that was on board at launch. a verifying key set, once in a rule set, is never changed. the formats are chosen so that no gate outcome can invalidate them (§1.13): that is why they are full width, integer, canonical and lifetime now, before the gates close. the nullifier set is lifetime and never reset: an epoch reset that keeps only a live table accepts an already spent nullifier (reproduced in the receipt); a node may prune storage only where an authenticated proof of non-membership over all history still rejects every old spend.
11. the network layer is not frozen. who broadcasts a transaction is hidden at the relay layer (a Dandelion-style stem over [[radio]], wallets choosing their first hop); it is a wallet and node matter, versioned freely. a stem hides the origin from peers, not from a global observer; whole proof bundles must be padded and scheduled as one, since packet counts identify a transaction even when packets are fixed size. until a reviewed mix composition with cover traffic passes G7, wallets state the relay-level guarantee and never claim end-to-end anonymity; a direct, identity-bearing submission is labelled as such.
12. consensus weight is frozen too. anything that touches consensus — a stake amplifier, private staking — is a rule set and needs a fork; so stake privacy is designed before any proof-of-stake activation, as records with a stake predicate, and never bolted on. private weighted votes need an exclusivity tag derived per conflict domain and round, never per proposal (a per-proposal tag lets one stake vote for every competitor), stake locked through the voting and slashing interval, and one rule for switching, withdrawal and reorgs; splitting a credential never creates weight. foculus's author-based conflict check (`foculus/specs/protocol.md`) loses its author under anonymous edges, and its safety and liveness argument is redone before rule set 2.
13. immutable, and from when. three layers:

    | layer | immutable from | what |
    |---|---|---|
    | the material, the supply, the formats | height 0 of each chain (pussy 2026-11-05 13:37:00 UTC; bostrom at its own genesis) | §1.10 |
    | the pool | the activation height of rule set 2 | `SPEND`, its verifying keys, the block-proof verifying key, the turnstile as live consensus |
    | everything else | never | provers (faster conforming implementations), wallets, the KEM profile, the relay layer, indexers, and the transparent edge of rule set 1, which may only narrow |

    immutable means: no later rule set changes, removes, reinterprets or adds to it. a record created under rule set 2 stays spendable under rule set 2's relation forever; there is no automatic fallback to a public proof, no master mint or recovery key, no verifier replacement by vote or by a later rule set. an implementation that fails the specification may be repaired; the specification may not. a chain that changes any of it is a different chain, which nobody can forbid and which is not this currency. the privacy promise holds from the activation height of rule set 2; everything before it, and every transparent action after it, is public forever. the burial snapshot was public before genesis, so the genesis balances lose nothing; what the transparent window costs is the activity of its own months, and the launch page says so.

## 2. the relation, once

one relation `SPEND`, verified by one verifying key per rule set.

public inputs:

| field | meaning |
|---|---|
| chain | the chain id (genesis particle) |
| anchor | a commitment-tree root among the last 1,000 |
| nullifiers | `n` of them, `1 ≤ n ≤ 4`, full width, pairwise distinct |
| commitments | `m` new commitments, `m ≤ 4` |
| outputs | public outputs: cyberlinks `(p, q, weight, payout)`, a transparent withdrawal `(to, token, amount)`, both or neither |
| fee | token and amount, paid to the miner |
| predicates | the predicate bindings of all inputs and outputs, in order |
| notes | ciphertexts, opaque to consensus, each bound to its output commitment |

witness: the input records, their Merkle paths and secrets `sk`, the output records' openings, the predicate programs' witnesses.

checks inside the proof:

1. every input commitment opens to a record whose owner is `hemera(D_OWNER ‖ sk)`, and sits in the tree at the anchor; every encoding is canonical, every limb `< p`;
2. every nullifier is `hemera(D_NULL ‖ chain ‖ sk ‖ rho)` of its input;
3. per token, Σ input values = Σ output values + fee + withdrawal as integers: every value a u64 in two 32-bit limbs under nox's word ops, the sums carried across limbs, never equality modulo p (the u64 value p is zero in the field and would pass a field equation; reproduced in the receipt);
4. every output commitment opens to its record with the committed randomness, and its rho is `hemera(D_RHO ‖ nf_0 ‖ j)`;
5. every input's and output's predicate program, run on the transaction's public inputs and the predicate's own witness, returns zero (the nox machine relation of `zheng/specs/machine.md` covers every opcode);
6. the public outputs equal what the predicates and records declare.

checks outside the proof, by every node: the anchor is recent, no nullifier is in the lifetime set or repeated within the block, the turnstile stays non-negative, the fee is at least the floor, the block proof verifies. a membership proof against a historical root proves the record existed, never that it is unspent; unspentness is the nullifier check, against the node's own canonical state.

deposits are transparent-side transactions that add commitments without nullifiers; the turnstile counts them.

## 3. what it costs today

measured on this harness (Apple M4 Max, Metal; `audit/proof-latency-2026-10.md` §5 and `zheng/audit/fast-path-2-2026-10.md`), for the closest shapes that exist:

| statement | prove | proof |
|---|---|---|
| veil, secret leaf, depth 32, node tree | 96 ms warm, 0.18 s cold | 165–181 KB |
| P1 authorship (mudra#22), depth 20 / 32, held key | 255 / 396 ms | 292 / 419 KB |
| transparent membership, depth 32 | 0.15 s cold | 46.7 KB |
| block proof (profile 6), any length | 37–46 s, aggregator | 64,213 B worst case |

a `SPEND` with two inputs at depth 40 and two predicates is estimated at under one second on Metal and 300–500 KB of proof; it has not been measured. every figure above uses 32-byte digests and one permutation per tree level (`zheng/audit/fast-path-2-2026-10.md`); the full-width digests of §1.3 cost two permutations per level at width 16 (one at width 24, the other option of hemera#15), also unmeasured, and no measured timing transfers to a different hash, tree, statement or parameter set. phones have not been measured at all. the veil proof's size comes from masked polynomials and a hiding Ligero opening; parameter changes buy at most 5 % (`fast-path-2` §b). this is why §5 has a research gate and not a tuning task.

## 4. what the alternatives would have cost

named once so they stay closed. the six counterexamples of [the research receipt](../audit/privacy-2026-10-16/README.md#reproduced-counterexamples) are the evidence for the second list below; `python3 -I audit/privacy-2026-10-16/counterexamples.py` reproduces them (rerun 2026-10-16 on main, output byte-identical, all assertions pass).

- transparent accounts with stealth addresses only: hides the recipient, not the sender or the amount; fails the machine.
- lattice-based zero knowledge (lattice commitments, LaBRADOR-class): a second material in consensus with its own break mode; the proofs are not smaller than the hash-only target and the arithmetic is foreign to the field.
- ring signatures and confidential transactions over a curve: not quantum-safe.
- homomorphic encryption for private computation: a third party computes on ciphertexts; it is a service pattern, not a consensus one, and TFHE parameters are the kind of thing one does not freeze.
- a separate privacy chain: splits the material and the turnstile; the pool must live where the supply lives.
- a second hash for security commitments (SHA3-512 / SHAKE256 beside hemera): a second material in consensus and a foreign hash inside every proof; the width it would buy is had from hemera's own squeeze (§1.3). SHA-3 stays the comparison baseline of gate G5, never a consensus primitive.
- deferring genesis until every gate closes: buys no evidence, because nothing the gates qualify is frozen before rule set 2 (§1.13, §9).
- a threshold encryption layer as a genesis prerequisite: at genesis there are no weights, no valence and no settlement on chain, so there is nothing for it to compute; it is a candidate for the rule set that activates them, under G6.

and the sketches that failed when written out, with the counterexample that closes each:

- conservation by a lattice ciphertext norm (`bbg/specs/privacy.md`): the displayed components cancel to the message, and a constant polynomial 1 is a nonzero imbalance with norm 1 < q/2 that passes the test. conservation is integer equality inside the proof.
- conservation modulo p: the u64 value p is zero in Goldilocks. limbs with carries.
- nullifier as one field element: 0 and p alias. full-width canonical digests.
- nullifier table reset at an epoch: a live-only table after the reset accepts an already spent nullifier. the set is lifetime.
- hidden contributions behind exact public aggregates: a release that moves from 100 to 107 reveals the contribution 7; colluders who know all summands but one recover it (17). aggregates over hidden inputs need a fixed release function, cadence and a stated residual leakage (G4, G6).

## 5. the gates, and whether this is research

yes. G1, G4 and G5 are research with uncertain outcomes; G2 is engineering on the harness; G3 is time. these five block rule set 2. G6 and G7 are research too, but they gate other things: G6 the rule set that activates a function over secret inputs held by many neurons, G7 the claim of end-to-end anonymity. G4–G7 come from the second draft (§9), with its numbering mapped there.

| gate | what passes | kind | estimate | if it fails |
|---|---|---|---|---|
| G1 · small zero knowledge | a hash-only zero-knowledge proof of `SPEND` ≤ 64 KB at ≥ 128 bits (hiding WHIR, or a zero-knowledge compiler over the succinct profile that masks only what the opening reveals), measured, reviewed like the five zheng reviews of October | research | 6–10 sessions, outcome uncertain | veil v1 as reviewed is frozen as the transaction proof; the block proof hides its size from consensus and the mempool pays the bandwidth |
| G2 · one proof per block | the block proof verifies transaction proofs in-circuit (the in-circuit WHIR verifier of `zheng/specs/recursion.md` extended to the veil opening), N transactions per block at a prover cost the aggregator can pay (target: 100 transactions per 30 s block on one M4 Max-class machine), byte-identical across backends, bit-flip scan 0 accepted | engineering on the harness | 8–12 sessions | the pool cannot launch: raw proofs in blocks allow one private transaction per block |
| G3 · the material | the profile decision of [hemera#15](https://github.com/cyberia-to/hemera/pull/15) taken before genesis (it is the one-profile decision of `cyber/whitepaper.md` §4.4: S-box in every round, round counts by a stated rule, digest width; the inverse-round degree argument is withdrawn by hemera's own research); then a published cryptanalysis campaign against the pinned profile with a bounty, third-party algebraic analysis (Gröbner, interpolation, the partial-round structure) | external, time | the decision: before 2026-11-05; the campaign: months, in parallel with the transparent chain, which bets on the same material | the pool waits; a hash break in a pool is undetectable counterfeit, bounded only by the turnstile (§1.5) |
| G4 · the leakage model | a written observer and corruption model for `SPEND`, link outputs, settlement and every released aggregate: what a node, a miner, a relay and a coalition of contributors learn; the author-swap game (two honest authors exchanging otherwise identical public assertions are indistinguishable under the declared network and corruption assumptions); sparse, overlapping and colluding releases analyzed with their residual leakage stated | research, specification | 2–4 sessions | the pool waits: without it the privacy claim of §0 has no content to prove G1 and G5 against |
| G5 · lifetime security | the deployed composition argued in the quantum random-oracle model (Fiat–Shamir, grinding, multi-target and lifetime losses, nested verification inside the block proof); the zero-knowledge error within a stated lifetime budget — today's ledger bounds it by `types · log(m) / p`, about 2^-58.4 in its own example (`zheng/specs/soundness.md` on `release/0.4`), an upper bound and not a leak, but not a lifetime budget either; fresh hiding entropy per proof; the pool digests at ≥ 128-bit generic quantum collision; an independent verifier and adversarial fixtures; SHA-3 as the comparison baseline | research | 4–8 sessions, outcome uncertain | the pool waits; security parameters are never lowered to pass G1 |
| G6 · secret inputs held by many | for every function whose inputs no single prover holds (valence positions and serum reports, launch row 29; hidden per-link weights; BTS surprise; contributor-grouped Shapley settlement): the exact function written as a bounded circuit and decomposed (local products proved by their owners, encrypted sums, what needs joint evaluation); a post-quantum threshold encryption protocol (baseline: exact RLWE threshold HE, multiparty BFV as the starting point, whose own model is semi-honest) or malicious-secure MPC, compared at the same leakage and corruption model; distributed key generation, epoch keys with fixed resharing and erasure, a stated corruption threshold and its liveness failure outcome; zheng proofs of input-encryption consistency, evaluation over exactly the closed input set and decryption bound to the approved result (a key committee that decrypts arbitrary ciphertexts is a decryption oracle for individual inputs); canonical bounded encodings with no plaintext-modulus wraparound and a decryption-failure budget; packed-slot sanitization; a release policy whose singleton, incremental and overlapping releases are analyzed under G4; dropout, exclusion, reorg and payout fixtures; [self S0](self.md#6-conformance-to-privacy-and-joint-freeze) passed on the same service state/law, bootstrap and release-accounting fixtures | research | open | blocks the complete private service and the rule set that activates that function (weights beyond public products, valence, private settlement), not the pool and not genesis; a central prover holding everyone's inputs is not a fallback |
| G7 · network anonymity | a reviewed post-quantum mix and delivery composition (fixed packet classes, whole-bundle padding, independent delay, cover traffic reaching the observed layer, replay defenses, an authenticated relay directory); active, low-load and intersection experiments; key-private recipient delivery for the KEM envelope | research and engineering | open | blocks the claim, not the pool: wallets state the relay-level guarantee of §1.11 until it passes |

the order is G2 first (it is needed whatever G1 yields), G4 at once (it defines what G1 and G5 must hide), G1 and G5 in parallel, hemera#15 before genesis and the G3 campaign from then. G6 closes jointly with self S0 on one privacy-service state machine. It runs against the real attention sum, valence, BTS and settlement functions, not an unrelated private sum, and its first deliverable is the decision of §1.6 (public products or hidden weights); G7 runs in radio. the transparent root chain does not wait for any of them (`cyber/launch.md`, decision 2026-10-15).

## 6. acceptance

the pool is switched on (rule set 2) when all hold:

1. `SPEND` specified with vectors in `specs/chain-conformance.md` (record, commitment, rho derivation, nullifier at full width, anchor, limb conservation, the public-input encoding, a signed-nothing spend, non-canonical encodings rejected);
2. prove ≤ 1 s on an Apple-silicon laptop for two inputs, two outputs, depth 40, two predicates, cold; measured on one Android phone and stated; the receipt reports medians and tails, peak memory, first and repeated verification, proof and ciphertext bytes, witness maintenance and recovery time, with revisions, hardware, parameters and commands; no security parameter is lowered to make a row green;
3. G1 or its fallback decided and recorded; G2 passed; G3's campaign running for at least 90 days against the profile pinned at genesis with no result that lowers hemera below 128 bits classical; G4 accepted; G5 closed;
4. the five-review discipline of October applied to the relation (attack tests for every malleability, the ledger rows, bit-flip scans, native-vs-circuit divergence tests), and a malicious-prover suite: inflation, limb wraparound, forged authority, same-transaction, same-block and cross-anchor double spends, two outputs given one rho, aliased and non-canonical nullifiers, a predicate that tries to relax checks 1–4; the reference nox evaluation of predicates agrees with the circuit on authorization, calls, branches, private reads and successor predicates;
5. the turnstile tested: a forged proof accepted by a deliberately broken verifier cannot withdraw more than the pool holds;
6. wallets: shielded by default, transparent balances flagged as not quantum-safe, note encryption versioned (ML-KEM-1024 in a reviewed envelope), spend, view and transport keys separate, viewing keys optional; recovery is part of ownership: a spell plus a public archive restores every record by a complete local scan (the reference algorithm, `mudra/specs/private-recovery.md`), membership paths are never fetched by address-specific RPC, witness updates survive anchor rotation and reorgs, and lost archives, cold restore and key epochs have executable behavior;
7. the shadow indexers of the cores read anonymous edges and produce the same φ* on three nodes;
8. the transparent edge decided and recorded on `cyber/launch.md`: whether rule set 2 narrows transparent-to-transparent transfers, and the height after which classical authorization serves only entry into the pool.

## 7. consequences the cores must accept

- rewards attach to links, not neurons. the payout key in the link receives what the link earns; Shapley over a cluster is over links. identity-weighted schemes become opt-in disclosures.
- sybil cost is the fee. with no identity, the only rate limit on speaking is the cost of a record; the fee floor of `specs/chain.md` is a consensus parameter and therefore frozen; the economics proposals price it.
- the nullifier set grows forever. it lives in a bbg dimension (chunked commitments, launch row 43's successor); its growth is the cost of this design and is accepted.
- anonymity is a set. at launch the pool is small and timing correlates; the relay stem (§1.11) and wallet defaults (shielded by default, delayed broadcast) are what make the set real. the protocol is not what fails here, usage is, and the page says so. every transparent transfer thins the set; a global observer is not defeated by a stem (G7).
- aggregates leak. any exact public sum over hidden contributions reveals a contribution to whoever knows the rest or sees two releases differing by one contributor. what is public per link stays public; what is hidden is released only through a fixed function and cadence with its residual leakage stated (G4, G6).
- the window is public. from genesis to rule set 2 the root chain is transparent and its history stays public forever (§1.13).
- settlement changes shape. the settlement ticket's public marginal vector (launch row 12) and foculus's author-based conflict check cannot survive anonymous edges; both are rewritten before rule set 2 (§1.6, §1.12).

## 8. what to do now

1. `specs/chain.md`: add a section "the pool" carrying the genesis-frozen formats of §1.10 at full width (record, rho derivation, nullifier, domain tags, tree depth, anchor window, limb conservation, the lifetime nullifier set, the turnstile) and the `SPEND` public-input encoding, so genesis freezes them even though rule set 2 is off. this is a specification change before genesis and is one of the two urgent items.
2. the owner decides [hemera#15](https://github.com/cyberia-to/hemera/pull/15) before 2026-11-05: the root chain freezes hemera at genesis whatever the pool does, and the pool's digest width follows from it. the second urgent item.
3. open the gates as tracked work: G2 in zheng (branch off `release/0.4`), G1 as a research page under `zheng/docs/explanation/` with the attempts and the measured results, G3 as `hemera/audit/campaign.md` with the bounty terms, G4 as a leakage-model specification beside `SPEND`, G5 as rows of `zheng/specs/soundness.md` (quantum random-oracle model, lifetime hiding budget), G6 with foculus and tru against launch rows 12 and 29, G7 in radio.
4. retarget mudra's P1 authorship (`mudra/specs/authorship.md`, mudra#22) onto `SPEND`: the authority set becomes the record tree, the scoped nullifier becomes the spend nullifier, and the content commitment becomes the link output. one relation, not two.
5. state the window on `cyber/launch.md`: from genesis to rule set 2 the root chain is public, its history stays public, and the privacy promise begins at the activation height.

## 9. the second draft, reconciled

on 2026-10-16, within the hour of this page, another session of the same owner opened soft3 PR #190: `proposals/privacy.md`, "privacy — a frozen anonymous machine", with a research receipt. it selects the same architecture — private programmable notes, hash-based ownership and nullifiers, anonymous public assertions — and reaches the opposite launch verdict: the current public chain cannot be the genesis of a permanently private post-quantum currency, so defer the irreversible monetary freeze until its gates close. this section is the reconciliation; the PR was closed unmerged and its receipt moved to [audit/privacy-2026-10-16](../audit/privacy-2026-10-16/README.md). This page owns the reconciled privacy design; [self](self.md) separately owns the private state-machine law and lifecycle required by §0.1. the draft has two commits: ea413d4, reconciled by soft3 #191 in the subsections up to "dropped", and b011039, pushed at 10:29 UTC while #191 was being written, reconciled in "the revision" at the end.

### the conflict, resolved

the genesis of 2026-11-05 freezes the material (hemera as pinned by hemera#15, Goldilocks, nox), the supply (S0, the 1 %/yr subsidy, the conservation invariant of `specs/chain.md` §3.4) and the pool's formats (§1.10); it freezes no relation and no verifying key of the pool. the pool's `SPEND` relation and verifying keys are pinned at the activation height of rule set 2, which waits for G1–G5, and from that height nothing adds to, removes or replaces them (§1.13). against that reading the second draft's objection does not stand as a whole — nothing its gates qualify is frozen before they close, so deferring genesis buys no evidence — but it stands in three parts, now folded in:

1. privacy cannot be retroactive. the root chain is public from height 0, so the promise is "private from the activation height of rule set 2", stated on this page (§0, §1.13), in the consequences (§7) and on the launch page (§8.5), and never "private from genesis".
2. the transparent edge is classical. secp256k1 authorization is not post-quantum, and keeping it indefinitely keeps its classical security; the post-quantum claim covers the pool only, and the narrowing of the edge is an owner decision required before rule set 2 (§1.9, §6.8).
3. a format frozen before its gate must survive the gate. a 32-byte digest frozen at genesis would cap the pool at about 85 bits of quantum collision resistance forever, a one-element nullifier aliases, a field-equality balance wraps, a resettable nullifier table forgets; so the formats freeze now at full width, canonical, integer-limb and lifetime (§1.3, §1.10, §2), and the material's own profile is decided before genesis (hemera#15, G3).

### decisions diff

| topic | this page (#188) | the second draft (#190) | result |
|---|---|---|---|
| architecture | one shielded pool of records under hemera, nullifiers, programs as predicates | one mandatory shielded note pool, hash ownership and nullifiers, bounded nox policies | agrees |
| commitment backend | hemera Merkle tree, depth 40 | conventional append-only hash tree first; polynomial or mutator-set backends only after qualification | agrees; mutator sets stay a measured alternative |
| public assertions | speaking is spending: edge public, author private, fee from the spent record | P1 publishes `(p, q, payload)` with anonymous eligibility; Sybil cost is the scarce resource | agrees |
| homomorphic and threshold schemes | off-chain services, never consensus | outside the monetary kernel; no global decryption key | agrees |
| verifier immutability | a verifying key set, once in a rule set, is never changed | no verifier replacement by vote, no public-proof fallback, no master key | agrees; folded into §1.13 |
| proof targets | ≤ 64 KB, ≤ 1 s warm on device | the same targets, security never lowered to meet them | agrees; §6.2 |
| evidence | measured latency figures | workspace claim table, `sources.json` with revisions and checksums, six reproducible counterexamples | adds; moved to the receipt, cited in §1, §2, §4 |
| who holds the witness | not distinguished; P3 called empty | owner-held witness vs secret inputs held by many neurons | adds; §1.6, §1.7, G6 |
| conservation and identifiers | u64 in two limbs; one nullifier formula | integer conservation with carries; full-width canonical identifiers; lifetime nullifier set | adds; §1.3, §1.10, §2 |
| digest width | 32-byte hemera | ≥ 128-bit quantum collision for security bindings; particle as locator only | adds; §1.3 at ≥ 48 bytes from hemera itself |
| lifetime security | hemera cryptanalysis only | quantum random-oracle model, hiding error budget, multi-target losses | adds; G5 |
| leakage model | not written | observer model, author-swap game, aggregate release analysis | adds; G4 |
| recovery | viewing keys optional | recovery is part of ownership: spell plus archive, full local scan, no address-specific RPC | adds; §1.4, §6.6 |
| delivery | ML-KEM-768, wallet-versioned | ML-KEM-1024 with a reviewed envelope; recipient-key privacy argued separately | adds; §1.2 (wallet default, no fork) |
| network | Dandelion-style stem, not frozen | a mix layer with cover traffic against a global observer, a freeze gate | partly adds: the layer stays unfrozen, the claim is gated, G7 |
| consensus weight | stake privacy designed before proof of stake | per-domain exclusivity tags; foculus conflict check redone | adds; §1.12 |
| settlement | P3 empty | tickets publish the marginal vector today; P3 is an owner-held proof | adds; §1.6, §7 |
| block verification | one folded proof is consensus | direct verification is the reference semantics; recursion an optimization | conflicts; folding stays consensus (raw proofs exceed the 256 KiB block), the per-transaction relation becomes the reference semantics with a differential gate, §1.8, G2 |
| security hash | hemera only | SHA3-512 / SHAKE256 for security commitments | conflicts; hemera kept (one material, and the chain already rides it), the width point taken from hemera's own squeeze, SHA-3 the G5 baseline only, §4 |
| transparent lane | transparent at the edges | no cheap public lane; no indefinite secp256k1 | conflicts in part; the edge stays in rule set 1 and its narrowing is an owner decision before rule set 2, §1.9, §6.8 |
| when the freeze happens | formats at genesis, pool at rule set 2 | defer the irreversible genesis until the gates close | conflicts; resolved above |

gate mapping, the second draft's numbering to this page's: its G1 (leakage) → G4; its G2 (hash and proof composition, quantum and lifetime) → G3 and G5; its G3 (conservation, spendability forever) → §2 and §6.4–6.6; its G4 (contracts under bounded execution) → §6.4; its G5 (secret collective inputs) → G6; its G6 (ingress and recovery against a global observer) → G7 and §6.6; its G7 (product budgets) → G1 and §6.2.

### dropped, and why

- deferring the 2026-11-05 genesis: the root chain is the owner's decision of 2026-10-15, and under §1.13 the deferral protects nothing the gates qualify.
- SHA-3 as a second consensus material: decision 1; the width it buys comes from hemera.
- direct transaction verification as the consensus rule: a 165–419 KB proof fills a 256 KiB block; kept as reference semantics only.
- a mandatory mix layer as a freeze condition: the network is not consensus and is versioned freely; the guarantee it would add is gated as a claim instead.
- "if hidden-input rewards define the currency, the whole genesis waits": the currency at genesis is the transparent root chain with a frozen supply; hidden-input functions arrive by the rule set G6 opens.
- the duplicate privacy proposal page: this page owns privacy; self owns the coupled private state-machine question (§0.1).

### the revision: encrypted collective computation

b011039 moves the second draft from "owner-held proofs plus anonymous public assertions, joint secrets in a conditional lane" to "a mandatory post-quantum collective computation layer at genesis": each neuron proves its contribution locally and submits it encrypted; the network sums or evaluates over the sealed contributions under a threshold key and releases only the authorized aggregate. individual weights and reports stay hidden; `A_eff` is released, and φ* is computed publicly over it. its argument is concrete: tru's attention sum carries per-neuron factors, BTS surprise needs the crowd, Shapley needs coalitions, and no local proof sums unknown witnesses.

resolution, on the reading of §1.13: the timing objection falls again, and three of its points stand.

- falls: a genesis prerequisite. rule set 1 carries no weights (amount 0, `specs/chain.md` §10 item 2), no valence and no settlement; nothing collective is computed on chain at genesis, so nothing waits for the layer. it is a candidate for the rule set that activates weights, valence or private settlement, behind G6.
- stands, folded into §1.6: per-link weights under tru's formula carry karma and will, which belong to the neuron; a published factor, or a product with public cofactors, names the author. the spender proves the product from private credentials and publishes only the product, and G4 states the residual.
- stands, folded into §1.6 and G6: whether per-link products are public or hidden with only the sum released is a real choice the weights rule set makes, and the hidden path is the revision's protocol. it is not decided here.
- stands, folded into §1.7 and G6: BTS surprise and contributor-grouped Shapley need joint evaluation; threshold HE (additive after local preprocessing, bounded exact beyond) is the baseline and malicious-secure MPC the comparison; key generation, epoch keys, decryption bound to the approved result, packed-slot sanitization, plaintext-modulus bounds and the liveness failure outcome are G6's obligations.
- conflicts with decision 1, resolved: a threshold layer is confidentiality, never soundness. every encryption, evaluation and decryption step is checked by a zheng proof over hemera; a lattice break or a colluding committee discloses and never forges.

| topic | this page | the revision (b011039) | result |
|---|---|---|---|
| collective computation | MPC lane for joint secrets, off consensus | mandatory threshold HE layer at genesis | conflicts in timing; a later rule set behind G6, confidentiality only, §1.7 |
| individual weights | public per link | hidden; only the aggregate released | open for the weights rule set; public products from private credentials by default, §1.6 |
| karma and will in the weight | not addressed | per-neuron factors of tru's attention sum | adds; never published as factors, §1.6 |
| BTS and Shapley settlement | G6, MPC baseline | the aggregate graph cannot settle them; joint evaluation required | agrees; G6 widened |
| key authority | not addressed | DKG, epoch keys, no master key, collusion discloses, absence halts, no plaintext fallback | adds; §1.7, G6 |
| trilateral | not addressed | three jobs: prove inputs, compute jointly, distribute disclosure | agrees for the joint functions; owner-held spends need only the first |

dropped from the revision: the layer as a genesis prerequisite (nothing to compute at genesis); "owner-only ZK and public individual outputs fails the machine" as a blanket verdict (it fails for functions over others' secrets and for per-neuron factors in clear, both now handled, and holds for spends and public assertions); the layer as part of consensus soundness (decision 1).
