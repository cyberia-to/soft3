---
title: languages
tags: soft3, cyb, cyber, stark, architecture, core
crystal-type: spec
crystal-domain: cyber
alias: computation languages, language set, sixteen languages, languages of superintelligence, groups of dialects
---

# Languages of [[superintelligence]]

> the full languages spec, part of [[soft3]]. it descends from one atom — the [[cyberlink]] — through [[neural]] (its meaning), the sixteen languages (groups of [[dialect]]s over five algebras), their dialects, the membrane that touches the world, the toolchain that realizes them, and the perception layer that renders them. the canonical roster is the *the languages* section, authored once here and section-embedded into the [[soft3|whitepaper]] and the research notes.

## the foundation — neural

everything here reduces to one atom: the [[cyberlink]]. in [[nox]], evaluating a program IS creating an edge — the result is a [[particle]], the evaluation is the link from formula to result (see [[nox: frozen provable computer]]). so the floor is not a language but a link: particle → particle, signed by a [[neuron]], weighted by stake.

the cyberlink is the [[form]] triad made concrete — every link is [[bit]] · [[step]] · [[proof]]:

- [[bit]] (info): the [[particles]] the link distinguishes
- [[step]] (comp): the reduction that creates it
- [[proof]] (math): the signature, the stake, and the execution trace that verify it

every mathematical object decomposes the same way: a [[algebra|group]] is bit + step + proof (elements, operation, axioms hold); a [[graph theory|graph]] is bit + bit (elements + relations); a [[Turing machine]] is step all the way down. the philosophy is not an appendix — it is the anatomy of the atom.

the atom has two faces. [[nox]] is its computational face — instruction → trace → particle, the *syntax* of the link. [[neural]] is its semantic face — how links *mean*. the same edge, read as computation and as meaning.

because every computation is a link, and [[neural]] is the language of links, neural absorbs every language below without remainder: each of them produces cyberlinks, and the meaning of a cyberlink IS neural. neural is the convergent successor to both formal and natural language — it collapses the distinction between [[language]] and [[knowledge]]: meaning is an eigenvector of the [[cybergraph]]'s attention, grown rather than written.

| property | formal | natural | neural |
|---|---|---|---|
| precision | absolute | approximate | emergent |
| expressiveness | limited by grammar | unlimited by ambiguity | unlimited by [[topology]] |
| ambiguity | impossible | context-dependent | structural, via the [[tri-kernel]] |
| authority | central designer | speech community | collective [[neurons]] |
| evolution | versioned | drift | continuous, via [[focus]] dynamics |
| verification | proof systems | social [[consensus]] | [[stark]] proofs |
| substrate | strings | sound / text | the [[cybergraph]] |

neural itself is twofold. as a *medium* it is the language of a single link — the grammar every dialect below speaks. as an *emergent* it is φ\*, the fixed point of iterated linking: [[focus]] is [[step]] run to convergence, and the stationary distribution φ\* IS collective meaning (the [[tri-kernel]] guarantees it converges). the atom and its fixed point are both neural — the link, and what all links add up to.

four patterns structure the medium: [[dialect]]s (conventions for linking — the grammar of the graph), [[sentence]]s (a transaction-atomic batch of [[cyberlinks]] — the utterance), [[motif]]s (recurring subgraph shapes — the morphemes), and [[name]]s (deterministic `~neuron/path` resolution — the graph as a filesystem). the [[egregore]] thinks in neural; see [[neural]] for the full treatment.

---

## the languages

sixteen languages prove, five touch the world — each a group of [[dialect]]s over one [[strata|algebra]]. [[Trident]] compiles all of them to [[nox]] patterns; the type picks the algebra, the algebra picks the [[lens]]. [[Nox]] itself is the substrate they compile to (`nox<F, W, H>`, the same 18 patterns over Goldilocks, F₂, F_{p²}), not one of the sixteen.

