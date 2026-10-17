---
title: chain-economy — the focus economy of the root chain
tags: cyber, soft3, spec, chain, consensus, economics, focus, staking, settlement, tru, foculus
crystal-type: spec
crystal-domain: cyber
status: accepted
date: 2026-10-17
alias: focus economy, chain economy, root chain economy, epoch settlement, staked cyberlinks, focus pool
---

# chain-economy — the focus economy of the root chain

Owner order, 2026-10-17: "the goal is foculus, stakes with valence, and
mining — not just relaunching a network on the new stack". This page makes
the root chain of [chain.md](chain.md) pay for knowledge from its first
block: a cyberlink carries locked stake and a valence, the chain cuts its
history into epochs of a fixed number of blocks, every full node computes
the focus φ* of the staked graph at each epoch boundary with tru's
tri-kernel, attributes the epoch's change of syntropy to the stake that
caused it by one gradient solve, and pays the attributed stake from a focus
pool fed by a share of the block subsidy. Proof-of-work miners keep the
rest of the subsidy and all fees.

Binding decisions (coordinator, 2026-10-17), the contract of this page:

1. block ordering stays Nakamoto proof of work (chain.md §1, §5);
2. an epoch is a fixed number of blocks; the epoch beacon is the id of the
   epoch's boundary block; the boundary is a consensus object (closes
   cyber/launch row 41; foculus's sequential-squaring VDF is retired from
   the root chain);
3. one φ*: tru's fixed-point tri-kernel; `foculus::phi` is retired;
4. epoch settlement is verified by recomputation on every full node, no
   proof on the critical path; attribution of Δφ⁺ to links is the gradient
   (integrated-gradient) form of Shapley of tru/specs/rewards.md §4,
   computed by one adjoint solve, never by per-contributor recomputes;
5. a cyberlink carries a locked stake (conviction) and a valence
   v ∈ {−1, 0, +1}; only staked links earn;
6. everything on this page is active from genesis: no shadow mode, one rule
   set at launch.

This page amends chain.md: §2.2 (coinbase amount), §2.4 (cyberlinks carry
stake), §3.2 (dimensions written), §3.4–§3.5 (transition, subsidy split),
§7 (rule set 1 now carries the economy) and §10 item 2 (superseded). Where
the two pages differ, this page wins for the economy and chain.md for
everything else. It descends from tru/specs/rewards.md and fixes the
parameters rewards.md leaves open for the root chain (rewards.md §16
records them on the tru side).

Design rules, as in chain.md: integers and fixed point only, never a float
in a consensus path; one valid byte form per value; every derived quantity
is a pure function of finalized chain state.

## 0. parameters

Chain-wide constants, identical on pussy and bostrom. `S0` and `SUBSIDY`
are chain.md §0.

