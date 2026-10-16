---
tags: cyber, soft3, proposal, foculus, zheng, nox, bbg, hemera, mining, consensus
crystal-type: process
crystal-domain: cyber
status: proposed
date: 2026-10-13
---
# proof-gated chain on goldilocks — what parano1d proves is possible, ported to zheng

> parano1d is a production proof-native chain: 393 kLOC, 20 crates, mainnet live since 2026-08-21, every block carrying a recursive validity proof, PoW ordering only. soft3 wants the same *shape* — the present proves the past — on the goldilocks stack ([[zheng]], [[nox]], [[hemera]], [[bbg]]). this page is the comparison and the plan: what to port as design, what to port as code, what to build new, and why the goldilocks version is *smaller* than the binary one, not bigger. it depends on [[steps-blocks-epochs]] for the consensus clock and on [[proof-system-repair]] for the accumulator.

## 1. the comparison

parano1d workspace (v2.0.3, clone 2026-10-13, 533 files / 393,551 LOC / 2,684 tests / 295 unsafe, all SIMD):

| layer | parano1d | soft3 today | verdict |
|---|---|---|---|
| field | GF(2^128) committed traces, GF(2^256) challenges | Goldilocks + fp3 | ours — keep |
| hash | Poseidon2b, t=4, 8 full + 58 partial rounds, CLMUL SIMD | hemera (Poseidon2-Goldilocks), t=16, experimental partial rounds | ours after the 2h repair (audited params) |
| per-block proof | HistoryStep: self-recursive, verifies parent's terminal transcript in-circuit; terminal **913 KB–1.08 MB**; prove 17.6 s (Small 63 payments, i7-1365U), verify 0.83 s | zheng public certificate per step — 15,608 B for one hash on M4 Max, linear-size (discloses witness) | different trade: tiny per-step proofs, no per-block recursion |
| from-genesis validity | recomputed *inside every block* (94 kLOC relation crate) | ARC accumulation over steps + epoch decider ([[proof-system-repair]]) | ours — cheaper, **the reason the port is smaller** |
| batching | FROST-GKR "killshots" over shared permutation spines | nox traces are already per-reduction rows; no batching layer needed | ours by construction |
| consensus machinery | ASERT DAA, fork choice, fees, emission, retention, mempool, template mechanics — **field-free, ~60–80 kLOC** | foculus: sync/epoch/tickets/fold-mining implemented; **no mempool, no DAA, no block production, no chain sync anywhere** | the entire gap; parano1d is the reference design |
| state | sparse UTXO vector, Poseidon2b sparse-Merkle root, occupancy-priced growth burn | [[bbg]]: one polynomial per public dimension | ours, plus their growth-fee rule (§3e of [[steps-blocks-epochs]]) |
| contracts | bounded 16-instruction integer core baked into the fixed relation | [[trident]]/nox: general bounded VM, cost report per program | ours — strictly more, at the price of a general trace |
| soundness | executable Category-1 certificate, premises named | soundness ledger planned in the repair proposal | build: a `soft3-soundness` equivalent counting hemera/zheng parameters |
| network | libp2p gossipsub + request/response, scoped RPC keys, GUI wallet | foculus sync planes + radio | build, their spec docs are the checklist |

one honest symmetry: parano1d's own FRI soundness beyond unique decoding is *conjectural* (RS capacity conjecture, Johnson ≈ 64 bits) and their docs force every security statement to name its regime. both chains ship named premises; neither is a free lunch.

## 2. why the goldilocks port is smaller

parano1d's size is recursion tax: `noid_recursive` (94 kLOC) + `noid_ivc_core` (73 kLOC) + `noid_gkr` (27 kLOC) exist because every block re-verifies the parent's complete proof transcript in-circuit, over a frozen, bank-authenticated matrix (`history_step_bank`, two-class, build-time pinned). swap the hash and the bank invalidates — the binary proof stack is all-or-nothing and cannot be half-reused.

the zheng design never pays that tax: each step proves only its own `apply`; the ARC fold chains steps; an epoch decider certifies the accumulator. the ~200 kLOC of recursion machinery maps to the fold+decider already specced in [[proof-system-repair]] (~30 field ops + 1 hash per fold). what remains to build is the **chain machine** — the part parano1d demonstrates is ~40 % of the work: block production, mempool, DAA, retention, p2p consensus, rpc, node.