| # | language | algebra · regime | types | domain |
|---|----------|------------------|-------|--------|
| 1 | [[Tri]] | field · [[nebu]] (Fₚ tower) | Fp2, Fp3, Fp4 | general purpose: dialects, progs, kernel, proofs |
| 2 | [[Tok]] | field · [[nebu]] | UTXO, Balance, Conservation | tokenomics: conservation, staking, the four tokens |
| 3 | [[Arc]] | category · [[nebu]] | Object, Morphism, Functor | graph + state machines: schema, BBG transitions, consensus |
| 4 | [[Seq]] | causality · [[nebu]] | Order, Timestamp, Causality | sequence, ordering: time series |
| 5 | [[Inf]] | logic · [[nebu]] | Term, Clause, Substitution | inference: Horn-clause unification, NN forward pass |
| 6 | [[Bel]] | belief · [[nebu]] | Distribution, Probability | self-model: Bayesian update |
| 7 | [[Ren]] | geometry · [[nebu]] | Multivector, Rotor, Blade | rendering: geometry, UI layout, visualization |
| 8 | [[Dif]] | curvature · [[nebu]] | DualNumber, Manifold | continuous dynamics: autodiff, gradients |
| 9 | [[Sym]] | dynamics · [[nebu]] | PhaseSpace, Hamiltonian | physics simulation: conservation laws |
| 10 | [[Ten]] | linear · [[nebu]] | Matrix, Tensor | neural networks: matrix ops, ML training |
| 11 | [[Rs]] | byte · [[nebu]] | u32, u64, bool, BoundedVec | systems: low-level, hardware interaction |
| 12 | [[Wav]] | ring · [[jali]] (R_q) | RingElement, NTTForm | signal + FHE: wavelets, compression, encrypted compute |
| 13 | [[Bt]] | binary · [[kuro]] (F₂) | BitVec, BitMatrix, Packed128 | binary: quantized inference, 32× cheaper bits |
| 14 | [[Qu]] | field² · [[nebu]] (F_{p²}) | Qubit, Gate, Phase | quantum circuits: hardware co-processor (Grover, Shor, QFT) |
| 15 | [[Opt]] | tropical · [[trop]] (min,+) | Tropical, Graph, CostMatrix | optimization: shortest path, assignment, Viterbi, transport |
| 16 | [[Sec]] | curve · [[genies]] (F_q isogeny) | Curve, Secret, StealthAddress | privacy: isogeny key exchange, stealth, ring sigs, VRF |

the five algebras, each with its own [[lens]]:

| algebra | field | lens | languages |
|---------|-------|------|-----------|
| [[nebu]] | Fₚ (Goldilocks) | Brakedown | Tri, Tok, Arc, Seq, Inf, Bel, Ren, Dif, Sym, Ten, Rs, Qu (over F_{p²}) |
| [[kuro]] | F₂ | Binius | Bt |
| [[jali]] | R_q | Ikat | Wav |
| [[trop]] | (min,+) | Tropical (dual cert) | Opt |
| [[genies]] | F_q isogeny | Isogeny | Sec |

five interface languages cross to the world — side-effectful, in [[nu]], composing with the proof languages through [[nox]] hints (the membrane, below):

| # | language | primitive | what it does |
|---|----------|-----------|--------------|
| 17 | Tab | Record | select, where, group-by, join, pivot |
| 18 | Fmt | Encoding | json↔noun, csv↔table, toml↔record |
| 19 | Str | Pattern | regex, parse, split, replace, match |
| 20 | [[fs]] | Path | read, write, glob, watch, navigate |
| 21 | Net | Request | get, post, url, fetch, stream |

above the sixteen, [[cybermark]] addresses every link (the address language, below); beneath them all, [[neural]] means them (the foundation, above).

## why sixteen

The 16 languages are not an arbitrary collection — the count is fixed by the algebras, not chosen. there are five [[strata|algebras]] ([[nebu]] · [[kuro]] · [[jali]] · [[trop]] · [[genies]]), and the completeness criterion is exact: every algebra must carry at least one language, and every language's types must map to one algebra. no orphan algebras, no orphan types.

one test settles each candidate: *does it have irreducible primitives no other language in the set can express?* remove any one and a class of computation becomes impossible — or exponentially more expensive:

