---
title: chain — the phase-1 root chain
tags: cyber, soft3, spec, chain, consensus, mining, bbg, hemera, mudra
crystal-type: spec
crystal-domain: cyber
status: accepted
date: 2026-10-15
alias: chain spec, root chain, phase-1 chain, soft3 chain, pussy chain, bostrom chain
---

# chain — the phase-1 root chain

The root chain of phase 1 is a public proof-of-work chain on the soft3
harness: hemera as the work function, bbg as the state, mudra NSIG1 as the
signature, cyber-radio as the wire, tade as the framing. It carries exactly
three things a neuron can do — mine, transfer, cyberlink — and nothing else.
The four phase-1 cores (settlement mining, fold, foculus consensus, privacy)
run beside it in shadow mode as indexers and activate later by a scheduled
fork (§7).

Owner decision 2026-10-15, recorded here as the contract: "pure PoW, public
chain, no features — mining, transfers, cyberlinks — on the whole soft3
harness; the four phase-1 cores run in shadow mode as indexers and activate
later by scheduled fork". Spec before code: implementation starts from this
page, [chain-wire](chain-wire.md) and [chain-conformance](chain-conformance.md).

Two chains run the same binary from two genesis files: pussy on
2026-11-05 13:37:00 UTC, bostrom at 13:22:42 UTC on the day bbg row 43 of
[cyber/launch](../../cyber/launch.md) is closed (the bostrom replay needs
incremental commitments; pussy does not).

The shape is parano1d's — fixed small header, ASERT, heaviest chain, bounded
reorg, compact bodies — with the recursion tax removed: no proof is on the
critical path. Numbers quoted from parano1d are from docs.parano1d.org
(v2 parameters page, read 2026-10-15) and are cited as reference points, never
copied blindly.

Design rules this page obeys:

- field arithmetic never floats in anything consensus-relevant; the DAA is
  integer-only (§1.5);
- zheng proofs are optional and later (checkpoint proofs, §7); nox programs
  are not required — validation is native Rust;
- canonical encodings: every value has one valid byte form; a decoder that
  accepts two forms for one value is a consensus bug;
- vocabulary: file · particle · cyberlink · neuron · token · focus;
- no rung of the [ladder](../roadmap/stack-completeness.md) depends on a rung
  above it (§9).

## 0. parameters

Every constant in one table. `S0` is the genesis supply of the chain's native
coin, read from the genesis file (§4).

| name | value | unit | where used |
|---|---|---|---|
| `HEADER_BYTES` | 168 | bytes | §1.1 |
| `BLOCK_TIME` (T) | 30 | s | §1.5 |
| `HALF_LIFE` (H) | 600 | s | §1.5 |
| `MTP_WINDOW` | 11 | headers | §1.4 |
| `MAX_FUTURE_DRIFT` | 120 | s | §1.4 |
| `POW_LIMIT` | 2^236 (bits `0x1e100000`) | target | §1.3 |
| `GENESIS_BITS` | `0x1e010000` (target 2^232) | bits | §1.3, §4 |
| `MAX_BLOCK_BYTES` | 262,144 | bytes, header + body | §1.6 |
| `MAX_BLOCK_TXS` | 1,024 | transactions incl. coinbase | §1.6 |
| `MAX_TX_BYTES` | 512 | bytes | §2 |
| `MIN_FEE_PER_BYTE` | 1 | native units per tx byte | §2.6 |
| `COINBASE_MATURITY` | 100 | blocks | §2.2 |
| `FINALITY_DEPTH` | 120 | blocks (60 min) | §5.2 |
| `CHECKPOINT_INTERVAL` | 120 | blocks | §5.3 |
| `CHECKPOINT_SUNSET` | 525,600 | blocks (≈ 6 months) | §5.3 |
| `BLOCKS_PER_YEAR` | 1,051,200 | blocks | §3.5 |
| `SUBSIDY` | floor(S0 / 105,120,000) | native units per block (1 %/yr) | §3.5 |
| `ORPHAN_POOL` | 1,024 headers, 600 s | — | §5.4 |
| `NONCE_WINDOW` | 16 | nonces ahead admitted to mempool | §2.5 |
| `MEMPOOL` | 8,192 txs / 16 MiB | — | chain-wire §4 |
| `MAX_REORG` | `FINALITY_DEPTH` − 1 = 119 | blocks | §5.2 |
| `POOL_DEPTH` | 40 | commitment-tree levels (rule set 2, frozen at genesis) | §11.4 |
| `POOL_DIGEST` | 64 | bytes of every wide pool digest (frozen at genesis) | §11.1 |
| `ANCHOR_WINDOW` | 1,000 | blocks (rule set 2, frozen at genesis) | §11.4 |
| `MAX_SPEND_TX_BYTES` | 9,985 | bytes of a kind-5 transaction (rule set 2) | §11.7 |

Rationale for the two that matter most: T = 30 s is the interval parano1d
measured its way to (15 → 20 → 30) for a proof-gated chain; our blocks carry
no proof, so 30 s is conservative, and conservative is the point of phase 1.
H = 600 s (20 blocks) is between parano1d's 180 s (reacts in six blocks,
amplifies timestamp play: a 120 s future-drift shifts the target 1.6× per
block) and BCH's two days (a 10× hash-rate loss would stall the chain for a
day); at 600 s a 120 s drift moves the target 1.15× and a 100× hash-rate
collapse recovers in about 66 minutes (§1.5).

## 1. block

### 1.1 header

A header is exactly 168 bytes, little-endian throughout, no varints, no
optional fields. 168 = 3 × 56: three full hemera rate blocks plus the
mandatory padded block, four permutations per hash.

| offset | size | field | type | meaning |
|---|---|---|---|---|
| 0 | 4 | `version` | u32 | bits 0–15 rule set (this spec = 1); bits 16–31 miner signals (§7) |
| 4 | 8 | `height` | u64 | 0 for genesis, `prev.height + 1` otherwise |
| 12 | 8 | `time` | u64 | unix seconds, §1.4 |
| 20 | 4 | `bits` | u32 | compact target, §1.3 |
| 24 | 16 | `nonce` | [u8; 16] | miner-chosen; external workers return it as 16 LE bytes |
| 40 | 32 | `prev` | [u8; 32] | block id of the parent; zero for genesis |
| 72 | 32 | `tx_root` | [u8; 32] | §1.7; for genesis the genesis file particle (§4) |
| 104 | 32 | `state_root` | [u8; 32] | bbg root after applying this block, §3.3 |
| 136 | 32 | `chain` | [u8; 32] | chain id = genesis file particle, §4.4 |

The block id is the flat hemera sponge over the 168 header bytes:

```
block_id = hemera(header_bytes)          // 32 bytes, no domain prefix
```

The chain id is a header field, so a header of one chain never validates on
the other and every header is self-identifying. It costs 32 bytes per block,
88 MB per year of permanent headers; accepted.

Siblings share the parent's target (§1.5 reads only the parent), so changing
a candidate's `time` never changes its work; miners cannot shop for an easier
target by timestamp.

### 1.2 proof of work

```
valid_pow(header) :=  u256_le(block_id) < target(bits)
```

Both sides read as 256-bit little-endian integers; equality fails. Genesis
(height 0) is exempt: its id is pinned by the genesis file, not mined.

Expected hashes per block are `2^256 / (target + 1)`, the block's work (§5.1).
Measured 2026-10-15 on an M4 Max, one core, `hemera::hash` over 168 bytes
from a scratch release build against `~/cyber/hemera/rs` (phase1.toml pin):
64,400 H/s — about 15 µs per header, far from the 0.3 µs-per-permutation
figure of steps-blocks-epochs §7, which is the permutation alone on the
optimized path. Ten cores give order 0.6 MH/s per machine; three such
machines together order 1–2 MH/s (the cybernode boxes are x86 and must be
measured separately). The genesis target (2^232,
2^24 ≈ 16.8 M expected hashes) is sized so one such machine finds a block in
about half a minute at that rate, and the DAA takes over from there. A
honeycrisp or GPU hemera kernel will be one to three orders faster; the
chain does not care, ASERT does (§1.5), and §8 says what it means.

### 1.3 compact target

`bits` is Bitcoin's compact form, with one tightening: the encoding must be
canonical.

```
exp  = bits >> 24
mant = bits & 0x007fffff
reject if bits & 0x00800000            // sign bit set
target = exp >= 3 ? mant << (8 * (exp - 3)) : mant >> (8 * (3 - exp))
```

Canonical encode:

```
size = byte_length(target)                 // 0 for target 0
mant = size <= 3 ? target << (8*(3-size)) : target >> (8*(size-3))
if mant & 0x00800000 { mant >>= 8; size += 1 }
bits = (size << 24) | mant
```

A header is invalid unless `bits == encode(decode(bits))` and
`decode(bits) <= POW_LIMIT` and `decode(bits) >= 1`. Examples:
`0x1e010000` ↔ 2^232 (canonical, `GENESIS_BITS`); `0x1d100000` ↔ 2^228
(canonical); `0x1e001000` decodes to 2^228 too and is rejected;
`0x1d800000` is rejected (sign). `POW_LIMIT` = 2^236 is the floor: at the
measured 64 kH/s one laptop core mines a floor block in about 16 s, so the
chain never stalls below one laptop core. Vectors in chain-conformance §2.

