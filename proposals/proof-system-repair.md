---
tags: cyber, soft3, proposal, zheng, lens, joy, proofs, post-quantum
crystal-type: process
crystal-domain: cyber
status: proposed
date: 2026-10-09
---
# one proof — the repair of zheng

> goal fixed by the owner on 2026-10-09: **a proof of any computation is at most 64 KB, post-quantum, verifies in at most 1 ms, and stays that size however long the computation — and the system that produces it is simple, reliable and flexible.** this page records what is broken, why the old numbers were never real, what the literature now allows, and the design that meets the goal with the fewest parts. it closes on the last merge, not the first.

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

### the layers and who owns them

| layer | owns | repo |
|---|---|---|
| relation | program + subject shape → CCS; public prefix `(1, io, cycles)`; jets as rows | zheng `execution/relation` (today), nox semantics |
| IOP | Spartan over CCS: outer sumcheck, inner sumcheck, one evaluation claim per matrix family | zheng `spartan` |
| commitment | multilinear polynomial → RS codeword → hemera Merkle; WHIR opening at an fp3 point; ARC accumulation of proximity claims | lens (new crate `lens/whir`; `brakedown` retired) |
| accumulation driver | uniform step relation; fold the IOP's evaluation claims into one ARC accumulator per run; decider = one WHIR proof of the accumulator | zheng `accumulate` (replaces `folding`) |
| statement & wire | profile, program digest, inputs, outputs, cycles, budget; serialisation; versioning | joy `rs/execution` + zheng `execution/statement` |
| independent verifier | the whole verifier in Trident on nox, cross-checked against the Rust one on every fixture | trident `lib/std/zheng` (new) |

### the proof, end to end (succinct profile)

1. **relation.** the prover compiles the program exactly as the verifier will: CCS with a public prefix `z = (1 ‖ io ‖ cycles ‖ w)`. the verifier evaluates the prefix's multilinear extension itself; only `w` is committed. this is what binds program and io once the witness leaves the wire — the hole of 2026-09-09, closed structurally.
2. **verifying key.** the verifier hashes the compiled CCS once (`vk = hemera(matrices)`) and absorbs `vk`, not every entry. the key is cached per program; the artifact carries the program, not the key.
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
| **total, 128 bit** | **~32–40 KB** |
| same at 100-bit conjectured (t ≈ 30) | ~16–20 KB |

verify: 64 queries × ≤ 30 hemera permutations + two short sumchecks ≈ 2,000 permutations ≈ **0.5–1 ms** native (hemera is ~0.3 µs per permutation on M4). the budget is dominated by Merkle authentication, as in every hash-based system; that is the price of "one hash" and it is within the goal.

for a program of `n = 2^20` the WHIR part grows by two more rounds and 10 more hashes per path: **~56–70 KB** at 128 bits — the number WHIR publishes.

### unbounded programs: accumulation, then one decider

a program longer than the relation limit (today 2^15 rows, 29 permutations) is proven step by step over a **uniform step relation** — one CCS for "one nox reduction step with continuity and memory arguments" (`zheng/audit/general-nox-relation-review.md:160-173` lists the requirements). each step yields one proximity claim about its committed witness; ARC folds claim after claim into one accumulator of fixed size (one root, one evaluation point, one value — a few hundred bytes plus the accumulated Merkle openings per step, which is where ARC's "small number of openings relative to the code rate" matters). at the end, one WHIR proof decides the accumulator. the proof of a million steps is the same ~60 KB as the proof of one.

this is the *recursion milestone* every zheng document pointed at. the difference from the 2026 plan is that it no longer needs a homomorphic commitment nor a verifier circuit of the verifier: ARC is accumulation in the random-oracle model, which is exactly the one assumption the stack makes.

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
| 2 | `lens/whir` (RS, hemera Merkle, fp3, Johnson parameters), Spartan over fp3, `succinct` profile | hash.tri **≤ 40 KB, verify ≤ 1 ms**; bit-flip scan clean; soundness ledger complete and all rows proven | lens, zheng, strata (fp3 exposure) |
| 3 | uniform step relation + ARC accumulation + decider | merkle-32 and a 10^6-step run both prove; proof size independent of length, **≤ 64 KB**; verify ≤ 1 ms | zheng, nox |
| 4 | `zk` profile (VEIL masking); Trident verifier | zk fixtures; Rust and Trident verifiers agree on every fixture | zheng, trident |
| 5 | delete: brakedown, folding, legacy formats, stale docs (§8) | `tokei` shows the proving path ≤ 12k lines; no document claims 2 KB or 100 ns | lens, zheng, joy, nox, bbg, crystal |

phases 1 and 2 are a month of focused work each; 3 is the research-grade one and is where ARC's prover cost must be measured before committing; 4 and 5 are weeks.

## 8. documents that still describe the old design as current

to be marked "superseded by proof-system-repair" or deleted in phase 5: `zheng/specs/{verifier,api,README,decider,accumulator,recursion}.md`, `zheng/docs/explanation/{recursive-brakedown,polynomial-commitments,whirlaway,fri-to-whir,zheng-vs-starks,performance}.md`, `zheng/CLAUDE.md` (Brakedown line), `nox/specs/jets/{decider,recursion}.md`, `lens/README.md`, `lens/specs/commitment.md`, `bbg/specs/{architecture,data-availability}.md` (the ~2 KiB / ~75 B lines), `cyber/whitepaper.md:1202` (~22 KB), `cyber/light.md:115`, `crystal/architecture.md:331-341`, `soft3/docs/polynomial-proof-system.md` (already bannered; the link to `roadmap/` is wrong).

## 9. what this proposal does not claim

no 2 KB. no 100 ns. no "Merkle-free". no proof smaller than the authentication of its own queries. the stack keeps one assumption and pays for it in bytes; 64 KB is what that honesty costs at 128 bits, and it is still the smallest in production.
