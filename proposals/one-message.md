---
tags: cyber, soft3, proposal, chain, cybernode, cybergraph, neural, signal, intent, cyberlink, box, genesis, pussy
crystal-type: process
crystal-domain: cyber
alias: one message, one kind of message, signals only, the chain speaks signals
status: proposed
date: 2026-11-02
revised: 2026-11-03 — rewritten from the soft3 specifications after the owner rejected the first draft (tasks/one-message.md)
---
# one message — the root chain carries the signal soft3 already defines

the owner, 2026-11-02 (translated): "in my design of soft3 there is only one kind of message — links — everything else is the semantics of neural"; "there are no transactions, there are signals". 2026-11-03: "the essence of a signal is that one neuron creates it as an intent with provable conditions of completion, and then any neurons can fulfil it, even in different blocks; a signal may fit one block only if it is minimal."

this page states the [[signal]] and the [[cyberlink]] as the specifications define them, expresses every act the chain needs through them, lists what the chain specifications carry against them, and names the places where no clause exists yet. it adds no format, field, tag or number. every source was read whole at its origin head: cybergraph `0a70b6a`, neural `47b086c`, soft3 main `3045d54` and `release/phase1` `e340d47`, mudra `3969c7a`, tok `c773592`, bbg `2294281`, tru `e660d1e`, cyber `68f1260a`, soma `1d3fe69`, cybernode main `52538a6`. a clause is cited `repo:file:line`; a bare file name is cybergraph `specs/`.

## 1. the signal

one [[neuron]] declares an [[intent]]: a scope it commits to, signed, nothing run yet. the scope is the provable condition of completion: a seal is accepted only when its proof attests that scope. any neurons may fulfil the intent, each with its own signal in its own block, and the parent is sealed over all of them. only a minimal signal fits in one block, declared and sealed together; every other signal is an intent first and is sealed later.

| concept | clause | what it says |
|---|---|---|
| intent | `intent.md:7–14`, `:44–47`, `:81–92` | `i = (ν, h₀, Σ, π_id)`; Σ names target particles, predicate, deadline, constraints; the scope is an executable specification of what to compute; its encoding is the dialect's |
| identity proof | `intent.md:75–79` | `π_id = Sign_ν(ν ‖ h₀ ‖ hash(Σ))`: unforgeable, no content proof |
| completion | `intent.md:44–52` | run the scope on [[nox]] over committed state, prove the run with [[zheng]] into σ, seal |
| seal binding | `intent.md:56–62`; `README.md:70–72` | `seal(i, s)` accepted iff `σ(s) ⊢ scope_hash(i)`; without it any signal could seal any intent |
| open fulfilment | `intent.md:38–40`, `:94–96`; `cyber:cascade.md:7–29`; `cyber:intent.md:28`; `soma:soma-spec.md:1082` | pending intents are readable by every neuron; subscribers self-assign and produce sub-signals; the parent is sealed with a recursive proof over the cascade; soma fetches "an intent … mine, or one I claim" |
| signal | `signal.md:7–21` | `s = (ν, net, ℓ⃗, Δφ*, σ, h₀, h₁)`; h₀ declaration, h₁ sealing, `h₁ − h₀` the span; before h₁ the signal exists as an intent |
| minimal signal | `neural:specs/wire.md:36`; `neural:specs/cli.md:97`; `docs/README.md:80`; `soma-spec.md:1094` | "intent → seal — a two-phase submit, or one-shot for small signals"; `link(signal)` is the one-shot path: `h₀ = h₁` |
| destination | `signal.md:27–31`; `network.md:53–55` | `net` names one network and is bound into the signal hash; the resolved network is recorded with the signal |
| cyberlink | `cyberlink.md:3–13`; `link.md:9–20` | `(p, q, τ, a, v)` = a [[link]] between two cards + a [[box]] `(τ, a)` + a [[valence]]; who and when belong to the signal |
| card | `link.md:20`; `soft3:specs/terms.md:71–74` | particles, neurons and networks are cards; a cyberlink is the only action and moves one token between two tokens |
| box | `box.md:21–38` | creating a cyberlink moves a box; its lifecycle is create, transfer, withdraw, spend |
| authorization | `soft3:specs/terms.md:96`; `link.md:15` | proving ownership of the `from` token is the only authorization |
| cost | `signal.md:3`; `neuron.md:25`; `validate.md:22–23` | every link consumes focus: `cost(ℓ) = a(ℓ) + base_fee`; box inputs equal outputs plus the fee |
| order | `order.md:15–20` | a signal takes its neuron's next step, extends `prev`, has no rival at that step |
| record | `bbg:specs/state.md:75` | a signal lives at `signal_key(neuron, step)` with `[neuron, network, link_count, block_height, proof_hash, step]` |
| meaning | `neural:specs/binding.md:16`, `:30–32`; `neural:specs/relation.md:12–23`, `:43` | a word is `hemera(name)`; a relation is a link on the axon; a dialect expands a sentence into links; neural writes and parses, never computes meaning |

