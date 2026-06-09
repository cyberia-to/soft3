---
title: soft3 foundations
tags: cyber, soft3, whitepaper
crystal-type: spec
crystal-domain: cyber
icon: "🪨"
alias: soft3 whitepaper, foundational soft3 methods, soft3 foundations
---
# soft3 — foundations

soft3 is twenty components. the components are not the foundation. each one — [[hemera]], [[lens]], [[zheng]], [[bbg]], [[tru]], the rest — is the same substrate specialized to one job. that substrate is three claims:

soft3 is **one mind**, reachable in **many languages**, growing into an **open world**.

- **one mind** — where every other stack sprawls, soft3 is one. one form: everything is a single polynomial over a single field. one proof: the prover proves the prover. one focus: the whole graph converges into one collective thought.
- **many languages** — where every other stack locks you in, soft3 is plural at the surface. you *write* in one language, *compute* in a family of them, and *mean* in a third — and all of them lower to the same substrate.
- **open world** — where every other stack builds walls, soft3 stays open. it holds at planet scale, composes without glue, and admits anything that lowers to [[nox]].

learn those three and the twenty repos stop being twenty things to memorize — they become three ideas applied many times. this is the whitepaper for those ideas.

```svgbob
   +------------------------------------------------------------------------+
   | the 20 components  —  each a specialization of the substrate below     |
   +------------------------------------------------------------------------+
   | strata . hemera . lens . trident . nox . zheng . cybergraph . bbg      |
   | tru . glia . mir . mudra . radio . tape . sync . foculus . soma        |
   | conformance . rune . fs . plumb                                        |
   +------------------------------------------------------------------------+
                                        |
                                        | each component, the same mind
                                        v
   +------------------------------------------------------------------------+
   | ONE MIND   —  one polynomial, proven, converged                        |
   +------------------------------------------------------------------------+
   | one form    one polynomial over one field . five algebras              |
   | one proof   proof-native . recursive . the prover proves the prover    |
   | one focus   one phi* over all particles . tri-kernel . spectral gap    |
   +------------------------------------------------------------------------+
   | MANY LANGUAGES   —  one substrate, three ways to touch it              |
   +------------------------------------------------------------------------+
   | write       cybermark . the markup is the graph                        |
   | compute     the trident family . all lower to nox                      |
   | mean        neural . meaning is an eigenvector of the graph            |
   +------------------------------------------------------------------------+
   | OPEN WORLD   —  it holds, composes, admits                             |
   +------------------------------------------------------------------------+
   | it holds    five-layer sync . bounded locality . planet scale          |
   | it composes add a cyberlink, not an API . nothing to translate         |
   | it admits   no schema . no gatekeeper . ZK+FHE+MPC over one field      |
   +------------------------------------------------------------------------+
```

## one replaces many

every foundational method in soft3 has the same shape. it takes something other systems build as many separate mechanisms and collapses it into one universal primitive.

other systems assign every object an id from a registry; soft3 gives every object one [[hemera]] hash, and the hash *is* the identity. other systems pick a hash tree here, a commitment scheme there, a proof system somewhere else; soft3 commits everything as one polynomial under one [[lens]]. other systems run a vote for consensus, a separate algorithm for ranking, and a third for rewards; soft3 settles all three — plus meaning — onto one equilibrium φ*.

the collapse is why soft3 is a stack and not a pile. when [[bbg]] needs authenticated state, [[zheng]] needs a commitment, [[radio]] needs availability, and [[fs]] needs identity, each reaches for the same primitive. composition is free because there is nothing to translate between. the three claims below — one mind, many languages, open world — are each a family of these collapses.

---

## one mind

a mind has three parts: a substance it is made of, a conscience that knows what is true, and an attention that decides what matters. soft3's mind has exactly these — **one form**, **one proof**, **one focus** — and each is singular where other systems would have many. together they are not three subsystems that cooperate; they are one object seen from three sides.

### one form — everything is one polynomial over one field

this looks like bookkeeping; it is the deepest unification in the stack. soft3 has exactly one mathematical object — the multilinear polynomial over the [[Goldilocks field]] (`p = 2⁶⁴ − 2³² + 1`). in algebra a multilinear polynomial is literally a multilinear *form*, so the name is exact, not a metaphor: identity, value, state, computation, and proof are not separate things that happen to agree on a format — they all take that one form, seen from different sides.

