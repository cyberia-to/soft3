---
tags: cyber, soft3, proposal, zheng, lens, joy, proofs, post-quantum
crystal-type: process
crystal-domain: cyber
status: proposed
date: 2026-10-09
---
# one proof — the repair of zheng

## 0. goal

fixed by the owner on 2026-10-09:

| class | statement | proof on the wire | verify | security |
|---|---|---|---|---|
| small | one relation ≤ 2¹⁶ rows: a hash, a signature, a transfer, a vote | ≤ 16 KB | ≤ 1 ms | 128 bit, post-quantum, hash-only |
| any | a computation of any length | ≤ 64 KB, constant in the number of steps | ≤ 1 ms | same |

and the system that produces it is simple, reliable and flexible. this page records what is broken, the design with the fewest parts, and the gates that decide each phase. it closes on the last merge.

## 1. where the numbers stand

| system | proof | verify | security |
|---|---|---|---|
| soft3 today — public certificate (joy 0.5.0 + zheng#45) | 15,608 B for one hash, witness disclosed | 30 ms | sound, linear, not succinct |
| soft3 Triton zk path (trisha) | 1.05 MB | 10–170 ms | hash-only, 160 bit |
| production chains — Stwo, Triton, RISC Zero, parano1d, Quantus | 150 KB – 1 MB | — | 96–173 bit |
| WHIR, hash-only PCS (2024) | 56–87 KiB at rate 1/16 | 0.4–0.8 ms | 128 bit |
| SmallWood / CAPSS, small statements (2025) | 9.5–15.5 KB at 24–35K constraints | — | 128 bit |
| LaBRADOR / Greyhound / LaBinius, lattices | 50–83 KB | 700 ms | PQ, second assumption |
| curves, for scale: Groth16 / Jolt / Mina | 0.2 / 7.5 / 22 KB | — | not post-quantum |

sources: `zheng/audit/compact-relation-2026-10-09.md`, [WHIR 2024/1586](https://eprint.iacr.org/2024/1586.pdf), [SmallWood](https://eprint.iacr.org/2025/1085), [LaBinius 2026/2103](https://eprint.iacr.org/2026/2103.pdf), [SoK repricing 2026/1367](https://eprint.iacr.org/2026/1367). the smallest post-quantum proof anyone has shown is ~50 KB; a 10 KB one for arbitrary computation does not exist; 64 KB is reachable hash-only and beats every production chain 3–15×.

## 2. what is broken

the ~2 KB "constant proof" of zheng 0.3.x rested on five false or unproven things. none was the Merkle tree.

| hole | evidence | fix (phase) |
|---|---|---|
| the opening checked nothing — verifier compared a prover value with itself; 2,080 dead proof bytes found by bit-flip scan | lens dbf472b, zheng#14 | opening by a PCS with a proof (2) |
| "Brakedown is Merkle-free" — a flat hash cannot be opened at one position; a local opening costs Θ(log n) digests in a hash-only world, by lower bound | `zheng/docs/explanation/recursive-brakedown.md:12`; §A | Merkle paths, deduplicated (2) |
| the fold was never checked — HyperNova needs a homomorphic commitment, hemera is a hash; the error vector was prover-chosen | `zheng/specs/decider.md:105` | ARC accumulation (3) |
| the statement was not bound — a satisfying witness of nothing verified for any statement | `decider.md:111`, test `attack_satisfying_but_meaningless_witness_passes_for_any_statement` | public prefix in the CCS (1) |
| the constant wire was free — the all-zero witness satisfied the universal instance | `decider.md:110`, test `attack_zeroed_constant_wire_satisfies_universal_instance` | pinned constant (1) |
| code distance unproven — minimum weight 1 is all that is proven, so `num_queries = 100·m` opens every column | lens#6, `lens/specs/scalar-field.md:28-32` | Reed–Solomon (2) |
| base-field challenges — 64-bit Goldilocks, no 128-bit claim possible | `zheng/specs/execution.md:75` | fp3 challenges (1) |
| λ counted in elements, not bits | `recursive-brakedown.md:244` | soundness ledger (1) |

the owner's 2026-09-11 fallback — derive the relation on the verifier, disclose the witness, check every row — is sound and stays as the `public` profile.

## 3. the design

seven rules, each a removal:

1. one field — Goldilocks for everything committed; its cubic extension fp3 (`strata/nebu`, written) for every challenge and evaluation point.
2. one hash — hemera for Merkle trees, Fiat–Shamir and program digests. the only assumption: hemera is a random oracle.
3. one code — Reed–Solomon over Goldilocks (NTT in nebu). no expander codes, no conjectured distance.
4. one IOP — Spartan sumcheck over CCS of any degree (zheng#45).
5. one accumulation — ARC: hash-based, unbounded depth, up to list-decoding radius, random oracle only ([2024/1731](https://eprint.iacr.org/2024/1731)); WARP ([2025/753](https://eprint.iacr.org/2025/753)) as the linear-time fallback.
6. one wire format, three profiles — `public` (disclosed witness, linear check), `succinct` (committed witness, ≤ 64 KB), `zk` (succinct + VEIL masking, [2026/683](https://eprint.iacr.org/2026/683.pdf)). one magic, one version, one profile byte.
7. the verifier derives everything it checks — the relation from the program, the public prefix from the statement — and trusts nothing from the prover but field elements and hashes.

| layer | owns | repo |
|---|---|---|
| relation | program + subject shape → CCS; public prefix `(1 ‖ io ‖ cycles)`; jets as rows; `vk = hemera(matrices)` | zheng `execution/relation` |
| IOP | Spartan: outer and inner sumcheck, one evaluation claim per matrix family | zheng `spartan` |
| commitment | multilinear → RS codeword → hemera Merkle; opening at an fp3 point by the phase-2 winner | lens |
| accumulation | uniform step relation; ARC folds evaluation claims; decider = one opening of the accumulator | zheng `accumulate` (replaces `folding`) |
| statement & wire | profile, program digest, io, cycles, budget; serialisation; versioning | joy `rs/execution` + zheng `execution/statement` |
| state certificates | bbg `QueryProof` rides the winning PCS | bbg |
| second verifier | the whole verifier in Trident on nox, agreeing with Rust on every fixture | trident `lib/std/zheng` |

the proof, succinct profile: relation (prover and verifier compile the same CCS; only `w` is committed) → `vk` (the verifier recompiles, hashes once, absorbs the digest) → commit (`w` as a multilinear polynomial, RS-encoded, one hemera root) → IOP (Spartan with fp3 challenges, one claim `w̃(r) = v`) → open (Johnson-bound parameters, 128 bit) → wire (statement · program · profile · root · sumcheck polynomials · evaluations · opening).

byte budget for one hash (`n = 2¹⁰`, degree-7 CCS, 128-bit Johnson, fp3 at 24 B), WHIR as the worked example: statement 0.3 KB · Spartan 3 KB · opening 28–36 KB · final polynomial 0.5 KB → 32–40 KB expected, 64 KB the ceiling; 16–20 KB at 100 bit. verify ≈ 2,000 hemera permutations ≈ 0.5–1 ms. at `n = 2²⁰`: 56–70 KB, WHIR's published number.

unbounded programs: a uniform step relation (one CCS for one nox reduction step with continuity and memory arguments, `zheng/audit/general-nox-relation-review.md:160-173`); each step yields one proximity claim; ARC folds claims into one accumulator of fixed size (one root, one fp3 point, one value, a few cosets of openings per step); one opening decides it. proof size independent of length. fallback if ARC's prover is too slow or its argument does not close on review: bounded-depth recursion through the Trident verifier — costlier, standard, sound.

## 4. phases and gates

every gate is a fixture and a command, not an opinion. a phase closes when its row is green on CI.

| phase | what | gate | repos |
|---|---|---|---|
| 0 ✓ | relation shrink: linear forms, native constants, degree-7 S-box (zheng#45) | hash.tri 294,861 → 15,608 B; prove/verify 460 → 36/30 ms; 259 tests | zheng |
| 1 | soundness floor: public prefix + pinned constant + `vk` digest + fp3 challenges + one format with a profile byte + the soundness ledger | `public` hash.tri ≤ 10 KB · fixtures `forged-io`, `forged-vk`, `zeroed-constant`, `meaningless-witness` rejected · ledger has no "conjectured" row | zheng, joy, strata |
| 2 | bake-off: small class — TensorMerkle+RS+fp3, WHIR, SmallWood; large class — WHIR, Ligerito/DeepFold; same fixtures, same ledger; one PCS per class ships, the rest is deleted; zip measured on the winner | small (hash.tri, a transfer): ≤ 16 KB, verify ≤ 1 ms · large (`n = 2²⁰`): ≤ 64 KB, ≤ 1 ms · bit-flip scan clean · bbg `QueryProof` migrated | lens, zheng, bbg |
| 3 | uniform step relation + ARC + decider; lattice-fold spike measured first (§B) | merkle-32 and a 10⁶-step run both prove · size independent of length, ≤ 64 KB · verify ≤ 1 ms | zheng, nox |
| 4 | `zk` profile (VEIL); Trident verifier | zk fixtures · Rust and Trident verifiers agree on every fixture | zheng, trident |
| 5 | delete: brakedown, folding, legacy formats, stale docs (§C) | proving path ≤ 12k lines by `tokei` · no document claims 2 KB or 100 ns | lens, zheng, joy, nox, bbg, crystal |

phases 1 and 2 are a month each; 3 is research-grade and measures before it commits; 4 and 5 are weeks. until 3 lands, `succinct` covers one relation of ≤ 2¹⁵ rows, and the "any computation" row of §0 is open.

## 5. how it stays sound

- soundness ledger (`zheng/specs/soundness.md`): one row per component — assumption, bits claimed, proven or conjectured, paper, controlling parameter. the release gate fails on any "conjectured" row in a production profile.
- attack fixtures that must be rejected: meaningless witness, zeroed constant, forged io, forged `vk`, truncated path, query-index replay, challenge reuse. each one a release gate.
- bit-flip scan over every byte of every fixture proof — the method that found the 2,080 dead bytes.
- differential against native: circuit digests against `nox::data::hash`, outputs against `nox::reduce`.
- two verifiers, Rust and Trident, agreeing on every fixture.
- frozen fixtures: a proof produced for a release stays accepted by every later verifier, or the format version moves.
- security parameters are data — queries, rate, grinding, extension degree in one struct, printed in the header, checked against the verifier's policy.

## 6. what disappears

`zheng/folding` (HyperNova over hemera) → `zheng/accumulate` (ARC). the lens loser of the bake-off; `UnsupportedRecursiveOpening`; the flat-hash `Tensor` opening; expander codes, `MIN_ABS_WEIGHT`, `num_queries = 100·EXPANSION·k2`. formats `zheng-hypernova-tensor-merkle-v2`, `JOYEXEC1`, `JOYEXEC2`, `JOYZK003` → one format, read by a `legacy` tool for one release. the experimental kernels on `release/0.4` leave the production path when `zk` lands. `PublicTensor` stays as the `public` profile. the proving path — relation, IOP, commitment, accumulation, wire — fits in ~12k lines against ~25k today.

## 7. open, and who closes it

| question | closed by |
|---|---|
| which PCS per class | phase 2, on numbers |
| ARC prover cost per step vs a Neo-style lattice fold (§B) | the phase-3 spike, on numbers |
| zip's measured gain on the winner | phase 2 |
| in-proof digest length under hemera profile v2 | decided: 32 B in trees (§B) |

## A. decisions recorded, with the argument compressed

the Merkle question. the owner asked whether the trees can go. they cannot: a flat hash `C = hemera(Enc(w))` has no local opening, and the column check `Enc(y)[j] = q₁ᵀ·col_j` is satisfiable by a prover who never touches `C` (pick `y'`, solve one linear equation per queried column). in a hash-only world a local opening costs Θ(log n) digests — a lower bound, not a layout; every hash-based system carries paths. the only escape is a homomorphism, i.e. lattices, at ~50 KB. the sound half of the old idea — Ligero geometry, RS rows, a column tree, `y` in the clear — is today's `TensorMerkle` and is a bake-off candidate; at `n = 2¹⁰` it and WHIR land in the same 25–40 KB band, at `2²⁰` folding wins. no version brings back `C = hemera(w)` or 1.3 KB.

the levers on the trees, from ~36 KB for one hash: a code with a proven distance (queries from every column to ~64 — the largest win); path deduplication −30…45 %; rate 1/32 −20 % at ×2 prover; grinding 20 bits −12 %; folding factor 5 −10 %; fp3 at 24 B −3 %. honest floor 25–40 KB at 128 bit; 15–20 KB if the levers compose; zip ×0.6 on top of either.

## B. lattices and the frontier, 2026-10-09

what lattices buy is the shape of accumulation, not the final byte count: a homomorphic fold with a tiny in-flight object (Neo/SuperNeo [2025/294](https://eprint.iacr.org/2025/294.pdf), LatticeFold+ [2025/247](https://eprint.iacr.org/2025/247.pdf), Symphony [2025/1905](https://eprint.iacr.org/2025/1905), PikkuFold ~5.7 KB per step [2026/1809](https://eprint.iacr.org/2026/1809.pdf)) at the price of a second assumption and norm bookkeeping. LaBinius ([2026/2103](https://eprint.iacr.org/2026/2103.pdf)) now wins the bytes at `2²⁴` — 82.9 KiB against WHIR's 300.9 at rate ¼ — and loses the verifier 650× (713 ms against 1.1). under the goal as fixed, hash-only stands; the spike's one question is whether any lattice verifier gets under 10 ms.

zip ([2025/1446](https://eprint.iacr.org/2025/1446)): black-box compression of hash-based proofs to ~60 %, standard assumptions. it composes with every lever because it acts on the finished proof; phase 2 measures it.

the in-proof digest: hemera profile v2 (hemera PR #15) raises the identity digest to 48 or 64 bytes for post-quantum collision resistance. a Merkle node inside a proof is ephemeral — it must be forged before the verifier runs, so "harvest now, break later" does not apply — and stays 32 bytes. one permutation, two squeeze lengths; a 48-byte node would add 50 % to the dominant term for nothing.

what this proposal does not claim: no 2 KB, no 100 ns, no "Merkle-free", no proof smaller than the authentication of its own queries.

## C. documents that still describe the old design

to be marked superseded or deleted in phase 5: `zheng/specs/{verifier,api,README,decider,accumulator,recursion}.md`, `zheng/docs/explanation/{recursive-brakedown,polynomial-commitments,whirlaway,fri-to-whir,zheng-vs-starks,performance}.md`, `zheng/CLAUDE.md` (Brakedown line), `nox/specs/jets/{decider,recursion}.md`, `lens/README.md`, `lens/specs/commitment.md`, `bbg/specs/{architecture,data-availability}.md`, `cyber/whitepaper.md:1202`, `cyber/light.md:115`, `crystal/architecture.md:331-341`, `soft3/docs/polynomial-proof-system.md`.

the long form of every argument above is in this file's history (`git log -p proposals/proof-system-repair.md` before this revision).
