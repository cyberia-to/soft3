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

one mind — shared, provable, self-improving. many languages — write, compute, mean — that compose without translation. an open world: planet scale, no schema, no gatekeeper. inside, identity, state, data and proof are one polynomial over one field — one object seen from different sides, so composition is free: there is nothing to translate between. you touch it three ways — you *write* in [[cybermark]], the system *computes* in the [[trident]] family, and it *means* in [[neural]].

the foundations behind these three lines are the [soft3 whitepaper](docs/). this repo is the developer experience layer on top — the SDKs, CLI, MCP server and wire schema that make a twenty-repo stack usable without learning all twenty repos.

## the core

soft3 is the [[cyber]] component of the [[cybics]] WORK triad — and internally it unfolds the same seven-triad spiral that [[cybics]] uses for all knowledge. form defines the primitive rules. mass is what knowledge is literally made of. space is where it exists at scale. life is where it becomes intelligent. word is what it means. work is how it runs. play is where we act together. each group earns its place: remove any one and a class of reasoning becomes impossible.

### [[cybics/form|form]]

| repo | verb | release | what it does |
|---|---|---|---|
| [[honeycrisp]] | accelerate | — | NEON/AMX/SME · Metal GPU · ANE · zero-copy unimem |
| [[strata]] | math | — | nebu 𝔽_p · genies 𝔽_q · jali R_q |
| [[hemera]] | hash | v0.2.0 | [[Poseidon2]] sponge · identity · trees · verified streaming |
| [[lens]] | commit | — | 5 PCS backends, one per algebra |

### [[cybics/mass|mass]]

| repo | verb | release | what it does |
|---|---|---|---|
| [[soft3/cybergraph\|cybergraph]] | link | — | jets · memos · types · knowledge |
| [[soft3/bbg\|bbg]] | store | — | 1 polynomial · 10 dims · ~200B proofs |
| [[cell]] | hold | — | local node · graph slice · signal chain · apply/prove loop |
| [[fs]] | mount | — | particles · patches · sync |
| [[tok]] | pay | — | Coin · Card · conservation |

### [[cybics/space|space]]

| repo | verb | release | what it does |
|---|---|---|---|
| [[mudra]] | encrypt | — | KEM · dCTIDH · AEAD · TFHE · threshold |
| [[radio]] | transmit | — | QUIC · BAO streaming · gossip |
| [[tape]] | frame | — | typed particle framing over any byte stream |
| [[foculus]] | sync | — | chain · VDF · equivocation · DAS · erasure · CRDT |
| [[cyb/root/ward\|ward]] | authorize | spec | effect router · emit/query/link/seal/host |

### [[cybics/life|life]]

| repo | verb | release | what it does |
|---|---|---|---|
| [[tru]] | converge | — | φ* · eigenvectors · cyberank |
| [[foculus]] | agree | — | [[collective focus theorem]] → finality |
| [[soft3/soma\|soma]] | think | — | four concurrent loops · tiered model stack |

### [[cybics/word|word]]

| repo | verb | release | what it does |
|---|---|---|---|
| [[neural]] | mean | — | sigil → word → link → sentence → motif → dialect |
| [[rune]] | eval | — | Rs · hint · host · eval · async |
| [[inf]] | query | — | datalog · fixed-point over BBG |

### [[cybics/work|work]]

| repo | verb | release | what it does |
|---|---|---|---|
| [[trident]] | compile | v0.1.0 | .tri → .nox |
| [[soft3/nox\|nox]] | run | — | 18 patterns + jets · unconditional proof |
| [[wysm]] | sandbox | — | WASM souls · gas-metered · cond/host |
| [[glia]] | infer | — | .model runtime · cond/model |
| [[kern]] | shade | — | Metal/Vulkan/WebGPU shaders · cond/host |
| [[zheng]] | prove | — | [[SuperSpartan]] · Brakedown · [[sumcheck]] |
| [[eidos]] | certify | — | CIC type-checker · theorem → cyberlink |

### [[cybics/play|play]]

| repo | verb | release | what it does |
|---|---|---|---|
| [[mir]] | render | — | positions + features → [[R-1.0]] world |
| [[prysm]] | paint | — | tape dialect · chunks → UI |
| [[cyb]] | embody | — | Bevy worlds · terminal · live cells |

### stability

| repo | verb | release | what it does |
|---|---|---|---|
| [[conformance]] | snapshot | scaffold | hemera fingerprint per encoding · stability harness |

the four runtimes in work — [[soft3/nox|nox]] · [[glia]] · [[wysm]] · [[kern]] — form a proof-contract ladder from unconditional down to conditional-on-host; the long-term arc lowers every soul toward nox through trident LIR. [[kern]] wraps wgpu into an authored component; [[nu]] (vendored Nushell) is the external piece outside the authored-verb table. the boundary analysis for the network tier lives in [[component-boundaries]].

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
