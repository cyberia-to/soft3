---
title: soft3
tags: cyber, soft3, sdk
crystal-type: spec
crystal-domain: cyber
icon: "👙"
alias: soft3 stack, cyb stack, software stack, proof pipeline
---
# soft3

> one mind, many languages, open world

the stack for a shared, provable, self-improving [[knowledge]] system. every web had one — web1 ran on LAMP, web2 on React + Node + Postgres, web3 on Solidity + EVM + RPC. soft3 is the next: every computation leaves a [[cryptographic proof]], every piece of meaning carries a measurable weight, and the whole graph converges to one mind.

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
   it admits    —  no schema, no gatekeeper, no closed set
```

soft3 is **one mind** where other stacks sprawl, **many languages** where they lock you in, and an **open world** where they build walls. inside, identity, state, data and proof are one polynomial over one field — one object seen from different sides, so composition is free: there is nothing to translate between. you touch it three ways — you *write* in [[cybermark]], the system *computes* in the [[trident]] family, and it *means* in [[neural]]. and it stays open: it holds at planet scale, composes without glue, and admits anything that lowers to [[soft3/nox|nox]].

the foundations behind these three lines are the [soft3 whitepaper](docs/). this repo is the developer experience layer on top — the SDKs, CLI, MCP server and wire schema that make a twenty-repo stack usable without learning all twenty repos.

## the core

the stack is layered, not flat. [[honeycrisp]] is the silicon floor and [[strata]] the math floor; every layer above reduces toward one field, one proof, one focus. [[soft3/cybergraph|cybergraph]] is the vertebra — soma's one API funnel, fanning out to bbg (store), sync (sync), and radio (transmit). [[soft3/soma|soma]] is the ceiling — the avatar's mind that thinks over the whole stack, and [[cyb]] is the body it wears. the boundary is sharp: below [[trident]], Rust bootstrap required; above it, everything lowers to [[soft3/nox|nox]].

| layer | repo | verb | what it does | release |
|---|------|------|-------------|---------|
| substrate | [[honeycrisp]] | accelerate | Apple Silicon drivers — NEON/AMX/SME · Metal GPU · ANE over zero-copy unimem | — |
| math | [[strata]] | math | 4 tiers × 5 algebras — the family includes nebu (𝔽_p), genies (𝔽_q), jali (R_q) | — |
| commit | [[hemera]] | hash | [[Poseidon2]] sponge. particle identity, trees, verified streaming | v0.2.0 |
| commit | [[lens]] | commit | 5 PCS backends, one per algebra | — |
| language | [[trident]] | compile | .tri → .nox | v0.1.0 |
| language | [[neural]] | mean | the semantic language — sigil → word → link → sentence → motif → dialect | — |
| language | [[rune]] | eval | Rs + hint + host + eval. dynamic async layer | — |
| language | [[inf]] | query | datalog query engine — the language of sets; fixed-point over BBG | — |
| runtime | [[soft3/nox\|nox]] | run | 18 patterns (16 compute + call + look) + jets — unconditional proof | — |
| runtime | [[wysm]] | sandbox | conventional WASM souls, gas-metered, jet-accelerated — conditional on host | — |
| runtime | [[glia]] | infer | universal .model runtime — conditional on model | — |
| runtime | wgpu | shade | GPU compute shaders (Metal/Vulkan/WebGPU) — conditional on host | ext |
| proof | [[zheng]] | prove | [[SuperSpartan]] + Brakedown + [[sumcheck]] — proves execution correct | — |
| proof | [[eidos]] | certify | CIC type-checker as a nox program — proves theorems correct; theorem → cyberlink | — |
| graph | [[soft3/cybergraph\|cybergraph]] | link | jets, memos, types, knowledge | — |
| graph | [[soft3/bbg\|bbg]] | store | 1 polynomial, 10 dims. ~200B proofs | — |
| graph | cell ([[cyb]]) | hold | the local node — a cybergraph slice + signal chain + apply/prove loop | — |
| graph | [[fs]] | mount | sovereign filesystem: particles, patches, sync | — |
| dynamics | [[tru]] | converge | .graph → .model. φ*, eigenvectors, cyberank | — |
| dynamics | [[foculus]] | agree | [[collective focus theorem]] → finality | — |
| network | [[mudra]] | encrypt | KEM, dCTIDH, AEAD, TFHE, threshold | — |
| network | [[radio]] | transmit | QUIC + BAO streaming + gossip | — |
| network | [[tape]] | frame | typed atomic particle framing over any byte stream | — |
| network | [[sync]] | sync | structural sync: chain, VDF, equivocation, DAS, erasure, CRDT | — |
| present | [[mir]] | render | positions + features → [[R-1.0]] world | — |
| present | [[prysm]] | paint | the reference tape dialect + renderer — particle chunks → UI | — |
| capability | [[cyb/root/ward\|ward]] | authorize | runtime-blind effect router — gates emit/query/link/seal/host | spec |
| cognition | [[soft3/soma\|soma]] | think | avatar cognitive architecture. four concurrent loops over a tiered model stack | — |
| value | [[tok]] | pay | the token language + value layer: Coin + Card + conservation | — |
| stability | [[conformance]] | snapshot | hemera fingerprint per encoding and mechanism. stability harness across the stack | scaffold |
| app | [[cyb]] | embody | the avatar app — Bevy worlds, terminal, live cell runtime | — |

the four runtimes — [[soft3/nox|nox]] · [[glia]] · [[wysm]] · wgpu — form a proof-contract ladder from unconditional down to conditional-on-host; the long-term arc lowers every soul toward nox through trident LIR. host and external pieces sit outside the authored-verb table: [[nu]] is vendored Nushell — the structured-data shell embedded in the cyb terminal — and wgpu is the external GPU-shader toolchain. the boundary analysis for the network tier lives in [[component-boundaries]].

## the SDK

| dir | what | status |
|-----|------|--------|
| [js/](js/) | JavaScript/TypeScript SDK (current Bostrom chain) | active |
| [schema/](schema/) | canonical wire format definitions | draft |
| [cli/](cli/) | `soft3` command-line tool | scaffold |
| [mcp/](mcp/) | MCP server — cybergraph tools for AI assistants | scaffold |
| [py/](py/) | Python SDK | scaffold |

every SDK exposes the same five operations regardless of language:

```text
particle(content)              → particle     hemera hash of bytes
cyberlink(from, to, neuron)    → signal       construct + sign a cyberlink
query(particle, dimension)     → value+proof  BBG Lens opening
verify(root, proof)            → bool         proof verification
submit(signal)                 → receipt      send signal to network
```

```ts
import { CyberClient } from '@cybercongress/cyber-js'

const client = await CyberClient.connect('https://rpc.bostrom.cybernode.ai')
const result = await client.rank.search('cyber')
```

dependencies not yet stabilised — full implementation is blocked on wire format finalisation (`schema/`), BBG proof serialisation (lens `Commitment`/`Opening` serde), and the query RPC protocol. the scaffold is in place; implementations land per component as deps stabilise.

see [soft3/docs](docs/) for the foundations whitepaper — the methods behind one mind, many languages, open world.
