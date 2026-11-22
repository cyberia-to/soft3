---
tags: cyber, soft3, project, rewards, tru, pussy, launch
crystal-type: process
crystal-domain: cyber
kind: project
status: proposed
owner: master
serves: launch spacepussy ($PUSSY)
part of: launch spacepussy
lane: wide
date: 2026-11-22
alias: complete rewards, full rewards, rewards complete in production, the closed loop of rewards
---
# complete rewards

On the public pussy chain a stake on a link earns and loses by the whole law of [[rewards]], and the loop closes: what an epoch learns changes what the next epoch weighs.

This page is the one place where the parts of that law, what was learned about them, and the pull requests that carry them stand together. The parts are projects of their own and children of this one.

## the owner's word · 2026-11-22

Translated from the owner's messages (no link exists). After it was shown that the miners' samples and the serum do not read each other: "do I understand correctly that our mechanism turns out not to be closed?" Then: "how do we structure this so that the rewards on spacepussy in production are complete — let us add a project about complete rewards and include in it all these insights and the existing pull requests."

It stands on 2026-11-04: "the full implementation of the rewards specification is necessary … without it I will not launch the chain"; and 2026-11-16: "either the mechanism exists or it does not; the key mechanism must be complete at once — rewards are the key mechanism."

## the loop, and where it is open

| step | what the law says | on the chain today | carried by |
|---|---|---|---|
| 1 | a neuron stakes on a link and says a side and what it expects of the crowd | a side only: the three-valued valence | [[activation]] |
| 2 | the serum counts the surprise of every link | a constant: every surprise is 1 | [[truth market]] |
| 3 | the surprise weighs each contribution in the epoch's game | every contribution weighs 1 | [[truth market]], with the work below |
| 4 | the miners sample the fair division of that game, and the sample is the chain's work | sampled, in a branch that is not merged; the game is the unweighted one | the work, below |
| 5 | the losing side pays the winning side; the winners take the premium | the premium only; nobody loses | [[truth market]] |
| 6 | how much there is to pay follows what security costs and what is staked | one constant a year, with a hard cap | [[hybrid economy]] |
| 7 | the score becomes karma, and karma and the serum's estimate weigh the link | karma is 1 and a link weighs its money | [[truth market]] |
| 8 | focus of the next epoch is computed over those weights | over the sums of stake | [[truth market]] |

Step 8 returns to step 1: whoever was right weighs more the next time. Today steps 1 and 4 run, the second in a branch, and nothing an epoch decides reaches the next one except the sums of stake. The chain pays agreement and capital.

One opening is in the law's own text: if focus weighs a link by its money, the serum's verdict is a record of payments and changes nothing the network holds true. What closes the loop is the rule of tru `specs/strong-truthfulness.md` — reinforce by the surprise, never by the raw price — and no contract states it yet.

## the parts

| part | what it settles | where | state |
|---|---|---|---|
| the work samples the rewards | what a miner computes and how it is paid: rewards §7 and §8 | tru `tasks/rewards-complete.md`; the director's P14, P15 | the specification awaits the owner; the code is a round behind it |
| activation | what a stake says | soft3 `projects/activation.md` | proposed; the owner leans to the signed byte |
| the truth market | how an epoch scores a stake and moves value: the serum, the market on a link, the redistribution, karma, the weight a link carries | tru `projects/truth-market-at-genesis.md` | proposed |
| the hybrid economy | how much there is to pay and how it divides with the work | soft3 `projects/hybrid-economy-at-genesis.md` | proposed |
| the annuity | what a link earns after its first epoch | soft3 `proposals/rewards-at-genesis.md` §1 | proposed, on a branch |

## what was learned, 2026-11-20 to 2026-11-22

Each line is recorded where its readers are; this list is the index.

The work:

1. A probe is hard by construction and never by a measurement: the base's syntropy without a drawn half of its axons, drawn by a 64-bit nonce. Recorded in tru#45.
2. One payout formula: the exact gradient counts as 64 samples, no threshold switches a rule, no sample is discarded. Recorded in tru#45; the code still pays by the threshold.
3. The miners' samples do set a payout in the node's branch, and they sample the unweighted game: the node's constant for the surprise is 1. The game has to be the one the serum weighs.

The record:

4. The serum needs two numbers from every staker, a side and a prediction of the crowd, and a three-valued valence carries one. The pages read it three ways.
5. The activation: one signed byte, the sign the side, the size the share of the stake expected against, in 128ths. The common is quiet and the new is loud; the rehearsal network's plus and minus one keep nearly their meaning, and a network of only those has no surprise at all.
6. False damps a link and nothing flows backward: the sign is spent before the kernel.