## 2. the block

1. a block is a bundle of steps, a step one `apply` into [[bbg]] (`soft3:proposals/steps-blocks-epochs.md:16–17`; `apply.md:11–15`); validate → order → apply is the write path of `link` and `seal` (`README.md:68`).
2. a block carries the intents declared at its height and the signals sealed at its height; a minimal signal is both. bbg stores an intent record (neuron, inception height, scope hash, signature) without advancing the height (`bbg:specs/native-state.md:8–9`, `:59`; `bbg:specs/storage.md:150`).
3. beside them the block applies its own rules: the subsidy to its miner and the epoch settlement. `soft3:proposals/proven-present.md:95` lists the coinbase beside the steps; `cybernode:specs/chain-economy.md:334–335`, `:674` make settlement and the pool's cut transitions of the block; the subsidy's proof certifies work and its earner is whoever computes (`tru:specs/rewards.md:203–217`).
4. each σ is proved by its author and folded into the block's proof, which verifies its parents' (`proven-present.md:26–39`, `:72`). until that proof is a validity condition, pussy validates natively, authorizes with mudra NSIG1 as π_id, and is proven after the fact (`proven-present.md:75–76`; `mudra:specs/neuron-auth.md:8–24`); a signal with no σ and the seal binding not yet enforced is release 0 of cybergraph (`cli.md:44`). nothing activates that the block relation cannot express (`proven-present.md:52`, `:85`). so pussy's rule set 1 carries minimal signals, and intents that span blocks open with the rule set that enforces `σ ⊢ scope_hash`.
5. the bytes are soft3's schema: header `ν ‖ t ‖ n`, link records `p ‖ q ‖ τ ‖ a ‖ v` of 105 bytes (`soft3:schema/README.md:20–38`). what the schema lacks is G1.

## 3. every act the chain needs

| act | through the model | clauses | cybernode today |
|---|---|---|---|
| pay a coin | a link from the payer's card to the payee's, box `(τ, a)`; the payer proves it owns `from` (see G2) | `terms.md:74`, `:96`; `cyberlink.md:3`; `link.md:20`; `tok:plumb.md:39` (transfer = pay); `tok:tsp-1.md:44` | kind 1, `chain.md:340–350` |
| hand over a card, the payout owner of a position included | a link `(owner, new owner, card, 1, v)` | `cyberlink.md:18`, `:34`, `:79`; `box.md:19`, `:36`; `tok:tsp-2.md:19` | kind 7, `chain-economy.md:312–324` |
| stake on an axon | a link `(p, q, τ_native, a > 0, v)` in ν's signal; the box stays bound to the record (see G2, G3) | `staking.md:13`; `box.md:35`; `cyberlink.md:41–43` | kind 2 with stake, `chain-economy.md:266–285` |
| unlock | withdraw: the box returns to its author's wallet, the record stays | `box.md:37`; `cyberlink.md:54`; `staking.md:13` | kind 6, `chain-economy.md:287–310` |
| claim a dormant legacy balance | a link from `legacy_owner(hrp, id)` to the claimant, one per token; ownership of `from` is proved by the legacy key whose account id is `id` | `terms.md:96`; `link.md:15`; `mudra:specs/identity.md:29–32`; `proven-present.md:76–84`; the dormant owner is genesis data, `chain.md:416–422` | kind 3, `chain.md:424–434` |
| announce an address | FOLLOW, ANTENNA, SOCKET, LOCUS: links from the neuron's name, their kind carried on the axon | `address.md:7–16`; `neural:specs/relation.md:12–23`, `:43`; `neural:specs/wire.md:32` | kind 2 with a word as `p`, `chain-wire.md:187–198` |
| coinbase and subsidy | a rule of the block, its record first in the body | `proven-present.md:95`; `tru:specs/rewards.md:217`; `chain.md:256`, `:322–338` | kind 0 |
| the focus pool and settlement | rules of the block; while σ is absent the reward is recomputed on every node, and the self-mint of the signal arrives with the ticket rule set | `chain-economy.md:32–35`, `:334–335`, `:674`, `:754–778`; `signal.md:45–50`; `tru:specs/rewards.md:213` | the same |
| deposit and spend (rule set 2) | box create into the commitment set and box spend by nullifier, inside a signal whose σ covers the box moves (see G6) | `box.md:38`, `:88`; `signal.md:37–39`; `validate.md:20–22` | kinds 4 and 5, `chain.md:1255–1290` |
| revert (P9) | an entry compiled into a release is configuration (see G8) | `chain.md:790–806`; `soft3:specs/execution-model.md:80–83`, `:97–100` | §5.6 draft, cybernode#12 |
| the fee | `base_fee` per link, paid to the miner (see G5) | `validate.md:22–23`; `neuron.md:25`; `soft3:proposals/privacy-frozen.md:76`; `chain-economy.md:700–701` | per byte, `chain.md:384–390` |

