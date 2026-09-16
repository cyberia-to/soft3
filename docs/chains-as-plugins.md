---
title: chains as plugins
tags: cyber, soft3, architecture, explainer
crystal-type: spec
crystal-domain: cyber
icon: "🔌"
alias: chain adapters, blockchain as plugin, universal verifying substrate, transport interception
---
# chains as plugins

The normative composition model is the [execution model](../specs/execution-model.md).
An OS/protocol adapter can serve any number of compatible network instances;
genesis and endpoints configure an instance, while worker placement and
hardware are separate choices. This page explains the substrate/adoption idea.

The adapter scenarios below are design targets. Current migration evidence
covers the [native signed profile](../specs/signed-native-adapter.md) and the
[declared consumer profiles](../specs/identity-consumers.md). The generic robot
foreign attachment is observation-only until that domain has its own implemented
custody and authorization adapter. Preserved foreign SDK/wallet formats retain
their original network rules.

every blockchain client rebuilds the same seven things. soft3 builds them once, and the chain becomes the small part.

this is the claim, and the second half of this document is the part that is actually hard: a faster transport does not get adopted because it is faster. it gets adopted because someone benefits from it **alone**.

## the weld

a "blockchain protocol" is not one protocol. it is six concerns welded into one binary, and every chain welds them slightly differently:

```text
   consensus rules   —  what is valid
   state model       —  what exists
   encoding          —  how it serializes
   transport         —  how bytes move
   discovery         —  how peers find each other
   sync strategy     —  what you fetch, in what order
```

bitcoin, ethereum and ergo differ **enormously** in the first two. they differ **trivially** in the last four — everyone ends up with a gossip mesh over TCP, a peer table, and a fetch-headers-then-bodies loop. yet because the six are welded, every chain reimplements all six, and every new client for that chain reimplements them again.

the weld is why a light client is a multi-year project instead of a weekend.

## the unweld

cut along the seam. what is genuinely chain-specific stays in an adapter. everything else is substrate, written once:

| concern | who owns it | why |
|---|---|---|
| consensus rules | **adapter** | irreducibly per-chain |
| state model | **adapter** | UTXO vs accounts vs eUTXO |
| encoding | **adapter** | RLP vs sigma vs protobuf |
| identity | [neuron](../specs/neuron.md), [[mudra]], domain adapter | Native H(compressed pubkey) or explicit foreign domain/address/network; preserve derivation, custody and signing rules |
| framing | [[tade]] | marker · sigil · render · varint · payload, over any byte stream |
| content identity | [[hemera]] | Poseidon2, trees, verified streaming |
| transport | [[radio]] | QUIC · hole-punching · relay · gossip |
| reconciliation | [[foculus]] and the domain adapter | Native availability/order rules; foreign fork-choice, finality and proof validation remain domain-specific |
| storage | [[soft3/bbg\|bbg]] | one polynomial, ten dims |
| proving | [[zheng]] | SuperSpartan · Brakedown · folding |
| query | [[inf]] | datalog with fixed-point over the store |
| execution | neuron/Rune/worker and selected [[soft3/nox\|nox]] / [[wysm]] family | Captured subject/network/grant, bounded work and declared evidence profile |
| acceleration | [[honeycrisp]] | the silicon under all of it |
| graph host | product GraphSession + cybergraph/BBG | Hosts retained multi-neuron graph state; process placement supplies no signing identity |

A robot attaches neurons for different keys, networks and devices. Progs,
invocations and worker placements identify work under those subjects. Endpoint
keys authenticate transport; network/genesis pins identify the destination;
neuron custody authorizes the exact action. A chain UID, address string, contract
ID or shard ID cannot be cast into a native NeuronId to manufacture authority.

the test of the claim is a ratio: the adapter should be thousands of lines, the substrate tens of thousands. if the adapter grows to match the substrate, the seam was cut in the wrong place.

