---
title: the assumptions registry
tags: cyber, soft3, eidos, hemera, zheng, proposal, security
crystal-type: proposal
crystal-domain: cyber
alias: assumptions registry, registry of assumptions, Axioms.ei, native proofs
status: proposed
date: 2026-10-27
---
# the assumptions registry

**status:** proposed · **date:** 2026-10-27, revised 2026-10-30 (review round 1) · **owner's call:** "a clear registry of scientific assumptions; Lean is foreign, eidos is native — whatever eidos lacks to replace Lean must be built."

## 0. the decision

The stack keeps one list of everything it assumes and cannot prove. That list is the axiom file of the stack's own proof corpus in [[eidos]]: an assumption is an axiom, a consequence is a theorem, and nothing load-bearing exists outside the two. Every assumption is classed, every class has a rule about what it may carry, every entry says what breaks if it is false, how it is measured, and what to watch. Proofs about the stack are written in eidos; Lean is accepted only as evidence from outside, never as the stack's own record. Where eidos cannot yet carry a proof the stack needs, the missing piece is built in eidos; the stack does not borrow a foreign kernel.

## 1. why now

Between 2026-10-05 and 2026-10-26, machine-found results knocked down three groups of hypotheses that complexity theory and cryptography had relied on:

- 2026-10-05: the 3SUM, APSP and Exact Triangle hypotheses. Alman and Vassilevska Williams give a truly subquadratic 3SUM and a truly subcubic APSP algorithm (arXiv:2610.06783, v1 of 5 October). The paper states that "Claude, an AI model developed by Anthropic, discovered the algorithm" (p. 1 and § Acknowledgments and Methodology), and that its main results are certified in Lean.
- 2026-10-06: the Unique Games Conjecture, proved in OpenAI's `openai/math` release ([[cyber/audit/cryptography-2026-10-08|the cryptography audit]] §6).
- two weeks after the 3SUM paper: OpenAI's preprint "The Exponential Time Hypothesis is False". It is dated 17 October 2026 and revised 25 October 2026, and has been in `openai/math` since 2026-10-26. It gives a deterministic algorithm that counts the satisfying assignments of a k-CNF in subexponential time and polynomial space, and states that this refutes ETH, SETH, Gap-ETH, NSETH and #ETH (its §1). Its Lean formalization proves a weaker time bound: the exponent has (log log n)² where the manuscript has log log n (`lean/docs/378.md` in `openai/math`). News coverage reports that the verification is led by Russell Impagliazzo, Ramamohan Paturi and Ryan Williams, who state that their involvement does not certify the claim. We report this; we have not verified it.

The machines that do this are the same machines that will be pointed at [[hemera]], at the economy and at the network.

No consensus rule of the stack rests on an asymptotic conjecture, so none of these results touches it:

- hemera's security is a concrete claim about one fixed permutation. A public campaign will measure it ([[hemera/audit/campaign|the campaign]], a draft whose day 0 comes when 0.4's constants, vectors and instance table are published together).
- [[zheng]]'s soundness rests on Schwartz–Zippel, code proximity, a random oracle, and the three rows its own ledger marks conjectured (C-04 to C-06 below).
- the economy is integer arithmetic on bbg.
- the network is Nakamoto consensus, with bounds measured by the simulator ([[simulation/audit/network-2026-10|the simulator receipt]]).

So far this has held by habit. This proposal makes it a rule, writes the list down in one place, and gives the list a proof checker of the stack's own.

Lean is the wrong checker for this list. Its kernel, its library and its community are outside the stack. A Lean certificate proves a statement to a Lean user. The stack can never re-execute it, store it as a particle, certify it with a zheng proof, or verify it inside a block.

eidos is designed to do all four. In that design the type checker is a [[nox]] program, zheng certifies its execution, and the certificates are cyberlinks (`eidos/specs/kernel.md` and [[eidos/specs/certificate|certificate]], both draft). That design is not built yet. Today eidos has:

- a Rust CIC kernel (`rs/src/kernel.rs`, 620 lines), an elaborator and a tactic engine;
- 139 tests;
- 27 theorems over nox and prysm, which rest on 107 declared axioms.

Several pieces are still missing:

- the nox kernel exists only as a draft specification;
- the certificate is an FNV-1a placeholder (`rs/src/certificate.rs:142`);
- `eidos verify` and `eidos search` print "not yet implemented" and exit 1;
- soft3's status page lists eidos as blocked.

A theorem about the stack belongs in the graph the stack maintains, checked by the stack's own machine. §4.2 lists every eidos gap that matters to the registry, and §4.3 puts the work in order.

## 2. the registry

### 2.1 classes and the load-bearing rule

