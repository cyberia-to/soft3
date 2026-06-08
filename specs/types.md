---
title: types
tags: cyber, soft3, spec
crystal-type: spec
crystal-domain: cyber
status: draft
alias: type theory, type mechanisms, typing, refinement types, nominal types, five mechanisms
---
# types

how any part of the [[soft3]] stack types its values. one uni-typed grammar at the bottom, every type above it. this is a stack-wide convention, not a component feature: [[nox]], [[inf]], [[trident]], [[rune]], [[cybergraph]], [[tape]] all type the same way, over the same grammar.

a value can be given a type in exactly five ways. the stack uses two of them, reserves one for substrate invariants, and refuses two. the result is a kernel with no type system at all, and two open type systems living above it for two different jobs.

## the grammar is not a type

every value is a [[noun]]: `noun = atom | cell`. an atom is one [[field]] element; a [[cell]] is an ordered pair. this is a minimal complete basis — each irreducible, neither derivable from the other, together expressing everything. a [[particle]] is a 4-leaf tree; a list is a spine of cells; a record is nested cells.

`field` and `cell` are the grammar of values, not a type system. the reducer must tell a leaf (do field arithmetic) from a branch (navigate) to evaluate at all, and no proposition can replace that — a proposition is a claim about an already-structured value, so the structure must exist first. at the commitment layer the distinction thins further: a noun is a multilinear polynomial over `field`, and atom-vs-cell is just zero-vs-some variables. one object, committed by [[lens]] regardless of shape.

so the kernel is uni-typed: it knows one thing, the noun. it imposes no type system, which is what lets each language above bring its own.

## the five mechanisms

a type distinction can be made manifest in exactly five places.

| mechanism | type is… | where it lives | open | in identity |
|---|---|---|---|---|
| framing | serialization metadata | the wire bytes (a prefix tag) | closed | by accident |
| capacity | a crypto domain | the [[hemera]] sponge capacity | closed, governance-scarce | yes |
| predicate | a proposition | the constraint system / proof | open | no |
| structural | data | the noun itself (a `cell`) | open | yes, legitimately |
| primitive | a kernel variant | the value model | closed | optional |

the deep rule behind the table: identity must come from semantic content, never from a serialization choice, and a type distinction belongs in identity only when confusing two same-content values would forge a proof.

## the allocation

two dropped, one reserved, two open.

### dropped — framing

a framing tag makes identity an artifact of how bytes were written rather than what the value is. it conflates serialization with semantics. [[hemera]] already states the principle for hashing: type is "not prepended to the input." framing is refused everywhere except transport length-framing, which is a property of a byte stream (see [[tape]]) and never touches identity or storage.

### dropped — primitive

`field` and `cell` are the grammar, not types (above). beyond them the stack adds no kernel-level type variants. every type with distinct proof obligations would otherwise want its own variant — `u8`, `u16`, signed, char — each a new hash domain, cost class, and cross-layer name, with no principled stopping point. the kernel stays type-agnostic; types live above it.

### reserved — capacity, for invariants only

capacity binds a distinction into cryptographic identity. it is the right tool for one job: substrate invariants that must never be confused, such as leaf / node / root in an authenticated tree, where confusing two same-content values forges an inclusion proof.

the criterion is invariance, not importance, ubiquity, or even security: is this a truth of the system that can never legitimately change? leaf-vs-node is a structural invariant of authenticated data, forever true — it earns capacity. ordinary concepts (`word`, `port`, `timestamp`, even `particle`) are conventions, and conventions evolve, so they live above. time is the litmus: you would never weld "this is a timestamp" into the cryptographic constitution, and time being the hardest case makes it the yardstick for the easy ones. consistent with the stack's model of time as data accreting in the [[cybergraph]], not a constant in the hash.

capacity is bit-abundant (one Goldilocks lane is 2⁶⁴ values) but governance-scarce: each entry is a system-wide frozen constant, changed only by a fork that shifts every identity. spend it on a handful of invariants, ever.

## the two open type systems

