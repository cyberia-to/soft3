---
tags: cyber, soft3, proposal, zheng, lens, proofs, ledger
crystal-type: process
crystal-domain: cyber
alias: proof repair ledger, stale proof claims
status: proposed
date: 2026-10-09
---
# proof-system repair — the ledger of stale claims

every place in the stack that still presents the old zheng/lens design as current: ~2 KB proofs, ~5 μs verification, "Merkle-free" Brakedown, HyperNova folding over hemera, the ~825/~89-constraint decider, algebraic Fiat–Shamir, "the accumulator is the proof". swept on 2026-10-09 across every repository under `~/cyber` (archives `zheng-022/`, `cyb-014/`, `.node-builds/`, vendored trees excluded; `zheng-pin/` is a worktree of zheng and follows it).

the rule: this page is empty when phase 5 of [[proof-system-repair]] closes. the gate is executable — `nu scripts/stale-proof-claims.nu` returns zero hits — and runs in the release train from the phase-5 candidate on.

actions: delete — the page is built on the old design and has no other content · superseded — keep, banner at the top pointing here, no other edit · rewrite — the section is rewritten against §3 of the proposal · number — replace the figure with the measured or target one (≤ 16 KB small, ≤ 64 KB any, ≤ 1 ms) · code — goes with the code removal in phase 5 · leave — a dated post or audit snapshot; add an erratum link only.

## already bannered — and whether the banner is enough

| file | banner | enough? |
|---|---|---|
| `soft3/docs/polynomial-proof-system.md:11-19` | "target architecture, not the current lens" | no — the body states the numbers as fact and links `roadmap/` where the page lives in `zheng/docs/explanation/` |
| `trident/roadmap/polynomial-target.md:254` | one inline line | no — lines 12, 75–86, 147–158, 196 are bare |
| `zheng/docs/explanation/{fri-to-whir,whir,whirlaway}.md` top | "historical … zheng has evolved to use recursive Brakedown" | no — the banner points at the unsound design as current |
| `zheng/README.md:5` | legacy commit/verify does not authenticate execution | yes |
| `zheng/specs/decider.md:105-115` | residuals named | partial — lines 9–15 still claim ~825 constraints, ~2.4 KiB |
| `cyber/launch.md:33` | "recursive form blocked" | yes for that row; line 77 is listed below |
| `cyberia-blog/blog/2026_09_12.md` | describes the Merkle-free lens as unsound | correct, leave |
| `zheng/docs/explanation/recursive-brakedown.md` | none | no |

## zheng