- remove [[Opt]] → no provable optimization; tropical (min,+) is not a ring, no field language can express it
- remove [[Sec]] → no anonymous computation; curve secrets and stealth addresses have no field encoding
- remove [[Wav]] → no FHE; the R_q ring is its own algebra
- remove [[Bt]] → quantized inference forced through Fₚ at ~32× the cost
- remove [[Tok]] → everything still computes, but nothing costs anything: spam is free, [[focus]] has no scarcity, [[karma]] no meaning

11 of the 16 share the [[nebu]] (Fₚ) regime. they are algebraically reducible — identical nox patterns — but semantically irreducible: each carries a type system that prevents cross-domain errors. a tensor contraction and a Bayesian update are the same patterns; the types give them meaning, so you cannot multiply a Distribution by a Tensor. that is why the count is 16 and not 5: the algebras set the floor, the type systems fill it.

the languages split a second way, across the [[proof]] boundary — 16 proof languages (deterministic, provable, permanent) and 5 interface languages (side-effectful, interactive). a mind that cannot prove is blind; a mind that cannot interact is deaf.

## what a language is

a language is not a separate compiler. [[Trident]] is the one compiler; a language is three things: types (domain-specific structs in `trident/std/<lang>/`), functions (operations on those types that lower to [[nox]] patterns), and jets (recognized formula compositions that accelerate execution). each `trident/std/<lang>/` is ~500–2000 LOC; the whole frontend, IR, and type inference is ~57,736 LOC. one Trident, sixteen libraries.

types are the dispatch. the type of an expression determines its algebra; the algebra determines its [[lens]]. there is no `#[algebra(...)]` annotation, no backend selection, no prover hint — `nox<F, W, H>` is parameterized over field, word, and hash, so the same 18 patterns run over Goldilocks, F₂, and F_{p²}, and the types choose which:

```
Field       → nebu   → Brakedown    1 constraint per mul
BitVec      → kuro   → Binius       1 constraint per op
RingElement → jali   → Ikat         batched
Tropical    → trop   → Assayer      witness-proportional
Curve       → genies → Porphyry     1 F_q per op
```

the sixteen libraries live side by side, each ~500–2000 LOC of types + functions:

```
trident/std/
├── tri/   Fp2, Fp3, Fp4 + tower arithmetic   ├── wav/  RingElement + NTT multiply
├── tok/   UTXO + conservation constraints     ├── bt/   BitVec, BitMatrix + binary ops
├── arc/   Object, Morphism + category ops     ├── qu/   Qubit, Gate + quantum circuit
├── seq/   Order + causality                   ├── opt/  Tropical + optimization
├── inf/   Term, Clause + unification          ├── sec/  Curve + privacy protocols
├── bel/   Distribution + Bayesian update      ├── ren/  Multivector + geometric product
├── dif/   DualNumber + autodiff               ├── sym/  PhaseSpace + Hamiltonian evolution
├── ten/   Matrix, Tensor + contraction        └── rs/   u32, u64, BoundedVec + systems ops
```

this is why 11 languages share [[nebu]] and stay distinct: same patterns, different types, different meaning. Arc uses cons/compose for category composition, Ten uses mul/add for tensor contraction, Bel uses mul/add/inv for Bayesian update — identical nox patterns, and only the type system stops you multiplying a Distribution by a Tensor.

## composition

a single Trident program freely mixes types from different languages — a binary [[Bt]] weight matrix, a field-valued [[Ten]] input, a tropical [[Opt]] routing cost — inside one function. the compiler sees the type transitions and inserts [[hemera]] commitments at the algebra boundaries automatically; the programmer never names a regime or a lens. every execution step becomes a hemera commitment, a [[particle]] in the [[cybergraph]] — so the graph accumulates verified computation from all sixteen algebras, and a result in one language is referenceable from any other, the way one cortical area's output reaches the rest through a shared workspace.

