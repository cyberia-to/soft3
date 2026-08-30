---
title: chains as plugins
tags: cyber, soft3, architecture, explainer
crystal-type: spec
crystal-domain: cyber
icon: "🔌"
alias: chain adapters, blockchain as plugin, universal verifying substrate, transport interception
---
# chains as plugins

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
| identity | [[mudra]] | a key is a key |
| framing | [[tape]] | marker · sigil · render · varint · payload, over any byte stream |
| content identity | [[hemera]] | Poseidon2, trees, verified streaming |
| transport | [[radio]] | QUIC · hole-punching · relay · gossip |
| reconciliation | [[foculus]] | availability, erasure, DAS, CRDT merge, fork-choice |
| storage | [[soft3/bbg\|bbg]] | one polynomial, ten dims |
| proving | [[zheng]] | SuperSpartan · Brakedown · folding |
| query | [[inf]] | datalog with fixed-point over the store |
| execution | [[soft3/nox\|nox]] · [[wysm]] | metered, deterministic, provable |
| acceleration | [[honeycrisp]] | the silicon under all of it |
| the node itself | [[cell]] | the app *is* the node |

the test of the claim is a ratio: the adapter should be thousands of lines, the substrate tens of thousands. if the adapter grows to match the substrate, the seam was cut in the wrong place.

this is the move LLVM made for compilers and VFS made for filesystems. neither invented a language or a disk format. both made the *next* one cheap.

## what falls out for free

once a chain is an adapter over the substrate, four things arrive that nobody built for that chain:

**provable reads.** state roots already exist — ethereum commits an MPT root, ergo commits an AVL+ root. that is the hard prerequisite, and it is already in consensus. [[inf]] answers a datalog query *with a proof against that root*. today the equivalent is an explorer or an indexer, and both are trusted. a query that carries its own proof is a different category of thing, not a better version of the same thing.

**a light client that is not a committee.** [[zheng]] folds header transitions: each block folds into an accumulator, history is never re-proved. the trust root moves from "512 validators signed" to "this proof verifies".

**one identity, one UI.** [[mudra]] for keys, [[cyb]] for the surface. a wallet for a new chain becomes a manifest, not a product.

**bridges as queries, not committees.** this is the sharpest consequence. if chain A's state and chain B's state are both particles in one graph, each proven against its own root, then a bridge is a *query over that graph*. the committee exists today only because the two states live in incompatible worlds and someone has to swear across the gap. put both in one verified graph and there is no gap to swear across.

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

this pattern is not speculative — it is a deployed industry. FIBRE and bloXroute exist because miners and validators pay real money for a hundred milliseconds, since latency is orphan risk and MEV. the difference is what you have to trust: those relays are trusted intermediaries. a relay carrying [[hemera]]-verified content is not trusted at all, because every chunk proves itself.

**4 — greenfield.** the surfaces with no incumbent are where the substrate simply *is* the protocol: light clients, proof distribution, the miner↔pool link, mobile. no displacement, no negotiation.

and the precedent is decisive: **ethereum already swapped transports.** the execution layer speaks devp2p; the consensus layer speaks libp2p. they did not migrate the old one — they *built the new surface on the new stack*. new surfaces adopt new transports. that is the whole opening.

### what content-addressing actually changes

the speed is the smaller half. the structural change is that peers stop mattering.

in devp2p you ask a peer for a block and trust it to send the right bytes; you find out at the end. with [[hemera]] verified streaming, verification is **incremental** — a lie is caught at the first bad chunk, not after the download. so you can fetch from anyone, from many at once, in parallel, and stop caring who they are.

that inverts the failure mode. a peer can no longer feed you wrong data. it can only **withhold** — and withholding is precisely what [[foculus]] is built against: erasure coding, data-availability sampling, reconciliation. the residual threat lands exactly on the component that already owns it.

### the rule that keeps this safe

an overlay that becomes your only path is also the way to eclipse you.

> **radio may make you faster. it may never make you blinder.**

concretely: keep a mandatory quota of legacy peers, always. treat the overlay as the *speed* path and the legacy mesh as the *honesty* path. validate everything under the same consensus rules regardless of which door it came through. an adapter that can be configured into overlay-only is a misconfiguration waiting to be exploited, so it should not be configurable that way.

## the costs, stated plainly

- **implementing a legacy protocol faithfully is unglamorous and long.** RLPx is an ECIES handshake, frame encryption and snappy; discv5 is its own world. get a detail wrong and you are disconnected or banned. this is months per chain, and no amount of substrate quality removes it.
- **relays cost money and centralize a little.** hole-punching fails on symmetric NAT, so relays are necessary, and someone runs them.
- **an open overlay invites sybils.** [[mudra]] gives real identity and the graph gives reputation, but the policy is design work that is not done.
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
