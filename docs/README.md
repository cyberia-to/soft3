---
title: soft3 foundations
tags: cyber, soft3, whitepaper
crystal-type: spec
crystal-domain: cyber
icon: "🪨"
alias: soft3 whitepaper, foundational soft3 methods, soft3 foundations
---
# soft3 — foundations

soft3 is twenty components. the components are not the foundation. each one — [[hemera]], [[lens]], [[zheng]], [[bbg]], [[tru]], the rest — is a specialization of the same small set of stack-wide methods to one job. learn the methods and the twenty repos stop being twenty things to memorize: they become twenty applications of four ideas.

this is the whitepaper for those ideas — the foundational methods of soft3. they are properties of the whole stack, the invariants every component obeys.

```svgbob
   +------------------------------------------------------------------------------------+
   | the 20 components  —  each a specialization of the substrate below                 |
   +------------------------------------------------------------------------------------+
   | strata . hemera . lens . trident . nox . zheng . cybergraph . bbg                  |
   | tru . glia . mir . mudra . radio . tape . sync . foculus . soma                    |
   | conformance . rune . fs . plumb                                                    |
   +------------------------------------------------------------------------------------+
                                              |
                                              | every component obeys the same four substrates
                                              v
   +------------------------------------------------------------------------------------+
   | I  .  ONE FORM              —  everything is one polynomial over one field         |
   +------------------------------------------------------------------------------------+
   | one field            every value is one goldilocks field element                   |
   | one object           state, data and proofs are one committed polynomial           |
   | five algebras        the field hosts five regimes of computation                   |
   | particle identity    a name is its content hash : a polynomial commitment          |
   +------------------------------------------------------------------------------------+
   | II  .  ONE PROOF            —  execution, programs, and the prover itself          |
   +------------------------------------------------------------------------------------+
   | proof-native         running a program and proving it are one act                  |
   | recursive closure    proofs verify proofs : all history folds to one               |
   | transparent          hash-based, post-quantum, no trusted setup                    |
   | conformance          every output fingerprinted : drift caught at commit           |
   | eidos                a proof assistant in the kernel : programs proven correct     |
   | self-hosting         the verifier is a nox program : the system closes on itself   |
   +------------------------------------------------------------------------------------+
   | III  .  ONE CONVERGENCE     —  the whole graph settles into one mind               |
   +------------------------------------------------------------------------------------+
   | global focus            one phi* distribution over every particle                  |
   | settle, do not derive   fixed points discovered, not theorems derived              |
   | tri-kernel              diffusion + springs + heat : the only local basis          |
   | one answer, four ways   phi* is consensus + rank + reward + meaning at once        |
   | speed of thought        the spectral gap is how fast the whole graph decides       |
   | stake-weighted          security is economic mass, not honest majority             |
   +------------------------------------------------------------------------------------+
   | IV  .  ONE FABRIC           —  it holds together at planet scale                   |
   +------------------------------------------------------------------------------------+
   | five-layer sync      validity . ordering . completeness . availability . merge     |
   | bounded locality     every change is local to a log-n neighborhood                 |
   | privacy trilateral   ZK + FHE + MPC compose over the shared field                  |
   +------------------------------------------------------------------------------------+

   you interface the whole stack in three languages :
   +----------------------+    +----------------------+    +----------------------+
   |        write         |    |       compute        |    |         mean         |
   +----------------------+    +----------------------+    +----------------------+
   |      cybermark       |    |  the trident family  |    |        neural        |
   +----------------------+    +----------------------+    +----------------------+
```

## one replaces many

every foundational method has the same shape. it takes something other systems build as many separate mechanisms and collapses it into one universal primitive.

other systems assign every object an id from a registry; soft3 gives every object one [[hemera]] hash, and the hash is the identity. other systems pick a hash tree here, a commitment scheme there, a proof system somewhere else; soft3 commits everything as one polynomial under one [[lens]]. other systems run a vote for consensus, a separate algorithm for ranking, and a third for rewards; soft3 settles all three — plus meaning — onto one equilibrium φ*.

the collapse is why soft3 is a stack and not a pile. when [[bbg]] needs authenticated state, [[zheng]] needs a commitment, [[radio]] needs availability, and [[fs]] needs identity, each reaches for the same primitive. composition is free because there is nothing to translate between.

four families of these collapses hold the stack up.

## I. one form — everything is one polynomial over one field

this looks like bookkeeping; it is the deepest unification in the stack. soft3 has exactly one mathematical object — the multilinear polynomial over the [[Goldilocks field]], a single algebraic form — and identity, value, state, computation, and proof are not separate things that happen to agree on a format. they all take that one form, seen from different sides.

- one field. every value is an element of the [[Goldilocks field]]. the field is the alphabet, chosen so that proofs, [[FHE]], and secret-sharing all operate inside it — cryptography is intrinsic, and there is no boundary between running a computation and proving it.
- one object. all state, all data, all proofs are a single committed polynomial. a read is a [[lens]] opening — one evaluation at one point, ~200 bytes — rather than a walk down a tree, and cross-index consistency is structural, not proven.
- five algebras. the one field hosts five algebraic regimes ([[strata]]) — truth, efficiency, encryption, optimization, privacy. a type picks its regime; the substance is the same in all of them.
- particle identity. a particle's name is the [[hemera]] hash of its content, which is itself a polynomial commitment — so content is identity, the same bytes always produce the same address, and the graph needs no registry.

this is why composition is free: when [[zheng]] hands [[bbg]] a commitment, or [[cybergraph]] hands [[hemera]] some content, they pass the same kind of object. there is nothing to serialize, because there is nothing to translate between. one field, one polynomial; everything else is a view of it.

## II. one proof — execution, programs, and the prover itself

