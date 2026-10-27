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

**status:** proposed · **date:** 2026-10-27 · **owner's call:** "a clear registry of scientific assumptions; Lean is foreign, eidos is native — whatever eidos lacks to replace Lean must be built."

## 0. the decision

The stack keeps one list of everything it assumes and cannot prove. That list is the axiom file of the stack's own proof corpus in [[eidos]]: an assumption is an axiom, a consequence is a theorem, and nothing load-bearing exists outside the two. Every assumption is classed, every class has a rule about what it may carry, every entry says what breaks if it is false, how it is measured, and what to watch. Proofs about the stack are written in eidos; [[lean]] is accepted only as evidence from outside, never as the stack's own record. Where eidos cannot yet carry a proof the stack needs, the missing piece is built in eidos; the stack does not borrow a foreign kernel.

## 1. why now

In the week of 2026-10-17 to 10-26 the Exponential Time Hypothesis, its strong, gap, nondeterministic and counting forms fell to one preprint (OpenAI, `openai/math`, Lean-checked at a weaker bound, under review by Impagliazzo, Paturi and Williams); the 3SUM and APSP hypotheses fell three weeks earlier (Alman and Vassilevska Williams, arXiv:2610.06783, algorithm found by Claude); the Unique Games Conjecture was proved in the release of 10-06 ([[cyber/audit/cryptography-2026-10-08|the cryptography audit]] §6). The machines that do this are the same machines that will be pointed at [[hemera]], at the economy and at the network.

The stack was not touched, and the reason is instructive: no consensus rule rests on an asymptotic conjecture. hemera's security is a measured attack on a fixed permutation ([[hemera/audit/campaign|the campaign]]); [[zheng]]'s soundness is Schwartz–Zippel, code proximity and a random oracle; the economy is integer arithmetic on bbg; the network is Nakamoto with measured bounds ([[simulation/audit/network-2026-10|the simulator receipt]]). That was true by habit. This proposal makes it true by rule, writes the list down in one place, and gives the list a proof checker of its own.

Lean is the wrong checker for this list. Its kernel, its library and its community are outside the stack; a Lean certificate proves a statement to a Lean user, and the stack cannot re-execute it, cannot store it as a particle, cannot certify it with a zheng proof, and cannot verify it inside a block. eidos is a CIC kernel whose type checker is a [[nox]] program, whose execution zheng certifies, and whose certificates are cyberlinks ([[eidos/specs/certificate|certificate]]). A theorem about the stack belongs in the graph the stack maintains, checked by the stack's own machine. Today eidos is small (a kernel, an elaborator, a tactic engine, 139 tests, 27 theorems over nox and prysm, no polynomial library, no zheng bridge, a stub certificate). §4 is the list of what it must grow to replace Lean for the registry's purposes, in order, with acceptance criteria.

## 2. the registry

### 2.1 classes and the load-bearing rule

| class | what it is | example | may carry consensus |
|---|---|---|---|
| A · information-theoretic | theorems that hold without any computational hypothesis | Schwartz–Zippel; sum-check error ≤ kd/p; conservation of tokens; determinism of settlement | yes, and each is proved in eidos |
| B · concrete cryptographic | the strength of one fixed primitive, equal to its best measured attack | hemera 0.4: generic collision 2^128, interpolation degree 7^60 > p | yes, with a measured attack and a standing campaign |
| C · oracle models | idealizations every hash-based proof system shares | random oracle for Fiat–Shamir, its quantum form, mutual correlated agreement for Reed–Solomon codes at the shipped rates | yes, named as such, never silently |
| D · asymptotic complexity conjectures | statements about all algorithms on all input sizes | ETH, SETH, Gap-ETH, OVC, UGC, P ≠ NP | never; context only |
| E · classical curve assumptions | discrete logarithm on a curve | secp256k1 for legacy claims | at the edge only, with a sunset |
| F · network and physical | the environment of a Nakamoto chain | honest-majority hashrate; partitions heal within Δ; clock skew; settlement liveness | yes, with bounds measured by the simulator |
| G · economic | parameters and objectives that are choices | ρ = 1, karma = 1, cap 1 %, lock 7 days, syntropy as the objective | yes, with measured attack payoffs |
| H · social | trust placed in a key or a person, with a sunset | the checkpoint key; owner-only promotion; the bounty | yes, with the sunset written |

The rule: a consensus rule may rest on A, B, C, F, G and H; never on D; on E only outside the pool and only with a sunset date. A specification that introduces a load-bearing assumption absent from the registry is red.

### 2.2 one entry