| file | lines | claim | action | phase |
|---|---|---|---|---|
| `docs/explanation/recursive-brakedown.md` | 12, 14, 82–85, 119–127, 157–165, 251–256, 289–304, 341–347, 355 | "Merkle-free lens, zero hash trees", "≈1.3 KiB", "~5 μs", "error ≤ 5·2⁻¹²⁸" | delete (or research/ with an "unsound" banner) | 5 |
| `specs/verifier.md` | 11, 44, 122–154, 183, 193–199 | "no Merkle verification … ~2 KiB"; ~825/~89 tiers; "verify(verify(proof)) to arbitrary depth"; "~5 μs … ~3 hemera calls"; "~30 field ops + 1 hemera per fold" | superseded; verifier spec rewritten in phase 2 | 2 |
| `specs/api.md` | 68, 104, 121, 136, 159, 202–204, 220 | "~2 KiB at 128-bit", "Brakedown … Merkle-free", decide ~825 | superseded | 2 |
| `specs/README.md` | 7, 20–22, 40, 48, 58, 69, 80–83 | HyperNova ~30 ops/fold, Merkle-free, ~660 ops ~5 μs, ~1.3 KiB, decide ~825 | rewrite | 2 |
| `specs/decider.md` | 9–15, 49, 56–65 | "~825 constraints — same whether N is 1 or 1,000,000"; "~2.4 KiB" | superseded (keep residuals) | 3 |
| `specs/recursion.md` | 9–36, 39–72, 86–94, 108, 114, 128, 176, 91, 180 | fold formula, "accumulator IS the proof", "~5 μs", "1000 levels → 2⁻¹¹⁸" | superseded; line 180 deleted | 3 |
| `specs/accumulator.md` | 9, 36–37, 53, 85, 90–97, 126 | "~200 bytes", "10–50 μs", "~30 field ops + 1 hemera" | superseded | 3 |
| `specs/transcript.md` | 11, 135–141, 153, 159 | "~3 hemera calls … algebraic Fiat–Shamir", "~2 KiB", "256-bit classical / 170+ PQ", "kd/p negligible" | rewrite | 1 |
| `specs/constraints.md` | 538, 552, 611 | "folded sub-proof ~825 constraints" | number | 3 |
| `specs/proof-types.md` | 70, 135, 183, 344 | "verify(π₀) → π₁ (~100–200 KB)", "290 μs, ~157 KiB" | number | 2 |
| `specs/tensor.md` | 73, 81 | "phone folds each step incrementally" | rewrite | 3 |
| `docs/explanation/zheng-vs-starks.md` | 16–18, 41, 274, 395, 448, 476–523, 558–582, 659, 692–836, 889–891, 975 | "computation IS proving", "240-byte checkpoint", "8.7×", "144K → 0" | delete or rewrite | 5 |
| `docs/explanation/polynomial-commitments.md` | 49, 80 | "Brakedown is Merkle-free" | rewrite | 2 |
| `docs/explanation/whirlaway.md` | 1, 105–112 | "eliminates the Merkle tree bottleneck entirely" | rewrite banner + section | 2 |
| `docs/explanation/fri-to-whir.md` | 1, 114–116 | "recursive Brakedown made it Merkle-free" | rewrite banner + section | 2 |
| `docs/explanation/whir.md` | 7, 105, 131 | banner; "290 μs" | rewrite banner | 2 |
| `docs/explanation/performance.md` | 15, 26, 121–128 | "~60 KiB / ~290 μs", "~157 KiB ~1.0 ms" | number | 2 |
| `docs/explanation/landscape.md` | 62–100 | "recursive Brakedown, the current frontier" | rewrite | 2 |
| `docs/explanation/superspartan.md` | 47–49 | "recursive Brakedown is the right choice" | rewrite | 2 |
| `docs/explanation/why-zheng.md` | 46, 91 | "recursive Brakedown is the PCS" | rewrite | 2 |
| `docs/explanation/stark.md` | 131 | "SuperSpartan + recursive Brakedown = current architecture" | rewrite | 2 |
| `docs/explanation/recursion.md` | 49, 136 | "1000 field ops + 1000 hashes ≈ microseconds" | rewrite | 3 |
| `docs/explanation/security.md` | 42–46, 65, 76–80 | "256-bit classical / 170-bit quantum", "kd/p negligible", "< 2⁻¹²⁸" | rewrite (the soundness ledger) | 1 |
| `docs/explanation/sumcheck.md` | 76 | "Hemera provides 128-bit security" | number | 1 |
| `CLAUDE.md` | 544, 554–555 | "fastest PCS verification (290 μs – 1.0 ms)" | number | 2 |
| `CHANGELOG.md` | 170 | "proof size is now a constant (~2.4 KiB + ~1.7 KiB)" | leave (historical) | — |
| `roadmap/gravity-commitment.md` | 41–51 | "~1 KiB / ~10 μs" on recursive Brakedown | superseded | 5 |
| `roadmap/ring-aware-fhe.md` | 74, 103 | "~30 field ops per fold" | rewrite | 5 |
| `.claude/plans/release-plan.md` | 5, 127–128, 152, 173–179 | "~2 KiB, ~5 μs, ~825", "~200 bytes", algebraic FS | superseded | 5 |
| `.claude/plans/axis-verifier-integration.md` | 335, 367 | "~2 KiB per proof", "~825" | superseded | 5 |
| `.claude/plans/minimal-zheng-structure-and-cli.md` | 171 | "~2 KiB spec target" | number | 5 |
| `rs/src/types.rs` | 49 | `/// ~2 KiB at 128-bit security for N = 2^20.` | code | 2 |
| `rs/src/transcript.rs` | 75 | `/// negligible at 128-bit security.` | code (fp3) | 1 |
| `rs/src/folding/{mod,fold,decide}.rs`, `types.rs:248`, `lib.rs:99,302`, `ccs/universal.rs:14`, `phi/mod.rs:10` | — | "HyperNova CCS folding" | code | 5 |
| `rs/Cargo.toml` | 5 | "SuperSpartan IOP + Brakedown PCS + sumcheck" | number (after the bake-off) | 2 |