## 3. the plan

### P0 — the accumulator (hard blocker, already scheduled)
land ARC (or WARP) in zheng per [[proof-system-repair]]; hemera to audited parameters (2h). nothing else starts before this — it replaces HistoryStep.

### P1 — the chain machine (foculus; new crates, design ported from parano1d)
1. **block-attempt state machine** — §3a of [[steps-blocks-epochs]]; parano1d's `noid_miner/src/miner.rs` phase-ordered loop and refresh triggers (new tip, payout change, invalidation, heartbeat) are the reference.
2. **mempool** — parano1d's `noid_mempool` design: proof-verified admission, cached authorization reuse, fee floor; against cybergraph intents instead of UTXO spends.
3. **DAA** — port `noid_chain/src/consensus/difficulty.rs` (ASERT, 1,021 LOC, integer-only, field-free; Apache-2.0, attribution header) with the proving-latency pricing of §3b.
4. **state + fees** — bbg as the state organ; parano1d's occupancy-scaled growth burn (1×/2×/4×/8× at 50/75/90 % occupancy) as the §3e starting point.
5. **retention** — §3d: headers permanent, bounded bodies, portable receipts, manifest-bound snapshot sync (their `docs/architecture/synchronization.md` is the checklist).
6. **classes** — §3c: reduction budgets via `trident build --costs` instead of frozen matrix banks; no bank pinning, no build-time matrix authentication — a class is a number, not a file.

### P2 — the work function
hemera-grinding PoW over a frozen template schedule (parano1d's 16-field POWHDR schedule, shape stolen, hash swapped); nonce-only external miner (`noid_extminer` is 634 LOC — the whole trust model in one file: template in, 16-byte nonce out). decision to record: goldilocks mining is GPU-friendly (quantus precedent) — we embrace GPU miners and put the trustlessness in the pool contract (§8 of [[steps-blocks-epochs]]) rather than fighting ASIC economics. the lottery split — tickets as the per-epoch gate, grind as the per-block race, the fold stitching them (§5 of [[steps-blocks-epochs]]) — is inherited, not re-decided, here.

### P3 — network, rpc, node
libp2p gossipsub + request/response following their protocol string/header-first/gossipsub layout; scoped bearer keys (mining key → template+submit only) per their 986-line rpc spec; node orchestration last.

### P4 — the soundness ledger
a `soft3-soundness` crate in the image of `noid_soundness`: an executable certificate importing production constants (hemera profile, ARC geometry, query counts, DAA parameters) and printing the resource ledger. the repair proposal's ledger becomes runnable.

## 4. effort, stated against evidence

parano1d: ~3 years, one author, 393 kLOC. this port starts with zheng/nox/bbg/foculus already live and skips the recursion tax. estimate: **P1 is 60–80 kLOC of new code in foculus-adjacent crates** — the comparison says so (their field-free consensus surface), P2–P4 another ~30 kLOC. at the stack's measured velocity this is an engineering schedule of 12–18 months, dominated by P1.3–P1.6, *after* P0.

## 5. should the two codebases combine?

yes, at three levels, in this order of commitment:

1. **design/specs — fully, now.** parano1d's `docs/` (195 files) is the only complete public record of a launched proof-native chain: block-attempt lifecycle, template trust boundaries, retention arithmetic, soundness accounting. free to take, already analyzed here.
2. **code — selectively, with attribution.** Apache-2.0; the field-free modules (ASERT, fork choice, fee/occupancy schedules, template selection) port nearly as-is; everything touching GF(2^128), the matrix bank, or Poseidon2b transcripts stays behind. their `experimental_object` naming trap (production consensus wearing an experimental label) is a warning about carrying module names across without reading them.
3. **runtime — no.** no shared binary, no bridge-by-default: two chains, two fields, two soundness ledgers. interop is a *guest-circuit* problem (bitsliced poseidon2b verification inside nox) for bridges, exactly as scoped in the field comparison — never a merged consensus.

the asymmetry that makes this worth doing: parano1d proves the product is buildable and shows the 40 % of the work soft3 never specced; soft3's accumulator design is the one structural improvement over parano1d's recursion tax. port the machine, skip the tax.

acceptance: P0 closes per [[proof-system-repair]] acceptance; P1 lands a devnet that produces blocks with per-step zheng certificates and an epoch decider, measured on commodity hardware, receipts in `audit/`.