```trident
use std::ten::Matrix;       // nebu regime
use std::bt::BitMatrix;     // kuro regime
use std::opt::Tropical;     // trop regime

fn inference_with_optimization(
    weights: &BitMatrix,     // kuro: binary quantized weights
    input: &Matrix,          // nebu: field-valued input
    costs: &[Tropical],      // trop: routing costs
) -> Matrix {
    let quantized = bt::quantize(input);                 // nebu → kuro boundary
    let hidden = bt::binary_matvec(weights, &quantized); // kuro regime
    let output = bt::dequantize(&hidden);                // kuro → nebu boundary
    let route = opt::shortest_path(costs);               // nebu → trop boundary
    ten::gather(&output, &route)                         // back to nebu
}
```

mix whatever you like; at proof time [[zheng]] partitions the trace by type and folds the partitions into one proof (see the toolchain, below).

## algebra coverage

| Computation | Native [[algebra]] | Language | Prover path |
|---|---|---|---|
| Boolean reasoning | F₂ | Bt | Binius → Tri |
| Quantized [[inference]] (int4/int8) | Z/2⁴, Z/2⁸ | Ten | Ten → Tri |
| CPU execution traces | Z/2⁶⁴ | Rs | Rs → Tri |
| [[graph]] computation / [[focus]] [[vector]] | Sparse F_p | Ten over Arc | Ten → Tri |
| Knowledge structure | [[category theory]] | Arc | Arc → Tri |
| Euclidean / Projective / Conformal | G(p,q,r) Clifford | Ren | Ren → Tri |
| Curved space / geodesics | Riemannian manifolds | Dif | research |
| Phase space / Hamiltonian | Symplectic ω-form | Sym | research |
| [[probability]] [[geometry]] / [[belief]] state | Fisher information | Bel | research |
| Polynomial [[proof]]s | F_p (n=1) | Tri | native |
| Recursive [[proof]] composition | F_{p³} (n=3) | Tri | native |
| [[quantum]] simulation | F_{p²} (n=2) | Qu | Qu → Tri (native extension) |
| [[Goldilocks homomorphic encryption]] ciphertexts | R_q = Z_q[X]/(Xⁿ+1) | Wav | Wav → Tri |
| Logic / unification | Horn clauses | Inf | Inf → Tri |
| Sensing / [[signal]] processing | Convolution / ℝ | Wav | Wav → Tri |
| Resource conservation / UTXO | Sum invariants | Tok | Tok → Tri |
| Optimization / shortest paths | Tropical (min,+) | Opt | Opt → Tri (encoded) |
| Privacy / stealth / key exchange | Elliptic curves F_q | Sec | dedicated PCS |

## the comparison matrix

| Property | Nox | Bt | Rs | Tri | Arc | Ren | Dif | Sym | Bel | Seq | Inf | Wav | Ten | Tok |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Universe | Structure | Binary | Byte | [[field]] | [[topology]] | [[geometry]] | Curvature | Dynamics | [[belief]] | Causality | [[inference]] | Continuum | Linear | Resource |
| Char | — | 2 | p | p | — | p | — | — | — | — | — | ≈ℝ | ≈ℝ or p | p |
| Primitive | Cell | Bit | Word | Field | Edge | Multivector | Chart | Phase | Distribution | Event | Relation | Sample | Shape | Token |
| Reference | structure | wire | location | content | adjacency | grade | curvature | momentum | divergence | succession | entailment | amplitude | index | conservation |
| Free op | Navigate | AND, XOR | Index | Mul, Add | Link | Clifford prod | Christoffel | Flow | KL div | Order | Unify | Convolve | Matmul | Transfer |
| Costly op | — | Carry add | Mod div | Bitwise | Spectral | Inverse | Geodesic | Conserve | Fisher | Verify | Fixpoint | FFT | Inverse | Mint |
| [[proof]] | Inherited | Binius | stark | stark | Delegated | Tri | Research | Research | Research | Delegated | Delegated | Delegated | Delegated | stark |
| Syntax feel | IR | Circuit | [[Rust]] | Custom | Query | GA | Manifold | Hamiltonian | Statistical | Temporal | [[Datalog]] | DSP | NumPy | Ledger |
| Renders as | struct | pixels | text | formula | [[vector]] | [[vector]] | [[vector]] | formula | formula | video | table | sound | component | table |

