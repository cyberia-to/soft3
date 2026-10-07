---
tags: cyber, soft3, architecture, proposal, network, foculus, radio
crystal-type: process
crystal-domain: cyber
status: proposed
date: 2026-10-07
---
# the network in planes — radio · sync · foculus · solong

> builds on [[soft3/proposals/cybergraph-sync-tade-architecture|cybergraph + sync + tade]] (2026-06, implemented), which merged the `sync` repo into foculus to kill two owners of one mechanism, and on [[soft3/roadmap/component-boundaries|component boundaries]] (executed 2026-10-07). this proposal cuts the result along a different line — by *plane*, not by *mechanism* — so that each mechanism still has exactly one owner, but the planes that change for different reasons and ship to different nodes stop living in one crate.

## the picture

the network of the stack, read as a stack:

| plane | verb | what | today | LOC |
|---|---|---|---|---|
| reach | dial | QUIC, hole-punching, relay, discovery — reach a peer by key | radio: iroh, iroh-base, iroh-relay, iroh-dns-server, quinn, nettools | ~110k, ~95 % upstream |
| carry | transmit | bytes by particle, verified streaming (hemera), pub-sub | radio: iroh-blobs, cyber-bao, iroh-gossip, iroh-car | ~40k, cyber's own code |
| frame | frame | typed bytes on any boundary | tade ([[soft3/proposals/tade-one-exchange|one exchange format]]) | 1k |
| **sync** | reconcile | how bytes and signals become agreed facts: ordering, availability, completeness, local merge, history trust | foculus (data plane) | ~4.5k |
| **foculus** | agree | who is right when facts conflict: fork-choice over φ\*, finality, epochs, beacon | foculus (control plane) | ~2.3k |
| **solong** | reward | who gets what for which work: tickets, fold-mining, Shapley settlement, receipts, the settle mesh | foculus (incentive plane) | ~3.8k |
| link | link | validate · order · apply · expose | cybergraph | — |

foculus today is three planes in one crate (13k LOC). they change for three different reasons — codecs (sync), the φ\* theory in tru (foculus), the token design in tru/specs/rewards (solong) — and ship to three different nodes: a light client needs sync and the tip; a validator needs all three; a miner needs solong and sync. the name "focus" describes 20 % of the lines.

## radio — not split

radio is already twelve crates; as a *repository* it is a fork tracked whole against upstream iroh. splitting it into repositories would cost the upstream tracking and buy nothing. two things change instead:

1. the **reach / carry** line is made explicit in `radio/README.md`: reach is upstream code with one cyber change (the handshake); carry is cyber's code (cyber-bao, the hemera substitution in blobs, gossip over particles).
2. the transport-independent cores of the reconciliation engines leave radio for sync: `iroh-docs/src/ranger.rs` (range-based set reconciliation) and willow's `proto/`. the QUIC sessions around them stay in radio and call sync. this row is already open in component boundaries; this proposal gives it its destination.

## foculus — three crates, one workspace, one arrow

```text
sync     reconcile   frames · signal_codec · chain · vdf · erasure · das · nmt · store · node · vdisk · tip · step   (+ ranger, willow proto)
   │
foculus  agree       conflict · fork · focus · reconcile · finality · finality_evidence · beacon · epoch · epoch_cert     (+ tru)
   │
solong   reward      tickets · ticket_proof · settlement · rewards · marginal_cert · wire (FSET) · gossip (settle mesh) · radio_settle   (+ tok)
```

the arrow points one way. a crate below never imports a crate above. the repository stays `foculus` with a workspace of three members; history is intact; separate repositories only if evolution proves the need.

### where each module goes, and why

