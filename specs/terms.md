---
title: terms
tags: cyber, soft3, spec
crystal-type: spec
crystal-domain: cyber
status: draft
alias: vocabulary, glossary, soft3 terms, lexicon, definitions
---
# terms

the canonical vocabulary of the [[soft3]] stack. one name per concept, grouped by layer. companion to [[types]] (the type theory) and [[cybergraph]]/specs/model (the graph ontology). where a term has a fuller spec, the definition here is the one-line anchor.

## substrate — the data model

the value layer every language lowers to. one leaf, one join; everything else is recursion.

- `field` — one Goldilocks element, `[0, p)`, `p = 2⁶⁴ − 2³² + 1`. the scalar substance. **8 bytes.**
- `atom` — a leaf: one `field`. the indivisible unit. **8 bytes.**
- `pair` — two children joined, each an `atom` or a `pair`. the structural constructor. its size is its children's; the smallest pair, two atoms, is **16 bytes**.
- `data` — an `atom` or a `pair`. any structure, **`8·N` bytes** for `N` leaves. the value. homoiconic: a program, its input, and its result are all data.
- `particle` — the *global* identity of data: its **32-byte** [[hemera]] hash (itself 4-atom data). content-derived and immutable — the same data always has the same particle, distinct data distinct particles. (within one execution a node also has a *local* identity, its `order` — see *computation*.)
- `link` — two `particles` joined, a `from → to` pair: **64 bytes**. the structural skeleton of a relation (a [[cyberlink]] is built on it — full definition under *graph — tokens and links* below).
- `file` — a piece of `data` identified by its `particle` and reachable by a `name`. the [[fs]] unit. data carries no label inside it: its identity is its particle (computed from content), its name is assigned separately.

the size ladder, counted in 8-byte `field`s — `atom` 8 · `pair` 16 · `particle` 32 · `link` 64. the encoding is **length-discriminated and tag-free**: structure is read from length, and the leaf/node distinction lives in [[hemera]] capacity (a substrate invariant), never in a tag byte inside the data.

note: data is the thing; its `particle` is its identity; a `name` is a separate, mutable label that points to it. every particle is data (a 4-atom one); not all data is a particle.

## identity and naming

content-derived identity versus assigned, mutable names — two different things often confused.

- `identity` — a content-derived address: `hemera(content) = particle`. immutable, unique, signatureless (`cybergraph`/specs/identity). a `neuron`'s identity is `hemera(secret)`; a `particle` is `hemera(data)`. distinct content always yields distinct identity.
- `name` — a mutable label resolved by supersession: the latest `cyberlink` by a `neuron` under a path (`inf`/specs/relations). names point to particles and change over time — move, rename, update are new cyberlinks under the same path. many names may point to one particle; a particle never changes.

## types — how values are typed

full theory in [[types]]. a value is typed by one of five mechanisms; the stack uses two, reserves one, drops two.

- `refinement type` — a value plus a proven proposition: `{ x : field | P(x) }`. identity-neutral, discharged as a constraint. e.g. `word`, `bool`.
- `nominal type` — a distinct referent carried as structure: `pair(domain, value)`. identity-distinct; the domain marker is a particle.
- `word` — a `field` proven in `[0, 2³²)`. bitwise / comparison unit.
- `bool` — a `field` proven in `{0, 1}`.
- `capacity` — a [[hemera]] sponge lane used for substrate invariants only (leaf/node/root). never for types.
- `grammar` — `field` + `pair`. the value model itself, not a type system. the kernel is type-agnostic.

## hash, commitment, proof

- `hemera` — the stack's one hash (Poseidon2 over Goldilocks, 4-element output). computes every `particle`. one pure function — no keyed/derive variance.
- `lens` — the polynomial commitment. commits data (as the multilinear polynomial over its leaves); `axis` opens in O(1).
- `zheng` — the proof system (SuperSpartan + Brakedown). a nox trace is its witness; running and proving are one act.
- `strata` — the algebra floor: four tiers × five algebras every proof reduces to.
- `digest` — a target-boundary alias for a fixed-width hash on a non-Hemera side target ([[trident]] only, `[Field; D]`). never used in the core, where the word is `particle`.

## graph — tokens and links

the cybergraph ontology: two objects, one primitive (see [[cybergraph]]/specs/model).

