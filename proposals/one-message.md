---
tags: cyber, soft3, proposal, chain, cybernode, cybergraph, neural, signal, intent, cyberlink, box, genesis, pussy
crystal-type: process
crystal-domain: cyber
alias: one message, one kind of message, signals only, the chain speaks signals
status: proposed
date: 2026-11-02
revised: 2026-11-03 — rewritten from the soft3 specifications after the owner rejected the first draft; round 2 after review round 1 (tasks/one-message.md)
---
# one message — the root chain carries the signal, as the owner defines it

the owner, 2026-11-02 (translated): "in my design of soft3 there is only one kind of message — links — everything else is the semantics of neural"; "there are no transactions, there are signals". 2026-11-03: "the essence of a signal is that one neuron creates it as an intent with provable conditions of completion, and then any neurons can fulfil it, even in different blocks; a signal may fit one block only if it is minimal."

the owner's sentence is the authority, and it corrects the specifications: nine of their lines define the signal as a batch committed atomically in one step (§5, A1–A10). this page states the [[signal]] and the [[cyberlink]] in the owner's sense, expresses every act the chain needs through them with the clause that already defines it, lists what the chain specifications carry against those clauses, and names where a clause is still missing. its formats, fields, tags and parameters are the specifications' own. every source was read whole at its origin head: cybergraph `0a70b6a`, neural `47b086c`, soft3 main `3045d54` and `release/phase1` `e340d47`, mudra `3969c7a`, tok `c773592`, bbg `2294281`, tru `e660d1e`, cyber `68f1260a`, soma `1d3fe69`, cybernode main `52538a6`. a clause is cited `repo:file:line`; a bare file name is cybergraph `specs/`.

## 1. the signal

one [[neuron]] declares an [[intent]]: a scope it commits to, signed, nothing run yet. the scope is the provable condition of completion: a seal is accepted only when its proof attests that scope. any neurons may fulfil the intent, each with its own signal, in blocks of their own, and the parent is sealed over all of them. a small signal fits one block, declared and sealed together; every other signal is an intent first and is sealed later. "minimal", "in blocks of their own" and "declared and sealed together" (`h₀ = h₁`) are the owner's words of 2026-11-03; the specifications say "one-shot for small signals" (`neural:specs/wire.md:36`) and give the test: two phases "when the action is a multi-step computation or invites coordination" (`soma:soma-spec.md:1094`).

| concept | clause | what it says |
|---|---|---|
| intent | `intent.md:7–14`, `:44–47`, `:81–92` | `i = (ν, h₀, Σ, π_id)`; Σ names target particles, predicate, deadline, constraints; the scope is an executable specification of what to compute; its encoding is the dialect's |
| identity proof | `intent.md:14`, `:75–79` | `π_id = Sign_ν(ν ‖ h₀ ‖ hash(Σ))`, of type Sig |
| completion | `intent.md:44–52` | run the scope on [[nox]] over committed state, prove the run with [[zheng]] into σ, seal |
| seal binding | `intent.md:56–62`; `README.md:70–72` | `seal(i, s)` accepted iff `σ(s) ⊢ scope_hash(i)`; without it any signal could seal any intent |
| open fulfilment | `intent.md:38–40`, `:94–96`; `cyber:cascade.md:7–29`; `cyber:intent.md:28`; `soma-spec.md:1082` | pending intents are readable by every neuron; subscribers self-assign and produce sub-signals; the parent is sealed with a recursive proof over the cascade; soma fetches "an intent … mine, or one I claim" |
| signal | `signal.md:7–21` | `s = (ν, net, ℓ⃗, Δφ*, σ, h₀, h₁)`; h₀ declaration, h₁ sealing, `h₁ − h₀` the span; before h₁ the signal exists as an intent |
| small signal | `neural:specs/wire.md:36`; `neural:specs/cli.md:97`; `docs/README.md:80`; `soma-spec.md:1094` | "intent → seal — a two-phase submit, or one-shot for small signals"; `link(signal)` is the one-shot path |
| destination | `signal.md:27–31`; `network.md:12–23`, `:41–55` | `net` is one network's card, bound into the signal hash; joining a shared network is opt-in; a node serving a network rejects others; the resolved network is recorded with the signal |
| cyberlink | `cyberlink.md:3–13`; `link.md:9–20` | `(p, q, τ, a, v)` = a [[link]] between two cards + a [[box]] `(τ, a)` + a [[valence]]; who and when belong to the signal |
| card | `link.md:20`; `soft3:specs/terms.md:71–74` | particles, neurons and networks are cards; a cyberlink is the only action and moves one token between two tokens |
| box | `box.md:21–38` | creating a cyberlink moves a box; its lifecycle is create, transfer, withdraw, spend |
| authorization | `soft3:specs/terms.md:96`; `link.md:15` | proving ownership of the `from` token is the only authorization |
| cost | `signal.md:3`; `neuron.md:25`; `validate.md:22–23` | every link consumes [[focus]]: `cost(ℓ) = a(ℓ) + base_fee`; box inputs equal outputs plus the fee |
| order | `order.md:15–20` | a signal takes its neuron's next step, extends `prev`, has no rival at that step |
| record | `bbg:specs/state.md:75` | a signal lives at `signal_key(neuron, step)` with `[neuron, network, link_count, block_height, proof_hash, step]` |
| meaning | `neural:specs/binding.md:16`, `:30–32`; `neural:specs/relation.md:12–23`, `:43` | a [[word]] is `hemera(name)`; a relation is a link on the [[axon]]; a [[dialect]] expands a sentence into links; neural writes and parses, never computes meaning |