**sync — the data plane.** `frames` (the cyber-dialect tade frames: signal, intent, chunk request/response — what rides on radio), `signal_codec` (the canonical bounded Signal bytes; cybergraph's most-used import), `chain` + `vdf` (layer 2 ordering; [[foculus/specs/delay|delay]]), `erasure` + `das` + `nmt` (layers 3–4; [[foculus/specs/erasure|erasure]]), `store` + `node` + `vdisk` (layer 5 local merge — the former sync repo), `tip` (clock C, history trust for light clients), `step` (the universal zheng step row shared by tip, tickets and pay — the lowest brick). sync's public API is exactly what cybergraph and the soft3 node import today.

**foculus — the control plane.** `conflict` (a pure function of content), `fork` (the ForkChoice trait, Serialize / MinHash), `focus` (the φ\* strategy), `reconcile` (detection wired to choice), `finality` (τ_D over the particle's own domain), `finality_evidence` (clock A objects for light clients), `beacon` (b_E over the VDF), `epoch`, `epoch_cert`. depends on sync (chain, vdf, tip) and tru (φ\*). nobody outside foculus imports these today — the control plane is internal, which is what makes it isolable.

**solong — the incentive plane.** `tickets` (settlement tickets, the fold-mining monoid), `ticket_proof` (σ for tickets and fold steps), `settlement` (magnitude, division, the leaderless lottery), `rewards` (claim → ρ → Shapley → receipt), `marginal_cert` (certified Δφ⁺ marginals), and — this was the surprise of the audit — `wire` (the `FSET` settle-gossip codec), `gossip` (epidemic push of claims and self-accumulators) and `radio_settle` (ALPN `foculus/settle/1`): the whole settle mesh is reward machinery, not a general data plane. depends on sync (step, frames), foculus (beacon, certificate) and tok. the name is Shapley's own game — *So Long Sucker* (1950, with Nash, Hausner and Shubik), the coalition game the value grew out of; here the coalition is paid by its contribution and betrayal is what [[karma]] cuts.

**not in any plane.** `pay_proof` (σ for money intents) is payment, not reward → [[tok]], sharing `step` with the tickets. `live` (the full / partial / light node assembled from all three planes) is the product on top → stays in the top-level crate, or moves to the soft3 node. `cli` stays with whichever binary owns it.

## the two cuts

the import graph today has cycles (`agree → settle` 9 edges, `sync → settle` 9, `sync → agree` 4). two decisions remove them; one is protocol, one is mechanical.

1. **the epoch certificate stops carrying the payout.** today `epoch` and `epoch_cert` import tickets, rewards and marginal_cert: the certificate "optionally settled rewards" — consensus and reward are one *object*. the doctrine says otherwise: rewards are self-minted against the proven Δφ\* ([[tru/specs/rewards|rewards]], "no aggregator"), i.e. a *function* of the certified epoch, not a field of it. after the cut the certificate binds φ\*, finality and the beacon; solong computes `reward(cert, tickets)` from it. the quantity stays one — φ\* decides both finality and pay — the object becomes two. **this is a protocol change: the certificate format changes.**
2. **the wire does not know the reward's frame kinds.** `gossip`, `live` and `wire` import tickets and settlement because the settle mesh was written inside the data plane. after the cut the settle mesh is solong's (it moves whole: wire, gossip, radio_settle), and any frame kind it needs on the shared transport is registered by its owner through the handler mechanism foculus already uses with radio (`on_frame`). mechanical.

`solong → foculus` keeps its two edges (the lottery takes randomness from the beacon) — that is the arrow pointing the right way.

## what it buys

- the control plane can be audited without token economics in scope; a trusted deployment runs sync + foculus with no solong at all (the ledger already promises "fork-choice is pluggable" — this makes the promise a crate boundary).
- the mining lottery and fold-mining evolve without touching finality; the cyb miner depends on solong + sync, not on consensus internals.
- the light client is sync + tip, nothing else.
- the tade proposal lands in one crate (sync) instead of across a 13k-line one.

## acceptance

this proposal closes only when all of the following hold:

1. `foculus/Cargo.toml` is a workspace of `sync`, `foculus`, `solong`; `cargo tree` shows no edge from a lower crate to a higher one;
2. `epoch_cert` carries no reward fields; solong's settlement is a function of the certificate and the tickets, with a test that the same certificate and tickets yield the same payout on two nodes;
3. `ranger` and willow `proto` build inside sync with hemera fingerprints, and radio's sessions call them;
4. cybergraph, soft3 node, cyb (miner) and zheng cli import from the plane they belong to, and `cargo test` is green in every repository touched;
5. every PR is merged — closed by the last merge, not the first.

## sequencing

specification first: this page, the certificate format in `foculus/specs/epoch.md`, the module map above as `foculus/README.md`. implementation after the harness work in flight — not before, not interleaved — in the order sync → foculus → solong, since each cut only frees the next. the ladder already reads in planes ([soft3.org/layers](https://soft3.org/layers), rung 7).

[[radio]] · [[foculus]] · [[tade]] · [[tok]] · [[tru]] · [[cybergraph]] · [[soft3/proposals/tade-one-exchange|tade — one exchange format]]