| class | what it is | example | may carry consensus |
|---|---|---|---|
| A · information-theoretic | theorems that hold without any computational hypothesis | Schwartz–Zippel; sum-check error ≤ (d + 1)/p³ per round over Fp3 | yes; each is a theorem, in the literature now and in eidos by the ladder |
| A′ · tested invariant (a sub-class of A) | an A-shaped statement about the stack's own code that holds by test, conformance vector or measurement and is not yet a theorem | conservation of tokens; ASERT exactness; determinism of settlement | yes, with its test named; it becomes A when its eidos theorem checks |
| B · concrete cryptographic | the strength of one fixed primitive, equal to its best known attack | hemera 0.4: generic collision 2^128, interpolation degree 7^60 > p | yes, with a measured attack and a standing campaign |
| C · proof-system conjectures | the idealizations every hash-based proof system shares, and every row zheng's soundness ledger marks conjectured | random oracle for Fiat–Shamir and its quantum form; recursive Fiat–Shamir; the relation compiler | yes, named as such, never silently |
| D · asymptotic complexity conjectures | statements about all algorithms on all input sizes | ETH, SETH, Gap-ETH, NSETH, OVC, UGC, P ≠ NP | never; context only |
| E · public-key assumptions at the edge | discrete logarithms on a curve; module lattices | secp256k1 for every phase-1 signature; ML-KEM-1024 for note delivery | outside the pool, with a sunset height; an entry that consensus never reads may instead be versioned without a fork |
| F · network and physical | the environment of a Nakamoto chain | honest-majority hashrate; partitions heal within the finality window; clock skew; settlement liveness | yes, with bounds measured by the simulator |
| G · economic | parameters and objectives that are choices | ρ = 1, karma = 1, the 1 % cap, the 7-day lock, syntropy as the objective | yes, with measured attack payoffs |
| H · social | trust placed in a key or a person, with a sunset | the checkpoint key; owner-only promotion; the bounty | yes, with the sunset written |

The rule:

- a consensus rule may rest on A, A′, B, C, F, G and H;
- it may never rest on D;
- it may rest on E only outside the pool, and only once a sunset height is written.

A specification that introduces a load-bearing assumption the registry does not list is red. §2.4 gives the mechanisms that enforce this.

### 2.2 one entry

An entry has two parts. Requirements live in the specification. Observations live in the audit receipt.

The specification part, in `specs/assumptions.md`:

```
id:          B-01
class:       B
statement:   hemera 0.4 (t = 16, R_F 6 + 6, R_P 48, x⁷ in every round, 32-byte
             digest) admits no collision and no preimage below 2^128 classical
consumers:   cybernode/specs/chain.md §1.2 (PoW), §3.3 (state root), §11.1
             (pool digests); bbg/specs/state.md; particle identity; the zheng
             hash row and every row that rests on it (zheng/specs/soundness.md)
parameter:   32-byte digest: collision 2^128 classical, 2^85.3 quantum (BHT);
             64-byte pool digests (chain.md §11.1): about 2^170 quantum
measured-by: hemera/audit/campaign.md §6 criterion 2; audit/campaign/p2rounds.py,
             audit/campaign/cico1.py
watch:       ePrint "Poseidon2" "Goldilocks" "CICO" "Gröbner" "interpolation";
             openai/math families over F_p polynomial systems; the bounty inbox
if-false:    a collision forges graph identity and state roots; response:
             scheduled fork to the next profile, every hemera-derived value
             re-derived; the pool's formats are already 64 bytes wide
eidos:       axiom hemera_0_4_collision_128, generated into Axioms.ei; no
             theorem consumes it yet (the first consumer is E4's transcript
             theorem)
```

The observation part, one row of `audit/assumptions/<date>.md`:

```
id:          B-01
status:      holds, on generic bounds only (the campaign has not started)
best-attack: 0.4: none below generic; interpolation degree 7^60 = 2^168.4 > p.
             0.3 (not frozen): CICO-1 degree 2^38.46 < p; reduced-round CICO-1
             solved in 92 s on one core
measured:    2026-10-16 (campaign §3, results log 1–3)
day 0:       not started; it begins when 0.4's constants, vectors and instance
             table are published together (launch.md, decision 2026-10-16)
next:        day 90 after day 0 (campaign §5.4)
```

The fields work together:

- `if-false` turns the registry into a response plan;
- `eidos` ties an entry to the proof corpus;
- `consumers` makes the reverse check mechanical;
- the receipt carries what changes with time, so the specification changes only when a requirement does.

### 2.3 one source, generated views

- source: `soft3/specs/assumptions.md` has one entry per heading, and ids are stable. It is the only file a person edits.
- generated views: `soft3/proofs/Axioms.ei` and the public mirror `cyber/assumptions.md` (the registry with wiki-links). `release/assumptions.py` writes both. It uses only the Python standard library, plus the hemera CLI at the `release/phase1.toml` pin to compute particles. Each generated file carries a header with the specification revision it came from, and nobody edits one by hand.
- same statement: two statements are the same when the particles (hemera digests) of their canonical forms are equal. The canonical form is the statement text in Unicode NFC, with every run of whitespace replaced by one space and the ends trimmed. Every generated item carries `statement: <particle>`, and the gate recomputes it from the specification.
- observations: status, best attack and measurement dates live in `soft3/audit/assumptions/<date>.md`, one receipt per review, under the placement rule (`specs/README.md` § report placement). The first receipt is [[soft3/audit/assumptions/2026-10-30|the receipt of 2026-10-30]].
- measurements stay where they are made: `hemera/audit/campaign.md`, `simulation/audit/`, `zheng/specs/soundness.md`, `tru/audit/`, `cyber/audit/cryptography-*.md`. The registry and the receipt link to them and never copy them.
- proofs: `soft3/proofs/` is an eidos development. It holds `Axioms.ei` and one theory file per class-A theorem.
- which entries get an eidos axiom. §0's "an assumption is an axiom" applies to every assumption a proof can rest on. Entries that no proof rests on are measured, not axiomatized:
  - B and C: one axiom each, because these are what proofs consume (E2, E4).
  - A and A′: none. These are theorems or candidate theorems. Turning a tested invariant into an axiom would let a proof rest on an unproved claim about our own code.
  - D: never.
  - E: none until a theorem needs one, and no rung of the ladder does.
  - F, G, H: none. They are measured, not used by proofs.