### 1.4 timestamps

- `time > MTP(prev)` where `MTP` is the median of the `time` of the previous
  `MTP_WINDOW` = 11 headers (fewer near genesis: all available, genesis
  included); median of an even count is the lower middle;
- `time <= now + MAX_FUTURE_DRIFT` against the validating node's clock; a
  header failing only this rule is held, not rejected, and re-evaluated for
  up to 600 s;
- genesis `time` is the genesis file's value.

### 1.5 difficulty adjustment — ASERT, integer only

The target of a block at height `h ≥ 1` with parent `P` (height `h − 1`) is a
pure function of the genesis anchor and the parent:

```
Δt     = P.time − genesis.time                   // i64 seconds
ideal  = (h − 1) · T                             // i64 seconds
e      = floor_div((Δt − ideal) · 65536, H)      // i64, floor toward −∞
shifts = e >> 16                                 // arithmetic shift
frac   = e & 0xffff                              // 0 ≤ frac < 65536
factor = 65536 + ((195766423245049·frac + 971821376·frac² + 5127·frac³ + 2^47) >> 48)
target = genesis_target · factor                 // u256 × u32 → u288 (use u512)
target = shifts < 0 ? target >> (−shifts) : target << shifts
target = target >> 16
target = clamp(target, 1, POW_LIMIT)
bits(h) = encode(target)
```

`factor` is the cubic approximation of `2^(frac/65536)` in 16.16 fixed point,
error below 0.01 % (the aserti3-2d polynomial, attributed to Bitcoin Cash,
MIT). Shifts beyond 256 saturate at the clamp. Everything is integer; a
floating-point `pow` anywhere in this path is a consensus bug. The parent's
`time`, not the candidate's, enters the formula; this is what gives siblings
one target.

Behavior:

- hash rate ×k, held: the interval shortens to T/k immediately and the
  chain runs ahead of its ideal clock; the target halves for every H = 600 s
  the parent is ahead of schedule, so the lead stops growing once the target
  has fallen by k and the interval is back at T — a 2× step is 90 %
  absorbed within about 50 minutes, a 10× step within about two hours. The
  error signal is the absolute drift of the parent's time from its ideal
  time, so there is no averaging window to game and no block-count lag.
- hash rate ÷k: the interval stretches to k·T at first; the target doubles
  every 600 s of wall time whether or not blocks arrive, so recovery takes
  H·log2(k): k = 10 → 33 min, k = 100 → 66 min, with the chain producing
  blocks at a rising rate throughout. This is the property that makes a
  small chain survivable when one farm leaves.
- timestamp play: a miner setting `time` at the +120 s drift limit moves the
  next target by at most 2^(120/600) = 1.149; the MTP rule bounds backward
  play to the window median. A miner controlling many consecutive blocks can
  lower difficulty by at most 1.149 per block — the attack is visible (wall
  time disagrees with block time) and self-correcting the moment honest
  blocks resume, because ASERT measures absolute drift, not per-block deltas.
- clock drift in the network is irrelevant beyond the two rules above; nodes
  need no synchronized clocks to agree on targets.

Reference points: parano1d (T 30 s, H 180 s, MTP 11, drift 120 s); Bitcoin
Cash (T 600 s, H 172,800 s).

### 1.6 body and budgets

```
body = count: u32 LE ‖ tx[0] ‖ tx[1] ‖ … ‖ tx[count−1]
```

- `tx[0]` is the coinbase (§2.2) and no other tx is;
- `count ≤ MAX_BLOCK_TXS`;
- `168 + len(body) ≤ MAX_BLOCK_BYTES`;
- each `tx` is self-delimiting (fixed size per kind, §2) and
  `len(tx) ≤ MAX_TX_BYTES`;
- the same txid never appears twice in a block.

Rationale: 256 KiB and 1,024 txs is 40 tx/s sustained, three orders of
magnitude above bostrom's historical rate (2.9 M links in five years ≈
0.02/s) and small enough that a block crosses three continents in under a
second over QUIC. parano1d's terminal cap is 1.1 MB because it carries proofs;
ours carries none.

### 1.7 transaction root

A hemera Merkle tree over the transactions in body order, domain-separated,
no duplication of odd nodes:

```
leaf_i = hemera("soft3/chain/leaf/v1\0" ‖ tx_bytes_i)
level  = [leaf_0 … leaf_{n−1}]
while len(level) > 1:
    next = []
    for pairs (L, R) in level:  next.push(hemera("soft3/chain/node/v1\0" ‖ L ‖ R))
    if len(level) odd:          next.push(last(level))      // promoted unchanged
    level = next
tx_root = level[0]
```

Promoting the odd node instead of duplicating it removes the duplicate-tx
malleability of Bitcoin's construction. A coinbase-only block has
`tx_root = leaf_0`.

## 2. transactions

### 2.1 envelope

```
tx = version: u8 (= 1) ‖ kind: u8 ‖ body ‖ sig
```

| kind | name | body bytes | sig | total bytes |
|---|---|---|---|---|
| 0 | coinbase | 80 | none | 82 |
| 1 | transfer | 120 | NSIG1 (102) | 224 |
| 2 | cyberlink | 153 | NSIG1 (102) | 257 |
| 3 | claim | 36 | NSIG1 (102) | 140 |

Any other `version` or `kind` is invalid under rule set 1; kinds 4 (deposit)
and 5 (spend) are the pool's, formats frozen at genesis, valid only from rule
set 2 (§11.7). Fixed sizes: a
decoder knows the length from the second byte; trailing or missing bytes are
a decode error.

```
txid       = hemera("soft3/chain/txid/v1\0" ‖ chain ‖ tx_bytes)
statement  = hemera("soft3/chain/sign/v1\0" ‖ chain ‖ tx_bytes_without_sig)
```

`chain` is the 32-byte chain id. Binding it into both hashes gives
cross-chain replay protection for free: a pussy transfer is not a valid
bostrom transfer even for the same key and nonce.

### 2.2 coinbase (kind 0)

```
height: u64 ‖ to: neuron[32] ‖ amount: u64 ‖ extra: [u8; 32]
```

- `height` equals the block height (prevents identical coinbases in two
  blocks, hence identical txids);
- `amount == SUBSIDY + Σ fees of the block's other txs`, exactly; less is
  invalid too (no burning by omission — supply must be a pure function of
  height, §3.5);
- `extra` is free: pool extranonce, a commitment, zeros;
- the output is credited to `balances[H(to ‖ τ_native)]` and is spendable
  only from height `height + COINBASE_MATURITY`, so a reorg never invalidates
  spends of a subsidy that was itself reorged out.

### 2.3 transfer (kind 1)

```
from: neuron[32] ‖ nonce: u64 ‖ token: [u8;32] ‖ amount: u64 ‖ to: [u8;32] ‖ fee: u64
```

Moves `amount` of `token` from `from` to `to` and `fee` of the native coin
from `from` to the block's coinbase. `to` is any 32-byte card id (a neuron
id, or a particle — value can be parked on a particle). `amount` may be 0
(a fee-only tx is valid, useful for claiming a nonce slot). `token` is a
token id (§3.1).

### 2.4 cyberlink (kind 2)

```
from: neuron[32] ‖ nonce: u64 ‖ token: [u8;32] ‖ amount: u64 ‖ p: particle[32] ‖ q: particle[32] ‖ valence: i8 ‖ fee: u64
```

