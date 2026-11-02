---
tags: cyber, soft3, proposal, chain, cybernode, cybergraph, neural, signal, cyberlink, lexicon, genesis, pussy
crystal-type: process
crystal-domain: cyber
alias: one message, one kind of message, signals only, protocol lexicon, the chain speaks links
status: proposed
date: 2026-11-02
---
# one message — the root chain carries signals of cyberlinks, and neural carries the rest

the owner, 2026-11-02 (translated): "how did we end up here? what kinds of transactions do we have now? I remind you: in my design of soft3 there is only one kind of message — links — everything else is the semantics of neural. Write a proposal to fix this." and, the same day: "there are no transactions, there are signals" (director plan row P12; vocabulary row in cyberia `dev.md`).

this page answers both questions and fixes the design. it is the structural half (b) of P12 and depends on [[signal]], [[cyberlink]] and [[box]] in cybergraph, on [[neural]]'s [[word]], [[lexicon]] and [[dialect]], on [[proven-present]] decision 9 and on the [root chain spec](../../cybernode/specs/chain.md) with [chain-economy](../../cybernode/specs/chain-economy.md) and [chain-wire](../../cybernode/specs/chain-wire.md), all read at cybernode `origin/main` `52538a6`.

## 1. the principle

a [[neuron]] says one thing to the chain: a [[signal]], a bundle of [[cyberlink|cyberlinks]] committed in one step under one authorization. each link is the cybergraph 5-tuple (p, q, τ, a, v): two [[particle|particles]], a [[box]] of token τ and amount a, a valence. what an act means — a payment, a stake, a claim, an address — is carried by which particles the links name. a small set of words, frozen at genesis, has a meaning the state transition enforces; every other word gets its meaning from the graph, ranked by [[focus]].

why it matters:

- one validation path. decode one envelope, verify one authorization, apply links in order, each protocol word with one rule. today there are eight decoders, eight size tables and three ways to name the signer (`from`, `neuron_of(pubkey)`, none).
- one proof relation for the proven present. the step relation of [[proven-present]] becomes "apply a signal", one relation with a word dispatch, instead of one per kind. a later rule set adds a word to that relation, never a kind (proven-present §2.11). one authorization per signal also spreads P5's measured cost over many links (§3.5).
- one format for every network type: the root chain, home books ([[network-repos]]), private networks (`net` of signal.md) and cyb's books carry one object. bbg already keys a `signals` dimension by `signal_key(neuron, step)`; the chain leaves it empty today.
- neural is where meaning grows without forks. addresses, follows, passports and labels are already plain links (chain-wire §6: `hemera("antenna")`, `hemera("socket")`, `hemera("follow")`). only an act that moves value or consensus state needs a protocol word; adding one is a rule set, as a kind is today, but the envelope, decoder, mempool, indexers and proof relation keep their shape.

## 2. what exists today

