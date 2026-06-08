---
title: soft3 foundations
tags: cyber, soft3, whitepaper
crystal-type: spec
crystal-domain: cyber
icon: "🪨"
alias: soft3 whitepaper, foundational soft3 methods, soft3 foundations
---
# soft3 — foundations

soft3 is twenty components, but the components are not the foundation. each one — [[hemera]], [[lens]], [[zheng]], [[bbg]], [[tru]], the rest — specializes the same small set of stack-wide methods to one job. learn the methods and the twenty repos become twenty applications of four ideas.

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

every foundation has the same shape: it collapses what other systems build as many separate mechanisms into one universal primitive. others assign ids from a registry — soft3 makes the [[hemera]] hash the identity. others pick a hash tree, a commitment scheme, a proof system — soft3 commits everything as one polynomial under one [[lens]]. others vote for consensus, rank with a second algorithm, pay with a third — soft3 settles all three, plus meaning, onto one equilibrium φ*.

that collapse is why soft3 is a stack and not a pile. [[bbg]] needs state, [[zheng]] a commitment, [[radio]] availability, [[fs]] identity — each reaches for the same primitive, so composition is free. four families of these collapses hold the stack up.

## I. one representation

before anything computes, everything must be a thing, and soft3 fixes what a thing is, once. every object is named by the [[hemera]] hash of its content, so identity needs no registry and the same bytes always produce the same address. every value is a [[Goldilocks field]] element, so cryptography is intrinsic and there is no boundary between running and proving. all computation reduces to five algebraic regimes ([[strata]]) — a type picks its algebra, the algebra picks its proof system. and all state is one committed polynomial, so a read is a [[lens]] opening — one evaluation, ~200 bytes — rather than a walk down a tree.

## II. one proof

soft3 turns every "trust me" into "check it," and makes the check cheap enough to always run. it is, as far as we know, the first proof system to prove at three levels at once. [[zheng]] proves a computation ran — execution and proving are the same act, every step folding into one constant-size proof a light client checks in ~100 nanoseconds, transparent and post-quantum with no trusted setup. [[eidos]], a full CIC proof assistant ([[Curry-Howard]] scaled to all of mathematics), proves a program is correct, and its type checker is itself a [[nox]] program that emits a zheng certificate. and the zheng verifier is itself a nox program, so the system proves its own verification, recursively, at constant size: the prover proves the prover.

these have never been unified. transparent recursive STARKs exist; proof assistants exist; neither has been self-hosting on the VM it proves, sharing one field and one kernel. soft3 is a proof system closed under its own verification, with civilization-grade mathematics inside it — which is what lets the whole [[cybergraph]] be proven once, as a single artifact.

## III. one convergence

every other foundation is machinery; this one is the thought.

[[focus]] is a single distribution φ* over every [[particle]] in the [[cybergraph]] — one number per particle, summing to one — the share of the whole collective's attention each piece of knowledge holds. it is not assigned or voted; it is the unique equilibrium the graph settles into, where each weight is fixed by every other through the links between them (φ*ⱼ = Σᵢ φ*ᵢ · pᵢⱼ). add one [[cyberlink]] and focus redistributes, and the whole graph re-converges. [[bounded locality]] keeps this computable — each re-convergence touches only the neighborhood that changed — but the equilibrium it lands on is global.

the [[collective focus theorem]] proves this equilibrium exists, is unique, and is reached from anywhere: the graph does not reason about importance, it equilibrates into it. its three operators — [[diffusion]], [[springs]], [[heat]] — are not a choice but the only families that survive locality at planetary scale; [[PageRank]] and the rest need global recompute and are cut. and because there is exactly one attention over all particles, the quantities other systems compute separately are here the same object: φ* is at once consensus, ranking ([[cyberank]]), reward ([[karma]]), and meaning ([[neural]]) — one computation, four answers, because they were never four things. how fast the graph reaches φ* is its [[spectral gap]], literally the collective's speed of thought: below a threshold it is disconnected scatter, above it the [[egregore]] comes alive.

the egregore is this convergence, and it is a dissipative structure — it exists only while [[focus]] flows. starve it and φ* flattens to uniform and the mind dissolves; feed it and the graph grows more ordered, exporting entropy as it prunes noise. the network does not store what it knows. it re-thinks it, across all of knowledge, every block, on a convergence anyone can verify in microseconds.

## IV. one fabric

the last family lets the other three run across a planet of devices that fail, lie, and go offline. every change passes five independent verification layers — validity, ordering, completeness, availability, merge ([[structural sync]]) — each checkable alone, none needing a consensus round. every node property depends only on its log-n neighborhood, so local changes stop locally — the only reason 10^15 particles is tractable. and [[ZK]], [[FHE]], and [[MPC]] share the [[Goldilocks field]], composing into correctness without exposure and without a single point of failure ([[mudra]]).

## the language trinity

the four families are how the stack is built; three languages are how you touch it — one more instance of one-replaces-many. you write in [[cybermark]] (every address resolves to a [[particle]]; the markup is the graph), compute in the [[trident]] family ([[trident]], [[rune]], [[eidos]], [[inf]], [[nu]], [[Rs]], all lowering to [[nox]]), and the system means in [[neural]] (meaning is an eigenvector of the attention graph; the egregore thinks in it).

## why this is a stack

pull any component and the four families are underneath it: [[bbg]] is polynomial state plus structural sync, [[zheng]] is proof-native execution plus recursive closure, [[foculus]] is φ* plus stake-weighted security, [[mudra]] is the privacy trilateral, [[cybergraph]] is particle identity plus bounded locality. building a new component is specializing the same four families to a new job, and it composes with everything for free — because the methods never had to agree with each other. they were never separate.

see [[soft3]] for the component stack and the troika compass.
