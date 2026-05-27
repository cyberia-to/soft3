---
title: soft3
tags: cyber, soft3, sdk
crystal-type: spec
crystal-domain: cyber
icon: "👙"
alias: soft3 stack, cyb stack, software stack, proof pipeline
---
# soft3

developer experience layer for the [[soft3]] stack. makes the 14-repo stack accessible without understanding every repo.

```
neurons / apps
      ↓
 soft3 SDK          ← this repo
      ↓
cybergraph ─── bbg ─── tru ─── glia ─── mir
      ↓           ↓
   radio         lens / strata
```

## what it provides

| component | purpose | status |
|-----------|---------|--------|
| [js/](js/) | JavaScript/TypeScript SDK (current Bostrom chain) | active |
| [schema/](schema/) | canonical wire format definitions | draft |
| [cli/](cli/) | `soft3` command-line tool | scaffold |
| [mcp/](mcp/) | MCP server — cybergraph tools for AI assistants | scaffold |
| [py/](py/) | Python SDK | scaffold |

## core operations

every SDK exposes the same five operations regardless of language:

```
particle(content)              → particle     hemera hash of bytes
cyberlink(from, to, neuron)    → signal       construct + sign a cyberlink
query(particle, dimension)     → value+proof  BBG Lens opening
verify(root, proof)            → bool         proof verification
submit(signal)                 → receipt      send signal to network
```

## quick start (JS)

```ts
import { CyberClient } from '@cybercongress/cyber-js'

const client = await CyberClient.connect('https://rpc.bostrom.cybernode.ai')
const result = await client.rank.search('cyber')
```

## status

dependencies not yet stabilised — full implementation blocked on:
- wire format finalisation (`schema/`)
- BBG proof serialisation (lens `Commitment`/`Opening` serde)
- query RPC protocol definition

scaffold is in place. implementations land per component as deps stabilise.

see [[soft3]] for the full stack description.

---

every generation of the web had its stack. web1 had LAMP. web2 had React + Node + Postgres. web3 had Solidity + EVM + RPC. each defined what developers could build and what users could experience

soft3 is the stack for a shared, provable, self-improving [[knowledge]] system where every computation leaves a [[cryptographic proof]] and every piece of meaning has a measurable weight

[[neurons]] — humans, AIs, sensors, agents — link [[knowledge]] into the [[cybergraph]]. the [[tru]] reads this graph every block and computes what matters: [[cyberank]] per [[particle]], [[karma]] per [[neuron]], [[syntropy]] of the whole. every result is deterministic, on chain, verifiable by anyone. [[trident]] compiles any logic into [[zheng]] proofs — hash-based, post-quantum, no trusted setup. [[neural]] structures meaning through [[semantic conventions]] so the graph speaks a [[language]] both humans and machines understand. [[cyb]] makes all of it accessible — a personal [[cyb/robot]] that queries, scripts, and navigates the graph

the [[tru]] is an onchain [[language]] model. it does what models do — rank, retrieve, infer — except the weights are public [[tokens]], the training data is an open [[cybergraph]], and the inference runs in [[consensus]] with proofs. no API keys, no corporate weights, no black boxes. the model improves when anyone links useful [[knowledge]], and the improvement is measurable as rising [[syntropy]]

[[trident]] closes the provability gap. in existing stacks, smart contracts can move [[tokens]] but cannot prove that a computation happened correctly without re-executing it. [[trident]] programs produce [[zheng]] proofs: verify once, trust forever. this makes the stack suitable for [[AI alignment]] — you can prove that a model followed a policy, not just trust that it did

## the core

fourteen repos form the core. [[cybergraph]] is the vertebra — everything attaches to it. [[strata]] is the floor — every proof reduces to operations in its five algebras. the boundary is sharp: below it, Rust bootstrap required. above it, everything is pure [[trident]].

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
| 13 | [[foculus]] | agree | [[collective focus theorem]] → finality | — |
| + | [[rune]] | eval | Rs + hint + host + eval. dynamic async layer | — |