The cyberlink `(p, q, token, amount, valence)` per
[cybergraph/specs/cyberlink.md](../../cybergraph/specs/cyberlink.md), authored
by neuron `from`. `p ≠ q`; `valence ∈ {−1, 0, +1}` (any other byte is
invalid). The box: `amount` of `token` leaves `from`'s balance and is bound
to the axon-particle `axon(p, q) = hemera(p ‖ q)` (bbg's `axon_id`):
`balances[H(axon ‖ token)] += amount`. In phase 1 the box is locked — there is
no withdraw transaction; withdraw and transfer of boxes arrive with the
cores' activation (§7). `amount = 0` is a bare assertion and is the common
case. State effects are exactly bbg's `insert` for one link (§3.2).

Why two neuron-signed kinds and not one: the two actions have different state
footprints (a transfer touches two balances; a link touches two particles,
two axon lists, an axon-particle and a balance) and different audiences
(wallets vs cyb). One tx per action keeps each a fixed-size record that is
trivial to index and impossible to mis-parse.

### 2.5 nonces and replay

Every neuron record carries `nonce: u64` (§3.2). A tx of kinds 1–3 is valid
only if `tx.nonce == neurons[from].nonce`; applying it increments the nonce.
Together with the chain id in the statement this closes replay within and
across chains. Mempool admission accepts nonces in
`[current, current + NONCE_WINDOW)` and orders them; block validation
accepts only the exact current nonce, in body order.

### 2.6 fees

`fee ≥ len(tx_bytes) · MIN_FEE_PER_BYTE` in the native coin, paid to the
block's miner through the coinbase amount. Nothing is burned. At 1 unit per
byte a transfer costs 224 BOOT (bostrom; BOOT has no decimals, supply ≈
1.15 × 10^15) or 224 PUSSY (pussy, supply ≈ 1.15 × 10^18): a spam floor, not
a market. Fee markets, growth fees and burns are cores-era rules (§7).

### 2.7 signatures — mudra NSIG1

`sig` is the 102-byte NSIG1 envelope of
[mudra/specs/neuron-auth.md](../../mudra/specs/neuron-auth.md):
`"NSIG1" ‖ pubkey[33] ‖ ecdsa[64]`, verified by
`mudra::neuron::verify_statement(subject, statement, sig)` with
`subject = tx.from` (kinds 1, 2) or `subject = claim::neuron_of(pubkey)`
(kind 3) and `statement` as in §2.1. The envelope already requires
`neuron_of(pubkey) == subject`, so the neuron id is the hash of the key and
no key registry exists. The curve is secp256k1; phase-1 signatures are
classical, stated plainly (§8).

### 2.8 the legacy bridge — implicit, with one explicit case

`claim::neuron_of(pubkey) = hemera(compressed pubkey)` is both mudra's native
id for a legacy key and the id genesis uses for every snapshot account whose
pubkey is known (`pubkeys.csv` lists all 61,675 bostrom accounts; the ones
that ever signed a transaction carry a key; §4). Such a holder signs transfers with the same key and the same
mnemonic they used on bostrom or pussy — the bridge is implicit; no claim tx
exists for them. The ADR-036 `claim` of
[mudra/specs/bridge.md](../../mudra/specs/bridge.md) (binding a legacy address
to a different native neuron) is not a chain transaction in phase 1: a holder
who wants a new neuron transfers to it.

The explicit case: accounts whose pubkey never appeared on the old chain
(8,390 wallets that never sent a transaction; address-only). Genesis parks
their balances under a dormant owner id

```
legacy_owner(hrp, account_id) = hemera("soft3/chain/legacy/v1\0" ‖ hrp ‖ account_id[20])
```

that no key controls, and kind 3 moves them:

```
claim (kind 3): account_id: [u8;20] ‖ nonce: u64 ‖ fee: u64
```

valid iff `ripemd160(sha256(sig.pubkey)) == account_id`
(mudra `cosmos::account_id`), the dormant owner holds at least `fee` of the
native coin, and `nonce == neurons[neuron_of(pubkey)].nonce`. Effect: every
balance of `legacy_owner(hrp, account_id)` (all tokens) moves to
`neuron_of(pubkey)`; fee to the miner. The hrp is the chain's (§4.1).

## 3. state

### 3.1 identities

- particle: 32 bytes, `hemera(bytes)` of a file per the `file` crate (row 19
  of cyber/launch: every crate computes this today); the chain never
  distinguishes a content particle from any other 32-byte card;
- neuron: 32 bytes, `neuron_id::NeuronId`; for legacy keys
  `hemera(compressed pubkey)`;
- token: 32 bytes, `token(denom) = hemera(denom_bytes)` — `hemera("boot")`,
  `hemera("pussy")`, `hemera("hydrogen")`, `hemera("tocyb")`, …; the native
  coin of a chain is `token(genesis.denom)`;
- axon-particle: `hemera(p ‖ q)`;
- balance key: `hemera(owner ‖ token)` (bbg `balance_key`).

### 3.2 bbg dimensions used

The state is one `bbg::BbgState`; the chain writes six of its dimensions and
leaves the rest empty but committed:

| dimension | written by | content |
|---|---|---|
| particles | cyberlink | `energy += amount` on `q`, `weight += amount` on `axon(p,q)`; records created at zero on first touch |
| axons_out, axons_in | cyberlink | `axon(p,q)` appended once per `(p,q)` |
| neurons | all | `NeuronRecord { focus, karma, stake, nonce }`; phase 1 writes `nonce` only, the other three stay 0 |
| coins | coinbase, genesis | `CoinRecord { total_supply }` per token |
| balances | all | `hemera(owner ‖ token) → u64` |
| locations, cards, files, time, signals, intents, A(x), N(x) | nobody | empty; their commitments are constants of the empty dimension |

Change requested of bbg (rung 3, one field): `NeuronRecord.nonce: u64`, with
`DIMENSION_VERSION` bumped to 3. The alternative — storing nonces as a
pseudo-token balance — avoids the change and leaves a permanent hack in the
state; rejected.

Balances live inside the root. Today the native node keeps pay balances in
`NativeNode.balances` outside bbg; under this spec that map is deleted and
every balance, including the miner's and the dormant legacy owners', is an
entry of the `balances` dimension and therefore under `state_root`. That is
the fix the launch page asks for.

No nullifiers, no private commitments: phase 1 is an account model with
nonces; `A(x)` and `N(x)` stay empty until the pool activates (§11), and the
pool's turnstile keys `T(τ)` in the balances dimension (§11.5) stay absent.

### 3.3 state root per block

`state_root = bbg root` (`BbgState::compute_root`, the 14-leaf hemera
compression over the 11 dimension commitments, A, N and stats) after the
block's transactions are applied in order, coinbase first. The root is
refreshed once per block, never per transaction.

Honest cost statement: `compute_root` today recommits every dimension in
full (cyber/launch row 43; measured quadratic on import). Per block this is
O(state), not O(changes · log n). For pussy (29 K links, ~3 K accounts) it is
milliseconds and launch proceeds. For bostrom (2.9 M links, 3.1 M particles,
61 K accounts) it is tens of seconds per block on one machine and genesis
replay is impossible until bbg's incremental commitments land — which is why
bostrom waits on row 43. The dimension serialization also falls back from
Brakedown to a hemera digest above 2^16 field elements
(`BRAKEDOWN_EVAL_CAP`); the root stays deterministic, look proofs for those
dimensions are unavailable, and nothing in phase 1 needs them.

### 3.4 transition and conservation

For a block B at height h on state S:

```
S' = S
apply coinbase: balances[H(to ‖ τ)] += amount ; coins[τ].total_supply += SUBSIDY
for tx in body[1..]:
    check nonce, signature, balances (every debit ≥ 0 after), kind rules
    apply; neurons[from].nonce += 1
assert state_root(B) == root(S')
```

Invariant checked by the node after every block and exposed in `/status`:

```
Σ balances[·, τ_native] (incl. boxes and dormant) == S0 + h · SUBSIDY
Σ balances[·, τ]        == genesis supply of τ        for every other token
```

Fees move, subsidy mints, nothing burns; supply is a function of height
alone. Conservation is checked by the `tok` crate's conservation helpers
(plumb), which own the arithmetic; the chain supplies the deltas.

### 3.5 subsidy — the stakeless onramp, bootstrap phase

The subsidy is the envelope of rewards.md §8 ("the subsidy envelope precedes
the lottery it will carry"): a per-block amount paid for a proof meeting a
difficulty target, karma-blind and stake-blind, the permissionless door. In
phase 1 the proof is a hemera header hash — synthetic work, tolerable
"exactly once: at bootstrap the subsidy's product is distribution, not
attribution". The settlement ticket of §7–§8 replaces the hash as the proof
by scheduled fork (§7); the envelope does not change.

```
SUBSIDY = floor(S0 / 105,120,000)        // = S0 · 1 % / BLOCKS_PER_YEAR
```

constant per block, 1 % of genesis supply per year, no halving. For bostrom
(S0 ≈ 1.15 × 10^15 BOOT) about 1.1 × 10^7 BOOT per block; for pussy
(S0 ≈ 1.15 × 10^18 PUSSY) about 1.1 × 10^10 PUSSY per block. Rationale for
constant-and-small: the only purpose is to pay the security floor of a canary
chain and to put native coin in the hands of newcomers who mine; a halving
schedule is a monetary policy decision the cores' activation makes with real
data (rewards.md §10 derives the floor from attack economics; nothing is
known at genesis). The number is an owner decision and is listed open in §10
with this default.

### 3.6 retention

- headers: permanent on every node (168 B × 1,051,200 per year = 177 MB/yr);
- bodies: phase-1 nodes are archival — every body is kept, because genesis
  replay and the shadow indexers read the whole stream; worst case
  256 KiB × 2,880 blocks/day = 720 MB/day, typical a few MB/day; a pruning
  mode (`--prune`, keep bodies above `tip − FINALITY_DEPTH` plus a bbg
  checkpoint) is permitted for non-archival nodes and is not a consensus
  rule;
- state: one bbg database per chain under `$home/<chain>/bbg`; recovery
  replays committed operations and verifies stored roots before readiness,
  as native-node.md already requires.

## 4. genesis

### 4.1 file

A genesis is one file: a tade stream, dialect `urn:cyberia:chain-genesis:1`,
first frame `(*, k)` with that URN, then exactly one `p` frame, then `b`
frames, then `l` frames, in the canonical order below, nothing else. Its
particle is the chain id.

| frame | name | data | size |
|---|---|---|---|
| `(~, p)` | params | `version: u32 (=1) ‖ name ‖ hrp ‖ denom ‖ time: u64 ‖ bits: u32 ‖ subsidy: u64 ‖ checkpoint_key: [u8;33] ‖ checkpoint_sunset: u64 ‖ pool: [u8;74]` where each string is `len: u8 ‖ utf8` and `pool` is the frozen 74-byte pool record of §11.10 | variable |
| `($, b)` | balance | `owner: [u8;32] ‖ token: [u8;32] ‖ amount: u64` | 72 |
| `(#, l)` | link | `neuron: [u8;32] ‖ p: [u8;32] ‖ q: [u8;32] ‖ height: u64` | 104 |

A frame is `0x1F ‖ sigil ‖ render ‖ LEB128(len) ‖ data` as the `tade` crate
encodes it (`Chunk::encode`); the pair column names `(sigil, render)`. A
reader rejects any other pair, any other order, a second `p`, and trailing
bytes — the file is canonical or it is not a genesis.

Canonical order: `b` frames sorted by `(owner, token)` bytewise, no
duplicates, `amount > 0`; `l` frames in the order of the source
(`cyberlinks_indexed.csv.gz` is in chain order: ascending old height, then
the tx order inside the block), duplicates kept (the old chain counted
re-links); the height-0 links of §4.2 (passports) precede the source links.
`subsidy` is written explicitly (computed by the pipeline from §3.5) so a
reader needs no arithmetic to know it. `checkpoint_key` all-zero means no
checkpoints (§5.3) and is what the pipeline writes until the foundation key
is set.

Applying genesis: every `b` becomes a balance entry and a `coins` total; every
`l` runs bbg `insert` for one link with `token = τ_native`, `amount = 0`,
`valence = 0`, authored by `neuron` at `height` (the old height is carried as
data in the link record for indexers; the chain's own height of every genesis
link is 0); every distinct owner and link author gets a neuron record with
`nonce = 0`. The resulting root is `state_root` of block 0.

### 4.2 derivation from the burial

Inputs, pinned by CID and sha256 in `bostrom/snapshot/manifest.json` and
`space-pussy/snapshot/manifest.json`:

| input | columns | use |
|---|---|---|
| `balances.csv` | address, denom, amount | liquid balances, every denom |
| `holdings.jsonl` | per account: liquid, delegated, undelegating, per-pool decomposition | delegated and undelegating amounts return to the account; LP positions dissolve into their underlying tokens at halt reserves |
| `pubkeys.csv` | address, pubkey_type, pubkey_base64 | `owner = neuron_of(pubkey)` for secp256k1 keys; address-only accounts → `legacy_owner(hrp, account_id)` |
| `cyberlinks_indexed.csv.gz` | particle_from, particle_to, neuron, height, timestamp, transaction_hash | `l` frames; `neuron` resolved as above |
| `passports.jsonl`, `passports_by_owner.json` | nickname, owner | see passports below |
| `supply.json` | total per denom | the check: Σ `b` per token equals it after decomposition |

Rules:

- denominations: all carried, each as its own token id; `boot`, `hydrogen`,
  `milliampere`, `millivolt`, `tocyb` on bostrom; `pussy`, `liquidpussy` and
  the rest on pussy; IBC and factory denoms carried under their full denom
  string as the token name;
- owners: `secp256k1` pubkeys → `neuron_of` (the pipeline recomputes the
  address from the key and refuses a mismatch); the module/contract accounts
  and address-only accounts → `legacy_owner`; multisig and non-secp256k1 key
  types → `legacy_owner` (claimable later by a fork rule, not by kind 3).
  `account_id` is the bech32 payload: 20 bytes for a key account, 32 for a
  contract (the passport contract is `legacy_owner(hrp, 32-byte id)`);
- staking pools: the delegated and undelegating amounts returned to accounts
  leave `bonded_tokens_pool` / `not_bonded_tokens_pool`; the rounding dust
  the pools still hold stays on `bonded_tokens_pool` under `legacy_owner`,
  so Σ of the native coin is exactly the bank supply at halt;
- pools: each LP position dissolves into the pool's reserve denoms,
  `floor(lp · reserve / pool_coin_supply)` per denom; the reserve account
  keeps the rounding dust; pool coins are not carried; a drained pool (no
  reserve left) dissolves to nothing;
- u64: a `b` frame and a `coins` total carry a u64; a denom with any
  balance or a total supply above 2^64 − 1 (18-decimal IBC vouchers) is not
  carried at all and is listed in the genesis report; the native coin must
  never be among them;
- stake: delegated and undelegating balances are liquid at genesis on the
  phase-1 chain — there is no staking in a pure-PoW rule set. The launch-page
  decision "stake at genesis is bonded uniformly from the snapshot" is a
  cores-era rule; the shadow staking indexer reads `delegations.csv` and the
  activation fork may re-bond from it;
- particles: the old `particle_from`/`particle_to` are IPFS CIDs; the
  pipeline re-addresses each per cyber/launch row 20 — `hemera(bytes)` when
  the bytes are in the burial blockstore, black hole `hemera(cid_string)`
  when not (2.37 % of bostrom particles) — and emits one naming link
  `hemera(cid_string) → particle` per re-addressed CID, authored by the
  genesis neuron `[0u8; 32]` at height 0, so the old address stays
  resolvable in the graph. The first genesis fetches no bytes: every CID is
  the black hole `hemera(cid_string)` and, the two identities coinciding,
  no naming link is emitted; the content particle sparks when bytes
  surface (file crate, row 20);
- passports: each of the 47,837 bostrom passports becomes one genesis link
  `owner_neuron → hemera("passport:" ‖ nickname)`, height 0; the 8.97 T BOOT
  held by the passport contract stays under
  `legacy_owner("bostrom", contract_account_id)`, dormant and unclaimable by
  kind 3 (no key derives that account id), until the owner decides — the open
  item on the launch page is untouched here;
- TOCYB: carried as the token `hemera("tocyb")` unchanged; the
  TOCYB × 666 → CYB rule belongs to $CYB genesis (phase 3) and is not
  executed by the phase-1 chain;
- pussy supply: `manifest.json` records `total_post_swap_pussy`
  (1,152,075,800,167,936,256) with a hypothetical minted-for-swap amount
  priced from pool reserves; no swap is executed at genesis (every denom is
  carried as its own token instead), so S0 for pussy is the bank supply at
  halt, 1,099,389,810,279,114,036, and `SUBSIDY = 10,458,426,657`.

The pipeline is a deterministic program — `soft3 genesis build` in
`soft3/crate/src/genesis/` (`specs/genesis.md`; `chain/genesis.rs` consumes
its output through `soft3::genesis::state::load`) — whose output file
particle is published on the launch page and in the binary; two independent
runs must produce byte-identical files (chain-conformance §10).

### 4.3 block 0

```
version = 1, height = 0, time = genesis.time, bits = GENESIS_BITS, nonce = 0,
prev = 0, tx_root = chain, state_root = root(apply(genesis)), chain = particle(genesis file)
```

No PoW check at height 0. `block_id(0)` is pinned in the binary beside the
chain id; a node whose computed genesis root differs refuses to start.

Genesis times (cyber/launch decisions 2026-09-18): pussy
2026-11-05T13:37:00Z = 1793885820; bostrom 2026-11-05T13:22:42Z =
1793884962, or the matching second of the day row 43 closes.

### 4.4 two chains, one binary

`soft3 node --chain pussy | bostrom [--genesis path]`. The binary embeds the
two chain ids and genesis block ids; `--genesis` is for rehearsal nets
(`pussy-rc`) and must match the embedded id for the named chain. All
protocol constants (§0) are identical on both chains; only the genesis file
differs. The chaosnet `spacepussy-test` keeps its current engine and port
and is not a chain of this spec.

## 5. consensus

### 5.1 fork choice — heaviest chain

```
work(header)   = floor(2^256 / (target(bits) + 1))         // u256
chainwork(B)   = chainwork(prev) + work(B)
```

The canonical tip is the valid block with the greatest `chainwork`. Ties
keep the first-seen tip (Bitcoin's rule); a deterministic hash tie-break
would hand a predictable advantage to whoever can grind for it.

### 5.2 reorg limit

A node never reorganizes more than `MAX_REORG` = 119 blocks: a competing
chain whose fork point is at or below `tip − FINALITY_DEPTH` is rejected
outright, however much work it carries, and logged. After one hour a block
is final for every node that saw it in time. The cost is honest: a partition
longer than an hour produces two chains that will not rejoin automatically;
the operator of the minority side resyncs from a checkpoint. For a chain run
on three known machines plus volunteers this trade is right; it bounds the
damage of a rented-hash-rate rewrite to one hour of history (§8).

### 5.3 foundation checkpoints, with a sunset

For the first months the chain's hash rate will be small enough that a
single GPU farm can out-mine it. The explicit, temporary mitigation is a
checkpoint signer:

- `genesis.checkpoint_key` is one secp256k1 public key held by the
  foundation;
- a checkpoint is `height: u64 ‖ block_id: [u8;32] ‖ sig: NSIG1` with
  `statement = hemera("soft3/chain/checkpoint/v1\0" ‖ chain ‖ height ‖ block_id)`
  and `subject = neuron_of(checkpoint_key)`;
- the signer publishes one checkpoint per `CHECKPOINT_INTERVAL` at heights
  that are multiples of 120 and at least `FINALITY_DEPTH` below its own tip,
  on the `checkpoints` gossip topic and at `/checkpoints`;
- a node with checkpoints enabled (default on) treats a checkpointed block
  as final: it rejects any chain not containing it and will not reorg below
  it. Checkpoints never make an invalid block valid and never move the tip
  forward — they only forbid alternatives;
- sunset: a node ignores every checkpoint with
  `height > CHECKPOINT_SUNSET` (525,600 ≈ 6 months after genesis), and the
  operator may disable them earlier (`--no-checkpoints`). The key cannot be
  rotated without a release; losing it simply ends checkpointing early.

This is centralization, named as such, bounded in power (cannot forge, cannot
censor, cannot rewrite — only veto deep reorgs) and bounded in time.

### 5.4 orphans and ordering

Headers whose `prev` is unknown go to a bounded pool (`ORPHAN_POOL`: 1,024
headers, evicted after 600 s or by PoW-weight when full) and trigger a
`get-headers` from the sender (chain-wire §3). Bodies are requested only for
headers that are on a chain with more work than the local tip or within
`FINALITY_DEPTH` of it. Every rejected block id is remembered (bounded set)
so it is never re-validated.

### 5.5 validation order

Cheap before expensive, and nothing stateful before PoW:

1. syntax: 168 bytes; `version & 0xffff == 1`; `chain` matches; `bits`
   canonical and within limits;
2. parent: `prev` known and valid, `height == prev.height + 1`;
3. time: MTP rule, future-drift rule;
4. target: `bits == expected bits from ASERT(prev)`;
5. proof of work: `block_id < target`; the header is now accepted and
   relayed;
6. body: size and count budgets, `tx_root` recomputed and equal, coinbase
   position and height;
7. transactions in order: decode, signature (`verify_statement`), nonce,
   balances, kind rules; coinbase amount equals subsidy plus fees;
8. state: apply on a copy of the parent state, `state_root` equal; commit.

A block failing 6–8 marks the header invalid (and every descendant);
failures at 1–5 never reach disk.

## 6. network

Node-to-node transport, gossip topics, header-first sync, discovery through
the graph, anti-spam, eclipse basics and the HTTP surface are specified in
[chain-wire.md](chain-wire.md). The consensus rules above do not depend on
anything there.

## 7. upgrades and shadow mode

### 7.1 rule sets by height

`version & 0xffff` names the rule set a block claims to follow. A node holds
an activation table

```
ACTIVATIONS: [(rule_set: u16, height: u64)]       // rule set 1 at height 0
```

compiled into the release and printed in `/status`. A block at height `h` is
valid only under the highest rule set whose activation height is `≤ h`, and
its `version & 0xffff` must equal that rule set. A node that does not know
the active rule set (its table ends earlier) stops accepting blocks at that
height and says so in `/status` — it does not follow a chain it cannot
validate. A scheduled fork is therefore a software release carrying the
table, announced on cyber/launch at least 40,320 blocks (two weeks) before
the height. Nothing on chain votes; this is the flag-day model, chosen
because the chain has no governance object and because every node in the
first months is run by someone reading the launch page.

`version >> 16` are miner signal bits: a miner may set them to advertise
readiness for a rule set; they are informational, never consensus.

### 7.2 what rule set 2 brings

Rule set 2 ("cores") is the activation of settlement mining, fold, foculus
consensus and privacy. It is specified by the cores' own pages; this page
only reserves what the chain must leave room for:

- the pool (§11): kinds 4 and 5, the turnstile, the commitment tree and the
  block proof, with every format frozen at genesis — rule set 2 brings only
  its verifying keys;
- further tx kinds ≥ 6 (settlement ticket, fold step, box withdraw/transfer,
  stake lock) — kinds are a u8, 250 remain;
- the coinbase `extra` field as the commitment slot for the epoch
  accumulator / beacon when they become consensus;
- the subsidy envelope re-targeted from the header hash to the settlement
  ticket (rewards.md §8), with the same `SUBSIDY` unless the activation
  table says otherwise;
- bbg dimensions today left empty (signals, time, A, N) start to be written;
  the root construction is unchanged, so rule-set-1 roots remain verifiable.

### 7.3 shadow indexers

The four cores run on every full node as indexers in phase 1: processes (or
threads) that read the block stream after validation and publish derived
results nobody is obliged to believe. The chain gives them exactly one
interface, the finalized block stream:

- `GET /v2/history?after=POSITION&limit=N` (existing contract) extended to
  carry the block id, height and state root of every operation; and
- the local event bus of the node process (`chain::Event::Block { header,
  body, state_root }`, emitted once a block is `FINALITY_DEPTH` deep —
  shadow indexers never see a reorg).

What each reads and publishes:

| indexer | reads | publishes (shadow topic / HTTP) |
|---|---|---|
| foculus consensus | the public graph (links), block clock | epoch boundaries `B_E` as if blocks were ticks (foculus/specs/epoch.md), φ*-finality verdicts; `/shadow/foculus` |
| settlement mining | the graph, the epoch beacon it derives from block ids | settlement tickets and the per-epoch Shapley estimate (rewards.md §7); `/shadow/settlement` |
| fold | published tickets | cluster accumulators and decider outputs; `/shadow/fold` |
| privacy | the tx stream | P1–P3 leakage reports on the public wire, stealth-address trial balances; `/shadow/privacy` |

Shadow topics are gossip topics named `shadow/<core>` (chain-wire §2), not
consensus, rate-limited like transactions. The point of shadow mode is that
by the activation height each core has months of outputs that the owner, the
red team and the community have compared across nodes; an indexer whose
outputs disagree between nodes is not ready to be consensus.

### 7.4 optional checkpoint proofs

Later, a zheng proof of a bbg state transition over a block range may be
attached to a checkpoint (bbg's `Checkpoint { root, acc, height }` already
has the slot). Nothing in phase 1 depends on it; a node without the proof
verifier runs identically.

## 8. security model

What is and is not protected at launch, stated plainly.

| property | status at launch |
|---|---|
| double spend inside the canonical chain | protected (nonce + balance rules, §2.5, §3.4) |
| forged transfer | protected by secp256k1 ECDSA (classical; no post-quantum claim) |
| cross-chain / cross-net replay | protected (chain id in statement) |
| supply inflation beyond schedule | protected (coinbase exactness, conservation check) |
| 51 % rewrite of recent history | not protected by work alone — hash rate at launch is tiny; bounded to one hour by `FINALITY_DEPTH` and, for six months, by checkpoints |
| censorship by a majority miner | not protected (no protocol can); visible in `/status` (miner distribution) |
| long-range attack from genesis | protected by the pinned genesis id, the reorg limit and checkpoints |
| eclipse | basics only (chain-wire §5); a determined adversary controlling a node's peers can feed it a lighter chain until it reconnects |
| timestamp manipulation | bounded to 1.149× per attacker block (§1.5) |
| spam | fee floor, size caps, per-peer limits; no fee market, so sustained spam at 1 unit/byte is cheap for a whale and fills blocks — accepted on a canary |
| mempool / network DoS | bounded pools and rate limits (chain-wire §4–5); not load-tested at launch |
| key loss | no recovery; a neuron is its key |
| privacy | none on the chain: author, amounts and links are public. P1–P3 are shadow until rule set 2 switches on the pool (§11), whose formats genesis already fixes |
| settlement / ranking correctness | not a chain property in phase 1; shadow outputs carry no money |

The work function: hemera is a Goldilocks Poseidon2 variant, GPU-friendly
(quantus precedent, goldilocks-parano1d §P2). There is no rental market for
hemera hash power, which removes the cheapest 51 % path (NiceHash-style) and
leaves the expensive one (a GPU farm pointed at the chain). The measured CPU
rate (64 kH/s per core, §1.2) makes the gap wide: a single GPU running a
competent hemera kernel is expected to out-hash every CPU miner on the
network at launch, so the first GPU is the majority miner until a second
arrives. Against that the chain has the reorg limit and the checkpoints,
nothing else; that is the honest security budget of a 1 %-per-year subsidy
on a canary, and the reason both mitigations exist.

Threats and mitigations, in one list:

| threat | mitigation | residual |
|---|---|---|
| hash-rate rental / farm rewrite | `FINALITY_DEPTH`, checkpoints to sunset | one hour of reorgable history |
| farm joins then leaves (difficulty cliff) | ASERT with H = 600 s | ≤ 66 min of slow blocks after a 100× drop |
| timestamp grinding | MTP(11), +120 s drift, parent-time ASERT | 1.149× per block |
| header spam | PoW checked before relay or storage | an attacker pays real work per header |
| tx spam | fee floor, 512 B cap, nonce window, per-peer rate | fills blocks at 1 unit/byte |
| block-size DoS | 256 KiB / 1,024 txs | none beyond cost |
| orphan-pool exhaustion | 1,024 entries, PoW-weight eviction | none |
| eclipse / partition | 8 outbound to distinct networks, hard-coded bootstrap, anchors | partition > 1 h needs an operator |
| checkpoint key compromise | key can only veto; sunset; operator switch | a compromised key can halt a node's progress on a fork until disabled |
| genesis disagreement | pinned chain id and block 0 id; byte-identical pipeline | none if the pipeline is deterministic (conformance §6) |
| state-root cost (bostrom) | launch bostrom only after bbg row 43 | pussy launches first |

## 9. node architecture

### 9.1 crate and placement

The chain is `soft3 node --chain <name>` in `soft3/crate` — the existing
node crate (rung 7 of the ladder: radio · foculus · tade), reusing `node.rs`
and the HTTP adapter, with a new `chain` module tree. Reasons: the ladder
places the network and its consensus at soft3, below cyb and cyber and above
cybergraph/bbg; the HTTP contract cyb and true-cyber consume already lives
here; and a separate `chain` crate would either sit on the same rung (no
gain) or pull radio below cybergraph (a violation). Dependencies point down
only: `hemera` (commit), `bbg` and `cybergraph` (graph), `mudra` and
`neuron-id` (language rung, identity), `tok` (conservation arithmetic),
`tade` (framing), `radio` (transport), `foculus` only for the shadow indexer
(feature `shadow`). cyb and cyber consume `soft3`; nothing in `soft3` names
them.

```
soft3/crate/src/chain/
  mod.rs          Chain: open(home, name) → tip, state, mempool; Event bus
  params.rs       §0 constants, ACTIVATIONS, embedded chain ids and genesis ids
  header.rs       Header encode/decode (168 B), block_id, bits codec (§1.1–1.3)
  pow.rs          valid_pow, work, chainwork (u256 helpers, no floats)
  daa.rs          ASERT (§1.5), MTP
  tx.rs           kinds 0–3: encode/decode, txid, statement, verify (mudra)
  pool/           §11: wide digest, record, rho, commitment, nullifier, tree node and tree, turnstile key, SPEND encoding, block statement, tags
  tree.rs         tx_root (§1.7)
  genesis.rs      tade genesis reader, apply, block 0, pipeline check
  state.rs        apply block on bbg (§3.4), conservation, nonce, maturity
  store.rs        headers (permanent), bodies, chainwork index, rejected set, bbg handle
  select.rs       fork choice, reorg limit, checkpoints (§5)
  mempool.rs      admission, nonce ordering, fee floor, eviction (chain-wire §4)
  miner.rs        template, nonce loop (CPU; pool template out), coinbase extra
  wire/           radio: topics, sync, discovery, limits (chain-wire)
  api.rs          HTTP routes (chain-wire §6) on the existing http.rs server
  shadow/         feature `shadow`: foculus/settlement/fold/privacy indexers over Event
```

Each file stays under 500 lines (quality doctrine); `wire/` and `shadow/` are
directories for that reason.

### 9.2 the three machines

Per `cybernode/`: cyberproxy (public entry, 8 GB), deimos (archive, 64 GB), jupiter
(RPC, 64 GB). Each runs one unit per chain:

| machine | unit | role | HTTP | QUIC |
|---|---|---|---|---|
| cyberproxy | `soft3-pussy.service`, `soft3-bostrom.service` | public entry behind nginx (`cyb.ai/pussy`, `cyb.ai/bostrom`), full node, no mining, checkpoint relay | 127.0.0.1:7781 / :7782 | 0.0.0.0:7791 / :7792 |
| deimos | same units | archive node, miner (CPU; the GTX 1080 when a GPU kernel exists), checkpoint signer (key on this box only) | 127.0.0.1:7781 / :7782 | 0.0.0.0:7791 / :7792 |
| jupiter | same units | full node, miner, second bootstrap | 127.0.0.1:7781 / :7782 | 0.0.0.0:7791 / :7792 |

Homes `/home/cyber/<chain>/` (`bbg/`, `blocks/`, `genesis.tade`, node key).
The three endpoint ids are the hard-coded bootstrap set; each node also
publishes its antenna and socket as cyberlinks from its own neuron
(chain-wire §5) so a fourth node can find them through the graph. The
chaosnet unit `spacepussy-test.service` on :7780 is untouched. Alerts: tip
age > 10 min, peer count < 2, conservation check failed, checkpoint older
than 2 h — on mimas like everything else.

## 10. open questions and defaults

Decisions this page could not settle alone, each with the default the
implementation uses until the owner says otherwise.

Owner decisions 2026-10-15 (recorded by the kernel implementation,
`crate/src/chain/`): every default below stands **except item 2** — in
rule set 1 a cyberlink's `amount` must be 0; there are no locked boxes,
and weights arrive with a later rule set. The kernel rejects
`amount > 0` (`TxError::Box`).

| # | question | default here | decided |
|---|---|---|---|
| 1 | subsidy rate (§3.5): 1 %/yr constant, 0.5 %, or a halving schedule | 1 %/yr constant, no halving | 1 %/yr, kept |
| 2 | locked cyberlink boxes (§2.4): lock until the cores add withdraw, or disallow `amount > 0` entirely in phase 1 | lock; withdraw arrives with rule set 2 | **amount must be 0** in rule set 1 (no boxes); weights with a later rule set |
| 3 | genesis stake (§4.2): make delegated/undelegating balances liquid, or hold them dormant until staking activates | liquid | liquid, kept |
| 4 | the 8.97 T BOOT on the passport contract | dormant under `legacy_owner`, untouched | dormant, kept |
| 5 | TOCYB × 666 → CYB | not executed; TOCYB carried as a coin; phase 3 | TOCYB as a coin, kept |
| 6 | `FINALITY_DEPTH` 120 (1 h) vs parano1d's 18 (9 min) | 120 — a canary with low hash rate should be slow to finalize | 120, kept |
| 7 | checkpoint sunset at 6 months vs owner-switched | hard sunset 525,600 blocks plus operator switch | sunset 525,600 + switch, kept |
| 8 | fee floor 1 unit/byte on both chains (very different unit values) | 1 unit/byte; revisit when a market exists | 1 unit/byte, kept |
| 9 | block time 30 s vs 60 s | 30 s | 30 s, kept |
| 10 | `NeuronRecord.nonce` in bbg vs chain-side nonce table outside the root | in bbg, under the root | in bbg, kept |
| 11 | multisig / non-secp256k1 legacy accounts | dormant; a later rule set adds their claim | dormant, kept |
| 12 | the particle definition (row 19) | `hemera(bytes)`, 32 bytes, as every crate computes today | `hemera(bytes)`, kept |

## 11. the pool (rule set 2, formats frozen at genesis)

The pool is the shielded money of
[privacy, frozen](../proposals/privacy-frozen.md): one tree of hemera
commitments, spends proven in zero knowledge by a hash-only zheng proof,
cyberlinks as public outputs of spends, one folded proof per block. It is
off at launch: rule set 1 knows none of its transaction kinds, and the
chain is public until rule set 2 activates; privacy is not retroactive
(proposal §1.13). What is on board at genesis is the material and the
formats of this section. The genesis `p` frame ends with the pool record
of §11.9, so the chain id commits to every constant below; the activation
of rule set 2 adds verifying keys (§11.10) and changes nothing written here.

Notation as in the rest of this page: `hemera(x)` is the flat sponge
`hemera::hash` (32 bytes), `‖` is concatenation, integers are
little-endian. Every value is computed through the hemera API, never from
pinned constants. The chain freezes hemera's profile #15 (x⁷ in every
round, 6 + 6 full rounds, 48 partial; hemera 0.4); the values quoted today
are computed against hemera 0.3 (`d5f0a09`) and are re-blessed, with the
genesis particle, when hemera 0.4 lands. The definitions do not change. Reference implementation: `crate/src/chain/pool/`; vectors:
chain-conformance §12.

### 11.1 wide digests and canonical limbs

Every consensus-critical digest of the pool — owner, rho, commitment,
nullifier, predicate binding, tree node, spend id, statement id, verifying
key set, tags — is wide: 64 bytes, eight Goldilocks elements. A 32-byte
hemera digest has about 85 bits of generic quantum collision resistance
(Brassard–Høyer–Tapp, `2^(n/3)`); eight elements give about 170, above the
128 the proposal requires (§1.3: at least 48 bytes), and a format frozen at
genesis can never be widened. The 32-byte particle stays a locator — token
ids, data, link endpoints, the chain id, transparent card ids — never an
authority over value or program.

```
W(x) = xof(x)[0..64]              // two squeezes of 32 bytes
```

`W` is hemera's own sponge, not a second hash: the plain mode of
`hemera::Hasher` absorbs `x` exactly as `hemera(x)` does, and `W(x)` is the
first 64 bytes of its extendable output (`Hasher::finalize_xof`) — the
first squeeze is `hemera(x)` itself, the second follows one more
permutation. Each 8-byte group is a canonical element. Domain separation
is the pool tag every `W` input begins with (§11.9); consensus never uses
the narrow digest of a pool-tagged input. Cost: one permutation more than
`hemera(x)`.

Canonical limbs: every 32-byte and 64-byte field of every pool encoding
(records, deposits, `spend_public`, the statement) is read as 8-byte
little-endian limbs, and every limb must be `< p = 2^64 − 2^32 + 1`. A
decoder refuses a limb `≥ p`: a field element `x` and `x + p` would name
one value twice. Integers (`value`, `weight`, `amount`, `fee`, counts,
lengths) are not limbs and take any value of their type.

### 11.2 record

```
record = token: [u8;32] ‖ value: u64 ‖ owner: [u8;64] ‖ rho: [u8;64] ‖ r: [u8;32] ‖ data: [u8;32] ‖ predicate: [u8;64]
```

| field | bytes | offset | meaning |
|---|---|---|---|
| `token` | 32 | 0 | token id (§3.1), a locator |
| `value` | 8 | 32 | amount of `token`, any u64 |
| `owner` | 64 | 40 | `W(D_OWNER ‖ sk)`, `sk` a 32-byte secret |
| `rho` | 64 | 104 | uniqueness nonce, derived (§11.3), never chosen by a sender |
| `r` | 32 | 168 | commitment randomness |
| `data` | 32 | 200 | a particle; zero bytes = none |
| `predicate` | 64 | 232 | predicate binding `W(D_PRED ‖ program)` over the nox program's canonical bytes; zero bytes = none |

296 bytes, fixed, every limb canonical.

### 11.3 owner, rho, commitment, nullifier

```
owner      = W(D_OWNER ‖ sk)                       // 20 + 32 bytes in
commitment = W(D_CM ‖ record)                      // 17 + 296 bytes in
nullifier  = W(D_NULL ‖ chain ‖ sk ‖ rho)          // 24 + 32 + 32 + 64 bytes in
rho_j      = W(D_RHO ‖ nf_0 ‖ j: u8)               // output j of a spend whose first nullifier is nf_0; 18 + 64 + 1 in
rho        = W(D_RHO ‖ txid ‖ 0: u8)               // a deposited record, txid of its deposit (§2.1); 18 + 32 + 1 in
```

`chain` is the chain id: a nullifier of pussy is never one of bostrom.
The rho of every output is derived inside the proof from the spend's first
nullifier, and the rho of a deposit by every node from its txid; the two
seeds differ in length (64 and 32 bytes, the length is in the capacity),
so the derivations never meet. Since nullifiers are unique for the life of
the chain (§11.5), no two records ever share a rho, and no sender can give
two records one nullifier to make one of them unspendable. Inputs carrying
`sk` are hashed in constant time (`hemera::Hasher::new_secret`, the same
digest).

### 11.4 commitment tree and anchors

- one tree of depth 40: positions `0 … 2^40 − 1`, filled in append order —
  blocks in height order, transactions in body order, a transaction's
  commitments in their listed order. A transaction that would append past
  position `2^40 − 1` is invalid.
- a leaf is the commitment itself (a tagged wide digest); an unfilled
  position holds `EMPTY = W(D_EMPTY)`.
- a node is the wide digest of its tagged children:

  ```
  node(L, R) = W(D_NODE ‖ L ‖ R)                    // 19 + 64 + 64 = 147 bytes in
  ```

  Cost at hemera's 56-byte rate (7-byte packing): three absorbing
  permutations, the last carrying the padding, and one for the second
  squeeze — four per node, where the narrow node jet
  (`hemera::tree::hash_node`, zheng#66) takes one over two four-limb
  children. A depth-40 path is 160 permutations natively and in the
  circuit. The construction uses only the sponge API; a dedicated wide
  node jet (children absorbed as elements, two permutations at width 16,
  one at width 24) would be a different format and can only be adopted
  with a re-blessing before genesis, never after.
- `E_0 = EMPTY`, `E_i = node(E_{i−1}, E_{i−1})`; `E_40` is the root of the
  empty tree. The depth is fixed, so the root needs no flag of its own. A path is verified leaf to root; bit `i` of the position set means
  the sibling at level `i` is on the left.
- `root_after(b)` is the tree root after every pool transaction of block
  `b`; `root_after(0) = E_40` (genesis holds no commitment, and rule-set-1
  blocks append none).
- anchor window: a spend in block `h` is valid only if its `anchor` equals
  `root_after(b)` for some `b` with `max(0, h − 1,000) ≤ b ≤ h − 1`. A
  spend never anchors on commitments of its own block; a node keeps the
  last 1,000 roots. A membership proof against a historical root proves
  that a record existed, never that it is unspent; unspentness is the
  nullifier check of §11.5.

### 11.5 the nullifier set and the turnstile

The nullifier set is lifetime: every nullifier ever revealed stays in it,
and it is never reset, pruned by epoch or rebuilt from a live window (an
epoch reset accepts an already spent nullifier; proposal §1.10). A node
may compact its storage only where an authenticated proof of
non-membership over the whole history still rejects every old nullifier.
A spend is invalid if any of its nullifiers is in the set or appears twice
in its block.

```
T(τ) = hemera(D_TURNSTILE ‖ τ)                     // 24 + 32 = 56 bytes in, 32 bytes out
```

`T(τ)` is a raw key of bbg's balances dimension (§3.2) holding the pool's
counter for token `τ`: a key of the transparent ledger, 32 bytes like every
balance key. A transparent key `hemera(owner ‖ τ)` hashes 64 bytes and the
input length is in the capacity, so no transfer, coinbase, claim or genesis
frame can address `T(τ)`; aiming one at it is a preimage search against a
fixed target among the chain's few genesis tokens, about 2^128 by Grover.
The counter is absent — zero — at genesis and is under `state_root` from
block 0.

| event | effect |
|---|---|
| deposit of `value` of `τ` (§11.7) | `T(τ) += value` |
| withdrawal `(to, τ, amount)` of a spend | `T(τ) −= amount`; `balances[hemera(to ‖ τ)] += amount` |
| fee `(τ_f, fee)` of a spend | `T(τ_f) −= fee`; paid to the block's miner through the coinbase |

A transaction whose debit would take any `T(τ)` below zero is invalid,
whatever its proof says. Hence, per token, at every height:

```
T(τ) = Σ deposits(τ) − Σ withdrawals(τ) − Σ spend fees(τ) ≥ 0
```

The pool's content can never exceed what entered it: a break of the proof
system or of hemera counterfeits at most `T(τ)` and never raises the supply.
Conservation (§3.4) is unchanged — deposits and withdrawals move value
between keys of one dimension, and the per-token sums include `T(τ)`.

### 11.6 the SPEND public inputs

One relation, `SPEND`. Its public inputs, encoded:

```
spend_public = chain: [u8;32]
             ‖ anchor: [u8;64]
             ‖ n: u8 ‖ nullifier[64] × n
             ‖ m: u8 ‖ commitment[64] × m
             ‖ k: u8 ‖ (p[32] ‖ q[32] ‖ weight: u64 ‖ payout[64]) × k
             ‖ w: u8 ‖ (to[32] ‖ token[32] ‖ amount: u64) × w
             ‖ fee_token: [u8;32] ‖ fee: u64
             ‖ predicate[64] × (n + m)
             ‖ c: u8 ‖ (len: u16 ‖ note[len]) × c
```

Canonical form, checked by every decoder:

- `1 ≤ n ≤ 4` inputs, `0 ≤ m ≤ 4` outputs, `0 ≤ k ≤ 4` cyberlinks,
  `w ∈ {0, 1}`, `0 ≤ c ≤ m` notes; a count over its cap is refused before
  anything it counts is read;
- no nullifier twice and no commitment twice within one spend;
- every cyberlink has `p ≠ q`;
- a withdrawal has `amount > 0` (no withdrawal is written `w = 0`);
- the predicates are the bindings of the inputs in order, then of the
  outputs in order; zero bytes means none;
- note `i` belongs to output `i`; every note has `len ≤ 2,048`; notes are
  opaque to consensus (an ML-KEM-1024 envelope bound to its output
  commitment, by wallet convention, proposal §1.2);
- every limb canonical (§11.1); no trailing bytes. The longest canonical
  encoding is 9,981 bytes.

A cyberlink output `(p, q, weight, payout)` is public: `weight` is a u64
that does not enter the value equation below; `payout` is an owner value
(§11.3) that receives what the link earns. The spend's name in the block
statement is

```
spend_id = W(D_SPEND ‖ spend_public)
```

The witness: the input records, their Merkle paths and `sk`, the output
records, the predicate programs' witnesses. Checks inside the proof, fixed
with the relation:

1. every input record is canonical, has `owner = W(D_OWNER ‖ sk)`, and its
   commitment sits in the tree at `anchor` (§11.4);
2. every nullifier is `W(D_NULL ‖ chain ‖ sk ‖ rho)` of its input;
3. conservation as integers, never modulo `p`: every value, `fee` and
   `amount` is a u64 held as two 32-bit limbs, each range-checked
   `< 2^32`; per token `τ`, Σ input values and Σ output values +
   `fee`·[τ = fee_token] + `amount`·[τ = token of the withdrawal] are each
   summed limb by limb with explicit carries into a third limb (at most
   six terms a side, below 2^67), and the two sums are equal limb for limb (the
   u64 value `p` is zero in the field and would pass a field equation);
4. every output commitment is `W(D_CM ‖ record)` of its canonical output
   record, and output `j`'s rho is `W(D_RHO ‖ nf_0 ‖ j)`;
5. every predicate of an input or an output, run on `spend_public` and its
   own witness, accepts (the nox machine relation, `zheng/specs/machine.md`);
   predicates only add constraints and never relax checks 1–4;
6. the public outputs are those the records and predicates declare.

Checks outside the proof, by every node: `chain` equals the chain id; the
anchor is in the window (§11.4); no nullifier is in the lifetime set or
repeated in the block (§11.5); the turnstile stays non-negative; `fee_token`
is the native token and `fee ≥ len(tx) · MIN_FEE_PER_BYTE`; the block proof
verifies (§11.8).

### 11.7 transaction kinds 4 and 5

| kind | name | body bytes | sig | total bytes |
|---|---|---|---|---|
| 4 | deposit | 280 | NSIG1 (102) | 384 |
| 5 | spend | `2 + len` | none | `4 + len`, at most 9,985 |

```
deposit = from: neuron[32] ‖ nonce: u64 ‖ token: [u8;32] ‖ value: u64 ‖ owner: [u8;64] ‖ r: [u8;32] ‖ data: [u8;32] ‖ predicate: [u8;64] ‖ fee: u64
spend   = len: u16 ‖ spend_public[len]
```

The envelope, `txid` and the signing statement are those of §2.1.

Deposit (transparent to commitment, no proof): the record without its rho,
which the node derives as `W(D_RHO ‖ txid ‖ 0)`. Signed by `from` as kinds
1 and 2; the nonce rule of §2.5; `fee ≥ 384 · MIN_FEE_PER_BYTE` in the
native coin; `value > 0`; every limb canonical; `from` holds `value` of
`token` (the native coin also covers the fee and respects maturity, §2.2).
Effects: `balances[hemera(from ‖ token)] −= value`, `T(token) += value`,
the fee to the coinbase, `commitment(record)` appended to the tree,
`nonce += 1`. A deposit reveals that record's owner and rho; its later
nullifier still needs `sk`, so the spend of a deposited record is linkable
to the deposit by value and timing only. Wallets deposit to a one-time
`sk` and spend into fresh records.

Spend: no signature — authorization is knowledge of `sk` inside the proof;
`len` is the exact length of a canonical `spend_public`. Effects, in this
order: the nullifiers join the lifetime set; the commitments are appended;
the withdrawal and the fee leave the turnstile (§11.5); the cyberlinks are
public cyberlinks. The transaction's own proof travels in the mempool only
(chain-wire §4), never in a block. A spend is bounded by 9,985 bytes, not
by `MAX_TX_BYTES`; a deposit fits `MAX_TX_BYTES`.

Under rule set 1 the decoder knows neither kind (§2.1): a block carrying
one is invalid.

### 11.8 the block-proof statement

A block of rule set 2 with at least one spend carries one folded proof (the
constant-size profile of `zheng/specs/recursion.md`) whose public input is

```
statement    = chain: [u8;32] ‖ height: u64 ‖ vk_set: [u8;64] ‖ count: u32 ‖ spend_id[64] × count
statement_id = W(D_BLOCK ‖ statement)
vk_set       = W(D_VK ‖ (len: u32 ‖ vk) × v)
```

`spend_id`s are listed in body order, one per kind-5 transaction; `vk_set`
digests the rule set's verifying keys in the order its activation entry
lists them. The statement is defined as the conjunction of the listed
spends' `SPEND` statements under those keys: a reference validator that
checks raw transaction proofs one by one reaches the same verdict on every
block. Nodes verify the block proof and never a transaction's proof. A
block without spends carries no proof.

### 11.9 domain tags

Chosen once, ASCII with a terminating NUL byte, never reused for any other
input; disjoint from the chain's `soft3/chain/…` tags (§1.7, §2.1).

| tag | bytes | length | hex |
|---|---|---|---|
| `D_OWNER` | `soft3/pool/owner/v1\0` | 20 | `736f6674332f706f6f6c2f6f776e65722f763100` |
| `D_CM` | `soft3/pool/cm/v1\0` | 17 | `736f6674332f706f6f6c2f636d2f763100` |
| `D_NULL` | `soft3/pool/nullifier/v1\0` | 24 | `736f6674332f706f6f6c2f6e756c6c69666965722f763100` |
| `D_RHO` | `soft3/pool/rho/v1\0` | 18 | `736f6674332f706f6f6c2f72686f2f763100` |
| `D_PRED` | `soft3/pool/predicate/v1\0` | 24 | `736f6674332f706f6f6c2f7072656469636174652f763100` |
| `D_EMPTY` | `soft3/pool/empty/v1\0` | 20 | `736f6674332f706f6f6c2f656d7074792f763100` |
| `D_NODE` | `soft3/pool/node/v1\0` | 19 | `736f6674332f706f6f6c2f6e6f64652f763100` |
| `D_SPEND` | `soft3/pool/spend/v1\0` | 20 | `736f6674332f706f6f6c2f7370656e642f763100` |
| `D_BLOCK` | `soft3/pool/block/v1\0` | 20 | `736f6674332f706f6f6c2f626c6f636b2f763100` |
| `D_TURNSTILE` | `soft3/pool/turnstile/v1\0` | 24 | `736f6674332f706f6f6c2f7475726e7374696c652f763100` |
| `D_VK` | `soft3/pool/vk/v1\0` | 17 | `736f6674332f706f6f6c2f766b2f763100` |
| `D_TAGS` | `soft3/pool/tags/v1\0` | 19 | `736f6674332f706f6f6c2f746167732f763100` |

```
tags = W(D_TAGS ‖ D_OWNER ‖ D_CM ‖ D_NULL ‖ D_RHO ‖ D_PRED ‖ D_EMPTY ‖ D_NODE ‖ D_SPEND ‖ D_BLOCK ‖ D_TURNSTILE ‖ D_VK)
```

Its value against hemera 0.3 is in chain-conformance §12.

### 11.10 what genesis commits to

The params frame of the genesis file (§4.1) ends with the pool record:

```
pool = format: u32 (= 1) ‖ depth: u8 (= 40) ‖ anchor_window: u32 (= 1,000) ‖ digest: u8 (= 64) ‖ tags: [u8;64]      // 74 bytes
     = 01000000 28 e8030000 40 ‖ tags
```

A reader rejects a genesis whose pool record differs in any byte from this
one. `format = 1` names this section as a whole: the record, the
derivations, the encodings, the caps (4 inputs, 4 outputs, 4 cyberlinks,
2,048-byte notes), the tree node, limb conservation, the lifetime
nullifier set, the turnstile and the statement shape. Through the record
the chain id commits to them, and through the chain id every transaction
and header.

### 11.11 rule sets

- Rule set 1 (launch): kinds 4 and 5 are unknown and invalid; `T(τ)` is
  zero for every token; the tree is empty and every `root_after(b)` is
  `E_40`; the nullifier set is empty. Nothing in rule set 1 reads or
  writes pool state, and everything it records is public forever.
- Rule set 2 (the pool) activates by the activation table (§7.1) when the
  gates of the proposal pass; the privacy promise holds from its
  activation height. Its activation entry carries the verifying key set
  of `SPEND` and of the block proof — and nothing else of this section: no
  format, tag, width, depth, window or accounting rule changes with it. A
  verifying key set, once in a rule set, is never changed; a new one is a
  new rule set.
- Where rule set 2 stores the commitment list and the nullifier set under
  `state_root` (bbg's A and N dimensions, empty and committed today, §3.2)
  is its choice; the tree root of §11.4 is a function of the append order
  alone and is what anchors and proofs use.

## 12. acceptance

The chain is done when chain-conformance.md is green on the reference
implementation, two nodes on two machines mine and sync pussy-rc from the
real genesis file for 24 h with zero conservation failures, a third node
joins from the graph alone (bootstrap list emptied) and reaches the tip, a
rented-hash-rate rehearsal (one GPU box) produces a reorg that the reorg
limit and checkpoints bound as specified, and cyb reads the chain through
the HTTP surface of chain-wire §6. Receipts go to `soft3/audit/chain/`.

See [chain-wire](chain-wire.md) for the network, [chain-conformance](chain-conformance.md)
for the vectors, [native-node](native-node.md) for the HTTP acceptance contract
this chain inherits, and [cyber/launch](../../cyber/launch.md) for the phase it serves.