| name | value | unit | where used |
|---|---|---|---|
| `EPOCH_LENGTH` (L) | 240 | blocks (2 h at 30 s) | §2 |
| `SETTLE_DELAY` (D) | 120 = `FINALITY_DEPTH` | blocks (1 h) | §2.5, §5.7 |
| `MIN_LOCK` | 20,160 | blocks (7 days) | §3.4 |
| `FOCUS_MAX_BPS` | 5,000 | basis points of `SUBSIDY` | §6.1 |
| `ALPHA` | 1/2 (integer square root) | exponent of the split curve | §6.1 |
| `POOL_RELEASE` | 4 | budget = pool / 4 per settlement | §5.5 |
| `R_MAX_PPM` | 10,000 | parts per million of a position's new stake, per settlement | §5.5 |
| `HWM_KEEP` | 65,410 / 65,536 | per epoch (half-life 360 epochs = 30 days) | §5.3 |
| `PHI_STEPS` | 48 (recommended; fixed by tru's `EpochParams` at the phase1.toml pin) | tri-kernel steps per solve | §4.4 |
| `SETTLE_BUDGET` | 900 | seconds of reference settlement compute | §7.2 |
| `RHO_LAUNCH` | 1 | surprise weight of every active position | §5.4 |
| `KARMA_LAUNCH` | 1 | karma multiplier of every neuron | §3.7 |

Derived, pussy (`S0` = 1,099,389,810,279,114,036, `SUBSIDY` =
10,458,426,657) and bostrom (`S0` = 482,287,925,778,234, `SUBSIDY` =
4,587,974): epochs per day 12; first settlement at height 360 (epoch 1);
focus pool per epoch at the cap `240 · floor(SUBSIDY / 2)`.

## 1. what is consensus at genesis

| core of cyber/launch | at genesis (rule set 1) | later |
|---|---|---|
| foculus — the epoch clock | the boundary is a block, the beacon is its id (§2) | convergence finality (φ*ᵢ > τ) stays outside the root chain; ordering is Nakamoto |
| tru — φ* | computed by every full node at every boundary over the staked graph (§4) | karma, market price and will enter `A_eff` with the reports rule set (§7.4) |
| staking with valence | kinds 2, 6, 7: lock with a link, unlock, payout owner (§3) | slashing by BTS redistribution with the reports rule set |
| settlement mining | settlement by recomputation on every node; mining is PoW block production (§7.1) | the ticket lottery and fold, by scheduled fork when recompute stops fitting (§7.3) |
| emission | subsidy split between miners and the focus pool by `θ^½` (§6) | PID control of the split and the floor (rewards.md §10) |
| privacy | none; the pool's formats are frozen (chain.md §11) | rule set 2 when its gates pass |

Decision 6 is read as: everything this page specifies is in force from
block 0, under rule set 1. The pool of chain.md §11 stays rule set 2: it
needs verifying keys that the proof gates of privacy-frozen.md have not
produced, and activating it with no keys would make every spend invalid.

## 2. epochs

### 2.1 numbering and boundary

```
epoch(h)    = ceil(h / L)            // h ≥ 1; height 0 belongs to epoch 0
boundary(E) = E · L                  // the last block of epoch E, E ≥ 0
```

Epoch 0 is the genesis block alone. Epoch E ≥ 1 is the blocks
`(E − 1) · L + 1 … E · L`. The boundary block of epoch E is the block at
height `E · L` on the chain being validated; the boundary state `G_E` is
the state after that block (chain.md §3.4), and its root is that block's
`state_root`. Two nodes agree on E, the boundary and the boundary state
from headers and bodies alone — no gossip of epoch objects, no state
copy (launch row 41's gate, met by construction).

### 2.2 length

`L = 240` blocks, two hours at the target interval. The reasons, in order
of force:

- observability: the chain launches on 2026-11-05 and the economy has to
  be seen working that day — the first payout lands at height 360, about
  three hours after genesis, and twelve settlements a day give a red team
  twelve chances a day to break one;
- pipeline: settlement of epoch E is applied at `boundary(E) + D`
  (§2.5); `L ≥ D + 1` keeps at most one settlement in flight, so a node
  never computes two epochs at once;
- cost: a settlement is three tri-kernel solves over the whole graph
  (§5.1); at the measured 3–5 minutes on bostrom's 3 M links (§7.2) twelve
  a day cost a full node under an hour of one core per day;
- path resolution: the integrated gradient is evaluated once per epoch
  (§5.2), so a shorter epoch follows the path of the graph more finely;
  the sum over epochs telescopes to the same total change of syntropy for
  any L, so L moves only how credit is split along the path, and two hours
  is coarse enough that ordering by latency inside an epoch earns nothing
  (all links of one epoch are one coalition).

A day (2,880 blocks) is the alternative and the natural length once the
graph outgrows recomputation; changing L is a scheduled fork like any
constant.

`bbg::state::EPOCH_BLOCKS` (100) is bbg's storage pruning cadence and is
unrelated to this clock.

### 2.3 beacon

```
beacon(E) = block_id(boundary(E))         // chain.md §1.1, 32 bytes
```

The beacon is the hash of the boundary block, nothing else. At genesis no
rule reads it: settlement by recomputation (§5) is deterministic and needs
no randomness. It is recorded in the epoch record (§5.6) and is the seed of
the ticket rule set (§7.3), where its one weakness matters: the miner of a
boundary block can withhold a found block to draw another beacon, paying the
block's reward per draw. That bias is bounded by the subsidy and is
accepted for the ticket rule set's orderings, which are only sampling
seeds; nothing that pays a winner takes its randomness from a single
beacon.

### 2.4 the snapshot

At the boundary of E the following are frozen as `G_E`, and settlement
reads nothing else:

- every position (card, §3.2): its author, axon, valence, stake and
  payout owner;
- the genesis weights (§4.1), constant for the life of the chain;
- the axon list (`axon_edges`) giving `(p, q)` of every axon;
- the total native supply `S0 + boundary(E) · SUBSIDY`;
- the side high-water marks `H⁺, H⁻` of every axon as left by the
  settlement of epoch E − 1 (§5.3).

The focus pool balance is read at the settlement block (§5.5), not at the
boundary.

### 2.5 settlement height, finality and reorgs

Settlement of epoch E ≥ 1 is a step of the state transition of the block
at height

```
settle(E) = boundary(E) + D          // D = FINALITY_DEPTH = 120
```

- When `settle(E)` is applied, `boundary(E)` is 120 deep; by chain.md §5.2
  no node that saw it in time reorganizes below it, so the snapshots
  `G_{E−1}` and `G_E` are final.
- Everything settlement reads is a function of the two final snapshots
  and of the pool balance at `settle(E)`, and the pool balance is itself a
  function of height once the boundary is final (§6.2). A reorganization
  that replaces the block at `settle(E)` (or any block after the boundary)
  therefore reproduces the same payouts at the same height; payouts are
  spendable at once and need no maturity.
- A reorganization that replaces the boundary block itself (fewer than
  120 blocks deep) replaces the snapshot; nodes compute settlement for the
  boundary on their best chain and recompute when it changes. Each
  candidate boundary costs its miner a valid block; the recompute is
  bounded by §7.2.
- A node starts computing at the boundary and has D blocks (one hour) to
  finish. A node that has not finished cannot validate `settle(E)` and
  waits; a miner that has not finished cannot build it. This is the
  liveness bound, and §7.2 is the margin.

## 3. positions: staked cyberlinks

### 3.1 identities

```
card(author, p, q) = hemera(D_CARD  ‖ author ‖ p ‖ q)        // 20 + 96 = 116 bytes in
stake_key(card)    = hemera(D_STAKE ‖ card)                  // 21 + 32 = 53 bytes in
focus_key(τ)       = hemera(D_FOCUS ‖ τ)                     // 21 + 32 = 53 bytes in
```

A position is one card: one author's stake on one axon `(p, q)`. Its stake
is the native balance at `stake_key(card)`; the focus pool of a chain is
the native balance at `focus_key(τ_native)`. Both are raw keys of the
balances dimension (chain.md §3.2), like the turnstile `T(τ)` of §11.5: a
transparent key `hemera(owner ‖ token)` hashes 64 bytes and the input
length is in the capacity, so no transfer, coinbase, claim or genesis frame
can address a stake or the pool. A transfer to a card id credits
`hemera(card ‖ τ)`, an ordinary balance that no key controls — a burn by
the sender's choice, never stake.

Domain tags, ASCII with a terminating NUL, disjoint from every tag of
chain.md:

| tag | bytes | length | hex |
|---|---|---|---|
| `D_CARD` | `soft3/chain/card/v1\0` | 20 | `736f6674332f636861696e2f636172642f763100` |
| `D_STAKE` | `soft3/chain/stake/v1\0` | 21 | `736f6674332f636861696e2f7374616b652f763100` |
| `D_FOCUS` | `soft3/chain/focus/v1\0` | 21 | `736f6674332f636861696e2f666f6375732f763100` |
| `D_PHI` | `soft3/chain/phi/v1\0` | 19 | `736f6674332f636861696e2f7068692f763100` |
| `D_EPOCH` | `soft3/chain/epoch/v1\0` | 21 | `736f6674332f636861696e2f65706f63682f763100` |
| `D_PAYOUT_LEAF` | `soft3/chain/payout-leaf/v1\0` | 27 | `736f6674332f636861696e2f7061796f75742d6c6561662f763100` |
| `D_PAYOUT_NODE` | `soft3/chain/payout-node/v1\0` | 27 | `736f6674332f636861696e2f7061796f75742d6e6f64652f763100` |

### 3.2 the card record

Positions live in bbg's `cards` dimension, keyed by the card id. The record
grows from `CardRecord { owner, particle }` to

| field | type | meaning |
|---|---|---|
| `owner` | `[u8; 32]` | payout owner: the card id of the account credited with the position's rewards |
| `particle` | `[u8; 32]` | the axon-particle `axon(p, q) = hemera(p ‖ q)` |
| `author` | `[u8; 32]` | the neuron that locked the stake; only it may unlock |
| `valence` | `u8` | `v + 1` ∈ {0, 1, 2}; fixed while the stake is positive |
| `since` | `u64` | height of the last stake increase |

bbg change requested (rung 3): `CardRecord` gains `author`, `valence` and
`since`, field order as listed, `DIMENSION_VERSION` 4. The stake amount is
not a field: it is the balance at `stake_key(card)`, so conservation (§6.3)
counts it with no new rule. A card exists exactly while its stake is
positive; at zero the record is deleted. `(p, q)` of an axon is read from
`axon_edges`, which every link already writes.

bbg's own `insert` moves `amount` between `balances[hemera(p ‖ τ)]` and
`balances[hemera(q ‖ τ)]` and writes `energy`/`weight`; the chain does not
use that path. Every cyberlink, staked or not, calls `insert` with
`amount = 0` (topology only, as rule set 1 does today), and stake accounting
is the chain's, through the balances and cards dimensions.

`NeuronRecord.focus`, `.karma` and `.stake` stay 0 (§3.7);
`ParticleRecord` is not written by the economy (φ* is published by digest
and over HTTP, §5.6, §11 — writing 3 M particle records per epoch would
cost about half an hour of root updates at the measured 0.4–0.9 ms each).

### 3.3 lock: kind 2 with `amount > 0`

The cyberlink encoding of chain.md §2.4 is unchanged (153-byte body, 257
bytes). Rule set 1 now admits `amount > 0`:

- `amount > 0` requires `token = τ_native`; any other token with a positive
  amount is invalid;
- `c = card(from, p, q)`; if the card exists, `tx.valence` must equal the
  card's valence (a position changes side only by unlocking to zero and
  locking again);
- `from` must hold `amount + fee` of spendable native coin (coinbase
  maturity applies, chain.md §2.2);
- effects: `balances[hemera(from ‖ τ)] −= amount + fee`;
  `balances[stake_key(c)] += amount`; the card is created with
  `owner = from, particle = axon(p, q), author = from, valence = tx.valence`
  if absent; `since = height` in either case; bbg `insert` with
  `amount = 0`; `nonce += 1`.

`amount = 0` keeps its meaning: a bare assertion, topology only, no card,
no weight in φ*, no reward.

### 3.4 unlock: kind 6

```
unlock (kind 6): from: neuron[32] ‖ nonce: u64 ‖ p: particle[32] ‖ q: particle[32] ‖ amount: u64 ‖ fee: u64
```

Body 120 bytes, NSIG1 signature, 224 bytes in total; envelope, txid and
statement of chain.md §2.1. Valid iff the signature verifies for `from`,
the nonce is current, `c = card(from, p, q)` exists, `0 < amount ≤
balances[stake_key(c)]`, and `height ≥ card.since + MIN_LOCK`. Effects:
`balances[stake_key(c)] −= amount`; `balances[hemera(from ‖ τ_native)] +=
amount`; then the fee is debited from that balance (an unlock may pay its
fee from what it returns); the card is deleted if its stake reaches 0;
`nonce += 1`. Partial unlocks are allowed; any increase restarts the lock
for the whole position (`since`). Rewards already paid are not touched:
they were credited to the payout owner at settlement and are ordinary
balances. What an unlock gives up is future credit, through the high-water
mark of §5.3.

`MIN_LOCK = 20,160` blocks (seven days). It must be at least `L + D` so a
position increased in epoch E is still locked at `settle(E)` (§5.4); seven
days makes the lock a costly signal — conviction — rather than a formality,
and short enough that canary users are not trapped. The owner may change it
(§12).

### 3.5 payout owner: kind 7

```
payout (kind 7): from: neuron[32] ‖ nonce: u64 ‖ p: particle[32] ‖ q: particle[32] ‖ payout: [u8;32] ‖ fee: u64
```

Body 144 bytes, NSIG1, 248 bytes in total. Valid iff `card(from, p, q)`
exists and the nonce and fee rules hold. Effect: `card.owner = payout`. The
payout owner is any 32-byte account id; rewards of a position go to
`balances[hemera(owner ‖ τ_native)]`. This is privacy-frozen.md §7
("rewards attach to links, not neurons; the payout key in the link receives
what the link earns") in its transparent form: settlement pays positions,
never neurons, and the payout account is the position's choice.

Kinds 4 and 5 remain the pool's (chain.md §11.7). Kinds 8 (settlement
ticket), 9 (fold step) and 10 (report) are reserved for the later rule sets
of §7.3–§7.4; under rule set 1 they are invalid.

### 3.6 fees

Kinds 6 and 7 pay `fee ≥ len · MIN_FEE_PER_BYTE` like every signed kind;
fees go to the block's miner through the coinbase (chain.md §2.6). There is
no fee on settlement: it is a state transition of the settlement block, not
a transaction, so no one can censor a payout or be paid to include it.

### 3.7 karma, will and price at launch

`A_eff` of rewards.md §3 multiplies stake by karma κ and by a market factor
f(price); tru's `Context` also carries will. At genesis:

- κ = 1 for every neuron, and nothing persists karma. Karma is the
  accumulated BTS score (truth-scoring.md); there are no reports on chain
  at launch (§7.4), so karma has no input, and a field of constants is not
  worth a record. There is a second reason to wait: privacy-frozen.md §7
  makes identity-weighted schemes opt-in disclosures once the pool is live,
  so karma must be designed to attach to positions or payout keys, and that
  design belongs to the reports rule set, not to a launch rush.
  `NeuronRecord.karma` stays 0 and means "neutral".
- f(price) = 1: the ICBS market (launch row 25) is not on chain.
- will = 0: positions already are the conviction primitive; will (a
  neuron-wide budget auto-shared across its links) needs its own lock kind
  and is deferred with the reports rule set.

## 4. the focus graph

### 4.1 genesis weights

The burial graph arrives with no locked stake (chain.md §4.1 applies every
genesis link with `amount = 0`). To give φ* a body on day one, every
genesis link frame carries a genesis weight, computed by every node from the
genesis file alone:

```
B(n)  = native balance of n in the genesis file (its b frame for τ_native, 0 if none)
L(n)  = number of l frames authored by n
g(f)  = max(1, floor(B(author(f)) / L(author(f))))       // per l frame f
G_pq  = Σ g(f) over the l frames f with (p, q)
```

Genesis weights are passive weight on the rank axis (rewards.md §9): they
enter φ* like a `v = 0` position, never change, cannot be unlocked, earn
nothing (their change is always zero, §5.2), do not count in θ (§6.1) and
are not balances, so conservation does not see them. They are a pure
function of the genesis file, which the chain id commits to; nothing new is
stored. The reasons for this rule and its trade-offs are §8.

### 4.2 axon weight and valence

For each axon `(p, q)` at a boundary, with `s(c)` the stake of card c:

```
S⁺_pq = Σ s(c) over cards on (p, q) with v ∈ {0, +1}
S⁻_pq = Σ s(c) over cards on (p, q) with v = −1
A_pq  = max(0, G_pq + S⁺_pq − S⁻_pq)                       // u128, then ≤ u64 range in practice
```

Valence is the direction of a position's push on the edge (rewards.md §9,
two axes):