### 2.4 the gate

`assumptions-gate` in `release/train_gates.py` is red when any of these holds:

1. the registry does not parse, an id is duplicated, or an id present in the previous candidate has disappeared;
2. a `consumers` target (a file and its § heading) does not exist;
3. regenerating `Axioms.ei` or the mirror does not reproduce the committed bytes, which includes any statement particle that differs;
4. the class-D scan finds a hit. The scan covers every consensus specification: `cybernode/specs/chain*.md` and `genesis.md`, `soft3/specs/`, `tru/specs/`, `zheng/specs/`, `hemera/specs/`, `bbg/specs/` and `lens/specs/`. A hit is either a whole-word, case-sensitive match of `ETH`, `SETH`, `Gap-ETH`, `NSETH`, `OVC`, `UGC`, `P ≠ NP` or `P != NP`, or a match of the pattern `unless … is false`. A line that names a D id as context is exempt;
5. a constant goes uncited: a row added since the previous candidate to a parameter table (`## 0. parameters` or its equivalent) of a consensus specification names neither a registry id nor a source;
6. eidos fails: `eidos check soft3/proofs` (a directory) fails, a theorem uses `sorry`, or `eidos axioms` for a theorem names an axiom outside `Axioms.ei` and the stdlib bootstrap set A-10.

An entry whose last measurement is older than 90 days does not turn the gate red. The gate records it as an info row in the candidate's receipt and opens a task for the sensor (§5). A candidate that changes nothing should not turn red only because time passed.

The gate has two prerequisites:

- eidos pinned in `release/phase1.toml`. That pin comes in a bump PR, not in this proposal;
- the eidos CLI work of E0 (§4.3).

## 3. the seed entries

This is the first registry, drawn from documents that exist today. Ids are provisional until `specs/assumptions.md` lands. The "measured by" column points to the evidence. Status and dates are in [[soft3/audit/assumptions/2026-10-30|the receipt]]. "chain.md" and "chain-economy" are `cybernode/specs/`.

