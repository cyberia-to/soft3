---
tags: soft3, audit, zheng, lens, joy, proofs, prover, latency
crystal-type: report
crystal-domain: cyber
date: 2026-10-14
---
# proof latency on user devices — 2026-10-14

Owner directive (2026-10-14): proving takes ≤ 1 s — "the product doesn't
exist without it". Scope (coordinator, recorded as the gate row in
[[proof-system-repair]] §0): every proof a user device produces on the
product path, end to end through `joy prove` (process start → envelope
written), warm and cold, on Apple Silicon Metal (the gate), also measured
on CPU and WebGPU. Constant-size compression of long computations (profile
6, the wrap chain) runs on aggregators off the critical path and is not
in this gate.

Stand `~/cyber/.stands/FAST`: zheng `feat/fast-path` 50b3d4d (zheng#64,
on `release/0.4` 15b9549), lens `feat/fast-path` e240123 (lens#25, on
`release/0.2` 61c2a8f), joy `release/0.4` d8cfae9 (no joy change), nox
`release/0.4`, trident `release/0.4`, bbg 48153b2, hemera cf28a64/main,
strata 494df7a, honeycrisp 9cbd480. Apple M4 Max (12P + 4E, 40-core GPU,
48 GB), macOS 26.4, rustc 1.95.0, `--release`. **The machine was shared
with other agents.** The tables below were taken at 17:29–17:43 with load
average 5–18. During the day's runs load reached 36, and Metal medians at
2^18 doubled under it. Proof bytes and sizes are deterministic. Every
backend produced the same bytes (SHA-1 per fixture in §3). Android: no
device attached (`adb` absent), not run.

## 1. inventory — the proofs a user device produces

Read from the default branches on 2026-10-14 (cyb, foculus, mudra,
cybergraph, bbg, tru, soft3; joy and zheng on their release lines).
**No product repository calls the sound prover yet.** Every proof a
device makes today goes through zheng's retired fold/decide API (feature
`legacy`, unsound: `zheng/rs/Cargo.toml`, `specs/decider.md`), and cyb's
body prover imports `zheng::{prove_spmv, …}` without that feature, so it
does not build against zheng 0.4. The table therefore maps each product
statement to the profile-1 relation that proves what it claims, and that
relation is what is measured in §3.