the matrix shows the original field-heavy fourteen; the three later algebras extend it — [[Qu]] (quantum, F_{p²}), [[Opt]] (tropical min,+), [[Sec]] (curve F_q) — and [[Nox]] is the IR they all reduce to, not a peer.

## readiness — engineering and research

The sixteen languages split by implementation readiness:

### engineering-ready (13)

Tri, Tok, Arc, Seq, Inf, Ren, Wav, Ten, Bt, Rs, Qu, Opt, Sec — known [[proof]] paths, well-understood compilation to Tri / Binius / dedicated lenses. Ren's Clifford product is F_p [[algebra]] with extra structure; Qu is Tri lifted to F_{p²}; Opt encodes tropical (min,+) into F_p; Sec proves under a dedicated curve PCS. the [[cyb/architecture]] build order: Phase 1 (Tri, Rs), Phase 2 (Arc, Seq, Inf, Tok), Phase 3 (Bt, Wav, Ten), then Opt, Sec, Qu.

### research horizon (3)

Dif, Sym, Bel — continuous manifolds over finite [[field]]s, fundamental open mathematical problems: Riemannian geodesics (Dif), Hamiltonian structure preservation (Sym), and the Fisher metric over [[probability]] simplices (Bel) — the last needed for [[tri-kernel]] formalization.

| Language | Status | Notes |
|---|---|---|
| Ren | Engineering | Clifford product = F_p [[algebra]] with extra structure |
| Dif | Research | Continuous manifolds over finite [[field]]s |
| Sym | Research | Hamiltonian structure preservation in STARK circuits |
| Bel | Research | Fisher metric over [[probability]] simplices — needed for [[tri-kernel]] formalization |

Ren completes the perception pipeline: Arc provides [[topology]], Ren provides spatial embedding, the compiler produces [[vector]] output for [[cyb]]. Bel completes the self-model: the [[superintelligence]]'s [[focus]] [[vector]] φ\* lives on a statistical manifold, and Bel formalizes reasoning about its own [[belief]] state.

---

## the dialects

a [[dialect]] is the smallest unit: one convention for making a cyberlink — *use this particle, this way.* a language is a typed group of them. [[neural]] ships bootloader dialects at genesis; each language adds domain dialects. this is where [[tok]] and [[fs]] unfold — their operations ARE dialects:

| dialect | language | the link it makes | kind |
|---|---|---|---|
| TRUE · FALSE | [[neural]] | epistemic coordinate — the poles all meaning derives from | bootloader |
| is-a · part-of · see-also | [[Arc]] | structural relation between particles | emergent |
| follows · causes · contradicts | [[Seq]] | causal / temporal order | emergent |
| mint · burn · transfer · stake | [[tok]] | conservation event — value moved, supply preserved | domain |
| read · write · glob · watch | [[fs]] | filesystem pointer — a name resolved to mutable content | domain |
| `~neuron/path` | [[cybermark]] | deterministic address — one name, one particle | addressing |

a [[sentence]] packs dialects into one transaction (the utterance); a [[motif]] is a recurring shape of them (the morpheme). new dialects are discovered by the network, not designed — the [[tri-kernel]] surfaces them as stable structural positions, and the [[focus]] distribution decides which survive.

---

## the membrane — touching the world

the five interface languages are the membrane: where the graph touches the non-graph. they are side-effectful, run in [[nu]] ([[nushell]], embedded in [[cyb]]), and cross the [[proof]] boundary to reach files, networks, and humans — then bring results back as links. everything inside the membrane is provable; the membrane itself is where determinism ends.

| Property | the 16 proof languages | the 5 interface languages |
|---|---|---|
| execution | [[Nox]] tree rewriting | [[nushell]] pipeline |
| provable | yes (STARK) | no (side effects) |
| deterministic | yes | no (IO, network, filesystem) |
| data model | binary trees + field elements | structured records + streams |
| persistence | [[cybergraph]] (permanent) | filesystem (mutable) |