## lens

| file | lines | claim | action | phase |
|---|---|---|---|---|
| `README.md` | 87–92 | "hemera hashes the codeword → 32-byte commitment … 20 positions" | rewrite | 2 |
| `specs/commitment.md` | 85, 287, 441 | "~1.3 KiB proof, ~660 field ops", "recursive tensor decomposition", "d 20–30 for 128-bit" | number + rewrite | 2 |
| `specs/binary-tower.md` | 101 | "~30 ops + 1 hemera per fold" | rewrite | 3 |
| `specs/polynomial-ring.md` | 75, 116 | "~30 field ops per fold" | rewrite | 3 |
| `.claude/plans/release.md` | 299 | "Brakedown opening verifier as a CCS instance (~825)" | superseded | 5 |
| `brakedown/`, `ikat/`, `porphyry/` `Cargo.toml` | 6 | "expander-graph codes" | number (with the bake-off) | 2 |

`specs/scalar-field.md` is already correct.

## nox

| file | lines | claim | action | phase |
|---|---|---|---|---|
| `specs/jets/decider.md` | 5–69 | "89 constraints ≈ 100 nanoseconds", "~200 bytes", "240 bytes" | delete or superseded | 3 |
| `specs/jets/recursion.md` | 24, 44 | "Brakedown (Merkle-free PCS) … ~825" | rewrite | 3 |
| `specs/jets.md` | 82, 103 | "Merkle-free … ~825 … ~89 … recursion to arbitrary depth" | rewrite | 3 |
| `specs/jets/README.md` | 23, 38 | "decider … 89 constraints" | number | 3 |
| `specs/trace.md` | 568, 656–669 | "no Merkle paths", "~825 / ~89", "~2 KiB … ~30 field ops + 1 hemera" | rewrite | 3 |
| `specs/reduction.md` | 509–550 | "each reduce() folds into HyperNova", "accumulator IS the proof", "~3 calls", "10–50 μs" | rewrite | 3 |
| `specs/jets/state.md` | 63–66 | "folds into accumulator (~30 field ops)" | rewrite | 3 |
| `specs/jets/polynomial-ring.md` | 33 | "HyperNova folds the F₂ sub-trace" | rewrite | 3 |
| `specs/vm.md` | 83, 105 | "~766 constraints per type transition" | number | 3 |
| `roadmap/decider-product.md` | 23–55 | "~200 bytes … 89 constraints, ~100 ns", "Brakedown is Merkle-free" | delete or superseded | 5 |
| `roadmap/strata-collapse.md` | 89, 162 | "cross-algebra composition via HyperNova folds" | rewrite | 5 |
| `docs/explanation/decider.md` | 3–75 | "all history in 89 constraints … 240 bytes … 100 nanoseconds" | delete | 3 |
| `docs/explanation/five-algebras.md` | 41, 68–72 | "universal accumulator (~200 bytes) … 89 constraints" | rewrite | 3 |
| `docs/explanation/self-verification.md` | 29, 53–61 | "Merkle-free … ~825 / ~89 … ~2 KiB per level" | rewrite | 3 |
| `docs/explanation/jets.md` | 9, 105, 111 | "Merkle-free … all-history verification in 89 constraints" | rewrite | 3 |
| `docs/explanation/layers.md` | 58 | "Brakedown (Merkle-free PCS) … ~825" | rewrite | 3 |
| `docs/explanation/README.md` | 27 | "decider.md — 89 constraints" | with the page | 3 |
| `docs/explanation/why-nox.md` | 88–90 | "1 trillion txs → 1 proof (~100 KiB)" | number | 3 |
| `.claude/plans/jet-registry-0.1.md` | 10, 26, 130, 170–178, 260 | "89/825-constraint verifier" | superseded | 5 |
| `rs/jets/decider.rs` | 6–32 | `//! 89 primary + 825 cross-term constraints` | code (the jet verifies nothing; launch #40) | 3 |
| `rs/jets/formulas.rs` | 352–356 | `/// Full 89/825-constraint verification` | code | 3 |

## bbg

| file | lines | claim | action | phase |
|---|---|---|---|---|
| `README.md` | 17, 50–51 | "~5 μs via zheng-2 folding", "~2 KiB (recursive Brakedown)" | number | 2 |
| `specs/architecture.md` | 91, 101, 193, 222, 238, 245–250, 277 | "~5 μs", "~75 bytes", "~2 KiB", "~240 bytes", "~200 bytes per namespace" | number | 2 |
| `specs/data-availability.md` | 42, 77–96 | "~75 bytes (recursive Brakedown)", "O(λ log log N)" | rewrite | 2 |
| `specs/state.md` | 73 | "~2 KiB, ~5 μs" | number | 2 |
| `specs/indexes.md` | 20–21, 46, 185 | "O(λ log log N) … ~5 μs", "~200 bytes per opening" | number | 2 |
| `specs/privacy.md` | 204, 273 | "~240 bytes constant", "~2 KiB" | number | 2 |
| `specs/neuron-state.md` | 178–179, 201, 211, 215, 234 | "~5 μs", "~2 KiB", "~240 bytes", "~200 bytes regardless of batch" | number | 2 |
| `specs/temporal.md` | 43 | "~5 μs" | number | 2 |
| `roadmap/verifiable-query.md` | 91, 140–150 | "~5 μs", "~1 / ~5 / ~2 KiB" | number | 2 |
| `docs/explanation/architecture-overview.md` | 21, 57, 67–79, 89, 107–115 | "240-byte checkpoint … ~5 μs", "accumulator IS the proof", "~3 hemera calls" | rewrite | 3 |
| `docs/explanation/why-signal-first.md` | 44–52, 82, 100–109 | "~240 bytes", "~5 μs", "~2 KiB" | number | 2 |
| `docs/explanation/why-polynomial-state.md` | 24, 90, 111, 129 | "~200 bytes", "10–50 μs", "~5 μs" | number | 2 |
| `docs/explanation/polynomial-privacy.md` | 22, 28, 55–56 | "O(1) verification, ~200 bytes" | number | 2 |
| `docs/explanation/data-availability.md` | 59, 88, 152–155 | "~200 bytes per sample" | number | 2 |
| `docs/explanation/signal-sync.md` | 45 | "~40,000 constraints verifiable in ~5 μs" | number | 2 |
| `.claude/plans/pattern17-look-integration.md` | 20 | "Brakedown opening (~825) is a folded sub-instance" | superseded | 5 |
| `CLAUDE.md` | 561 | "Brakedown evaluation proofs" | number (with the bake-off) | 2 |

## hemera

| file | lines | claim | action | phase |
|---|---|---|---|---|
| `roadmap/algebraic-fiat-shamir.md` | 11, 25, 40–46, 72 | "8.7× fewer hemera calls" | delete | 1 |
| `roadmap/README.md` | 26, 28, 53–63, 69–74 | algebraic FS row; "~3 calls per execution"; "ZERO hemera calls", "Brakedown is Merkle-free", "144K to 0", "each permutation = one fold step" | rewrite | 2 |
| `specs/README.md` | 26, 113 | "one Hemera call for binding hash" | rewrite | 2 |
| `specs/tree.md` | 161–168 | "~75 bytes of proof, O(1) random access" | number | 2 |

## soft3

| file | lines | claim | action | phase |
|---|---|---|---|---|
| `docs/polynomial-proof-system.md` | 11–19, 21, 33, 47–62, 87–91, 127–137, 155, 165, 173–185, 211–237, 250 | the whole old design with its numbers | delete (fix the link in the meantime) | 5 |
| `docs/README.md` | 94, 105, 123 | "~200 bytes", "~30 field ops … 100 nanoseconds", "microseconds" | rewrite | 2 |
| `specs/terms.md` | 97 | "~200-byte query proofs" | number | 2 |
| `status.md` | 26 | "~200B proofs" | number | 2 |
| `specs/languages.md` | 158, 345 | "HyperNova folds all partitions" | rewrite | 3 |

## cyber — site, whitepaper, research

| file | lines | claim | action | phase |
|---|---|---|---|---|
| `whitepaper.md` | 672, 684, 1202, 1014, 1042, 1419 | "expander-graph codes, HyperNova folding"; "each level constant (~100–200 KB)"; "O(1) global state (~22kb)"; "≥ 1 − 2⁻¹²⁸ by zheng soundness"; "fold of winning tickets" | rewrite §9.3, number §18, soundness-ledger wording, ARC in §14.3 | 2–3 |
| `whitepaper.md` | 678 | SNARK ~200 B vs zheng ~100–200 KB (comparison table) | number (≤ 64 KB target) | 2 |
| `light.md` | 66, 125, 160 | "one proof of ~100–200 KB" | number | 2 |
| `network.md` | 133 | "one recursive proof (~100–200 KB)" | number | 2 |
| `communication.md` | 125, 198 | "after recursive composition: ~100–200 KB" | number | 2 |
| `launch.md` | 77, 228 | "one O(1) accumulator by HyperNova folding" | rewrite (ARC) | 3 |
| `security.md` | 54 | "≥ 1 − 2⁻¹²⁸" | rewrite | 1 |
| `restructure.md` | 481 | "continuous fold / ~30 field ops / ~200-byte accumulator" | number | 3 |
| `research/algorithmic essence of superintelligence.md` | 31, 88, 100–111, 143–144, 281–283, 418 | "~2 KiB", "~5 μs", "~3 calls", "recursive Brakedown" | superseded | 5 |
| `research/universal law.md` | 85–87, 116 | "~2 KiB, ~50 μs", "~30 field operations" | number | 5 |
| `research/bbg.md` | 26, 97–101, 122, 175, 192, 205–259, 279, 304 | "~200 bytes, 10–50 μs", "Merkle-free lens … 144,000", "240 bytes" | superseded | 5 |
| `research/polynomial nouns.md` | 35, 48, 57, 109, 168–189, 232–234, 313, 342–366 | "~3 hemera calls", "~200 bytes", "each reduce() folds" | superseded | 5 |
| `research/programmable state.md` | 30, 147, 207 | "~200 bytes, 10–50 μs" | superseded | 5 |
| `research/cybergraph model architecture.md` | 103, 174–175, 194, 213 | "10–50 μs", "~30 field ops", "240-byte checkpoint" | superseded | 5 |
| `research/egregore properties.md` | 29, 112–118 | "240-byte accumulator", "10–50 μs" | number | 5 |
| `research/algebraic state commitments.md` | 40, 61–63, 168–169, 294–297, 314 | "~200 bytes", "50 μs", "no Merkle tree" | superseded | 5 |
| `research/data availability strategy.md` | 318, 326 | "~200 bytes per sample" | superseded | 5 |
| `research/structural-sync.md` | 164–185 | "HyperNova (~30 ops per step) … 700×" | rewrite | 3 |
| `research/provable consensus.md` | 12, 40, 208–256, 287 | "~50 μs to verify" | number | 3 |
| `research/vec formalization.md` | 145, 232 | "~50 μs"; "error ≤ 2⁻⁵¹² at k=16" | number | 3 |
| `research/data structures for polynomial state.md` | 48 | "10 ZB of state in 50 μs" | number | 5 |
| `research/five algebras.md` | 207 | "~30 field ops per fold" | number | 5 |
| `research/nox - frozen provable computer.md` | 70 | "ONE HyperNova accumulator" | rewrite | 3 |
| `research/256 symbols.md` | 155, 169 | "decider … 89 constraints" | number | 3 |
| `research/bootstrap.md` | 276–280, 319 | "HyperNova folder … 89 constraints" | number | 3 |
| `research/unified mining.md` | 29, 77, 190 | algebraic NMT openings as real work | review | 3 |

## other repositories

| file | lines | claim | action | phase |
|---|---|---|---|---|
| `crystal/architecture.md` | 331–341, 478, 501 | "constant-size global state (~22kb)" | rewrite + number | 3 |
| `crystal/stark.md` | 31, 40 | "~60–200 KB"; "HyperNova accumulator (~200 bytes)" | rewrite | 3 |
| `crystal/topoisomerase.md` | 36 | "~5μs per step" | number | 2 |
| `eidos/specs/certificate.md` | 19, 159, 172 | "~825 constraints … constant time" | number | 3 |
| `trident/roadmap/polynomial-target.md` | 12, 75–86, 147–158, 196 | "~8K → ~89", "~2 KiB … ~0.1 μs", "~30 ops/fold" | superseded (extend the line-254 banner) | 5 |
| `foculus/specs/structural-sync.md` | 113, 142, 282, 327, 375–386 | "~2 KiB", "~5 μs", "~200 bytes", "~30 ops" | number | 3 |
| `foculus/specs/fold-mining.md` | 13, 34, 83, 125–139 | "HyperNova IVC folding … decider O(1)" | rewrite (ARC) | 3 |
| `foculus/specs/provable-consensus.md` | 12, 191, 277 | "~50 us" | number | 3 |
| `foculus/specs/vec.md` | 127, 212, 238 | "~50 us"; "2⁻⁵¹² at k=16"; "batched folding (IVC)" | number / review | 3 |
| `foculus/specs/gossip.md` | 33 | "verify σ (tens of microseconds)" | number | 3 |
| `foculus/docs/explanation/latency-targets.md` | 68, 188, 211–212 | "Lens ~200 B", "O(1) fold (~30 field ops)" | number | 3 |
| `foculus/docs/explanation/life-of-a-signal.md` | 78, 98 | "roughly fifty microseconds" | number | 3 |
| `foculus/README.md` | 17 | "HyperNova fold tree" | rewrite (ARC) | 3 |
| `foculus/proposals/view-certificates.md` | 89 | "π_att HyperNova fold" | review | 3 |
| `foculus/src/{tip,tickets,marginal_cert,pay_proof,ticket_proof,rewards,step,epoch}.rs` | doc comments | "HyperNova σ / seal / fold" | code (ARC) | 3 |
| `tok/programming-model.md` | 127, 213 | "fold into one constant-size proof via HyperNova"; "~200 bytes" | rewrite / number | 3 |
| `inf/specs/proof.md` | 66, 72 | "~200 bytes"; "~5 μs" | number | 2 |
| `inf/specs/cost.md` | 114 | "~5 μs, one decider" | number | 2 |
| `inf/README.md:16`, `inf/docs/README.md:11`, `cybergraph/docs/README.md:11` | — | "in microseconds" | number | 2 |
| `tru/specs/rewards.md` | 189 | "self-folds using zheng's HyperNova IVC" | rewrite (ARC) | 3 |
| `strata/jali/specs/noise.md:62`, `strata/jali/docs/explanation/lattice-security.md:97`, `strata/jali/README.md:77` | — | "~30 field ops per fold"; ring-aware Brakedown | number | 5 |
| `cybics/crypto/graphy.md` | 55 | "Brakedown — 5 μs verification" | number | 2 |
| `soma/soma-spec.md` | 714 | "~5 μs" | number | 2 |
| `fs/sync.md` | 147 | "ONE zheng proof … (~50 μs)" | number | 3 |
| `evy/specs/evy.md` | 398 | "~50μs for BBG recommit + proof" | number | 5 |
| `cyberia-blog/blog/2026_03_24.md:9-21`, `2026_03_26.md:46`, `2026_03_27.md:38` | — | "recursive brakedown (the perfect PCS)", "~200 bytes, 10–50 μs" | leave (dated posts) — erratum link | 5 |
| `warriors/audit/parano1d-vs-uhash-2026-09-27.md` | 108, 187, 194 | "HyperNova folding"; "decider ~2.1 KB measured" | leave (audit snapshot) — footnote | — |
| `joy/CHANGELOG.md` | 71 | "SuperSpartan + Brakedown + HyperNova accumulators" | leave (historical) | — |

generic STARK/SNARK comparison figures that are not zheng claims (`cybics/crypto/zero-knowledge.md:18`, `trident/docs/explanation/stark-proofs.md:566`, `trident/docs/guides/verifying-proofs.md:38`, `trident/docs/explanation/provable-computing.md:170-175`, `cyber/research/privacy trilateral.md:307`, `trident/docs/explanation/privacy.md:283`) stay.

## counts

| repo | rows | delete | superseded | rewrite | number | code |
|---|---|---|---|---|---|---|
| zheng | 36 | 2 | 10 | 14 | 7 | 4 |
| nox | 21 | 3 | 2 | 11 | 3 | 2 |
| bbg | 17 | 0 | 1 | 3 | 13 | 0 |
| cyber | 27 | 0 | 8 | 7 | 12 | 0 |
| others | ~45 | 0 | 2 | 10 | 30 | 1 |

see [[proof-system-repair]] §C for how the ledger is closed, and `scripts/stale-proof-claims.nu` for the gate.