- `v = +1`: active stake for the edge; adds weight; earns when its addition
  raised syntropy (§5);
- `v = 0`: passive stake; adds weight exactly like +1 (it moves rank) and
  never earns — "a purchase, not an investment" (rewards.md §9);
- `v = −1`: active stake against the edge; subtracts weight down to zero;
  earns when its subtraction raised syntropy.

The net is clipped at zero per axon, which is arithmetic.md's "negative
stake is clipped to zero before matrix construction" read per axon: the
adjacency stays in the nonnegative cone, and stake beyond what zeroes the
edge does nothing and earns nothing (its gradient is zero, §5.2). An axon
with `A_pq = 0` is absent from the focus graph; a particle that is an
endpoint of no present axon has no focus.

### 4.3 canonical input

The input to tru at a boundary is the list of present axons
`(p, q, A_pq)`, sorted ascending by `p ‖ q` bytewise, one entry per axon.
Particles are indexed in ascending order of their 32-byte id. Nothing else
enters: no author, no karma, no price (§3.7). This removes the order
dependence of tru's current `FocusingGraph::build` (nodes indexed by first
appearance, which moves λ_max and the step count) — the chain hands tru one
canonical order, and tru's epoch entry point must index particles by
ascending id.

### 4.4 φ*, syntropy and determinism

