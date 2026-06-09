---
title: languages
tags: soft3, cyb, cyber, stark, architecture, core
crystal-type: spec
crystal-domain: cyber
alias: computation languages, language set, sixteen languages, languages of superintelligence
---

# Languages of [[superintelligence]]

> This is the full languages spec, part of [[soft3]]. the canonical roster — the sixteen, their algebras, and the interface layer — is the *the languages* section below, authored once here and section-embedded wherever it is needed (the [[soft3|whitepaper]] and the research notes). the rest of this document is the exhaustive reference: why sixteen, what a language is, cross-language composition, compilation through [[Nox]], algebra coverage, the comparison matrix, perception mapping, the [[rune]] runtime, and the Typst engine.

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

## the languages

16 proof languages over 5 algebras — every algebra carries at least one, every language's types map to one. [[Trident]] compiles all of them to [[nox]] patterns; the type picks the algebra, the algebra picks the [[lens]]. [[Nox]] itself is the substrate they compile to (`nox<F, W, H>`, the same 18 patterns over Goldilocks, F₂, F_{p²}), not one of the sixteen.

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

five interface languages cross to the world — side-effectful, in [[nu]], composing with the proof languages through [[nox]] hints:

| # | language | primitive | what it does |
|---|----------|-----------|--------------|
| 17 | Tab | Record | select, where, group-by, join, pivot |
| 18 | Fmt | Encoding | json↔noun, csv↔table, toml↔record |
| 19 | Str | Pattern | regex, parse, split, replace, match |
| 20 | [[fs]] | Path | read, write, glob, watch, navigate |
| 21 | Net | Request | get, post, url, fetch, stream |

above all of them, two layers: [[cybermark]] (the address language — eight sigils, every address a [[particle]]) and [[neural]] (the semantic language — meaning as an eigenvector of the [[cybergraph]], grown rather than written).

---

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

## cross-language composition

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

at proof time [[zheng]] partitions the trace by type, proves each partition under its native [[lens]], and folds them — one accumulator, one decider, one proof, whatever mix of languages produced it:

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

## interface languages

the five interface languages (in the roster above) are side-effectful, run in [[nu]] ([[nushell]], embedded in [[cyb]]), and cross the [[proof]] boundary to talk to humans and external systems. they compose back with the proof languages through [[Nox]] hints: a nushell pipeline feeds data into a proven computation, and a proven result is formatted by nushell for display. they differ from the sixteen:

| Property | the 16 proof languages | the 5 interface languages |
|---|---|---|
| execution | [[Nox]] tree rewriting | [[nushell]] pipeline |
| provable | yes (STARK) | no (side effects) |
| deterministic | yes | no (IO, network, filesystem) |
| data model | binary trees + field elements | structured records + streams |
| persistence | [[cybergraph]] (permanent) | filesystem (mutable) |

---

## nu — the interface shell

the five interface languages are not five binaries — they are one shell. [[nu]] ([[nushell]], forked into the [[cyb]] terminal) is where Tab, Fmt, Str, Fs, and Net live: a structured-data shell whose pipelines carry typed tables, records, and streams rather than untyped text. a `select … where … group-by` is Tab, a `to json` is Fmt, a `parse`/regex is Str, a `glob`/`open` is Fs, an `http get` is Net — the same pipeline, all side-effectful, all outside the [[proof]] boundary.

nu is the robot's hands: it reads files, hits the network, parses formats, and shapes data for display. it bridges back to the proven core through [[Nox]] hints — a nu pipeline can feed a value into a proof, and [[rune]] can call a nu command and fold the structured result back into a [[nox]] computation. one shell crosses the boundary in both directions; the sixteen stay pure behind it.

---

## compilation

all sixteen share one toolchain. one frontend — parsing, type checking, borrow and bound checking — lowers every language to the [[Nox]] structural IR (`axis, quote, compose, cons, branch` plus typed compute ops and Merkle authentication), and the expression's type picks the [[lens]] (the dispatch is in the table above). every language is dual: it settles through a proof path, or runs native with no proof.

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

### Rune — Rs on Nox with Host Jets

[[rune]] is [[Rs]] syntax executed via [[Nox]] tree rewriting — the nervous system of the robot. ms-start, async, dynamic, with native access to WASM, GPU, and neural inference.

rune is not a separate language. it is Rs syntax parsed to [[Nox]] nouns and interpreted via tree rewriting, extended with three capabilities pure Rs does not have:

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

