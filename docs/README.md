---
title: soft3 foundations
tags: cyber, soft3, whitepaper
crystal-type: spec
crystal-domain: cyber
icon: "🪨"
alias: soft3 whitepaper, foundational soft3 methods, soft3 foundations
---
# soft3 — foundations

Architecture contract: [execution model](../specs/execution-model.md).
It defines VM/OS ownership, open-ended network instances, proof-profile
selection and worker placement. [Neuron](../specs/neuron.md) and
[cyb architecture](../../cyb/specs/architecture.md) define the current subject,
robot and prog boundaries. The substrate explanation below is an architectural
vision; concrete codec, authority, proof and deployment claims follow the
declared profiles and [implementation evidence](../audit/neuron-cell/implementation.md).

Soft3's [component registry](../status.md) assigns each mechanism a job. The
foundation relates those jobs through shared mathematical and composition
contracts. Its three architectural claims are:

soft3 is **one mind**, reachable in **many languages**, growing into an **open world**.

```text
one mind
   one form     —  one multilinear form over one field
   one proof    —  the prover proves the prover
   one focus    —  the graph settles into one φ*

many languages
   write        —  cybermark
   compute      —  the trident family
   mean         —  neural

open world
   it holds     —  one fabric, planet scale
   it composes  —  add a cyberlink, not an API
   it admits    —  open formats, explicit authority, bounded work
```

- **one mind** — where every other stack sprawls, soft3 is one. one form: everything is a single polynomial over a single field. one proof: the prover proves the prover. one focus: the whole graph converges into one collective thought.
- **many languages** — where every other stack locks you in, soft3 is plural at the surface. you *write* in one language, *compute* in a family of them, and *mean* in a third — and all of them lower to the same substrate.
- **open world** — where every other stack builds walls, soft3 stays open. it holds at planet scale, composes without glue, and admits anything that lowers to [[nox]].

These ideas organize the component registry. Repository count, host-resident
roles and publication status remain separate from the foundation.

```svgbob
   +------------------------------------------------------------------------+
   | components and shared contracts over the substrate                    |
   +------------------------------------------------------------------------+
   | honeycrisp . strata . hemera . lens . cybergraph . bbg . neuron . fs   |
   | tok . mudra . radio . tade . foculus . ward . tru . soma . neural      |
   | rune . inf . trident . nox . wysm . glia . kern . zheng . eidos        |
   | mir . prysm . vault . lytics . conformance                             |
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
   | it admits   open formats . subject authority . bounded execution       |
   +------------------------------------------------------------------------+
```

## one replaces many

every foundational method in soft3 has the same shape. it takes something other systems build as many separate mechanisms and collapses it into one universal primitive.

Content uses hemera addressing; compatible state and proof profiles reuse lens
commitments. Native subject identity is H(compressed public key), while foreign
subjects retain their domain-qualified addresses. These typed identities name
different roles even when represented by 32 bytes. The focus design relates
ranking, rewards and consensus, each under its declared network rules.

BBG, zheng, radio and fs reuse common primitives where their profiles match.
Composition preserves exact codecs, network domains, current authority and
evidence semantics at every boundary. The three claims below organize that reuse.

## subjects, progs and the implemented local profile

One robot attaches several neurons for keys, networks and devices. A neuron can
run several progs and invocations with independent state and retained work IDs.
The hierarchy ends at the subject: progs, tasks, operations, workers and graph
hosts introduce no additional signer. Native and foreign identity rules are
specified by [consumer boundaries](../specs/identity-consumers.md).

| Role | Current ownership |
|---|---|
| Neuron | Subject identity; optional bounded prog execution, continuation and recovery |
| GraphSession / node | Multi-neuron graph hosting, retained native coordinator and network observations |
| Ward / vault | Current binding/grant checks, captured subject/network, custody and scoped encrypted state |
| Soma | Durable tasks, context/model capture, tools, children, schedules and proposal-only learning |
| Rune / worker | Evaluation and admitted effects under declared machine/evidence/resource profiles |
| Cybergraph / BBG / Log | Authoritative history, durable publication/storage, rendered history view |

