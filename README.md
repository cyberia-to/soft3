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

soft3 is the [[cyber]] component of the [[cybics]] WORK triad — and internally it unfolds the same seven-triad spiral that [[cybics]] uses for all knowledge. form defines the primitive rules. mass is what knowledge is literally made of. space is where it exists at scale. life is where it becomes intelligent. word is what it means. work is how it runs. play is where we act together. each group earns its place: remove any one and a class of reasoning becomes impossible.

### [[form]]

| repo | verb | what it does | release |
|---|---|---|---|
| [[honeycrisp]] | accelerate | Apple Silicon drivers — NEON/AMX/SME · Metal GPU · ANE over zero-copy unimem | — |
| [[strata]] | math | 4 tiers × 5 algebras — the family includes nebu (𝔽_p), genies (𝔽_q), jali (R_q) | — |
| [[hemera]] | hash | [[Poseidon2]] sponge. particle identity, trees, verified streaming | v0.2.0 |
| [[lens]] | commit | 5 PCS backends, one per algebra | — |

### [[mass]]

| repo | verb | what it does | release |
|---|---|---|---|
| [[soft3/cybergraph\|cybergraph]] | link | jets, memos, types, knowledge | — |
| [[soft3/bbg\|bbg]] | store | 1 polynomial, 10 dims. ~200B proofs | — |
| cell ([[cyb]]) | hold | the local node — a cybergraph slice + signal chain + apply/prove loop | — |
| [[fs]] | mount | sovereign filesystem: particles, patches, sync | — |

### [[space]]

| repo | verb | what it does | release |
|---|---|---|---|
| [[mudra]] | encrypt | KEM, dCTIDH, AEAD, TFHE, threshold | — |
| [[radio]] | transmit | QUIC + BAO streaming + gossip | — |
| [[tape]] | frame | typed atomic particle framing over any byte stream | — |
| [[sync]] | sync | structural sync: chain, VDF, equivocation, DAS, erasure, CRDT | — |

### [[life]]

| repo | verb | what it does | release |
|---|---|---|---|
| [[tru]] | converge | .graph → .model. φ*, eigenvectors, cyberank | — |
| [[foculus]] | agree | [[collective focus theorem]] → finality | — |
| [[soft3/soma\|soma]] | think | avatar cognitive architecture. four concurrent loops over a tiered model stack | — |

### [[word]]

| repo | verb | what it does | release |
|---|---|---|---|
| [[neural]] | mean | the semantic language — sigil → word → link → sentence → motif → dialect | — |
| [[rune]] | eval | Rs + hint + host + eval. dynamic async layer | — |
| [[inf]] | query | datalog query engine — the language of sets; fixed-point over BBG | — |

### [[work]]

| repo | verb | what it does | release |
|---|---|---|---|
| [[trident]] | compile | .tri → .nox | v0.1.0 |
| [[soft3/nox\|nox]] | run | 18 patterns (16 compute + call + look) + jets — unconditional proof | — |
| [[wysm]] | sandbox | conventional WASM souls, gas-metered — conditional on host | — |
| [[glia]] | infer | universal .model runtime — conditional on model | — |
| wgpu | shade | GPU compute shaders (Metal/Vulkan/WebGPU) — conditional on host | ext |
| [[zheng]] | prove | [[SuperSpartan]] + Brakedown + [[sumcheck]] — proves execution correct | — |
| [[eidos]] | certify | CIC type-checker as a nox program — proves theorems correct; theorem → cyberlink | — |

### [[play]]

| repo | verb | what it does | release |
|---|---|---|---|
| [[mir]] | render | positions + features → [[R-1.0]] world | — |
| [[prysm]] | paint | the reference tape dialect + renderer — particle chunks → UI | — |
| [[tok]] | pay | the token language + value layer: Coin + Card + conservation | — |
| [[cyb/root/ward\|ward]] | authorize | runtime-blind effect router — gates emit/query/link/seal/host | spec |
| [[cyb]] | embody | the avatar app — Bevy worlds, terminal, live cell runtime | — |

### stability

| repo | verb | what it does | release |
|---|---|---|---|
| [[conformance]] | snapshot | hemera fingerprint per encoding and mechanism. stability harness across the stack | scaffold |

the four runtimes in work — [[soft3/nox|nox]] · [[glia]] · [[wysm]] · wgpu — form a proof-contract ladder from unconditional down to conditional-on-host; the long-term arc lowers every soul toward nox through trident LIR. [[nu]] (vendored Nushell) and wgpu are external pieces outside the authored-verb table. the boundary analysis for the network tier lives in [[component-boundaries]].

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