```
id:          H-02
class:       B
statement:   hemera 0.4 (t = 16, RF 12, RP 48, x⁷ every round) admits no preimage
             below 2^128 and no collision below 2^128 generic
consumers:   chain.md §3 (PoW), §4 (state root); bbg/specs/state.md; particle
             identity; zheng transcript; the pool digests (privacy-frozen §1.3)
parameter:   128-bit classical; 85-bit quantum collision at the 32-byte particle,
             128-bit at the 48-byte pool digest
best-attack: interpolation closed (7^60 > p); CICO-2 Bézout 2^77 against 0.3,
             generic against 0.4; campaign day 0 = 2026-10-16
measured-by: hemera/audit/campaign.md, p2rounds.py, partial_subspace.py
watch:       eprint "Poseidon2" "Goldilocks" "CICO" "Gröbner" "interpolation";
             openai/math families over F_p polynomial systems; the bounty inbox
if-false:    particle collision forges graph identity and state roots;
             response: scheduled fork to the next profile, formats unchanged
             (48-byte digests reserved), every hemera-derived value re-derived
eidos:       axiom hemera_0_4_preimage_128 : Axioms.ei — consumed by
             Transcript.ei, Pool.ei
status:      holds · reviewed 2026-10-27 · next review 2027-01-14 (campaign)
```

`if-false` turns the registry into a response plan; `eidos` ties it to the proof corpus; `consumers` makes the reverse check mechanical.

### 2.3 placement

- contract: `soft3/specs/assumptions.md`, the registry, one entry per heading, id-stable;
- proofs: `soft3/proofs/` as an eidos development: `Axioms.ei` (one axiom per entry of class B, C, E, F, G, H), one theory file per class-A theorem, checked by `eidos check` in CI;
- measurements stay where they are made: `hemera/audit/campaign.md`, `simulation/audit/`, `zheng/specs/soundness.md`, `cyber/audit/cryptography-*.md`; the registry links, never copies;
- public page: `cyber/assumptions.md`, the registry mirrored with wiki-links;
- the train: `assumptions-gate` in `release/train_gates.py` — the registry parses, every `consumers` target exists, every `eidos` axiom exists in `Axioms.ei` with the same statement, no class-D id appears in a consensus specification, no entry is unreviewed for more than 90 days, `eidos check soft3/proofs` is green.

## 3. the seed entries

The first registry, drawn from documents that exist today. Ids are provisional until `specs/assumptions.md` lands.

