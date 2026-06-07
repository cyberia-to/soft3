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
   | I  .  ONE REPRESENTATION    —  everything is encoded one way                       |
   +------------------------------------------------------------------------------------+
   | particle identity    content is identity : one hemera hash, no registry            |
   | field-native         one goldilocks field element per value                        |
   | five algebras        all computation reduces to five algebraic regimes             |
   | polynomial state     all state and proofs are one committed polynomial             |
   +------------------------------------------------------------------------------------+
   | II  .  ONE PROOF            —  trust is computed, never granted                    |
   +------------------------------------------------------------------------------------+
   | proof-native         running a program and proving it are one act                  |
   | recursive closure    proofs verify proofs : all history folds to one               |
   | transparent          hash-based, post-quantum, no trusted setup                    |
   | conformance          every output fingerprinted : drift caught at commit           |
   +------------------------------------------------------------------------------------+
   | III  .  ONE CONVERGENCE     —  agreement is settled, not voted                     |
   +------------------------------------------------------------------------------------+
   | settle, do not derive   results are fixed points, not theorems                     |
   | tri-kernel  phi*        one equilibrium : consensus + rank + reward + meaning      |
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

## I. one representation — everything is encoded one way

before anything computes, everything must be a thing. soft3 fixes what a thing is, once, for the whole stack.

- particle identity. every object — a [[particle]], a [[cyberlink]], a [[neuron]], a proof, a polynomial — is named by the [[hemera]] hash of its content. content is identity. two agents that produce the same bytes produce the same address, with no registry and no coordinator. this is what makes the graph permissionless and memoizable.
- field-native. every value is one element of the [[Goldilocks field]]. computation already lives in the field where proofs, [[FHE]], and secret-sharing operate, so cryptography is intrinsic rather than bolted on. there is no boundary to cross between running and proving.
- five algebras. all computation reduces to five algebraic regimes ([[strata]]). a type chooses its algebra, the algebra chooses its proof system. truth, efficiency, encryption, optimization, and privacy each get the regime built for them, and the five span the whole surface.
- polynomial state. all state, all data, all proofs are one multilinear polynomial, committed once. a read is a [[lens]] opening — one evaluation at one point, ~200 bytes — rather than a walk down a tree. cross-index consistency is structural.

## II. one proof — trust is computed, never granted

soft3 turns every "trust me" into "check it," and makes the check cheap enough to always run.

- proof-native execution. running a program and proving it ran correctly are the same act. the [[nox]] execution trace is the constraint system, with no separate arithmetization step; every computation emits its witness as a byproduct.
- recursive closure. proofs verify proofs. each step folds into an accumulator at ~30 field ops, and the entire history collapses to one constant-size proof behind a single final check. a light client validates all of history in roughly 100 nanoseconds.
- transparent. trust bottoms out on hash collision resistance alone — no trusted setup, no elliptic curves, no pairings. the proofs are post-quantum and verification stays stable for decades.
- conformance. every canonical mechanism output is fingerprinted with [[hemera]] ([[conformance]]). drift surfaces at commit time, so the protocol cannot shift underneath you silently.

## III. one convergence — agreement is settled, not voted

soft3 computes a fixed point and reads the answer off it.

- settle, do not derive. results are equilibria the network converges onto, not theorems derived from axioms. this is what lets the system scale past what any formal system can prove or any single mind can hold.
- tri-kernel φ*. three operators — [[diffusion]], [[springs]], [[heat]] — reach a unique fixed point φ* independent of where they start. that one distribution is at once the consensus (what is final), the ranking ([[cyberank]], what matters), the reward (who gets paid), and the meaning ([[neural]], what a [[particle]] is). one computation, four answers.
- stake-weighted. an attacker controls outcomes only by controlling φ*-mass, which costs stake. security is an economic equilibrium rather than an honest-majority assumption.

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