| kind | name | bytes | fields | signer | state it moves | specified | entered |
|---|---|---|---|---|---|---|---|
| 0 | coinbase | 82 | height, to, amount, extra[32] | none (the block's work) | mints `SUBSIDY − cut(h) + fees` to `to`, `cut(h)` to `focus_key` | chain §2.2, economy §6.2 | 2026-10-15, soft3#182 (`824b8ab`) |
| 1 | transfer | 224 | from, nonce, token, amount, to, fee | NSIG1 of `from` | balance `from → to`, fee | chain §2.3 | 2026-10-15, soft3#182 |
| 2 | cyberlink | 257 | from, nonce, token, amount, p, q, valence, fee | NSIG1 of `from` | axon lists; `amount > 0` → `stake_key(card)`, card record | chain §2.4, economy §3.3 | soft3#182 with `amount = 0` only; stake 2026-10-17, soft3#225 |
| 3 | claim | 140 | account_id[20], nonce, fee | NSIG1 of the legacy key | every balance of `legacy_owner` → `neuron_of(pubkey)` | chain §2.8 | 2026-10-15, soft3#182 |
| 4 | deposit | 384 | from, nonce, token, value, owner[64], r, data, predicate[64], fee | NSIG1 of `from` | balance → turnstile `T(τ)`, commitment appended | chain §11.7 | 2026-10-16, soft3#193 (`c79706a`), rule set 2 |
| 5 | spend | ≤ 9,985 | len, spend_public | none (the SPEND proof) | nullifiers, commitments, withdrawal, public cyberlinks | chain §11.6–11.7 | 2026-10-16, soft3#193, rule set 2 |
| 6 | unlock | 224 | from, nonce, p, q, amount, fee | NSIG1 of `from` | `stake_key(card)` → balance | economy §3.4 | 2026-10-17, soft3#225 (`6f9cb76`, `bc1cbf6`) |
| 7 | payout | 248 | from, nonce, p, q, payout, fee | NSIG1 of `from` | `card.owner` | economy §3.5 | 2026-10-17, soft3#225 |
| 8 | ticket | reserved | E, cluster, n, miner, m(n), proof | — | settlement sample | economy §7.3 | 2026-10-17, soft3#225 |
| 9 | fold step | reserved | unspecified | — | — | economy §3.5 | 2026-10-17, soft3#225 |
| 10 | report | reserved | from, nonce, p, q, prediction, fee | NSIG1 | BTS input | economy §7.4 | 2026-10-17, soft3#225 |
| — | revert statement | 191 | version, chain, height, keep, counter, anchor_time, sig | NSIG1 of `checkpoint_key` | rewinds the block tree | chain §5.6 draft (cybernode#12), code cybernode#15 | 2026-11-02, P9c |
| — | envelope v2 (draft) | + auth | kinds 1, 2, 6, 7 with an AUTH1 proof | proof of the secret | — | chain §2.9 draft (cybernode#11) | 2026-11-01, P5 |
| (2) | address announcement | 257 each | (antenna or socket or follow, q) | NSIG1 | topology only | chain-wire §6 | 2026-10-15, soft3#182 — already a cyberlink |

how the chain drifted. on 2026-10-15 the owner cut phase 1 to "pure PoW, public chain, no features — mining, transfers, cyberlinks" (cyber `launch.md`). the launch page turned that into "two transaction types (transfer, cyberlink) … parano1d compactness (fixed types, no scripting, a block budget)", and the spec merged the same day shipped four kinds by the typed-envelope template of [[goldilocks-parano1d]] and Bitcoin: one fixed-size record per action. chain §2.4 records the moment the question came up. "why two neuron-signed kinds and not one" was answered by state footprint and ease of indexing, and nobody checked it against signal.md or box.md ("creating a cyberlink is a transaction: the author moves a box"). after that, each new need took the next number by the same template: the pool's formats (4, 5) on 10-16, the economy's unlock and payout (6, 7) and three reserved slots on 10-17, an auth envelope on 11-01, a revert format on 11-02. every step was locally reasonable, and none of them asked whether it was a new word or a new kind. the same spec set had shown the neural way all along: its address announcements are links with word particles.

## 3. the design

### 3.1 the envelope

```
signal = version: u8 (= 2) ‖ neuron[32] ‖ step: u64 ‖ fee: u64
       ‖ n: u8 ‖ link × n ‖ plen: u16 ‖ payload[plen] ‖ scheme: u8 ‖ auth
link   = p[32] ‖ q[32] ‖ τ[32] ‖ a: u64 ‖ v: i8                     // 105 bytes, the cybergraph 5-tuple
scheme = 0 work (coinbase only, empty auth) | 1 NSIG1 (102 bytes) | 2 AUTH1 proof (P5) | 3 pool proof (rule set 2)
id     = hemera("soft3/chain/signal/v1\0" ‖ chain ‖ signal)
stmt   = hemera("soft3/chain/signal-sign/v1\0" ‖ chain ‖ signal without scheme and auth)
```

`net` of signal.md is the chain id, bound in `id` and `stmt` as today. `step` is the neuron's nonce, one per signal. h₀ = h₁ = the block height; Δφ* stays empty in phase 1 (settlement computes it). the scheme byte says how ν proves it is ν; P5 shrinks the schemes toward one. `payload` is opaque except where a protocol word fixes its layout. links apply in order, the signal is atomic, protocol links carry `v = 0`. the fee, `≥ len · MIN_FEE_PER_BYTE`, is debited from ν after the links apply, so a claim or a withdrawal pays from what it brings (economy §3.4's unlock rule, made general).

sizes, derived from the formats: one link with NSIG1 is 260 bytes (transfer today 224, cyberlink 257); four links 575 against 1,028; sixteen 1,835 against 4,112.

### 3.2 the protocol lexicon

a protocol word is `hemera(word bytes)`, the spelling chain-wire §6 already uses. it is enforced in the p position of a link, and a link that names it as q is an ordinary link. the words follow box.md's lifecycle (create, transfer, withdraw, spend) wherever box.md has a word. they join TRUE and FALSE as [[dialect|bootloader dialect]], with the difference that the state transition enforces their meaning.

| word | link | rule | rule set |
|---|---|---|---|
| any other p | (p, q, τ_native, a ≥ 1, v) | a position: stake on `card(ν, p, q)`, economy §3.3 unchanged | 1 |
| `transfer` | (transfer, to, τ, a, 0) | τ a token: a·τ from ν to `hemera(to ‖ τ)`. τ a card authored by ν with a = 1: `card.owner = to` | 1 |
| `withdraw` | (withdraw, c, τ_native, a, 0) | `c = card(ν, p, q)`, `MIN_LOCK` passed, `0 < a ≤ stake`; stake → ν; card deleted at zero | 1 |
| `claim` | (claim, legacy_owner(hrp, id), τ, a, 0) | NSIG1 whose `cosmos::account_id(pubkey) = id`; `0 < a ≤` dormant balance of τ; moves to ν | 1 |
| `coinbase` | (coinbase, miner, τ_native, a, 0) | signal 0 only, ν = the protocol neuron `[0; 32]`, step = height, scheme work, payload = extra[32]; `a = SUBSIDY − cut(h) + Σ fees`; maturity as §2.2 | 1 |
| `deposit` | (deposit, pool, τ, value, 0) | payload = owner ‖ r ‖ data ‖ predicate (192 bytes); rho = `W(D_RHO ‖ id ‖ 0)`; a·τ → `T(τ)`, commitment appended | 2 |
| `spend` | (spend, pool, τ_native, fee, 0) and the spend's public links | ν = pool, step 0 (the nullifiers close replay), scheme pool, payload = spend_public | 2 |
| `ticket`, `fold`, `report` | (ticket, cluster, …), (report, c, τ_native, bond, 0) | their rule sets, economy §7.3–§7.4 | later |

`pool` is the particle of the 74-byte genesis pool record (§11.10).

### 3.3 the mapping decisions

- transfer carries a word because the canon is ambiguous without one: cyberlink.md reads (neuron, particle, CYB, 500, +1) as a stake and (neuron, neuron, card, 1, 0) as a transfer, and the chain never tells a neuron id from a particle. payout is box.md's "transfer: the box → a new owner" with the card as the box (cyberlink.md's non-fungible case); the author keeps the unlock right, as economy §3.5 does.
- unlock is the word `withdraw`. a reverse box cannot be written: a is a u64, and v is the BTS prediction (the funded short). q is the card id, author included.
- the legacy claim is a link from the word to the dormant owner, authorized by the old key's NSIG1 now and by an in-circuit secp256k1 proof later; the P5 receipt shows a preimage proof authenticates nothing for a legacy key. one link per token; claim and onward transfer to a new neuron fit in one signal, closing chain §2.8's two-step path.
- the coinbase is a signal of the protocol neuron authorized by the block's work: the header stays 168 bytes, the PoW template unchanged, the body one decoder; the protocol neuron already authors genesis's naming links. a header field would change the mined bytes for nothing.
- address announcements stay ordinary links: antenna, socket, follow, locus, passport are a node dialect that nodes and cyb read and consensus ignores. under P11 they are 1-unit positions; an address costs one signal of two links (365 bytes) instead of two of 257.
- pool: the record, commitment, nullifier, tree, turnstile, block statement and tags of §11 stay; only the envelope changes. one field does not fit: spend_public writes links as (p, q, weight, payout), weight outside the value equation, no box, no valence (Q5).

### 3.4 interactions

- P11 (`a ≥ 1`): it holds for every link with no exemption. each protocol word moves a box: a payout moves its card with a = 1, and later words carry a bond. genesis `l` frames keep `a = 0`, as P11 already decides.
- P12: its rename (a) is the vocabulary of §3.1: signal id, `signals_root`, topic `signals`, `/signal`, `chain signal`. this page is its structure (b2) plus the lexicon that turns kinds 0, 1, 3 and 4–10 into words. (b3), σ over the whole signal, follows by rule set.
- P5: one authorization per signal. the measured AUTH1 envelope is 128,389–131,251 bytes (`audit/chain/proof-auth-2026-11.md` §7); today two proof-authorized kinds fit in a block. by arithmetic from those figures, one proof-authorized signal carries up to about 1,240 links (`MAX_BLOCK_LINKS` caps it at 1,024).
- P9: the revert statement stays a statement about the block tree, beside headers and checkpoints. a signal in a block above H would be erased by the revert it carries. it keeps P9c's 191 bytes and its own tag, and may take the scheme byte so P5 can retire NSIG1 there too.
- budget: `MAX_BLOCK_TXS` becomes `MAX_BLOCK_LINKS` = 1,024, counted over every link including the coinbase. this keeps economy §7.2's bound on positions per epoch. `MAX_TX_BYTES` becomes `MAX_SIGNAL_BYTES`, sized by `MAX_LINKS_PER_SIGNAL` (Q6).
- conformance: the kind vectors become signal vectors with one case per word, and `chain.snap` is re-blessed.
- genesis: the `b` frame is a box and the `l` frame is a link, read as height-0 signals of the protocol neuron. the file does not change for this page. dormant balances stay under `legacy_owner` for `claim`.

## 4. what changes, and the path

| where | what | size (estimated) |
|---|---|---|
| cybernode `specs/chain.md` | §0, §1.6–1.7, §2 rewritten as signals and the lexicon; §3.4, §5.5, §11.6–11.7 | ~350 lines |
| `chain-economy.md`, `chain-wire.md`, `chain-conformance.md`, `genesis.md` | §3.3–3.5, §6.2, §7.3–7.4; topics, mempool, §6; vectors; wording | ~150 lines |
| cybernode `rs/src/chain` | `tx.rs` (211 lines today) → `signal.rs`; `apply.rs` (521) word dispatch; `engine_tx.rs`, `sigs.rs`, `net/codec.rs`, `net/mempool.rs` (528), `net/discovery.rs`; `wallet/`, `node/chain_routes.rs`, conformance, tests | ~2,000 lines changed |
| siblings | simulation (every consumer, the P9b lesson), cyb's signer and explorer | ~300 lines |
| cybergraph, neural | signal.md: the root chain's profile; dialect.md and lexicon.md: the protocol lexicon as bootloader | ~60 lines |

phase A, before pussy's genesis, in one re-bless with P9 and P12: the envelope, NSIG1 and work, `transfer`, `withdraw`, `claim`, `coinbase`, positions, P11, P12's rename; `deposit` and `spend` written, unknown under rule set 1. the genesis frames stay; the pool record changes only if Q5 picks format 2, and then the chain id moves with P9's key, so P4's pins move once.

phase B, by rule set: the pool words active, AUTH1 as scheme 2, the `signals` dimension written per signal (one trie update; bbg measured 0.4–0.9 ms per link insert), the legacy claim proven in circuit once trident's secp256k1 is real (a stub at `8f5c1a7`).

phase C: `ticket`, `fold`, `report`; σ over the whole signal as the step relation of [[proven-present]]; Δφ*.

measured: the kind sizes (spec), the AUTH1 envelope (P5 receipt), the bbg insert cost. estimated: every new size above (derived from the formats), the line counts (read from module sizes on cybernode `release/phase1`), and the schedule. phase A is about two days of focused work plus the gates. it fits 11-05 only if it starts on the owner's yes today.

## 5. open questions for the owner

1. when: phase A at pussy's genesis with the P9/P12 re-bless, or P12's rename alone (b1) and phase A as the first rule set after. default: at genesis if its PR is green by the re-bless, else b1. in neither case a third envelope.
2. spelling: plain `hemera(word)` as chain-wire uses, with the p position reserved, or domain-tagged word particles that leave the plain words free. default: plain, so one word is one particle.
3. names: box.md's `transfer` and `withdraw`, or the chain's `transfer`, `unlock` and `payout`. default: box.md's, with payout as a card transfer.
4. coinbase: a signal of the protocol neuron authorized by work, or a miner field in the header. default: the signal.
5. pool links: re-freeze §11.6 as format 2, so a spend's links are boxed 5-tuples inside the value equation (no verifying key exists yet), or keep format 1 and read weight as an unboxed link exempt from P11. default: format 2, in the same re-bless.
6. links per signal: default 64 under NSIG1 (6,875 bytes); under AUTH1 only the block budget bounds it.
7. revert: a statement outside the signal stream, or a signal of the revert key replayed into the new branch. default: the statement.