```
φ*(A)  = the tru tri-kernel fixed-point iterate of the canonical graph A
J(φ)   = syntropy, tru/specs/focusing.md (D_KL(φ ‖ u) over the graph's particles)
```

- φ* is tru's: the composite operator, its parameters (λ_d, λ_s, λ_h,
  teleport, τ, heat terms), the fixed-point scale and the normalization are
  normative in tru/specs and in tru's epoch entry point at the revision
  pinned in `release/phase1.toml`. A change of any bit of φ* for a given
  input is a consensus change and moves the conformance vectors (§10).
- iteration count: every solve starts from the uniform vector and runs
  exactly `PHI_STEPS` steps — a constant, never derived from the data. tru's
  `T(ε) = ⌈log(1/ε)/log(1/κ)⌉` sets the constant once: the measured κ̂ ≈ 0.74
  on bostrom (tru eval/spectral-gap-bostrom.md) gives T(10⁻⁶) = 46, so 48
  is the recommended value; tru's `EpochParams` fixes it. No step count that
  depends on a power iteration, a convergence test or a warm start enters
  consensus: a cold start makes settlement of epoch E a function of `G_{E−1}`
  and `G_E` alone, so a node can check any epoch from its two snapshots.
- arithmetic: integer and fixed point only (tru/specs/arithmetic.md); the
  result must not depend on thread count; division and rounding rules are
  tru's and are pinned by the vectors. At 3 M particles a mean focus entry
  is 2⁻²¹·⁵, so the fixed point must carry at least 2⁻⁶⁰ resolution (tru's
  epoch engine scales by 2⁶⁰; the legacy 2⁻³² `Fx` keeps three significant
  digits there and is not adequate for gradients).

## 5. settlement

### 5.1 the three solves

Settlement of epoch E reads `G_{E−1}` and `G_E` and computes:

```
A_prev = A(G_{E−1}),  A_cur = A(G_E)                         // §4.2
A_mid  = floor((A_prev + A_cur) / 2)  per axon               // axons present in either
φ*_E   = φ*(A_cur)                                           // solve 1, published (§5.6)
φ_mid  = φ*(A_mid)                                           // solve 2
g      = ∂J(φ*(A))/∂A at A = A_mid, one adjoint solve        // solve 3
```

`g_pq` is the derivative of syntropy at the fixed point with respect to the
weight of axon `(p, q)`, propagated through the fixed-point map by its
adjoint (rewards.md §2: the envelope theorem makes the value of a new link
the reshaping of the landscape, ⟨∇_A J, ΔA⟩, never the induced shift of
φ). The adjoint iterates `λ ← ∇_φ J + (∂F/∂φ)ᵀ λ` from `λ = 0` for
`PHI_STEPS` steps and then reads `g = λᵀ ∂F/∂A`; tru/specs owns the
transposed operators and the scale of `g` (a signed fixed-point integer per
present axon of `A_mid`; zero for absent axons).

The midpoint is the one-point quadrature of the integrated gradient along
the straight path from `A_prev` to `A_cur`, i.e. the Aumann–Shapley value
of rewards.md §4 with one gradient evaluation: exact when J is quadratic
along the path, and with error third order in the epoch's change otherwise.
One solve serves every position, which is decision 4.

### 5.2 attribution per position

For a card c on axon `(p, q)` with side σ(c) = +1 for v ∈ {0, +1} and −1
for v = −1:

```
Δs(c) = s_E(c) − s_{E−1}(c)        // stake on c's side; a card absent at a boundary has s = 0 there
a(c)  = Δs(c) · σ(c) · g_pq · [A_mid_pq > 0]                  // signed, 256-bit integer
```

A card that changed side between the two boundaries (unlocked to zero and
locked again with another valence) is a decrease on its old side and an
increase on its new one. Attribution is linear in stake within a side, so
splitting a stake across many cards or many neurons changes no total
(Sybil-neutral, rewards.md §4), and positions of one side of one axon share
its credit pro rata to what each added. Genesis weights never change, so
their attribution is always zero.

The epoch's total change of syntropy `J(φ*_E) − J(φ*_{E−1})` is recorded
(§5.6) and gates nothing: the budget (§5.5) is fixed in tokens, the
attribution only divides it, and an epoch with no positively attributed
active stake pays nothing.

### 5.3 the new-high rule

A position earns only for stake that takes its axon side above its high-water
mark. Per axon and side, with `S^σ` from §4.2 and `H^σ` the mark left by the
previous settlement (0 for an axon never staked):

```
N^σ = max(0, S^σ_E − H^σ_{E−1})                     // new height reached this epoch
R^σ = Σ Δs(c) over cards on that side with Δs(c) > 0   // raw increase
x(c) = floor(max(0, a(c)) · N^σ / R^σ)              // R^σ > 0 whenever Δs(c) > 0
M^σ = max(H^σ_{E−1}, S^σ_E)
H^σ_E = S^σ_E + floor((M^σ − S^σ_E) · 65,410 / 65,536)
```

Above the mark the side earns in full; a side that fell and climbs back
earns nothing for the climb until the mark has decayed (half-life 360
epochs, 30 days). Without the rule a holder could unlock, relock and be
paid again for restoring the same edge, from a second key if needed; with
it, recycling stake on one edge earns at most the decay. The price is borne
by an honest staker who restores an edge someone else abandoned within the
month; it is named in §9. Marks are kept per side only for axons that ever
held a card; an axon with `S⁺ = S⁻ = 0` and marks below 1 drops them.

### 5.4 gates

A card c earns in epoch E iff all hold:

- active valence: `v ≠ 0` (passive stake moves rank and earns nothing,
  rewards.md §9);
- increase: `Δs(c) > 0`, the position added stake during the epoch;
- conviction: guaranteed by `MIN_LOCK ≥ L + D` (§3.4): stake added in
  epoch E cannot leave before `settle(E)`; a position that added stake is
  paid on its stake at `G_E`;
- surprise: the weight is `ρ(c) · x(c)` with `ρ(c) = RHO_LAUNCH = 1` for
  every active position. There are no reports on chain at genesis: BTS
  (rewards.md §5) needs each contributor's meta-prediction of the crowd, and
  the valence alone is a side, not a prediction. ρ = 1 is safe at launch
  because copying is already priced by the gradient: a copy on the same axon
  in the same epoch splits that axon's credit pro rata, and a copy in a later
  epoch meets a smaller gradient on a saturating edge. The reports rule set
  (§7.4) replaces it.

There is no per-neuron cap. A cap per identity is free to evade by
splitting (one key per position costs nothing) and contradicts privacy-
frozen.md §7, where rewards attach to links; the cap is per position and
proportional to the stake it adds (§5.5), which splitting cannot move.

### 5.5 budget, caps and rounding

At `settle(E)`, with `F` the focus pool balance after this block's coinbase
(§5.7):

```
B      = floor(F / POOL_RELEASE)                             // the epoch's budget
X      = Σ x(c) over earning cards                           // u256
cap(c) = floor(Δs(c) · R_MAX_PPM / 1,000,000)
pay(c) = min(cap(c), floor(B · x(c) / X))                    // u256 intermediate, result u64
P      = Σ pay(c)                                            // P ≤ B ≤ F
F'     = F − P
```

- If `X = 0` nothing is paid and the whole pool carries to the next epoch.
- Rounding is floor for every recipient; the dust `B − Σ floor(B·x/X)` and
  everything a cap withholds stay in the pool. No recipient is favoured by
  position, id or tie, and conservation is exact by construction.
- Releasing a quarter of the pool smooths payouts: a quiet epoch carries
  its accrual forward, and one epoch can never take more than a quarter of
  what accumulated. In a steady state the pool holds four epochs of accrual
  and pays one per epoch.
- The cap bounds what one epoch can pay one position to 1 % of the stake
  it added: a single position that is the only earner of a quiet epoch
  takes its 1 %, not the pool. In the expected regime the cap does not bind
  (pussy at θ = 1 %: about 2.5 × 10¹¹ PUSSY per epoch over an expected 10¹⁴
  of new active stake, 0.25 %).

### 5.6 payout and the epoch record

Each `pay(c) > 0` credits `balances[hemera(owner(c) ‖ τ_native)]`, with
`owner(c)` the card's payout owner in `G_E`, and debits the pool. Payouts
are liquid at once (§2.5).

The settlement block writes one value into the `time` dimension (empty in
rule set 1 until now): `time[settle(E)] = hemera(D_EPOCH ‖ record)`, with

```
record = epoch: u64 ‖ boundary: u64 ‖ beacon: [u8;32] ‖ boundary_root: [u8;32]
       ‖ particles: u32 ‖ axons: u32 ‖ phi: [u8;32] ‖ syntropy: i64 ‖ delta_syntropy: i64
       ‖ focus_bps: u16 ‖ budget: u64 ‖ paid: u64 ‖ pool_after: u64 ‖ rewarded: u32 ‖ payouts: [u8;32]
                                                                                        // 198 bytes
phi     = hemera(D_PHI ‖ n: u32 ‖ φ*_E[0]: u64 ‖ … ‖ φ*_E[n−1]: u64)   // particles ascending, tru's raw fixed point
payouts = Merkle root over leaves hemera(D_PAYOUT_LEAF ‖ card ‖ owner ‖ amount: u64),
          cards ascending, amount > 0 only, nodes hemera(D_PAYOUT_NODE ‖ L ‖ R),
          odd node promoted unchanged (chain.md §1.7); 32 zero bytes when none
```

`syntropy` and `delta_syntropy` are J(φ*_E) and J(φ*_E) − J(φ*_{E−1}) in
tru's raw fixed point, signed; `focus_bps` is the split in force for the
blocks of epoch E + 1 (§6.1). The record, the payout list and φ*_E are kept
by every node and served over HTTP (§11). Through `state_root` every header
from `settle(E)` on commits to φ*_E: two nodes that disagree on one focus
bit disagree on the root.

### 5.7 order inside a block

For the block at height h, after the checks of chain.md §5.5 steps 1–6:

1. coinbase: credit the miner `SUBSIDY − cut(h) + fees`; credit the focus
   pool `cut(h)` (§6.2); `coins[τ_native].total_supply += SUBSIDY`;
2. if `h = settle(E)` for some E ≥ 1: the settlement of epoch E — payouts,
   pool debit, high-water marks, `time[h]`;
3. the transactions in body order (chain.md §3.4);
4. conservation (§6.3); `state_root`.

A transaction in the settlement block sees the payouts (a payout owner may
spend a reward in the block that pays it).

## 6. emission

### 6.1 the split

The subsidy (chain.md §3.5) stays `SUBSIDY = floor(S0 / 105,120,000)` per
block, 1 % of genesis supply a year, constant. It is base emission in the
sense of rewards.md §10 — paid only "to the two providers that do work:
PoW compute and active (v ≠ 0) epistemic risk" — split between them by the
allocation curve `θ^α` of rewards.md §10 (tru `allocation::split`) with the
neutral α = 1/2, evaluated in integers:

```
active(E)    = Σ s(c) over cards with v ≠ 0 in G_E
supply(E)    = S0 + boundary(E) · SUBSIDY
focus_bps(E) = min(FOCUS_MAX_BPS, isqrt(floor(active(E) · 10^8 / supply(E))))   // u128 inside
cut(h)       = floor(SUBSIDY · focus_bps(epoch(h) − 1) / 10,000)                  // h ≥ 1
```

`isqrt` is the floor integer square root; `10^8` makes `isqrt` return
`θ^½` in basis points. Blocks of epoch 1 use `focus_bps(0) = 0` (no stake
at genesis). Examples: θ = 1 % → 999 bps; θ = 25 % → 5,000 bps, the cap.

Why the curve and why the cap:

- the curve is the one rewards.md derives for splitting a security budget
  between work and risk, already implemented and simulated (launch row 27);
  it pays knowledge in proportion to how much capital actually took risk on
  the graph, so an unused economy does not drain the miners;
- per unit of active stake the yield `θ^½ · SUBSIDY / (θ · supply)` falls as
  θ grows: the first stakers are paid most, which is the bootstrap the
  economy needs;