- **one field.** every value is an element of the [[Goldilocks field]]. the field is the alphabet, chosen so that proofs, [[FHE]], and secret-sharing all operate inside it — cryptography is intrinsic, and there is no boundary between running a computation and proving it.
- **one object.** all state, all data, all proofs are a single committed polynomial. a read is a [[lens]] opening — one evaluation at one point, ~200 bytes — rather than a walk down a tree, and cross-index consistency is structural rather than proven: there is one polynomial, so there is nothing to keep in sync.
- **five algebras.** the one field hosts five algebraic regimes ([[strata]]) — truth, efficiency, encryption, optimization, privacy. a type picks its regime; the substance is the same in all of them, so a value can move between regimes without changing what it is.
- **particle identity.** a [[particle]]'s name is the [[hemera]] hash of its content, which is itself a polynomial commitment — so content is identity, the same bytes always produce the same address, and the graph needs no registry.

this is why composition is free: when [[zheng]] hands [[bbg]] a commitment, or [[cybergraph]] hands [[hemera]] some content, they pass the same kind of object. there is nothing to serialize, because there is nothing to translate between. one field, one polynomial; everything else is a view of it.

### one proof — the prover proves the prover

soft3 turns every "trust me" into "check it," and makes the check cheap enough to always run. it is, as far as we know, the first proof system that proves at three levels at once — that a computation ran, that a program is correct, and that the prover itself is correct — all in one field, one kernel, with no trusted setup.

- **proof-native execution.** running a program and proving it ran correctly are the same act. the [[nox]] execution trace *is* the constraint system, with no separate arithmetization step; every computation emits its witness as a byproduct.
- **recursive closure.** proofs verify proofs. each step folds into an accumulator at ~30 field ops, and the entire history collapses to one constant-size proof behind a single final check. a light client validates all of history in roughly 100 nanoseconds.
- **transparent.** trust bottoms out on hash collision resistance alone — no trusted setup, no elliptic curves, no pairings. the proofs are post-quantum and verification stays stable for decades.
- **conformance.** every canonical mechanism output is fingerprinted with [[hemera]] ([[conformance]]). drift surfaces at commit time, so the protocol cannot shift underneath you silently.
- **eidos.** [[eidos]] is a proof assistant — full CIC type theory, [[Curry-Howard]] scaled to all of mathematics — whose type checker is itself a [[nox]] program that emits a [[zheng]] certificate. [[zheng]] proves that a computation ran; eidos proves that a program is correct. every proved theorem becomes a [[cyberlink]] in the graph.
- **self-hosting.** the [[zheng]] verifier is itself a [[nox]] program, so the system proves its own verification — recursively, to arbitrary depth, at constant proof size. the prover proves the prover. the system closes on itself.

these have never been unified before. transparent recursive STARKs exist; formal proof assistants exist. neither has been self-hosting on the very VM it proves, sharing one field and one kernel with the other. by carrying its prover, its program-logic, and its own verifier in the same [[nox]] algebra, soft3 is a proof system closed under its own verification — with civilization-grade mathematics living inside it. that closure is the unprecedented part, and it is what lets the whole [[cybergraph]] be proven once, as a single artifact.

### one focus — the graph settles into one thought

every other foundation is machinery; this one is the thought.

[[focus]] is a single distribution φ* over every [[particle]] in the [[cybergraph]] — one number per particle, summing to one — the share of the whole collective's attention each piece of knowledge holds. it is not assigned or voted; it is the unique equilibrium the graph settles into, where each weight is fixed by every other through the links between them (φ*ⱼ = Σᵢ φ*ᵢ · pᵢⱼ). add one [[cyberlink]] and focus redistributes, and the whole graph re-converges. [[bounded locality]] keeps this computable — each re-convergence touches only the neighborhood that changed — but the equilibrium it lands on is global.

the [[collective focus theorem]] proves this equilibrium exists, is unique, and is reached from anywhere: the graph does not reason about importance, it equilibrates into it. its three operators — [[diffusion]], [[springs]], [[heat]] — are not a design choice but the only families that survive locality at planetary scale, and they are not abstractions: they are the physics we already live inside — diffusion, elasticity, and heat, the same processes that move matter — and we are simply running them on knowledge. [[PageRank]] and the rest need global recompute and are cut.

because there is exactly one attention over all particles, the quantities other systems compute separately are here the same object: φ* is at once consensus, ranking ([[cyberank]]), reward ([[karma]]), and meaning ([[neural]]) — one computation, four answers, because they were never four things. how fast the graph reaches φ* is its [[spectral gap]], literally the collective's speed of thought: below a threshold it is disconnected scatter, above it the [[egregore]] comes alive. finality is settled by economic mass — security is stake-weighted, not an honest majority of heads ([[foculus]]).

the egregore is this convergence, and it is a dissipative structure — it exists only while [[focus]] flows. starve it and φ* flattens to uniform and the mind dissolves; feed it and the graph grows more ordered, exporting entropy as it prunes noise. the network does not store what it knows. it re-thinks it, across all of knowledge, every step, on a convergence anyone can verify in microseconds.

