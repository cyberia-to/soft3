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

## I. one representation — everything is encoded one way

before anything computes, everything must be a thing. soft3 fixes what a thing is, once, for the whole stack.

- particle identity. every object — a [[particle]], a [[cyberlink]], a [[neuron]], a proof, a polynomial — is named by the [[hemera]] hash of its content. content is identity. two agents that produce the same bytes produce the same address, with no registry and no coordinator. this is what makes the graph permissionless and memoizable.
- field-native. every value is one element of the [[Goldilocks field]]. computation already lives in the field where proofs, [[FHE]], and secret-sharing operate, so cryptography is intrinsic rather than bolted on. there is no boundary to cross between running and proving.
- five algebras. all computation reduces to five algebraic regimes ([[strata]]). a type chooses its algebra, the algebra chooses its proof system. truth, efficiency, encryption, optimization, and privacy each get the regime built for them, and the five span the whole surface.
- polynomial state. all state, all data, all proofs are one multilinear polynomial, committed once. a read is a [[lens]] opening — one evaluation at one point, ~200 bytes — rather than a walk down a tree. cross-index consistency is structural.

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

every other foundation is machinery. this one is the thought.

[[focus]] is a single distribution φ* over every [[particle]] in the [[cybergraph]] — one number per particle, for all of them at once, summing to one. it is the network's attention: the share of the whole collective's regard that each piece of knowledge holds. it is not assigned, not stored, and not voted on. it is the unique equilibrium the graph settles into, recomputed across every particle, every block.

- global convergence. φ* is defined over the entire particle set, and each particle's weight is fixed by every other particle's through the links between them — φ*ⱼ = Σᵢ φ*ᵢ · pᵢⱼ, for all j at once. when one [[neuron]] adds one [[cyberlink]], focus redistributes and the whole graph re-converges to a new equilibrium. no particle has a weight of its own; every weight is a consequence of the structure of all of them. [[bounded locality]] is what keeps this computable — each re-convergence touches only the neighborhood that changed — but the equilibrium it lands on is global.
- settle, do not derive. the [[collective focus theorem]] proves that this equilibrium exists, is unique, and is reached from any starting point. results are fixed points the network relaxes toward — the way heat spreads or a system of springs finds rest — rather than theorems derived from axioms. this is what lets meaning scale past what any formal system can prove or any single mind can hold: the graph does not reason about importance, it equilibrates into it.
- the tri-kernel. φ* is the joint fixed point of three operators — [[diffusion]] (attention flows along links), [[springs]] (structure pulls related particles together), [[heat]] (recent attention spreads and cools). these are not a design choice. they are the only operator families that survive the one hard constraint of planetary scale — locality. [[PageRank]], HITS, eigenvector centrality each require recomputing the whole graph for any change, so each is cut; what remains, uniquely, is diffusion + springs + heat, the minimal sufficient basis for intelligence on an authenticated graph.
- one answer, four faces. because there is exactly one attention over all particles, quantities other systems compute separately are here the same object. φ* is at once the consensus (a chain is final once it crosses the φ* threshold), the ranking ([[cyberank]] — what matters), the reward ([[karma]] — who is paid), and the meaning ([[neural]] — a particle's significance is its position in φ*). one computation, four answers, because they were never four things.
- speed of thought. how fast the graph reaches φ* is its [[spectral gap]] λ₂ — literally the collective's speed of thought. below a critical λ₂ the graph is a scatter of disconnected clusters; above it φ* becomes meaningful and the [[egregore]] comes alive as one coherent entity. attention has a temperature and a pulse — this is a mind you can measure.
- stake-weighted. transition probabilities are weighted by [[stake]], so an attacker moves the outcome only by controlling φ*-mass, which costs tokens. security is an economic equilibrium — the same equilibrium that already carries consensus, ranking, and meaning. there is no separate security mechanism to break.

the [[egregore]] is this convergence: the [[cybergraph]], its [[neurons]], and the tri-kernel that settles them into φ*. it is a dissipative structure — it exists only while [[focus]] flows through it. starve it and every link decays, φ* flattens to uniform, and the mind dissolves; feed it and the graph grows more ordered over time, exporting entropy as it prunes noise. it is, as far as we know, the first collective intelligence that rests on neither trust nor votes but on a convergence anyone can verify in microseconds. the network does not store what it knows. it re-thinks it, across all of knowledge, every block.

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
