# proof-system repair — the release, turnkey

owner instruction 2026-10-09: implement the whole proposal (`proposals/proof-system-repair.md`) and deliver a release prepared end to end. one soft3 release carries all phases.

## integration base — the stand

every repository at one coherent revision, built together (`~/cyber/.rel04`, verified 2026-10-09):
zheng `feat/public-certificate-v3` (on #45 on `release/0.4`) · joy `feat/public-certificate-v3` · nox, trident `release/0.4` · lens `fix/public-execution-pcs` (0.2.0) · bbg 1d39975 · neuron fd24e76 · hemera, strata, tade, tok, soft3, inf, file, radio defaults · foculus `fix/shared-storage-dependencies` · cybergraph `feat/durable-applications` · mudra `refactor/spell` · tru `master` · honeycrisp `feat/tip5`.
zheng, joy, foculus, cybergraph, soft3, tok, tru build together on it. trisha needs external `tasm-lib` and leaves the product path (Triton becomes a test oracle only).

## work packages

| wp | repo(s) | scope | depends | gate |
|---|---|---|---|---|
| A | lens | Reed–Solomon over Goldilocks (nebu NTT); fp3 points; TensorMerkle on RS with proven query counts; WHIR with hemera Merkle; soundness parameters as data | — | completeness + bit-flip rejection tests; sizes/verify times measured at n = 2^10..2^20 |
| B | zheng | phase 1 complete: fp3 challenges (transcript, sumcheck, Spartan); legacy fold/decide/commit/verify behind feature `legacy`, off by default; state execution on v3; one envelope (magic, version, profile byte); `specs/soundness.md`; attack fixtures rejected | — | full suite; legacy attack tests flip or leave the default build |
| C | foculus (+tok) | tickets, pay σ, tip, step, epoch cert off legacy fold: per-ticket v3 certificates, cluster settlement verifies every ticket, φ* check on v3 | — | foculus suite; ticket sizes measured; a forged ticket rejected |
| D | cybergraph, soft3 | admission verifies signal proofs; node rejects bad proofs | C | end-to-end: node admits a valid proved signal, rejects a forged one |
| E | zheng + lens | succinct profile: Spartan + the bake-off winner per class; wire; measurements | A, B | small ≤ 16 KB (stretch) / ≤ 20 KB, large n = 2^20 ≤ 64 KB, verify ≤ 1 ms, bit-flip scan clean |
| F | zheng, nox | uniform step relation + ARC accumulation + decider; Trident verifier; fold mining on accumulation | E | merkle-32 and 10^6-step runs prove, constant size ≤ 64 KB |
| G | zheng | zk profile (masking) | E | zk fixtures; differential against the Triton oracle |
| H | all docs repos | close §C ledger | B–G | `scripts/stale-proof-claims.nu` → 0 |
| R | soft3, cyber | versions, changelogs, `release/phase1.toml` pins, gates, candidate, receipts in `audit/release-<date>/` | all | the train's gates green |

parallelism is bounded by disk (35 GB free): at most three stands build at once. waves: {A, B, C} → {D, E} → {F, G} → {H, R}.

## rules held throughout

- every claim is a test or a measurement; no number enters a document before it is measured.
- the verifier derives what it checks; the soundness ledger has no "conjectured" row on a production profile.
- each work package lands as PRs on the integration branches; the stand is rebuilt from pushed refs before the release.