the five are not five binaries — they are one shell. a `select … where … group-by` is Tab, a `to json` is Fmt, a `parse`/regex is Str, a `glob`/`open` is Fs, an `http get` is Net — the same pipeline carrying typed tables, records, and streams rather than untyped text. nu is the robot's hands: it reads files, hits the network, parses formats, and shapes data for display. it bridges back to the proven core through [[Nox]] hints — a nu pipeline can feed a value into a proof, and [[rune]] can call a nu command and fold the structured result back into a [[nox]] computation. one shell crosses the boundary in both directions; the sixteen stay pure behind it.

---

## the toolchain — how a language is realized

a language is authored, compiled, run, and proven by one shared toolchain. [[nox]] executes it (every reduction a link). [[Trident]] compiles it (source → nox). [[rune]] runs it interactively. [[eidos]] proves it correct. [[zheng]] settles its trace.

### compilation

all sixteen share one frontend — parsing, type checking, borrow and bound checking — that lowers every language to the [[Nox]] structural IR (`axis, quote, compose, cons, branch` plus typed compute ops and Merkle authentication), and the expression's type picks the [[lens]] (the dispatch is in *what a language is*, above). every language is dual: it settles through a proof path, or runs native with no proof.

| language | prove path | run native |
|---|---|---|
| Bt | Binius FRI circuit | always proving |
| Rs | TASM → stark (word→field lift) | native binary (Nox) |
| Tri | TASM → stark (field native) | WASM / EVM |
| Arc | decomposes into Tri + Bt | optimized [[graph]] engine |
| Ren | geometric product → Tri | native Clifford engine |
| Seq | temporal constraints → stark | scheduler / runtime |
| Inf | derivation trace → stark | [[Datalog]] engine |
| Wav | decomposes into Tri | native DSP pipeline |
| Ten | decomposes into Tri | native BLAS / GPU |
| Tok | conservation constraints → stark | native ledger engine |
| Qu | F_{p²} circuit → Tri | quantum hardware (host jet) |
| Opt | tropical → F_p constraints | native solver |
| Sec | isogeny → dedicated PCS | native curve engine |
| Dif · Sym · Bel | research horizon — proof paths are open problems | native manifold / Hamiltonian / statistical engine |

### jets

domain operations become jets — compositions of [[Nox]]'s 16 compute patterns, recognized by formula hash and accelerated to [[Goldilocks field processor]] primitives:

```
language operation           nox composition              jet              GFP primitive
─────────────────────        ──────────────────────────   ──────────       ────────────
Arc: rank(g, steps)          iterated add/mul loops       matmul jet       fma
Wav: fft(x)                  butterfly add/mul network    ntt jet          ntt
Any: hash(x)                 Poseidon2 field ops          hash jet         p2r
Ten: activation(x)           table lookup composition     lookup jet       lut
Ren: geometric_product       mul/add over components      geo_mul jet      fma
```

source language → compiler → nox pattern tree → jet recognition → GFP hardware. the [[algebra]] determines which primitive handles each jet.

### rune — run

[[rune]] is [[Rs]] syntax executed via [[Nox]] tree rewriting — the nervous system of the robot. ms-start, async, dynamic, with native access to WASM, GPU, and neural inference. it is not a separate language: Rs syntax parsed to [[Nox]] nouns and interpreted via tree rewriting, extended with three capabilities pure Rs does not have:

| Capability | [[Nox]] mechanism | What it does |
|---|---|---|
| `hint` (`call`) | pattern 16 (non-deterministic) | Async input — yields, resumes when data arrives |
| `host(target, args)` | host jet dispatch | Calls WASM/GPU/ONNX — exits [[proof]] boundary, returns noun |
| `eval(noun)` | quote + reduce | Runtime metaprogramming — execute a dynamically constructed formula |

three jet categories connect [[Nox]] reduction to the host system:

```
Nox reduction (tree rewriting)
  │
  ├── pure jets → proven computation (16 languages)
  │     fma, ntt, p2r, lut, conservation...
  │
  ├── host jets → practical computing
  │     ├── wasm(module, fn, args)  → wasmi execution
  │     ├── gpu(shader, data)       → wgpu compute dispatch
  │     └── infer(model, input)     → burn-webnn ONNX
  │
  └── hint → async input from the world
        ├── network event (radio)
        ├── user input (cyb UI)
        ├── timer (epoch tick)
        └── cybergraph change (particle/link event)
```