The [Soma composition audit](../../soma/audit/neuron-composition.md) includes actual
local glia inference and CLI/Bevy processes. Unknown effects retain the original
attempt and do not automatically execute again. A selection change cannot
retarget pending work. Local model/host observations, graph commits, signed
Signals, endpoint receipts and finalized network facts are distinct evidence.
The implemented local profile has trusted host adapters and cooperative GPU
limits; broader providers, hard isolation and consensus require their own work.

---

## one mind

a mind has three parts: a substance it is made of, a conscience that knows what is true, and an attention that decides what matters. soft3's mind has exactly these — **one form**, **one proof**, **one focus** — and each is singular where other systems would have many. together they are not three subsystems that cooperate; they are one object seen from three sides.

### one form — everything is one polynomial over one field

The field-native design centers the multilinear polynomial over the
[[Goldilocks field]] (`p = 2⁶⁴ − 2³² + 1`). In algebra this is a multilinear
form. Data, state and proofs can share its machinery while preserving their
specific codecs and validation rules. Current native signing uses secp256k1;
foreign addresses and original signed history retain their native bytes.

- **one field.** compatible field-native computation/proof profiles share the Goldilocks alphabet. Host observations, native key signatures and foreign protocol adapters retain their separately declared cryptography and evidence.
- **one object.** all state, all data, all proofs are a single committed polynomial. a read is a [[lens]] opening — one evaluation at one point, ~200 bytes — rather than a walk down a tree, and cross-index consistency is structural rather than proven: there is one polynomial, so there is nothing to keep in sync.
- **five algebras.** the one field hosts five algebraic regimes ([[strata]]) — truth, efficiency, encryption, optimization, privacy. a type picks its regime; the substance is the same in all of them, so a value can move between regimes without changing what it is.
- **particle identity.** a [[file]]'s particle is the [[hemera]] hash of its content under the exact content/codec profile. Native NeuronId remains H(compressed pubkey); prog and invocation IDs refer to data under that subject. A content hash alone supplies no signing authority.

Compatible zheng/BBG commitments and cybergraph/hemera content share primitives.
Serialization, domain separation and schema versions preserve their exact
meaning. Immutable legacy cell suites and foreign signing formats remain readable
under their original profiles throughout migration.

### one proof — the prover proves the prover

The proof design relates three levels: execution correctness, program/theorem
correctness and recursive verification. Each completed path must name the exact
statement, verifier and parameters. These are architectural targets whose
implementation status belongs to the relevant component and proof profile.

- **proof-native execution.** running a program and proving it ran correctly are the same act. the [[nox]] execution trace *is* the constraint system, with no separate arithmetization step; every computation emits its witness as a byproduct.
- **recursive closure.** a supported folding profile can accumulate its declared transition statements. Size, cost and security require that profile's measurements and assumptions; a local native receipt is not such a folded history proof.
- **transparent.** transparent proof profiles avoid a trusted setup under their stated assumptions. This does not change the implemented native subject's secp256k1 signature scheme or establish post-quantum custody.
- **conformance.** every canonical mechanism output is fingerprinted with [[hemera]] ([[conformance]]). drift surfaces at commit time, so the protocol cannot shift underneath you silently.
- **eidos.** the proof assistant checks theorem terms. Its intended zheng certificate/cyberlink path still has an explicit bridge gap in [stack completeness](../roadmap/stack-completeness.md); term checking alone does not publish a settled proof.
- **self-hosting.** the [[zheng]] verifier is itself a [[nox]] program, so the system proves its own verification — recursively, to arbitrary depth, at constant proof size. the prover proves the prover. the system closes on itself.

