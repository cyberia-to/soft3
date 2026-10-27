---
title: the metabolic oracle
tags: cyber, soft3, tru, proposal, economy
crystal-type: proposal
crystal-domain: cyber
alias: metabolic oracle, metabolism, M(t), vital signs
status: proposed
date: 2026-10-23
---
# the metabolic oracle

**status:** proposed · **date:** 2026-10-23 · **owner's call:** "this mechanism — without the agents — belongs in the core of cyber, not in cyberia's architecture."

cyberia's [[cyberia/architecture|architecture]] carried a mechanism that answers one question for a whole network: *is it alive?* It was written for cyberia's 147 agents. The mechanism does not depend on agents; it depends on three numbers the chain already has or can have. This proposal moves it into the core: the measure becomes a consensus object of the root chain, its weights the one vote the protocol cannot avoid, and its derivative the governor of every parameter the protocol is allowed to retune. The agents, where they exist, become its users.

## 1. the picture

Three vital signs, one compound, one derivative.

| signal | measures | source in the core | exists |
|---|---|---|---|
| $\text{cap}(t)$ | external validation | circulating supply × price; price = the stake-weighted median of privately committed reports, aggregate public, individual private (§3) | not before a market |
| $J(t)$ | internal order | [[syntropy]] $D_{\mathrm{KL}}(\phi^* \,\|\, u)$, emitted by [[focusing]] every epoch — already in consensus with the [[soft3/specs/chain-economy|focus economy]] | yes, genesis |
| $H(t)$ | subjective wellbeing | [[happiness]]: stake-weighted aggregate of private $h_\nu \in [0,100]$, committed in [[bbg]], aggregated under a [[zheng]] proof | with the privacy rule set |

$$M(t) = \text{cap}(t)^{w_c}\cdot J(t)^{w_s}\cdot H(t)^{w_h}, \qquad w_c + w_s + w_h = 1,\quad w \ge 0$$

The form is a weighted geometric mean on purpose: a collapse in any one sign drags the compound toward zero, so gaming one signal at the expense of another lowers $M$. A network with zero happiness is metabolically dead however rich or ordered it is; a network nobody values from outside is dead however happy.

$\dot M$ — the change of $M$ per epoch — is the reward signal of the protocol as a whole. **It pays no one directly.** Individual rewards stay what [[tru/specs/rewards|rewards]] §11 says: Shapley of surprising syntropy, the subsidy for proven work, fees for service, the stake yield. $\dot M$ acts one level up: it is the objective every *parameter change* is measured against, and the sensor the emission floor's control loop reads.

The weights $(w_c, w_s, w_h)$ are the only normative choice the system cannot make on its own. They define what "health" means. They are set by a vote — the one vote — and nothing else in the economy is.

## 2. why the core, and why not the agents

- The three signs are chain state or can be: $J$ is computed by every node at every epoch boundary already; $H$ and the price are commitments of the same kind the privacy rule set introduces for everything else — aggregate public, individual private. Nothing here needs an agent, a model or an oracle committee.
- A measure of the network's health that lives outside consensus is an opinion; one that lives inside is a constraint. The [[happiness]] page already states the constraint — parametrization may retune emission, difficulty and blend weights only within bounds that do not drive $H$ down — without a place where it is enforced. This is the place.
- cyberia's agents, and any neuron's own agency, read $M$ and $\dot M$ the same way a validator reads the chain: as the weather. What they optimize is their business; what the protocol retunes is governed here.

## 3. the moves

### 3.1 the signs as epoch records

The epoch record of [[soft3/specs/chain-economy|chain-economy]] §2 gains a `metabolic` block, computed at the boundary by every node like the focus graph:

```
metabolic(E) = { J: u64 (bits, fixed-point), H: u64 | none, cap: u128 | none, M: u128, dM: i128, w: (w_c, w_s, w_h) in bps }
```

- `J` is `syntropy(G_E)` of the focusing pass — integer fixed point in the Goldilocks regime, never a float (tru/specs/arithmetic.md).
- `H` is the stake-weighted aggregate of the happiness commitments present at the boundary, proven by the aggregation proof of the privacy rule set; `none` while that rule set is off, and then $w_h$ is forced to 0.
- `cap` is `supply(E) × price(E)`, `price(E)` the stake-weighted median of the price commitments of the epoch; `none` while no market exists, and then $w_c$ is forced to 0.
- `M` is evaluated in integers: logs in fixed point, weights in basis points, one rounding rule in the conformance vectors. `dM = M(E) − M(E−1)`.

A sign that is `none` contributes nothing and its weight is renormalised over the signs that exist. **At genesis $w = (0, 1, 0)$ and $M = J$**: the chain measures its health as syntropy alone, which is what the focus economy already maximises — nothing new runs at launch, and the signs plug in as their rule sets arrive.