| id | class | statement | consumers | measured by | if false |
|---|---|---|---|---|---|
| A-01 | A | Schwartz–Zippel: a nonzero polynomial of total degree d over a field F vanishes at a uniformly random point of Fⁿ with probability ≤ d/\|F\|; zheng draws its challenges from Fp3, \|F\| = p³ | zheng ledger rows "succinct — Spartan IOP over Fp3" and "zk — masked Spartan IOP" | theorem in the literature; in eidos at E2 | — |
| A-02 | A | sum-check over Fp3: a round of degree d + 1 errs with probability ≤ (d + 1)/p³, ≈ 2^-189 at d = 7. The IOP total is ε_IOP ≤ \|Λ\|·(log m·(d + 2) + 2(ℓ + 1) + t − 1)/p³, ≤ 2^-173.5 at m = 2^20. The masked IOP's total is (log m·(d + 2) + 2ℓ + t + 3)/p³ | zheng ledger rows "succinct — Spartan IOP over Fp3" and "zk — masked Spartan IOP" | theorem in the literature (ledger: proven); in eidos at E2 | — |
| A-03 | A′ | Σ native balances (owners, dormant owners, stakes, the focus pool, and the turnstile from rule set 2) = S0 + h·SUBSIDY at every height; every other token's sum is its genesis supply | chain.md §3.4; chain-economy §6.3 | node check after every block; conformance over 2,000 random blocks (cybernode `rs/src/chain/economy/vectors.rs:318–364`); simulator, every block | mint or burn by a bug; the check flags the block (`/status`, alerts) |
| A-04 | A′ | the settlement of epoch E is a function of G_{E−1} and G_E alone, and the epoch record under `state_root` is identical on every node | chain-economy §4.4, §5.6 | simulator: equal records on every node, bit-reproducible on macOS arm64 and Linux x86_64; tru determinism suite | divergence at settle(E): the divergent node halts (chain-economy §9) |
| A-05 | A′ | ASERT target arithmetic is exact in integers and bounded: no overflow at any height, and shifts saturate at the clamp | chain.md §1.5 | kernel tests (cybernode `rs/src/chain/daa.rs`); chain-conformance §4 rows | difficulty wraps: the chain halts or floods |
| A-06 | A′ | genesis is a bijection of the burial snapshot; Σ per token = supply | cybernode/specs/genesis.md; chain.md §4.2 | `cybernode genesis verify` (genesis.md still names `soft3 genesis verify`) | unowned or duplicated balances |
| A-07 | A′ | φ* after `PHI_STEPS` = 48 cold steps is within 10⁻⁷ in ℓ1 of the converged fixed point on the launch graphs | chain-economy §0, §4.4; tru/specs/epoch.md | measurement: tru audit/epoch-settlement-2026-10 §2.1, §6.3. The contraction bound κ itself is a spectral statement that no rung of §4.3 proves | payouts drift from the fixed point they are meant to price; determinism is unaffected, because the step count is fixed |
| A-08 | A′ | attribution pays only for new height: a relock earns at most the mark's decay, and an exact +1/−1 hedge earns zero | chain-economy §5.2, §5.3, §9 | kernel tests, red: soft3#234 findings 1 and 4 | recycling and hedging drain the pool |
| A-09 | A′ | the epoch record's `syntropy` and `delta_syntropy` fit `i64` in tru's raw fixed point | chain-economy §5.6 | simulator and kernel test, red: soft3#234 finding 3 | the record saturates at `i64::MAX`, so syntropy and its change are wrong in consensus |
| A-10 | A | eidos's stdlib bootstrap set, true in arithmetic but trusted without proof: `Nat.div` (an opaque constant), `Nat.min_le_right`, `Nat.div_le_self`, `Nat.div_le_div_right`, `Nat.mul_le_mul_right` and `Nat.mul_div_cancel_left`. None is kernel-checked, and the last three carry the placeholder type `Prop` | every eidos theorem that uses them | `eidos/rs/src/stdlib/mod.rs:114–158`, `rs/src/stdlib/le.rs` | a wrong lemma proves anything built on it; response: prove them (an E1 prerequisite) |
| B-01 | B | hemera 0.4 admits no collision and no preimage below 2^128 classical (§2.2) | chain.md §1.2, §3.3, §11.1; particle identity; the zheng hash row | hemera/audit/campaign §6 criterion 2 | identity forgery: fork to the next profile |
| B-02 | B | the 0.4 permutation passes the G3 structural criterion: there is no tier-A4 result (CICO-2 on half the partial budget at full R_F), and the per-layer margin rule holds (each outer full layer is at least 3× the full rounds a published attack crosses or skips, and the partial layer is at least 1.5× the longest published subspace trail) | zheng rows that treat the permutation as ideal (field-native transcript; query indices taken from single limbs); PoW | campaign §6 criterion 3, §3.1–§3.3. 0.3 failed this criterion: its CICO-1 degree is 2^38.46 < p, and a reduced-round CICO-1 (R_F 1 + 1, all 16 inverse partial rounds) was solved in 92 s on one core | Fiat–Shamir challenges and PoW lose their ideal-permutation argument; response: re-parameterize before genesis or fork after it (campaign §6) |
| B-03 | B | proof-of-work search has no shortcut below generic preimage search at the target | chain.md §1.2, §8 | campaign (every header byte crosses at least two permutations, §3.2) | hashrate measurements lie; ASERT and finality bounds shift |
| C-01 | C | the random-oracle model for Fiat–Shamir over hemera | every ledger row marked "hemera-as-RO conjectured" (succinct, zk, MITH, composition over N steps) | named idealization | proofs forgeable; rule set 2 waits (G5) |
| C-02 | C | the quantum random-oracle model, with a lifetime loss budget | privacy-frozen §5 G5 | open research | the pool's post-quantum claim lapses |
| C-03 | C | mutual correlated agreement for Reed–Solomon codes at the shipped rates | the ledger's WHIR rows | the ledger marks it proven up to the Johnson bound (BCGM ePrint 2025/2051 Lemma 9.3), where the shipped parameters sit; it becomes a conjecture only beyond Johnson | proof size or soundness margin shrinks |
| C-04 | C | relation compiler: the compiled CCS is satisfiable with prefix (1, io, cycles) exactly when nox reduces the program to that output at that cost | every zheng profile (ledger row "relation compiler") | ledger: "conjectured (tested, not proven)": differential tests, mutated witnesses, malleability analysis. This is a statement about our own compiler, so it leaves C once proved | a proof certifies a computation nox did not perform |
| C-05 | C | program binding by digest (profiles 4–6): a proof verifies against a different program only through a hemera collision of the two programs' token encodings | ledger row "program binding by digest" | ledger: "conjectured (hash row)", i.e. it reduces to B-01 | a proof of one program passes as another's |
| C-06 | C | recursive Fiat–Shamir: a recursively composed Fiat–Shamir argument whose random oracle is evaluated inside the relation is knowledge-sound | ledger rows "IVC composition" and "wrap composition" (profiles 5–6); the block proof (privacy-frozen §1.8); the proven present | named heuristic, with no proof in the ROM; the per-step reductions are proven | folded and wrapped proofs become forgeable |
| D-00 | D | none is load-bearing. Context rows: ETH, SETH, Gap-ETH, NSETH and #ETH (claimed refuted by a preprint dated 2026-10-17, revised 10-25, under review); 3SUM, APSP and Exact Triangle (refuted, arXiv 2026-10-05); UGC (proved, `openai/math` 2026-10-06) | — | `cyber/audit/cryptography-*`; the receipt | nothing in the stack |
| E-01 | E | ECDLP on secp256k1 | chain.md §2.7: every phase-1 signature (kinds 1, 2, 3, and chain-economy kinds 6, 7); §5.3: `checkpoint_key`; §2.8: legacy claims (mudra bridge) | literature; chain.md §8 makes no post-quantum claim | transfers, stakes, claims and checkpoints are forgeable for every exposed key. Sunsets: the checkpoint key stops at 525,600 blocks (chain.md §5.3); transaction authorization has no sunset (privacy-frozen §1.9, §6 item 8). This breaks the class-E rule, so the entry is red until the owner decides (§7) |
| E-02 | E | ML-KEM-1024 (FIPS 203, module lattices) for note delivery | privacy-frozen §1.2; wallets from rule set 2 | literature; not in use before rule set 2 | past notes are disclosed and nothing is forged; wallets version the scheme without a fork |
| F-01 | F | honest hashrate > ½ over any 120-block window | chain.md §5.1–§5.2, §8 | simulator `reorg-140`, `reorg-140-checkpoints` | reorgs up to 119 blocks; while H-01 is open, deeper released rewrites capture late joiners |
| F-02 | F | eclipses and partitions heal within the finality window (1 h) | chain.md §5.2; chain-wire §5 | simulator `eclipse`, `partition-long` (receipt, for the owner, item 2) | stranded nodes need an operator resync; peer-diversity work |
| F-03 | F | honest clocks agree within `MAX_FUTURE_DRIFT` (120 s) | chain.md §1.4, §1.5; cybernode `rs/src/chain/daa.rs` | simulator `clock-skew` | timestamp play moves difficulty by at most 1.149× per block |
| F-04 | F | the settlement engine finishes without panic on every state within D = 120 blocks | chain-economy §2.5, §5; the tru epoch engine | simulator `settle-slow`, `settle-panic` (receipt, for the owner, items 3 and 4) | the chain halts at settle(E); no deterministic fallback exists |
| F-05 | F | hashrate does not fall more than 100× inside one half-life (600 s) | chain.md §1.5, §8 | simulator `asert-100x` (receipt, for the owner, item 5): first block 6,929 s after the step (exponential, mean 3,000 s); back to 30 s ± 20 % at 8,613 s | liveness is lost for hours |
| G-01 | G | ρ = 1 and karma = 1 at launch | chain-economy §0, §3.7, §5.4 | choice | novelty is over- or under-rewarded |
| G-02 | G | syntropy J = D_KL(φ ‖ u) is the objective; concentration is paid as knowledge | chain-economy §4.4; §9 (self-linking cycles) | simulator `economy` (receipt, for the owner, item 8) | rank collapses onto a few particles |
| G-03 | G | per settlement a position earns at most 1 % of the stake it added (`R_MAX_PPM`); the budget is a quarter of the pool (`POOL_RELEASE`); stake locks for 7 days (`MIN_LOCK`) | chain-economy §0, §3.4, §5.5 | kernel tests; simulator `economy` | recyclers drain the pool |
| G-04 | G | the pool share is `focus_bps = min(5,000, isqrt(θ·10^8))`, i.e. min(½, √θ) of the subsidy | chain-economy §6.1; owner question §12 item 1 | choice | miners or stakers are starved |
| G-05 | G | genesis weights are passive and constant, and earn nothing | chain-economy §4.1, §8; owner question §12 item 2 | genesis | migrated holders farm without staking |
| H-01 | H | a checkpoint key exists, signs during the first months, and sunsets at 525,600 blocks | chain.md §5.3; genesis.md | simulator receipt, for the owner, item 1: both genesis files carry an all-zero key and no signer exists | late joiners follow deep rewrites |
| H-02 | H | only the owner promotes a candidate, publishes or tags | `~/cyber/CLAUDE.md` § release train; `cyberia/dev.md` § release train | doctrine | a bad release ships |
| H-03 | H | the hemera bounty is funded and its inbox is read | campaign §4; launch.md, decision 2026-10-16 | owner action pending | attacks go unreported |

