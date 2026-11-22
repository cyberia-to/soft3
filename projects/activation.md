---
tags: cyber, soft3, project, cyberlink, valence, tru, pussy, launch
crystal-type: process
crystal-domain: cyber
kind: project
status: proposed
owner: master
serves: launch spacepussy ($PUSSY)
part of: complete rewards
lane: wide
date: 2026-11-21
alias: activation, repair the valence, the valence repair, signed activation, incorrect valence
---
# activation — repair the valence

A staked cyberlink on pussy's chain says two things the truth serum can score — which side its author stands on, and what its author expects of the crowd — in a record the owner has chosen between two candidates: one signed number on the link, or two links.

It blocks the launch: without the second statement the serum has nothing to measure, and the chain pays agreement as if it were knowledge.

## the owner's word · 2026-11-21

Translated from the owner's messages (no link exists), in the order they were said while an explainer of truth was rewritten four times in one day.

- on a table that showed a side beside the valence: "we have no construction of for or against in the cyberlink — there is the valence, which is exactly for this. where did we mix it up?"
- "my assumption was that the valence is the meta-prediction, and the information is, as it were, laid in the very fact of the link — the link said: this is true."
- "I dreamed that every edge has true or false — it does many good things: it composes well and it lies well on the structures of neural activations."
- "we need to create markets of truth — we have the coupling with two curves, one for the truth and one for the untruth."
- "two valences for the protocol is too much"; and on a prediction placed on a second link: "it seems the record is dirty and the valence is simply stretched over two different meanings."
- "maybe the valence will simply have more values? a matrix of four decisions."
- "so you propose, in place of the valence, a field activation, a signed hundred? in our field theory a hundred is not a round number."
- "I think this is a project in soft3, and it is a blocker for the launch of pussy — formulate the project as the solution of the problem of the incorrect valence. In the project consider the idea of the split into two links — there is sense in it too. I am not ready to decide now."
- later the same night, after the explainer was rewritten on candidate A and the size of the field was settled at one byte: "we really did find a neat way out — it comes out verbose enough; now there is certainly enough information and the construction of the protocol is not dirty — I like it." This is the owner's leaning toward candidate A; the choice is still his to sign after V1.

## the problem

The serum of [[rewards]] §5 needs two numbers from every staker: its own answer, and its prediction of how the others answer. The winner is the answer more common than the crowd predicted. One three-valued field cannot be both, and the pages read it three ways.

| where | the valence is | the second number |
|---|---|---|
| tru `specs/rewards.md` §5, `docs/explanation/valence.md` (BTS section) | the prediction of the crowd | the answer is the fact of the link with its stake |
| tru `docs/explanation/valence.md` (what each value does), soft3 `specs/chain-economy.md` §4.2 — what runs | the side: `+1` adds to the link, `−1` takes from it | none; a report in basis points is promised to a later rule set (§7.4) |
| tru `docs/terms/market.md` | the prediction of where the market converges | the side is a purchase of true or false on the link's market, a separate act |

What it costs today: the chain has no prediction at all, so surprise is the constant 1 and a copy is paid like a discovery (chain-economy §5.4); the scorer in tru takes a belief and a prediction as two fractions and has no record to read them from (`rs/truth_scoring.rs`); and no page can tell a reader what `−1` means.

## what was ruled out, and why

| idea | why it fails |
|---|---|
| a second valence on the link | the owner: too much for the protocol |
| a sign on the amount | it exists already: the wire is `amount: u64 ‖ valence: i8`, so the amount's sign is the valence |
| the prediction read from the price at the start of the epoch | no neuron predicts anything; an epoch is won by the larger new money, so capital is paid and knowledge is not |
| a three-valued prediction | it cannot say "a tenth will answer otherwise": a majority that predicts itself leaves every other answer at zero, and any small ring is then surprisingly popular |
| disagreement as a competing link from the same particle | it leaves an edge without a false side: no coupling, no inhibition, and the owner's edge that is true or false is gone |

## candidate A · activation — the owner's leaning

The valence byte keeps its place and widens. The cyberlink carries one signed number, the activation:

```
activation: i8      0            no part: the stake weighs the link, takes no side, earns nothing
                    +1 … +127    true,  and I expect m/128 of the active stake on this link to stand against me
                    −1 … −127    false, and I expect m/128 of the active stake on this link to stand against me
```

The sign is the answer. The size is the prediction, counted from the other side (the owner, 2026-11-21, on the reading "a small number is the strongest signal": "and what if we turn it over?" — "yes"). So the size reads as strength: `+1` is "true, as everyone thinks", quiet; `+102` is "true, and four in five will disagree", loud. The common is silent and the new sounds, which is how a neuron signals a prediction error. Shouting does not pay: the size is still a prediction, checked against the count.

An edge keeps two reserves, true and false, the sums of the stakes of each sign, and the coupling runs on them. The crowd's prediction of the true share is the stake-weighted mean of `1 − m/128` over the true stakes and `m/128` over the false ones.

| for | against |
|---|---|
| one field, one stake, one signal; the pair of answer and prediction comes from one neuron and cannot be separated — the form the theorem is proved for | the root term changes its meaning, and with it its name: every page, the cybergraph specification, the chain's validity rule for kind 2, cyb's stake screen |
| a majority can say "a tenth will disagree", so a ring smaller than a tenth surprises nobody and pays: the hole of the three-valued prediction is closed as the original closes it | a majority that names too little dissent is exposed to any ring larger than it named; by the theorem that is the price of a bad prediction, and V1 measures it first |
| the rehearsal network's `+1` and `−1` keep nearly their meaning — true, false, as everyone thinks — so the record may not need a restart | a neuron must name a number when it stakes; cyb needs a control and a sensible default |
| a power of two: the share is a shift, no rounding; 0 and 1 cannot be said, which a logarithmic score needs | — |
| the wire keeps its size, one byte; half a byte is too coarse (eighths) and saves nothing on a byte-aligned record; more bytes buy a precision no neuron has | — |
| the scorer already takes a belief and a prediction | — |