## 2. the block

1. a block is a bundle of steps, a step one `apply` into [[bbg]] (`soft3:proposals/steps-blocks-epochs.md:16–17`; `apply.md:11–15`); validate → order → apply is the write path of `link` and `seal` (`README.md:68`). a block carries the seals applied at its height: the signals it records at `signal_key(ν, step)` with their `block_height` (`bbg:specs/state.md:75`).
2. an intent is recorded and readable at its inception height (`intent.md:34–36`, `:94–96`); bbg applies an intent as an event of its own, without advancing the height (`bbg:specs/native-state.md:8–9`, `:59`). for the chain to agree on that record, the intent rides the block of its height: no clause says so yet (G7).
3. beside them the block applies its own rules: the subsidy to its miner and the epoch settlement. `soft3:proposals/proven-present.md:95` lists the coinbase beside the steps; `cybernode:specs/chain-economy.md:334–335`, `:674` make settlement and the pool's cut transitions of the block; the subsidy's proof certifies work and its earner is whoever computes (`tru:specs/rewards.md:203–217`).
4. each σ is proved by its author and folded into the block's proof, which verifies its parents' (`proven-present.md:26–39`, `:72`). until that proof is a validity condition, pussy validates natively and is proven after the fact (`proven-present.md:75`); its admitted signature is mudra NSIG1 (`proven-present.md:80`; `mudra:specs/neuron-auth.md:8–24`), the Sig of π_id. a signal without σ and with the seal binding unenforced is release 0 of cybergraph (`cli.md:44`); nothing activates that the block relation cannot express (`proven-present.md:52`, `:85`). so the first rule set that carries signals carries small signals, and intents that span blocks open with the rule set that enforces `σ ⊢ scope_hash`. what pussy launches with is the owner's question of §6; this page recommends today's kinds.
5. the canonical form of a signal on the wire is a [[tade]] frame — ZAP a signal, KET an intent, the frames foculus mints and radio carries (`cli.md:21–31`; `bbg:specs/cli.md:27–31`). its field set is G1.

## 3. every act the chain needs