Five entries are red today, and each is already tracked elsewhere:

- A-08 and A-09: soft3#234, findings 1, 4 and 3;
- E-01: privacy-frozen §6 item 8;
- F-04 and H-01: the simulator receipt, for the owner, items 4 and 1.

The registry creates no new work for them; it puts that work in one place.

Seeding the registry also turned up three contradictions between documents. Each is a follow-up for its owner, not an edit here:

1. privacy-frozen §1.1 says "the only conjectures are" the random-oracle model and mutual correlated agreement. The zheng ledger disagrees on both counts: it marks the relation compiler, program binding and the IVC/wrap compositions as conjectured (C-04 to C-06), and it marks MCA as proven in the regime zheng ships (C-03).
2. chain.md §1.5 and §8 say a 100× hashrate drop recovers in about 66 minutes. The simulator measured 8,613 s (F-05) to get back to 30 s ± 20 %.
3. the zheng ledger still computes every figure on hemera 0.3.1 (zheng#69 / #73), while B-01 and B-02 are about 0.4.

## 4. eidos replaces lean

### 4.1 the stance

eidos is the stack's proof assistant. A statement about the stack has one of three statuses:

- checked: `eidos check` accepts it with no `sorry`, and `eidos axioms` lists only axioms from `Axioms.ei` and the bootstrap set A-10. The registry records this as `checked (rust kernel, eidos <rev>)`. It is the only status that can be reached before E3, and only once E0 is done.
- certified: from E3 on, the type check runs on nox, zheng certifies that run, and the certificate is a particle.
- anything else is an assumption (a registry entry) or a claim.

Today `eidos check` cannot be the record. When a proof ends in `sorry`, `check_theorem` prints a warning to stderr and registers the statement as if it were proved. The CLI then prints "proved" and exits 0 (`rs/src/surface/check.rs:206–213`, `cli/src/main.rs`). E0 makes `sorry` an error in the gate and lists it in `eidos axioms`.

Lean proofs enter as evidence in `measured-by`, for example an attacker's certificate or an external team's bound. They are then ported, and the ported eidos proof becomes the record. All of the stack's own proofs, proof files and CI proof steps are in eidos.

A theorem proves a statement about an eidos model of the code, not about the Rust itself. Conformance links the two:

- the model is run with `eidos eval` over vectors the Rust already passes. For A-03 that is the 2,000-block corpus in cybernode `rs/src/chain/economy/vectors.rs:318–364`;
- every A entry whose consumer is code states this gap;
- running the model at the kernel's scale needs E1's binary integers, because a unary `Nat` cannot hold S0 ≈ 1.1 × 10^18.

### 4.2 what lean has that the registry needs, and the eidos answer

| need | lean today | eidos today (`c1e8993`) | build |
|---|---|---|---|
| a checked axiom list | `axiom` + `#print axioms` | `axiom` exists, but there is no `eidos axioms`. `sorry` is accepted as a silent axiom (§4.1). Six stdlib constants are trusted without a kernel check (A-10). The nox corpus has its own axioms (`nox/proofs/T3.ei` `sort_permutation_invariant`, `Model.ei`) | E0 |
| checking a development | `lake build` | `eidos check` takes one file (`cli/src/main.rs:146`). An `import` needs a base directory, so it fails from the CLI, which calls `check_file` without one (`rs/src/surface/check.rs:160–163`) | E0: `eidos check <dir>` through `check_path` |
| integer arithmetic | Mathlib `Nat`, `Int`, `omega` | what exists: a unary `Nat` (`nat_lit` builds n successors, `rs/src/stdlib/nat.rs:12`), `Bool`, `List`, `Le`. `Nat.div` is an opaque constant. Missing: `Int`, `Lt`, isqrt, u64/u128 wrap, the `add_comm`/`add_assoc`/`mul` lemmas, `List` sum and fold lemmas, finite maps. `omega` closes only goals that normalize to `Eq` by reflexivity, or `Le` between literals; otherwise `try_arith_laws` returns `None` (`rs/src/tactic_ext/arith.rs`, `rs/src/surface/omega.rs`) | E1 |
| polynomials over a finite field | Mathlib `Polynomial`, `ZMod`, `Finset.sum` | none | E2: `Goldilocks.ei` (field axioms with p concrete), `Fp3.ei`, `Poly.ei` (degree, evaluation, roots ≤ degree), `Multilinear.ei`; Schwartz–Zippel as a counting statement |
| probability for soundness error | Mathlib `MeasureTheory` | none, and not wanted | E2: soundness as a counting bound over the challenge space ("at most d·\|S\|^{k−1} bad points of \|S\|^k"); the probabilistic reading is the class-C note |
| random-oracle arguments | game-based on paper, rarely in Lean | none | E4 |
| a re-executable kernel | the Lean kernel, trusted by convention | a Rust CIC kernel (`rs/src/kernel.rs`, 620 lines). The nox kernel is a draft spec (`specs/kernel.md`). zheng certification is specified (`specs/certificate.md`), but the certificate code is an FNV-1a placeholder (`rs/src/certificate.rs:142`), and `eidos verify` exits 1 | E3, per `specs/certificate.md`: `axon = hemera(KERNEL_ID ‖ claim ‖ proof)` under domain `0xA1`; the artifact carries the zheng proof in its `stark` field; the cyberlink is `axon → OK` |
| a library and a community | Mathlib, Zulip | 27 theorems over nox and prysm, resting on 107 declared axioms | not replicated; the registry corpus is small and stack-specific by construction |
| importing outside proofs | — | a `port-from-lean` guide is planned (eidos `roadmap/README.md`) | E5 |
| machine-found proofs | agents write Lean | agents can write eidos (the nox and prysm corpus was written that way) | the capability that fells conjectures writes eidos too; the corpus is the training signal |

### 4.3 the ladder

| milestone | delivers | acceptance | when |
|---|---|---|---|
| E0 · the axiom file and the CLI | in eidos: `eidos check <dir>` with imports; `eidos axioms <file> [theorem]`, listing every axiom a theorem depends on, including `sorry` and the A-10 set; `sorry` as an error under `--deny-sorry`, which the gate uses. In soft3: `release/assumptions.py`, which generates `Axioms.ei` (classes B and C) and the mirror; the gate of §2.4; eidos pinned in `release/phase1.toml` (a bump PR) | every class-B and class-C id has exactly one generated axiom, every generated axiom has exactly one id, and their statement particles are equal; the gate is green | after genesis. It needs eidos CLI work and a new pin, and no genesis byte depends on it |
| E1 · conservation, symbolically; then the integer library | first, `Conservation.ei` (A-03). Over abstract `S0`, `SUBSIDY` and h, with a block modelled as a finite list of moves (debit, credit, mint), it proves Σ after = Σ before + `SUBSIDY` per block, and so Σ_h = S0 + h·SUBSIDY by induction. Its prerequisites are: `add_comm`, `add_assoc` and the `mul` lemmas proved instead of trusted; `List` sum and fold with their lemmas; a finite-map or association-list model of balances; an `omega` that decides linear `Nat` goals; the A-10 lemmas proved. Second, the integer library: binary `Nat` and `Int` with u64/u128 wrap semantics, `Lt`, and floor division and isqrt as definitions with lemmas. Then `Asert.ei` (A-05), `Attribution.ei` (A-08, after the economy fix) and `Settlement.ei` (A-04 as a function of its inputs) | the theorems check with no `sorry` and no axiom beyond A-10, and none at all once A-10 is proved; each A′ entry covered moves to A and names its conformance link (§4.1) | the symbolic lemma after genesis; the integer theorems at the first fork window |
| E2 · the field and the sum-check | `Goldilocks.ei`, `Fp3.ei`, `Poly.ei`, `Multilinear.ei`, `Sumcheck.ei` (A-01, A-02) | the ledger rows "succinct — Spartan IOP over Fp3" and "zk — masked Spartan IOP" each cite a theorem id for their bound | first fork window |
| E3 · the zheng bridge | the nox kernel of `specs/kernel.md`; the type check runs on nox and zheng certifies it; the certificate of `specs/certificate.md` replaces the FNV-1a placeholder; `eidos verify` works | a registry theorem's certificate is a particle | with the proven present |
| E4 · oracle interfaces | random-oracle and QROM interfaces as axioms with explicit query budgets; Fiat–Shamir composition (C-01, C-02) | the transcript spec's security row cites a theorem with its budget | after E2 |
| E5 · the port | a statement-level Lean reader that renders a Lean theorem's statement in eidos surface syntax; proofs are rewritten by hand or by an agent; the hemera bounty's terms name eidos as canonical and Lean as accepted evidence (an owner decision, §7) | the first ported attacker certificate | when the first one arrives |

Three things stay off the ladder:

- A-07's contraction bound, a spectral statement over the reals;
- C-04's compiler correctness, a candidate after E2, to be sized then;
- C-06, which has no known proof.

### 4.4 what this costs

- E0: CLI work in eidos (directory check, `eidos axioms`, `sorry` as an error), plus a generator and a gate. One to two sessions, and a bump PR for the pin.
- E1, first half: the symbolic lemma is small once its prerequisites exist. The prerequisites themselves (proved `Nat` lemmas, `List` sums, a model of balances, a real decision procedure for linear arithmetic) take two to four sessions.
- E1, second half: the integer library behind the kernel-faithful theorems is larger. It is sized once the first half lands.
- E2: this is the milestone that makes eidos a proof assistant for the stack rather than for toy programs. A finite-field and polynomial library is weeks of work, and the sum-check proof is the first theorem that would otherwise have been written in Lean.
- E3: already on eidos's roadmap, and blocked on the same zheng work the proven present needs.

Each milestone proves something the chain already uses.

### 4.5 follow-ups in eidos

- the README's `axon = H(check_kernel, H(π, P))` has drifted from `specs/certificate.md`, which uses `hemera(KERNEL_ID ‖ claim ‖ proof)` under domain `0xA1`. The README should follow the spec;
- `roadmap/replace-lean.md` mirrors the first version of this ladder. It needs the revised E0, E1 and E3 above.

## 5. watch and measurement

- capability watch: the sensor of the crypto domain runs it (`cyberia/dev.md` § the roles: the sensor perceives and fires, and holds the critic and tester skills). It scans each of these sources against the `watch` fields:
  - every release of `openai/math`;
  - Anthropic's and DeepMind's research artifacts;
  - a weekly cut of ePrint and of arXiv cs.CC and cs.CR.

  Within a day of a release, it writes one row per affected entry into `soft3/audit/assumptions/<date>.md`. The literature narrative stays in `cyber/audit/cryptography-<date>.md`. A status change on any entry becomes a row in `cyber/launch.md` and an owner decision.
- measurement: every class-B, F and G entry names a script or a suite. The models are hemera's campaign (`p2rounds.py`, `cico1.py`) for B, the simulator's verdict tables for F and G, and the conformance vectors for A′. A measurement older than 90 days becomes an info row in the candidate's receipt and a task for the sensor (§2.4); it never turns the gate red.
- the response plan: an entry carries its `if-false` from the day it is accepted.

## 6. launch

The freeze is 2026-11-02 00:00 UTC (`cyber/launch.md`, decision of 2026-10-17). From then until genesis, only security fixes and bug fixes with a failing test merge. H-01 is the only item in this proposal that changes a genesis file, and that change is the owner's call.

- before the freeze:
  - H-01, if the owner sets a key. Both genesis particles then change and must be re-blessed (soft3#398, the director's comment; soft3#399).
  - `specs/assumptions.md` and the public mirror, if the proposal is accepted in time. Both are text only: no code, no genesis byte.
- after genesis: E0 (the eidos CLI work, the generator, the gate, and the eidos pin as a bump PR) and E1's symbolic lemma. The receipts continue in `audit/assumptions/`.
- if H-01 misses the freeze: the chains launch with checkpoints off. After that, a key can arrive only through a scheduled fork, because chain.md §5.3 says "the key cannot be rotated without a release". Until then, late joiners stay exposed to deep rewrites (F-01).

## 7. decisions for the owner

1. H-01, the checkpoint key. Deadline: before 2026-11-02 00:00 UTC, because a key changes both genesis files. The choice is a key with its sunset, or an explicit decision to launch without checkpoints and accept F-01's measured consequence. Default: a foundation key in both genesis files, with the sunset chain.md §5.3 already fixes (525,600 blocks, about six months). Links: [the simulator receipt, for the owner, item 1](https://github.com/cyberia-to/simulation/blob/main/audit/network-2026-10.md#for-the-owner--properties-measured-not-bugs); [soft3#398, the director's comment](https://github.com/cyberia-to/soft3/pull/398#issuecomment-6441630702); [soft3#399](https://github.com/cyberia-to/soft3/pull/399).
2. E-01, the sunset of classical authorization. The class-E rule requires a sunset height. chain.md writes one for the checkpoint key and none for transaction signatures, and privacy-frozen leaves that height to the owner, to be set before rule set 2 (§1.9, §6 item 8). Default: from the activation of rule set 2, classical signatures authorize only entry into the pool (the narrowing of privacy-frozen §1.9). Link: no link yet; the open item is privacy-frozen §6 item 8.
3. Class G and H entries in the same registry. Economic parameters and trust in keys are choices, not science. The proposal lists them anyway, because `if-false` is exactly the plan the stack needs for them. Default: one list, eight classes. Link: no link yet (this proposal, soft3#398).
4. E2 before or after the first fork. The sum-check proof in eidos is the first theorem that replaces Lean in substance, and genesis does not need it. Default: after launch, in the first fork window. Link: no link yet.
5. E5, the bounty's canonical form. The hemera bounty's terms would name eidos as the canonical form of a submitted proof and Lean as accepted evidence. Opening the bounty is the owner's call (launch.md, decision 2026-10-16). Default: yes, in the terms the bounty opens with. Link: no link yet; the terms are `hemera/audit/campaign.md` §4.2.

## sources

- OpenAI, "The Exponential Time Hypothesis is False", `openai/math`, `preprints/The-Exponential-Time-Hypothesis-is-False-October-25-2026/paper.pdf`. The paper is dated "17 October 2026, revised 25 October 2026"; it first appeared in the repository in commit `42907c9`, 2026-10-26. Its §1 lists the hypotheses it refutes. The Lean scope is `lean/docs/378.md` in the same repository. All read 2026-10-30.
- Alman, Vassilevska Williams, "Truly Subquadratic 3SUM and Truly Subcubic APSP via Triangles in Sparse Lopsided Graphs", arXiv:2610.06783, v1 2026-10-05. In the PDF read 2026-10-30, p. 1 and § Acknowledgments and Methodology attribute the algorithm to Claude.
- the ETH verification team, from news coverage (reported, not verified by us): xenospectrum.com/en/openai-exponential-time-hypothesis-false-claim; aiweekly.co/alerts/openai-hands-eth-kakeya-yang-mills-proofs-to-reviewers (2026-10-26).
- [[zheng]] `specs/soundness.md` on `origin/master` (`ecf2491`); [[tru]] `audit/epoch-settlement-2026-10.md` on `origin/master`; [[cybernode/specs/chain|chain]] and [[cybernode/specs/chain-economy|chain-economy]] (moved from soft3 on 2026-10-27, section numbers unchanged).
- [[cyber/audit/cryptography-2026-10-08|the cryptography audit]] §2 (what the stack relies on at genesis), §6 (the 10-06 release read against the primitives)
- [[hemera/audit/campaign|hemera campaign]]; [[simulation/audit/network-2026-10|the simulator receipt]]; [[soft3/proposals/privacy-frozen|privacy, frozen]] G1–G7; [[eidos/specs/certificate|eidos certificate]]
- [[eidos]] at `c1e8993`: `rs/src/kernel.rs`, `rs/src/certificate.rs`, `rs/src/stdlib/{nat,le,mod}.rs`, `rs/src/surface/{check,omega}.rs`, `rs/src/tactic_ext/arith.rs`, `cli/src/main.rs`, `specs/kernel.md`, `specs/certificate.md`; `soft3/status.md` (eidos: blocked)
- [[soft3/audit/assumptions/2026-10-30|the receipt of 2026-10-30]]: the observed status of every seed entry, with revisions
