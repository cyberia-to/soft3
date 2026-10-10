# Changelog

## 0.11.0 — proposed, unreleased (2026-10-10)

The proof-system repair (`proposals/proof-system-repair.md`). soft3 0.11.0 is the release that closes over it; it is not releasable until the sibling code below reaches the default branches (see "where the code is"). Every number below is quoted from the audit named next to it; the audits record machine, load and command. Times were taken on a shared Apple M4 Max under load and are upper bounds of a quiet machine; sizes are deterministic.

### what was unsound and is retired

1. zheng 0.3.x `commit` / `verify` / `fold` / `decide`, the universal CCS and the phi SpMV proof: the fold was never checked, the statement was not bound, the constant wire was free, the opening compared a prover value with itself. The API moves behind zheng's `legacy` feature, off by default, never on a production path (zheng#47, `zheng/specs/soundness.md`).
2. joy's legacy trace-statement artifact (`zheng-hypernova-tensor-merkle-v2`) is removed; `joy verify` refuses it (joy#34).
3. foculus settlement tickets, fold seal, pay σ, tip and the epoch-certificate seal rode the legacy fold and established nothing; they move to per-ticket v3 certificates verified natively (foculus#61, `foculus/audit/sound-settlement-proofs-2026-10.md`).
4. bbg's legacy zheng checkpoint accumulator is replaced by the v3 certificate path (bbg#37).
5. cybergraph admission and the soft3 node did not verify signal proofs; admission now runs `foculus::check_signal_proof` (cybergraph#15, soft3#167).

### profiles — one `ZHENGPF1` envelope with a profile byte

| profile | what | sound | measured (fixture `hash.tri`, 61 cycles, unless named) | source |
|---|---|---|---|---|
| 0 public (certificate v3) | verifier derives the relation, witness disclosed, every row checked | yes, not succinct | certificate 294,861 → 15,608 B after the compact relation (zheng#45), → 6,495 B with certificate v3 (zheng#46) | `zheng/audit/compact-relation-2026-10-09.md`, `soft3/audit/proof-system-repair-status-2026-10-09.md` |
| 1 succinct | Spartan over Fp3, witness committed, one WHIR opening (rate 1/64, k = 4, 24 grinding bits) | yes, 128.00 proven bits | proof 15,921 B, envelope 16,148 B; verify 7.96 ms with relation compile, 1.5 ms with a cached verifying key; chain-11 41,405 B; synthetic 2^20 71,081 B | `zheng/audit/succinct-profile-2026-10.md`, `zheng/audit/zk-profile-2026-10.md` §2 |
| 2 zk (veil) | masked Spartan + hiding RS tensor commitment | yes, 128.2 proven bits, HVSZK in the ROM | 63,861 B median (63,125–64,901), prove 142.9 ms, verify 9.985 ms (4.132 ms cached key); MPC-in-the-head on the same statement 8,884,423 B | `zheng/audit/zk-profile-2026-10.md` |
| 3 state | public execution over authenticated bbg state; zheng authenticates every read itself (`StateEvidence`) | yes | — | zheng#49 |
| 4 machine | the nox machine as one uniform step relation + hash-based accumulation + decider | yes | add.tri 83,408 B; hash.tri 96,968 B, verify 5.43 ms; merkle-32 145,500 B, verify 8.64 ms; grows ~96 KB per 2^14-row segment; 512 statements → one decider 8.9 MB, verify 1.17 s | `zheng/audit/accumulation-2026-10.md` |
| 5 recursive (IVC) | the accumulation verifier inside the next step's relation | yes; 2,275,056 bit flips of a full proof, 0 accepted | 282–285 KB flat from 33 to 1,572,850 cycles (121 steps); verify 17.9–38.4 ms; rate 1/64: 233,562 B | `zheng/audit/recursion-2026-10.md`, `zheng/audit/recursive-envelope-2026-10.md` — zheng#53/#54, not merged |

The goal (proof ≤ 64 KB, verify ≤ 1 ms, constant in the computation's length) is not met: profile 5 is constant but 3.6–4.4× over 64 KB and 18–38× over 1 ms. The wrap step that targets it is in progress.

### per repository

1. zheng (0.4.0 line, `release/0.4`): compact relation — linear forms, native constants, degree-7 S-box (#45); public certificate v3 (#46); soundness floor — legacy behind a feature, state v3, Fp3 challenges, one envelope, `specs/soundness.md`, attack fixtures rejected (#47); succinct profile (#48); zk profile, state binding, verifying keys (#49); proof-repair ledger docs (#50); phase 3 accumulation over the nox machine (#51). Open: every nox opcode in the machine (#52), IVC (#53), recursive envelope (#54).
2. joy (`release/0.4`): JOYEXEC3 public certificates (#32); public and state artifacts on `ZHENGPF1` (#33); legacy trace-statement artifact removed (#34); `joy prove --succinct` (#35); `joy prove --zk` on veil, JOYST002 (#36); succinct falls back to the machine beyond the relation limit (#37). Open: recursive proofs (#38, needs zheng#53/#54).
3. lens: Reed–Solomon multilinear PCS — TensorRs and WHIR over Goldilocks with hemera Merkle (#17); batched Merkle verification (#18); WHIR over Fp3 messages and leaf access (#19); docs (#20). At n = 2^20, 128 proven bits: WHIR rate 1/16 83,716 B, verify 11.6 ms; TensorRs 480,838 B, 53.5 ms; every bit-flip scan clean (`lens/audit/rs-whir-pcs-2026-10.md`). Stacked on lens#14 (cyber-lens 0.2.0, owner decision); not merged.
4. foculus: sound settlement proofs (#61) — ticket proof 45–101 B on the wire, verify 0.26–24.6 ms for 1–8 contributors; cluster settlement as one zheng accumulation (#63) behind the `accumulated-settlement` feature, native per-ticket verification stays the default (512 tickets, 3 contributors: native 31,232 B / verify 636 ms against accumulated 850,628 B / 704.7 ms) (`foculus/audit/sound-settlement-proofs-2026-10.md`, `foculus/audit/accumulated-settlement-2026-10.md`). Stacked on foculus#5 (owner decision); not merged.
5. cybergraph: admission verifies signal proofs before they are applied (#15). Stacked on cybergraph#3 (owner decision); not merged.
6. bbg: checkpoint on zheng v3 certificates instead of the legacy accumulator (#37, merged into `chore/coordinated-release-20260916`); docs retired (#36, #38).
7. soft3 node: `/v2/frame` admits a valid proved signal and rejects a tampered, a foreign and a legacy proof, root unchanged (#167, merge after cybergraph#15).
8. mudra: constant-time `hash_secret` for secret entropy (#19); the `prove` demo on v3 certificates (#20, stacked on mudra#2, owner decision).
9. inf: the `prove` feature lowers to zheng v3 certificates (#8).
10. hemera: permutation 1.4× single, ~5× batched, bit-exact, `hash_secret` (#17).
11. strata/nebu: `Goldilocks::dot` overflow at the second product fixed (#14).
12. docs: stale-claim ledger closed in nox (#27, #28), trident (#124, #125), bbg (#36, #38), evy (#2, #3), cybics (#1, #2), glia (#9), cybergraph (#16), cyber (#114, #115), hemera (#18), soft3 (#162, #168).

### where the code is

`release/phase1.toml` in this bump pins the default-branch heads of 2026-10-10. zheng, lens and foculus pins do not move: their repaired code is on integration lines (`release/0.4` for zheng, nox, joy, trident; the heads of lens#14, foculus#5, cybergraph#3, mudra#2) that reach the default branches only through the owner's decisions (zheng#33 and the 0.4 line, lens#14, foculus#5, cybergraph#3, mudra#2). Until then a candidate cut from the default branches carries none of the repair, and this bump stays a draft.
