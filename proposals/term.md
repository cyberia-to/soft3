---
tags: cyber, soft3, proposal, trident, tok, tade, foculus, oikos, trading, contracts, zheng
crystal-type: process
crystal-domain: cyber
status: proposed
date: 2026-10-13
alias: term, covenant, contract language, trading substrate, term dsl
---
# term — programmable rights, and the trading substrate

> three inheritances, one language. **sword** (sword-smith's declarative derivative language, descended from peyton-jones *composing contracts*) answers *what is this worth, when, conditioned on what*. **parano1d's** contract core answers *who may cause the next transition, under what limits*. **[[oikos]]** answers *where does it settle, and against what evidence*. on soft3's base the third question classical trading could never answer joins them: *prove it*. the result is a contract language whose collateralization is a theorem rather than a runtime check, and — because trading is the hardest customer of contract semantics — the design of the best trading substrate we know how to build. this page fixes the language semantics and the substrate; it spec-ifies the two open settlement protocols of [[oikos]] rather than leaving them as prose.

## 1. what each inheritance contributes

| inheritance | contributes | leaves behind |
|---|---|---|
| **sword** | valuation trees: `zero / transfer / scale(escrow, expr) / both / translate(delay) / if-within`; fully collateralized by construction; party tokens as tradable positions | EVM gas, trusted `obs` addresses, ERC1155 plumbing |
| **parano1d core** | policy + predicated straight-line program + carried counters; commitment-as-identity; pull-only execution; receipts | the 16-instruction ceiling, two-counter state, six hardcoded templates |
| **[[oikos]]** | one token one chain: home books, conservation laws, conditional settlement at home, the no-free-option and freshness protocols | prose-level protocols — this page makes them executable |
| **[[tok]]** | Coin (fungible), Card (non-fungible), PLUMB: pay · lock · update · mint · burn | — |
| **[[tade]]** | one exchange format from wire to screen — orders are frames | — |
| **zheng/nox** | bounded programs with cost reports; one universal fast-path relation; ARC fold validity | the accumulator, still scheduled ([[proof-system-repair]]) |

the thesis: **sword supplies the algebra of value, parano1d the algebra of rights, oikos the algebra of settlement, zheng the algebra of proof.** none suffices in pairs; the quadruple is the point.

## 2. the language — term

a term is the public description of a live right. readable grammar, seven flow constructs:

```
flow  f ::= give(amount, card)                 -- pay value to a card (PLUMB pay)
          | lock(amount, card)                 -- escrow under this term (PLUMB lock)
          | scale(escrow, e, f)                -- sword: lock `escrow` up front, multiply the
                                               --   obligation by e — collateralization by construction
          | both(f1, f2)                       -- compose: parallel rights, split collateral
          | after(height, f)                   -- sword translate: the right acquires at a height
          | if e within height then f1 else f2 -- sword time-discrimination
          | settle_as(amount, card)            -- the residual: sword's hidden party-token 0,
                                               --   as a named remainder — nothing can be stranded

expr  e ::= const | obs(condition) | e1 op e2

condition ::= { event, book, evidence_tier ≥ τ, checkpoint, max_age }
          -- an oikos freshness condition, NOT an oracle address:
          -- the obs word of the language is an evidence object

terms   t ::= authority · ceilings · deadline   -- the policy: who may call, under what limits (parano1d CNTPOL)
state   s ::= counters carried output to output  -- the fold: balances, claim counts, due heights
```

design rules, each inherited and each load-bearing:

1. **straight-line predicated compilation.** the compiler emits no `branch` — only predicated writes (parano1d's discipline) — so the cost report holds for *every* input. fee predictability is a consensus property here, and timing side channels are excluded by construction.
2. **failure = reject, one mode.** checked arithmetic, false assertions, collateral shortfall: the whole call is unprovable. collateralization is enforced by the relation, not by runtime checks — undercollateralized states have no witness.
3. **pull-only.** no timers, no schedulers. `after(h, f)` is a height predicate; a due acquisition is a submitted call, exactly like parano1d's recurring payments.
4. **obs is an evidence object.** every external value enters as an [[oikos]] condition — event, source book, tier, checkpoint, age — attested by the stake quorum that already finalizes epochs. oracle trust becomes economic and slashable instead of an address someone chose to believe. (mitigated, not eliminated — the oracle problem is named, not hidden.)
5. **commitment-as-identity.** three domain-separated commitments — terms (policy), flow (program), state (counters) — composed into one address; changing a counter changes the address; no in-place upgrades.
6. **composition is cryptographic.** `both(f1, f2)` is a commitment tree, so proofs compose and capabilities compose — the PLUMB/Gold-Standard observation from the neptune thread made literal: party positions are Cards, and a matched trade is a `both` whose two sides are satisfied simultaneously or not at all.

## 3. compilation targets

- **surface → three commitments.** terms/flow/state commit separately (parano1d's `CNTCODE_`/`CNTPOL__`/`CNTOBJ__`, generalized by trident's `Digest`).
- **flow → the predicated subset of tri**, compiled with `trident build --costs`; the cost report is the fee schedule input.
- **fast path: one universal term-step CCS** in zheng — flow and terms as public inputs, so every standard term verifies in the same frozen relation (parano1d's uniformity, our generality); exotic terms pay the general per-program path. two tiers, one algebra, seven constructs.
- **validity in the fold** per [[steps-blocks-epochs]]: steps carry calls, the ARC accumulator carries history, epochs decider it.

## 4. the trading substrate — provable clearing

trading is the hardest customer of contract semantics. the substrate, with the optimizations that make it tractable.

### 4.1 intents are terms

an order is a live right: `lock(10 A, me)` composed with `if obs(price) within h then give(x B, taker) else settle_as(remainder, me)`. intents land continuously on 1 s steps as [[tade]] frames; they live as carried outputs until filled, expired, or cancelled — the counter-carry pattern, across rounds and epochs. partial fills are successor outputs with reduced quantity; cancel is a closing call; GTC is an expiry-height term.

### 4.2 rounds — the clearing cadence

a **round** closes every r steps (parameter; r = 1 is legal but the clearing class will set r from measured circuit size, §4.5). the round closes with the order set frozen under a Merkle root committed in that step — this is [[oikos]]'s protocol 1 made executable: *no free option*, because both legs of every match bind to the same beacon and explicit deadline; there is no sequential commitment, so the second committer gets no option. epochs remain the finality/decider cadence; rounds are the trading cadence inside them.

### 4.3 the clearing rule and its certificate — formally

rule: uniform-price double auction per (book pair, round). given the committed multiset of revealed orders — bids (limit, qty) and asks (limit, qty) — a clearing is `(p, M)`: a single price and a set of matches, all executed at p.

the quality certificate is three predicates over `(p, M)`:

1. **conservation** — per pair: matched quantities balance (Σbuy = Σsell = Q) and values balance at p (Σbuy value = Σsell value = p·Q); every home book reconciles ([[oikos]]'s complete-books foundation);
2. **uniform price** — every match in M executes at p;
3. **no blocking pair** — every unmatched bid has limit < p; every unmatched ask has limit > p.

**lemma (maximal volume).** C + U + N ⟹ Q is the maximum volume tradeable at any single uniform price. sketch: with orders sorted (bids desc, asks asc), the maximal-volume index k = max{i : b_i ≥ a_i} is unique; any uniform-price matching of volume > k must match a pair (b_i, a_j) with i > k or j > k, i.e. an unmatched bid with limit ≥ p facing an unmatched ask with limit ≤ p — contradicting N. ∎

**corollary (certificate uniqueness).** with a deterministic tie-break (price rule fixed in the pair's terms, e.g. p in the feasible interval [max(b_{k+1}, a_k), min(b_k, a_{k+1})] by least-integer; ties in qty pro-rata by commitment-hash order), C + U + N determine (p, M) uniquely. **any two valid certificates for the same committed batch clear identical volume at an identical price.**

the design move, stated precisely: the relation never *computes* an optimum (NP-hard, off the table); it *verifies* a witnessed certificate (polynomial, straight-line). verify-don't-compute — the same pattern as the rest of the proof stack.

### 4.4 circuit engineering — the certificate is branch-free

the solver supplies the sorted order sequence, the permutation (Merkle openings of each sorted item into the round's committed root), k, p, and the match vector. the relation checks:

- **permutation** — every sorted item opens into the frozen root (batched Merkle verification; the FROST-style batching applies),
- **order** — adjacent sorted-sequence comparisons (bids non-increasing, asks non-decreasing),
- **predicates C, U, N** — aggregate sums and the boundary checks against k and p.

all of it data-independent straight-line arithmetic: comparisons, sums, hash openings. no branches, no data-dependent control flow — the predicated subset of §2 swallows the entire certificate, which means uniform cost per round regardless of order content. orders themselves are prunable after the round; receipts carry the root + proof, portable like everything else in the stack.

### 4.5 the clearing class

at n ≈ 10³ orders per pair the certificate is ~10⁵–10⁶ reductions (dominated by the batched openings and the sorting-network-free adjacent checks) — too big for the ordinary step budget, so clearing is its own **block class** in the §3c sense: a reduction-budgeted relation admitted by measured cost, verified by every node, paid for by the round fee. this is the reduction-budget machinery's first real customer.

### 4.6 solver economics — the open question, closed

the §5.5 question "the shapley marginal of what" dissolves: by the uniqueness corollary, no matching-*quality* marginal exists — every valid certificate is economically identical. the solver's only axis is **speed**. therefore:

- **first valid certificate wins** a deterministic fee fixed in the pair's terms (per-round, bps of cleared volume); verification is cheap, so any node confirms the winner, and solvers race on proving latency alone — the same race the grind runs at block level ([[steps-blocks-epochs]] §5),
- the proving work itself still earns settlement tickets as ordinary computation — [[fold-mining]] unchanged,
- no new tokenomics, no shapley attribution for solvers. shapley tickets remain exactly where a real marginal exists: ranking computation.

### 4.7 surplus, netting, and the residual attack surface

- **surplus**: the spread between limits and p is real value. policy per pair, in the terms: `surplus: makers | burn | solver` — default `makers`, rebated pro-rata to matched size (liquidity incentive); the choice is committed, so it is part of the certificate's statement.
- **netting**: cross-book legs accumulate as [[oikos]] conditions; when both directions of a pair have open obligations, a netting round cancels opposites before settlement — counter-carry of netted amounts, roughly halving evidence conditions in two-way markets.
- **cancel racing**: none — the batch order set is frozen at close; cancels apply from the next round. deterministic, no race to exploit.
- **reveal griefing**: non-reveal burns the order bond; a round with fewer than two revealed orders is void (zero fee).
- **invalid certificates**: cost the solver proving work and pay nothing; nodes verify cheaply, so spam is self-taxing.
- **cross-round price movement**: informational — the part no design can remove, and the part that is legitimately tradable.

**privacy** stays commit-reveal (§5.2): an order is a bonded commitment during the window; the batch structure does the work; [[proof-system-repair]]'s VEIL profile is available for amount privacy where a pair wants it.

## 5. what the trading lens exposes — the forgotten checklist

the contract-only view of the previous proposals missed these; each becomes a child spec or a module:

1. **intent mempool admission** — term well-formedness, bond, fee floor (extends §3a of [[steps-blocks-epochs]] with an intent flavor).
2. **commit-reveal window rules** — window lengths, bond sizes, replay protection, per-epoch round parameter (one batch per epoch, or sub-epoch rounds — a parameter, not a fork).
3. **the clearing certificate format** — the three predicates as a zheng relation column; who proves them (any node — verification is cheap; solvers compete on speed, not on trust).
4. **order lifecycle across epochs** — partial fills as successor outputs; GTC = expiry-height term; cancels = closing calls.
5. **solver fee rule** — closed by the uniqueness corollary (§4.6): first-valid certificate, deterministic fee fixed in the pair's terms; settlement tickets unchanged for real marginals (ranking).
6. **standard generators** — the six parano1d templates (refund, vault, allowance, budget, recurring, vesting) plus the trading set: limit/IOC orders, funding (recurring on a price condition), vesting unlocks, collateralized derivatives via `scale(escrow, obs(condition), …)`.
7. **portfolio evidence tiers** — every observation a wallet shows carries its τ ([[oikos]] integration row for sigma); trading UIs must display tier, not just balance.

## 6. relation to the other proposals

- **[[oikos]]**: its two settlement protocols (no free option, freshness) are spec-ified here as the clearing batch and the condition object; its integration table gains this page as the typed trade-operation owner.
- **[[steps-blocks-epochs]]**: §5's two lotteries carry the substrate — grind orders blocks, tickets price the marginals (including the solver's), the fold stitches.
- **[[goldilocks-parano1d]]**: P1's chain machine delivers the steps these intents ride; this page is the demand side that makes P1's mempool spec concrete.
- **[[proof-system-repair]]**: the fast path assumes the universal CCS + accumulator land (P0); until then everything here is specified against the public profile.

## 7. acceptance

1. the grammar of §2 parses the six parano1d templates and the four trading generators of §5.6, each with a cost report from `trident build --costs`.
2. the maximal-volume lemma and the uniqueness corollary of §4.3 stated and reviewed; a conformance clearing of n = 10³ orders per pair, certificate proven in zheng, settled against two home books on the chaosnet, circuit size and verify time measured and published to `audit/` with command and revision named (the first customer of the §3c clearing class).
3. `foculus/specs/parameters.md` gains the window/bond/round parameters; [[oikos]]'s two protocols marked specified, with this page as their normative reference.