types live above the grammar, in two systems for two kinds of type. both need nothing beyond the grammar and the proof layer the stack already has.

### refinement — type as proposition

a refinement is a base value plus a proposition it satisfies, identity-neutral:

```
word    = { x : field | 0 ≤ x < 2³² }
bool    = { x : field | x·(x−1) = 0 }
nonzero = { x : field | x ≠ 0 }
port    = { x : field | x < 65536 }
```

this is the studied theory of refinement types, and it lines up with [[type theory]] point for point:

- propositions as types ([[Curry–Howard]]): a typed value is a proof. `word` is the proposition `x < 2³²`; the witness satisfying its range constraint is the proof. a proof-native machine makes this literal — types and proofs coincide, and types are not erased, they are the proof.
- the decision procedure: refinement systems need one to discharge the predicate. the stack uses the [[zheng]] prover / constraint system where others use an SMT solver. a predicate is accounted as a constraint, paid where the value is used — often one already owed (the bit-decomposition `xor` needs is itself the range proof, so `word` for `xor` is free).
- subtyping by implication: `word <: field`, `port <: field`.
- decidability: arbitrary predicates are undecidable; the discipline is a tractable fragment. [[nox]]'s bounded locality, bounded recursion, and finite budget are that fragment — they make every refinement's obligation decidable and bounded-cost.

a refinement leaves identity alone: `word 5` and `field 5` are the same value, the same [[particle]], one carrying a proof it is small.

### nominal — type as data

a nominal type is a distinct kind of thing that happens to share content with another. it is not a constraint; it is a different referent. it is expressed as structure — a `cell` pairing the value with a domain-marker that is itself a [[particle]]:

```
count 5      = 5                    id = H(5)
timestamp 5  = cell(TIMESTAMP, 5)   id = H(cell(TIMESTAMP, 5)) ≠ H(5)
```

different content gives different identity, automatically, through ordinary content-addressing — no special mechanism. and the marker is a graph node, so a nominal type is itself a [[particle]]: a typed value is a cyberlink-shaped pairing of the type-particle with the value, and types live in the [[cybergraph]] as nodes you can link to, define, and version.

this is the newtype pattern, native to the grammar, and it is strictly better than capacity for the job: cryptographically distinct (the wrapper is content, so it is in the hash), open (any domain, no fork to add one), linkable, composable (`cell(TIMESTAMP, cell(UTC, 5))`), and opt-in.

a framing tag and a structural marker look alike but differ in kind: framing is metadata, a byte in a closed codec; a marker is data, a particle in an open vocabulary. the litmus is — can you cyberlink to the type? data yes, metadata no. that is why a marker lives legitimately in identity while a framing tag does not.

## identity reflects content, ignores refinement

the choice "same identity or different identity" maps exactly onto "refinement or referent":

- identity ignores refinement — `word 5 == field 5`, same referent, constraint re-proven at use.
- identity reflects structure — `timestamp 5 ≠ count 5`, different referent, different content.
- capacity carries neither — only commitment invariants.

one rule, both cases: identity follows content; a refinement is a proposition about content, so it is invisible to identity; a nominal type is content, so it is visible. type safety is a use-site obligation discharged by a constraint; commitment safety is an identity obligation discharged by domain separation; they defend against different attacks and use different mechanisms.

## applying it across the stack

the kernel imposes no type system, so each language types as it needs, all over the same grammar and lowering to the same nouns:

- [[nox]] — the value model: `field` + `cell`, refinements for `word`/`bool`, nominal wrappers for domains. the encoding is length-discriminated and tag-free. see the nox application for specifics (`nox/roadmap/value-model.md`).
- [[inf]] — relational types over the same atoms; `particle`/`neuron` are nominal.
- [[trident]] — refinement and contract types on the surface; substrate names (Triton's) live at the lowering boundary, not the surface vocabulary.
- [[rune]] — its own discipline, same grammar.

one name per concept, owned by the value layer and imported upward; `hash` is the verb (the [[hemera]] permutation), `particle` the noun it produces. naming and the nox-specific encoding are detailed in the nox application.
</content>
