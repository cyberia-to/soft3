---
title: rewards at genesis
tags: cyber, soft3, tru, economy, proposal
crystal-type: proposal
crystal-domain: cyber
alias: rewards at genesis, annuity at genesis, mint budget
status: proposed
date: 2026-11-16
---
# rewards at genesis

status: proposed · date: 2026-11-16 · the owner, 2026-11-16: "either the mechanism exists or it does not; the key mechanism must be complete at once — rewards are the key mechanism"; and on the focus pool: "the mechanism must be dynamic in how much is staked".

Two rows of [[soft3/specs/chain-economy|chain-economy]] §12 currently default to the smaller mechanism. This page sets both to the complete one and names what changes.

## 1. the annuity ships at genesis

chain-economy §12 row 5 defaults to "pulse only at genesis; the annuity with the reports rule set". The default becomes the annuity at genesis: an active position earns the pulse when its stake lifts an axon to a new high (§5.3) and earns the annuity every epoch after, the time-integral of its target's focus growth attributable to the link ([[tru/specs/rewards|rewards]] §11, §12). Foundational work, the code and specs of the stack among it, is paid as it matters, which is what the epistemic asset of cyber/whitepaper.md §14.1 promises.

Moves: tru's epoch engine computes the per-position annuity from consecutive φ* (the attribution it already does for the pulse, integrated over epochs instead of settled once); chain-economy §5 gains the annuity term in `pay(c)`; the conformance vectors gain an epoch sequence where a link earns nothing at settlement and grows over ten epochs.

## 2. the mint budget scales with stake

chain-economy §6.1 pays knowledge from a cut of the miner subsidy, `SUBSIDY · θ^½`, capped at half the subsidy. The cut is dynamic in θ, the staked share of supply, but the ceiling is fixed by a constant emission, so more stake above the cap buys nothing. rewards.md §8 keeps the mint "a budget of its own, held separate from the security budget", bounded by the global focus shift. The proposal restores that separation:

```
subsidy(h)   = SUBSIDY                                   // to the miners, constant, the security floor
mint(E)      = floor(active(E) · MINT_PPM_EPOCH / 10^6)  // to the focus pool, proportional to active stake
pay(c)       = min(cap(c), floor(mint(E) · x(c) / X))    // divided by attribution as today (§5.5)
```

`active(E)` is the active stake at the boundary (§5.2); `MINT_PPM_EPOCH` is a chain parameter such that the yearly mint on active stake is a few percent at genesis, tunable by the [[metabolic-oracle|metabolic oracle]] later. The mint goes to the focus pool and nowhere else; an epoch with no positively attributed stake mints nothing (the pool keeps it, as today). Supply becomes `S0 + h · SUBSIDY + Σ mint(E)`, a function of the chain, no longer of height alone; the conservation check of §6.3 adds the mint column.

Why this shape: the yield per unit of active stake is constant in θ, so staking is never discouraged by others staking; the subsidy no longer competes with the pool, so the miner floor is whole; the pool grows exactly with what is at risk on the graph, which is the owner's requirement.

## 3. what changes where

| file | change |
|---|---|
| soft3/specs/chain-economy.md | §5.5 annuity term; §6 mint line and supply; §12 rows 1 and 5 closed |
| tru/specs/rewards.md | §10 names the mint as the second emission line; §11 unchanged |
| tru (epoch engine) | annuity integration; `mint(E)` |
| soft3 (kernel economy) | coinbase step credits the pool `mint(E)/240` per block of the epoch, or once at the boundary |
| conformance vectors | annuity sequence; mint at θ = 0, 1 %, 25 % |

## 4. acceptance

- a position staked at epoch 1 on a particle whose focus grows for ten epochs earns in each of the ten, bit-exact on two nodes;
- with no active stake the pool receives nothing; with θ doubled the pool doubles;
- supply equals `S0 + h · SUBSIDY + Σ mint` at every boundary;
- the self-linking ring of §9 stays capped by `cap(c)` per epoch.

related: [[soft3/specs/chain-economy|chain-economy]] · [[tru/specs/rewards|rewards]] · [[metabolic-oracle|the metabolic oracle]] · cyber/whitepaper.md §14.1
