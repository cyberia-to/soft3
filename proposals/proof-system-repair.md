---
tags: cyber, soft3, proposal, zheng, lens, joy, proofs, post-quantum
crystal-type: process
crystal-domain: cyber
status: proposed
date: 2026-10-09
---
# one proof — the repair of zheng

> goal fixed by the owner on 2026-10-09: **a proof of any computation is at most 64 KB, post-quantum, verifies in at most 1 ms, and stays that size however long the computation — and the system that produces it is simple, reliable and flexible.** refined on the same day from the 2025–26 frontier (§3): **a small statement — a signature, a hash, a transfer — is at most 16 KB.** this page records what is broken, why the old numbers were never real, what the literature now allows, and the design that meets the goal with the fewest parts. it closes on the last merge, not the first.

## 1. the state of the art, measured

numbers that ship or are published, not claims. ours are measured on an M4 Max on 2026-10-08/09.

| system | primitives | security | proof on the wire | verify | source |
|---|---|---|---|---|---|
| **soft3 today** (joy 0.5.0 + zheng#45) | nox CCS · Spartan · PublicTensor (full witness) | linear check, no succinctness | one hash: **15,608 B** (was 294,861) | 30 ms | `zheng/audit/compact-relation-2026-10-09.md` |
| soft3 Triton zk path (trisha) | Triton VM STARK, Tip5 | hash-only, 160 bit | add 768 KB · hash **1.05 MB** | 10–170 ms | same |
| Starknet · Stwo | Circle STARK over M31, FRI | 96 bit proven · "conjectured" | **617 KB** · 221 KB (hash chain 2^14) | — | [proven.provably.fast](https://github.com/starknet-innovation/proven.provably.fast) |
| Neptune · Triton VM | STARK, Tip5, recursion in production | hash-only | **~1 MB** per block | — | [docs.neptune.cash](https://docs.neptune.cash/) |
| Quantus · wormhole | Plonky2 `standard_recursion_config`, Poseidon2, ML-DSA-87 | **100 bit conjectured** | **151 KB** private batch · 224 KB public batch · cap 512 KiB; one Plonky2 recursion proof measured by us: **127,192 B** | — | `Quantus-Network/chain` `pallets/wormhole/src/lib.rs:30-44` (482c5b9); plonky2 5d9da5a `bench_recursion` |
| parano1d | FRI-Binius over GF(2^128), Poseidon2b | NIST Cat 1 (~173 bit) | **913–981 KB** per block, cap 1.1 MB | 0.8–2.5 s | [performance.md](https://raw.githubusercontent.com/proof-native/parano1d/main/docs/reference/performance.md) |
| RISC Zero succinct receipt | FRI STARK + recursion | hash-only | **~200 KB**, constant | — | [risczero](https://dev.risczero.com/api/recursion) |
| WHIR (PCS) | RS proximity, hash-only | **128 bit** | rate 1/16: **56–87 KiB** · rate 1/2: 120–187 KiB | **0.4–0.8 ms** | [eprint 2024/1586](https://eprint.iacr.org/2024/1586.pdf) |
| STIR (PCS) | RS proximity, hash-only | 128 bit | 114 KiB (degree 2^26, rate 1/4) | — | same |
| LaBRADOR · Greyhound | lattices, Module-SIS | PQ | **~50–58 KB** | — | [2022/1341](https://eprint.iacr.org/2022/1341.pdf) · [2024/1293](https://eprint.iacr.org/2024/1293.pdf) |
| LaBinius (2026) | Binius + LaBRADOR compressor | PQ | **< 100 KiB** for a standard hash | — | [2026/2103](https://eprint.iacr.org/2026/2103.pdf) |
| not post-quantum, for scale | Jolt KZG wrapper 7.5 KB · Mina ~22 KB · Groth16 ~0.2 KB | curves | — | — | — |

two facts follow. the smallest post-quantum proof anyone has shown is about 50 KB at 128 bits; every post-quantum chain in production ships 150 KB to 1 MB. a 10 KB post-quantum proof of an arbitrary computation does not exist in the literature, and a 2 KB one is not on any horizon. the 2025 SoK adds a repricing: the capacity-soundness conjectures behind the "conjectured" columns above were disproven over large fields ([2026/1367](https://eprint.iacr.org/2026/1367)); honest parameters are Johnson-bound parameters, i.e. more queries.

**64 KB is therefore the right goal:** it is reachable with hash-only primitives at 128 bits (WHIR at rate 1/16 is already there for large polynomials), it beats every production chain by 3–15×, and it keeps the stack's one assumption — a hash.

## 2. what was broken, with evidence

the ~2 KB "constant proof" of zheng 0.3.x rested on five things that were false or unproven. none of them was the Merkle tree.

| hole | where | evidence |
|---|---|---|
| the opening checked nothing | lens dbf472b (2026-04-16), `brakedown/src/lib.rs` | the verifier compared the prover-supplied final value with itself; `let _ = (expected_idx, qidx); // transcript consistency ensured by absorption`; modulus `EXPANSION_M_PLACEHOLDER = 1024 // verifier doesn't know m exactly`. zheng#14 found 2,080 proof bytes that changed nothing by bit-flip scan |
| "Brakedown is Merkle-free" | `zheng/docs/explanation/recursive-brakedown.md:12` | a misreading: GLSTW21 §5 Brakedown is a Merkle tree over columns. a flat hash of a codeword binds it but cannot be opened at one position; spot checks need a vector commitment |
| the fold was never checked | `zheng/specs/decider.md:105` | HyperNova needs an additively homomorphic commitment; hemera is a hash. the error vector was prover-chosen; "leaves the verifier trusting the fold" |
| the statement was not bound | `decider.md:111`, tests `attack_satisfying_but_meaningless_witness_passes_for_any_statement` | program/input/output hashes only absorbed into Fiat–Shamir; a satisfying witness of nothing verified for any statement. `output_hash` hashed arena ids, not values |
| the constant wire was free | `decider.md:110`, `attack_zeroed_constant_wire_satisfies_universal_instance` | the all-zero witness satisfied the universal instance |
| code distance unproven, queries miscounted | lens#6 (open), `lens/specs/scalar-field.md:28-32` | "20 queries → 2^-20" assumed a proven distance; today only minimum weight 1 is proven, so the sampling count is 100·m and every column is opened |
| base-field challenges | `zheng/specs/execution.md:75` | all sumcheck and opening challenges are Goldilocks elements; 64-bit field ⇒ no 128-bit claim without an extension |
| λ counted in elements, not bits | `recursive-brakedown.md:244` ("≈ 2^{-λ}") next to a 128-element remainder | the document's size arithmetic treats 128 field elements as 128 bits of soundness; the soundness ledger of §5 counts bits only |

what the owner authorised on 2026-09-11 (`zheng/.claude/plans/authenticated-execution-release.md`) was the honest fallback: derive the relation on the verifier, disclose the witness, check every row. it is sound and it is not succinct. zheng#45 (2026-10-09) shrank that path 19× by shrinking the relation; it cannot shrink it further than the witness.

## 3. the design

### principles

- **one field**: Goldilocks for everything committed; its cubic extension (`strata/nebu` `fp3`, already written) for every challenge and every evaluation point. no second field.
- **one hash**: hemera (Poseidon2 over Goldilocks) for Merkle trees, for Fiat–Shamir, for program digests. the only cryptographic assumption is that this hash is a random oracle.
- **one code**: Reed–Solomon over Goldilocks (NTT in `nebu`), proximity by WHIR. no expander codes, no conjectured distance.
- **one IOP**: Spartan sumcheck over CCS, as today. CCS of any degree, as of zheng#45.
- **one accumulation**: ARC — hash-based accumulation of Reed–Solomon proximity claims, unbounded depth, up to list-decoding radius, random oracle only ([2024/1731](https://eprint.iacr.org/2024/1731), CRYPTO 2025). it replaces the homomorphic fold HyperNova needed and we never had. WARP ([2025/753](https://eprint.iacr.org/2025/753)) is the linear-time successor and the fallback if ARC's prover is too slow.
- **one wire format**, three profiles: `public` (disclosed witness, linear check — the fallback and the debugging tool), `succinct` (committed witness, WHIR, ≤ 64 KB), `zk` (succinct plus VEIL-style masking, [2026/683](https://eprint.iacr.org/2026/683.pdf)). one magic, one version, a profile byte.
- **the verifier derives everything it checks** — the relation from the program, the public prefix from the statement — and trusts nothing from the prover but field elements and hashes. this is the rule that survived from 2026-09-11, kept.

### the commitment, chosen from the whole 2023–2026 frontier

why WHIR and not STIR: STIR (CRYPTO 2024) is the older one; WHIR (late 2024, same authors) is its successor — multilinear, "super-fast verification", and smaller: 56–87 KiB vs 114 KiB at 128 bits, 1.0 ms vs 3.8 ms to verify. but the question is right in general: the proposal must choose from the full row of hash-based commitments, not from one paper. the row, with what each is for:

| scheme | year | code · geometry | what it is good at | size (128 bit unless noted) | fit |
|---|---|---|---|---|---|
| Basefold | 2023 | foldable codes, FRI-style | field-agnostic, simple | large | superseded by WHIR on RS |
| STIR | 2024 | RS, shrinking domains | fewer queries than FRI | 114 KiB (2^26, rate 1/4) | superseded by WHIR |
| **WHIR** | 2024 | RS, constrained folding, multilinear | smallest verifier; proven regime beyond unique decoding | **56–87 KiB** at rate 1/16; 36 KiB at 100 bit (SoK) | large polynomials, the accumulation decider |
| DeepFold | 2025 | RS, multilinear | optimal prover, concise proofs | large | alternative to WHIR, same class |
| Ligerito | 2025 | Ligero recursion + partial sumcheck, any linear-time code | linear-time prover; sizes ≈ WHIR in the proven regime; verifier heavier (code switching) | 255 KiB at 2^24 over a 32-bit binary field | if prover time wins over verifier time; not here |
| **SmallWood** | 2025 | hash-based PCS + ZK argument **for witnesses 2^6–2^16** | the smallest proofs in exactly our small-statement range | **< 25 KB**; CAPSS signatures on it: **9.5–15.5 KB for 24–35K R1CS constraints** | signature-like statements: one hash, one transfer, one vote |
| ReedWeave | 2026 | RS, interleaving + folding | fastest prover measured | 595 KiB (2^24, rate 1/4): big | prover-bound settings, not ours |
| DeepBrake | 2026 | row-wise RS, arbitrary points | Brakedown geometry with RS rows | — | the tensor track of §9 with a paper |
| FRI-Binius · Blaze | 2024–25 | binary towers | bit-heavy traces | ~1 MB blocks in parano1d | wrong field for a Goldilocks stack |

two size classes follow, and the goal splits into two numbers that are both met by hash-only schemes with papers:

- **small statements** (one relation of up to ~2^16 rows: a signature, a hash, a transfer, a vote, a lookup): the frontier is SmallWood/CAPSS at **10–16 KB at 128 bits**. this is the class the p2p market trades in most, and it is the class where "fits in a few packets" is honestly reachable. phase-2 acceptance for this class: **≤ 16 KB** (expected 10–16, measured against CAPSS's 9.5–15.5 at comparable constraint counts).
- **unbounded computation**: WHIR as the decider of an ARC accumulator, **≤ 64 KB**, constant in the number of steps.

phase 2's bake-off is therefore three-way on the small class — TensorMerkle+RS (today's code, repaired), WHIR, SmallWood — and two-way on the large class — WHIR, Ligerito/DeepFold — on the same fixtures, same ledger; one PCS per class ships, the rest is retired.

### the layers and who owns them

| layer | owns | repo |
|---|---|---|
| relation | program + subject shape → CCS; public prefix `(1, io, cycles)`; jets as rows | zheng `execution/relation` (today), nox semantics |
| IOP | Spartan over CCS: outer sumcheck, inner sumcheck, one evaluation claim per matrix family | zheng `spartan` |
| commitment | multilinear polynomial → RS codeword → hemera Merkle over columns or cosets; opening at an fp3 point by **TensorMerkle (Ligero geometry, RS rows) or WHIR — chosen by the phase-2 bake-off**; ARC accumulation of proximity claims | lens (`brakedown` becomes `tensor` with an RS code; `whir` added for the bake-off; the loser is retired) |
| state certificates | bbg's `QueryProof` is a lens commitment plus openings over the state polynomial; it rides whichever PCS wins and migrates in the same phase | bbg |
| accumulation driver | uniform step relation; fold the IOP's evaluation claims into one ARC accumulator per run; decider = one WHIR proof of the accumulator | zheng `accumulate` (replaces `folding`) |
| statement & wire | profile, program digest, inputs, outputs, cycles, budget; serialisation; versioning | joy `rs/execution` + zheng `execution/statement` |
| independent verifier | the whole verifier in Trident on nox, cross-checked against the Rust one on every fixture | trident `lib/std/zheng` (new) |

### the proof, end to end (succinct profile)

1. **relation.** the prover compiles the program exactly as the verifier will: CCS with a public prefix `z = (1 ‖ io ‖ cycles ‖ w)`. the verifier evaluates the prefix's multilinear extension itself; only `w` is committed. this is what binds program and io once the witness leaves the wire — the hole of 2026-09-09, closed structurally.
2. **verifying key.** the verifier recompiles the CCS from the program — as today, O(program), never from the prover — hashes it once (`vk = hemera(matrices)`) and absorbs `vk` instead of every entry. a light client caches `vk` per program it accepts; the step relation of §3 has one fixed `vk`. the artifact carries the program, never matrices.
3. **commit.** `w` as a multilinear polynomial in `log n` variables; RS-encode at rate 1/16; hemera Merkle over cosets of 2^4 leaves. one root.
4. **IOP.** Spartan: outer sumcheck (degree = CCS degree, 7 on hash programs) with challenges in fp3; inner sumcheck batching the matrix evaluations; one claim `w̃(r) = v`.
5. **open.** WHIR for `w̃(r)` with Johnson-bound parameters at 128 bits: folding factor 4, rate 1/16, grinding 16 bits. the verifier's work is `t` Merkle paths of hemera hashes plus a sumcheck of logarithmic length.
6. **wire.** statement · program · profile · root · sumcheck polynomials · matrix evaluations · WHIR rounds (roots, folded polynomials, query openings with deduplicated paths) · final polynomial.

### byte budget (estimate, to be measured at phase 2)

for one hash, `n = 2^10` after zheng#45, degree-7 CCS, 128-bit Johnson parameters, fp3 challenges (24 B per element):

| component | bytes |
|---|---|
| statement, program, profile | ~0.3 KB |
| root + Spartan: 10 outer rounds × 8 coefficients + 10 inner rounds × 3, 5 matrix evaluations | ~3 KB |
| WHIR: 3 rounds; t ≈ 64 queries; per query a coset of 16 elements (128 B) and a hemera path of 14 → 10 → 6 hashes (32 B each), paths deduplicated across queries (~−30 %) | ~28–36 KB |
| final polynomial, grinding nonce | ~0.5 KB |
| **total, 128 bit** | **~32–40 KB expected · 64 KB is the acceptance ceiling** |
| same at 100-bit conjectured (t ≈ 30) | ~16–20 KB |

the query count `t ≈ 64` is Johnson-bound arithmetic (`log₂(1/√ρ)` = 2 bits per query at rate 1/16), not a conjecture; it is also exactly where the 2025 repricing bites, so the budget is written with the ceiling, not the expectation, as the gate. the fp3 element is 24 B (three Goldilocks limbs) wherever a challenge or an evaluation travels.

verify: 64 queries × ≤ 30 hemera permutations + two short sumchecks ≈ 2,000 permutations ≈ **0.5–1 ms** native (hemera is ~0.3 µs per permutation on M4). the budget is dominated by Merkle authentication, as in every hash-based system; that is the price of "one hash" and it is within the goal.

for a program of `n = 2^20` the WHIR part grows by two more rounds and 10 more hashes per path: **~56–70 KB** at 128 bits — the number WHIR publishes.

### unbounded programs: accumulation, then one decider

a program longer than the relation limit (today 2^15 rows, 29 permutations) is proven step by step over a **uniform step relation** — one CCS for "one nox reduction step with continuity and memory arguments" (`zheng/audit/general-nox-relation-review.md:160-173` lists the requirements). each step yields one proximity claim about its committed witness; ARC folds claim after claim into one accumulator of fixed size (one root, one evaluation point, one value — a few hundred bytes plus the accumulated Merkle openings per step, which is where ARC's "small number of openings relative to the code rate" matters). at the end, one WHIR proof decides the accumulator. the proof of a million steps is the same ~60 KB as the proof of one.

this is the *recursion milestone* every zheng document pointed at. the difference from the 2026 plan is that it no longer needs a homomorphic commitment nor a verifier circuit of the verifier: ARC is accumulation in the random-oracle model, which is exactly the one assumption the stack makes.

what phase 3 must pin down before it is built — the part the first draft left as one line:

- **the accumulator on the wire**: one Merkle root of the accumulated codeword, one evaluation point in fp3, one claimed value, and the per-step openings ARC needs — "a small number of Merkle openings relative to the code rate" ([2024/1731](https://eprint.iacr.org/2024/1731)); at rate 1/16 that is a handful of cosets per step, so the in-flight object is a few KB and does not grow with the number of steps;
- **the per-step cost**: one RS encoding of the step witness, one Merkle tree, one proximity reduction; measured on the nox step relation before phase 3 commits, next to the Neo-style lattice fold of §10;
- **the decider**: one opening (TensorMerkle or WHIR, whichever phase 2 chose) of the final accumulator — the same ≤ 64 KB as a single-shot proof;
- **the fallback** if ARC's prover is too slow or its argument does not close on review: bounded-depth recursion — the Trident verifier of §5 proven inside nox, one level at a time. costlier, standard, sound; it keeps the goal, not the elegance.

until phase 3 lands, phase 2 delivers succinct proofs for one relation of at most 2^15 rows (about 29 hemera permutations), not for "any computation"; the goal of §0 is met only when phase 3 does.

### zero knowledge

the `zk` profile masks the committed polynomial and the sumcheck with random low-degree terms (VEIL, 2026) — a few KB and a few percent of prover time over `succinct`, no second proof system. the Triton path (trisha) remains as the independent zk oracle for differential testing until `zk` is reviewed, and is then retired from the product path.

## 4. simplicity: what disappears

- `lens/brakedown` (PublicTensor, TensorMerkle, the recursive stub `UnsupportedRecursiveOpening`): replaced by `lens/whir`.
- `zheng/folding` (HyperNova over hemera, unsound by construction): replaced by `zheng/accumulate` (ARC).
- formats `zheng-hypernova-tensor-merkle-v2`, `JOYEXEC1`, `JOYEXEC2`, `JOYZK003`: one format with a profile byte; the old ones read by a `legacy` tool for a release, then gone.
- the disclosed/tagged/native-private experimental kernels on `release/0.4` keep their audits and leave the production path once `zk` lands.
- expander codes, `MIN_ABS_WEIGHT`, `num_queries = 100·EXPANSION·k2`: gone with the code.

what stays: the relation compiler (now 1/6 of its old size per hash), Spartan, hemera, nebu, nox, joy's statement model. the whole proving path — relation, IOP, commitment, accumulation, wire — should fit in about 12k lines of Rust against today's ~25k across lens, zheng/folding and zheng/execution.

## 5. reliability: how it stays sound

- **a soundness ledger** (`zheng/specs/soundness.md`, new): one row per component — assumption, bits claimed, proven or conjectured, the paper, the parameter that controls it. the release gate fails if any row is "conjectured" on the production profile.
- **attack tests that must fail.** the two residual tests of 0.3.2 flip from "passes, documenting a hole" to "must be rejected": a meaningless satisfying witness for a statement; a zeroed constant wire. plus: forged public output, forged `vk`, truncated Merkle path, query index replay, challenge reuse across rounds. every one is a fixture, every one is a release gate.
- **bit-flip scan** over every byte of every fixture proof: a byte that can change without the verifier noticing is a bug (the method that found the 2,080 dead bytes of 0.3.0).
- **differential against native**: every circuit digest against `nox::data::hash`, every output against `nox::reduce`, as today.
- **two verifiers**: the Rust verifier and a Trident verifier on nox, run on every fixture, must agree — the `cross-verified` feature made real for the proof system itself. the Trident verifier is also the seed of the decider circuit, if recursion is ever wanted on top of accumulation.
- **frozen fixtures**: a proof, once produced for a release, is a fixture that every later verifier must still accept (or the format version moves).

## 6. flexibility: what the design leaves open

- any CCS degree (zheng#45): jets as single high-degree rows, lookups via state tables as today.
- profiles are a byte, not a fork: a market can require `succinct`, a dispute can demand `public`, a wallet can insist on `zk`.
- the verifier is program-independent through `vk`: a light client holds `vk`s, not programs.
- the step relation is the only thing that needs to grow for new nox patterns; the commitment, accumulation and wire do not know what a nox is.
- security parameters are data: queries, rate, grinding, extension degree live in one struct, printed into the proof header, checked by the verifier against its policy.

## 7. the path, with acceptance

| phase | what | acceptance | repos |
|---|---|---|---|
| 0 ✓ | relation shrink: linear forms, native constants, degree-7 S-box (zheng#45) | hash.tri 294,861 → 15,608 B, prove/verify 460 → 36/30 ms; 259 tests | zheng |
| 1 | public prefix + `vk` digest + paths dropped from the public profile; one format with a profile byte | hash.tri public ≤ 10 KB; forged-io and forged-vk fixtures rejected; the two residual tests flip | zheng, joy |
| 2 | **bake-off** (§3): small class — today's `TensorMerkle` with a Reed–Solomon code and fp3, WHIR, SmallWood; large class — WHIR, Ligerito/DeepFold. same fixtures, same ledger; one PCS per class ships, the rest is retired | small class (hash.tri, a transfer): **≤ 16 KB, verify ≤ 1 ms**, against CAPSS's 9.5–15.5 KB at 24–35K constraints; large class (`n = 2^20` fixture): **≤ 64 KB, ≤ 1 ms**; bit-flip scan clean; ledger complete, all rows proven; bbg `QueryProof` migrated | lens, zheng, bbg, strata (fp3 exposure) |
| 3 | uniform step relation + ARC accumulation + decider | merkle-32 and a 10^6-step run both prove; proof size independent of length, **≤ 64 KB**; verify ≤ 1 ms | zheng, nox |
| 4 | `zk` profile (VEIL masking); Trident verifier | zk fixtures; Rust and Trident verifiers agree on every fixture | zheng, trident |
| 5 | delete: brakedown, folding, legacy formats, stale docs (§8) | `tokei` shows the proving path ≤ 12k lines; no document claims 2 KB or 100 ns | lens, zheng, joy, nox, bbg, crystal |

phases 1 and 2 are a month of focused work each; 3 is the research-grade one and is where ARC's prover cost must be measured before committing; 4 and 5 are weeks.

## 8. documents that still describe the old design as current

to be marked "superseded by proof-system-repair" or deleted in phase 5: `zheng/specs/{verifier,api,README,decider,accumulator,recursion}.md`, `zheng/docs/explanation/{recursive-brakedown,polynomial-commitments,whirlaway,fri-to-whir,zheng-vs-starks,performance}.md`, `zheng/CLAUDE.md` (Brakedown line), `nox/specs/jets/{decider,recursion}.md`, `lens/README.md`, `lens/specs/commitment.md`, `bbg/specs/{architecture,data-availability}.md` (the ~2 KiB / ~75 B lines), `cyber/whitepaper.md:1202` (~22 KB), `cyber/light.md:115`, `crystal/architecture.md:331-341`, `soft3/docs/polynomial-proof-system.md` (already bannered; the link to `roadmap/` is wrong).

## 9. the Merkle question — can the trees go?

the owner asked this to be dug properly on 2026-10-09: the trees are the byte bottleneck, the Merkle-free design was "a solid calculation", is the blocked soundness gap just a bug? here is the dig.

### the design, reconstructed

`zheng/docs/explanation/recursive-brakedown.md`: commit `C = hemera(Enc(w))`, one hash of the whole codeword. to open ⟨w, q⟩ = v with `q = q₁ ⊗ q₂`: the prover sends `y = q₁ᵀ W`; the verifier draws t columns and checks `Enc(y)[j] = q₁ᵀ · col_j`; then recurse — commit `y` the same way instead of sending it. "32 bytes per level, zero trees, ~1.3 KiB".

### the step that breaks

the column check means something only if `col_j` was fixed before `q₁` was drawn. the only thing that fixes columns is `C` — and `C` is a hash of the whole word: to check that a received `col_j` is the j-th column of `Enc(W)` the verifier needs the entire preimage. a flat hash has no local opening.

the attack, on an honest implementation that does read the columns:

1. commit any `C` (the hash of zeros will do);
2. receive `q₁`;
3. pick any `y'`, compute `Enc(y')`;
4. for each queried column j solve `q₁ᵀ · col_j = Enc(y')[j]` — one linear equation in k₁ unknowns, infinitely many solutions;
5. every check passes; `⟨y', q₂⟩` is whatever `v` the prover wants.

no step touches `C`. that is why lens dbf472b "worked": its verifier did not read the columns at all, and the difference from this attack is cosmetic. the recursion changes nothing — every level has the same flat hash and the same hole, smaller.

### why one more hash does not fix it

the verifier must recompute `C` from what it receives. `C` depends on all n symbols; the verifier wants to read one. the other n−1 symbols must therefore arrive compressed, as hash outputs, each covering some subtree of hash calls. a chain `H(H(…), cⱼ)` costs one digest per position — linear. a balanced tree costs `(arity−1)·log n` digests, and that is minimal among structures where a digest covers a subtree. in a hash-only world a Merkle path is not an implementation choice but a lower bound: **a local opening costs Θ(log n) digests, and no rearrangement of hash calls changes it.** this is why WHIR, STIR, Basefold, FRI-Binius, Plonky2 and parano1d all carry paths.

the only thing that escapes the bound is a **homomorphism**: if the commitment is linear — `C = A·Enc(w)` over a lattice, Ajtai/SIS — then "this column is consistent with C" is checked by algebra without the preimage, and a Bulletproofs-style recursion gives logarithmic size. that is exactly LaBRADOR and Greyhound, and they cost ~50 KB because lattice elements are heavy. the idea "everything algebraic, no trees" is alive; its name is lattices and its price is tens of KB, not 2.

### what the intuition gets right: attack the cost of the trees

for one hash at 128 bits, starting from the ~36 KB of §3:

| lever | what it does | effect |
|---|---|---|
| **a code with a proven distance** (Reed–Solomon instead of the one-layer expander whose proven minimum weight is 1) | query count from `100·m` (every column) to the Johnson count (~64 at 128 bits) | the first and largest win: it is what turns "open everything" into an opening at all |
| path deduplication | t queries share the top levels; send shared nodes once | −30…−45 % |
| rate 1/32 instead of 1/16 | Johnson bits per query `log₂(1/√ρ)`: 2 → 2.5; queries 64 → 52 | −20 % of path bytes, prover ×2 |
| grinding 20 bits | 20 bits for free; 8 fewer queries | −12 % |
| folding factor 5 | 32-element leaves, fewer levels, fewer rounds | −10 % |
| fp3 challenges | 24 B per element instead of 32 | −3 % |
| **together** | | **~15–20 KB** |
| 100 bits instead of 128 | queries ×0.78 | ~12–16 KB |

this is the real floor for "one hash, post-quantum": 15–20 KB at 128 bits, well inside the 64 KB goal and below every production chain. recursive compression (proving the WHIR verifier inside nox) buys nothing in a hash-only world — the final proof carries its own paths again. phase 2 adopts these levers as parameters, not as separate work.

### the tensor track, with its arithmetic

the sound half of the old idea survives: Ligero geometry — a `k₁ × k₂` matrix, RS rows, a Merkle tree over columns, the combination `y = q₁ᵀW` sent in the clear, t columns opened — is today's `TensorMerkle`, and GLSTW21 §5 is its proof. "do not send y, commit it and recurse" is the part that needs an argument for composing levels (open question 1 of `recursive-brakedown.md`), and it has a byte arithmetic that bounds what it can win: **each query opens a whole column of k₁ elements**, so a level costs `t · k₁ · 8 B` before any path.

| n | geometry | columns opened | y | paths (dedup.) | ≈ total |
|---|---|---|---|---|---|
| 2^10 | 32 × 32, rate 1/16 | 64 × 32 × 8 = 16 KB | 256 B | ~10 KB | **~27–35 KB** |
| 2^20 | 1024 × 1024 | 64 × 1024 × 8 = **512 KB** | 8 KB | — | not viable |
| 2^20 | k₁ = 64, recurse y (2^14 → 64 × 256 → 256) | 32 KB + 32 KB | 2 KB | ~20 KB | **~80–100 KB** |

at 2^10 the tensor opening and WHIR land in the same 25–40 KB band; at 2^20 the recursion of y pays `t · k₁` at every level and ends at or above WHIR's 56–87 KiB — because WHIR *is* this recursion (folding by 2^k per round is "k₁ = 2^k"), done with a proximity argument across rounds. that is why the proposal does not pick a winner on paper: phase 2 measures both on the same fixtures and keeps the smaller sound one. what no version of the track brings back is `C = hemera(w)`, and with it 1.3 KB.

## 10. lattices — what they buy, and what they do not

the owner's second question: if lattices are the only algebraic route, is there something there beyond the 50 KB, or is it not worth the weight? read the recent literature:

- **Neo / SuperNeo** (Nguyen–Setty, [2025/294](https://eprint.iacr.org/2025/294.pdf), [2026/242](https://eprint.iacr.org/2026/242.pdf)): a lattice folding scheme *for CCS over small prime fields* — HyperNova's folding with Ajtai commitments instead of curves, one sumcheck per fold over a small-field extension, and **pay-per-bit** commitment cost: committing a vector of bits is 64× cheaper than a vector of 64-bit integers. this is our setting exactly: Goldilocks, CCS, hash traces full of bits.
- **LatticeFold+** ([2025/247](https://eprint.iacr.org/2025/247.pdf)): folding proof size `O(κd + log n)`, no decomposed commitments, smaller verification circuit.
- **HyperWolf** ([2025/922](https://eprint.iacr.org/2025/922)): lattice PCS, O(log N) proof size — and still **~436 KB at N = 2^20**. **Greyhound** ~53 KB; **LaBRADOR** ~50 KB as the final compressor.

so the honest reading:

| | hash-only (WHIR + ARC) | lattices (Neo + LaBRADOR) |
|---|---|---|
| final proof | 15–40 KB (128 bit) | ~50 KB; lattice PCS alone 400 KB+ |
| the accumulator between steps | a root + openings: a few KB, Merkle-authenticated | **one Ajtai commitment + short vectors: a few KB, no tree, pure algebra** |
| per-step prover | encode + Merkle + proximity sumcheck | one sumcheck + a matrix-vector product; **pay-per-bit** makes bit-heavy traces cheap |
| verifier | hashes along paths | ring/matrix arithmetic; slower per element |
| assumptions | one: the hash | two: the hash (Fiat–Shamir) **and** SIS; parameters need norm bookkeeping |
| maturity | WHIR implemented and audited in several stacks | folding implementations young; soundness proofs still moving (LatticeFold → +, Neo → SuperNeo within a year) |

what lattices buy is **not the final byte count** — the trees come back in the decider, or the lattice PCS is bigger than the trees. what they buy is the **shape of accumulation**: a homomorphic fold with a tiny in-flight object and no proximity machinery per step. if the thing that must fit in a packet is *what travels between steps of a p2p computation* rather than the final proof, lattices are the only way to make that object a few KB of algebra. that is a real reason, and it is also a second assumption.

**decision recorded:** phase 2 is hash-only WHIR with the levers of §9 (one assumption, measured floor 15–20 KB). phase 3's accumulation is implemented first as ARC; a **Neo-style lattice fold is a scheduled research spike** before phase 3 commits: measure the fold's in-flight size and per-step cost on the nox step relation against ARC's, and decide on numbers. nothing in phases 1–2 depends on the outcome.

## 11. what this proposal does not claim

no 2 KB. no 100 ns. no "Merkle-free". no proof smaller than the authentication of its own queries. the stack keeps one assumption and pays for it in bytes; 64 KB is what that honesty costs at 128 bits, and it is still the smallest in production.