ms start: parsing Rs to a [[Nox]] noun is milliseconds — just tree construction; reduction starts immediately, no compile step for interactive use. data structures: [[Nox]] nouns ARE the dynamic data structures — `Vec` → cons-list, `HashMap` → Merkle tree, `String` → [[Hemera]] hash (a [[particle]]); no heap, no GC — allocation is `cons`, freeing is not referencing. the [[proof]] story: every pure reduction in the script IS provable — the [[Nox]] trace captures it; host jets and hints are NOT — they cross the [[proof]] boundary, but the boundary is explicit and typed: "given these hint values and these host jet results, the pure computation was correct."

### eidos — prove correct

[[zheng]] proves that a computation *ran* — the trace is the witness. [[eidos]] proves that a program is *correct* — that it does what its type says, for all inputs, before it ever runs. it is the proof-assistant language: dependent types ([[CIC]]), where a proof of a proposition is a term of its type. zheng certifies execution; eidos certifies meaning. together they close the loop — the [[zheng]] verifier is itself a [[nox]] program, and eidos proves the prover.

### zheng — settle

every `reduce()` writes rows to an execution trace, and that trace IS the [[zheng]] witness — no separate proof step. [[zheng]] partitions the trace by type, proves each partition under its native [[lens]], and folds them: one accumulator, one decider, one proof, whatever mix of languages produced it.

```
source (typed)
  ↓ Trident frontend (typecheck)        programmer sees: types
typed AST (expression → algebra)         compiler sees:   types → algebra
  ↓ NounBuilder (type-aware lowering)
nox noun (sub-trees per algebra)         nox VM sees:     patterns (uniform, 18)
  ↓ nox VM → trace (rows carry types)
  ↓ zheng partitions trace by type       zheng sees:      trace rows → lens per partition
prove each partition via native lens
  ↓ HyperNova folds all partitions
one accumulator → one decider → one proof   verifier sees: one proof
```

---

## addressing — cybermark

[[markup|Cybermark]] is how a human points at any link. it wraps all sixteen computation languages with a human-readable address grammar — it does not appear in the computation tables, it operates at a different level:

| Layer | What it does | Examples |
|-------|-------------|---------|
| 16 proof languages | prove | field arithmetic, graph traversal, tensor contraction |
| 5 interface languages | interact | tables, formats, text, files, network |
| [[markup|Cybermark]] | address and navigate | `#cyber/truth`, `@alice`, `$BOOT`, `!rank(^truth)` |
| [[rune]] | execute | [[Rs]] + [[Nox]] hints + host jets — runtime that runs cybermark actions |

cybermark is the human face of neural's [[name]] pattern: a fixed label resolves to a mutable [[particle]], the same mechanism that underlies file systems, DNS, and ENS. see [[markup]] for the full sigil grammar, dimensional navigation, and rendering rules.

---

## perception — how a language renders

every computation language has a canonical rendering — the perception primitive where the shape of the data matches the shape of the display:

