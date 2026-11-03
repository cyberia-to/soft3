---
tags: cyber, soft3, proposal, chain, cybernode, cybergraph, neural, signal, intent, cyberlink, box, pussy
crystal-type: process
crystal-domain: cyber
alias: one message, one kind of message, signals only, the chain speaks signals
status: proposed
date: 2026-11-02
revised: 2026-11-03 — v3 after the re-cut; round 2 of unit 2 disposes review U2-1…U2-17 (tasks/one-message.md)
---
# one message — the root chain carries signals

the owner, 2026-11-02 (translated): "in my design of soft3 there is only one kind of message — links — everything else is the semantics of neural"; "there are no transactions, there are signals".

what a [[signal]] is, is defined by [[soft3/proposals/signal-is-an-intent|signal is an intent]] ([soft3#678](https://github.com/cyberia-to/soft3/pull/678)): one [[neuron]] declares an [[intent]] with provable conditions of completion; any neurons fulfil it, possibly across blocks, each authoring its own links in its own sub-signal (`intent.md:38–40`; `cyber:cascade.md:23–24`); a small signal fits one block. this page gives the chain one field set, every action through existing clauses, the removals, the gaps and the launch row. a clause is `repo:file:line` at origin head (the heads are in the task page); a bare file name is cybergraph `specs/`.

## 1. one field set

four field sets: `signal.md:7–17`; `neural:specs/wire.md:19–25`; the envelope of `cli.md:21–23` with `box_moves` (`:44`); `soft3:schema/README.md:24–37`. three code shapes: bbg `cli/src/frame.rs:17–18` (ZAP signal, KET intent); the foculus struct `src/chain.rs:45–60`; the foculus wire, which `cli.md:23` names canonical, `src/frames.rs:132–170`, with per-link `neuron` and `height` and without the network and the proof (`frames.rs:56–58`). field by field:

| field | carried by | chosen by |
|---|---|---|
| `ν` neuron | all seven | `signal.md:11`; absent from a pool spend (§2) |
| `net` destination | `signal.md:12`; `cli.md:23`; the foculus struct | `signal.md:27–31`: one network per signal, bound into the hash; its value for a chain is `H(canonical genesis bytes)` (`cyber:specs/node-product.md:101`), the chain id (`cybernode:specs/chain.md:118`) |
| `step`, `prev` | `cli.md:23`, `:44`; both foculus shapes | `order.md:16–17`; the record key `signal_key(ν, step)` (`bbg:specs/state.md:75`) |
| `h₀` inception | `signal.md:16`; KET `h0` | `intent.md:12`; `signal.md:19–21` |
| `ℓ⃗` links, each `(p, q, τ, a, v)` | all seven | `cyberlink.md:13`: "who made it and when belong to the signal"; foculus's per-link `neuron`, `height` and neural's tokenless `Link` (`wire.md:25`; `neural:specs/relation.md:35`) fold into it |
| `box_moves`, each `{nullifier, commitment}` | `cli.md:44`; ZAP; both foculus shapes | `box.md:37–38`, `:88`; `validate.md:20–21`; `signal.md:15` |
| `Δφ*` impulse | `signal.md:14`; both foculus shapes | `signal.md:14`, `:48`; `validate.md:24` |
| `π_id` identity proof | KET `signature`; `intent.md:14` | `signal.md:21`: before sealing a signal is "declared, identity-proven"; NSIG1 on pussy (`soft3:proposals/proven-present.md:80`; `mudra:specs/neuron-auth.md:11–17`) |
| `σ` proof | `signal.md:15`; neural `proof`; the foculus struct | `signal.md:33–41`; absent in release 0 (`cli.md:44`) and until the block proof is a validity condition (`proven-present.md:75`) |
| `h₁` sealing | `signal.md:17`; foculus `height`, "0 until finalized" | written by the block that records the signal, `SignalRecord.block_height` (`bbg:specs/state.md:75`) |

an intent is the KET frame `(ν, h₀, scope_hash, π_id)` (`intent.md:7–14`; `frame.rs:18`; `cybergraph:src/api.rs:64–71`). the schema's unix `t` and neural's `time` give way to the two heights (`signal.md:16–17`).

what the choice amends:

- `signal.md` §structure gains `step`, `prev`, `box_moves` and `π_id`; `soft3:schema/README.md:12–38` takes this set, the SDKs with it (`soft3:AGENTS.md`); neural `wire.md:42` already defers to the schema. the foculus wire may keep omitting `net`, as a node serves one network (`network.md:47–50`), once π_id binds it (`signal.md:31`; `chain.rs:63` does so for the hash).
- what π_id signs: `intent.md:77` signs `ν ‖ h₀ ‖ hash(Σ)`, `validate.md:18` `H(ℓ⃗ ‖ Δφ* ‖ t)`; `mudra:specs/identity.md:36–51` binds "at least" seven things, and "a valid proof for one action cannot authorize another action, network, root". network ← `net`; action ← `ℓ⃗`, `box_moves`, `Δφ*`; operation identity ← `step`, `prev`; validity interval ← `h₀` and the scope's deadline (`intent.md:89`); profile ← the NSIG1 tag (`neuron-auth.md:15`) and the rule set (`chain.md:790–799`); state root, program, policy and public result ← σ's statement against the [[bbg]] root (`signal.md:15`), absent while σ is. amend both lines to bind these.
- the height rule: `order.md:15` "the signal's height t equals the current block height; past-height signals are stale" names one height, and a signal that spans blocks has its `h₀` in the past. the rule tests `h₁`, the seal's block; `h₀` is the height of the intent on record (`intent.md:12`, `:36`) and the deadline bounds the gap. amend `order.md:15` to say so.
- time: `order.md:18` and `validate.md:25` require "a valid VDF proof"; the root chain retired the VDF and orders by work (`cybernode:specs/chain-economy.md:27–30`; `cyber:launch.md:83`). amend the temporal row of `order.md` for a network ordered by work.
- `native-node.md:18` (soft3, cybernode), "one accepted signal advances one block", and `:23`, the node's atomic batch of 64 events, are the native node's policy and stay; the root chain's block records the signals of its height (`bbg:specs/state.md:75`).

## 2. every action through existing clauses

| action | expressed as | decisive clauses |
|---|---|---|
| pay | a [[cyberlink]] from the payer's card to the payee's carrying the [[box]] `(τ, a)`; the payer proves it owns `from`; tok calls it `pay` (G2) | `soft3:specs/terms.md:74`, `:96`; `tok:plumb.md:14` |
| [[stake]] | a cyberlink `(p, q, τ_native, a > 0, v)`; the chain moves the box from the author's balance to `stake_key(card)`; tok and `$CYB` call it `lock` (G2, G3) | `box.md:35`; `cybernode:specs/chain-economy.md:278–279`; `cyber:cyber/$CYB.md:722–724` |
| unlock | a box withdraw back to the author's wallet, the record staying; only the author, after `MIN_LOCK` (G2) | `box.md:37`; `bbg:specs/state.md:85`; `chain-economy.md:295–306` |
| payout owner | a card transfer: a cyberlink `(owner, new owner, card, 1, 0)` by `cyberlink.md:34`, or tok's `update`, "ownership change" (G2, G3) | `cyberlink.md:34`, `:79`; `tok:tsp-2.md:19`, `:40`; `bbg:specs/state.md:83` |
| legacy claim | the holder's own key, off chain: a holder whose key is known signs signals with it; the bridge binding is signed "with no on-chain transaction"; the 8,390 address-only balances parked under `legacy_owner` need a route (G15) | `chain.md:406–414`, `:416–434`; `mudra:specs/bridge.md:64–66` |
| authorization | π_id over the statement of `identity.md:36–51` (§1); NSIG1 on pussy while legacy balances remain; proved inside σ once the block proof is a validity condition | `proven-present.md:76`, `:80` |
| fee | the box inputs pay it: `Σ inputs = Σ outputs + fee`; each link consumes [[focus]], `cost(ℓ) = a(ℓ) + base_fee`, checked against `focus(ν)` (G5); one base fee per primitive under the EIP-1559 rule, its value network data; on pussy the fee goes to the miner | `validate.md:22`, `:23`; `cyber:whitepaper.md:1198`; `soft3:proposals/network-repos.md:108`; `chain-economy.md:700–701` |
| the 1 % tax | scoped to "every transfer … that moves CYB", half burned (`$CYB.md:423`). this page infers: pussy moves PUSSY (`proven-present.md:80`), so the tax does not reach it; CYB launches with claims only, so the tax sleeps there too | `$CYB.md:423`, `:431–448`; `network-repos.md:76`; `chain-economy.md:700–701` |
| address announcement | four cyberlinks from the neuron's name, each typed on its [[axon]] by a relation word (G13) | `address.md:7–16`; `neural:specs/relation.md:12–23` |
| coinbase, subsidy | a rule of the block, its record first; the subsidy pays a proof meeting a target | `proven-present.md:95`; `tru:specs/rewards.md:207–210` |
| settlement, the pool's cut | state transitions of the block, "not a transaction" | `chain-economy.md:334–335`, `:671–674` |
| revert (P9) | a statement of the checkpoint key, held by the owner, applied as a block-validity rule (G8) | `soft3:release/director/plan.md:170` on `release/phase1`; cybernode#12 `specs/chain.md:754` |
| the pool (rule set 2) | deposit and spend in `box_moves`, under σ; the turnstile is a block rule. a spend hides its author, so it carries neither `ν` nor a signature; its link `(p, q, weight, payout)` keeps the weight out of the value equation, against the 5-tuple, frozen in pussy's genesis as format 1 for rule set 2 to reconcile. launch row 10 (open) registers the plaintext signal as "the inverse of P1" | `soft3:proposals/privacy-frozen.md:36–38`, `:59`; `chain.md:1219–1221`, `:1346–1352`; `cyber:launch.md:110`, `:180` |

meaning stays [[neural]]'s: the chain checks conservation, ownership of `from` and the cost per link (`validate.md:15–26`); what a link means is read by the [[dialect]] that owns its [[word]] (`neural:specs/relation.md:41–45`).

## 3. what leaves the chain specifications

at cybernode main `52538a6`, each against its clause:

1. `chain.md` §2, the typed kinds, against one action and one carrier (`terms.md:74`): the envelope `version ‖ kind ‖ body ‖ sig` and its table (`291–311`), txid and signing statement (`313–320`), transfer (`340–350`), cyberlink with "why two neuron-signed kinds" (`352–373`); and what counts or dispatches on kinds — `MAX_BLOCK_TXS`, `MAX_TX_BYTES` (`72–73`), fixed size per kind (`259–261`), the root over kinds (`269–287`, becoming the root over the block's signals), the apply loop (`506–508`), validation step 7 (`772–773`), further kinds (`822–823`), `tx.rs` (`953`). the claim (`424–434`) stays until G15 has its route.
2. `chain.md:465`, the signals dimension written by nobody, against `network.md:53–55` and `bbg:specs/state.md:75`, which record every accepted signal there. the nonce stays: bbg keeps it in `NeuronRecord` under the root (`state.md:31`, `:73`), and the genesis root that block 0 commits (`chain.md:681`, `:684–685`) would move with it.
3. `chain.md:384–390` and `MIN_FEE_PER_BYTE` (`74`), against the cost per link (`validate.md:23`; `neuron.md:25`).
4. `chain-economy.md`, kinds duplicating the box lifecycle (`box.md:36–37`) and the one action (`terms.md:74`): lock as kind 2 (`266–285`), unlock's format (`289–293`), payout's format (`314–316`), kinds 8–10 reserved (`326–328`), the summary row (`86`). their rules — native token only, valence fixed while staked, author-only unlock after `MIN_LOCK` — stay as rules of the moves of §2.
5. `chain-economy.md:784–794`, kind 10 `report`, against `valence.md:27`: the valence is the BTS meta-prediction, with no separate submission.
6. `chain-wire.md:187–198`, an address as `p = hemera("antenna" | "socket" | "follow")`, against `address.md:9–14` and `neural:specs/relation.md:27–31` (G13); and the kinds of item 1 on the wire: the `txs` topic (`42`), get-txs (`74–75`), the mempool (`125–137`), the `/tx` routes (`221`, `233`, `253–266`).

on soft3 main the node specs moved to cybernode are still listed (`soft3:specs/README.md:27–30`), against "moved means moved" (`soft3:proposals/cybernode.md:65`).

## 4. gaps — where no clause answers

- G2 · how a link names its operation. the vocabulary is written: pay, lock, update, mint, burn (`tok:plumb.md:10–18`), transfer by `update` (`tok:tsp-2.md:19`), pay, lock, unlock (`$CYB.md:696–732`); so are the landings, `H(from ‖ τ) → H(to ‖ τ)` (`bbg:specs/state.md:105`) and `stake_key(card)` (`chain-economy.md:278–279`). `cyberlink.md:32–35` reads one 5-tuple both ways, and a card transfer has two carriers (`cyberlink.md:34`; tok's `update`). no clause says how one cyberlink declares its operation. amend `box.md` §moving a box with `tok:plumb.md`.
- G3 · one card per record or per axon, and who holds it. `cyberlink.md:77` makes every record a distinct card, told apart by its signal's ν and t; `bbg:specs/state.md:79–96` makes one card per author and axon, `card_id(author, p, q)`, with `author` (who may unlock) apart from `owner` (who is paid). amend one of the two.
- G5 · focus on pussy. `validate.md:23` checks `focus(ν) ≥ Σ cost(ℓ)` against the neurons dimension, and the chain keeps `NeuronRecord.focus` at 0 (`chain-economy.md:261`), so every link with a cost would fail. amend `validate.md`'s focus row or `chain-economy.md` §3.7 before the signal rule set.
- G7 · intents on the chain. intents stay on record at inception height (`intent.md:34–36`, `:94–96`), outside bbg's root until sealed or abandoned (`bbg:specs/storage.md:150`; `native-state.md:8–9`, `:66`; `soma:soma-spec.md:1061`; `cybernode:specs/native-node.md:18–22`), with a 64-byte signature (`native-state.md:59`) where NSIG1 has 102 bytes (`neuron-auth.md:11`). a seal names its intent only as an argument (`cli.md:41`; `cybergraph:src/api.rs:190`), and a sub-signal names the intent it fulfils nowhere (`cascade.md:10`, `:21–24`). no clause puts the intent in the block or its key in a frame. amend `bbg:specs/state.md` and `signal.md` §structure.
- G8 · keyed statements beside the signals. the checkpoint (`chain.md:724–748`) and the owner's revert (cybernode#12 `specs/chain.md:754`, `:789–797`; code in cybernode#15) are statements of one key outside any neuron's chain of signals; no clause of cybergraph, neural or bbg names a consensus input other than the signal. amend `chain.md` §5 to say so, or carry both as signals of `neuron_of(checkpoint_key)`.
- G11 · is the chain id a card. the value of `net` for the root chain exists: `H(canonical genesis bytes)` (`cyber:specs/node-product.md:101`; `cyber:README.md:49–50`, `:66–68`; `cyber:specs/cyb-node.md:79–82`), the chain id (`chain.md:118`; `network-repos.md:106`). `network.md:22` makes the cards dimension "the network registry" with `CardRecord { owner, particle }` (`:18`), while bbg's cards dimension holds positions `[owner, particle, author, valence, since]` (`bbg:specs/state.md:34`, `:79`); no clause says whether the chain id is a card there. amend `network.md` §a network is a card with `chain.md` §4.4.
- G13 · three address shapes. (a) the neuron's name at `from`, one link per kind (`address.md:9–14`); (b) a well-known word at `from`, `ANTENNA → endpoint-id`, `SOCKET → ip:port` (`cyb:decide/wire.md:23–29`; `cyb:crates/cyb/src/wire.rs:112`, `:377`; `chain-wire.md:187–198`); (c) a name record `~neuron/endpoint → particle(network, endpoint_id, relay_url, direct, revision)`, marked "proposed" (`cyber:network.md:71–77`). recommended (a), typed on its axon: `relation.md:27–31` rejects the route through a word as a super-node, which (b) makes of ANTENNA; `relation.md:12–23` types a link with no new mechanism; `:55` lets the words enter by use. amend `cyb:decide/wire.md` and `cyber:network.md:71–77` to (a), and `address.md` with the relation words of its four links.
- G15 · the address-only claim. kind 3 moves the 8,390 balances under `legacy_owner`, "that no key controls" (`chain.md:416–434`). the bridge gives their holders a signed binding `legacy address → native neuron`, verified natively (`mudra:specs/bridge.md:16–21`, `:82–84`), "not a chain transaction in phase 1" (`chain.md:411–414`); under signals a declared program would verify it, and pussy declares none (`network-repos.md:74`, `:109`). until then kind 3 stays. amend `network-repos.md`'s pussy column, or `chain.md` §2.8.

## 5. pussy's launch

the owner's words: pussy goes "on the air as soon as possible with the holes known and named in public" (`cyber:launch.md:68`); the ticket lottery, fold, the serum and market, and privacy "follow by scheduled fork" (`:99`); the freeze is withdrawn (D19, `soft3:release/director/plan.md:23`, `:86` on `release/phase1`; [cyber#155](https://github.com/cyberia-to/cyber/pull/155), open, so `launch.md:85` still reads it). proposed by this page: the chain launches with typed kinds as a named hole, and the signal rule set follows by the same route, a fork announced ≥ 40,320 blocks ahead (`chain.md:802–804`). the row for the holes table of `launch.md:70–80`:

| hole | measured consequence | fix lane | where |
|---|---|---|---|
| the chain speaks typed kinds, not signals: rule set 1 carries kinds 0–3, 6 and 7 (`cybernode/specs/chain.md` §2, `chain-economy.md` §3) where soft3 specifies one message, the signal | not measured; every block until the fork carries kinds, and the history before it stays in kinds | proposed: a scheduled fork to the signal rule set, announced on cyber/launch ≥ 40,320 blocks before its height, once its gaps have clauses | [soft3#677](https://github.com/cyberia-to/soft3/pull/677) |