The score and the money:

7. The losing side always pays and the winning side has a premium on top. The chain pays the premium alone.
8. Settlement is paid in rounds: a slice of the losing stake each epoch, in proportion to the surprise, and it stops when the crowd expects what it sees. The slice is also the security parameter of the count: what a captured count can take in an epoch.
9. The count pays for knowing more than the crowd expected. Honesty is the best play and no guarantee.
10. Known limits: a minority that shares one error and expects to be few wins one count; a majority that names too little dissent is exposed to any ring larger than it named; a majority of all stake that inverts its reports wins.

The budget:

11. The chain mints a constant with a hard cap where the law has a floor derived from attack economics, fees, the whole curve and feedback. The long form of the law had left every working tree and is restored.
12. At the law's starting floor of 3 % a year a stake yields 30 % when 1 % of the supply is staked and 5 % when 36 % is: the network starts itself and stops near a third. On the chain's constant it stops at 4 %.
13. The miners' pay is a hill: highest when the gross rate is half of what a staker demands, nothing when the two are equal. A floor cannot be raised for the sake of the work past that top.
14. What the chain counts as value at risk decides whether the control settles or runs away.

The pages:

15. The explainer [truth, from claim to payout](https://cyberia.blog/soft3/tru/docs/explanation/truth/) is written on the activation and says the record is under decision. Every rate in its calculator is invented and the contract fixes the real ones.

## the pull requests

| pull request | what it carries | state on 2026-11-22 |
|---|---|---|
| [tru#45](https://github.com/cyberia-to/tru/pull/45) | rewards §7 and §8 complete: the lottery, the probe by construction, the one payout formula; its §16 still fixes karma, price, surprise and the annuity at constants | open, awaits the owner |
| [tru#46](https://github.com/cyberia-to/tru/pull/46) | the game, the ordering and the ticket on the epoch engine | open; behind tru#45 by the probe and the formula |
| [cybernode#18](https://github.com/cyberia-to/cybernode/pull/18) | the ticket as the chain's work; the settlement that reads the samples | open; behind by the same |
| [soft3#682](https://github.com/cyberia-to/soft3/pull/682) | the facade without a GPU miner; the genesis identities | open |
| [soft3#689](https://github.com/cyberia-to/soft3/pull/689) | the annuity at genesis and the split without its cap | open |
| [cyber#161](https://github.com/cyberia-to/cyber/pull/161) | the long form of adaptive hybrid consensus economics, restored | open |
| tru#8, tru#9, tru#10, tru#11, plumb#2 | the market's cost function, the serum's scorer at scale, the allocation curve, the annuity's accrual, the stake-yield term | merged; none is called by the chain |
| plumb#15, mudra#5 | the market's reserves and positions; the audit of valence on the wire | closed without merging |
| tru#47 to tru#61 | the explainer and its calculator | merged, live |

## the tasks

| id | task | gate | waits for |
|---|---|---|---|
| C1 | the work lands: tru#45 to the owner; one round of code for the probe and the formula in tru#46 and cybernode#18; the vectors; the alpha on the new ticket | the alpha mines 24 hours on two machines by the specification as written | the owner's approval of tru#45 |
| C2 | the record is chosen: activation V1 and V2 | the owner's signed answer | — |
| C3 | the contract of the score and the money: truth market T1, with the weight of a link named — the surprise, never the raw price | no constant stands in for a mechanism | C2 |
| C4 | the contract of the budget: hybrid economy H1, with H2 and H3 beside it | the same | — |
| C5 | the game the miners sample is the game the serum weighs: rewards §3 and §7 stated together for the root chain, then in the engine and the node | a sample of a copy is zero | C1, C3 |
| C6 | the annuity: soft3#689 landed on the release line or folded into C3 | an old link that keeps carrying focus earns in a later epoch | C3 |
| C7 | the closed loop on the alpha, the measure below | a receipt in `audit/chain/` | C1 to C6 |
| C8 | the explainer and the launch pages follow the contracts: real rates, no page calling a part a later rule set | read against the contracts | C3, C4 |

C1, C2 and C4 wait for nothing but the owner's two answers. C3 is the longest.

## the measure

One run on the public rehearsal network, read from the chain's records by anyone:

- an informed minority stakes on a link the majority stakes against;
- the count names the minority's side the surprising one;
- the majority's stakes pay the minority's, by a slice, and the minority takes a premium whose division the miners sampled in the game the surprise weighs;
- both sides' karma moves;
- in the next epoch that link carries more focus than its money alone would give it.

While any line of this fails, the rewards are not complete and the loop is open.