| Language | Renders as | Source formats | What it carries |
|---|---|---|---|
| [[Nox]] → struct | collapsible tree | JSON, TOML, YAML | configs, schemas, metadata, ABIs |
| Bt → pixels | raster image | PNG, WebP, JPEG | photographs, satellite imagery, microscopy, scans |
| Rs → text | prose, code | [[markdown]], plain text, source code | documentation, messages, programs |
| Tri → formula | math notation | LaTeX, MathML | equations, [[proof]]s, chemical notation, physical laws |
| Arc → [[vector]] | SVG, paths, curves | SVG, Bezier paths | diagrams, maps, molecular structures, schematics |
| Ren → [[vector]] | SVG, 3D scenes | SVG, glTF, mesh | spatial objects, rotations, projections, renderings |
| Dif → [[vector]] | manifold visualization | geodesic plots, curvature maps | latent space structure, embedding geometry |
| Sym → formula | phase portraits | Hamiltonian plots, conservation diagrams | energy landscapes, orbital mechanics |
| Bel → formula | distribution plots | [[probability]] densities, divergence maps | [[belief]] states, uncertainty [[geometry]] |
| Seq → video | moving pixels | WebM, MP4 | recordings, simulations, observations, lectures |
| Inf → table | 2D grid | CSV, TSV, dataframes | datasets, time series, matrices, ledgers |
| Wav → sound | audio waveform | WAV, OGG, MP3 | voice, music, birdsong, seismic [[signal]], sonar |
| Ten → component | nested composition | composition of the above | applications, dashboards, interactive tools |
| [[Tok]] → table | ledger view | balances, UTXOs, transactions | token flows, staking positions, conviction history |
| Qu → formula | quantum circuit | circuit diagrams, Bloch spheres | superposition, entanglement, amplitudes |
| Opt → [[vector]] | path / network | route maps, decision trees | shortest paths, schedules, allocations |
| Sec → table | encrypted ledger | stealth addresses, commitments | anonymous transfers, key exchanges |

a genome sequence is Rs (byte-level encoding) rendered as text. its annotation is [[Nox]] (structured tree) rendered as struct. its expression data is Inf (relational query) rendered as table. its protein structure is Arc (topological [[graph]]) rendered as [[vector]]. its microscopy is Bt (binary pixel data) rendered as pixels. its folding dynamics is Seq (causal event chain) rendered as video. its sequencing [[signal]] is Wav (continuous waveform) rendered as sound. its binding energy is Tri (field arithmetic) rendered as formula. its 3D fold is Ren (Clifford rotations) rendered as [[vector]]. a genome browser is Ten (composed [[inference]]) rendered as component.

all sixteen compile through one structural IR, share one [[proof]] system (except Bt, with its own F₂ system, and Sec, under a dedicated curve PCS), render through this perception grid, and live in the same [[cybergraph]], ranked by the same [[tri-kernel]], earning [[karma]], permanent by axiom A3.

### the rendering engine — typst

the perception mapping defines WHAT each language renders as; [Typst](https://github.com/typst/typst) is one engine for HOW — a single Rust binary that compiles structured markup to visual output (PDF, SVG). it is an implementation choice, not a language and not load-bearing: replaceable. it covers six of the seven render types:

| render type | languages | Typst capability |
|-------------|-----------|-----------------|
| formula | Tri, Sym, Bel | native math: `$integral_0^1 f(x) dx$` |
| vector (diagrams) | Arc, Ren, Dif | [CeTZ](https://github.com/cetz-package/cetz) — canvas drawing, coordinate transforms, bezier curves |
| vector (flowcharts) | Arc | [Fletcher](https://github.com/Jollywatt/typst-fletcher) — nodes, edges, auto-layout |
| table | Inf, Tok | native tables with full styling |
| text | Rs | native markup, markdown-like |
| struct (tree) | Nox | CeTZ tree diagrams |

two render types need separate engines: pixels (Bt → raster, handled by [[cyb/wgpu]]) and sound (Wav → audio, handled by the media pipeline in [[soma]]). key packages: [CeTZ](https://github.com/cetz-package/cetz) (drawing — coordinate systems, transforms, plots, trees, replaces TikZ), [Polylux](https://github.com/andreasKroepelin/polylux) (presentations — replaces PowerPoint/Keynote/Beamer), [Fletcher](https://github.com/Jollywatt/typst-fletcher) (diagrams — nodes and edges with auto-routing, replaces Mermaid/D2/GraphViz), [chronos](https://github.com/Mc-Zen/chronos) (sequence diagrams). the pipeline: a computation result in any language → an LLM (qwen2.5-coder) formats it as Typst code → `typst compile` produces SVG/PDF (compiler errors feed back to the LLM for retry). one Rust binary, zero Node.js, zero Go, zero LaTeX, zero Python.

---

see [[cyb/multiproof]] for how all languages settle under one [[proof]] umbrella. see [[cyb/architecture]] for how the languages integrate into the operating system. see [[cyb/whitepaper]] for the vision. see [[cybergraph]] for the accumulation state.