| act | through the model | clauses | cybernode today |
|---|---|---|---|
| pay a [[token]] | a link from the payer's card to the payee's, box `(τ, a)`; the payer proves it owns `from` (G2) | `terms.md:74`, `:96`; `cyberlink.md:3`; `link.md:20`; `tok:plumb.md:39`; `tok:tsp-1.md:44` | kind 1, `chain.md:340–350` |
| hand over a card, the payout owner of a position included | a link `(owner, new owner, card, 1, v)` | `cyberlink.md:18`, `:34`, `:79`; `box.md:19`, `:36`; `tok:tsp-2.md:19` | kind 7, `chain-economy.md:312–324` |
| stake on an axon | a link `(p, q, τ_native, a > 0, v)` in ν's signal; the box stays bound to the record (G2, G3) | `staking.md:13`; `box.md:35`; `cyberlink.md:41–43`; `bbg:specs/state.md:79–96` | kind 2 with stake, `chain-economy.md:266–285` |
| unlock | withdraw: the box returns to its author's wallet, the record stays; a signal carries the spend of an existing box, its σ attesting the box is live and owned (G10) | `box.md:37`; `cyberlink.md:54`; `staking.md:13`; `validate.md:20–21` | kind 6, `chain-economy.md:287–310` |
| claim a dormant legacy balance | the [[bridge]]: the holder signs `legacy address → native neuron` and the binding returns control to the original holder; afterwards the neuron owns the dormant card and moves its balances by links (G12) | `mudra:specs/bridge.md:16–21`, `:28`, `:46`, `:73–76`; `chain.md:410–422`; `terms.md:96` | kind 3, `chain.md:424–434` |
| announce an address | FOLLOW, ANTENNA, SOCKET, LOCUS: four ordinary cyberlinks from the neuron's name (G13) | `address.md:7–16`; `neural:specs/relation.md:12–23` | kind 2 with a word as `p`, `chain-wire.md:187–198` |
| coinbase and subsidy | a rule of the block, its record first in the body | `proven-present.md:95`; `tru:specs/rewards.md:217`; `chain.md:256`, `:322–338` | kind 0 |
| the focus pool and settlement | rules of the block; while σ is absent the reward is recomputed on every node; the self-mint of the signal arrives with the ticket rule set | `chain-economy.md:32–35`, `:334–335`, `:674`, `:754–778`; `signal.md:45–50`; `tru:specs/rewards.md:213` | the same |
| deposit and spend (rule set 2) | box create into the commitment set and box spend by nullifier, inside a signal whose σ covers the box moves (G6, G10) | `box.md:38`, `:88`; `signal.md:37–39`; `validate.md:20–22` | kinds 4 and 5, `chain.md:1255–1290` |
| revert (P9) | an entry compiled into a release, beside the activation table (G8) | `chain.md:790–806` | §5.6 draft, cybernode#12 |
| the destination | the root chain's network, named in every signal, since a signal without it lands in its sender's private network (G11) | `signal.md:29`; `network.md:41–51`; `validate.md:43` | the chain id in every statement, `chain.md:313–320` |
| the fee | `base_fee` per link, paid to the miner (G5) | `validate.md:22–23`; `neuron.md:25`; `soft3:proposals/privacy-frozen.md:76`; `chain-economy.md:700–701` | per byte, `chain.md:384–390` |

meaning is [[neural]]'s. the chain enforces value and authority — conservation, ownership of `from`, the cost of each link (`tok:plumb.md:61–70`; `validate.md:15–26`); what a link means — follow, antenna, passport, is-a — is read by the dialect that owns its word, from the genesis seed (G14).

## 4. what goes from the chain specifications

at cybernode main `52538a6`, each against the clause it contradicts or duplicates:

1. `chain.md` §2, the typed kinds, against one action and one carrier (`terms.md:74`; `neural:specs/sigils.md:31–33`, "cyberlinks are the only primitive"): the envelope `version ‖ kind ‖ body ‖ sig` and its table (`291–311`), txid and signing statement with their tags (`313–320`), transfer (`340–350`), cyberlink with "why two neuron-signed kinds" (`352–373`), claim (`424–434`); and every place that counts or dispatches on kinds — `MAX_BLOCK_TXS`, `MAX_TX_BYTES` (`72–73`), fixed size per kind (`259–261`), the root over kinds (`269–287`, becoming the root over the block's signals), the apply loop (`506–508`), validation step 7 (`772–773`), further kinds (`822–823`), `tx.rs` (`953`).
2. `chain.md:465`, the signals dimension written by nobody, against `network.md:53–55` and `order.md:20`, which record every accepted signal there. the nonce stays: bbg keeps it as the root chain's replay counter in `NeuronRecord`, under the root (`bbg:specs/state.md:31`, `:73`), and the step as the key of a signal record (`:75`); moving the nonce would change the neurons leaf and so block 0's root (`chain.md:685`), and this page proposes neither.
3. `chain.md:384–390` and `MIN_FEE_PER_BYTE` (`74`), against the cost per link (`validate.md:23`; `neuron.md:25`).
4. `chain-economy.md`, kinds duplicating the box lifecycle's transfer and withdraw (`box.md:36–37`) and the one action (`terms.md:74`): lock as kind 2 (`266–285`), unlock's format (`289–293`), payout's format (`314–316`), the reservation of kinds 8–10 (`326–328`), the summary row (`86`). their rules — native token only, valence fixed while staked, author-only unlock after `MIN_LOCK` — stay as rules of the moves of §3.
5. `chain-economy.md:784–794`, kind 10 `report`, against `valence.md:27`: the valence is the BTS meta-prediction, with no separate submission.
6. `chain-wire.md:187–198`, an address as `p = hemera("antenna" | "socket" | "follow")`, against `address.md:9–14`, which puts the neuron's name at `from`, and `relation.md:27–31`, which rejects the word-through route as a super-node; and the `txs` topic (`42`), get-txs (`74–75`), the mempool by kind (`125–137`), the `/tx` routes and kind names (`221`, `233`, `253–266`), which carry the kinds of item 1.
7. `chain.md:465` lists `intents` as a committed empty dimension, against bbg, which commits eleven dimensions without it (`bbg:specs/state.md:27–38`) and keeps intent records outside the root (`bbg:specs/native-state.md:66`).
8. the P5 draft's "envelope v2" with an AUTH1 proof (cybernode#11, §2.9), against `validate.md:18` (the signature is checked by σ) and `proven-present.md:76` (authorization proved inside the step's proof).
9. `chain-conformance.md:142–200` and `chain-economy.md:887–894`, the per-kind vectors: they follow items 1 and 4 and contradict nothing on their own.

the pool's link output and kinds 4 and 5 (`chain.md:1196`, `:1219–1221`, `:1255–1290`) are G6: `chain.md:1346–1352` freezes them in the genesis file. on the soft3 side, soft3 main still carries the node specs moved to cybernode (`soft3:specs/README.md:27–30` and the files), against "moved means moved; the soft3 copies are deleted" (`soft3:proposals/cybernode.md:65`).

## 5. amendments and gaps

### amendments — the owner's sentence into the specifications

| id | clause | old | new |
|---|---|---|---|
| A1 | `signal.md:3` | "a bundle of cyberlinks a neuron commits in a single step — the atomic broadcast unit in cyber." | "an intent a neuron declares with provable conditions of completion, fulfilled by any neurons, possibly across blocks, and sealed when its proof attests the scope; a small signal is declared and sealed in one block." |
| A2 | `signal.md:35` | "σ is a single zheng proof that covers the entire signal atomically:" | "σ is a single zheng proof over the whole fulfilment of the intent, its sub-signals folded in recursively:" |
| A3 | `proof.md:9` | "one proof, covering the whole batch atomically" | "one proof, covering the whole fulfilment of the intent" |
| A4 | `cyberlink.md:23` | "cyberlinks are bundled into signals for broadcast … a recursive zheng proof covering the entire batch." | "cyberlinks are carried by signals … a recursive zheng proof covering every fulfilment of the signal's intent." |
| A5 | `intent.md:62`; `README.md:72` | "one transaction" | "one signal" |
| A6 | `intent.md:23` | "sealed — neuron finalizes: adds sealing height + zheng proof" | "sealed — the fulfilment, by any neurons, is proven: sealing height + zheng proof" |
| A7 | `soft3:specs/terms.md:76` | "a signed, staked batch of cyberlinks; carries the subject (neuron) and block height." | "an intent a neuron declares and signs, fulfilled in cyberlinks by any neurons, possibly across blocks; carries the subject and the heights of declaration and sealing." |
| A8 | `neural:specs/wire.md:16`, `:37` | "a signal is one atomic submission — a batch that lands together or not at all"; "atomicity — the whole signal commits or aborts" | "a signal is one intent and its seal"; "the seal commits the whole fulfilment or nothing; an unsealed intent stays on record" |
| A9 | `neural:specs/cli.md:95` | "an atomic batch of links by one neuron" | "an intent by one neuron, sealed once its links are in; a small signal in one shot" |
| A10 | `neural:specs/README.md:37` | "one atomic batch of links." | "an intent by one neuron and its seal." |

