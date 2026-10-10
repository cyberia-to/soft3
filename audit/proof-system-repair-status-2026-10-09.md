---
tags: soft3, audit, zheng, proofs
crystal-type: report
crystal-domain: cyber
date: 2026-10-09
---
# proof-system repair — status, 2026-10-09

status of [[proof-system-repair]] across every repository it touches. evidence: the build stand `~/cyber/.rel04` (zheng v3 branch, nox/joy/trident `release/0.4`, lens `fix/public-execution-pcs`, bbg 1d39975, neuron fd24e76, hemera/strata/tade defaults) and the consumer sweep of the same day.

## phases

| phase | required | state | done |
|---|---|---|---|
| 0 | relation shrink | built, unmerged | zheng#45 into `release/0.4` |
| 1 | public prefix, pinned constant, vk, fp3 challenges, one format with a profile byte, soundness ledger, attack fixtures | ~25 % | public profile only: certificate v3 (zheng#46, joy#32) — hash.tri 15,608 → 6,495 B, add.tri 2,214 → 216 B; zheng 274+6+12 and joy 216 tests green. missing: fp3 (unused in zheng), vk, one format (six remain: JOYEXEC3, JOYST001, JOYZH001/ZHMITH01, JOYZK003, hypernova-v2, legacy trace), `zheng/specs/soundness.md`, legacy attack tests still pass |
| 2 | PCS bake-off, succinct ≤ 16/64 KB ≤ 1 ms, bbg QueryProof | 0 % | no RS code, no WHIR in lens |
| 3 | ARC, step relation, decider, Trident verifier, recursion, fold-mining on ARC | 0 % | — |
| 4 | zk profile, second verifier | 0 % | private paths: MPC-in-the-head (ZHMITH01, linear), Triton (JOYZK003, ~0.8 MB) |
| 5 | delete legacy, close the ledger | 0 % | `scripts/stale-proof-claims.nu`: ~740 hits outside the stand |

## repositories

| repo | role | state |
|---|---|---|
| zheng | core | v3 in #46; legacy commit/verify/fold/decide unsound and public; #33 (0.4.0 → master) awaits decision; master is 0.3.3 without `execution` |
| lens | PCS | 0.2.0 only in lens#14; expander code with proven distance 1; Porphyry known-unsound |
| joy | wire, CLI | JOYEXEC3 in #32; state (JOYST001) and private artifacts untouched |
| foculus | settlement tickets, fold seal, pay σ, tip, φ* — launch cores 1–2 | entirely on legacy fold/decide; origin pins zheng 0.3; untouched |
| bbg | state, QueryProof, checkpoint accumulator | legacy types; joy needs bbg 1d39975, not on the default branch; untouched |
| cybergraph | signal admission | verifies no proof; untouched |
| cyb | body mining | legacy phi SpMV; `crates/cyb` on prove_pay/TipProver; untouched |
| trisha | Triton zk backend | untouched |
| mudra, inf | optional legacy proofs | `prove` features broken (row 38); untouched |
| nox | step relation, decider jet | jet claims 89/825 constraints and checks nothing; untouched |
| trident | second verifier | — |
| strata/nebu | fp3 | present, unused |
| hemera | hash | profile v2 (#15) undecided; fixes Merkle and constants of any succinct profile |
| soft3 | node, release | `release/phase1.toml` pins zheng 0.3.3 and lens master; joy, trisha, trident unpinned; #162 unmerged |
| cyber, crystal, tok, tru, eidos and others | documents | every §C ledger row open |

## blockers

1. default branches do not build together (launch row 39): foculus, bbg, cybergraph pin zheng 0.3 on origin; joy, trisha, inf pin 0.4. the train cuts from default branches only; no soft3 candidate with working proofs exists until the 0.4 line merges (zheng#33, lens#14, nox/joy/trident `release/0.4`, zheng#45/#46, joy#32).
2. the launch cores run on the unsound fold: foculus settlement tickets, fold seal, pay σ, tip.
3. the node verifies no proof at admission.
4. hemera profile v2 is undecided.

## remaining for each release

- 0.11 — soundness floor everywhere: v3 for state artifacts; foculus tickets, pay σ and tip moved from legacy fold to v3 certificates with per-ticket verification; admission verification in cybergraph and the node; zheng legacy API behind a `legacy` feature and off every production path; fp3 where challenges remain (private paths); spec and soundness ledger; phase-1 ledger rows; pins in `phase1.toml`. estimate 6–10 sessions plus the owner's merges of the 0.4 line.
- 0.12 — phase 2 in full: 2–4 weeks.
- 0.13 — phase 3, research-grade.