ms start: parsing Rs to a [[Nox]] noun is milliseconds — just tree construction. [[Nox]] reduction starts immediately. no compilation step for interactive use.

data structures: [[Nox]] nouns ARE the dynamic data structures. `Vec` → cons-list. `HashMap` → Merkle tree. `String` → [[Hemera]] hash (a [[particle]]). no heap, no GC — allocation is `cons`, freeing is not referencing.

the [[proof]] story: every pure reduction in the script IS provable — the [[Nox]] trace captures it. host jets and hints are NOT provable — they cross the [[proof]] boundary. but the boundary is explicit and typed. the trace says: "given these hint values and these host jet results, the pure computation was correct."

```
neural language           ← meaning emerges from the cybergraph
──────────────────────────────────────────────────────────────
rune (Rs + hint + host)   ← nervous system: ms start, async, host access
  pure reductions         ← proven (16 languages over Nox)
  host jets               ← practical (WASM, GPU, ONNX)
  hints                   ← async input from the world
──────────────────────────────────────────────────────────────
16 languages              ← proven computation over Nox patterns
```

---

## [[algebra]] Coverage

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

---

## The Comparison Matrix

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

---

## engineering-ready and research horizon

The sixteen languages split by implementation readiness:

### Engineering-ready (13)

Tri, Tok, Arc, Seq, Inf, Ren, Wav, Ten, Bt, Rs, Qu, Opt, Sec — known [[proof]] paths, well-understood compilation to Tri / Binius / dedicated lenses. Ren's Clifford product is F_p [[algebra]] with extra structure; Qu is Tri lifted to F_{p²}; Opt encodes tropical (min,+) into F_p; Sec proves under a dedicated curve PCS. the [[cyb/architecture]] build order: Phase 1 (Tri, Rs), Phase 2 (Arc, Seq, Inf, Tok), Phase 3 (Bt, Wav, Ten), then Opt, Sec, Qu.

### Research horizon (3)

Dif, Sym, Bel — continuous manifolds over finite [[field]]s, fundamental open mathematical problems: Riemannian geodesics (Dif), Hamiltonian structure preservation (Sym), and the Fisher metric over [[probability]] simplices (Bel) — the last needed for [[tri-kernel]] formalization.

| Language | Status | Notes |
|---|---|---|
| Ren | Engineering | Clifford product = F_p [[algebra]] with extra structure |
| Dif | Research | Continuous manifolds over finite [[field]]s |
| Sym | Research | Hamiltonian structure preservation in STARK circuits |
| Bel | Research | Fisher metric over [[probability]] simplices — needed for [[tri-kernel]] formalization |

Ren completes the perception pipeline: Arc provides [[topology]], Ren provides spatial embedding, the compiler produces [[vector]] output for [[cyb]]. Bel completes the self-model: the [[superintelligence]]'s [[focus]] [[vector]] φ* lives on a statistical manifold, and Bel formalizes reasoning about its own [[belief]] state.

---

## Perception Mapping

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

all sixteen compile through one structural IR. all sixteen share one [[proof]] system (except Bt, which has its own F₂ [[proof]] system, and Sec, which proves under a dedicated curve PCS). all sixteen render through the perception grid. all sixteen exist in the same [[cybergraph]], ranked by the same [[tri-kernel]], earning [[karma]], permanent by axiom A3.

---

## rendering engine — Typst