- `token` — the only value-carrying object. two standards: `coin` (TSP-1, fungible) and `card` (TSP-2, unique).
- `coin` — a fungible token; conserved by `Σ balances = supply`.
- `card` — a unique token; conserved by `owner_count(id) = 1`. particles and neurons are cards with skills.
- `cyberlink` — the only action: moves one token between two tokens. a record `(from, to, token, amount, valence)` — structural + economic + epistemic in one. a Layer-1 record, more than a `link`.
- `link` — a bare relation between two `particles` (a `from → to` pair). the structural skeleton a `cyberlink` is built on.
- `signal` — a signed, staked batch of `cyberlinks`; carries the `subject` (neuron) and block height.
- `box` — what persists between two cyberlinks: a token holding. private (encrypted on chain) or public.

## graph — actors and weights

- `neuron` — an identity `card` (id = `hemera(pubkey)`). the actor: signs cyberlinks, runs progs, bears karma and focus. the identity is the wallet.
- `particle` (knowledge card) — a `card` whose id is `hemera(content)`; accumulates conviction from incoming cyberlinks.
- `subject` — the `neuron` that asserts (signs) a cyberlink. an actor, itself a particle.
- `object` — the data a `formula` runs on ([[nox]] term). the acted-upon.
- `valence` — the epistemic prediction on a cyberlink: `+1` affirm / `0` agnostic / `−1` challenge.
- `stake` / `amount` — the conviction backing a cyberlink (a `field`).
- `focus` (φ*) — the equilibrium attention distribution over particles.
- `karma` — accumulated reputation of a neuron.
- `cyberank` — token-weighted rank of a particle; the graph's PageRank.

## graph — programs and validation

- `skill` — a composable hook adding behavior to a token (knowledge, identity, conviction, staking, …). skills install into `plumb` slots and compose.
- `dialect` — the rules that validate a class of cyberlinks. applications are dialects, not contracts; the graph is the state.
- `prog` — autonomous behavior installed on a neuron: listens for events, reads state, emits cyberlinks. runs in [[rune]] (dynamic) or [[trident]] (provable).
- `plumb` — the unified validation system: WHO (auth), WHAT (conservation), HOW (hooks). proving ownership of the `from` token is the only authorization.
- `bbg` — authenticated state: one polynomial, ten dimensions, ~200-byte query proofs.

## computation — nox

- `object` — the data environment of a reduction (the input). Nock's "subject," renamed.
- `formula` — the program applied to an object. itself data (homoiconic).
- `reduction` — one execution: a neuron applies a formula to an object under a budget, producing a result. it holds every `data` node it builds; the trace is the proof.
- `order` — a `data` node's *local* identity inside a `reduction`: its slot. the local twin of `particle` — `particle` is global (content-derived, identical in every reduction), `order` is local (a position, valid only inside its own reduction).
- `pattern` — one of nox's 18 reduction rules (16 compute + `call` + `look`).
- `jet` — an accelerated implementation of a hot formula (hash, ntt, merkle, …).
- `look` — the pattern that reads committed [[bbg]] state.
- `call` — the pattern that injects a non-deterministic witness (the proving hint point).

## languages

all lower to [[nox]] data.

- `trident` — the compiled, STARK-provable language. `.tri → .nox`. write once, prove anywhere.
- `rune` — the dynamic, async, hot-reloadable runtime. default for most neurons.
- `inf` — the Horn-clause query language over the cybergraph.
- `nox` — the proof-native VM; the substrate all languages target.
- `neural` — the semantic language: meaning is an eigenvector of the graph.
- `cybermark` — the markup language; the markup is the graph.
- `eidos` — the proof/kernel language.

## components

the nineteen repos, each the substrate specialized to one job (verb in parens).

- `strata` (math) · `hemera` (hash) · `lens` (commit) · `trident` (compile) · `nox` (run) · `zheng` (prove) · `cybergraph` (link) · `bbg` (store) · `tru` (converge) · `glia` (infer) · `mir` (render) · `mudra` (encrypt) · `radio` (transmit) · `tape` (frame) · `foculus` (sync + agree) · `soma` (think) · `conformance` (snapshot) · `rune` (eval) · `fs` (mount) · `plumb` (pay).

## naming decisions

resolved this design session:

- `noun → data` — the value-model term. "code is data" is homoiconicity; `data` says it plainly and everyone already knows the word. `data model` over `noun model`.
- `cell → pair` — the substrate join. `link` stays for relations between `particles`; `cell` stays free for higher-level apps; `pair` is the simple structural word with no clash.
- `subject` / `object` stay as actor (neuron) and data — not repurposed as the two halves of a `pair`/`link`, to avoid colliding with the graph's `subject = neuron`.
- `particle`, not `digest`/`Cid`, in the core (mono-hash). `digest` lives only at trident's target boundary.
</content>
