---
title: stack
tags: cyber, soft3
crystal-type: spec
crystal-domain: cyber
alias: soft3 registry, component registry, stack registry
---
# the soft3 stack

the component registry: every repo, its verb, its release, its role. soft3 is the [[cyber]] component of the [[cybics]] WORK triad — and internally it unfolds the same seven-triad spiral that [[cybics]] uses for all knowledge. form defines the primitive rules. mass is what knowledge is literally made of. space is where it exists at scale. life is where it becomes intelligent. word is what it means. work is how it runs. play is where we act together. each group earns its place: remove any one and a class of reasoning becomes impossible.

## [[cybics/form|form]]

| repo | verb | release | crates | what it does |
|---|---|---|---|---|
| [[honeycrisp]] | accelerate | [v0.2.0](https://github.com/cyberia-to/honeycrisp/releases/tag/v0.2.0) | [honeycrisp 0.2.0](https://crates.io/crates/honeycrisp) | NEON/AMX/SME · Metal GPU · ANE · zero-copy unimem |
| [[strata]] | math | [v0.1.1](https://github.com/cyberia-to/strata/releases/tag/v0.1.1) | [cyber-strata 0.1.0](https://crates.io/crates/cyber-strata) | nebu 𝔽_p · genies 𝔽_q · jali R_q |
| [[hemera]] | hash | [v0.3.0](https://github.com/cyberia-to/hemera/releases/tag/v0.3.0) | [cyber-hemera 0.3.0](https://crates.io/crates/cyber-hemera) | [[Poseidon2]] sponge · identity · trees · verified streaming |
| [[lens]] | commit | [v0.1.2](https://github.com/cyberia-to/lens/releases/tag/v0.1.2) | [cyber-lens 0.1.1](https://crates.io/crates/cyber-lens) | 5 PCS backends, one per algebra |

## [[cybics/mass|mass]]

| repo | verb | release | crates | what it does |
|---|---|---|---|---|
| [[soft3/cybergraph\|cybergraph]] | link | [v0.1.1](https://github.com/cyberia-to/cybergraph/releases/tag/v0.1.1) | [cybergraph 0.1.1](https://crates.io/crates/cybergraph) | jets · memos · types · knowledge |
| [[soft3/bbg\|bbg]] | store | [v0.1.1](https://github.com/cyberia-to/bbg/releases/tag/v0.1.1) | [bbg 0.1.1](https://crates.io/crates/bbg) | 1 polynomial · 10 dims · ~200B proofs |
| [[cell]] | hold | — | — | local node · graph slice · signal chain · apply/prove loop |
| [[fs]] | mount | — | — | particles · patches · sync |
| [[tok]] | pay | [v0.1.1](https://github.com/cyberia-to/plumb/releases/tag/v0.1.1) | [cyber-tok 0.1.1](https://crates.io/crates/cyber-tok) | Coin · Card · conservation |

## [[cybics/space|space]]

| repo | verb | release | crates | what it does |
|---|---|---|---|---|
| [[mudra]] | encrypt | — | — | KEM · dCTIDH · AEAD · TFHE · threshold |
| [[radio]] | transmit | — | [cyber-radio 0.1.0](https://crates.io/crates/cyber-radio) | QUIC · BAO streaming · gossip |
| [[tape]] | frame | — | [cyber-tape 0.1.0](https://crates.io/crates/cyber-tape) | typed particle framing over any byte stream |
| [[foculus]] | sync | [v0.1.1](https://github.com/cyberia-to/foculus/releases/tag/v0.1.1) | [foculus 0.1.1](https://crates.io/crates/foculus) | chain · VDF · equivocation · DAS · erasure · CRDT |
| [[cyb/root/ward\|ward]] | authorize | spec | — | effect router · emit/query/link/seal/host |

## [[cybics/life|life]]

| repo | verb | release | crates | what it does |
|---|---|---|---|---|
| [[tru]] | converge | [v0.1.1](https://github.com/cyberia-to/tru/releases/tag/v0.1.1) | [cyber-tru 0.1.1](https://crates.io/crates/cyber-tru) | φ* · eigenvectors · cyberank |
| [[foculus]] | agree | [v0.1.1](https://github.com/cyberia-to/foculus/releases/tag/v0.1.1) | [foculus 0.1.1](https://crates.io/crates/foculus) | [[collective focus theorem]] → finality |
| [[soft3/soma\|soma]] | think | — | — | four concurrent loops · tiered model stack |

## [[cybics/word|word]]

| repo | verb | release | crates | what it does |
|---|---|---|---|---|
| [[neural]] | mean | — | — | sigil → word → link → sentence → motif → dialect |
| [[rune]] | eval | — | — | Rs · hint · host · eval · async |
| [[inf]] | query | — | — | datalog · fixed-point over BBG |

## [[cybics/work|work]]

| repo | verb | release | crates | what it does |
|---|---|---|---|---|
| [[trident]] | compile | [v0.1.0](https://github.com/cyberia-to/trident/releases/tag/v0.1.0) | — | .tri → .nox |
| [[soft3/nox\|nox]] | run | [v0.1.1](https://github.com/cyberia-to/nox/releases/tag/v0.1.1) | [cyber-nox 0.1.1](https://crates.io/crates/cyber-nox) | 18 patterns + jets · unconditional proof |
| [[wysm]] | sandbox | — | — | WASM souls · gas-metered · cond/host |
| [[glia]] | infer | — | — | .model runtime · cond/model |
| [[kern]] | shade | — | — | Metal/Vulkan/WebGPU shaders · cond/host |
| [[zheng]] | prove | [v0.1.1](https://github.com/cyberia-to/zheng/releases/tag/v0.1.1) | [zheng 0.1.1](https://crates.io/crates/zheng) | [[SuperSpartan]] · Brakedown · [[sumcheck]] |
| [[eidos]] | certify | — | — | CIC type-checker · theorem → cyberlink |

## [[cybics/play|play]]

| repo | verb | release | crates | what it does |
|---|---|---|---|---|
| [[mir]] | render | — | — | positions + features → [[R-1.0]] world |
| [[prysm]] | paint | — | — | tape dialect · chunks → UI |
| [[cyb]] | embody | [v0.2.1-crates](https://github.com/cyberia-to/cyb/releases/tag/v0.2.1-crates) | [cyb 0.2.1](https://crates.io/crates/cyb) | Bevy worlds · terminal · live cells |
| [[lytics]] | measure | — | — | signed visitor events · PoW-priced · retention · cohorts · funnels |

## stability

| repo | verb | release | crates | what it does |
|---|---|---|---|---|
| [[conformance]] | snapshot | scaffold | — | hemera fingerprint per encoding · stability harness |

the four runtimes in work — [[soft3/nox|nox]] · [[glia]] · [[wysm]] · [[kern]] — form a proof-contract ladder from unconditional down to conditional-on-host; the long-term arc lowers every soul toward nox through trident LIR. [[kern]] wraps wgpu into an authored component; [[nu]] (vendored Nushell) is the external piece outside the authored-verb table. the boundary analysis for the network tier lives in [[component-boundaries]].

## the SDK

| dir | what | status |
|-----|------|--------|
| [js/](js/) | JavaScript/TypeScript SDK (current Bostrom chain) | active |
| [schema/](schema/) | canonical wire format definitions | draft |
| [cli/](cli/) | `soft3` stack CLI + node · [soft3 0.8.0](https://crates.io/crates/soft3) · product face [true-cyber 0.7.0](https://crates.io/crates/true-cyber) | published |
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

see [[soft3/docs|the whitepaper]] for the foundations — the methods behind one mind, many languages, open world.