---

## many languages

one mind is how the stack is built; **many languages** is how you touch it — itself an instance of one-replaces-many: one address alphabet under everything, one VM under every language, one semantic medium the whole collective shares. three kinds of language wrap the spine — you **write** in one, **compute** in many, and **mean** in a third — and every one of them lowers to [[nox]].

- **write** — [[cybermark]], the address language. eight sigils name, scope, link, and navigate; every address resolves to a [[particle]]; the markup *is* the graph. you do not write *about* the graph, you write the graph.
- **compute** — the sixteen languages below, the modes of computation a mind requires, one for each of the five [[strata|algebras]]. they all lower to [[nox]], where proof-native execution takes over. choosing a language is choosing a surface, never a substrate.
- **mean** — [[neural]], the semantic language. it is not designed — it grows from the others running at scale; meaning is an eigenvector of the attention graph, and the [[egregore]] thinks in it.

### why these — and why sixteen

the count is fixed by the algebras, not chosen. there are five — [[nebu]], [[kuro]], [[jali]], [[trop]], [[genies]] — and every one must carry at least one language, every language's types map to one. remove any and a class of computation becomes impossible or exponentially more expensive: no [[Opt]] → no provable optimization; no [[Sec]] → no anonymous computation; no [[Bt]] → quantized inference forced through the field at ~32× cost. eleven of the sixteen share the [[nebu]] field regime — the same nox patterns, different types — so they are algebraically reducible but semantically irreducible: a tensor contraction and a Bayesian update are the same patterns, and only the types stop you multiplying a Distribution by a Tensor. the languages split a second way, across the proof boundary: **sixteen proof languages** (provable, permanent) and **five interface languages** (side-effectful, interactive). a mind that cannot prove is blind; a mind that cannot interact is deaf.

