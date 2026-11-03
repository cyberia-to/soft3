---
tags: soft3, task, chain, signal
kind: task
status: doing
owner: claude
serves: pussy
lane: wide
date: 2026-11-03
---

# one message — the chain carries signals of cyberlinks, as soft3 specifies

## the claim

the root chain of pussy carries one kind of message, the signal of `cybergraph/specs/signal.md` and `intent.md`, with nothing added that the soft3 specifications do not already define; every action the chain needs today (transfer, claim, stake, unlock, payout owner, address announcement, coinbase, pool, revert) is expressed through it, its meaning is neural's (`neural/specs/`), and the proposal says so with a trace to the clause behind every sentence.

## the owner's corrections (2026-11-03, translated)

- the signal format of the first draft (soft3#677: `version ‖ neuron ‖ step ‖ fee ‖ n ‖ link × n ‖ payload ‖ scheme ‖ auth`) is not what the specifications say; the specification is cluttered and must be cleaned; nothing superfluous.
- "a bundle of cyberlinks committed in one step under one authorization" is false to the philosophy of the signal: one neuron creates the signal as an intent with provable conditions of completion; then any neurons may fulfil it, even in different blocks; a signal fits in one block only when it is minimal.
- the chain is implemented as if soft3 had no specification; the first draft piled noise on top.
- every open question of the first draft is already answered in the soft3 specifications; the work is to read them fully, make several passes and checks, and follow the agreed process.

## outputs

1. `proposals/one-message.md` rewritten from scratch on this branch (soft3#677).
2. a trace table in the proposal: every rule → the spec clause (repo, file, heading or line) it comes from; a rule with no clause is either removed or listed as a gap for the owner with the clause it would amend.
3. the list of what in `cybernode/specs/chain.md`, `chain-economy.md`, `chain-wire.md` contradicts or duplicates the soft3/cybergraph/neural specifications and must be removed.

## checks

- every sentence of the proposal that states a rule carries its clause; a reviewer of another model re-reads each cited clause and confirms it (review node, lenses of `cyberia/quality.md`, at most two rounds, findings with ids, each disposed).
- no format, field, scheme, word or number appears that no spec defines, unless listed as a gap with the reason.
- the signal is described as the specs describe it: an intent by one neuron with provable conditions, fulfilled by any neurons, possibly across blocks (intent → seal), minimal signals in one block.
- the proposal is shown to the owner only after the review is green.

## log

- 2026-11-03 · task opened after the owner rejected the first draft.
- 2026-11-03 · design node (claude): read whole at origin heads cybergraph `specs/` (all 26), neural `specs/` (all 12), soft3 `specs/` main and `release/phase1`, `schema/README.md`, proposals proven-present, steps-blocks-epochs, goldilocks-parano1d, network-repos, privacy-frozen, cybernode, state-economics; mudra neuron-auth, identity, seal; tok plumb, tsp-1, tsp-2; bbg state, native-state, storage; tru rewards §6–8; cyber cascade, intent; soma §7; cybernode main chain, chain-economy, chain-wire, genesis, chain-conformance, and the P5/P9 drafts (#11, #12). trace built first (proposal §1 table), then the page rewritten from scratch.
- 2026-11-03 · self-check (i), every clause re-opened: one wrong range fixed (`chain-economy.md:31–35` → `32–35`); every other cited line says what the sentence claims.
- 2026-11-03 · self-check (ii), nothing superfluous: the "abandoned" trace row deleted (G7 cites its clause directly); the meaning paragraph folded to one sentence; the first draft's envelope, schemes, tags, lexicon of words, sizes and seven questions are gone, none replaced; the only numbers left are measured module sizes and cited clauses.
- 2026-11-03 · self-check (iii), the owner's sentence against `intent.md`: intent by one neuron with the scope as provable condition (`intent.md:56–62`), fulfilment by any neurons in their own blocks (`intent.md:38–40`, `:94–96`, `cyber:cascade.md`), and "only a minimal signal fits in one block" (neural `wire.md:36`) now stated in that order in §1; the opening sentence had the converse only and was corrected. the remaining divergence inside cybergraph (`intent.md:23` "the neuron finalizes") is listed as G10, not decided here.

## review round 1 — 2026-11-03 · review node (a different body from the author), commit `b94677a`

method: every cited clause re-opened at origin head (cybergraph `0a70b6a`, neural `47b086c`, soft3 `3045d54`, cybernode `52538a6` and `599cd85`, bbg `2294281`, mudra `3969c7a`, tok `c773592`, tru `e660d1e`, cyber `68f1260a`, soma `1d3fe69`); a grep of the specifications for each gap. page lines are `proposals/one-message.md`.

verdict per check: 1 clause fidelity red · 2 invention red · 3 signal philosophy red · 4 gaps red · 5 removal list green (all line ranges and the measured module sizes verified; three items lack their contradicted clause, R1-9) · 6 superfluous green with nits · 7 vocabulary green with nits · 8 genesis red (R1-5) plus a question for the owner.

blocking

- R1-1 (check 3) the page does not state that the specifications themselves carry the opposite of the owner's sentence, and cleans none of it. `signal.md:3` "a bundle of cyberlinks a neuron commits in a single step", `signal.md:35` and `proof.md:9` "atomically", `cyberlink.md:23` "the entire batch", `soft3:specs/terms.md:76` "a signed, staked batch", `neural:specs/wire.md:37` "the whole signal commits or aborts", `neural:specs/cli.md:95` "an atomic batch of links by one neuron", `intent.md:62` and `README.md:72` "one transaction". G10 (page line 93) covers only `intent.md:23`. the owner's correction 2 says these must be cleaned: list each as an amendment of the gap table.
- R1-2 (check 4) G3 (page line 86) says no clause gives the card id; `bbg:specs/state.md:79–96` does: `card_id(author, p, q) = H(D_CARD ‖ author ‖ p ‖ q)`, `D_CARD = "soft3/chain/card/v1\0"`, one position per author and axon, with `CardRecord [owner, particle, author, valence, since]`. the real conflict is `cyberlink.md:77` (every record a distinct card by ν and t) against `bbg state.md:79` (records of one author on one axon are one card). restate G3 as that conflict.
- R1-3 (check 2) §4 item 2 (page line 70) "the nonce is the step of `order.md:15–17`": `order.md` does not mention a nonce, and bbg keeps both, nonce as `NeuronRecord` field under the root (`state.md:31`, `:73`) and step as the signals' key (`state.md:75`). a rule with no clause and no gap; it also moves the neurons leaf, hence the block 0 root (`chain.md:685`) pinned at genesis: §6 does not say it.
- R1-4 (check 2) §3 claim row (page line 55): "ownership of `from` is proved by the legacy key whose account id is `id`". `chain.md:420` defines the dormant owner as an id "that no key controls", and `terms.md:96` makes ownership of `from` the only authorization; `identity.md:29–32` (cited) says only that existing identifiers keep their meaning. the clause that exists is `mudra:specs/bridge.md` (holder signs `legacy address → native neuron`), uncited. an exception to the authorization rule: cite bridge.md and list it as a gap, or drop the model sentence.
- R1-5 (check 8) §2.4 (page line 44) "so pussy's rule set 1 carries minimal signals" against §6 (page line 101) "until then pussy runs today's kinds, and signals come as a scheduled rule set". `chain.md` §7.1 names today's kinds rule set 1. one page, two answers to what launches.

should

- R1-6 (check 1/3) `wire.md:36` says "one-shot for small signals", not "minimal"; no clause defines `h₀ = h₁` (page line 28), "in its own block" (line 18, 26), or "minimal". the criterion the specs give is `soma-spec.md:1094`: two-phase "when the action is a multi-step computation or invites coordination". attribute "minimal", "own block" and `h₀ = h₁` to the owner's sentence of 2026-11-03 explicitly, and quote `soma-spec.md:1094` as the test.
- R1-7 (check 1) weak or wrong cites: `execution-model.md:80–83, :97–100` (page line 60) speak of network descriptors, not of a revert entry compiled into a release (`chain.md:790–806` alone supports it); `proven-present.md:75–76` (line 44) for NSIG1 as π_id: the NSIG1 row is `:80`, "as π_id" is the page's mapping; `address.md` (line 56) says four ordinary cyberlinks and nowhere "their kind carried on the axon", the typing is `relation.md`, the combination is the page's and a gap (how FOLLOW and ANTENNA are told apart on the wire); `privacy-frozen.md:36` (G6) is "no signatures in the pool", the hidden record is `:38`; `dialect.md:22–23` (line 63) puts only TRUE and FALSE in the genesis seed, `relation.md:47–53` adds the relation words: note the divergence.
- R1-8 (check 4) G1 (line 84): `cybergraph:specs/cli.md:21–31` and `bbg:specs/cli.md:27–31` already name the canonical signal wire, tade frames (ZAP signal, KET intent, foculus codec, `foculus:src/frames.rs`); the gap is the field set, not the form. G2 (line 85): `bbg:specs/state.md:96, :105` give both landings (insert moves `H(from‖τ)` to `H(to‖τ)`; a lock moves to `stake_key(card)`); the residual is the discriminator. G5 (line 88): `cyber:whitepaper.md:1198` gives the mechanism of `base_fee` (EIP-1559 exponential rule per primitive); the missing part is the value and the balance.
- R1-9 (check 5) §4 items 1, 4 and 9 name no soft3 or cybergraph clause that the item contradicts or duplicates; item 10 is a soft3 file, not a cybernode one, and duplicates nothing in the model.
- R1-10 (check 1) no row says what carries a withdraw or a spend of an existing box (unlock row, line 54): `box.md:37–38` names the operations, `signal.md:37` covers boxes a link creates. name the carrier or list a gap. likewise the `net` of the root chain's signals (`WrongNetwork`, `validate.md:43`) is unaddressed.
- R1-11 (check 2) §2.2 "a block carries the intents declared at its height": `intent.md:36, :94` and `native-state.md:8–9` say intents are recorded and readable at inception height, not that a block body carries them.

nit

- R1-12 §6 third bullet: measured module sizes and kind-match counts (verified, 460/898/68…) are cost, no check requires them. the header says "adds no number".
- R1-13 negations: line 99 "a pool link that is not a cyberlink"; G6 "a spend shows no neuron"; line 14 "adds no format".
- R1-14 wiki-links appear only in §1 and the opening; §3–§5 tables name concepts bare.

question for the owner (check 8, not decided here): `chain.md:1351` makes the chain id commit to the pool record through the genesis file, so the messages stay outside it and only G6 binds genesis, as the page says. but the genesis time of `genesis.md` is 2026-11-05, and "pussy runs today's kinds" ships the typed-kind envelope that the page itself lists for removal, while the owner's direction is that the chain follows soft3. does pussy launch with today's kinds and move by a scheduled rule set, or does the launch wait for the signal rule set? R1-3 widens it: if the nonce leaves `NeuronRecord`, block 0's root moves.

disposition: round 1 of at most 2; the author disposes each id (fixed, rejected with the clause, or moved to the gap table) in round 2.

## round 2 — 2026-11-03 · author (design node), dispositions of review round 1

page lines are `proposals/one-message.md` after this commit.

- R1-1 fixed at `:14` (the page says the owner's sentence corrects the specifications) and `:84–97`, amendments A1–A10 old → new: `signal.md:3`, `:35`, `proof.md:9`, `cyberlink.md:23`, `intent.md:62` with `README.md:72`, `intent.md:23` (old G10), soft3 `terms.md:76`, neural `wire.md:16`, `:37`, `cli.md:95`, neural `README.md:37` (the last two lines found while checking `wire.md`).
- R1-2 fixed at `:103`: G3 restated as the conflict `cyberlink.md:77` against `bbg:specs/state.md:79–96`, both cited, the choice amends one; the stake row cites `state.md:79–96` (`:53`).
- R1-3 fixed at `:71`: "the nonce is the step" dropped; bbg keeps the nonce (`state.md:31`, `:73`) and the step (`:75`), and moving the nonce would move block 0's root (`chain.md:685`) — not proposed; `NONCE_WINDOW` left out of the removal list.
- R1-4 fixed at `:55` and G12 `:112`: the claim stated as `mudra:specs/bridge.md:16–21, :28, :46, :73–76` define it, with `chain.md:410–422`; carrying the binding as a signal listed as G12.
- R1-5 fixed at `:44` and `:116–127`: one question for the owner with options A and B and their measured consequences, A recommended with its reason; §2.4 now follows A.
- R1-6 fixed at `:18`: "minimal", "in blocks of their own" and `h₀ = h₁` attributed to the owner's sentence; `wire.md:36` "small" and `soma-spec.md:1094` quoted as the test; the trace row renamed "small signal".
- R1-7 fixed: revert row cites `chain.md:790–806` only (`:60`); NSIG1 cited at `proven-present.md:80` with π_id's type `intent.md:14` (`:44`, `:23`); address row keeps `address.md` and moves the typing to gap G13 (`:56`, `:113`); G6 cites `privacy-frozen.md:38` for the hidden author (`:106`); the `dialect.md:22` / `relation.md:47–53` divergence is G14 (`:114`).
- R1-8 fixed: G1 now names the form (tade frames, `cybergraph:specs/cli.md:21–31`, `bbg:specs/cli.md:27–31`; `:45`, `:101`) and leaves the field set as the gap; G2 cites both landings `bbg:specs/state.md:96`, `:105` and narrows to the discriminator (`:102`); G5 cites `cyber:whitepaper.md:1198` and narrows to value and balance (`:105`).
- R1-9 fixed at `:70` (item 1 against `terms.md:74`, `neural:specs/sigils.md:31–33`), `:73` (item 4 against `box.md:36–37`, `terms.md:74`), `:78` (item 9 follows items 1 and 4); the soft3 copies moved out of the cybernode list to `:80`.
- R1-10 fixed: unlock and the pool rows name the signal as carrier with `validate.md:20–21` and list the missing field as G10 (`:54`, `:110`); the root chain's `net` is a row (`:61`, `signal.md:29`, `network.md:41–51`, `validate.md:43`) and gap G11 (`:111`).
- R1-11 fixed at `:41–42`: a block carries the seals applied at its height (`state.md:75`); intents are recorded at inception height (`intent.md:34–36`, `:94–96`, `native-state.md:8–9`), and their riding the block is stated as no clause yet, in G7 (`:107`).
- R1-12 rejected in part: the measured module sizes stay, only as the measured consequence of option B (`:124`), which the coordinator's direction for R1-5 asks for; the header no longer says "adds no number" (`:14`).
- R1-13 fixed: "a pool link that is not a cyberlink" → "a pool link outside the 5-tuple" (`:120`); "a spend shows no neuron" → "the author of a spend stays hidden" (`:106`); "adds no format" → "its formats, fields, tags and parameters are the specifications' own" (`:14`).
- R1-14 fixed: wiki-links added in §2–§3 (tade, token, bridge, neural, word, axon, dialect, focus).
- question for the owner: carried as §6 (`:116–127`) on soft3#677, recommendation A.
- self-check (i) round 2: every new clause re-opened (`state.md:31, :73, :79–96, :105`, `bridge.md:16–21, :73–84`, `whitepaper.md:1198`, `network.md:41–51`, `validate.md:43`, `chain.md:410–414, :684–685, :802–804`, the ten amendment lines quoted verbatim); no range off.
- self-check (ii): nothing added beyond what a finding or the coordinator's direction required; the numbers left are cited clauses and option B's measured consequence.
- self-check (iii): §1 states the owner's sentence in its order — intent by one neuron with the scope as provable condition, open fulfilment across blocks, small signals in one block — and A1–A10 carry it into every line that said "batch", "atomic" or "one transaction".