### gaps — where no clause answers

- G1 · the signal's field set. the form is named, tade frames (§2.5); `soft3:schema/README.md:20–38` gives fixed-width fields `ν ‖ t ‖ n ‖ links` that predate `net`, step, `prev`, h₀/h₁, π_id and σ (`signal.md:7–17`; `order.md:15–18`; `intent.md:75–79`) and cites `cybergraph/specs/graph`, which is gone. amend `soft3:schema/README.md` (the SDKs move with it) and the foculus frame codec together.
- G2 · which landing a link takes. both exist: bbg's `insert` moves a conviction box `H(from ‖ τ) → H(to ‖ τ)` and a lock moves it to `stake_key(card)` (`bbg:specs/state.md:105`, `:96`); `terms.md:74` and `cyberlink.md:3` read a payment, `box.md:23`, `:35–37` and `staking.md:13` a position bound to the record; `cyberlink.md:32–35` reads one shape both ways by what `to` is, and `particle.md:79` types an address only by where it appears in bbg. the source differs too: `from` in `box.md:23` and `link.md:15`, the author's wallet in `box.md:35`. what is missing is the discriminator inside one message. amend `box.md` §moving a box.
- G3 · one card per record or per axon. `cyberlink.md:77` makes every record a distinct card, told apart by its signal's ν and t; `bbg:specs/state.md:79–96` makes one card per author and axon, `card_id(author, p, q) = H(D_CARD ‖ author ‖ p ‖ q)`. the choice amends one of the two.
- G4 · what the identity proof binds. `intent.md:77` signs `ν ‖ h₀ ‖ hash(Σ)`, `cybergraph.md:41` `H(s)`, `validate.md:18` `H(ℓ⃗ ‖ Δφ* ‖ t)`; `signal.md:31` binds `net`; `mudra:specs/identity.md:36–51` requires network, operation identity and action bound. amend `intent.md` §identity proof and `validate.md`'s signature row to bind `net` and the step.
- G5 · the value of `base_fee` and the balance it debits. the mechanism is written: one base fee per primitive under the EIP-1559 exponential rule (`cyber:whitepaper.md:1198`); `validate.md:23` debits `focus(ν)`, the chain keeps `NeuronRecord.focus` at zero (`chain.md:462`) and pays in the native coin, and no clause gives the value (`soft3:proposals/network-repos.md:108` makes such values network data). amend `validate.md`'s focus row.
- G6 · the pool. `privacy-frozen.md:38` and `chain.md:1196`, `:1219–1221` write a pool link as `(p, q, weight, payout)`, the weight outside the value equation, against the 5-tuple whose box is conserved (`cyberlink.md:3–13`; `validate.md:22`); the author of a spend stays hidden (`privacy-frozen.md:38`) while every signal carries its neuron's signature (`cybergraph.md:41`). amend `privacy-frozen.md` §1.6 and `chain.md` §11.6 for the link (this half binds genesis, §6); `cybergraph.md` A2 for the hidden author, by rule set 2.
- G7 · intents on the chain. `intent.md:34–36`, `:94–96` keep every intent on record at its inception height; bbg keeps intent records outside its root (`native-state.md:66`) with a 64-byte signature (`native-state.md:59`) where NSIG1 is 102 bytes (`neuron-auth.md:11–17`); no clause puts intents in the block. amend `bbg:specs/state.md` and the chain's block before intents span blocks.
- G8 · the revert by key. the P9 draft adds a signed revert statement (cybernode#12, §5.6) beside the signed checkpoint (`chain.md:724–748`): two keyed formats for one power, the veto of blocks. amend `chain.md` §5 to one keyed format.
- G9 · time. `order.md:18` requires a VDF; `chain-economy.md:27–30` retired it from the root chain, where blocks ordered by work carry the height; `validate.md:25` hands ordering to the chain. amend `order.md`'s temporal row for a network ordered by work.
- G10 · the field that names an existing box. `box.md:37–38` names withdraw and spend, `validate.md:20–21` makes a live, owned box a σ condition of a signal, `box.md:88` names bbg's `BoxMove { nullifier, commitment }`; the signal's fields (`signal.md:7–17`) hold links only. amend `signal.md` §structure.
- G11 · the root chain's network card. a network is a card in the cards dimension (`network.md:12–23`); the root chain names itself by its genesis particle (`chain.md:118`; `network-repos.md:106`), which no card record holds. amend `network.md` or `chain.md` §4.4.
- G12 · the bridge as a signal. the bridge is "the one place a classical signature is structurally required", verified natively in phase 1 (`mudra:specs/bridge.md:18–21`, `:80–84`), and `chain.md:410–414` keeps it outside the chain while kind 3 moves the dormant balances; how the binding rides a signal is unwritten. amend `chain.md` §2.8 with `bridge.md`.
- G13 · telling the four address links apart. `address.md:7–16` makes them ordinary links from the name; `relation.md:12–23` types a link on its axon; no clause says which relation words FOLLOW, ANTENNA, SOCKET and LOCUS are. amend `address.md`.
- G14 · the genesis seed. `neural:specs/dialect.md:22` seeds TRUE and FALSE alone; `relation.md:47–53` adds the relation words and the image schemas. amend one of the two.

## 6. pussy's genesis — the owner's question

the genesis file holds the `p` frame with the pool record, the `b` balances and the `l` links (`chain.md:565–600`, `:1337–1352`); the chain id is its particle (`chain.md:569–570`). the `l` frames already apply as cyberlinks `(p, q, τ_native, 0, 0)` by their neuron at height 0 (`chain.md:593–598`); `legacy_owner` and the naming links of the genesis neuron are genesis data and stay (`chain.md:416–422`, `:646–655`). the messages live outside the file: they change by a rule set at an announced height (`chain.md:788–806`). either way G6's link half binds the file: format 1 freezes a pool link outside the 5-tuple into pussy; a 5-tuple link is a new pool format, moving the pool record, the chain id and the pins once.

the question, on [soft3#677](https://github.com/cyberia-to/soft3/pull/677):

| option | what happens | measured consequence |
|---|---|---|
| A · launch on today's kinds | pussy starts at the genesis time of `chain.md:687–689` with rule set 1 as written; signals come by a later rule set announced at least 40,320 blocks ahead (`chain.md:803–804`) | the canary's public history keeps the typed kinds; cyb's signer and the simulation carry both forms through the switch |
| B · wait for the signal rule set | genesis waits until G1, G2, G4, G5, G10–G12 have clauses and the node carries signals | the modules that dispatch on kinds, on cybernode `release/phase1` `599cd85`: `net/codec.rs` 898 lines (68 kind matches), `tx.rs` 211 (16), `net/mempool.rs` 528, `net/discovery.rs` 330, `corpus.rs` 460, `engine_tx.rs` 226, with tests and vectors, cyb's signer and the simulation; the date is unknown while those gaps are open |

recommended: A. B's date waits on clauses whose owners — soft3 schema, cybergraph, mudra — have not written them, and code written ahead of them would invent, which is the fault the owner found in the first draft; A changes nothing frozen beyond the G6 decision that B needs as well. §2.4 follows A.