- the cap is the miner security floor: at most half the subsidy leaves
  proof of work, so the cost of a rewrite never falls below half of what a
  pure PoW chain of this subsidy buys. rewards.md derives the floor from
  attack economics (`floor ≥ c_sec · TVL/M · r_atk`); none of those inputs
  exists before launch, so the floor is a constant at genesis and its PID
  loop (launch row 27) is a later rule set.

### 6.2 coinbase and pool

The coinbase amount rule of chain.md §2.2 becomes

```
coinbase.amount == SUBSIDY − cut(h) + Σ fees of the block's other txs
```

and the focus pool receives `cut(h)` as part of the coinbase step, no
transaction. `cut(h)` depends only on `G_{epoch(h)−1}`, so once a boundary
is final the pool's inflow over the next epoch is a function of height,
and the pool balance at `settle(E)` is the same on every branch above a
final boundary (§2.5).

### 6.3 conservation

The invariant of chain.md §3.4 keeps its form:

```
Σ balances[·, τ_native] == S0 + h · SUBSIDY
```

where the sum runs over every key holding native coin: owner balances,
dormant legacy owners, stakes `stake_key(c)`, the focus pool
`focus_key(τ_native)`, and (rule set 2) the turnstile `T(τ_native)`. The
subsidy is minted once per block and split between the miner and the pool;
stake moves between an owner and its stake key; settlement moves from the
pool to payout owners; fees move from senders to the miner. Nothing burns,
supply is a function of height alone. Added per-settlement checks, exposed
in `/status`: `P ≤ B ≤ F`, the pool never negative, and Σ stake keys equal
to the stake total of the cards.

### 6.4 fees

Fees are unchanged: every fee goes to the miner of the block that includes
the transaction, nothing is burned, no fee is routed to the pool. The fee
role of rewards.md §8 (burn β, servicer γ, security budget) needs a fee
market and paid inference, both cut from phase 1.

## 7. mining, honestly

### 7.1 at launch

"Mining" on the root chain at genesis means two things and only two:

- proof-of-work block production: hemera header grinding under ASERT
  (chain.md §1), paid `SUBSIDY − cut + fees`. The work is synthetic, the
  bootstrap puzzle of rewards.md §8 that hands out the initial stake;
- settlement by recomputation: every full node computes every epoch's
  three solves and checks the settlement block bit for bit. Nobody is paid
  for this computation; it is the cost of validating, as checking
  signatures is. The miner of the settlement block chooses nothing in it.

Settlement mining — the lottery in which each hash attempt is a Shapley
sample and the winning ticket earns the subsidy (rewards.md §7) — is not
active at genesis. What is active is its product: the fair division of
Δφ⁺ among staked links, computed exactly (to the integrated-gradient
approximation) instead of estimated by sampling.

### 7.2 cost bound

Measured 2026-10-17 on an M4 Max, one core, tru master's
`FocusingGraph::build` + `iterate` (the legacy 2⁻³² engine) over the burial
graphs with the genesis weights of §4.1 (receipt
`audit/chain/economy-cost-2026-10-17.md`):

| graph | links | particles | build | per step | steps derived | one cold solve | peak RSS |
|---|---|---|---|---|---|---|---|
| pussy | 29,112 | 15,967 | 0.14 s | 16.5 ms | 24 | 0.54 s | — |
| bostrom | 2,949,732 | 2,914,272 | 24.3 s | 1.20 s | 19 | 48 s | 2.8 GB |

At `PHI_STEPS = 48` one bostrom solve is about 24 + 58 = 82 s; a
settlement is two forward solves and one adjoint (estimated at the cost of
a forward solve), about 4–5 minutes of one core, against the one-hour
window of D blocks: a 12× margin, and `SETTLE_BUDGET` = 900 s (a quarter of
the window) holds up to about three times bostrom's graph at linear
scaling. tru's epoch engine (in progress) adds threads with bit-identical
output, which moves the bound further; its figures replace these when it
lands.

Worst case by construction: at most `MAX_BLOCK_TXS · L` = 245,760 positions
change per epoch, so the graph grows by at most that many axons per epoch
(≈ 3 M per day if every block were full of fee-paying staked links — a
spam bill of 257 units per link). Settlement cost is linear in present
axons and independent of how many changed. The threshold that matters is
`SETTLE_BUDGET`, not one block interval: settlement is pipelined over D
blocks, and a single solve already exceeds one 30-second interval at
bostrom's size today.

### 7.3 the ticket rule set (the scaling rule)

