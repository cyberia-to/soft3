---
tags: cyber, soft3, proposal, chain, cybernode, cybergraph, neural, signal, intent, cyberlink, box, pussy
crystal-type: process
crystal-domain: cyber
alias: one message, one kind of message, signals only, the chain speaks signals
status: proposed
date: 2026-11-02
revised: 2026-11-03 — v3 after the re-cut (tasks/one-message.md, diagnosis of the cut): one field set, every action through existing clauses, the removals, six gaps, the launch row
---
# one message — the root chain carries signals

the owner, 2026-11-02 (translated): "in my design of soft3 there is only one kind of message — links — everything else is the semantics of neural"; "there are no transactions, there are signals".

what a [[signal]] is, is defined by [[soft3/proposals/signal-is-an-intent|signal is an intent]] (`proposals/signal-is-an-intent.md`, branch `prop/signal-intent`, written in parallel and not yet on origin): one [[neuron]] declares an [[intent]] with provable conditions of completion; any neurons fulfil it, possibly across blocks; a small signal fits one block. this page gives the chain one field set, expresses every action the chain needs through clauses that already exist, lists what leaves the chain specifications, names the six places where no clause answers, and gives the launch row. sources at origin heads: cybergraph `0a70b6a`, neural `47b086c`, soft3 main `3045d54` and `release/phase1` `e340d47`, bbg `2294281`, foculus `cdff8dc`, mudra `3969c7a`, tru `e660d1e`, cyber `68f1260a`, cybernode main `52538a6` and `release/phase1` `599cd85`. a clause is `repo:file:line`; a bare file name is cybergraph `specs/`.

## 1. one field set

four field sets exist: `signal.md:7–17` `(ν, net, ℓ⃗, Δφ*, σ, h₀, h₁)`; `neural:specs/wire.md:19–25` `{neuron, links, time, proof}`; the envelope of `cli.md:21–23` `(neuron, links, prev, step, network)` with `box_moves` (`cli.md:44`); `soft3:schema/README.md:24–37` `ν ‖ t ‖ n ‖ links`. the code has two shapes: bbg `cli/src/frame.rs:17–18` (ZAP signal `neuron ‖ height ‖ links ‖ box_moves`; KET intent `neuron ‖ h0 ‖ scope_hash ‖ signature`) and foculus `src/chain.rs:45–60` `Signal { neuron, network, links, delta_pi, box_moves, prev, step, height, proof }`. field by field:

| field | carried by | chosen by |
|---|---|---|
| `ν` neuron | all six | `signal.md:11` |
| `net` destination | `signal.md:12`; `cli.md:23`; foculus `chain.rs:49` | `signal.md:27–31`: one signal, one network, bound into the signal hash; `validate.md:26`, `:43` |
| `step`, `prev` | `cli.md:23`, `:44`; foculus `chain.rs:55–56` | `order.md:16–17`: the chain link and no equivocation; the record key `signal_key(ν, step)` (`bbg:specs/state.md:75`) |
| `h₀` inception | `signal.md:16`; KET `h0` | `intent.md:12`; `signal.md:19–21` |
| `ℓ⃗` links, each `(p, q, τ, a, v)` | all six | `cyberlink.md:13`: "five stored fields … who made it and when belong to the signal"; the schema's link record (`schema/README.md:30–35`) is this tuple; foculus's per-link `neuron` and `height` (`chain.rs:21–29`) and neural's `Link` without a token (`wire.md:25`, against `neural:specs/relation.md:35`) fold into it |
| `box_moves`, each `{nullifier, commitment}` | `cli.md:44`; ZAP; foculus `chain.rs:53` | `box.md:37–38`, `:88`; `validate.md:20–21`; `signal.md:15`: σ covers "all conviction box movements" |
| `Δφ*` impulse | `signal.md:14`; foculus `delta_pi` | `signal.md:14`, `:48`; `validate.md:24` |
| `π_id` identity proof | KET `signature`; `intent.md:14` | `signal.md:21`: before sealing a signal is "declared, identity-proven"; NSIG1 on pussy (`soft3:proposals/proven-present.md:80`; `mudra:specs/neuron-auth.md:11–17`) |
| `σ` proof | `signal.md:15`; neural `proof`; foculus `proof`, optional | `signal.md:33–41`; absent in release 0 (`cli.md:44`) and until the block proof is a validity condition (`proven-present.md:75`) |
| `h₁` sealing | `signal.md:17`; foculus `height`, "0 until finalized" | written by the block that records the signal, `SignalRecord.block_height` (`bbg:specs/state.md:75`) |

