---
tags: cyber, soft3, proposal, state, economics, tok, bbg, foculus, oikos, emission
crystal-type: process
crystal-domain: cyber
status: proposed
date: 2026-10-17
alias: state economics, dna economics, state pricing, burn or lock, asymptotic tail
---
# state economics — price the act, free the being, fund the machine forever

> two products, three channels, four rules, one curve. **writing state is energy (landauer): pay once at the margin. holding state is persistence: free forever — no rent, no dormancy, no eviction, no death.** security is funded by an asymptotic emission tail whose cap is the size of the field — eternal budget, provable bound, and this is not a trade-off. this page spec-ifies §3e of [[steps-blocks-epochs]] (state-growth fees) and answers the funding question that [[oikos]] left open: who pays the machine in 2140.

## 1. why this page

the question *who pays for state, and who pays for security* is fundamental, and every surveyed chain answers it by conflating two independent layers. survey, one line each:

| chain | state mechanics | the real lesson |
|---|---|---|
| solana | rent / rent-exempt deposit to validators | one-time deposit = permanent exemption: state priced once, badly |
| near | storage staking: lock tokens per byte, refund on delete | clean deposit model; price rides the token, no ladder |
| sui | storage fund endowment + rebate on delete | pay once, fund validators from a fund forever |
| **nervos ckb** | capacity: 1 CKB = 1 byte; two-layer emission (capped base + eternal uncapped secondary) | the structure is right — state as property, separate funding layer; the secondary emission is left **unconverged** |
| ergo | storage rent to miners after emission | rent conflates pricing with security funding; eviction taxes money itself |
| parano1d | burn on net-new × occupancy ladder 1-2-4-8, expansion valve, holding free, unbounded 1-NOID tail | the best pricing layer; the tail has no cap |

the synthesis: **nervos found the structure (two layers), parano1d found the pricing layer, the asymptotic curve closes the funding layer.** rent as a class dies: the tail funds security forever, so state pricing is freed to be purely about scarcity discipline.

## 2. the two products (separate the flies from the cutlets)

1. **creation / deletion = energy.** writing information into shared consensus is irreversible work; the price is one-time, paid at the act (landauer). deleting works *for* the system and restores capacity — it is free and reduces the bill.
2. **holding = persistence.** being is not an action and is not a service. a gene pays no rent for sitting in the genome. therefore: **no rent, no dormancy rule, no eviction — zero mortality terms anywhere in the mechanism.** an agent that wants to reserve itself for a long term does not pay repeatedly — it *locks* (rule 1).

## 3. the mechanism — four rules + one curve

**rule 1 — the price of writing: burn or lock, holder's choice.** a net-new slot costs `W(φ) = W₀ × M(φ)`, where φ is state occupancy and `M ∈ {1, 2, 4, 8}` steps at φ ≥ {50 %, 75 %, 90 %}. payment has two forms:
- **burn** — the fee is destroyed. for ephemeral state: apps, short cycles. payment to *everyone* through deflation.
- **lock** — `k × W(φ)` worth of token is frozen while the slot lives, unlocked on deletion. for reservations: agents, long-term positions, books. cost of holding = immobilized capital's opportunity cost; mechanically zero, terminally infinite. **this is the no-death reservation: keep the lock, keep the slot, forever.**
- deletion is always free and always netted: only `created − deleted` slots are priced in a transaction.

**rule 2 — holding is free.** no member of any formula charges for `be`. permanence is unconditional once written or locked.

**rule 3 — expansion, the stress valve.** 10 of the last 18 finalized blocks at φ ≥ 75 % → capacity ×2, φ → φ/2, M → 1×. growth of capacity is bookkeeping (old root = left child, canonical empty tree = right), never an event.

**rule 4 — the metabolism: one asymptotic curve.** `supply(h) = C · (1 − q^h)` with `C ≤ p` (the field size, goldilocks). one curve feeds two channels by fixed proportion: **earners** (miners and stakers — the two lotteries of [[steps-blocks-epochs]] §5) and the **machine** (a fixed share that may be 0). eternal security budget, hard cap, closed form provable inside the step relation.

**anchor.** `W₀` is a fraction of the block reward, not a bare constant — the price of writing scales with the value of the network; early dust is never subsidized forever for a fixed penny.

## 4. what the mechanism refuses

as important as what it does: no rent (existence is not tariffed) · no death (no dormancy, no eviction) · no auction (no demand oracle, no EIP-1559 orchestra) · no miner-earned growth fee (a miner paid from state scarcity is structurally long that scarcity — burn and lock sit with the *holder*, the miner stays neutral) · no conflation of state pricing with security funding (two layers, two channels, one direction each).

## 5. relation to the other pages

- **[[steps-blocks-epochs]] §3e**: this page is its specification — the occupancy ladder, the expansion valve, and now the burn-or-lock choice. the child spec closes when these rules are written into `foculus/specs/parameters.md` with numbers.
- **[[oikos]]**: books settle value at home; citizenship is a long-term reservation → books live in the **lock** channel by default. agents of the graph reserve their presence by lock, not by rent — matching the no-death position.
- **[[term]]**: contract state follows the same rule: creating a live right pays at the margin; holding it is free; the lock variant is what timelocked vaults and long rights use.
- **monetary policy**: the asymptotic tail replaces both ergo-rent and parano1d's unbounded tail. *not a trade-off*: the sum of an eternal geometric series is finite, and the cap chosen at the field size makes the whole monetary policy one closed form.
- **[[bbg]]**: state lives in the polynomial; φ is computed from committed counters only — the entire mechanism is step functions and closed forms, arithmetizable inside the step relation.

## 6. the complexity budget — six numbers

| constant | meaning | default posture |
|---|---|---|
| `W₀` | base price of a net-new slot | fraction of block reward (anchored) |
| `M` ladder | 1-2-4-8 at 50-75-90 % | consensus constants |
| expansion window | 10 of 18 finalized ≥75 % | consensus constants, rescaled to step cadence |
| `q`, `C` | tail decay, cap ≤ field size | monetary policy, one proposal to change |
| `k` | lock coefficient | one constant |
| machine share | tail fraction to the machine | may be 0 |

## 7. open questions

1. **per-dim φ**: bbg carries many public dimensions; is occupancy priced per dimension or on an aggregate? (books suggest: aggregate capacity, per-book filters.)
2. **lock without unlocking horizon**: an agent that never deletes holds a lock forever — is the opportunity cost a sufficient discipline, or does `k` need an occupancy-dependence too?
3. **rescaled expansion window** for a 1 s step cadence: 10-of-18 blocks becomes what in steps?
4. **machine share default**: 0 until the machine names its first need, or small from genesis?

## 8. acceptance

1. the four rules and the curve stated as closed forms over committed counters, reviewed.
2. conformance on the chaosnet: a book created via lock, held across epochs, deleted → capital unlocked, net burn zero; net growth burns correctly at each ladder step; `supply(h) = C(1−q^h)` verified against the emission schedule in `audit/`.
3. `foculus/specs/parameters.md` gains the six numbers; [[steps-blocks-epochs]] §3e marked specified with this page as its normative reference.