this is the move LLVM made for compilers and VFS made for filesystems. neither invented a language or a disk format. both made the *next* one cheap.

## what an adapter must establish

The substrate provides reusable mechanisms for four capabilities. Each chain
adapter must implement and test the binding from those mechanisms to its own
state and consensus rules:

**Provable reads.** Ethereum's MPT and Ergo's AVL+ roots belong to their native
commitment schemes. An adapter must authenticate the selected root and verify
the mapping from the queried data to that root. Indexing it in BBG or returning
an inf relation alone does not supply that proof. The current neuron runtime
query projection explicitly reports local, unproved data.

**Light verification.** A folded header profile must express the foreign
transition/finality rules, trusted starting point, coverage and verifier
parameters. Folding compresses the evidence for that statement; the original
chain's authority and trust assumptions remain part of the statement.

**One robot, several neurons.** Cyb provides one surface over explicitly attached
native and foreign identities. Mudra and domain adapters retain the proper keys,
addresses and message bytes. A manifest can select a supported adapter and
network; it cannot create the missing verifier, custody or spending authority.

**Cross-domain conditions.** A query can read authenticated state from two
domains. Settlement still needs each home ledger's issuer/spending rules,
freshness/finality assumptions, expiry and correlated receipts. The
[domain ladder](../../cyber/specs/domain-ladder.md) and
[3C contract](../../cyber/3c.md) preserve these obligations. Content IDs and shared
storage make facts referenceable; they do not execute a transfer or waive a
foreign chain's consensus rules.

## the transport question

here is the honest part.

soft3's wire is faster than devp2p. QUIC gives multiplexing without head-of-line blocking, 0-RTT reconnect, and connection migration — your node survives a laptop moving from wifi to cellular. hole-punching and relays mean a node behind a home NAT actually works, which is not a performance note but a decentralization one: a large share of ethereum nodes live in datacenters partly because home nodes are annoying to run.

none of that is an argument for adoption. **network protocols have brutal network effects.** a node that speaks only [[radio]] can talk to nobody. being right is not a deployment strategy.

### the only question that matters

> **what is the smallest number of participants who must adopt before adoption pays off?**

call it N. it decides everything:

| N | outcome |
|---|---|
| all | dead on arrival — requires a fork and a social campaign |
| 2 | viable — any two peers who adopt beat the ones who did not |
| 1 | unstoppable — a single participant benefits alone |

rank the surfaces by N and the roadmap writes itself:

| surface | N | why |
|---|---|---|
| miner ↔ pool | **1** | we own both ends — [erga](https://github.com/cyberia-to/erga) and the pool. stratum is ancient; replacing it needs nobody's permission |
| light client ↔ network | **1** | there is no incumbent P2P for light clients. today they call centralized RPC. we compete with a trusted API, not with a protocol |
| proof distribution | **1** | proofs are a new data type. no legacy protocol carries them, so no legacy protocol has to be displaced |
| node ↔ node, via sidecar relay | **2** | two nodes running the sidecar beat gossip between them |
| replacing devp2p wholesale | **all** | do not try |

the strategy is not "convince the network". it is **start where N=1 and let the overlay accrete**.

### four ways in

**1 — advertise, upgrade, fall back.** exactly how HTTP/3 was deployed. a server answers over TCP with `Alt-Svc: h3=":443"` — *I also speak QUIC over there*. the client remembers, tries QUIC next time, and silently falls back forever if it fails. no flag day, no coordination, no permission.

the blockchain analogue already exists: devp2p **negotiates capabilities**. a node advertises `eth/68`, `snap/1`. advertising `radio/1` alongside them is protocol-legal today. two nodes that both speak it hand off to QUIC out of band; every other peer sees an ordinary node and notices nothing.

**2 — the bilingual gateway.** the adapter speaks the legacy protocol faithfully — to the old network it is an unremarkable peer. among themselves, soft3 nodes use the substrate. every such node is therefore a **gateway**: it pulls from the legacy mesh and republishes into the overlay. dual-stack is how IPv6 shipped; it is the only pattern that has ever worked for replacing a live protocol.

**3 — the sidecar.** do not modify the node at all. run the relay beside an unmodified geth or ergo node and attach over its normal P2P port. the node stays exactly as its operator installed it; it simply starts receiving blocks earlier, because the sidecar has a fast backchannel to other sidecars.

FIBRE and bloXroute are the historical industry examples motivating the sidecar
proposal. A soft3 relay carrying hemera-verified content can verify each chunk
against an expected content root. The adapter still authenticates that root and
checks protocol validity, freshness and availability; a valid content hash alone
does not establish any of those properties.

**4 — greenfield.** the surfaces with no incumbent are where the substrate simply *is* the protocol: light clients, proof distribution, the miner↔pool link, mobile. no displacement, no negotiation.

and the precedent is decisive: **ethereum already swapped transports.** the execution layer speaks devp2p; the consensus layer speaks libp2p. they did not migrate the old one — they *built the new surface on the new stack*. new surfaces adopt new transports. that is the whole opening.

### what content-addressing actually changes

the speed is the smaller half. the structural change is that peers stop mattering.

in devp2p you ask a peer for a block and trust it to send the right bytes; you find out at the end. with [[hemera]] verified streaming, verification is **incremental** — a lie is caught at the first bad chunk, not after the download. so you can fetch from anyone, from many at once, in parallel, and stop caring who they are.

For a correctly authenticated expected root, content verification detects changed
bytes. Peers can still withhold data, and an unauthenticated or stale root remains
an adapter-level failure. Erasure coding, data-availability sampling and
reconciliation address their declared availability assumptions; they complement
the domain's validity, coverage and finality checks.

### the rule that keeps this safe

an overlay that becomes your only path is also the way to eclipse you.

> **radio may make you faster. it may never make you blinder.**

concretely: keep a mandatory quota of legacy peers, always. treat the overlay as the *speed* path and the legacy mesh as the *honesty* path. validate everything under the same consensus rules regardless of which door it came through. an adapter that can be configured into overlay-only is a misconfiguration waiting to be exploited, so it should not be configurable that way.

## the costs, stated plainly

- **implementing a legacy protocol faithfully is unglamorous and long.** RLPx is an ECIES handshake, frame encryption and snappy; discv5 is its own world. get a detail wrong and you are disconnected or banned. this is months per chain, and no amount of substrate quality removes it.
- **relays cost money and centralize a little.** hole-punching fails on symmetric NAT, so relays are necessary, and someone runs them.
- **an open overlay invites sybils.** Mudra verifies key claims; a person can hold many keys. Reputation, admission cost and the selected network's Sybil policy remain explicit design and validation work.
- **fragmentation is a real risk.** if soft3 nodes prefer each other too strongly they drift from the network they are supposed to serve. the legacy quota is the mitigation, and it must be enforced, not advised.
- **[[zheng]] has not had an adversarial audit.** until it does, proofs are an engineering claim, not a security guarantee.

## the order of operations

start where N=1, prove the substrate on a chain small enough to finish, then carry the same machinery to the chain with the market.

| # | step | permission needed |
|---|---|---|
| 0 | miner↔pool over the substrate — both ends ours | none |
| 1 | provable light client on an existing state root | none — the root is already in consensus |
| 2 | sidecar relay beside unmodified nodes | none |
| 3 | `radio/1` as an advertised capability | none — devp2p already negotiates |
| 4 | folded header proofs — sync as one verification | none |
| 5 | provable execution over [[wysm]] | none, and years |

nothing on that list needs a fork. that is not an accident — it is the design constraint that made the list.

---

**the shape of the claim.** the goal is not to be the best client for any one chain. it is that the *next* verifying client, for any chain, costs an adapter instead of a company. chains become plugins; verification becomes the product; and the hardware it runs on is the hardware people already own.