an intent is the KET frame `(ν, h₀, scope_hash, π_id)` (`intent.md:7–14`; `frame.rs:18`; `cybergraph:src/api.rs:64–71`). the schema's unix `t` and neural's `time` give way to the heights (`signal.md:16–17`; `order.md:15`).

what the choice amends:

- `signal.md` §structure gains `step`, `prev`, `box_moves` and `π_id`, by the clauses of the third column;
- `soft3:schema/README.md:12–38` takes this set in place of its header (which cites `cybergraph/specs/graph`, gone), the SDKs with it (`soft3:AGENTS.md`, do-not-touch zones); neural `wire.md:42` already defers to the schema;
- what π_id signs: `intent.md:77` signs `ν ‖ h₀ ‖ hash(Σ)` and `validate.md:18` `H(ℓ⃗ ‖ Δφ* ‖ t)`, while `mudra:specs/identity.md:36–51` binds "network / genesis", the "canonical action" and the "nonce / operation identity": "a valid proof for one action cannot authorize another action, network, root". both lines are amended to bind `net`, and for a signal `step` and `box_moves`.

## 2. every action through existing clauses

| action | expressed as | clauses | pussy today |
|---|---|---|---|
| pay | a link from the payer's [[card]] to the payee's with the box `(τ, a)`; the payer proves it owns `from` (G2) | `soft3:specs/terms.md:74`, `:96`; `link.md:15`, `:20`; `box.md:23` | kind 1, `cybernode:specs/chain.md:340–350` |
| stake | a link `(p, q, τ_native, a > 0, v)`; the box goes from the author's wallet to the position at `stake_key(card)` (G2, G3) | `box.md:35`; `staking.md:13`; `bbg:specs/state.md:79–105`; `cybernode:specs/chain-economy.md:36–37` | kind 2 with `amount > 0`, `chain-economy.md:266–285` |
| unlock | a box withdraw: the box returns to the author's wallet, the record stays; only the author, after `MIN_LOCK` (G2) | `box.md:37`; `cyberlink.md:54`; `bbg:specs/state.md:85`; `chain-economy.md:295–306` | kind 6, `chain-economy.md:287–310` |
| payout owner | a card transfer `(owner, new owner, card, 1, 0)`: beneficial ownership moves, the record stays (G3) | `cyberlink.md:34`, `:79`; `box.md:19`, `:36`; `bbg:specs/state.md:83` | kind 7, `chain-economy.md:312–324` |
| legacy claim | no chain message: a holder whose key is known signs with the same key; the ADR-036 claim is built "with no on-chain transaction"; a holder who wants a new neuron pays to it. the 8,390 address-only balances parked under `legacy_owner` move by kind 3 today; under signals the written route is a program the network declares from a rule set ("in time it moves into contracts") | `mudra:specs/bridge.md:64–66`; `chain.md:406–414`, `:416–434`; `proven-present.md:80`; `soft3:proposals/network-repos.md:109` | kind 3, `chain.md:424–434` |
| authorization | π_id over the statement of `mudra:specs/identity.md:36–51` (§1); NSIG1 on pussy while legacy balances remain, proved inside σ once the block proof is a validity condition | `proven-present.md:76`, `:80`; `validate.md:18` | NSIG1 over `chain ‖ tx` (`chain.md:313–320`); the P5 draft, AUTH1 (cybernode#11) |
| fee | `cost(ℓ) = a(ℓ) + base_fee`, paid out of the box inputs; one base fee per primitive under the EIP-1559 rule; its value is network data; on pussy it goes to the miner | `validate.md:22–23`; `neuron.md:25`; `cyber:whitepaper.md:1198`; `network-repos.md:108`; `chain-economy.md:700–701` | per byte, `chain.md:384–390` |
| the 1 % tax | a rule of "every transfer … that moves CYB", half burned, half to the reward budget; pussy moves PUSSY, so the tax applies to the cyber network and its CYB, and pussy keeps "nothing is burned" | `cyber:cyber/$CYB.md:423`, `:431–448`, `:724`, `:728`; `proven-present.md:80`; `network-repos.md:104`; `chain-economy.md:700–701` | none |
| address announcement | FOLLOW, ANTENNA, SOCKET, LOCUS: four ordinary cyberlinks from the neuron's name, typed as [[neural]] types any link, on its [[axon]] (G13) | `address.md:7–16`; `neural:specs/relation.md:12–23` | kind 2 with a word as `p`, `cybernode:specs/chain-wire.md:187–198` |
| coinbase, subsidy | a rule of the block, its record first; the subsidy pays a proof meeting a target | `proven-present.md:95`; `tru:specs/rewards.md:207–210`; `chain.md:256`, `:322–338` | kind 0 |
| settlement, the pool's cut | state transitions of the block, "not a transaction" | `chain-economy.md:334–335`, `:671–674` | the same |
| revert (P9) | a statement signed by the checkpoint key, which the owner holds, applied as a block-validity rule beside the checkpoint (G8) | the owner, 2026-11-02, `soft3:release/director/plan.md:170` on `release/phase1`; cybernode#12 `specs/chain.md:754`, `:789–797` at `9b0705a`; cybernode `release/phase1` `599cd85` (#15); `chain.md:724–748` | absent from cybernode main |
| the pool (rule set 2) | deposit and spend as box create and spend in `box_moves`, under σ. the pool's link `(p, q, weight, payout)` against the 5-tuple is launch row 10, open ("the inverse of P1"); pussy's genesis freezes it in format 1; the transparent side of rule set 1 "may only narrow" | `box.md:38`, `:88`; `validate.md:20–22`; `cyber:launch.md:110`, `:180`; `soft3:proposals/privacy-frozen.md:38`, `:49`, `:59`; `chain.md:1196`, `:1219–1221`, `:1346–1352` | kinds 4 and 5, invalid under rule set 1, `chain.md:1255–1290` |

meaning stays neural's: the chain checks conservation, ownership of `from` and the cost per link (`validate.md:15–26`); what a link means is read by the [[dialect]] that owns its [[word]] (`neural:specs/relation.md:41–45`).

## 3. what leaves the chain specifications

at cybernode main `52538a6`, each against the clause it contradicts or duplicates:

1. `chain.md` §2, the typed kinds, against one action and one carrier (`terms.md:74`; `neural:specs/sigils.md:31–33`): the envelope `version ‖ kind ‖ body ‖ sig` and its table (`291–311`), txid and signing statement (`313–320`), transfer (`340–350`), cyberlink with "why two neuron-signed kinds" (`352–373`), claim (`424–434`); and what counts or dispatches on kinds — `MAX_BLOCK_TXS`, `MAX_TX_BYTES` (`72–73`), fixed size per kind (`259–261`), the root over kinds (`269–287`, becoming the root over the block's signals), the apply loop (`506–508`), validation step 7 (`772–773`), further kinds (`822–823`), `tx.rs` (`953`).
2. `chain.md:465`, the signals dimension written by nobody, against `network.md:53–55` and `order.md:20`, which record every accepted signal there. the nonce stays: bbg keeps it in `NeuronRecord` under the root (`bbg:specs/state.md:31`, `:73`), and moving it would change block 0's root (`chain.md:685`).
3. `chain.md:384–390` and `MIN_FEE_PER_BYTE` (`74`), against the cost per link (`validate.md:23`; `neuron.md:25`).
4. `chain-economy.md`, kinds duplicating the box lifecycle (`box.md:36–37`) and the one action (`terms.md:74`): lock as kind 2 (`266–285`), unlock's format (`289–293`), payout's format (`314–316`), kinds 8–10 reserved (`326–328`), the summary row (`86`). their rules — native token only, valence fixed while staked, author-only unlock after `MIN_LOCK` — stay as rules of the moves of §2.
5. `chain-economy.md:784–794`, kind 10 `report`, against `valence.md:27`: the valence is the BTS meta-prediction, with no separate submission.
6. `chain-wire.md:187–198`, an address as `p = hemera("antenna" | "socket" | "follow")`, against `address.md:9–14` (the neuron's name at `from`) and `neural:specs/relation.md:27–31` (the route through a word makes a super-node); and the kinds of item 1 on the wire: the `txs` topic (`42`), get-txs (`74–75`), the mempool (`125–137`), the `/tx` routes (`221`, `233`, `253–266`).

on soft3 main the node specs moved to cybernode are still listed (`soft3:specs/README.md:27–30`), against "moved means moved" (`soft3:proposals/cybernode.md:65`).

## 4. gaps — where no clause answers

- G2 · which landing a link takes. bbg's `insert` moves a box `H(from ‖ τ) → H(to ‖ τ)`, a lock moves it to `stake_key(card)` (`bbg:specs/state.md:105`, `:96`); `terms.md:74` reads a payment, `box.md:35–37` and `staking.md:13` a position, `cyberlink.md:32–35` one shape both ways; the chain today sends every cyberlink to `insert` with `amount = 0` and keeps stake itself (`chain-economy.md:255–259`). nothing inside one signal tells pay, lock, withdraw and card transfer apart. amend `box.md` §moving a box.
- G3 · one card per record or per axon, and who holds it. `cyberlink.md:77` makes every record a distinct card, told apart by its signal's ν and t; `bbg:specs/state.md:79–96` makes one card per author and axon, `card_id(author, p, q)`, with `author` (who may unlock) apart from `owner` (who is paid). amend one of the two.
- G7 · intents on the chain. intents stay on record at inception height (`intent.md:34–36`, `:94–96`); bbg keeps them outside its root (`bbg:specs/native-state.md:8–9`, `:66`) with a 64-byte signature (`:59`) where NSIG1 is 102 bytes (`mudra:specs/neuron-auth.md:11`); a seal names its intent only as an argument (`cli.md:41`; `cybergraph:src/api.rs:190`), and a sub-signal names the intent it fulfils nowhere (`cyber:cascade.md:10`, `:21–24`). no clause puts the intent in the block or its key in a frame. amend `bbg:specs/state.md` and `signal.md` §structure.
- G8 · keyed statements beside the signals. the checkpoint (`chain.md:724–748`) and the owner's revert (cybernode#12 `specs/chain.md:754`) are statements of one key, outside any neuron's chain of signals; no clause of cybergraph, neural or bbg names a consensus input other than the signal. amend `chain.md` §5 to say so, or carry both as signals of `neuron_of(checkpoint_key)`.
- G11 · the root chain's network card. a network is a card whose record is `CardRecord { owner, particle }` (`network.md:14–23`); bbg's cards dimension holds positions `[owner, particle, author, valence, since]` (`bbg:specs/state.md:34`, `:79`); the root chain names itself by its genesis particle (`chain.md:118`; `network-repos.md:106`), which no card record holds, so `net` has no value for it. amend `network.md` §a network is a card with `chain.md` §4.4.
- G13 · telling the four address links apart. a LOCUS is read from its particle's content (`address.md:20–23`) and the seed has `follows` (`neural:specs/relation.md:52`); no word or content tag marks ANTENNA and SOCKET, and relation words are "not decreed" (`relation.md:55`). amend `address.md`.

## 5. pussy's launch

pussy launches on today's chain by the owner's own words: "go on the air as soon as possible with the holes known and named in public" (`cyber:launch.md:68`), and what is not in rule set 1 follows "by scheduled fork" (`cyber:launch.md:99`). the freeze of 2026-11-02 is withdrawn by the owner (D19, `soft3:release/director/plan.md:23`, `:86` on `release/phase1`; the launch page's edit is [cyber#155](https://github.com/cyberia-to/cyber/pull/155), open, so `launch.md:85` still reads the freeze). the signal rule set is that scheduled fork, announced at least 40,320 blocks ahead (`chain.md:802–804`). the row for the holes table of `cyber:launch.md:70–80`:

| hole | measured consequence | fix lane | where |
|---|---|---|---|
| the chain speaks typed kinds, not signals: rule set 1 carries kinds 0–3, 6 and 7 (`cybernode/specs/chain.md` §2, `chain-economy.md` §3) where soft3 specifies one message, the signal | not measured; every block until the fork carries kinds, the staking CLI signs kinds (`cyber:launch.md:79`), and the history before the fork stays in kinds | scheduled fork: the signal rule set of this page, announced on cyber/launch ≥ 40,320 blocks before its height, once G2, G3, G7, G8, G11 and G13 have their clauses | [soft3#677](https://github.com/cyberia-to/soft3/pull/677) |
