---
title: status
tags: cyber, soft3
crystal-type: spec
crystal-domain: cyber
alias: stack, soft3 registry, component registry, stack registry, soft3 status
---
# soft3 status

the component registry and its honest state: every repo, its verb, its release, and where it actually stands. soft3 is the [[cyber]] component of the [[cybics]] WORK triad — and internally it unfolds the same seven-triad spiral that [[cybics]] uses for all knowledge. form defines the primitive rules. mass is what knowledge is literally made of. space is where it exists at scale. life is where it becomes intelligent. word is what it means. work is how it runs. play is where we act together.

each component is in one of six states. **live** — carries real traffic today. **published** — released and correct, but not yet in the live loop. **unwired** — built, released, and not connected to the thing that needs it. **spec** — paper only. **dormant** — no commits in months while holding a named seat. **blocked** — waiting on another component's gap.

| repo | verb | state | release | crates | what it does |
|---|---|---|---|---|---|
| [[soft3]] | compose | live | [0.10.0](https://github.com/cyberia-to/soft3/releases/tag/v0.10.0) | [0.10.0](https://crates.io/crates/soft3) | the stack CLI · `soft3 node` · a release is a [[soft3/conformance|conformance snapshot]] |
| [[cybics/form|form]] |  |  |  |  |  |
| [[honeycrisp]] | accelerate | live | [0.2.0](https://github.com/cyberia-to/honeycrisp/releases/tag/v0.2.0) | [0.2.0](https://crates.io/crates/honeycrisp) | NEON/AMX/SME · Metal GPU · ANE · zero-copy unimem |
| [[strata]] | math | live | [0.1.1](https://github.com/cyberia-to/strata/releases/tag/v0.1.1) | [0.1.1](https://crates.io/crates/cyber-strata) | nebu 𝔽_p · genies 𝔽_q · jali R_q |
| [[hemera]] | hash | live | [0.3.0](https://github.com/cyberia-to/hemera/releases/tag/v0.3.0) | [0.3.0](https://crates.io/crates/cyber-hemera) | [[Poseidon2]] sponge · identity · trees · verified streaming |
| [[lens]] | commit | published | [0.1.2](https://github.com/cyberia-to/lens/releases/tag/v0.1.2) | [0.1.2](https://crates.io/crates/cyber-lens) | 5 PCS backends, one per algebra |
| [[cybics/mass|mass]] |  |  |  |  |  |
| [[soft3/cybergraph\|cybergraph]] | link | live | [0.1.1](https://github.com/cyberia-to/cybergraph/releases/tag/v0.1.1) | [0.1.1](https://crates.io/crates/cybergraph) | jets · memos · types · knowledge |
| [[soft3/bbg\|bbg]] | store | live | [0.1.2](https://github.com/cyberia-to/bbg/releases/tag/v0.1.2) | [0.1.2](https://crates.io/crates/bbg) | 1 polynomial · 10 dims · ~200B proofs |
| [[cell]] | hold | spec | — | — | local node · graph slice · signal chain · apply/prove loop · overlaps cyb-core, to converge |
| [[fs]] | mount | spec | — | — | particles · patches · sync · repo is empty |
| [[tok]] | pay | published | [0.1.1](https://github.com/cyberia-to/plumb/releases/tag/v0.1.1) | [0.1.1](https://crates.io/crates/cyber-tok) | Coin · Card · conservation · no live economy yet |
| [[cybics/space|space]] |  |  |  |  |  |
| [[vault]] | keep | spec | — | — | secret storage: seeds, caps, sealed state — a seat proposed, not yet a repo |
| [[mudra]] | encrypt | live | [0.1.0](https://github.com/cyberia-to/mudra/releases/tag/v0.1.0) | [0.1.0](https://crates.io/crates/cyber-mudra) | KEM · dCTIDH · AEAD · TFHE · threshold |
| [[radio]] | transmit | live | [0.1.0](https://github.com/cyberia-to/radio/releases/tag/v0.1.0) | [0.1.0](https://crates.io/crates/cyber-radio) | QUIC · BAO streaming · gossip · carries `cyb/sync/0` |
| [[tape]] | frame | published | [0.1.0](https://github.com/cyberia-to/tape/releases/tag/v0.1.0) | [0.1.0](https://crates.io/crates/cyber-tape) | typed particle framing over any byte stream |
| [[foculus]] | sync | unwired | [0.1.2](https://github.com/cyberia-to/foculus/releases/tag/v0.1.2) | [0.1.2](https://crates.io/crates/foculus) | chain · VDF · equivocation · DAS · erasure · CRDT · engine partial, never deployed |
| [[cyb/root/ward\|ward]] | authorize | spec | — | — | effect router · emit/query/link/seal/host |
| [[cybics/life|life]] |  |  |  |  |  |
| [[tru]] | converge | published | [0.1.1](https://github.com/cyberia-to/tru/releases/tag/v0.1.1) | [0.1.1](https://crates.io/crates/cyber-tru) | φ* · eigenvectors · cyberank |
| [[foculus]] | agree | spec | [0.1.2](https://github.com/cyberia-to/foculus/releases/tag/v0.1.2) | [0.1.2](https://crates.io/crates/foculus) | [[collective focus theorem]] → finality · fork-choice unproven in the wild |
| [[sigma]] | evaluate | spec | — | — | assessment: scores and gradings become links — a seat proposed, not yet a repo |
| [[soft3/soma\|soma]] | think | live | — | — | four concurrent loops · tiered model stack · phase 1 in [[cyb]] |
| [[cybics/word|word]] |  |  |  |  |  |
| [[neural]] | mean | spec | — | — | sigil → word → link → sentence → motif → dialect |
| [[trident]] | compile | published | [0.1.0](https://github.com/cyberia-to/trident/releases/tag/v0.1.0) | [0.1.0](https://crates.io/crates/trident-lang) | .tri → .nox |
| [[rune]] | eval | published | [0.1.0](https://github.com/cyberia-to/rune/releases/tag/v0.1.0) | [0.1.0](https://crates.io/crates/cyber-rune) | Rs · hint · host · eval · async |
| [[inf]] | query | live | [0.1.0](https://github.com/cyberia-to/inf/releases/tag/v0.1.0) | [0.1.0](https://crates.io/crates/inf-cli) | datalog · fixed-point over BBG |
| [[cybics/work|work]] |  |  |  |  |  |
| [[soft3/nox\|nox]] | run | published | [0.1.2](https://github.com/cyberia-to/nox/releases/tag/v0.1.2) | [0.1.2](https://crates.io/crates/cyber-nox) | 18 patterns + jets · unconditional proof |
| [[zheng]] | prove | blocked | [0.1.2](https://github.com/cyberia-to/zheng/releases/tag/v0.1.2) | [0.1.2](https://crates.io/crates/zheng) | [[SuperSpartan]] · Brakedown · [[sumcheck]] · axis/hash/look opening derivation open |
| [[glia]] | infer | live | — | — | .model runtime · cond/model |
| [[eidos]] | certify | blocked | — | — | CIC type-checker · theorem → cyberlink · zheng bridge is a stub |
| [[wysm]] | sandbox | dormant | — | — | WASM souls · gas-metered · cond/host · no commits since june |
| [[kern]] | shade | spec | — | — | Metal/Vulkan/WebGPU shaders · cond/host |
| [[cybics/play|play]] |  |  |  |  |  |
| [[mir]] | render | live | — | — | positions + features → [[R-1.0]] world |
| [[prysm]] | paint | live | — | — | tape dialect · chunks → UI |
| [[lytics]] | measure | live | — | — | signed visitor events · PoW-priced · cohorts · funnels — cybernetics is impossible without measurement |

**the stack has a heart, and products grow on its skin.** at the centre sits the trinity the whole stack exists to turn: [[cyb]] the body, [[cyber]] the mind, [[cyberia]] the state — a cycle (bodies feed the mind, the mind guides the state, citizens embody). they are not seats in the registry; the registry is what they are made of. listing cyb inside the stack it depends on was a structural error, corrected 2026-09-02. [[lytics]] however IS a seat — measurement is how cybernetics closes its loop — and returned to play. the stack's own top-tier seats are [[cell]] (hold), [[cyb/root/ward|ward]] (authorize), [[vault]] (keep) and [[sigma]] (evaluate). the full interaction graph, with every concrete input and output, is drawn at [soft3.org/chart](https://soft3.org/chart); the boundary rule and the loop — four motions, two guarantees, one attractor, every verb from this table — live in [[crystallization]] and are drawn at [soft3.org/scheme](https://soft3.org/scheme).

| grown on soft3 | verb | what it is |
|---|---|---|
| [[warriors/erga|erga]] | mine | the Autolykos warrior — honeycrisp's first paying customer |
| [[optica]] | publish | the graph, rendered — cyber.page and every subgraph site |
| [[Signal Studio|studio]] | sign | hemera particles · mudra neurons · ADR-036 signals |
| [[warriors/zoya|zoya]] · [[warriors/mona|mona]] · [[warriors/xena|xena]] | mine | the other warriors, each a physics question answered |

the four runtimes in work — [[soft3/nox|nox]] · [[glia]] · [[wysm]] · [[kern]] — form a proof-contract ladder from unconditional down to conditional-on-host; the long-term arc lowers every soul toward nox through trident LIR. [[kern]] wraps wgpu into an authored component; [[nu]] (vendored Nushell) is the external piece outside the authored-verb table. the layered map and the five architectural gaps live in [[stack-completeness]]; the network-tier borders in [[component-boundaries]].

## the verdict

**the stack is complete in principle and peer-pair in fact.** [[lytics]] remains the strongest single-node evidence: cybergraph + bbg + hemera + mudra + inf holding real, adversarial-facing traffic in production. and as of 2026-09-01 **the wire exists, and the graph routes it**: `cy wire up` converges cells over [[radio]] QUIC with no cybernode between them — and *following is a cyberlink*. `FOLLOW → neuron` on my chain is the subscription; `ANTENNA → endpoint` and `SOCKET → ip:port` on a node's own chain are its address record; the wire obeys those links, syncing exactly the followed chains and dialing any followed neuron whose antenna the cell holds. proven with three nodes on one machine: a cell that knew only one peer followed a stranger's neuron, received its chain through the friend, unpacked the address *from graph content*, and dialed the stranger directly — discovery, subscription and routing as signals, replicated by the mechanism they route.

**milestone №1 is crossed in its first form; the next three are named:** the same wire across two machines (relay / hole-punching, which radio already carries); gossip beyond a pair (fan-out is [[foculus]]'s reconciliation seat, not a pair's rebroadcast); and the [[cyb]] shell app speaking the protocol its own `cy` already speaks — with [[mudra]] keys instead of endpoint-derived neurons. the fork-choice has still never chosen a fork; that remains foculus's unproven half.

## critical, and out of scope

seven questions no roadmap in the stack currently owns. each one is ignorable today and existential at the first real scale.

1. **the economics of writing.** φ* ranks particles, but who may write, and at what cost? lytics prices a signal at 0.042 s of PoW — one app's local answer. an open cybergraph with free writes drowns in spam on day one. [[tok]] is published; a live economy, and the bridge from the [[bostrom]] snapshot to it, is nowhere.
2. **key lifecycle.** [[mudra]] derives and claims; nothing recovers, rotates, or revokes. a lost seed is a lost neuron forever, and one person on two devices is formally two neurons. for a state whose citizens are keys, identity loss is not a bug — it is a constitutional crisis without a constitution.
3. **protocol upgrade.** foculus's parameters are derived and documented; the procedure for *changing* them on a running network does not exist. a hard fork of a φ*-converged network is undefined behavior.
4. **deletion and liability.** an append-only content-addressed graph will be handed illegal bytes — every such network has been. right-to-be-forgotten versus content addressing, moderation versus censorship: the hardest political question, currently unasked.
5. **storage economics.** state grows forever; bbg makes reads provable but gives nobody a reason to hold the bytes. a filecoin-shaped hole between [[soft3/bbg|bbg]] and [[tok]].
6. **adversarial audit.** the consensus theorem is conditional, the truth-serum unaudited, and mudra's cryptography is self-built. before real value moves, someone paid to break it must try. no roadmap budgets this.
7. **operations.** one duplicated launchd job recently ate the strongest dev machine for hours. a network of nodes needs health, metrics and alerting as protocol citizens, not as an afterthought — networks die of boring things.

## the SDK

| dir | what | status |
|-----|------|--------|
| [js/](js/) | JavaScript/TypeScript SDK (current Bostrom chain) | active |
| [schema/](schema/) | canonical wire format definitions | draft |
| [cli/](cli/) | `soft3` stack CLI + node [0.10.0](https://crates.io/crates/soft3) · `cyber` product face — true-cyber [0.7.0](https://crates.io/crates/true-cyber) | published |
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