soft3 turns every "trust me" into "check it," and makes the check cheap enough to always run. it is, as far as we know, the first proof system that proves at three levels at once — that a computation ran, that a program is correct, and that the prover is correct — all in one field, one kernel, with no trusted setup.

- proof-native execution. running a program and proving it ran correctly are the same act. the [[nox]] execution trace is the constraint system, with no separate arithmetization step; every computation emits its witness as a byproduct.
- recursive closure. proofs verify proofs. each step folds into an accumulator at ~30 field ops, and the entire history collapses to one constant-size proof behind a single final check. a light client validates all of history in roughly 100 nanoseconds.
- transparent. trust bottoms out on hash collision resistance alone — no trusted setup, no elliptic curves, no pairings. the proofs are post-quantum and verification stays stable for decades.
- conformance. every canonical mechanism output is fingerprinted with [[hemera]] ([[conformance]]). drift surfaces at commit time, so the protocol cannot shift underneath you silently.
- eidos. [[eidos]] is a proof assistant — full CIC type theory, [[Curry-Howard]] scaled to all of mathematics — whose type checker is itself a [[nox]] program that emits a [[zheng]] certificate. [[zheng]] proves that a computation ran; eidos proves that a program is correct. every proved theorem becomes a [[cyberlink]] in the graph.
- self-hosting. the [[zheng]] verifier is itself a [[nox]] program, so the system proves its own verification — recursively, to arbitrary depth, at constant proof size. the prover proves the prover. the system closes on itself.

these have never been unified before. transparent recursive STARKs exist; formal proof assistants exist. neither has been self-hosting on the very VM it proves, sharing one field and one kernel with the other. by carrying its prover, its program-logic, and its own verifier in the same [[nox]] algebra, soft3 is a proof system closed under its own verification — with civilization-grade mathematics living inside it. that closure is the unprecedented part, and it is what lets the whole [[cybergraph]] be proven once, as a single artifact.

## III. one convergence — the whole graph settles into one mind

every other foundation is machinery; this one is the thought.

[[focus]] is a single distribution φ* over every [[particle]] in the [[cybergraph]] — one number per particle, summing to one — the share of the whole collective's attention each piece of knowledge holds. it is not assigned or voted; it is the unique equilibrium the graph settles into, where each weight is fixed by every other through the links between them (φ*ⱼ = Σᵢ φ*ᵢ · pᵢⱼ). add one [[cyberlink]] and focus redistributes, and the whole graph re-converges. [[bounded locality]] keeps this computable — each re-convergence touches only the neighborhood that changed — but the equilibrium it lands on is global.

the [[collective focus theorem]] proves this equilibrium exists, is unique, and is reached from anywhere: the graph does not reason about importance, it equilibrates into it. its three operators — [[diffusion]], [[springs]], [[heat]] — are not a design choice but the only families that survive locality at planetary scale, and they are not abstractions: they are the physics we already live inside — diffusion, elasticity, and heat — the same processes that move matter, and we are simply running them on knowledge. [[PageRank]] and the rest need global recompute and are cut. and because there is exactly one attention over all particles, the quantities other systems compute separately are here the same object: φ* is at once consensus, ranking ([[cyberank]]), reward ([[karma]]), and meaning ([[neural]]) — one computation, four answers, because they were never four things. how fast the graph reaches φ* is its [[spectral gap]], literally the collective's speed of thought: below a threshold it is disconnected scatter, above it the [[egregore]] comes alive.

the egregore is this convergence, and it is a dissipative structure — it exists only while [[focus]] flows. starve it and φ* flattens to uniform and the mind dissolves; feed it and the graph grows more ordered, exporting entropy as it prunes noise. the network does not store what it knows. it re-thinks it, across all of knowledge, every step, on a convergence anyone can verify in microseconds.

## IV. one fabric — it holds together at planet scale

the last family lets the other three run across a planet of devices that fail, lie, and go offline.

- five-layer structural sync. every change passes five independent verification layers — validity, ordering, completeness, availability, merge ([[structural sync]]). each layer is checkable on its own, a verifier can check any subset, and none needs a consensus round.
- bounded locality. every property of a node depends only on its log-n-hop neighborhood, so a local change recomputes a local neighborhood and stops. this is the only reason 10^15 particles is tractable.
- privacy trilateral. [[ZK]] proves correctness, [[FHE]] hides data, [[MPC]] distributes trust; because all three share the [[Goldilocks field]] they compose into correctness without exposure and without a single point of failure ([[mudra]]).

## the language trinity

the four families are how the stack is built. three languages are how you touch it — itself an instance of one-replaces-many: one address alphabet under everything, one VM under every language, one semantic medium the whole collective shares.

- write — [[cybermark]], the markup and address language. every address resolves to a [[particle]]; the markup is the graph.
- compute — the [[trident]] family ([[trident]], [[rune]], [[eidos]], [[inf]], [[nu]], [[Rs]]). they all lower to [[nox]], where proof-native execution takes over.
- mean — [[neural]], the semantic language. meaning is an eigenvector of the attention graph; the egregore thinks in it.

## why this is a stack

pull any component and the four families are underneath it. [[bbg]] is polynomial state plus structural sync. [[zheng]] is proof-native execution plus recursive closure. [[foculus]] is tri-kernel φ* plus stake-weighted security. [[mudra]] is the privacy trilateral. [[cybergraph]] is particle identity plus bounded locality. the components are where the architecture meets a job.

building a new component is specializing the same four families to a new job. it composes with everything else for free — same identity, same field, same proof, same convergence, same fabric. that is the deal: a small number of universal methods, applied many times, that never need to agree with each other because they were never separate.

see [[soft3]] for the component stack and the troika compass.