### what one byte on an edge gives the system

| to | what |
|---|---|
| the protocol | one primitive: no second field, no new kind of signal, no particle with a rule of its own; a range check of 256 values is one lookup in a proof; every count is by stake and linear, so splitting keys buys nothing |
| the edge | three quantities where there was one: how the crowd answered, how it expected to answer, and the gap between them |
| knowledge | a second dimension beside weight: a positive gap marks private knowledge, a negative one a herd; search can tell the contested and the new from the merely popular |
| karma | calibration: how well a neuron's expectation of the crowd meets the count — it cannot be bought and it is measured on every stake |
| the compiled model | a signed graded value on every edge in place of three values |
| agents | a model has probabilities; "this is so, and seven in ten will agree" is its ordinary output, so machine neurons are scored on calibration beside people |
| the vocabulary | neuron, link, focus, activation: excitation and inhibition stop being a metaphor |

### false damps an edge and does not flow back

The sign is spent before the kernel: an edge's two reserves become one weight that is never negative, and focus diffuses over that weight as it does today. A link the crowd calls false carries less; nothing runs backward. The owner: "it is somehow really super logical."

What the weight reads is a line of the contract (tru `projects/truth-market-at-genesis.md`, line 4): the balance of money on the two sides, or the serum's estimate of the true share. On the second, a link is damped when an informed minority wins, though more money stands on true — focus follows knowledge and not capital. tru `specs/strong-truthfulness.md` states it as the design rule: reinforce by the surprise, never by the raw price. Measured by nobody yet.

### the name

Valence means three values to everyone who has read a page of this stack; a name kept over a changed meaning brings the confusion back. The owner, 2026-11-21: "it used to seem to me that the valence was the last word — but what we have squeezed into one byte is a really crazy amount of fundamental information, and the semantics did not suffer; rather the reverse — activation is cool." The rename passes through tru's vocabulary, the cybergraph specification, the chain specification, the node and cyb: task V6.

## candidate B · two links

The valence stays three-valued and means one thing everywhere: the statement this link makes is true, or false. The second number is a second link, whose target is the particle of the edge.

| the link | `+1` | `−1` |
|---|---|---|
| `p → q` | this is true | this is false |
| `crowd → axon(p, q)` | the crowd will hold it true | the crowd will hold it false |

Each has two reserves and a price. The second resolves every epoch against a fact, the true share of the first, so it is a prediction market in the ordinary sense.

| for | against |
|---|---|
| the record does not change at all: one primitive used twice, and it composes | the protocol must know one particle, `crowd`, and score its links against the edges they point at: a rule that reads content |
| the prediction is priced by a market that resolves, and anyone who sees it wrong profits by correcting it; speculators need hold no view of the truth | two stakes for one opinion: a neuron decides how to divide its money between belief and prediction |
| predictions give attention to the claim as an object, which a contested claim arguably deserves | the pair is no longer one neuron's: the theorem's score is a sum of two independent parts, so the incentives plausibly survive, and that is an argument, not a proof |
| — | one neuron's prediction is two-valued; only the aggregate is graded |
| — | twice the state for an edge that carries predictions |

## side by side

| | A · activation | B · two links |
|---|---|---|
| fields changed | the valence's range and meaning | none |
| signals per staker | one | two |
| the theorem | carries over as stated | by argument; to be shown |
| special rules | none | the `crowd` particle |
| what a neuron enters | a side and a share | two stakes and two sides |
| migration | the alpha from block 0 | none for the record; new settlement |

## the tasks

| id | task | repository | gate | waits for |
|---|---|---|---|---|
| V1 | candidate A first, then B, under the same crowds on tru's scorer: an honest majority, an informed minority, a ring on a junk link, a whale, a copier, a majority that names too little dissent, and the rehearsal network's old `±1` read as activations; who pays whom, in numbers | tru, simulation | a receipt with the seven crowds for A and for B | — |
| V2 | the decision put to the owner as a prepared card, with V1's numbers and one page to try each candidate on | apparatus | the owner's signed answer | V1 |
| V3 | the record in the contract: the cyberlink's field, the edge's reserves, the crowd's prediction, the score, in integers — lines 1 and 2 of tru `projects/truth-market-at-genesis.md` | cybergraph `specs/`, tru `specs/`, soft3 `specs/chain-economy.md` §4.2, §5.4, §7.4 | no page reads the field two ways | V2 |
| V4 | the code: the validity of kind 2, the edge and position records, the settlement's surprise, the scorer joined to the epoch engine, cyb's stake screen | cybernode, tru, cyb | conformance vectors; adversary tests | V3 |
| V5 | the rehearsal: the alpha from block 0 on the new record; the explainer's epoch reproduced on chain | cybernode | a receipt in `audit/chain/` | V4 |
| V6 | the pages: the explainer, `valence.md`, the term pages, the vocabulary; the name of the field | tru, cybergraph, cyber | one meaning in every page | V2 |

V1 starts now and decides nothing by itself. The explainer [truth, from claim to payout](https://cyberia.blog/soft3/tru/docs/explanation/truth/) is written on candidate A while the decision is open, and says so.

## the measure

A neuron stakes on a link from cyb, enters a side and what it expects of the crowd, and at the next settlement every node computes the same surprise for that edge from the records alone.
