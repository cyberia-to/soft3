---
tags: cyber, soft3, project, economy, emission, tru, pussy, launch
crystal-type: process
crystal-domain: cyber
kind: project
status: proposed
owner: master
serves: launch spacepussy ($PUSSY)
part of: launch spacepussy
lane: wide
date: 2026-11-22
alias: hybrid economy at genesis, the hybrid economy at genesis, dynamic inflation, correct inflation, adaptive emission at genesis
---
# the hybrid economy at genesis

Pussy's chain mints by the law of [[adaptive hybrid consensus economics]]: the budget of an epoch is a security floor derived from what an attack costs plus the fees, it is split between work and active stake by the allocation curve, and the floor, the exponent and the burn share move by feedback on what the chain observes — in place of a fixed subsidy with a hard cap.

It is a component of the launch: the owner will not launch on the constant.

## the owner's word · 2026-11-22

Translated from the owner's messages (no link exists). On an analysis of the staking incentive made from the chain's own formula: "something is definitely wrong — in my proofs there was another story. It seems you took the crippled implementation and ignored the good specification with the proofs of convergence at the marks of about 40 %. Raise it; one has to think, and not by a crooked formula taken out of laziness." On the long form, found in the history of cybics, and its baseline that settles at a security margin of 1.52: "yes, this is it. I need to be sure that there is a project for the correctness of inflation as a component of the launch of spacepussy — there is a crutch there now; why code such a thing when a competent specification exists. Make sure it is clean there. Return the long work into scope."

Earlier, 2026-11-16, in soft3 `proposals/rewards-at-genesis.md`: "the mechanism must be dynamic in how much is staked"; "either the mechanism exists or it does not".

## the law

