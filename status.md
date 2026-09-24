---
title: status
tags: cyber, soft3
crystal-type: spec
crystal-domain: cyber
alias: stack, soft3 registry, component registry, stack registry, soft3 status
---
# soft3 status

the component registry and its honest state: every repo, its verb, its release, and where it actually stands. soft3 is the [[cyber]] component of the [[cybics]] WORK triad — and internally it unfolds the same seven-triad spiral that [[cybics]] uses for all knowledge. form defines the primitive rules. mass is what knowledge is literally made of. space is where it exists at scale. life is where it becomes intelligent. word is what it means. work is how it runs. play is where we act together.

States distinguish evidence: **local** — implemented and tested in the current
checkout, with its supported profile declared; **live** — recorded deployed
traffic; **published** — a recorded package release; **unwired** — built but
awaiting integration; **spec** — paper only; **dormant** — inactive work;
**blocked** — waiting on another component. Release/crates columns preserve the
recorded release links. The 2026-09-13 neuron migration updates local source
evidence; it neither republishes those releases nor establishes remote upgrades.

| repo | verb | state | release | crates | what it does |
|---|---|---|---|---|---|
| [[soft3]] | compose | live | [0.10.0](https://github.com/cyberia-to/soft3/releases/tag/v0.10.0) | [0.10.0](https://crates.io/crates/soft3) | the stack CLI · `soft3 node` · a release is a [[soft3/conformance|conformance snapshot]] |
| [[cybics/form|form]] |  |  |  |  |  |
| [[honeycrisp]] | accelerate | live | [0.2.0](https://github.com/cyberia-to/honeycrisp/releases/tag/v0.2.0) | [0.2.0](https://crates.io/crates/honeycrisp) | NEON/AMX/SME · Metal GPU · ANE · zero-copy unimem |
| [[strata]] | math | live | [0.1.1](https://github.com/cyberia-to/strata/releases/tag/v0.1.1) | [0.1.1](https://crates.io/crates/cyber-strata) | nebu 𝔽_p · genies 𝔽_q · jali R_q |
| [[hemera]] | hash | live | [0.3.1](https://github.com/cyberia-to/hemera/releases/tag/v0.3.1) | [0.3.1](https://crates.io/crates/cyber-hemera) | [[Poseidon2]] sponge · identity · trees · verified streaming |
| [[lens]] | commit | published | [0.1.3](https://github.com/cyberia-to/lens/releases/tag/v0.1.3) | [0.1.3](https://crates.io/crates/cyber-lens) | 5 PCS backends, one per algebra · serde for Commitment/Opening |
| [[cybics/mass|mass]] |  |  |  |  |  |
| [[soft3/cybergraph\|cybergraph]] | link | live | [0.1.1](https://github.com/cyberia-to/cybergraph/releases/tag/v0.1.1) | [0.1.1](https://crates.io/crates/cybergraph) | jets · memos · types · knowledge |
| [[soft3/bbg\|bbg]] | store | live | [0.2.1](https://github.com/cyberia-to/bbg/releases/tag/v0.2.1) | [0.2.1](https://crates.io/crates/bbg) | 1 polynomial · 10 dims · ~200B proofs · QueryProof serde |
| [neuron](../neuron/README.md) | act | local | — | — | one subject · identity-only library · durable progs/invocations · bounded Rune execution · continuation/recovery; [evidence](audit/neuron-cell/implementation.md) |
| [[fs]] | mount | spec | — | — | particles · patches · sync · repo is empty |
| [[tok]] | pay | published | [0.1.1](https://github.com/cyberia-to/plumb/releases/tag/v0.1.1) | [0.1.1](https://crates.io/crates/cyber-tok) | Coin · Card · conservation · no live economy yet |
| [[cybics/space|space]] |  |  |  |  |  |
| [vault](../cyb/specs/private-vault.md) | keep | local | — | — | cyb custody and encrypted private state · subject/network namespaces · legacy ciphertext preserved; host-owned role, no standalone release |
| [[mudra]] | encrypt | live | [0.1.0](https://github.com/cyberia-to/mudra/releases/tag/v0.1.0) | [0.1.0](https://crates.io/crates/cyber-mudra) | KEM · dCTIDH · AEAD · TFHE · threshold |
| [[radio]] | transmit | live | [0.1.0](https://github.com/cyberia-to/radio/releases/tag/v0.1.0) | [0.1.0](https://crates.io/crates/cyber-radio) | QUIC · BAO streaming · gossip · carries `cyb/sync/0` |
| [[tade]] | frame | published | [0.1.0](https://github.com/cyberia-to/tade/releases/tag/v0.1.0) | [0.1.0](https://crates.io/crates/tade) | typed particle framing over any byte stream |
| [[foculus]] | sync | unwired | [0.1.2](https://github.com/cyberia-to/foculus/releases/tag/v0.1.2) | [0.1.2](https://crates.io/crates/foculus) | chain · VDF · equivocation · DAS · erasure · CRDT · engine partial, never deployed |
| [ward](../cyb/specs/runtime-authority.md) | authorize | local | — | — | current binding/grant checks at admission and dispatch · captured subject/network · revocation and worker generation fences; cyb host role |
| [[cybics/life|life]] |  |  |  |  |  |
| [[tru]] | converge | published | [0.1.1](https://github.com/cyberia-to/tru/releases/tag/v0.1.1) | [0.1.1](https://crates.io/crates/cyber-tru) | φ* · eigenvectors · cyberank |
| [[foculus]] | agree | spec | [0.1.2](https://github.com/cyberia-to/foculus/releases/tag/v0.1.2) | [0.1.2](https://crates.io/crates/foculus) | [[collective focus theorem]] → finality · fork-choice unproven in the wild |
| [[sigma]] | evaluate | spec | — | — | assessment: scores and gradings become links — a seat proposed, not yet a repo |
| [soma](../soma/README.md) | think | local | — | — | durable tasks · context/model capture · tools/children/schedules · real local glia provider and CLI/Bevy bodies; [evidence](../soma/audit/neuron-composition.md) |
| [[cybics/word|word]] |  |  |  |  |  |
| [[neural]] | mean | spec | — | — | sigil → word → link → sentence → motif → dialect |
| [[trident]] | compile | live | [0.2.0](https://github.com/cyberia-to/trident/releases/tag/v0.2.0) | [0.2.0](https://crates.io/crates/trident-lang) | .tri → .nox by default — the full chain build → prove → verify runs through [[joy]] (verified 2026-09-08) |
| [[rune]] | eval | published | [0.1.0](https://github.com/cyberia-to/rune/releases/tag/v0.1.0) | [0.1.0](https://crates.io/crates/cyber-rune) | Rs · hint · host · eval · async |
| [[inf]] | query | live | [0.1.0](https://github.com/cyberia-to/inf/releases/tag/v0.1.0) | [0.1.0](https://crates.io/crates/inf-cli) | datalog · fixed-point over BBG |
| [[cybics/work|work]] |  |  |  |  |  |
| [[soft3/nox\|nox]] | run | live | [0.2.0](https://github.com/cyberia-to/nox/releases/tag/v0.2.0) | [0.2.0](https://crates.io/crates/cyber-nox) | 18 patterns + jets · unconditional proof · executes trident programs via joy |
| [[zheng]] | prove | live | [0.3.1](https://github.com/cyberia-to/zheng/releases/tag/v0.3.1) | [0.3.1](https://crates.io/crates/zheng) | [[SuperSpartan]] · Brakedown · [[sumcheck]] · universal step CCS 2026-09-09: any program ≤ 2 accumulator groups, ~1–2 KB proofs, every wire byte verifier-read; residuals: verifier-checked fold (degree ≥ 2 rows), lens Brakedown evaluation binding (lens#6) |
| [[glia]] | infer | live | — | — | .model runtime · cond/model |
| [[eidos]] | certify | blocked | — | — | CIC type-checker · theorem → cyberlink · zheng bridge is a stub |
| [[wysm]] | sandbox | dormant | — | — | WASM souls · gas-metered · cond/host · no commits since june |
| [[kern]] | shade | spec | — | — | Metal/Vulkan/WebGPU shaders · cond/host |
| [[cybics/play|play]] |  |  |  |  |  |
| [[mir]] | render | live | — | — | positions + features → [[R-1.0]] world |
| [[prysm]] | paint | live | — | — | tade dialect · chunks → UI |
| [[lytics]] | measure | live | — | — | signed visitor events · PoW-priced · cohorts · funnels — cybernetics is impossible without measurement |

[[Cyb]] is the robot product, [[cyber]] the protocol, and [[cyberia]] the state.
The robot attaches neurons for keys, networks and devices. A neuron may execute
several progs; prog/task/operation IDs identify retained work and introduce no
additional signer. Ward authorizes, vault keeps custody, and Soma composes the
mind's tasks under that same subject. GraphSession hosts many neurons over
cybergraph/BBG; Log renders retained history. Sigma presents identities, holdings
and actions. Repository extraction and release status are separate decisions
from these ownership boundaries. The current contract is
[neuron](specs/neuron.md), with [cyb architecture](../cyb/specs/architecture.md)
and [component evidence](audit/neuron-cell/implementation.md). The chart and
scheme at [soft3.org](https://soft3.org) are publication surfaces whose deployed
revision must be checked independently.

| grown on soft3 | verb | what it is |
|---|---|---|
| [[warriors/erga|erga]] | mine | the Autolykos warrior — honeycrisp's first paying customer |
| [[optica]] | publish | the graph, rendered — cyber.page and every subgraph site |
| [[Signal Studio|studio]] | sign | hemera particles · mudra neurons · ADR-036 signals |
| [[joy]] | prove | the cyber warrior — runs, proves, verifies trident programs on nox; `cargo install cyber-joy` ([0.3.0](https://github.com/cyberia-to/joy/releases/tag/v0.3.0)) |
| [[warriors/zoya|zoya]] · [[warriors/mona|mona]] · [[warriors/xena|xena]] | mine | the other warriors, each a physics question answered |

The four execution families — [[soft3/nox|nox]], [[glia]], [[wysm]], [[kern]] —
have distinct supported machine and evidence profiles. Neuron reuses Rune and
worker adapters; a recorded host result is an observation, while a proof must
identify its statement and verifier. Soul is retained configuration, and its
progs can target compatible workers. The trusted Nu console and cooperative
GPU driver declare their own limits. The [execution model](specs/execution-model.md)
and [stack completeness](roadmap/stack-completeness.md) define these boundaries.

## the verdict

The current local implementation closes one subject and execution model across
neuron, cyb, Soma and native clients. Signed native publication pins network,
subject, request and complete payload; retained unknown outcomes reconcile the
original request. The [consumer audit](audit/neuron-cell/consumers.md) records
unchanged mudra/domain/ADR-036 bytes, identity-only dependency profiles, foreign
adapter boundaries and migrated SDK consumers. The [launcher audit](../cyber/audit/neuron-launcher.md)
covers explicit legacy import/authentication and real process restart tests.
These checks establish local behavior. Full Hermes provider/channel parity,
remote consensus and production rollout require their own evidence.

Historical evidence, 2026-09-01: the [[wire]] demonstration used `cy wire up`
and three graph hosts on one machine. FOLLOW, ANTENNA and SOCKET links supplied
subscription and endpoint discovery; a host discovered a third peer through an
existing peer. Its original transport/author profile remains the evidence for
that run. It does not establish the current signed-native profile over radio.
Likewise, the recorded lytics deployment is historical traffic evidence; its
current source replay/provenance fixes are assessed by the consumer audit.
Cross-machine radio integration, broader gossip and consensus fork-choice remain
separate network acceptance work.

## critical, and out of scope

Broader network questions extend beyond the local migration. Existing local
controls resolve parts of them; the remaining network/economic work needs its
own acceptance evidence.

1. **the economics of writing.** φ* ranks particles, but who may write, and at what cost? lytics prices a signal at 0.042 s of PoW — one app's local answer. an open cybergraph with free writes drowns in spam on day one. [[tok]] is published; a live economy, and the bridge from the [[bostrom]] snapshot to it, is nowhere.
2. **key lifecycle.** Current custody can reopen or explicitly attach existing keys; bindings/devices can be changed or revoked and private state recovered with its retained key material. One neuron can be attached across devices. Changing the key changes H(compressed pubkey) in the native profile. Stable-subject key rotation, recovery without retained custody and network-wide revocation require an explicit versioned authority protocol; local binding revocation alone cannot provide them.
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
| [cli/](cli/) | recorded releases: `soft3` [0.10.0](https://crates.io/crates/soft3), `true-cyber` [0.7.0](https://crates.io/crates/true-cyber); current [native client](../true-cyber/README.md) and [node launcher](../cyber/specs/cli.md) use compatible sibling sources | published snapshot + local migration |
| [mcp/](mcp/) | MCP server — cybergraph tools for AI assistants | scaffold |
| [py/](py/) | Python SDK | scaffold |

The common adapter vocabulary targets these operations. Implemented native and
foreign profiles expose their actual supported subset; scaffolds do not acquire
an implementation from this table:

```text
particle(content)              → particle     hemera hash of bytes
cyberlink(context, from, to)  → action       bind subject/network/grant and exact content
query(particle, dimension)     → value+proof  BBG Lens opening
verify(root, proof)            → bool         proof verification
submit(signed_action)          → receipt      declared network/profile; endpoint receipt or proof tier
```

```ts
import { CyberClient } from '@cybercongress/cyber-js'

const client = await CyberClient.connect('https://rpc.bostrom.cybernode.ai')
const result = await client.search('PARTICLE_CID') // use the chain's original particle encoding
```

BBG/lens proof serde has a versioned implementation; generic query RPC and schema
adoption still require their declared profile. The native signed adapter and
tested Cosmos SDK are separate wire contracts. Preserve foreign chain IDs,
addresses and signing bytes, and use [identity consumer boundaries](specs/identity-consumers.md)
when connecting another SDK. A native Hash or transport key supplies no authority
by itself.

see [[soft3/docs|the whitepaper]] for the foundations — the methods behind one mind, many languages, open world.
