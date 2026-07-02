---
title: stack completeness
tags: cyber, soft3, roadmap, architecture
crystal-type: roadmap
crystal-domain: cyber
status: draft
---

# stack completeness — coverage, gaps, and the deep overlaps

a map of the whole [[soft3]] stack as layers, a verdict on what it covers end to end, and the gaps and overlaps that remain once the new components ([[eidos]], [[honeycrisp]], [[wysm]], [[neural]]) and the cyb-resident pieces ([[cyb/root/ward|ward]], cell) are placed. the network-tier borders are settled separately in [[component-boundaries]]; this document takes the full-stack view.

## the layered model

the stack is layered, not a flat list of repos. each layer reduces toward the one below it, and the three through-lines (one field, one proof, one focus) run vertically through all of them.

```text
app          cyb (embody)
value        tok (pay)
cognition    soma (think)
capability   ward (authorize)
present      mir (render world)        prysm (paint UI)
network      mudra · radio · tape · foculus
dynamics     tru (converge φ*)         foculus (agree)
graph        cybergraph · bbg · cell · fs
proof        zheng (execution)         eidos (theorem)
runtime      nox · wysm · glia · wgpu  (the proof-contract ladder)
language     trident · neural · rune · inf     (+ cybermark, nu host)
commit       hemera (hash)             lens (commit)
math         strata — nebu 𝔽_p · genies 𝔽_q · jali R_q
substrate    honeycrisp (accelerate)
```

two verbs share a layer when they are orthogonal readings of it, not duplicates: zheng proves execution while eidos certifies theorems; mir paints the spatial world while prysm paints the symbolic surface; tru converges the field while foculus decides finality on it.

## the proof-contract ladder

the runtime layer is four seats, ordered by how much trust their output carries. this is the spine of the execution model, already specified in [[wysm]]:

| runtime | compiler | what runs | proof contract |
|---|---|---|---|
| [[soft3/nox|nox]] | trident | proven .nox programs, jets | unconditional |
| [[glia]] | tru | .model inference | conditional on model |
| [[wysm]] | external | conventional WASM souls | conditional on host |
| wgpu | external | GPU compute shaders | conditional on host |

a soul starts in the weakest seat it can (wasm accepts any language that compiles to WebAssembly) and graduates toward nox as its hot loops are lowered through trident LIR. the conditional runtimes do not break provability — each host call returns a noun recorded as a witness in the calling rune program, so the surrounding pure computation stays provable conditional on that witness. trident LIR is the single waist every path funnels through.

## completeness verdict

the stack covers the full arc end to end: silicon → field → identity → language → execution → proof → graph → convergence → consensus → network → render → capability → cognition → value → app. nothing in that chain is missing a home. the three through-lines hold:

- one form — identity, state, data and proof are one multilinear form over one Goldilocks field ([[strata]] / nebu); composition is free because there is nothing to translate.
- one proof — every computation lowers to nox and is settled by zheng; eidos lifts the same machinery from execution-correctness to theorem-correctness.
- one focus — the graph settles into one φ\* ([[tru]]), which ranks every particle, drives reward, and is the fork-choice that [[foculus]] finalizes.

what completeness analysis added that the prior table omitted: nebu/genies/jali are named as the strata family (the field was load-bearing but invisible); [[inf]] (query) and [[prysm]] (paint) were missing language and present-layer repos; [[honeycrisp]] is the substrate under everything rather than a stage in the chain; the four-runtime ladder makes the execution model legible; and two real stack components were hiding inside cyb — [[cyb/root/ward|ward]] and cell.

## the gaps

coverage is complete in principle; these are the places where a layer exists in spec but not yet in working code, or where a seam is named but unbuilt.

1. capability enforcement is unbuilt. ward is a spec ([[cyb/root/ward]]); the only effect handler in the running system is cyb's `TerminalHost`, which performs `emit` ungated. until ward exists, every runtime touches the world without a permission boundary — the object-capability story is a design, not a control.
2. eidos settles nothing yet. its zheng bridge is a stub (`certificate.rs`: `stark_bytes` empty, zheng and bbg crates unlinked). the CIC kernel checks terms, but a checked theorem does not yet become a settled, memoized cyberlink. the theorem-proof half of the proof layer is not closed.
3. the witness ladder is partial. the proof-contract ladder is specified, but the mechanism that records a glia/wysm/wgpu host call as a witness noun in the calling rune program is not uniformly implemented across the three conditional runtimes. without it, conditional proofs cannot actually compose.
4. there is no single runtime contract. the four runtimes share a shape — `(subject, event) → (subject, effects)` plus a witness — but no common trait or spec names it. rune orchestrates them ad hoc. a unified runtime contract would let ward gate, and zheng witness, all four the same way.
5. cell and cyb-core overlap. both assemble signed `Signal`s (cell holds a local graph + chain; cyb-core's `SignalBus` queues them in-process). they should converge into one local-node abstraction rather than two signal builders.

## the deep overlaps

these are places where two components touch the same noun. each is either a clean complement to keep, or a seam to clarify.

| overlap | reading | verdict |
|---|---|---|
| two nox on-ramps — trident (.tri / .mir) and wysm-trident (.wasm) | both emit trident LIR, the single waist | complement — keep; guard that LIR stays the sole lowering target |
| two prove verbs — zheng (execution) / eidos (theorem) | different claims over the same kernel: ran-correctly vs is-well-typed | complement — orthogonal, both belong on the proof layer |
| two render verbs — mir (3D world from the graph) / prysm (UI from a particle dialect) | different input and output: spatial graph viz vs symbolic chunk stream | complement — distinct, name the border so neither grows into the other |
| three terminal surfaces — rune (provable eval → nox) / neu (neural command-as-sentence) / nu (host OS shell) | provable vs semantic vs host; neu may displace nu over time | clarify — three languages, one prompt; document which is which |
| the field stack — strata (algebra API) / nebu (portable 𝔽_p, NTT) / acpu (accelerated) | one API, one portable impl, one silicon impl | clean once nebu is named; acpu is the fast path nebu dispatches to |
| local-state assembly — cell / cyb-core / foculus (local) / cybergraph write-path | several places build and hold signals | converge cell + cyb-core; foculus owns availability, cybergraph the write-path |
| ward vs rune caps | rune carries `~caps`; ward owns the verbs, policy, and enforcement | clarify — rune transports capability, ward is the gate |

## closing the gaps

ordered by leverage, the moves that take the stack from complete-in-principle to complete-in-fact:

- define one runtime contract (gap 4) — the trait that nox / wysm / glia / wgpu all satisfy, with the witness shape explicit. this unblocks both ward (one place to gate) and the witness ladder (one place to record).
- build ward (gap 1) — promote cyb's `TerminalHost` into the runtime-blind effect router, so the conditional runtimes run behind a capability boundary.
- close the witness ladder (gap 3) — record every conditional host call as a witness noun, so conditional proofs compose.
- link eidos to zheng (gap 2) — fill the certificate stub so a checked theorem settles as a memoized cyberlink.
- converge cell and cyb-core (gap 5) — one local-node abstraction.