meaning is neural's. the chain enforces value and authority — conservation, ownership of `from`, the cost of each link (`tok:plumb.md:61–70`; `validate.md:15–26`); what a link means — follow, antenna, passport, is-a — is read by the dialect that owns its word, from the genesis seed of TRUE, FALSE and the relation words (`neural:specs/dialect.md:22`; `relation.md:47–53`).

## 4. what goes from the chain specifications

at cybernode main `52538a6`:

1. `chain.md` §2, the typed kinds: the envelope `version ‖ kind ‖ body ‖ sig` and its table (`291–311`), txid and signing statement with their tags (`313–320`), transfer (`340–350`), cyberlink with "why two neuron-signed kinds" (`352–373`), claim (`424–434`); and every place that counts or dispatches on kinds — `MAX_BLOCK_TXS`, `MAX_TX_BYTES` (`72–73`), fixed size per kind (`259–261`), the root over kinds (`269–287`, becoming the root over the block's intents and signals), the apply loop (`506–508`), validation step 7 (`772–773`), further kinds (`822–823`), `tx.rs` (`953`). replaced by §1–§2.
2. `chain.md:375–382` and `NONCE_WINDOW` (`82`): the nonce is the step of `order.md:15–17`; the signals dimension, left empty (`chain.md:465`), records each signal (`bbg:specs/state.md:75`; `network.md:53–55`).
3. `chain.md:384–390` and `MIN_FEE_PER_BYTE` (`74`): the cost is per link (`validate.md:23`; `neuron.md:25`).
4. `chain-economy.md`: lock as kind 2 (`266–285`), unlock's format (`289–293`), payout's format (`314–316`), the reservation of kinds 8–10 (`326–328`), the summary row (`86`). their rules — native token only, valence fixed while staked, author-only unlock after `MIN_LOCK` — stay as rules of the moves of §3.
5. `chain-economy.md:784–794`, kind 10 `report`: the valence is the BTS meta-prediction, with no separate submission (`valence.md:27`).
6. `chain-wire.md:187–198`: an address as `p = hemera("antenna" | "socket" | "follow")` puts a word where `address.md:9–14` puts the neuron's name, the word-through route `relation.md:27–31` rejects as a super-node; and the `txs` topic (`42`), get-txs (`74–75`), the mempool by kind (`125–137`), the `/tx` routes and kind names (`221`, `233`, `253–266`) become carriers of signals.
7. `chain.md:465` lists `intents` as a committed empty dimension; bbg commits eleven dimensions without it (`bbg:specs/state.md:27–38`) and its root excludes intent records (`bbg:specs/native-state.md:66`).
8. the P5 draft's "envelope v2" with an AUTH1 proof (cybernode#11, §2.9): the proof that ν holds its secret belongs to σ (`validate.md:18`; `proven-present.md:76`).
9. `chain-conformance.md:142–200` and `chain-economy.md:887–894`, the per-kind vectors: one signal vector per act of §3.
10. soft3 main still carries the moved node specs (`soft3:specs/README.md:27–30` and the files): "moved means moved; the soft3 copies are deleted" (`soft3:proposals/cybernode.md:65`).

the pool's link output and kinds 4 and 5 (`chain.md:1196`, `:1219–1221`, `:1255–1290`) are G6: `chain.md:1346–1352` freezes them in the genesis file.

## 5. gaps — where no clause answers

- G1 · the signal's bytes. `soft3:schema/README.md:20–38` predates `net`, step, `prev`, h₀/h₁, π_id and σ (`signal.md:7–17`; `order.md:15–18`; `intent.md:75–79`) and cites `cybergraph/specs/graph`, which no longer exists. amend `soft3:schema/README.md`, a do-not-touch zone: the SDKs move with it.
- G2 · where a created box lands and where it leaves from. `terms.md:74` and `cyberlink.md:3` move it between `from` and `to`, a payment; `box.md:23`, `:35–37` and `staking.md:13` bind it to the record for its author, withdrawable, a position; `cyberlink.md:32–35` reads one shape both ways by what `to` is, and `particle.md:79` types an address only by where it appears in bbg, where a neuron that never signed does not appear. the source is `from` in `box.md:23` and `link.md:15`, the author's wallet in `box.md:35`; for a staked `(p, q)` by ν these differ. the chain answers with two kinds; the model answers in the box rule. amend `box.md` §moving a box.
- G3 · the card of a record. `cyberlink.md:71–79` makes every record a card told apart by its signal's ν and t, and `query.md:48–65` keys records `(neuron, step, idx)`; no clause gives that card's 32-byte id. `chain-economy.md:206–233` writes `card(author, p, q)` with its own tags and merges an author's records on one axon. amend `cyberlink.md` §the card.
- G4 · what the identity proof binds. `intent.md:77` signs `ν ‖ h₀ ‖ hash(Σ)`, `cybergraph.md:41` signs `H(s)`, `validate.md:18` signs `H(ℓ⃗ ‖ Δφ* ‖ t)`, `signal.md:31` binds `net`, and `mudra:specs/identity.md:36–51` requires network, operation identity and action bound. amend `intent.md` §identity proof and `validate.md`'s signature row to bind `net` and the step.
- G5 · which balance pays focus. `validate.md:23` debits `focus(ν)`; the chain keeps `NeuronRecord.focus` at zero (`chain.md:462`) and pays in the native coin; no clause equates them, and `base_fee` has no value anywhere (`soft3:proposals/network-repos.md:108` makes such values network data). amend `validate.md`'s focus row.
- G6 · the pool. `privacy-frozen.md:38` and `chain.md:1196`, `:1219–1221` write a pool link as `(p, q, weight, payout)` — no box, the weight outside the value equation — against the 5-tuple whose box is conserved (`cyberlink.md:3–13`; `validate.md:22`); a spend shows no neuron (`privacy-frozen.md:36`) while every signal carries its neuron's signature (`cybergraph.md:41`). amend `privacy-frozen.md` §1.6 and `chain.md` §11.6 for the link; `cybergraph.md` A2 for the hidden signer, by rule set 2.
- G7 · intents under the root. `intent.md:34–36`, `:94–96` keep every intent on record and readable; bbg's root excludes them (`native-state.md:66`), and its intent record keeps a 64-byte signature (`native-state.md:59`) where an NSIG1 envelope is 102 bytes (`neuron-auth.md:11–17`). amend `bbg:specs/state.md` before intents span blocks.
- G8 · the revert by key. the P9 draft adds a signed revert statement (cybernode#12, §5.6) beside the signed checkpoint (`chain.md:724–748`): two keyed formats for one power, the veto of blocks; the release table needs no message (`chain.md:790–806`). amend `chain.md` §5 to one keyed format.
- G9 · time. `order.md:18` requires a VDF; `chain-economy.md:27–30` retired it from the root chain, where blocks ordered by work carry the height; `validate.md:25` hands ordering to the chain. amend `order.md`'s temporal row for a network ordered by work.
- G10 · who fulfils. `intent.md:23` says the neuron finalizes, where `intent.md:40`, `:96`, `cyber:cascade.md:21–24` and `soma-spec.md:1082` let any neuron fulfil and the lead seal. amend `intent.md` §lifecycle to name the fulfillers as cascade does.

## 6. pussy's genesis

the genesis file holds the `p` frame with the pool record, the `b` balances and the `l` links (`chain.md:565–600`, `:1337–1352`); the chain id is its particle (`chain.md:569–570`). rule set 1's messages live outside the file: they change before genesis without moving the chain id, and after it only by a scheduled rule set with two weeks' notice (`chain.md:788–806`).

- one decision binds genesis: G6's link half. format 1 freezes a pool link that is not a cyberlink into pussy forever; a 5-tuple link is a new pool format, so the pool record, the chain id and the pins move once.
- the `l` frames already apply as cyberlinks `(p, q, τ_native, 0, 0)` by their neuron at height 0 (`chain.md:593–598`); `legacy_owner` and the naming links of the genesis neuron are genesis data and stay (`chain.md:416–422`, `:646–655`).
- rule set 1 as signals (§4 items 1–6, 8, 9) touches the modules that dispatch on kinds, measured on cybernode `release/phase1` `599cd85`: `net/codec.rs` 898 lines (68 kind matches), `tx.rs` 211 (16), `net/mempool.rs` 528, `net/discovery.rs` 330, `corpus.rs` 460, `engine_tx.rs` 226, with their tests and vectors, plus cyb's signer and the simulation. it waits on G1 and G2, whose owners are soft3 schema and cybergraph; written before them, it would invent. until then pussy runs today's kinds, and signals come as a scheduled rule set.
- §4 items 7 and 10 are text with no consensus effect and can land at once.
