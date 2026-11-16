---
title: rewards at genesis
tags: cyber, soft3, tru, economy, proposal
crystal-type: proposal
crystal-domain: cyber
alias: rewards at genesis, annuity at genesis, the split uncapped
status: proposed
date: 2026-11-16
---
# rewards at genesis

status: proposed · date: 2026-11-16 · the owner, 2026-11-16: "either the mechanism exists or it does not; the key mechanism must be complete at once — rewards are the key mechanism"; and on the focus pool: "the mechanism must be dynamic in how much is staked".

Two rows of [[soft3/specs/chain-economy|chain-economy]] §12 currently default to the smaller mechanism. This page sets both to the complete one and names what changes.

## 1. the annuity ships at genesis

chain-economy §12 row 5 defaults to "pulse only at genesis; the annuity with the reports rule set". The default becomes the annuity at genesis: an active position earns the pulse when its stake lifts an axon to a new high (§5.3) and earns the annuity every epoch after, the time-integral of its target's focus growth attributable to the link ([[tru/specs/rewards|rewards]] §11, §12). Foundational work, the code and specs of the stack among it, is paid as it matters, which is what the epistemic asset of cyber/whitepaper.md §14.1 promises.

Moves: tru's epoch engine computes the per-position annuity from consecutive φ* (the attribution it already does for the pulse, integrated over epochs instead of settled once); chain-economy §5 gains the annuity term in `pay(c)`; the conformance vectors gain an epoch sequence where a link earns nothing at settlement and grows over ten epochs.

## 2. the split is the allocation curve, whole

The mechanism is already formalized: [[adaptive hybrid economics]] (cyber/specs/adaptive hybrid economics.md), normative equations in [[tru/specs/rewards|rewards]] §10, implemented and simulated for registry row 27 (tru#10, merged). With `S` the active staked fraction and `B` the gross budget of the window:

```
R_PoS = B · S^α        R_PoW = B · (1 − S^α)        α = ½ at genesis, PID-tuned later
```

Nothing staked, everything to work; as stake enters, the curve moves the budget to active risk; at full stake, everything to it. One parameter, no kink, self-calibrating.

chain-economy §6.1 implements this curve and then clamps it: `focus_bps(E) = min(FOCUS_MAX_BPS, isqrt(…))` with `FOCUS_MAX_BPS = 5,000`, so above `S = 25 %` stake buys nothing. The clamp is the deviation; the fix removes it:

```
focus_bps(E) = isqrt(floor(active(E) · 10^8 / supply(E)))     // S^½ in basis points, 0 … 10,000, no cap
cut(h)       = floor(SUBSIDY · focus_bps(epoch(h) − 1) / 10,000)
```

`B` stays the subsidy at genesis, as §6.1 has it: the floor of the hybrid page (`floor ≥ c_sec · TVL/M · r_atk`) has no inputs before launch, so the floor is the constant `SUBSIDY` and its PID loop is the later rule set. What the clamp was protecting, a miner floor against rewrites, is the floor's own job in the hybrid page, priced by attack economics; a hard half is a second, unformalized floor on top of it and is deleted. The fee leg of `B` (`(1−γ)(1−β)F`) enters when fees exist (chain-economy §6.4, phase 1 has none).

The curve, `α`, the floor and the fee split are parameters of a network's configuration ([[network-repos|a network is a repository]] §1: consensus values in `params.rs` today, one set for every network). They move into the network manifest with these defaults, so every network raised from the stack, the shared ones and every home, carries the hybrid mechanism without choosing it.

## 3. what changes where

| file | change |
|---|---|
| soft3/specs/chain-economy.md | §5.5 annuity term; §6.1 clamp removed, `FOCUS_MAX_BPS` deleted from §0; §12 rows 1 and 5 closed |
| tru/specs/rewards.md | unchanged: §10 is the curve, §11 the annuity |
| tru (epoch engine) | annuity integration over epochs; the split already in tru (row 27) |
| soft3 (kernel economy) | `cut(h)` without the clamp; the network manifest carries α, the floor, the fee split (network-repos) |
| conformance vectors | annuity sequence; the split at S = 0, 1 %, 25 %, 100 % |

## 4. acceptance

- a position staked at epoch 1 on a particle whose focus grows for ten epochs earns in each of the ten, bit-exact on two nodes;
- at S = 0 the pool receives nothing and the miners the whole subsidy; at S = 1 the pool receives the whole subsidy; at S = 25 % the pool receives half, and at 49 % seven tenths, with no clamp;
- supply stays `S0 + h · SUBSIDY` at every boundary, the split moves nothing in or out;
- the self-linking ring of §9 stays capped by `cap(c)` per epoch.

related: [[adaptive hybrid economics]] · [[soft3/specs/chain-economy|chain-economy]] · [[tru/specs/rewards|rewards]] · [[network-repos|a network is a repository]] · [[metabolic-oracle|the metabolic oracle]] · cyber/whitepaper.md §14.1