The desired closure reuses the same nox algebra across execution, program logic
and verification. A complete graph-wide claim must also cover external effects,
network transitions, availability and the selected checkpoint/finality rules.
Those obligations survive composition and are not supplied by a shared runtime ID.

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

A new cyberlink can reference code, content, a schema or a supported capability.
Executing the referenced work requires a compatible prog/worker profile and
current authority. Common primitives reduce adapter work; exact interfaces,
codecs, resource bounds and evidence still define each composition boundary.

### it admits — open formats and explicit authority

Anyone can address content and implement a compatible component. Reading a page
or naming a prog is separate from permission to execute, spend, disclose or
publish. These effects pass the subject's current host and network policies.

- **open content.** a compatible content-addressing profile can name arbitrary bytes without central allocation. Protocol actions and retained runtime records use explicit schemas; adding another schema does not change existing hashes or signatures.
- **extensible computation.** new languages and progs can target supported machines. Admission captures subject/network/grant and a finite resource allowance; interpreter support alone grants no external authority.
- **open yet private.** the [[privacy trilateral]] — [[ZK]] proves correctness, [[FHE]] hides data, [[MPC]] distributes trust — composes over the shared [[Goldilocks field]] into correctness without exposure and without a single point of failure ([[mudra]]). the world admits you without forcing you to reveal yourself; aggregate truth stays public while individual contribution stays private.

### the stack as dataflow

the bedrock diagram shows soft3 as a substrate — three principles stacked under the components. the same stack, seen as dataflow, is who talks to whom: neurons **write** through [[cybermark]] into the [[cybergraph]] spine, which fans out to store, sync, and transmit, settles through the proof floor, and is re-read every step into the collective's [[focus]] — then **means** itself back out through [[neural]] into the one mind at the bottom.

```svgbob
   +----------------------------------------------------------------------------------+
   | cyb : the robot                                                                  |
   +----------------------------------------------------------------------------------+
   | soma : mind     glia : infer     worlds : face     honeycrisp : silicon          |
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
                                                           | neuron context -> ward/vault -> admit/publish
                                                           v
   +--------------------+               +------------------------------------+      +--------------+
   | cyberia : social   |               | cybergraph                         |      | fs           |
   +--------------------+               +------------------------------------+ patch+--------------+
   | . contract         |  cyberlinks   | :  link                            |<-----| :  mount     |
   | . service          |-------------->| the  spine                         |      +--------------+
   | mimi  midao  my    |  focus . karma|                                    |
   +--------------------+               |                                    |
    mudra -----public evidence--------->|                                    |
    tok -------value ------------------>|                                    |
                                        +-------+-----------+-----------+----+
                                                |           |           |
                                                |store      |sync       |transmit
                                                v           v           v
                                            +-------+   +-------+   +-------+
                                            |  bbg  |   |foculus|   | tade  |
                                            +-------+   +-------+   +-------+
                                            |:store |   | :sync |   |:frame |
                                            +-------+   |:agree |   +-------+
                                                |       +-------+       |
                                                |verify                 |frames
                                                |                       |
                                                |                       v
                                                |                   +-------+
                                                |                   | radio |
                                                |                   +-------+
                                                |                   |:trans |
                                                |                   +-------+
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

## docs in this folder

| page | role |
|------|------|
| this page | foundations whitepaper — methods under the stack |
| [execution model](../specs/execution-model.md) | normative execution architecture and composition invariants |
| [[soft3/docs/launch\|launch spacepussy-test]] | how to launch the product network (chaosnet) |
| [[soft3/docs/chains-as-plugins\|chains as plugins]] | existing chains as adapters over the substrate — and how a transport actually gets adopted |
| [[soft3/docs/polynomial-proof-system\|polynomial proof system]] | the technical companion to "one form/one proof" above — the five operations (commit, open, verify, fold, identify), the numbers behind them, and what they make possible |

see [[soft3]] for the component stack and the troika compass.