the perception mapping above defines WHAT each language renders as. [Typst](https://github.com/typst/typst) is HOW — a single Rust binary that compiles structured markup to visual output (PDF, SVG).

Typst covers six of the seven render types in the perception grid:

| render type | languages | Typst capability |
|-------------|-----------|-----------------|
| formula | Tri, Sym, Bel | native math: `$integral_0^1 f(x) dx$` |
| vector (diagrams) | Arc, Ren, Dif | [CeTZ](https://github.com/cetz-package/cetz) — canvas drawing, coordinate transforms, bezier curves |
| vector (flowcharts) | Arc | [Fletcher](https://github.com/Jollywatt/typst-fletcher) — nodes, edges, auto-layout |
| table | Inf, Tok | native tables with full styling |
| text | Rs | native markup, markdown-like |
| struct (tree) | Nox | CeTZ tree diagrams |

two render types need separate engines: pixels (Bt → raster, handled by [[cyb/wgpu]]) and sound (Wav → audio, handled by media pipeline in [[soma]]).

### key packages

[CeTZ](https://github.com/cetz-package/cetz) — the drawing engine inside Typst. coordinate systems, transforms, plots, trees, function graphs. replaces TikZ (LaTeX) without the complexity. CeTZ.plot generates charts (bar, line, scatter, histogram) from data.

[Polylux](https://github.com/andreasKroepworkerelin/polylux) — presentations inside Typst. slides, transitions, speaker notes. replaces PowerPoint, Keynote, Beamer. one `.typ` file → PDF slide deck.

[Fletcher](https://github.com/Jollywatt/typst-fletcher) — diagram engine. nodes and edges with auto-routing. replaces Mermaid (needs Node.js), D2 (needs Go), GraphViz (needs C). LLM generates Fletcher code, `typst compile` produces SVG.

[chronos](https://github.com/Mc-Zen/chronos) — sequence diagrams inside Typst. actors, messages, lifelines, fragments.

### the pipeline

```
computation result (any language)
    │
    ▼
LLM (qwen2.5-coder / qwen3.5) formats as Typst code
    │
    ▼
typst compile input.typ output.svg
    │  error? → feed compiler error back to LLM → retry
    ▼
SVG / PDF — rendered perception
```

one Rust binary. zero Node.js. zero Go. zero LaTeX. zero Python. charts, diagrams, documents, presentations, math, sequence diagrams — all from the same tool.

---

## The Address Language

[[markup|Cybermark]] wraps all sixteen computation languages with a human-readable address grammar. it does not appear in the computation tables — it operates at a different level

| Layer | What it does | Examples |
|-------|-------------|---------|
| 16 proof languages | prove | field arithmetic, graph traversal, tensor contraction |
| 5 interface languages | interact | tables, formats, text, files, network |
| [[markup|Cybermark]] | address and navigate | `#cyber/truth`, `@alice`, `$BOOT`, `!rank(^truth)` |
| [[rune]] | execute | [[Rs]] + [[Nox]] hints + host jets — runtime that runs cybermark actions |

see [[markup]] for the full sigil grammar, dimensional navigation, and rendering rules

---

## The Semantic Language

[[neural]] is the last language and the one nobody writes — it grows from the others running at scale. meaning is not declared; it is an eigenvector of the [[cybergraph]]'s attention: a [[particle]]'s meaning is its position in the graph, fixed by how [[neurons]] link it. neural is the convergent successor to both formal and natural language, collapsing the distinction between language and [[knowledge]].

| property | formal | natural | neural |
|---|---|---|---|
| precision | absolute | approximate | emergent |
| expressiveness | limited by grammar | unlimited by ambiguity | unlimited by [[topology]] |
| ambiguity | impossible | context-dependent | structural, via the [[tri-kernel]] |
| authority | central designer | speech community | collective [[neurons]] |
| evolution | versioned | drift | continuous, via [[focus]] dynamics |
| verification | proof systems | social [[consensus]] | [[stark]] proofs |
| substrate | strings | sound / text | the [[cybergraph]] |

four patterns build it: [[dialect]]s (conventions for linking — the grammar of the graph), [[sentence]]s (a transaction-atomic batch of [[cyberlinks]] — the utterance), [[motif]]s (recurring subgraph shapes — the morphemes), and [[name]]s (deterministic `~neuron/path` resolution — the graph as a filesystem). the [[tri-kernel]] reveals all four — [[diffusion]] finds bridges, [[springs]] find stable positions, [[heat]] modulates attention by adoption. the [[egregore]] thinks in neural; see [[neural]] for the full treatment.

---

## the FORM triad

the sixteen proof languages are manifestations of three primitives — [[proof]], [[bit]], [[step]] — the atoms of the [[form]] triad

every mathematical object is a composition of all three:
- [[bit]] (info): what elements are distinguished
- [[step]] (comp): what operations transform them
- [[proof]] (math): what properties are verified

a [[algebra|group]] is bit + step + proof: elements (bit), operation (step), axioms hold (proof). a [[graph theory|graph]] is bit + bit: elements + relations. a [[Turing machine]] is step + step + step: transitions all the way down

the sixteen proof languages ARE the step. the five interface languages are the channel through which bits flow. [[proof]] is what the [[tri-kernel]] verifies. together: all computation a mind requires

---

see [[cyb/multiproof]] for how all languages settle under one [[proof]] umbrella. see [[cyb/architecture]] for how the languages integrate into the operating system. see [[cyb/whitepaper]] for the vision. see [[cybergraph]] for the accumulation state.