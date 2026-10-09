---
tags: cyber, soft3, proposal, zheng, lens, joy, proofs, post-quantum
crystal-type: process
crystal-domain: cyber
alias: proof system repair, one proof, zheng repair
status: proposed
date: 2026-10-09
---
# one proof — the repair of zheng

## 0. goal

fixed by the owner on 2026-10-09:

| class | statement | proof on the wire | verify | security |
|---|---|---|---|---|
| small | one relation ≤ 2¹⁶ rows: a [[hemera]] hash, a signature, a transfer, a vote | ≤ 16 KB | ≤ 1 ms | 128 bit, post-quantum, hash-only |
| any | a [[nox]] computation of any length | ≤ 64 KB, constant in the number of steps | ≤ 1 ms | same |

and the system that produces it is simple, reliable and flexible. this page records what is broken in [[zheng]] and [[lens]], the design with the fewest parts, where [[recursion|recursion]] is and is not needed, and the gates that decide each phase, and how the change rides the release train through nine repositories. it closes on the last merge. the boundaries it respects are [[soft3/roadmap/component-boundaries|component boundaries]]; the sibling proposals are [[network-planes]] and [[tade-one-exchange]].

## 1. where the numbers stand

| system | proof | verify | security |
|---|---|---|---|
| [[soft3]] today — public certificate ([[joy]] 0.5.0 + zheng#45) | 15,608 B for one hash, witness disclosed | 30 ms | sound, linear, not succinct |
| soft3 [[Triton VM]] zk path (trisha) | 1.05 MB | 10–170 ms | hash-only, 160 bit |
| production chains — Stwo, Triton, RISC Zero, parano1d, Quantus | 150 KB – 1 MB | — | 96–173 bit, [[STARK]]-family |
| [[WHIR]], hash-only PCS (2024) | 56–87 KiB at rate 1/16 | 0.4–0.8 ms | 128 bit |
| SmallWood / CAPSS, small statements (2025) | 9.5–15.5 KB at 24–35K constraints | — | 128 bit |
| LaBRADOR / Greyhound / LaBinius, lattices ([[Module-SIS]]) | 50–83 KB | 700 ms | PQ, second assumption |
| curves, for scale: Groth16 / Jolt / Mina | 0.2 / 7.5 / 22 KB | — | not post-quantum |

sources: `zheng/audit/compact-relation-2026-10-09.md`, [WHIR 2024/1586](https://eprint.iacr.org/2024/1586.pdf), [SmallWood](https://eprint.iacr.org/2025/1085), [LaBinius 2026/2103](https://eprint.iacr.org/2026/2103.pdf), [SoK repricing 2026/1367](https://eprint.iacr.org/2026/1367); the lineage of the commitment choice is told in [[fri-to-whir]] and [[whirlaway]], and the comparison with STARKs in [[zheng-vs-starks]]. the smallest post-quantum proof anyone has shown is ~50 KB; a 10 KB one for arbitrary computation does not exist; 64 KB is reachable hash-only and beats every production chain 3–15×.

## 2. what is broken

the ~2 KB "constant proof" of zheng 0.3.x rested on five false or unproven things. none was the Merkle tree ([[merklezation]]).

| hole | evidence | fix (phase) |
|---|---|---|
| the opening checked nothing — the [[zheng/specs/verifier|verifier]] compared a prover value with itself; 2,080 dead proof bytes found by bit-flip scan | lens dbf472b, zheng#14 | an opening with a proof, by a real [[polynomial commitments|polynomial commitment]] (2) |
| "[[Brakedown]] is Merkle-free" — a flat hash cannot be opened at one position; a local opening costs Θ(log n) digests in a hash-only world, by lower bound | [[recursive-brakedown]] §1; §A | Merkle paths, deduplicated (2) |
| the fold was never checked — HyperNova needs a homomorphic commitment ([[commitments]]), hemera is a hash; the error vector was prover-chosen | [[zheng/specs/decider|decider]] §soundness | ARC accumulation (3) |
| the statement was not bound — a satisfying witness of nothing verified for any statement | `decider.md:111`, test `attack_satisfying_but_meaningless_witness_passes_for_any_statement` | public prefix in the [[CCS]] (1) |
| the constant wire was free — the all-zero witness satisfied the universal instance | `decider.md:110`, test `attack_zeroed_constant_wire_satisfies_universal_instance` | pinned constant (1) |
| code distance unproven — minimum weight 1 is all that is proven, so `num_queries = 100·m` opens every column | lens#6, `lens/specs/scalar-field.md:28-32` | Reed–Solomon (2) |
| base-field challenges — 64-bit [[Goldilocks field|Goldilocks]], no 128-bit claim possible | `zheng/specs/execution.md:75` | [[fp3]] challenges (1) |
| λ counted in elements, not bits | `recursive-brakedown.md:244` | soundness ledger (1) |

the owner's 2026-09-11 fallback — derive the relation on the verifier, disclose the witness, check every row — is sound and stays as the `public` profile.

## 3. the design

seven rules, each a removal:

1. one field — [[Goldilocks field|Goldilocks]] ([[nebu]]) for everything committed; its cubic extension [[fp3]] ([[extension-fields]]) for every challenge and evaluation point.
2. one hash — [[hemera]] ([[Poseidon2]] over Goldilocks) for Merkle trees, Fiat–Shamir and program digests. the only assumption: hemera is a random oracle ([[hash function selection]]).
3. one code — Reed–Solomon over Goldilocks (NTT in nebu, [[polynomial-arithmetic]]). no expander codes, no conjectured distance.
4. one IOP — [[SuperSpartan|Spartan]] [[sumcheck]] over [[CCS]] of any degree (zheng#45).
5. one accumulation — ARC: hash-based, unbounded depth, up to list-decoding radius, random oracle only ([2024/1731](https://eprint.iacr.org/2024/1731)); WARP ([2025/753](https://eprint.iacr.org/2025/753)) as the linear-time fallback. it replaces [[folding]] by homomorphism, which the stack never had.
6. one wire format, three profiles — `public` (disclosed witness, linear check), `succinct` (committed witness, ≤ 64 KB), `zk` (succinct + VEIL masking, [2026/683](https://eprint.iacr.org/2026/683.pdf); the stack's [[zero knowledge]] page). one magic, one version, one profile byte, carried by [[tade]].
7. the verifier derives everything it checks — the relation from the program, the public prefix from the statement — and trusts nothing from the prover but field elements and hashes ([[verification]]).

| layer | owns | repo |
|---|---|---|
| relation | program + subject shape → [[CCS]]; public prefix `(1 ‖ io ‖ cycles)`; jets as rows; `vk = hemera(matrices)` | [[zheng]] `execution/relation`, [[nox]] semantics |
| IOP | [[SuperSpartan|Spartan]]: outer and inner [[sumcheck]], one evaluation claim per matrix family | zheng `spartan` |
| commitment | multilinear → RS codeword → hemera Merkle; opening at an fp3 point by the phase-2 winner | [[lens]] |
| accumulation | uniform step relation; ARC folds evaluation claims; decider = one opening of the [[zheng/specs/accumulator|accumulator]] | zheng `accumulate` (replaces `folding`) |
| statement & wire | profile, program digest, io, cycles, budget; serialisation; versioning | [[joy]] `rs/execution` + zheng `execution/statement` |
| state certificates | [[bbg]] `QueryProof` rides the winning PCS | bbg |
| second verifier | the whole verifier in [[trident]] on nox, agreeing with Rust on every fixture | trident `lib/std/zheng` |
| settlement | tickets, self-fold, cluster tree and root decider of [[fold mining]] ride the accumulation of §3 and the recursion of §4 | [[foculus]], [[tok]] |

the proof, succinct profile: relation (prover and verifier compile the same CCS; only `w` is committed) → `vk` (the verifier recompiles, hashes once, absorbs the digest) → commit (`w` as a multilinear polynomial, RS-encoded, one hemera root) → IOP (Spartan with fp3 challenges, one claim `w̃(r) = v`) → open (Johnson-bound parameters, 128 bit) → wire (statement · program · profile · root · sumcheck polynomials · evaluations · opening).

byte budget for one hash (`n = 2¹⁰`, degree-7 CCS, 128-bit Johnson, fp3 at 24 B), WHIR as the worked example: statement 0.3 KB · Spartan 3 KB · opening 28–36 KB · final polynomial 0.5 KB → 32–40 KB expected, 64 KB the ceiling; 16–20 KB at 100 bit. verify ≈ 2,000 hemera permutations ≈ 0.5–1 ms. at `n = 2²⁰`: 56–70 KB, WHIR's published number.

unbounded programs: a uniform step relation (one CCS for one nox reduction step with continuity and memory arguments, `zheng/audit/general-nox-relation-review.md:160-173`); each step yields one proximity claim; ARC folds claims into one accumulator of fixed size (one root, one fp3 point, one value, a few cosets of openings per step); one opening decides it. proof size is independent of length only once each step's openings are checked inside the next step (IVC, §4); without it the proof grows per segment.

## 4. recursion — what it is here, and where it is needed

two words that the old specs ([[zheng/specs/recursion|recursion]], [[zheng/specs/accumulator|accumulator]]) used as one:

- accumulation — folding *claims* without verifying *proofs*: each step batches the claims of its input words into one accumulator — one root and a fixed number of claims, whatever the depth — and one decider opens the last accumulator. the accumulator is constant; the proof is not. every step leaves spot-check openings in every word it folded, and without a verifier for them inside the computation the final verifier checks every step: bytes and verify time grow per segment. measured on zheng#51 (`zheng/audit/accumulation-2026-10.md`): ~96 KB per 2¹⁴-row segment (~8 KB of AIR messages, ~88 KB of accumulation openings), hash.tri verifies in 5.4 ms, three segments in 40 ms; 512 statements fold into one 35 KB decider but 8.9 MB with their per-step proofs. accumulation alone saves openings, not size. it is the default step.
- recursion — a proof that verifies a proof: [[IVC]] and [[PCD]] proper. constant size comes from IVC: the accumulation verifier — not the whole proof verifier — runs inside the next step's relation, so each step's openings are checked in-relation and consumed, and what travels is one accumulator plus one decider, independent of length (ARC §2.3, WARP §1). this, and not accumulation alone, is how "any computation" is proven at constant size; it is work package R1 of the release plan, in progress. the full [[verification|verifier]] as a nox program, `verify(verify(π))`, does not shrink a single proof in a hash-only world — the outer proof carries its own Merkle paths again — so it is a tool for *composition*.

where the stack needs recursion proper, and what each costs:

| need | shape | cost, with the §3 numbers |
|---|---|---|
| [[fold mining]]: a cluster of tickets becomes one [[decider|decider]] before [[tok]] mints | each ticket is a proof of `m(n)`; the miner's self-fold and the cluster tree are accumulation steps; the root is one decider. `σ_f` per fold step is an accumulation step, not a proof of a proof — but its openings are checked by the next step only under IVC, otherwise by the settler, step by step | without IVC: one decider per cluster plus every step's openings — 512 statements 8.9 MB, verify 1.17 s at test parameters (zheng#51). ≤ 64 KB per cluster needs IVC |
| a [[light client]] accepts a checkpoint over many [[epoch|epochs]] | accumulate per epoch with IVC, decide at the checkpoint; the client verifies one accumulator and one decider | constant with IVC; without it linear in epochs |
| composition across domains, or across a version boundary (an old proof inside a new one) | recursion: the Trident verifier of §3 run inside nox | ≈ 2,000 hemera permutations ≈ 1.2M constraints ≈ a 2²⁰–2²¹-row relation; one step ≈ 56–70 KB and seconds of proving — affordable once, never per step |
| self-hosting: the proof that nox proves nox ([[cyber/launch\|launch]] row 45, "the self-hosting proof is a fold") | accumulation with IVC over the self-hosting trace; full recursion only to close the loop at the end | one decider plus at most one recursion step |
| the fallback if ARC's prover is too slow or its argument does not close on review | bounded-depth recursion through the Trident verifier, one level at a time | costlier, standard, sound; keeps the goal, not the elegance |

so IVC — the accumulation verifier as part of the step relation — is built in phase 3 and is what makes size constant: one step at `ℓ = 20` checks 3 × 53 leaves with ~15-level paths, ≈ 3,000 hemera permutations plus byte decompositions, a ≈ 2¹⁷-row relation per step before the segment's own rows (zheng audit §7). full recursion is built once — as the second verifier of §3, in Trident — and used for composition and as the fallback; accumulation does the daily work, inside IVC. the two numbers the old specs carried for recursion, "~825 constraints per level" and "1000 levels → 2⁻¹¹⁸", described a nox verifier that was never built and are retired in §C.

## 5. phases and gates

every gate is a fixture and a command, not an opinion. a phase closes when its row is green on CI.

| phase | what | gate | repos |
|---|---|---|---|
| 0 ✓ | relation shrink: linear forms, native constants, degree-7 S-box (zheng#45) | hash.tri 294,861 → 15,608 B; prove/verify 460 → 36/30 ms; 259 tests | zheng |
| 1 | soundness floor: public prefix + pinned constant + `vk` digest + fp3 challenges + one format with a profile byte + the soundness ledger | `public` hash.tri ≤ 10 KB · fixtures `forged-io`, `forged-vk`, `zeroed-constant`, `meaningless-witness` rejected · ledger has no "conjectured" row | zheng, joy, [[strata]] |
| 2 | bake-off: small class — TensorMerkle+RS+fp3, WHIR, SmallWood; large class — WHIR, Ligerito/DeepFold; same fixtures, same ledger; one PCS per class ships, the rest is deleted; zip measured on the winner | small (hash.tri, a transfer): ≤ 16 KB, verify ≤ 1 ms · large (`n = 2²⁰`): ≤ 64 KB, ≤ 1 ms · bit-flip scan clean · bbg `QueryProof` migrated | lens, zheng, bbg |
| 3 | uniform step relation + ARC + decider + IVC (the accumulation verifier inside the step relation, §4); lattice-fold spike measured first (§B); the Trident verifier as a nox program; one recursion fixture | merkle-32 and a 10⁶-step run both prove · size independent of length, ≤ 64 KB · verify ≤ 1 ms · `verify(verify(π))` at depth 2 agrees across Rust and Trident · a [[fold mining]] cluster of 512 tickets decides in one proof | zheng, nox, trident, foculus |
| 4 | `zk` profile (VEIL) | zk fixtures; P1 and P3 of [[cyber/launch\|launch]] unblocked | zheng |
| 5 | delete: brakedown, folding, legacy formats; close the ledger of stale claims (§C) | proving path ≤ 12k lines by `tokei` · `nu scripts/stale-proof-claims.nu` returns 0 hits across the workspace | lens, zheng, joy, nox, bbg, hemera, foculus, tok, cyber, crystal |

phases 1 and 2 are a month each; 3 is research-grade and measures before it commits; 4 and 5 are weeks. until 3 lands, `succinct` covers one relation of ≤ 2¹⁵ rows, and the "any computation" row of §0 is open.

## 6. how it stays sound

- soundness ledger (`zheng/specs/soundness.md`): one row per component — assumption, bits claimed, proven or conjectured, paper, controlling parameter. the release gate fails on any "conjectured" row in a production profile.
- attack fixtures that must be rejected: meaningless witness, zeroed constant, forged io, forged `vk`, truncated path, query-index replay, challenge reuse. each one a release gate.
- bit-flip scan over every byte of every fixture proof — the method that found the 2,080 dead bytes.
- differential against native: circuit digests against `nox::data::hash`, outputs against `nox::reduce`.
- two verifiers, Rust and Trident, agreeing on every fixture.
- frozen fixtures: a proof produced for a release stays accepted by every later verifier, or the format version moves.
- security parameters are data — queries, rate, grinding, extension degree in one struct, printed in the header, checked against the verifier's policy.

## 7. what disappears

`zheng/folding` (HyperNova over hemera) → `zheng/accumulate` (ARC). the lens loser of the bake-off; `UnsupportedRecursiveOpening`; the flat-hash `Tensor` opening; expander codes, `MIN_ABS_WEIGHT`, `num_queries = 100·EXPANSION·k2`. formats `zheng-hypernova-tensor-merkle-v2`, `JOYEXEC1`, `JOYEXEC2`, `JOYZK003` → one format, read by a `legacy` tool for one release. the experimental kernels on `release/0.4` leave the production path when `zk` lands. `PublicTensor` stays as the `public` profile. the proving path — relation, IOP, commitment, accumulation, wire — fits in ~12k lines against ~25k today.

## 8. open, and who closes it

| question | closed by |
|---|---|
| which PCS per class | phase 2, on numbers |
| ARC prover cost per step vs a Neo-style lattice fold (§B) | the phase-3 spike, on numbers |
| zip's measured gain on the winner | phase 2 |
| in-proof digest length under hemera profile v2 | decided: 32 B in trees (§B) |
| whether [[fold mining]]'s `σ_f` stays a per-step proof or becomes an ARC step | phase 3, with foculus |

## 9. release — this proposal as the first run of the train

the repair touches nine repositories and every product that verifies a proof. it is therefore the first change shipped end to end by the [[cyberia/dev|release train]]: candidates cut from origin on fridays, gates executable, bumps as pull requests, promotion by the owner only. the proposal is the test of that process as much as of the proofs.

propagation order — each row pins the one above it; a bump is one PR `chore: <component> <version>` touching `Cargo.toml`, `CHANGELOG.md` and the sibling pins:

| order | component | today | phase 1 | phase 2 | phase 3 | what changes for its dependents |
|---|---|---|---|---|---|---|
| 1 | [[strata]] / [[nebu]] | — | fp3 exposed | — | — | the challenge field |
| 2 | [[lens]] | brakedown 0.2.0 | — | 0.3: RS code, the winning opening, `QueryProof` API | — | commitment and opening types |
| 3 | [[zheng]] | 0.4.0 | 0.5: soundness floor, one format, soundness ledger | 0.6: `succinct` profile | 0.7: `accumulate`, decider; 0.8: `zk` | proof format, verifier API |
| 4 | [[joy]] | 0.5.0 | 0.6: statement model with the profile byte | 0.7 | 0.8 | the wire |
| 5 | [[nox]] | 0.3.0 | — | — | 0.4: uniform step relation; decider jet retired | the step relation |
| 6 | [[bbg]] | — | — | `QueryProof` on the winning PCS | — | state certificates |
| 7 | [[foculus]] · [[tok]] | 0.1.3 · — | — | — | tickets, self-fold and cluster tree on ARC; `σ_f` decided (§8) | settlement |
| 8 | [[trident]] | 0.3.0 | — | — | `lib/std/zheng`: the second verifier | the recursion seed |
| 9 | [[soft3]] | 0.10.0 | 0.11 | 0.12 | 0.13 | the node |
| 10 | [[cyber]] · cyb | — | docs rows of the ledger | docs | docs | the products |

what each soft3 version means: 0.11 — every proof the node accepts is sound (the public profile, fp3, bound statements); 0.12 — succinct proofs ≤ 16 / 64 KB on the wire, the first version a phone verifies in a millisecond; 0.13 — any computation at constant size, settlement on accumulation. the next soft3 version is 0.11 and it ships phase 1 alone.

the train's rules apply unchanged: one candidate per friday from `origin/main` of every repo in the closure; `sources.json` and receipts in `<repo>/audit/release-<date>/`; freeze from cut to verdict; a red gate ships as a red candidate; the owner merges bumps, promotes, publishes. two gates join the train's set for this work: the soundness ledger has no "conjectured" row on the production profile (from 0.11), and `scripts/stale-proof-claims.nu` returns zero (from 0.13). the launch page gets one row per candidate.

## A. decisions recorded, with the argument compressed

the Merkle question. the owner asked whether the trees can go. they cannot: a flat hash `C = hemera(Enc(w))` has no local opening, and the column check `Enc(y)[j] = q₁ᵀ·col_j` is satisfiable by a prover who never touches `C` (pick `y'`, solve one linear equation per queried column). in a hash-only world a local opening costs Θ(log n) digests — a lower bound, not a layout; every hash-based system carries paths ([[merklezation]], [[hash chain]]). the only escape is a homomorphism, i.e. lattices, at ~50 KB. the sound half of the old idea — Ligero geometry, RS rows, a column tree, `y` in the clear — is today's `TensorMerkle` and is a bake-off candidate; at `n = 2¹⁰` it and WHIR land in the same 25–40 KB band, at `2²⁰` folding wins. no version brings back `C = hemera(w)` or 1.3 KB.

the levers on the trees, from ~36 KB for one hash: a code with a proven distance (queries from every column to ~64 — the largest win); path deduplication −30…45 %; rate 1/32 −20 % at ×2 prover; grinding 20 bits −12 %; folding factor 5 −10 %; fp3 at 24 B −3 %. honest floor 25–40 KB at 128 bit; 15–20 KB if the levers compose; zip ×0.6 on top of either.

## B. lattices and the frontier, 2026-10-09

what lattices buy is the shape of accumulation, not the final byte count: a homomorphic fold with a tiny in-flight object (Neo/SuperNeo [2025/294](https://eprint.iacr.org/2025/294.pdf), LatticeFold+ [2025/247](https://eprint.iacr.org/2025/247.pdf), Symphony [2025/1905](https://eprint.iacr.org/2025/1905), PikkuFold ~5.7 KB per step [2026/1809](https://eprint.iacr.org/2026/1809.pdf)) at the price of a second assumption ([[Module-SIS]]) and norm bookkeeping. LaBinius ([2026/2103](https://eprint.iacr.org/2026/2103.pdf)) now wins the bytes at `2²⁴` — 82.9 KiB against WHIR's 300.9 at rate ¼ — and loses the verifier 650× (713 ms against 1.1). under the goal as fixed, hash-only stands; the spike's one question is whether any lattice verifier gets under 10 ms. the binary-tower track ([[Binius]], `lens/specs/binary-tower.md`) is the wrong field for a Goldilocks stack and stays out.

zip ([2025/1446](https://eprint.iacr.org/2025/1446)): black-box compression of hash-based proofs to ~60 %, standard assumptions. it composes with every lever because it acts on the finished proof; phase 2 measures it.

the in-proof digest: hemera profile v2 ([hemera#15](https://github.com/cyberia-to/hemera/pull/15)) raises the identity digest to 48 or 64 bytes for post-quantum collision resistance. a Merkle node inside a proof is ephemeral — it must be forged before the verifier runs, so "harvest now, break later" does not apply — and stays 32 bytes. one permutation, two squeeze lengths; a 48-byte node would add 50 % to the dominant term for nothing.

what this proposal does not claim: no 2 KB, no 100 ns, no "Merkle-free", no proof smaller than the authentication of its own queries.

## C. the ledger of stale claims

every place in the stack that still presents the old design as current — ~2 KB proofs, ~5 μs verification, "Merkle-free" Brakedown, HyperNova over hemera, the ~825/~89-constraint decider, algebraic Fiat–Shamir, "the accumulator is the proof" — swept on 2026-10-09 across every repository under `~/cyber` (archives, vendored trees and `zheng-pin/`, a worktree of zheng, excluded). one row per document; the line numbers are in the sweep's output and in the script.

the rule is executable: `scripts/stale-proof-claims.nu` greps the workspace for every family of stale claim and exits 1 on any hit outside a short allow-list of dated posts and changelogs (736 raw hits in 214 files today). it is the phase-5 gate and, from the 0.13 candidate on, a gate of the release train. each earlier phase closes its own rows — phase 1 the security claims, phase 2 the sizes and the Merkle-free pages, phase 3 the fold, decider and recursion pages — so the table shrinks with the work.

actions: delete — built on the old design, no other content · superseded — keep with a banner pointing here · rewrite — the section is rewritten against §3 · number — the figure becomes the measured or target one (≤ 16 KB small, ≤ 64 KB any, ≤ 1 ms) · code — goes with the phase-5 code removal · leave — dated post or audit snapshot, erratum link only.

### zheng

| document | stale claim | action | phase |
|---|---|---|---|
| [[soft3/zheng/docs/explanation/recursive-brakedown|zheng/docs/explanation/recursive-brakedown.md]] | "Merkle-free lens, zero hash trees", "≈1.3 KiB", "~5 μs", "error ≤ 5·2⁻¹²⁸" | delete (or research/ with an "unsound" banner) | 5 |
| [[soft3/zheng/specs/verifier|zheng/specs/verifier.md]] | "no Merkle verification … ~2 KiB"; ~825/~89 tiers; "verify(verify(proof)) to arbitrary depth"; "~5 μs … ~3 hemera calls"; "~30 field ops + 1 hemera per fold" | superseded; verifier spec rewritten in phase 2 | 2 |
| [[soft3/zheng/specs/api|zheng/specs/api.md]] | "~2 KiB at 128-bit", "Brakedown … Merkle-free", decide ~825 | superseded | 2 |
| [[soft3/zheng/specs|zheng/specs/README.md]] | HyperNova ~30 ops/fold, Merkle-free, ~660 ops ~5 μs, ~1.3 KiB, decide ~825 | rewrite | 2 |
| [[soft3/zheng/specs/decider|zheng/specs/decider.md]] | "~825 constraints — same whether N is 1 or 1,000,000"; "~2.4 KiB" | superseded (keep residuals) | 3 |
| [[soft3/zheng/specs/recursion|zheng/specs/recursion.md]] | fold formula, "accumulator IS the proof", "~5 μs", "1000 levels → 2⁻¹¹⁸" | superseded; line 180 deleted | 3 |
| [[soft3/zheng/specs/accumulator|zheng/specs/accumulator.md]] | "~200 bytes", "10–50 μs", "~30 field ops + 1 hemera" | superseded | 3 |
| [[soft3/zheng/specs/transcript|zheng/specs/transcript.md]] | "~3 hemera calls … algebraic Fiat–Shamir", "~2 KiB", "256-bit classical / 170+ PQ", "kd/p negligible" | rewrite | 1 |
| [[soft3/zheng/specs/constraints|zheng/specs/constraints.md]] | "folded sub-proof ~825 constraints" | number | 3 |
| [[soft3/zheng/specs/proof-types|zheng/specs/proof-types.md]] | "verify(π₀) → π₁ (~100–200 KB)", "290 μs, ~157 KiB" | number | 2 |
| [[soft3/zheng/specs/tensor|zheng/specs/tensor.md]] | "phone folds each step incrementally" | rewrite | 3 |
| [[soft3/zheng/docs/explanation/zheng-vs-starks|zheng/docs/explanation/zheng-vs-starks.md]] | "computation IS proving", "240-byte checkpoint", "8.7×", "144K → 0" | delete or rewrite | 5 |
| [[soft3/zheng/docs/explanation/polynomial-commitments|zheng/docs/explanation/polynomial-commitments.md]] | "Brakedown is Merkle-free" | rewrite | 2 |
| [[soft3/zheng/docs/explanation/whirlaway|zheng/docs/explanation/whirlaway.md]] | "eliminates the Merkle tree bottleneck entirely" | rewrite banner + section | 2 |
| [[soft3/zheng/docs/explanation/fri-to-whir|zheng/docs/explanation/fri-to-whir.md]] | "recursive Brakedown made it Merkle-free" | rewrite banner + section | 2 |
| [[soft3/zheng/docs/explanation/whir|zheng/docs/explanation/whir.md]] | banner; "290 μs" | rewrite banner | 2 |
| [[soft3/zheng/docs/explanation/performance|zheng/docs/explanation/performance.md]] | "~60 KiB / ~290 μs", "~157 KiB ~1.0 ms" | number | 2 |
| [[soft3/zheng/docs/explanation/landscape|zheng/docs/explanation/landscape.md]] | "recursive Brakedown, the current frontier" | rewrite | 2 |
| [[soft3/zheng/docs/explanation/superspartan|zheng/docs/explanation/superspartan.md]] | "recursive Brakedown is the right choice" | rewrite | 2 |
| [[soft3/zheng/docs/explanation/why-zheng|zheng/docs/explanation/why-zheng.md]] | "recursive Brakedown is the PCS" | rewrite | 2 |
| [[soft3/zheng/docs/explanation/stark|zheng/docs/explanation/stark.md]] | "SuperSpartan + recursive Brakedown = current architecture" | rewrite | 2 |
| [[soft3/zheng/docs/explanation/recursion|zheng/docs/explanation/recursion.md]] | "1000 field ops + 1000 hashes ≈ microseconds" | rewrite | 3 |
| [[soft3/zheng/docs/explanation/security|zheng/docs/explanation/security.md]] | "256-bit classical / 170-bit quantum", "kd/p negligible", "< 2⁻¹²⁸" | rewrite (the soundness ledger) | 1 |
| [[soft3/zheng/docs/explanation/sumcheck|zheng/docs/explanation/sumcheck.md]] | "Hemera provides 128-bit security" | number | 1 |
| [[soft3/zheng/CLAUDE|zheng/CLAUDE.md]] | "fastest PCS verification (290 μs – 1.0 ms)" | number | 2 |
| [[soft3/zheng/CHANGELOG|zheng/CHANGELOG.md]] | "proof size is now a constant (~2.4 KiB + ~1.7 KiB)" | leave (historical) | — |
| [[soft3/zheng/roadmap/gravity-commitment|zheng/roadmap/gravity-commitment.md]] | "~1 KiB / ~10 μs" on recursive Brakedown | superseded | 5 |
| [[soft3/zheng/roadmap/ring-aware-fhe|zheng/roadmap/ring-aware-fhe.md]] | "~30 field ops per fold" | rewrite | 5 |
| [zheng/.claude/plans/release-plan.md](https://github.com/cyberia-to/zheng/blob/main/.claude/plans/release-plan.md) | "~2 KiB, ~5 μs, ~825", "~200 bytes", algebraic FS | superseded | 5 |
| [zheng/.claude/plans/axis-verifier-integration.md](https://github.com/cyberia-to/zheng/blob/main/.claude/plans/axis-verifier-integration.md) | "~2 KiB per proof", "~825" | superseded | 5 |
| [zheng/.claude/plans/minimal-zheng-structure-and-cli.md](https://github.com/cyberia-to/zheng/blob/main/.claude/plans/minimal-zheng-structure-and-cli.md) | "~2 KiB spec target" | number | 5 |
| [zheng/rs/src/types.rs](https://github.com/cyberia-to/zheng/blob/main/rs/src/types.rs) | `/// ~2 KiB at 128-bit security for N = 2^20.` | code | 2 |
| [zheng/rs/src/transcript.rs](https://github.com/cyberia-to/zheng/blob/main/rs/src/transcript.rs) | `/// negligible at 128-bit security.` | code (fp3) | 1 |
| [zheng/rs/src](https://github.com/cyberia-to/zheng/tree/main/rs/src): `folding/{mod,fold,decide}.rs`, `types.rs`, `lib.rs`, `ccs/universal.rs`, `phi/mod.rs` | "HyperNova CCS folding" | code | 5 |
| [zheng/rs/Cargo.toml](https://github.com/cyberia-to/zheng/blob/main/rs/Cargo.toml) | "SuperSpartan IOP + Brakedown PCS + sumcheck" | number (after the bake-off) | 2 |

### lens

| document | stale claim | action | phase |
|---|---|---|---|
| [[soft3/lens|lens/README.md]] | "hemera hashes the codeword → 32-byte commitment … 20 positions" | rewrite | 2 |
| [[soft3/lens/specs/commitment|lens/specs/commitment.md]] | "~1.3 KiB proof, ~660 field ops", "recursive tensor decomposition", "d 20–30 for 128-bit" | number + rewrite | 2 |
| [[soft3/lens/specs/binary-tower|lens/specs/binary-tower.md]] | "~30 ops + 1 hemera per fold" | rewrite | 3 |
| [[soft3/lens/specs/polynomial-ring|lens/specs/polynomial-ring.md]] | "~30 field ops per fold" | rewrite | 3 |
| [lens/.claude/plans/release.md](https://github.com/cyberia-to/lens/blob/main/.claude/plans/release.md) | "Brakedown opening verifier as a CCS instance (~825)" | superseded | 5 |
| [lens/brakedown/](https://github.com/cyberia-to/lens/blob/main/brakedown/) · [lens/ikat/](https://github.com/cyberia-to/lens/blob/main/ikat/) · [lens/porphyry/` `Cargo.toml](https://github.com/cyberia-to/lens/blob/main/porphyry/` `Cargo.toml) | "expander-graph codes" | number (with the bake-off) | 2 |

### nox

| document | stale claim | action | phase |
|---|---|---|---|
| [[soft3/nox/specs/jets/decider|nox/specs/jets/decider.md]] | "89 constraints ≈ 100 nanoseconds", "~200 bytes", "240 bytes" | delete or superseded | 3 |
| [[soft3/nox/specs/jets/recursion|nox/specs/jets/recursion.md]] | "Brakedown (Merkle-free PCS) … ~825" | rewrite | 3 |
| [[soft3/nox/specs/jets|nox/specs/jets.md]] | "Merkle-free … ~825 … ~89 … recursion to arbitrary depth" | rewrite | 3 |
| [[soft3/nox/specs/jets|nox/specs/jets/README.md]] | "decider … 89 constraints" | number | 3 |
| [[soft3/nox/specs/trace|nox/specs/trace.md]] | "no Merkle paths", "~825 / ~89", "~2 KiB … ~30 field ops + 1 hemera" | rewrite | 3 |
| [[soft3/nox/specs/reduction|nox/specs/reduction.md]] | "each reduce() folds into HyperNova", "accumulator IS the proof", "~3 calls", "10–50 μs" | rewrite | 3 |
| [[soft3/nox/specs/jets/state|nox/specs/jets/state.md]] | "folds into accumulator (~30 field ops)" | rewrite | 3 |
| [[soft3/nox/specs/jets/polynomial-ring|nox/specs/jets/polynomial-ring.md]] | "HyperNova folds the F₂ sub-trace" | rewrite | 3 |
| [[soft3/nox/specs/vm|nox/specs/vm.md]] | "~766 constraints per type transition" | number | 3 |
| [[soft3/nox/roadmap/decider-product|nox/roadmap/decider-product.md]] | "~200 bytes … 89 constraints, ~100 ns", "Brakedown is Merkle-free" | delete or superseded | 5 |
| [[soft3/nox/roadmap/strata-collapse|nox/roadmap/strata-collapse.md]] | "cross-algebra composition via HyperNova folds" | rewrite | 5 |
| [[soft3/nox/docs/explanation/decider|nox/docs/explanation/decider.md]] | "all history in 89 constraints … 240 bytes … 100 nanoseconds" | delete | 3 |
| [[soft3/nox/docs/explanation/five-algebras|nox/docs/explanation/five-algebras.md]] | "universal accumulator (~200 bytes) … 89 constraints" | rewrite | 3 |
| [[soft3/nox/docs/explanation/self-verification|nox/docs/explanation/self-verification.md]] | "Merkle-free … ~825 / ~89 … ~2 KiB per level" | rewrite | 3 |
| [[soft3/nox/docs/explanation/jets|nox/docs/explanation/jets.md]] | "Merkle-free … all-history verification in 89 constraints" | rewrite | 3 |
| [[soft3/nox/docs/explanation/layers|nox/docs/explanation/layers.md]] | "Brakedown (Merkle-free PCS) … ~825" | rewrite | 3 |
| [[soft3/nox/docs/explanation|nox/docs/explanation/README.md]] | "decider.md — 89 constraints" | with the page | 3 |
| [[soft3/nox/docs/explanation/why-nox|nox/docs/explanation/why-nox.md]] | "1 trillion txs → 1 proof (~100 KiB)" | number | 3 |
| [nox/.claude/plans/jet-registry-0.1.md](https://github.com/cyberia-to/nox/blob/main/.claude/plans/jet-registry-0.1.md) | "89/825-constraint verifier" | superseded | 5 |
| [nox/rs/jets/decider.rs](https://github.com/cyberia-to/nox/blob/main/rs/jets/decider.rs) | `//! 89 primary + 825 cross-term constraints` | code (the jet verifies nothing; launch #40) | 3 |
| [nox/rs/jets/formulas.rs](https://github.com/cyberia-to/nox/blob/main/rs/jets/formulas.rs) | `/// Full 89/825-constraint verification` | code | 3 |

### bbg

| document | stale claim | action | phase |
|---|---|---|---|
| [[soft3/bbg|bbg/README.md]] | "~5 μs via zheng-2 folding", "~2 KiB (recursive Brakedown)" | number | 2 |
| [[soft3/bbg/specs/architecture|bbg/specs/architecture.md]] | "~5 μs", "~75 bytes", "~2 KiB", "~240 bytes", "~200 bytes per namespace" | number | 2 |
| [[soft3/bbg/specs/data-availability|bbg/specs/data-availability.md]] | "~75 bytes (recursive Brakedown)", "O(λ log log N)" | rewrite | 2 |
| [[soft3/bbg/specs/state|bbg/specs/state.md]] | "~2 KiB, ~5 μs" | number | 2 |
| [[soft3/bbg/specs/indexes|bbg/specs/indexes.md]] | "O(λ log log N) … ~5 μs", "~200 bytes per opening" | number | 2 |
| [[soft3/bbg/specs/privacy|bbg/specs/privacy.md]] | "~240 bytes constant", "~2 KiB" | number | 2 |
| [[soft3/bbg/specs/neuron-state|bbg/specs/neuron-state.md]] | "~5 μs", "~2 KiB", "~240 bytes", "~200 bytes regardless of batch" | number | 2 |
| [[soft3/bbg/specs/temporal|bbg/specs/temporal.md]] | "~5 μs" | number | 2 |
| [[soft3/bbg/roadmap/verifiable-query|bbg/roadmap/verifiable-query.md]] | "~5 μs", "~1 / ~5 / ~2 KiB" | number | 2 |
| [[soft3/bbg/docs/explanation/architecture-overview|bbg/docs/explanation/architecture-overview.md]] | "240-byte checkpoint … ~5 μs", "accumulator IS the proof", "~3 hemera calls" | rewrite | 3 |
| [[soft3/bbg/docs/explanation/why-signal-first|bbg/docs/explanation/why-signal-first.md]] | "~240 bytes", "~5 μs", "~2 KiB" | number | 2 |
| [[soft3/bbg/docs/explanation/why-polynomial-state|bbg/docs/explanation/why-polynomial-state.md]] | "~200 bytes", "10–50 μs", "~5 μs" | number | 2 |
| [[soft3/bbg/docs/explanation/polynomial-privacy|bbg/docs/explanation/polynomial-privacy.md]] | "O(1) verification, ~200 bytes" | number | 2 |
| [[soft3/bbg/docs/explanation/data-availability|bbg/docs/explanation/data-availability.md]] | "~200 bytes per sample" | number | 2 |
| [[soft3/bbg/docs/explanation/signal-sync|bbg/docs/explanation/signal-sync.md]] | "~40,000 constraints verifiable in ~5 μs" | number | 2 |
| [bbg/.claude/plans/pattern17-look-integration.md](https://github.com/cyberia-to/bbg/blob/main/.claude/plans/pattern17-look-integration.md) | "Brakedown opening (~825) is a folded sub-instance" | superseded | 5 |
| [[soft3/bbg/CLAUDE|bbg/CLAUDE.md]] | "Brakedown evaluation proofs" | number (with the bake-off) | 2 |

### hemera

| document | stale claim | action | phase |
|---|---|---|---|
| [[soft3/hemera/roadmap/algebraic-fiat-shamir|hemera/roadmap/algebraic-fiat-shamir.md]] | "8.7× fewer hemera calls" | delete | 1 |
| [[soft3/hemera/roadmap|hemera/roadmap/README.md]] | algebraic FS row; "~3 calls per execution"; "ZERO hemera calls", "Brakedown is Merkle-free", "144K to 0", "each permutation = one fold step" | rewrite | 2 |
| [[soft3/hemera/specs|hemera/specs/README.md]] | "one Hemera call for binding hash" | rewrite | 2 |
| [[soft3/hemera/specs/tree|hemera/specs/tree.md]] | "~75 bytes of proof, O(1) random access" | number | 2 |

### soft3

| document | stale claim | action | phase |
|---|---|---|---|
| [[soft3/docs/polynomial-proof-system|soft3/docs/polynomial-proof-system.md]] | the whole old design with its numbers | delete (fix the link in the meantime) | 5 |
| [[soft3/docs|soft3/docs/README.md]] | "~200 bytes", "~30 field ops … 100 nanoseconds", "microseconds" | rewrite | 2 |
| [[soft3/specs/terms|soft3/specs/terms.md]] | "~200-byte query proofs" | number | 2 |
| [[soft3/status|soft3/status.md]] | "~200B proofs" | number | 2 |
| [[soft3/specs/languages|soft3/specs/languages.md]] | "HyperNova folds all partitions" | rewrite | 3 |

### cyber — site, whitepaper, research

| document | stale claim | action | phase |
|---|---|---|---|
| [[cyber/whitepaper|cyber/whitepaper.md]] | "expander-graph codes, HyperNova folding"; "each level constant (~100–200 KB)"; "O(1) global state (~22kb)"; "≥ 1 − 2⁻¹²⁸ by zheng soundness"; "fold of winning tickets" | rewrite §9.3, number §18, soundness-ledger wording, ARC in §14.3 | 2–3 |
| [[cyber/whitepaper|cyber/whitepaper.md]] | SNARK ~200 B vs zheng ~100–200 KB (comparison table) | number (≤ 64 KB target) | 2 |
| [[cyber/light|cyber/light.md]] | "one proof of ~100–200 KB" | number | 2 |
| [[cyber/network|cyber/network.md]] | "one recursive proof (~100–200 KB)" | number | 2 |
| [[cyber/communication|cyber/communication.md]] | "after recursive composition: ~100–200 KB" | number | 2 |
| [[cyber/launch|cyber/launch.md]] | "one O(1) accumulator by HyperNova folding" | rewrite (ARC) | 3 |
| [[cyber/security|cyber/security.md]] | "≥ 1 − 2⁻¹²⁸" | rewrite | 1 |
| [[cyber/restructure|cyber/restructure.md]] | "continuous fold / ~30 field ops / ~200-byte accumulator" | number | 3 |
| [[cyber/research/algorithmic essence of superintelligence|cyber/research/algorithmic essence of superintelligence.md]] | "~2 KiB", "~5 μs", "~3 calls", "recursive Brakedown" | superseded | 5 |
| [[cyber/research/universal law|cyber/research/universal law.md]] | "~2 KiB, ~50 μs", "~30 field operations" | number | 5 |
| [[cyber/research/bbg|cyber/research/bbg.md]] | "~200 bytes, 10–50 μs", "Merkle-free lens … 144,000", "240 bytes" | superseded | 5 |
| [[cyber/research/polynomial nouns|cyber/research/polynomial nouns.md]] | "~3 hemera calls", "~200 bytes", "each reduce() folds" | superseded | 5 |
| [[cyber/research/programmable state|cyber/research/programmable state.md]] | "~200 bytes, 10–50 μs" | superseded | 5 |
| [[cyber/research/cybergraph model architecture|cyber/research/cybergraph model architecture.md]] | "10–50 μs", "~30 field ops", "240-byte checkpoint" | superseded | 5 |
| [[cyber/research/egregore properties|cyber/research/egregore properties.md]] | "240-byte accumulator", "10–50 μs" | number | 5 |
| [[cyber/research/algebraic state commitments|cyber/research/algebraic state commitments.md]] | "~200 bytes", "50 μs", "no Merkle tree" | superseded | 5 |
| [[cyber/research/data availability strategy|cyber/research/data availability strategy.md]] | "~200 bytes per sample" | superseded | 5 |
| [[cyber/research/structural-sync|cyber/research/structural-sync.md]] | "HyperNova (~30 ops per step) … 700×" | rewrite | 3 |
| [[cyber/research/provable consensus|cyber/research/provable consensus.md]] | "~50 μs to verify" | number | 3 |
| [[cyber/research/vec formalization|cyber/research/vec formalization.md]] | "~50 μs"; "error ≤ 2⁻⁵¹² at k=16" | number | 3 |
| [[cyber/research/data structures for polynomial state|cyber/research/data structures for polynomial state.md]] | "10 ZB of state in 50 μs" | number | 5 |
| [[cyber/research/five algebras|cyber/research/five algebras.md]] | "~30 field ops per fold" | number | 5 |
| [[cyber/research/nox - frozen provable computer|cyber/research/nox - frozen provable computer.md]] | "ONE HyperNova accumulator" | rewrite | 3 |
| [[cyber/research/256 symbols|cyber/research/256 symbols.md]] | "decider … 89 constraints" | number | 3 |
| [[cyber/research/bootstrap|cyber/research/bootstrap.md]] | "HyperNova folder … 89 constraints" | number | 3 |
| [[cyber/research/unified mining|cyber/research/unified mining.md]] | algebraic NMT openings as real work | review | 3 |

### other repositories

| document | stale claim | action | phase |
|---|---|---|---|
| [[cybics/crystal/architecture|crystal/architecture.md]] | "constant-size global state (~22kb)" | rewrite + number | 3 |
| [[cybics/crystal/stark|crystal/stark.md]] | "~60–200 KB"; "HyperNova accumulator (~200 bytes)" | rewrite | 3 |
| [[cybics/crystal/topoisomerase|crystal/topoisomerase.md]] | "~5μs per step" | number | 2 |
| [[neural/eidos/specs/certificate|eidos/specs/certificate.md]] | "~825 constraints … constant time" | number | 3 |
| [[neural/trident/roadmap/polynomial-target|trident/roadmap/polynomial-target.md]] | "~8K → ~89", "~2 KiB … ~0.1 μs", "~30 ops/fold" | superseded (extend the line-254 banner) | 5 |
| [[soft3/foculus/specs/structural-sync|foculus/specs/structural-sync.md]] | "~2 KiB", "~5 μs", "~200 bytes", "~30 ops" | number | 3 |
| [[soft3/foculus/specs/fold-mining|foculus/specs/fold-mining.md]] | "HyperNova IVC folding … decider O(1)" | rewrite (ARC) | 3 |
| [[soft3/foculus/specs/provable-consensus|foculus/specs/provable-consensus.md]] | "~50 us" | number | 3 |
| [[soft3/foculus/specs/vec|foculus/specs/vec.md]] | "~50 us"; "2⁻⁵¹² at k=16"; "batched folding (IVC)" | number / review | 3 |
| [[soft3/foculus/specs/gossip|foculus/specs/gossip.md]] | "verify σ (tens of microseconds)" | number | 3 |
| [[soft3/foculus/docs/explanation/latency-targets|foculus/docs/explanation/latency-targets.md]] | "Lens ~200 B", "O(1) fold (~30 field ops)" | number | 3 |
| [[soft3/foculus/docs/explanation/life-of-a-signal|foculus/docs/explanation/life-of-a-signal.md]] | "roughly fifty microseconds" | number | 3 |
| [[soft3/foculus|foculus/README.md]] | "HyperNova fold tree" | rewrite (ARC) | 3 |
| [[soft3/foculus/proposals/view-certificates|foculus/proposals/view-certificates.md]] | "π_att HyperNova fold" | review | 3 |
| [foculus/src](https://github.com/cyberia-to/foculus/tree/main/src): `tip`, `tickets`, `marginal_cert`, `pay_proof`, `ticket_proof`, `rewards`, `step`, `epoch` | "HyperNova σ / seal / fold" | code (ARC) | 3 |
| [[soft3/tok/programming-model|tok/programming-model.md]] | "fold into one constant-size proof via HyperNova"; "~200 bytes" | rewrite / number | 3 |
| [[neural/inf/specs/proof|inf/specs/proof.md]] | "~200 bytes"; "~5 μs" | number | 2 |
| [[neural/inf/specs/cost|inf/specs/cost.md]] | "~5 μs, one decider" | number | 2 |
| [[neural/inf|inf/README.md]] · [[neural/inf/docs|inf/docs/README.md]] · [[soft3/cybergraph/docs|cybergraph/docs/README.md]] | "in microseconds" | number | 2 |
| [[soft3/tru/specs/rewards|tru/specs/rewards.md]] | "self-folds using zheng's HyperNova IVC" | rewrite (ARC) | 3 |
| [[soft3/strata/jali/specs/noise|strata/jali/specs/noise.md]] · [[soft3/strata/jali/docs/explanation/lattice-security|strata/jali/docs/explanation/lattice-security.md]] · [[soft3/strata/jali|strata/jali/README.md]] | "~30 field ops per fold"; ring-aware Brakedown | number | 5 |
| [[cybics/crypto/graphy|cybics/crypto/graphy.md]] | "Brakedown — 5 μs verification" | number | 2 |
| [[soft3/soma/soma-spec|soma/soma-spec.md]] | "~5 μs" | number | 2 |
| [fs/sync.md](https://github.com/cyberia-to/fs/blob/main/sync.md) | "ONE zheng proof … (~50 μs)" | number | 3 |
| [[cyb/evy/specs/evy|evy/specs/evy.md]] | "~50μs for BBG recommit + proof" | number | 5 |
| [cyberia-blog/blog/2026_03_24.md](https://github.com/cyberia-to/cyberia-blog/blob/main/blog/2026_03_24.md) · [2026_03_26.md](https://github.com/cyberia-to/cyberia-blog/blob/main/blog/2026_03_26.md) · [2026_03_27.md](https://github.com/cyberia-to/cyberia-blog/blob/main/blog/2026_03_27.md) | "recursive brakedown (the perfect PCS)", "~200 bytes, 10–50 μs" | leave (dated posts) — erratum link | 5 |
| [[warriors/audit/parano1d-vs-uhash-2026-09-27|warriors/audit/parano1d-vs-uhash-2026-09-27.md]] | "HyperNova folding"; "decider ~2.1 KB measured" | leave (audit snapshot) — footnote | — |
| [[warriors/joy/CHANGELOG|joy/CHANGELOG.md]] | "SuperSpartan + Brakedown + HyperNova accumulators" | leave (historical) | — |

### pages already bannered — and whether the banner is enough

| document | banner | enough? |
|---|---|---|
| [[soft3/docs/polynomial-proof-system|soft3/docs/polynomial-proof-system.md]] | "target architecture, not the current lens" | no — the body states the numbers as fact and links `roadmap/` where the page lives in `zheng/docs/explanation/` |
| [[neural/trident/roadmap/polynomial-target|trident/roadmap/polynomial-target.md]] | one inline line | no — the rest of the page is bare |
| [[fri-to-whir]] · [[soft3/zheng/docs/explanation/whir|whir]] · [[whirlaway]] | "historical … zheng has evolved to use recursive Brakedown" | no — the banner points at the unsound design as current |
| [[soft3/zheng|zheng/README.md]] | legacy commit/verify does not authenticate execution | yes |
| [[zheng/specs/decider|zheng/specs/decider.md]] | residuals named | partial — the header still claims ~825 constraints, ~2.4 KiB |
| [[cyber/launch|cyber/launch.md]] row 33 | "recursive form blocked" | yes for that row; line 77 is in the table |
| [cyberia-blog/blog/2026_09_12.md](https://cyberia.blog/blog/2026-09-12) | describes the Merkle-free lens as unsound | correct, leave |
| [[recursive-brakedown]] | none | no |

generic STARK/SNARK comparison figures that are not zheng claims ([[cybics/crypto/zero-knowledge]], trident's `stark-proofs`, `verifying-proofs`, `provable-computing`, [[cyber/research/privacy trilateral|privacy trilateral]]) stay.


the long form of every argument above is in this file's history (`git log -p proposals/proof-system-repair.md` before this revision).