| # | statement | produced at | prover today | what it must prove | profile-1 fixture (relation) |
|---|---|---|---|---|---|
| 1 | body PoW ticket (SpMV over the device's graph) | cyb `shell/src/worlds/body/prover.rs:252` | legacy `prove_spmv` (phi), verified locally, never sent | `y = A·x` over the device graph, `x` from the beacon | `synthetic-<k>`: a 2^k-row relation of the compiler's shape (relation-level; no statement type wraps SpMV in profile 1 yet) |
| 2 | pay σ (signal `proof`) | cyb `core/src/money.rs:383` → foculus `pay_proof.rs:77` | legacy fold+decide on one add row; the hash is computed outside the circuit | one hemera hash of content id, total, leg count | `hash-3` / `hash-8` |
| 3 | settlement ticket seal (4 tickets by default) | cyb `money.rs:785` → foculus `ticket_proof.rs:143` | legacy; the per-ticket hash outside the circuit | 4 hashes of (beacon, cluster, nonce, miner, commitment) | `batch-4` (4 × hash of 8 inputs) |
| 4 | fold-tree merge seal | foculus `epoch.rs:201`, `ticket_proof.rs:162` | legacy, one add row per merge | hash(level, pair, L, R, result) per merge | `hash-12` per merge |
| 5 | light-tip fold per block | cyb `money.rs:911` → foculus `tip.rs:210` | legacy | hash(height, root, leaves hash) | `hash-12` |
| 6 | balance opening | cyb `money.rs:284` → bbg `proof.rs:256` | lens Brakedown opening, no zheng; path ≤ 12 hemera nodes (`query_wire.rs:54`) | Merkle membership, depth ≤ 12 | `merkle-12` |
| 7 | hash / Merkle membership (generic) | — (spec) | — | membership to depth 16 / 32 | `merkle-16`, `merkle-32` |
| 8 | P1: edge public, author private | launch core 6 | not wired (zk profile 2 = veil exists) | knowledge of a secret preimage / secret leaf in a set | `veil-hash.tri`, `veil-merkle-12` (profile 2) |
| 9 | P2 stealth + veil balances, P3 hidden ego-net | mudra `specs/stealth.md`, `veil.md`; launch rows 11–12 | no code | — | not measurable: no statement defined |
| 10 | private note spend | cyb `money.rs:497` | no proof (nullifier only) | — | — |
| 11 | oikos home book | soft3 `docs/oikos.md` | doc only, "recursive zheng proofs" | — | not measurable: no statement defined |
| 12 | legacy-key bridge | mudra `specs/bridge-proof.md` | demo, `--features prove` broken | 10–20 M trace rows per claim | out of the one-relation class (aggregator side) |

Relation sizes (columns = committed variables after padding): `hash-3`
2^11, `hash-8` 2^13, `hash-12` 2^13 (2^14 rows), `batch-4` 2^15,
`merkle-12` 2^16 (2^17 rows), `merkle-16` 2^17, `merkle-32` 2^18. One
hemera hash of a digest costs ~2,700 constrained wires because nox hashes
nouns structurally (every atom is decomposed into 64 bits and hashed by
two permutations); a Merkle level hashes both children in the circuit
(the direction mux makes the sibling a wire), ~5,000 wires a level.

## 2. what was done

Profile 1 (one relation: Spartan over Fp3 + one WHIR opening) is the
user-device profile. The levers, in order of what they bought:

| lever | change | where |
|---|---|---|
| (b) grinding | `device_params(vars)`: rate 1/64, folding 4 (≤ 2^16) / 5 (above). Grinding is 16 bits up to 2^17, 22 at 2^18, 24 beyond: the least grinding that keeps the proof ≤ 64 KB. 24-bit grinding had been the whole cost of hash.tri (one 2^24 search). `params_for` (machine, IVC, wrap keys) is unchanged | zheng#64 |
| (c) Spartan | the outer sumcheck evaluates each pair's line incrementally, runs its first round over Goldilocks, uses threads and drops the per-fold claim recompute; the inner sumcheck and the row products are threaded. 2^15: 200 → 19 ms | zheng#64 |
| (c) WHIR commitment | the commitment trees are hemera's batched kernel in CPU threads (they were single-leaf, CPU-only). On Metal one command buffer builds the whole coset tree, with leaves cut from the codeword on the device (`Backend::byte_tree`). WebGPU uses permute batches | lens#25 |
| (a) one relation up to 2^20 | the compiler cap 32,768 → `MAX_RELATION` = 2^20. The compiler is 3× faster (the internal MDS layer normalises once) and the key digest 7× (threaded tree). Merkle paths of depth 8–32 now fit profile 1 instead of falling to profiles 4/5/6 | zheng#64 |
| (d) held key | `ProvingKey` / `prove_with_key`: relation, key digest, layout and relabelled relation derived once per program, reused across proofs. Same bytes (test) | zheng#64 |
| veil | private proving compiled its relation twice; now once | zheng#64 |
| (d) joy CLI | measured: ~25–35 ms of process, parse and native check over the library. The first process after a kernel change pays the Metal shader compile, ~0.7 s, once; later processes hit the OS shader cache (§3). No offline Metal compiler on this machine (`xcrun metal` absent), so no precompiled metallib | — |
| (e) phones | `ZHENG_THREADS` caps zheng's workers. lens's threads follow the available parallelism. Not measured on a phone | zheng#64 |

The verifier is unchanged in kind. It reads the WHIR parameters from the
proof and admits them only at ≥ 128 proven bits, so earlier proofs still
verify. Verifier paths stay CPU-only: the key digest uses
`Tree::build_on_cpu`, and lens's verifier never reached a device.

## 3. measurements

`cold` = `joy prove` in a fresh process (median of 3; prove ms as joy
reports it, then wall seconds from `/usr/bin/time`, then peak RSS), OS
shader cache warm. `warm` = in-process with a held `ProvingKey`
(`zheng` example `fast_path`, `FAST_KEY=1`, median of 5): a device that
proves the same statement type again. Size = joy artifact bytes. Verify =
`joy verify` wall time in a fresh process (relation recompiled, CPU).
Commands: `/tmp/fast-tools/joy_table.sh`, `warm_table.sh`; fixtures via
`FAST_DUMP` (raw `.nox` + inputs).

### profile 1 (succinct), product-shaped statements

| statement (fixture, committed vars) | metal cold | metal warm | cpu cold | cpu warm | webgpu cold | webgpu warm | size B | verify | bits (no grinding) |
|---|---|---|---|---|---|---|---|---|---|
| hash.tri (2^10) | 41 ms · 0.07 s · 52 MB | 15 ms | 47 ms · 0.05 s · 48 MB | 34 ms | 52 ms · 0.07 s · 57 MB | 26 ms | 16,756 | < 10 ms | 128.08 (112.08) |
| pay σ, one hash of 3 inputs (hash-3, 2^11) | 39 ms · 0.06 s · 57 MB | 18 ms | 53 ms · 0.05 s · 52 MB | 43 ms | 56 ms · 0.07 s · 62 MB | 31 ms | 19,834 | 10 ms | 128.08 (112.08) |
| ticket, hash of 8 inputs (hash-8, 2^13) | 80 ms · 0.10 s · 103 MB | 45 ms | 100 ms · 0.10 s · 78 MB | 88 ms | 118 ms · 0.13 s · 105 MB | 125 ms | 37,456 | 10 ms | 128.08 (112.08) |
| tip / fold merge, hash of 12 inputs (hash-12, 2^13 × 2^14 rows) | 91 ms · 0.11 s · 106 MB | 58 ms | 112 ms · 0.11 s · 75 MB | 104 ms | 143 ms · 0.16 s · 111 MB | 112 ms | 37,632 | 20 ms | 128.08 (112.08) |
| settlement seal, 4 tickets (batch-4, 2^15) | 165 ms · 0.19 s · 272 MB | 113 ms | 247 ms · 0.25 s · 156 MB | 217 ms | 266 ms · 0.28 s · 314 MB | 212 ms | 44,581 | 40 ms | 128.08 (112.08) |
| balance opening, depth 12 (merkle-12, 2^16 × 2^17 rows) | 340 ms · 0.36 s · 625 MB | 162 ms | 470 ms · 0.47 s · 366 MB | 325 ms | 545 ms · 0.56 s · 554 MB | 336 ms | 50,818 | 120 ms | 128.08 (112.08) |
| membership depth 16 (merkle-16, 2^17) | 461 ms · 0.48 s · 932 MB | 252 ms | 664 ms · 0.67 s · 565 MB | 476 ms | 756 ms · 0.78 s · 974 MB | 506 ms | 59,722 | 150 ms | 128.08 (112.08) |
| membership depth 32 (merkle-32, 2^18) | **1,100 ms · 1.13 s** · 2.09 GB | **614 ms** | 1,614 ms · 1.62 s · 1.01 GB | 1,063 ms | 1,799 ms · 1.82 s · 1.72 GB | 1,192 ms | 64,071 | 310 ms | 128.04 (106.04) |

Relation-level (no statement type wraps it in profile 1 yet), body SpMV
ticket shape, in-process without a key: `synthetic-14` 65 / 124 / 141 ms
(metal / cpu / webgpu), 40,781 B proof; `synthetic-16` 157 / 329 / 332 ms,
49,805 B.

First process after a kernel change (Metal shader compile, OS cache
cold): hash.tri 0.78 s wall (prove 46 ms), veil hash 0.76 s. Every later
process: 0.05–0.10 s.

### profile 2 (veil, zero knowledge) — P1-shaped

The program's input is secret: hash of a secret preimage; membership with
a secret leaf.

| statement | metal cold | cpu cold | webgpu cold | size B | verify |
|---|---|---|---|---|---|
| author key preimage (veil-hash.tri) | 75 ms · 0.10 s · 47 MB | 58 ms · 0.06 s · 35 MB | 83 ms · 0.10 s · 50 MB | 63,6xx–65,1xx (masked, varies) | < 10 ms |
| secret leaf, depth 12 (veil-merkle-12) | 690 ms · 0.71 s · 413 MB | 685 ms · 0.69 s · 321 MB | 739 ms · 0.76 s · 414 MB | ~295,600 | 120 ms |
| secret leaf, depth 16 (in-process, metal) | 938 ms | — | — | 418,981 | 185 ms |
| secret leaf, depth 32 (in-process, metal; before the single-compile fix) | 2,024 ms | — | — | 545,365 | 347 ms |

### before → after (Metal, `joy prove`, fresh process)

| statement | before (release lines, 2026-10-14 morning) | after |
|---|---|---|
| hash.tri profile 1 | 1,229 ms prove · 1.47 s wall (cpu 3.5 s, webgpu 2.7 s), 16,308 B | 41 ms · 0.07 s (cpu 0.05 s, webgpu 0.07 s), 16,756 B |
| 2^15 relation (merkle-3, library) | 1,556 ms | 151 ms |
| merkle-8 … merkle-32 | did not fit profile 1 (compiler `Limit`); profile 5 = 128.8 s for merkle-32 (`zheng/audit/recursion-2026-10.md`), profile 6 a minute-scale wrap chain | 0.36–1.13 s in profile 1, ≤ 64 KB |

### same bytes, ledger, soundness

- Byte identity: profile-1 envelopes for add, hash, chain-4, merkle-3 and synthetic-14 at fixed parameters are SHA-1 equal to release/0.4 on cpu, metal and webgpu. Every row above is SHA-1 equal across the three backends. Held-key proofs equal cold proofs (asserted on every warm run; test `a_held_key_proves_the_same_bytes_and_refuses_other_programs`).
- Ledger: `device_parameters_meet_the_ledger_at_every_size` gives ≥ 128 proven bits (ROM, grinding counted) at every size 2^1…2^20. Without grinding the bound is 112 bits up to 2^17, 106 at 2^18 and 104 beyond; it was 104 at every size under the old 24-bit choice.
- Bit-flip scans, 0 accepted: `device_params` hash.tri 134,048 flips; folding 5 with 16 / 22 grinding bits 155,232 / 152,352 flips; the veil envelope 512,960 flips.
- Suites: zheng 371 passed, 0 failed, 18 ignored; joy 219 passed against the branches; lens rspcs 43 and GPU differential 14 (`LENS_GPU_REQUIRE=metal,webgpu`).
- Merged 2026-10-14: lens#25 into `release/0.2`, after rebasing onto lens#26 (kernel speed): differential 15, rspcs 43, bytes unchanged. zheng#64 into `release/0.4`, after rebasing onto zheng#65: lib 305 and the succinct, veil, key and transcript suites green. One spot check after the rebases (load 20): merkle-32 `joy prove` Metal 999 ms prove, merkle-16 459 ms, same artifacts.

## 4. verdict

**≤ 1 s on Metal, warm and cold (fresh process):** every product-shaped
statement up to Merkle depth 16. That covers the ticket, pay σ, tip/fold
merge, the 4-ticket settlement seal, the depth-12 balance opening and
depth-16 membership in profile 1, plus P1-shaped veil proofs up to depth
12. CPU and WebGPU are also under 1 s for these, except veil depth 16.

**Not ≤ 1 s:**
- Membership depth 32 (2^18), cold: 1.13 s Metal, 1.6 s CPU, 1.8 s WebGPU. It is 0.61 s warm with a held key on Metal, so only the warm gate is met. Why: one Merkle level costs ~5,000 constrained wires. nox hashes nouns structurally (each atom is bit-decomposed and permuted twice), and the direction mux puts both children in the circuit. At 2^18 the 64 KB cap also forces 22 grinding bits (~0.1–0.2 s of expected grinding, high variance). The remaining cold costs are the relation compile (~200 ms) and the key digest (~60 ms). Next levers: a cheaper relation for structural hashing (a digest-pair hash without per-atom decomposition), or a disk-held proving key in joy.
- Veil (P1) beyond depth 12: depth 16 0.94 s Metal and depth 32 2.0 s. The veil prover is not device-accelerated: ~45 % of it is the hiding commitment's sequential constant-time XOF (mask and salt coins, one permutation per 4 elements) plus single-threaded CT leaf hashing. Its proofs are 295–545 KB, outside 64 KB. Parallel counter-mode coins would change the randomness design and need their own review; this branch does not touch them.
- The first Metal process after a kernel change pays ~0.7 s of shader compilation, once per installed version.
- P2 (stealth/veil balances), P3 (hidden ego-net) and oikos have no defined statement, so they could not be measured. Tickets, pay, seals and tips are still produced by the unsound legacy API in cyb/foculus. Moving them to profile 1 relations like the fixtures above is the product-side work this measures for.
- Peak RSS at 2^17–2^18 on Metal is 0.9–2.1 GB, from staging buffers (codeword copies). This matters on phones; it was not reduced here.

## 5. follow-up — fast path 2 (same day, 21:30)

zheng#66, lens#27, joy#42, merged into `release/0.4` / `release/0.2`;
report `zheng/audit/fast-path-2-2026-10.md`. Load 24–50.

| gap in §4 | after |
|---|---|
| membership depth 32 (1.13 s cold, 64,071 B, 22 grinding bits) | through zheng's hemera node jet (one permutation per level, the `hemera::merkle` tree bbg uses): **0.15 s** cold Metal (0.20 CPU, 0.23 WebGPU), 62 ms warm, 46,719 B, 16 grinding bits; depth 12/16: 0.12 s. Pattern-15 trees keep the old cost: a statement moves to the node tree in its own spec |
| veil beyond depth 12 (0.94 / 2.0 s, 295–545 KB) | counter-mode coins + held `veil::ProvingKey`: structural depth 32 638 ms warm (was 2,024); node tree depth 32 96 ms warm / 0.18 s cold, 165–181 KB. mudra#22 P1 (pattern-15 tree): depth 20 / 32 = 255 / 396 ms Metal with a held key (was 965 / 1,342). ≤ 64 KB not reached: needs hiding WHIR or a list-decoding zk opening (estimated, not built) |
| first Metal process ~0.7 s | binary-archive shader cache: `Metal::new` 0.15 s with the archive present on a cold OS cache (4.0 s sequential / 1.9 s concurrent without); a first process after a kernel change still compiles |