The long form is back in the tree as cyber `specs/adaptive hybrid consensus economics.md` ([cyber#161](https://github.com/cyberia-to/cyber/pull/161)); its short statement is cyber `specs/adaptive hybrid economics.md`, and tru `specs/rewards.md` §10 carries the equations.

| part | the law |
|---|---|
| the budget | `G = φ · M + (1 − β) · F`: the floor times the supply, plus the fees that are not burned |
| the floor | derived, never chosen: `φ ≥ k · (TVL / M) · r`, the margin of safety times the share of value an attack can take times the attacker's cost of capital |
| the split | `R_PoS = G · S^α`, `R_PoW = G · (1 − S^α)`, `S` the active staked share |
| the equilibrium | theorem 3.1: for any budget one staked share exists at which the yield of a stake equals what capital costs, `S* = (G / (r · M))^(1/(1−α))`; stable by proposition 6.2 |
| the control | `α`, `β` and `φ` move by proportional, derivative and integral feedback on observed errors; theorem 8.1 bounds the gains by a Lyapunov argument; theorems 10.1 and 10.2 read the rules as gradient ascent on welfare and give convergence to a neighbourhood of the optimum |
| the start | `α₀ = 0.5`, `β₀ = 0`, `φ₀ = 0.03`, a warm-up with wider bounds and no integral term, then full control |
| the measures | the security margin held at `k = 1.5` or above; alerts below 1.2, on a staked share under 0.1 or over 0.9, on net inflation over 5 % |

Where the mark of about 40 % comes from, by this page's own arithmetic, to be confirmed in H2. At the starting floor the law gives `S* = (φ₀ / r)²` at `α = ½`: 36 % when capital costs 5 % a year, 40 % at 4.7 %. This equilibrium is the stable one of theorem 3.1 and proposition 6.2: below it a stake earns more than capital costs and stake enters, above it stake leaves.

One consequence the short page does not state, and a warning with it. With the floor in the budget the cost of capital cancels: `S* = (k · TVL / M)^(1/(1−α))`. When the value at risk is given from outside, as the law assumes, that is again a stable point. When the value at risk is taken to be the stake itself, `TVL = S · M`, the point `S* = 1 / k²` exists — 44 % at `k = 1.5` — and it is unstable: more stake raises the floor, the floor raises the yield, the yield draws more stake, and the share runs to a bound; less stake runs the other way. So what the chain counts as value at risk decides whether the system settles or runs away, which is why it is the first line the contract settles. An earlier revision of this page named `1 / k²` as the mark; that was wrong.

## what the law implies for the staking budget

Arithmetic on theorem 3.1 at `α = ½`, with `g = G / M` the gross reward rate a year and `r` what capital demands; to be confirmed by H2.

| | formula | at `r = 5 %` |
|---|---|---|
| the staked share | `(g / r)²` | 4 % at `g = 1 %`, 16 % at 2 %, 36 % at 3 %, 64 % at 4 %, all of it at 5 % |
| the staking budget, a year, of the supply | `g² / r` | 0.2 %, 0.8 %, 1.8 %, 3.2 %, 5 % |
| the work's budget, a year, of the supply | `g · (1 − g / r)` | 0.8 %, 1.2 %, 1.2 %, 0.8 %, nothing |

1. Nobody sets the staking budget. It is the square of the gross rate over the cost of capital: double the rate and stake takes four times as much.
2. The work's budget is a hill. It is highest at `g = r / 2`, a quarter of `r`, and it is nothing at `g = r`. A floor raised past half the cost of capital pays the miners less, and at `g = r` the chain has no paid work at all. The starting floor of 3 % with capital at 5 % stands just past the top.
3. The staked share follows `1 / r²`: capital that demands twice as much stakes a quarter as much. `r` is what the least informed staker demands, risk of losing to the count included, and no observable of the chain gives it.
4. Fees enter the same `g`. When they push it toward `r` the floor has to fall or the burn has to rise, or the work is starved: this is what the loops on `φ` and `β` are for, and the reason they cannot be left for later.
5. The chain's constant of 1 % with its cap sits at the bottom of the first column: a staked share of 4 %.

## what the chain runs

soft3 `specs/chain-economy.md` §6.1, in consensus on the rehearsal network:

| the law | the chain |
|---|---|
| the budget is the floor plus the fees | one constant `SUBSIDY`, 1 % a year of the genesis supply |
| the floor follows the value at risk | no floor; nothing reads the value at risk |
| the split is `S^α` on its whole range | `min(½, S^½)`: above a staked quarter, stake moves nothing |
| `α` moves between 0.3 and 0.7 | `α = ½`, a constant |
| `β` burns a share of the fees | no fee leg in the budget |
| feedback on observed errors | none; `tru::Pid` exists and no loop runs it |
| supply follows what security required | `S₀ + h · SUBSIDY`, a function of height |

What it does to the incentive: with a budget of 1 % a year and capital at 5 %, the chain's own curve settles at a staked share of 4 %, a tenth of the law's, on markets too thin for a stake against an error to matter. The proposal of 2026-11-16 removes the cap and keeps the constant: it says the floor "has no inputs before launch".

## what exists

| piece | where | state |
|---|---|---|
| the split and the floor's formula over fixed point | tru `rs/allocation.rs`, receipt `audit/allocation-curve.md` | merged; the simulation sweeps the curve, not the dynamics |
| the control primitive | tru `rs/pid.rs` | merged: clamped, anti-windup; no loop, no gains, no observable defined |
| the long form | cyber `specs/` | restored 2026-11-22; appendix A's complete proofs are placeholders; theorem 2.3 carries a revised proof |
| the simulations of the long form | none in any tree | the paper reports results; the notebooks are not in a repository |

## what the contract has to settle

1. The value at risk on a chain with no contracts and no bridges: the stake on links, the focus pool, the supply itself. The law's `TVL` was written for a chain with a market in front of it.
2. The attacker's cost of capital and the staker's, with no price oracle: the law says the core loop needs none; the floor's formula names `r`.
3. The observable of work: hashrate when the work is settlement sampling, in the chain's own units.
4. The fee leg: the root chain has fees per signal and no service fees; whether `β` has anything to act on at genesis.
5. The clock: the law's epoch is a week and its warm-up a year; the chain's epoch is two hours.
6. What counts as active stake once the record is the activation (soft3 `projects/activation.md`): a quiet stake on a link nobody contests carries no risk and would draw the floor.
7. Standing stake counts in `S` and only stake added in the epoch is paid, until the annuity runs (soft3 `proposals/rewards-at-genesis.md` §1).
8. The supply's invariant: with a floor that moves, supply is a function of the chain's history; conservation is restated and every node checks it.
9. Whether the starting floor of 3 % is the launch value, and who may move a bound during the warm-up.

## the tasks

| id | task | repository | gate | waits for |
|---|---|---|---|---|
| H1 | the contract: the budget, the floor, the split and the three control loops as integers a node computes at an epoch boundary, the nine lines above settled, every contradicting clause listed old → new | tru `specs/`, soft3 `specs/chain-economy.md` §6, cyber `specs/` | no constant stands in for a mechanism; two review rounds; the owner approves | — |
| H2 | the dynamics simulated on the chain's own parameters: the seven scenarios of the long form, the convergence of the margin and of the staked share, the fixed point `1 / k²` confirmed or refuted | simulation, tru | a receipt with the trajectories; the paper's baseline reproduced | — |
| H3 | the proofs made whole: appendix A written out, theorem 2.3 with one proof | cyber `specs/` | no placeholder in the page | — |
| H4 | the engine: the floor and the loops over fixed point, deterministic on two machines | tru | tests, digests, the cost per epoch | H1 |
| H5 | the chain: emission by the law, the pool and the coinbase fed by it, conservation restated, conformance vectors | cybernode, soft3 | vectors blessed; adversary tests on the loops | H1, H4 |
| H6 | the rehearsal: the alpha through a warm-up with stake entering and leaving, the staked share and the margin read from the chain | cybernode | a receipt in `audit/chain/` | H5 |
| H7 | the pages: launch.md row 27, pussy's critical path, `proposals/rewards-at-genesis.md` §2 folded into this project | cyber, cyberia, soft3 | no page calls the floor's control a later rule set | H1 |

H1, H2 and H3 start now and wait for nothing.

## beside the other two

soft3 `projects/activation.md` settles what a stake says; tru `projects/truth-market-at-genesis.md` settles how an epoch scores and pays it; this project settles how much there is to pay and how it is divided with the work. Line 6 above is shared with the first, line 7 with the second.

## the measure

On the public chain the staked share, the security margin, the floor and the split are read from the epoch record, they move when stake enters or leaves, and no number in the chain's emission is a constant somebody chose.
