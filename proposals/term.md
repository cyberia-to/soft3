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

## 4. the trading substrate

trading is the hardest customer of contract semantics, and designing for it exposes what a contracts-only view misses. the substrate, top to bottom:

**intents are terms.** an order is a live right: `lock(10 A, me)` composed with `if obs(price) within h then give(x B, taker) else settle_as(remainder, me)`. intents land continuously on 1 s steps as [[tade]] frames; they live as carried outputs until filled, expired, or cancelled — the counter-carry pattern, across epochs.

**epochs are clearing batches.** the epoch boundary (the [[foculus]] beacon) closes a batch: commit window → reveal → clear → settle at home. this is [[oikos]]'s protocol 1 made executable — *no free option*: both legs of every match bind to the same beacon and an explicit deadline; the second committer gets no option because there is no sequential commitment.

**the clearing certificate — the design move that makes provable trading tractable.** do not prove optimality of matching (NP-hard, and a proof of optimality is its own research problem). choose the clearing rule so that its *quality certificate is a cheap predicate*. a uniform-price double auction clearing is fully certified by three efficiently checkable properties:

1. **conservation** — every leg sums; home books reconcile ([[oikos]]'s complete-books foundation);
2. **uniform price** — one clearing price per batch per pair;
3. **no blocking pair** — no unmatched buy ≥ p facing an unmatched sell ≤ p.

conservation + uniform price + no blocking pair implies maximum traded volume at p, and all three check in O(n log n) *inside the step relation*. fairness that other chains enforce with trusted sequencers becomes here a theorem with a short proof. MEV analysis: intra-batch ordering is valueless (uniform price, committed order set), cross-batch price movement is information, not extraction — what remains is what cannot be removed by any design.

**order privacy by commit-reveal.** during the window an order exists as a commitment (bonded); reveal phase opens it; non-reveal burns the bond. no threshold crypto, no encrypted mempool committee — the batch structure does the work.

**the solver is paid by shapley.** matching is a marginal contribution to the batch, and soft3 already prices marginals: settlement-mining tickets ([[steps-blocks-epochs]] §5, [[fold-mining]]) pay exactly this shape of computation. solver compensation is wired to the existing ticket machinery, not a new tokenomics. *open economic question, named for the child spec: the marginal of what — value cleared? surplus? — is a parameter choice, not an afterthought.*

**settlement at home.** clearing produces conditional reassignments; each home book settles locally under its own conservation law, with the counterparty's leg carried as an [[oikos]] freshness condition (tier τ, checkpoint, max age) until evidence or expiry. halting counterparties decay gracefully: freshness expires, obligations resolve under the named policy.

## 5. what the trading lens exposes — the forgotten checklist

the contract-only view of the previous proposals missed these; each becomes a child spec or a module:

1. **intent mempool admission** — term well-formedness, bond, fee floor (extends §3a of [[steps-blocks-epochs]] with an intent flavor).
2. **commit-reveal window rules** — window lengths, bond sizes, replay protection, per-epoch round parameter (one batch per epoch, or sub-epoch rounds — a parameter, not a fork).
3. **the clearing certificate format** — the three predicates as a zheng relation column; who proves them (any node — verification is cheap; solvers compete on speed, not on trust).
4. **order lifecycle across epochs** — partial fills as successor outputs; GTC = expiry-height term; cancels = closing calls.
5. **solver ticket wiring** — Shapley marginal of matching, per §4; the economic parameter named there.
6. **standard generators** — the six parano1d templates (refund, vault, allowance, budget, recurring, vesting) plus the trading set: limit/IOC orders, funding (recurring on a price condition), vesting unlocks, collateralized derivatives via `scale(escrow, obs(condition), …)`.
7. **portfolio evidence tiers** — every observation a wallet shows carries its τ ([[oikos]] integration row for sigma); trading UIs must display tier, not just balance.

## 6. relation to the other proposals

- **[[oikos]]**: its two settlement protocols (no free option, freshness) are spec-ified here as the clearing batch and the condition object; its integration table gains this page as the typed trade-operation owner.
- **[[steps-blocks-epochs]]**: §5's two lotteries carry the substrate — grind orders blocks, tickets price the marginals (including the solver's), the fold stitches.
- **[[goldilocks-parano1d]]**: P1's chain machine delivers the steps these intents ride; this page is the demand side that makes P1's mempool spec concrete.
- **[[proof-system-repair]]**: the fast path assumes the universal CCS + accumulator land (P0); until then everything here is specified against the public profile.

## 7. acceptance

1. the grammar of §2 parses the six parano1d templates and the four trading generators of §5.6, each with a cost report from `trident build --costs`.
2. a conformance clearing: a batch of N intents cleared with the three-predicate certificate, proven in zheng, settled against two home books on the chaosnet, receipt in `audit/` with command and revision named.
3. `foculus/specs/parameters.md` gains the window/bond/round parameters; [[oikos]]'s two protocols marked specified, with this page as their normative reference.