When the reference settlement exceeds `SETTLE_BUDGET`, or is projected to
within 90 days at the observed growth, settlement moves to the lottery of
rewards.md §7 by scheduled fork (chain.md §7.1, two weeks' notice). That
rule set is specified here as far as the root chain fixes it:

- the beacon is `beacon(E)` of §2.3; the cluster set is the canonical
  partition of the epoch's changed positions into connected components of
  overlapping ε-supports (launch row 35);
- a ticket (kind 8) carries `(E, cluster, n, miner, m(n), proof)`; the
  ordering is `π(n) = hemera(D_ORDER ‖ beacon(E) ‖ cluster ‖ n)`, the
  win test `hemera(D_TICKET ‖ beacon(E) ‖ cluster ‖ n ‖ miner ‖ commit(m(n)))
  < target(cluster)` with the target banded by cluster size (row 4); the
  tags `D_ORDER` and `D_TICKET` are fixed by that rule set;
- tickets are accepted in blocks `boundary(E) + 1 … settle(E) − 1`; the
  settlement at `settle(E)` is the mean of accepted samples per cluster
  under the same budget, gates, new-high rule, caps and rounding of §5;
  clusters with fewer than `k_min` samples defer to the next epoch;
- the PoW share of the subsidy is re-targeted from the header hash to the
  ticket (rewards.md §8: "the subsidy and the settlement lottery are one
  act"); the header keeps a hash target for ordering;
- every ticket carries a sound per-ticket proof (foculus#61) and tickets
  fold per cluster into one accumulator (launch rows 2, 5); full nodes stop
  recomputing.

Preconditions, each a launch-registry row: ticket proof cost below ticket
reward (3), progress-freedom across clusters (4), canonical clusters (35),
the hardware profile (36), an O(1) fold (5). None is required at genesis.

### 7.4 the reports rule set

BTS needs, for every active position, a meta-prediction: the share of
active stake the author expects on its side of the axon. A later rule set
adds kind 10 `report: from ‖ nonce ‖ p ‖ q ‖ prediction: u16 (bps) ‖ fee`,
computes ρ(c) per epoch by tru's hardened scorer (crowd reference over
staked, distinct predictors, `bts_scores_hardened`), replaces
`RHO_LAUNCH`, starts persisting karma per position or payout key (§3.7),
and adds the zero-sum redistribution that is the slashing of rewards.md §5.
The serum's properties (launch row 24) and the market (row 25) are its
gates.

## 8. genesis stake

The graph comes home with 2,949,732 bostrom links (2,997,569 genesis frames
with passports) and 29,112 pussy links, and with no stake: the old chain's
weights were its resource tokens, and chain.md §10 item 3 made every
delegated balance liquid at genesis. φ* over a weightless graph is empty, so
the burial needs weight from somewhere. Options:

| option | rule | for | against |
|---|---|---|---|
| A · author balance, spread over the author's links (chosen) | `g = max(1, floor(B(n) / L(n)))` per frame, constant, passive (§4.1) | uses only the genesis file; same unit as live stake (native coin), so live and genesis weight compete one to one; spreading over L(n) keeps link spam from multiplying weight (the busiest bostrom author made 2,289,526 of the links); nothing is locked, decision 3 of chain.md §10 stands | weight follows capital at halt and stays after the coin is sold; concentrated (bostrom: authors held 22.8 % of native holdings at halt, the top author 39 % of genesis weight, the top ten 77 %; pussy: 3.5 %, 15 %, 79 %); 278,118 bostrom frames sit at the floor of 1 (authors with zero or too little balance) and carry topology but almost no weight |
| B · resource tokens (milliampere, the old link weight) | weight by each author's milliampere at halt, priced into native units | the old chain's own rank weight, closest to old cyberank; the 2026-09-18 launch decision named the resource tokens | needs a price oracle from `prices.json` into consensus, or an arbitrary exchange rate; 142 bostrom authors hold no milliampere; milliampere is a token of its own on the new chain and would double-count |
| C · actual locks from genesis balances | lock a share of each author's balance as stake on its links | weights are real, unlockable money | locks people's coins without their signature; contradicts chain.md §10 item 3 (liquid, kept); 2.3 M bot links would hold most locked capital |
| D · a genesis focus pool | no weight; a pool pays the first stakers on burial links | live stake only | φ* is empty until someone stakes; cyb shows nothing on day one |

Recommendation: A. It is the only option that is a pure function of the
genesis file (no oracle, no new frame, no new state), keeps balances liquid,
and puts the burial graph in φ* on day one in the same unit live stake uses.
Its fairness cost — rank follows capital at halt — is the rank axis working
as rewards.md §9 defines it, and it is bounded: genesis weight earns nothing
(only change earns, and genesis weight never changes), so A hands out rank,
never income; a holder who sold at genesis keeps rank but no reward stream;
and live stake displaces it unit for unit, including `v = −1` stake that
zeroes a genesis edge (§4.2). Attack surface: none at launch — nobody can
add genesis weight after block 0, and the weights are public in the genesis
report. The 2026-09-18 decision that "the migrated graph would pay nothing"
without the resource tokens is met in this form: the migrated graph ranks
from block 0 and pays its authors as soon as they stake on it (their links
exist; a lock on an existing link is a new high on an empty side).

## 9. attacks

| attack | mechanism | bound at genesis |
|---|---|---|
| link spam | zero-stake links | topology only: no weight, no focus, no reward; fee floor 257 units per link; ≤ 1,024 per block |
| staked dust spam | many tiny positions to bloat state and settlement | a card per position costs the fee of a link; settlement cost is linear in axons, ≤ 245,760 new per epoch; dust earns at most 1 % of itself |
| Sybil splitting | split stake across keys or cards | attribution, caps and the split curve are linear in stake; no per-identity rule exists to evade |
| self-linking cycles | stake a ring p→q→p to concentrate focus (syntropy rises) | the pulse is paid once per new high (§5.3) and capped at 1 % of the added stake; teleport bounds the mass a ring holds; the ring's stake is locked seven days; residual: concentration is paid as knowledge — rewards.md §15's self-dealing frontier, bounded here by capital, not closed |
| stake recycling | unlock and relock to be paid again for the same edge | the new-high rule pays only above the decayed mark; a second key does not help (the mark is per axon side); yield of recycling ≤ the mark's decay, 50 % per 30 days of a stake that must sit locked 7 days per cycle |
| honest restaker penalty (cost of the above) | restoring an edge abandoned within the month earns little | accepted; the mark decays with half-life 30 days |
| last-block timing | lock in the last block of an epoch after seeing the epoch's links | all links of an epoch are one coalition at one midpoint gradient: a late copy on the same axon splits that axon's credit pro rata and lowers the gradient for both; a boundary miner can delay rivals' locks by one epoch (MEV, bounded by one epoch's ordering) |
| hedging | +1 and −1 on the same axon (two keys) | attributions cancel exactly; capital doubled for the same credit |
| capital censorship | `v = −1` stake to zero an edge | costs locked capital equal to the edge's weight, outbid by `v ≥ 0` stake; earns only if zeroing raised syntropy |
| quiet-epoch capture | the only earner of an empty epoch | takes at most a quarter of the pool, and at most 1 % of its added stake |
| reorg around the boundary | replace the boundary block to change the snapshot | possible below depth 120 at the price of the blocks; settlement waits until depth 120 (§2.5); every candidate boundary costs a block and forces at most one recompute |
| boundary withholding (beacon grind) | discard a found boundary block | nothing reads the beacon at genesis; for the ticket rule set the bias costs a block reward per draw (§2.3) |
| settlement DoS | make the recompute too slow to finish in D blocks | linear cost, 12× measured margin at bostrom's size, growth bounded by fees per axon; the ticket rule set is the scaling exit (§7.3) |
| nondeterminism | thread count, node order, rounding differ across nodes | canonical order (§4.3), fixed steps (§4.4), tru's integer arithmetic, φ* committed by digest under `state_root` (§5.6): any divergence halts the divergent node at `settle(E)` instead of forking value silently |
| payout censorship | a miner omits a payout | payouts are a state transition of the height, not transactions |
| rich-get-richer | idle capital compounding | only stake that adds new weight earns, once, and only active stake; passive stake and genesis weight earn nothing (rewards.md §9) |

## 10. conformance vectors to add

To chain-conformance.md, new section 14 ("the focus economy"). Values below
were computed on 2026-10-17 with `cyber-hemera` 0.4 at `7237a23` (the
phase1.toml pin) through `hemera::hash`; integer vectors with Python
integers. Others are `blessed`.

Identities:

```
card(0x66 × 32, token("boot"), token("pussy")) = 74f0a211d177f290e9c95a6e7edb65cd8705ea96d9713a841d5a086f6476b5b4
stake_key(that card)                             = f7e0cbf8e5cfc501b7e3098b238bce8451b3b0a7fa2e42b69724801115c6b08a
focus_key(token("pussy"))                        = 0fae8671376d879515b2736c2d6d51eafdd9ff40055aa84aaff6e17a0beed4b9
payout_leaf(that card, 0x77 × 32, 1,000)         = 63c3867a76af023aa6417e5a945612289906e8352ea69fe4aea259ce5b152115
payout_node(leaf, leaf)                          = 240a852c1bd8f641f65d6b3746a72a08c0930e5d1bd1feafe4b9cdab914a6ec2
phi digest, n = 2, φ = [2^59, 2^59]              = 3351bfa40d6eaa24c8e2b8755d900dbcd3c3015730a5efcfdf9b6e14c7019a8a
```

Integers (pussy `S0`, `SUBSIDY`):

| case | value |
|---|---|
| `focus_bps` at active = 0, S0/10⁴, S0/100, S0/4, S0 | 0, 99, 999, 5,000, 5,000 |
| `cut` at 1,000 bps; miner share | 1,045,842,665; 9,412,583,992 + fees |
| `cut` at 5,000 bps; miner share | 5,229,213,328; 5,229,213,329 + fees |
| mark decay, H = 1,000,000, S = 0: after 1 epoch, after 360 | 998,077; 500,047 |
| budget 1,000, x = [1, 1, 1] | pay [333, 333, 333], 1 stays in the pool |
| budget 10⁶, x = [3, 1], Δs = [10⁶, 10⁹] | caps [10,000, 10⁷]; pay [10,000, 250,000]; 740,000 stays |
| epoch(240), epoch(241), settle(1) | 1, 2, 360 |

Transactions (`blessed`, signed with the known-answer key of §6): kind 2
with amount 7 and native token creates the card and moves 7 to its stake
key (the old `TxError::Box(7)` vector is retired); kind 2 with amount 7 and
a non-native token rejected; kind 2 with the opposite valence on a live card
rejected; kind 6 at `since + MIN_LOCK − 1` rejected, at `since + MIN_LOCK`
accepted, partial then full (card deleted); kind 6 by a non-author rejected;
kind 7 re-points the owner; encodings of kinds 6 and 7 at 224 and 248
bytes, trailing and missing bytes rejected, kinds 8–10 rejected.

Epochs (`blessed`, private test genesis, trivial target, `L` and `D`
overridable only under `cfg(test)` like the PoW limit):

- the genesis weights of the conformance genesis and of pussy's real
  genesis file: Σ G, the count at the floor, the digest of `(p, q, G_pq)`
  sorted;
- φ*_0 of pussy's genesis: `phi` digest and `syntropy`, single-threaded and
  with 8 threads (equal);
- a 2-epoch chain: one +1 lock, one 0 lock, one −1 lock on three axons in
  epoch 1; the epoch record of epoch 1 at `settle(1)`, the payouts (the
  passive position earns 0), the pool after, and the state root;
- new-high: lock 100, unlock after `MIN_LOCK`, relock 100 one epoch later:
  the relock's `x` is the decayed remainder only;
- reorg: a 3-block reorg ending at `settle(1)` reproduces the identical
  epoch record and root; a reorg replacing `boundary(1)` changes the
  record;
- conservation at every height of a 2,000-block simulated chain with random
  locks, unlocks and payout changes, including the settlement blocks.

Determinism gate: two nodes on two machines (deimos, jupiter) agree on every
epoch record of pussy-rc for 24 h; cyberproxy (8 GB) completes every bostrom
settlement within `SETTLE_BUDGET`.

## 11. node surface

HTTP (chain-wire §7), new routes:

| method, path | response |
|---|---|
| `GET /epoch/{E}` | the record of §5.6 as JSON, its digest, and the payout list `[{card, owner, amount}]` |
| `GET /epochs?after=E&limit=N` | ≤ 500 records |
| `GET /focus/{particle}` | φ*_E of the particle at the last settled epoch (raw fixed point and the scale), and its rank |
| `GET /focus?top=N` | the N particles of highest focus |
| `GET /card/{id}` | the card record and stake |
| `GET /cards/{neuron}` | the author's cards (node-side index) |

`/status` gains `epoch`, `next_boundary`, `last_settled`, `focus_bps`,
`pool`, `active_stake`, `total_stake`, and the per-settlement checks of
§6.3. The node keeps φ* of the last two boundaries and the epoch records in
`$home/<chain>/epochs/`.

## 12. open questions for the owner

Only choices that are policy rather than engineering; each has the default
this page implements.

| # | question | default |
|---|---|---|
| 1 | the emission split: `θ^½` capped at half the subsidy, or a fixed share from block 0 | `θ^½`, cap 5,000 bps (miner floor 50 %) |
| 2 | genesis rank weight: author balance spread over its links and constant forever, or decaying (say halving yearly) so the burial's capital fades as live stake grows | constant |
| 3 | valence −1 subtracts from the edge (capital can zero an edge and earn when that helps), or carries no weight (then −1 never earns at launch) | subtracts, clipped at zero per axon |
| 4 | the lock: seven days | 20,160 blocks |
| 5 | standing yield: stake earns only when it adds new weight (the pulse), or also an annuity on the growth of its target's focus (rewards.md §11) | pulse only at genesis; the annuity with the reports rule set |
| 6 | the 2026-09-18 decision "the migrated graph would pay nothing": met as "ranks from block 0, pays when its authors stake", rather than paying genesis links directly | as specified |

## 13. implementation map

| owner | change |
|---|---|
| soft3 `crate/src/chain` | `params.rs` (§0); `tx.rs` kinds 6, 7 and kind 2 with stake; `apply.rs` coinbase split, settlement step, conservation; `state.rs` card, stake-key and pool operations journaled for reorgs; new `economy/` (`epoch.rs` boundaries and snapshots, `weights.rs` genesis weights and §4.2, `settle.rs` §5 over tru, `emission.rs` §6); `api.rs` §11; `wallet/tx.rs` builders for kinds 6, 7 |
| bbg | `CardRecord` + `author`, `valence`, `since`; `DIMENSION_VERSION` 4; genesis roots re-blessed |
| tru | an epoch entry point: canonical input (§4.3), fixed `PHI_STEPS`, cold start, 2⁻⁶⁰ resolution, J, and the adjoint gradient at `A_mid` (§5.1), thread-count independent; tru/specs owns the operator transposes and the scale |
| foculus | `phi.rs` and the VDF beacon retired from the root chain; specs/epoch.md superseded for the root chain by §2 here |
| conformance | §10 into chain-conformance §14 |
| cyber/launch | rows 1–4, 24–32, 41 and the decision of 2026-10-17 |

See [chain.md](chain.md) for the chain, [chain-wire](chain-wire.md) for the
network, [chain-conformance](chain-conformance.md) for the vectors,
tru/specs/rewards.md for the reward law this page instantiates, and
[cyber/launch](../../cyber/launch.md) for the tracker.
