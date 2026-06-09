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

soft3 is **one mind** where other stacks sprawl, **many languages** where they lock you in, and an **open world** where they build walls. inside, identity, state, data and proof are one polynomial over one field — one object seen from different sides, so composition is free: there is nothing to translate between. you touch it three ways — you *write* in [[cybermark]], the system *computes* in the [[trident]] family, and it *means* in [[neural]]. and it stays open: it holds at planet scale, composes without glue, and admits anything that lowers to [[nox]].

the foundations behind these three lines are the [soft3 whitepaper](docs/). this repo is the developer experience layer on top — the SDKs, CLI, MCP server and wire schema that make a twenty-repo stack usable without learning all twenty repos.

## the core

twenty repos form the stack. [[cybergraph]] is the vertebra — soma's one API funnel, fanning out to bbg (store), sync (sync), and radio (transmit). [[strata]] is the floor — every proof reduces to operations in its five algebras. [[soma]] is the ceiling — the avatar's mind that thinks over the whole stack. the boundary is sharp: below it, Rust bootstrap required; above it, everything is pure [[trident]].

| # | repo | verb | what it does | release |
|---|------|------|-------------|---------|
| 0 | [[strata]] | math | 4 tiers × 5 algebras | — |
| 1 | [[hemera]] | hash | [[Poseidon2]] sponge. particle identity | v0.2.0 |
| 2 | [[lens]] | commit | 5 PCS backends, one per algebra | — |
| 3 | [[trident]] | compile | .tri → .nox | v0.1.0 |
| 4 | [[nox]] | run | 18 patterns (16 compute + call + look) + jets | — |
| 5 | [[zheng]] | prove & verify | [[SuperSpartan]] + Brakedown + [[sumcheck]] | — |
| 6 | [[cybergraph]] | link | jets, memos, types, knowledge | — |
| 7 | [[bbg]] | store | 1 polynomial, 10 dims. ~200B proofs | — |
| 8 | [[tru]] | converge | .graph → .model. φ*, eigenvectors, cyberank | — |
| 9 | [[glia]] | infer | universal .model runtime | — |
| 10 | [[mir]] | render | positions + features → [[R-1.0]] world | — |
| 11 | [[mudra]] | encrypt | KEM, dCTIDH, AEAD, TFHE, threshold | — |
| 12 | [[radio]] | transmit | QUIC + BAO streaming + gossip | — |
| 13 | [[tape]] | frame | typed atomic particle framing over any byte stream | — |
| 14 | [[sync]] | sync | structural sync: chain, VDF, equivocation, DAS, erasure, CRDT | — |
| 15 | [[foculus]] | agree | [[collective focus theorem]] → finality | — |
| 16 | [[soma]] | think | avatar cognitive architecture. four concurrent loops over a tiered model stack | — |
| 17 | [[conformance]] | snapshot | hemera fingerprint per encoding and mechanism. stability harness across the stack | scaffold |
| + | [[rune]] | eval | Rs + hint + host + eval. dynamic async layer | — |
| + | [[fs]] | mount | sovereign filesystem: particles, patches, sync | — |
| + | [[tok]] | pay | the token language + value layer: Coin + Card + conservation | — |

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