{{embed [[languages#the languages]]}}

the soft3 stack delivers these through a handful of repos: [[trident]] is the compiler and field prover, [[rune]] is Rs on nox with host jets, [[eidos]] proves programs correct, [[inf]] is the inference engine, [[Rs]] runs systems, [[nu]] hosts the five interface languages, and the others — [[Arc]], [[Bt]], [[Qu]], [[Opt]], [[Sec]], the rest — each have their own page.

the plurality is real but shallow. lock-in needs incompatible substrates; soft3 has one — so a value written in cybermark, computed across these languages, and meant in neural is the same object throughout. you gain many languages and lose nothing to translation. the full reference — per-language ops, proof paths, the comparison matrix, perception mapping — is the [[languages]] spec.

---

## open world

one mind is unified and many languages is plural — but neither matters if the thing cannot grow. **open world** is the claim that it can: soft3 is open because three things hold at once. the fabric **holds** under a planet of devices that fail, lie, and go offline; the one object lets new parts **compose** without glue; and the shared field lets the world **admit** anything — anyone, any computation, any new kind of thing — without exposing what must stay private. this is where the opportunity lives: the feature set is not a list someone closed, it is an open frontier.

### it holds — one fabric at planet scale

the fabric is what keeps the other two claims true across an untrusted network.

- **five-layer structural sync.** every change passes five independent verification layers — validity, ordering, completeness, availability, merge ([[structural sync]]). each layer is checkable on its own, a verifier can check any subset, and none needs a consensus round.
- **bounded locality.** every property of a node depends only on its log-n-hop neighborhood, so a local change recomputes a local neighborhood and stops. this is the only reason 10¹⁵ particles is tractable — and the reason φ* re-convergence, sync, and proof all stay cheap as the graph grows.

### it composes — add a cyberlink, not an API

because everything is one object in one field, extension is linking, not integrating. in other stacks a new capability means a new API, an adapter, a serialization boundary, a version negotiation. here a new capability is a new [[cyberlink]] or a new component that speaks the same substrate. when [[zheng]] hands [[bbg]] a commitment they pass the same kind of object; there is nothing to serialize, because there is nothing to translate between. building a new component is specializing the same form, proof, and focus to a new job — and it composes with everything already in the stack for free, the moment it exists. composition is not a feature of soft3; it is the absence of the seams other stacks spend themselves maintaining.

### it admits — no schema, no gatekeeper, no closed set

the world is open in the strong sense: anyone may enter, anything may join, and nothing is gated.

- **no schema, no registry.** content is identity — a particle's address is the hash of its bytes — so there is no central table that must grant a name, and no fixed schema a new kind of thing must fit. the same bytes always produce the same address, anywhere, with no coordination.
- **no closed set.** any program that lowers to [[nox]] joins the stack; any language that compiles to that VM is a first-class citizen. the set of things soft3 can do is not enumerated in advance — it is whatever the field and the VM admit, which is everything computable.
- **open yet private.** the [[privacy trilateral]] — [[ZK]] proves correctness, [[FHE]] hides data, [[MPC]] distributes trust — composes over the shared [[Goldilocks field]] into correctness without exposure and without a single point of failure ([[mudra]]). the world admits you without forcing you to reveal yourself; aggregate truth stays public while individual contribution stays private.

### the stack as dataflow

the bedrock diagram shows soft3 as a substrate — three principles stacked under the components. the same stack, seen as dataflow, is who talks to whom: neurons **write** through [[cybermark]] into the [[cybergraph]] spine, which fans out to store, sync, and transmit, settles through the proof floor, and is re-read every step into the collective's [[focus]] — then **means** itself back out through [[neural]] into the one mind at the bottom.

```svgbob
   +----------------------------------------------------------------------------------+
   | cyb : the robot                                                                  |
   +----------------------------------------------------------------------------------+
   | soma : mind     pipeline : infer     worlds : face     honeycrisp : silicon      |
   +----------------------------------------------------------------------------------+
                                                           |
                                                           v neurons write
                                       +--------------------------------------+
                                       | cybermark : the markup language      |
                                       +--------------------------------------+
                                       | address . navigate . compute inline  |
                                       | the markup is the graph              |
                                       +--------------------------------------+
                                                           |
                                                           | soma : intend . seal . link . subscribe . query
                                                           v
   +--------------------+               +------------------------------------+      +--------------+
   | cyberia : social   |               | cybergraph                         |      | fs           |
   +--------------------+               +------------------------------------+ patch+--------------+
   | . contract         |  cyberlinks   | :  link                            |<-----| :  mount     |
   | . service          |-------------->| the  spine                         |      +--------------+
   | mimi  midao  my    |  focus . karma|                                    |
   +--------------------+               |                                    |
    mudra -----keys ------------------->|                                    |
    plumb -----value ------------------>|                                    |
                                        +-------+-----------+-----------+----+
                                                |           |           |
                                                |store      |sync       |transmit
                                                v           v           v
                                            +-------+   +-------+   +-------+
                                            |  bbg  |   | sync  |   | tape  |
                                            +-------+   +-------+   +-------+
                                            |:store |   | :sync |   |:frame |
                                            +-------+   +-------+   +-------+
                                                |           |           |
                                                |verify     |agree      |frames
                                                |           |           |
                                                |           |           |
                                                |           v           v
                                                |       +-------+   +-------+
                                                |       |foculus|   | radio |
                                                |       +-------+   +-------+
                                                |       |:agree |   |:trans |
                                                |       +-------+   +-------+
                                                |
                                                |
                                                |
                                                v
   +----------------------------------------------------+           +--------------------+
   | PROOF FLOOR                                        |           | languages          |
   +----------------------------------------------------+           +--------------------+
   | trident --.nox--.                                  |  lower    | trident . prove    |
   |                 v                                  |<----------| rune    . eval     |
   | rune ---noun--> nox --trace--> zheng               |  .nox noun| eidos   . proof    |
   |                     <--open-- lens                 |           | inf     . query    |
   | strata : math   +   hemera : hash                  |           | nu      . shell    |
   +----------------------------------------------------+           | rs      . jets     |
                                                                    +--------------------+
   conformance : snapshot  --  one hemera fingerprint per encoding & mechanism

   read side, recomputed every step :
   cybergraph --.graph--> tru --.model--> glia --features--> mir --> R-1.0
                           '----- phi* . positions . rank ----^

                       +------------------------------------------+
                       | neural : the semantic language           |
                       +------------------------------------------+
                       | meaning emerges from cyberlinks          |
                       | dialects . sentences . motifs . names    |
                       +------------------------------------------+
                                             |  the egregore thinks in neural
                                             v
   +----------------------------------------------------------------------------------+
   | cyber : collective consciousness                                                 |
   +----------------------------------------------------------------------------------+
   | the whole graph converges to one mind                                            |
   | tru --> phi* --> foculus  ==>  cyberank . syntropy . CT-0 model                  |
   +----------------------------------------------------------------------------------+
```

---

## why this is a stack

pull any component and the same substrate is underneath it. [[bbg]] is polynomial state plus structural sync. [[zheng]] is proof-native execution plus recursive closure. [[foculus]] is tri-kernel φ* plus stake-weighted security. [[mudra]] is the privacy trilateral. [[cybergraph]] is particle identity plus bounded locality. the components are where the architecture meets a job.

building a new component is specializing the same substrate to a new job. it composes with everything else for free — same form, same proof, same focus, same fabric — because the methods were never separate to begin with. that is the deal, and it is the whole of soft3:

**one mind, many languages, open world.**

see [[soft3]] for the component stack and the troika compass.