### 3.2 reports: the same regime as everything private

`h_ν` and `price_ν` are reports, not transactions of value. They live as commitment cards in bbg under the privacy rule set ([[soft3/proposals/privacy-frozen|privacy, frozen]]): the individual value is never revealed, the aggregate is certified by one proof per epoch, and the report's weight is the reporter's active stake — a fresh sybil carries nothing. A report is edited whenever the reporter wants; the boundary reads the standing value.

Honesty of the price report rests on the same two legs the happiness page gives: it is private (no performed price), and it is stake-weighted (a lie costs the liar's own weight in a number that governs the liar's own economy). It is not "ungameable": a stake majority can lie about the price as it can about anything. cyberia's line "cannot be gamed internally" was true of an external market read by agents; inside consensus the honest statement is *costly to game, by the same stake that secures the chain*. The proposal says so rather than inheriting the claim.

### 3.3 the one vote

The weights are a chain parameter changed by a vote transaction, and by nothing else:

- a proposal names $(w_c, w_s, w_h)$ in basis points summing to 10,000;
- the vote is weighted by active stake (positions with $v \neq 0$, chain-economy §3), open for `W_VOTE` epochs, passes at a two-thirds supermajority of participating stake and a quorum of one third of active stake;
- a passed vote takes effect at the next epoch boundary and cannot be changed again for `W_HOLD` epochs;
- the weights of absent signs are forced to zero whatever the vote says, so a vote cannot point the compound at a number the chain does not have.

Everything else in the economy — α, the floor, `FOCUS_MAX_BPS`, the lock, τ, the valence bounds — is a parameter, not a value, and is retuned by §3.4, never voted.

### 3.4 what $\dot M$ governs

1. **Parametrization.** A change of any economic parameter is admitted only as an experiment with a predicted $\Delta M \ge 0$ and a horizon of `K` epochs; after `K` epochs the measured $\dot M$ decides: non-negative keeps the change, negative reverts it to the previous value at the next boundary. The futarchy-style comparison the happiness page describes, with $M$ in place of $H$ alone.
2. **The emission floor.** rewards.md §10 says the floor, α and β "follow PID control on observable signals" without naming the signals. The signals are the three signs: the loop's error term is $-\dot M$ clipped to the metabolic safety bounds, so the floor rises only while the compound falls and decays toward zero while it rises.
3. **Safety bounds.** No parameter move may be admitted whose predicted effect drives any single sign below its floor (`H_MIN`, `J_MIN`, `cap_MIN`, all chain parameters, all zero at genesis): the geometric mean punishes it anyway, but a bound refuses it before it runs.

What $\dot M$ does **not** do: it never scales an individual's reward, never mints, never burns, never moves stake. A rising $M$ is not a payout; it is permission for the protocol to keep its current shape.

## 4. acceptance

- conformance vectors in `soft3/specs/chain-conformance.md`: `M` over ten synthetic epochs with every sign present, with `H = none`, with `cap = none`; the genesis degenerate case `M = J`; a weights vote that passes, one that fails quorum, one that names a weight on an absent sign;
- the monotonicity property as a test: raising one sign while lowering another by the same factor never raises $M$ beyond the weight ratio — gaming one sign costs on the others;
- a parametrization experiment that reverts on negative $\dot M$ after `K` epochs, bit-exact on two nodes;
- the happiness and price aggregates verify under the privacy rule set's proof, and their absence forces their weights to zero.

## 5. order

1. this page; the homes named below take the substance: rewards.md gains §16 "the metabolic oracle" (the measure and what $\dot M$ governs), chain-economy.md gains the `metabolic` epoch record and the weights vote, happiness.md and syntropy.md point here instead of to the agents;
2. tru: `metabolic::m(J, H, cap, w)` in fixed point, with `syntropy` already there; conformance vectors;
3. chain-economy: the record and the vote; genesis $w = (0,1,0)$ — nothing else at launch;
4. privacy rule set: the two report cards and the aggregation proof — $H$ and the price enter when the pool does;
5. parametrization and the floor's loop — rule set 3, after the first experiments can be predicted at all;
6. cyberia/architecture: the section keeps its table and points at the core; the 147 agents read $M$, they no longer own it.

## 6. what is cut from the cyberia version

- the agents as the optimiser: the protocol's parameters are governed by $\dot M$, not by anyone "optimising collectively";
- "the agents set the weights through explicit consensus": the weights are a stake vote of the chain's own participants, the agents vote like any neuron with active stake;
- "cannot be gamed internally": replaced by what is true inside consensus (§3.2).

related: [[tru/specs/rewards|rewards]] · [[soft3/specs/chain-economy|chain-economy]] · [[happiness]] · [[syntropy]] · [[soft3/proposals/privacy-frozen|privacy, frozen]] · [[cyberia/architecture|cyberia's architecture]] (the origin)