| id | class | statement | consumers | measured by | if false |
|---|---|---|---|---|---|
| A-01 | A | Schwartz–Zippel: a nonzero polynomial of degree d over F_p vanishes at a random point with probability ≤ d/p | zheng sum-check, superspartan | theorem | — |
| A-02 | A | sum-check soundness error ≤ d·log m / p per round, recursion ≤ k·ε | zheng/specs/superspartan, recursion | theorem (to prove in eidos, E2) | — |
| A-03 | A | native supply + stakes + pool = S0 + h·SUBSIDY at every height | chain.md §9, chain-economy §7 | conformance; 2,000-block random test | mint or burn by bug: halt rule |
| A-04 | A | the epoch record is a function of the two boundary block ids and the state alone | chain-economy §5 | 3-node determinism, simulator gate | divergence at settlement: finality broken |
| A-05 | A | ASERT target arithmetic is exact in integers and bounded (no overflow at any height) | chain.md §6 | kernel tests | difficulty wraps: chain halts or floods |
| A-06 | A | genesis is a bijection of the burial snapshot; Σ per token = supply | genesis.md | `soft3 genesis verify` | unowned or duplicated balances |
| A-07 | A | φ* at 48 cold steps is within 1e-7 ℓ1 of the fixed point; the contraction is ≤ 0.775 per step | tru/specs/epoch, chain-economy §4 | tru/audit/epoch-settlement-2026-10 | payouts not reproducible across nodes |
| A-08 | A | attribution never exceeds the new-height share × cap; an exact hedge earns zero | chain-economy §5 | kernel tests (open: soft3#234 is red) | recycling and hedging drain the pool |
| B-01 | B | hemera 0.4 collision ≥ 2^128 generic, preimage ≥ 2^128 | everything | hemera/audit/campaign | identity forgery: fork to the next profile |
| B-02 | B | hemera 0.4 has no distinguisher from a random permutation below 2^128 | zheng transcript, PoW | campaign (0.3 failed this at 2^38.46) | Fiat–Shamir and PoW weaken |
| B-03 | B | PoW grinding has no shortcut below the generic preimage search at the target | chain.md §3 | campaign | hashrate measurements lie |
| C-01 | C | the random-oracle model for Fiat–Shamir over hemera | zheng/specs/transcript | named idealization | proofs forgeable: rule set 2 waits (G5) |
| C-02 | C | the quantum random-oracle model, lifetime loss budget | privacy-frozen G5 | open research | the pool's post-quantum claim lapses |
| C-03 | C | mutual correlated agreement for Reed–Solomon at the shipped rates | zheng/specs/soundness, WHIR | named idealization, literature | proof size or soundness margin shrinks |
| D-00 | D | none load-bearing. context rows: ETH family (fallen 2026-10-26, unconfirmed), 3SUM/APSP (fallen 2026-10-05), UGC (proved 2026-10-06) | — | cyber/audit/cryptography-* | nothing in the stack |
| E-01 | E | ECDLP on secp256k1 for legacy claims | chain.md §2.7, mudra bridge | literature | claims forged: narrow the edge; sunset per privacy-frozen §6.8 |
| F-01 | F | honest hashrate > ½ over any 120-block window | chain.md §7 | simulator: 60 % attacker rewrites 140 without checkpoints | reorgs: checkpoints (H-01) carry the first months |
| F-02 | F | partitions and eclipses heal within one hour | net wire, sync | simulator nightly (expected red beyond 1 h) | stranded nodes: operator resync; peer diversity work |
| F-03 | F | clock skew between honest nodes ≤ the ASERT tolerance | daa.rs | simulator | timestamp games move difficulty |
| F-04 | F | the settlement engine terminates on every state | chain-economy §5, tru epoch | simulator (a panic halts the chain: open) | chain halts: deterministic fallback required |
| F-05 | F | hashrate does not collapse more than 100× inside one half-life | ASERT | simulator (50-minute blocks after 100×) | liveness lost for hours |
| G-01 | G | ρ = 1 and karma = 1 at launch | chain-economy §0 | choice | over- or under-rewarding novelty |
| G-02 | G | syntropy is the objective; concentration is a known property | chain-economy §12 | red team (to measure) | rank collapses to a few particles |
| G-03 | G | cap 1 % per position on new height, lock 7 days | chain-economy §5 | kernel tests, simulator | recycler drain |
| G-04 | G | pool share min(½, √θ) | chain-economy §7 | choice | miners or stakers starved |
| G-05 | G | genesis weights are passive and earn nothing | chain-economy §4 | genesis | migrated holders farm without staking |
| H-01 | H | a checkpoint key exists, signs the first months, and sunsets at a published height | chain.md §7 | **open: both genesis files carry a zero key** | late joiners follow deep rewrites |
| H-02 | H | only the owner promotes a candidate, publishes, tags | release train | doctrine | a bad release ships |
| H-03 | H | the hemera bounty is funded and its inbox read | campaign | owner action pending | attacks go unreported |

Three entries are red today and are already tracked elsewhere: A-08 (soft3#234), F-04 and H-01 (`simulation/audit/network-2026-10.md`, decisions 1 and 4). The registry does not create work; it makes the work visible in one place.

## 4. eidos replaces lean

### 4.1 the stance

eidos is the stack's proof assistant. A statement about the stack is "proved" when `eidos check` accepts it and its certificate is a particle; until then it is an assumption or a claim. Lean proofs are accepted as evidence in `measured-by` (an attacker's certificate, an external team's bound) and ported; the ported eidos proof is the record. The stack adds no Lean dependency, no Lean CI step, no Lean file.

### 4.2 what lean has that the registry needs, and the eidos answer

| need | lean today | eidos today | build |
|---|---|---|---|
| a checked axiom list | `axiom` + `#print axioms` | `axiom` keyword exists; no corpus-level listing | E0: `Axioms.ei` as the registry; `eidos axioms <file>` prints every axiom a theorem depends on |
| integer and fixed-point arithmetic | Mathlib `Int`, `Nat`, `omega` | `Nat`, `Bool`, `List`, `le`, an omega-style tactic | E1: `Int` with bounded `u64`/`u128` semantics as used by the kernel; lemmas for floor division, isqrt, saturation |
| polynomials over a finite field | Mathlib `Polynomial`, `ZMod`, `Finset.sum` | none | E2: `Goldilocks.ei` (field axioms with p concrete), `Poly.ei` (degree, evaluation, roots ≤ degree), `Multilinear.ei`; Schwartz–Zippel as a counting statement without probability |
| probability for soundness error | Mathlib `MeasureTheory` | none, and not wanted | E2: soundness as a counting bound over the challenge space ("at most d·|S|^{k−1} bad points of |S|^k"); the probabilistic reading is the registry's class-C note, not a theorem |
| random-oracle arguments | game-based pen-and-paper, rarely in Lean | none | E4: the oracle as an axiomatized interface with an explicit query budget; composition theorems take the budget as a parameter |
| a trusted, re-executable kernel | the Lean kernel, trusted by convention | a CIC kernel as nox patterns, re-executable by design; zheng certification designed, not built | E3: the zheng bridge; certificate = `H(check_kernel, H(π, P))` as a cyberlink, replacing the FNV stub |
| a library and a community | Mathlib, Zulip | 27 theorems | not replicated; the registry corpus is small and stack-specific by construction |
| importing outside proofs | — | `port-from-lean` guide planned | E5: a statement-level Lean reader that renders a Lean theorem's statement in eidos surface syntax for porting; proofs are rewritten by hand or by an agent |
| machine-found proofs | agents write Lean | agents can write eidos (nox/prysm corpus was) | the capability that fells conjectures writes eidos too; the corpus is the training signal |

### 4.3 the ladder

| milestone | delivers | acceptance | when |
|---|---|---|---|
| E0 · the axiom file | `soft3/proofs/Axioms.ei`, one axiom per registry entry of class B, C, E, F, G, H, statements mirrored verbatim from `specs/assumptions.md`; `eidos axioms` lists dependencies | every registry id has one axiom and vice versa; `assumptions-gate` green | before 2026-11-02 |
| E1 · integer theorems of the economy | `Conservation.ei` (A-03), `Attribution.ei` (A-08 bounds), `Asert.ei` (A-05), `Settlement.ei` (A-04 as a function of its inputs) | the theorems check; their statements quote the kernel's constants; the kernel tests cite the theorem ids | before 2026-11-02 for A-03 and A-05; A-08 with the economy fix |
| E2 · the field and the sum-check | `Goldilocks.ei`, `Poly.ei`, `Multilinear.ei`, `Sumcheck.ei` (A-01, A-02) | the zheng soundness ledger's rows become theorem ids; one row = one proved bound | first fork window |
| E3 · the zheng bridge | the type check of a proof runs on nox and is certified by zheng; certificate as cyberlink in bbg | a registry theorem's certificate is a particle; `eidos verify` exists | with the proven present |
| E4 · oracle interfaces | random-oracle and QROM as axiomatized interfaces with query budgets; Fiat–Shamir composition (C-01, C-02) | the transcript spec's security row cites a theorem with its budget | after E2 |
| E5 · the port | statement-level Lean reader; the bounty's rules name eidos as the canonical form and Lean as accepted evidence | the first ported attacker certificate | when the first one arrives |

### 4.4 what this costs

E0 is a day. E1 is the first real corpus and sizes eidos's integer library: two to four sessions. E2 is the one that makes eidos a proof assistant for the stack rather than for toy programs: a finite-field and polynomial library is weeks of work, and the sum-check proof is the first theorem that would have been written in Lean elsewhere. E3 is already on eidos's roadmap and blocked on the same zheng work the proven present needs. The order is chosen so that each milestone proves something the launch uses.

## 5. watch and measurement

- capability watch: every release of `openai/math`, of Anthropic's and DeepMind's research artifacts, and a weekly cut of ePrint and arXiv cs.CC and cs.CR are scanned against the `watch` fields; the result is one row per entry in `cyber/audit/cryptography-<date>.md` within a day of the release. A status change on any entry is a row in `launch.md` and an owner decision.
- measurement: every class-B, F and G entry names a script and a date; hemera's campaign is the model (`p2rounds.py`), the simulator's verdict tables are the model for F and G, conformance for A. An entry whose measurement is older than 90 days is red in the gate.
- the response plan: `if-false` is written before the entry is accepted. An entry without a response is not an entry.

## 6. launch

Nothing in the genesis files changes. Before 2026-11-02: `specs/assumptions.md` with the seed entries of §3, `soft3/proofs/Axioms.ei` (E0), E1 for A-03 and A-05, the gate wired, the public mirror. H-01 remains the owner's decision and is the only red entry that changes a genesis file.

## 7. decisions for the owner

1. **Class G and H entries in the same registry.** Economic parameters and trust in keys are choices, not science; the proposal lists them anyway because `if-false` is exactly the plan the stack needs for them. Default: one list, eight classes.
2. **Lean evidence.** Accepted in `measured-by`, never as the record. Default: yes.
3. **E2 scope before or after the first fork.** The sum-check proof in eidos is the first theorem that replaces Lean in substance; it is not needed for genesis. Default: after launch, first fork window.
4. **H-01.** A checkpoint key and its sunset, or an explicit decision to launch without checkpoints and accept F-01's measured consequence. Default: a key, signed by the foundation, sunset at 180 days.

## sources

- OpenAI, "The Exponential Time Hypothesis is False", `openai/math` preprint, 2026-10-17 rev. 10-25; Lean scope `lean/docs/378.md`
- Alman, Vassilevska Williams, arXiv:2610.06783, 2026-10-05
- [[cyber/audit/cryptography-2026-10-08|the cryptography audit]] §2 (what the stack relies on at genesis), §6 (the 10-06 release read against the primitives)
- [[hemera/audit/campaign|hemera campaign]]; [[simulation/audit/network-2026-10|the simulator receipt]]; [[soft3/proposals/privacy-frozen|privacy, frozen]] G1–G7; [[eidos/specs/certificate|eidos certificate]]
