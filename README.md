---
title: soft3
tags: cyber, soft3, sdk
crystal-type: spec
crystal-domain: cyber
icon: "👙"
alias: soft3 stack, cyb stack, software stack, proof pipeline
---
# soft3

developer experience layer for the [[soft3]] stack. makes the 20-repo stack accessible without understanding every repo.

a compass over one machinery. the [[cyb]] robot on top, the [[cyber]] collective consciousness at the bottom, [[cyberia]] the social layer on the left, the languages that program the troika on the right — all wrapped around the soft3 spine. every edge is labelled with the artifact that crosses it, the language each pair speaks.

```svgbob
   +----------------------------------------------------------------------------------+
   | cyb : the robot                                                                  |
   +----------------------------------------------------------------------------------+
   | soma : mind     pipeline : infer     worlds : face     honeycrisp : silicon      |
   +----------------------------------------------------------------------------------+
                                                           |
                                                           |  soma : intend . seal . link . subscribe . query
                                                           v
   +--------------------+               +------------------------------------+      +--------------+
   | cyberia : social   |               | cybergraph                         |      | fs           |
   +--------------------+               +------------------------------------+ patch+--------------+
   | . contract         |  cyberlinks   | :  link                            |<-----| :  mount     |
   | . service          |-------------->| the  spine                         |      +--------------+
   | mimi  midao  my    |  focus . karma|                                    |
   +--------------------+               |                                    |
    mudra -----keys ------------------->|                                    |
    plumb -----value ------------------>|                                    |
                                        +-------+-----------+-----------+----+
                                                |           |           |
                                                |store      |sync       |transmit
                                                v           v           v
                                            +-------+   +-------+   +-------+
                                            |  bbg  |   | sync  |   | tape  |
                                            +-------+   +-------+   +-------+
                                            |:store |   | :sync |   |:frame |
                                            +-------+   +-------+   +-------+
                                                |           |           |
                                                |verify     |agree      |frames
                                                |           |           |
                                                |           |           |
                                                |           v           v
                                                |       +-------+   +-------+
                                                |       |foculus|   | radio |
                                                |       +-------+   +-------+
                                                |       |:agree |   |:trans |
                                                |       +-------+   +-------+
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
   | strata : math   +   hemera : hash                  |           | nu . rs . tools    |
   +----------------------------------------------------+           +--------------------+
   conformance : snapshot  --  one hemera fingerprint per encoding & mechanism

   read side, recomputed every block :
   cybergraph --.graph--> tru --.model--> glia --features--> mir --> R-1.0
                           '----- phi* . positions . rank ----^

   +----------------------------------------------------------------------------------+
   | cyber : collective consciousness                                                 |
   +----------------------------------------------------------------------------------+
   | the whole graph converges to one mind                                            |
   | tru --> phi* --> foculus  ==>  cyberank . syntropy . CT-0 model                  |
   +----------------------------------------------------------------------------------+
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

[[neurons]] — humans, AIs, sensors, agents — link [[knowledge]] into the [[cybergraph]]. the [[tru]] reads this graph every block and computes what matters: [[cyberank]] per [[particle]], [[karma]] per [[neuron]], [[syntropy]] of the whole. every result is deterministic, on chain, verifiable by anyone. [[trident]] compiles any logic into [[zheng]] proofs — hash-based, post-quantum, no trusted setup. [[neural]] structures meaning through [[dialects]] so the graph speaks a [[language]] both humans and machines understand. [[cyb]] makes all of it accessible — a personal [[cyb/robot]] that queries, scripts, and navigates the graph

the [[tru]] is an onchain [[language]] model. it does what models do — rank, retrieve, infer — except the weights are public [[tokens]], the training data is an open [[cybergraph]], and the inference runs in [[consensus]] with proofs. no API keys, no corporate weights, no black boxes. the model improves when anyone links useful [[knowledge]], and the improvement is measurable as rising [[syntropy]]

[[trident]] closes the provability gap. in existing stacks, smart contracts can move [[tokens]] but cannot prove that a computation happened correctly without re-executing it. [[trident]] programs produce [[zheng]] proofs: verify once, trust forever. this makes the stack suitable for [[AI alignment]] — you can prove that a model followed a policy, not just trust that it did

## the core

twenty repos form the core. [[cybergraph]] is the vertebra — soma's one API funnel, fanning out to bbg (store), sync (sync), and radio (transmit). [[strata]] is the floor — every proof reduces to operations in its five algebras. [[soma]] is the ceiling — the avatar's mind that thinks over the whole stack. [[tape]] is the wire codec radio carries; [[sync]] owns the full structural-sync protocol below cybergraph. [[conformance]] is the meta-layer — one [[hemera]] fingerprint per canonical encoding and mechanism output, drift surfaces at commit time. the boundary is sharp: below it, Rust bootstrap required. above it, everything is pure [[trident]].

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
| + | [[plumb]] | pay | value layer: Coin + Card + five operations | — |
