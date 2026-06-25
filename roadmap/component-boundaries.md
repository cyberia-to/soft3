---
title: component boundaries
tags: cyber, soft3, roadmap, architecture
crystal-type: roadmap
crystal-domain: cyber
status: draft
---

# component boundaries — the transport, crypto and consensus tier

six repos sit between [[soft3/nox|nox]] execution and the wire: [[hemera]], [[mudra]], [[radio]], [[tape]], [[sync]], [[foculus]]. they grew independently, and the borders blurred — most of the blur is concentrated in one place. this roadmap fixes each component to a single responsibility and records the moves that get there.

the layer-protocol view lives in [[sync/specs/sync|sync.md]] (which signal field belongs to which of five verification layers). this document is the orthogonal view: which component owns which mechanism, and where the same mechanism is implemented twice.

## the diagnosis

the hashing fear is unfounded. there is one hash home — [[hemera]] (Poseidon2 over Goldilocks). no sibling reimplements blake3/sha/keccak; the blake3 in [[radio]] is vendored upstream iroh, and radio's own `cyber-bao` delegates every hash to `hemera::tree`. the mess is structural, and it has one root.

[[radio]] is a wholesale fork of iroh. forking the whole P2P stack to swap one hash dragged four other components' responsibilities into radio:

- `iroh-docs` (range-based set reconciliation) + `iroh-willow` (Willow sync) — these are [[sync]]'s reconciliation domain.
- vendored `rustls` + `ring` TLS and `ed25519-dalek` — these are [[mudra]]'s key-agreement and signature domain (and classical, which strains the post-quantum claim).
- per-protocol postcard / irpc framing — this is [[tape]]'s framing domain.
- `cyber-bao` verified streaming — a reimplementation of `hemera::stream` (see below).

the remaining duplications sit on the hemera↔radio, hemera↔sync, and mudra↔sync seams, plus spec-versus-code ownership drift in sync and foculus.

## clean boundaries

one responsibility per component. the verb is the whole job.

| component | owns | consumes |
|---|---|---|
| [[hemera]] | content identity and its proof: Poseidon2 sponge, Merkle / NMT / sparse trees, content-defined chunking, and the verified-streaming codec (`stream` / `stream_async` / `async_io`) | nebu (field) |
| [[mudra]] | confidentiality and key distribution: seal (KEM), stealth (NIKE), veil (FHE), quorum (threshold), the identity definition, and the VDF primitive | hemera (hash), nebu / genies / jali (algebras) |
| [[tape]] | wire framing: marker + sigil + render + varint + payload, plus the minimal stream-control set | bytes only |
| [[radio]] | transmit: iroh transport (QUIC, hole-punching, relay, gossip), piping a hemera-encoded verified stream over the wire | hemera (streaming), tape (framing), mudra (transport crypto) |
| [[sync]] | structural availability and merge: erasure coding (Reed-Solomon), DAS, CRDT reconciliation | hemera (NMT, hash), nebu (RS), mudra (VDF), radio (transport), tape (frames), bbg (state) |
| [[foculus]] | global finality: the τ-threshold rule over φ\*, conflict and fork-choice, nullifier double-spend, the epoch beacon | tru (φ\*), sync (conflict and VDF outputs), zheng (proof) |

dependency order is a clean DAG, bottom up:

```text
hemera   tape          (leaves)
   │       │
 mudra     │
   │       │
 radio ────┘
   │
 sync
   │
foculus
```

## verified streaming belongs to hemera

`hemera/rs/src/stream.rs` already implements the full combined pre-order BAO codec — `encode`, `decode`, `outboard`, incremental verification against a root — with an async twin in `stream_async.rs` and `async_io.rs`. this is hemera's proof-of-content path: given a root, prove the bytes match it.

`radio/cyber-bao` reimplements the identical format on top of the same `hemera::tree` primitives. the move: delete `cyber-bao`, expose a thin async wrapper over `hemera::stream_async`, and let radio carry the hemera-encoded stream rather than re-encode it. radio then neither hashes, nor builds the tree, nor encodes the stream — it dials, holepunches, relays, gossips, and pipes.

## the duplication ledger

each row is one mechanism implemented twice. the move makes it one.

| mechanism | second home (to remove) | canonical home | move |
|---|---|---|---|
| verified-streaming codec | `radio/cyber-bao` | `hemera::stream` / `stream_async` | radio wraps hemera; drop the fork |
| NMT node hashing | `sync/src/nmt.rs` (String / hex) | `hemera::tree::hash_node_nmt` (field) | sync calls hemera |
| VDF | `sync/src/vdf.rs` (impl) | `mudra::delay` (primitive) | sync consumes mudra's `vdf_prove` / `vdf_verify` |
| erasure / DAS | `hemera/roadmap/erasure-coding.md` (claim) | `sync/src/{erasure,das}.rs` | retire the hemera roadmap; sync owns it |
| CRDT reconciliation | `radio` iroh-docs / iroh-willow | `sync` reconciliation | move the engines to sync; radio transmits |
| φ\* / tri-kernel | `foculus/reference/provable-consensus.md` (re-derived) | [[tru]] | foculus references φ\*, never derives the kernel |
| classical transport crypto | radio rustls / ed25519 | [[mudra]] seal / stealth | aspirational: route the QUIC handshake through mudra |

## ownership decisions to settle

two borders need a call before the moves land.

layer-2 ordering (hash chain, VDF, equivocation detection). the code has `Signal`, `SignalChain`, and equivocation in `sync/src/chain.rs`, and [[soft3/cybergraph|cybergraph]] re-exports them. [[sync/specs/sync|sync.md]] says cybergraph owns layer 2. one of the two is the source of truth; pick it, move the code or rewrite the spec to match, and fix the stale comments that point at a non-existent `cybergraph/src/vdf.rs`.

mudra scope creep. `mudra/specs/place.md` (location via RTT and MDS) proves position through network geometry — structural, closer to [[sync]] than to confidentiality (network geometry, not a confidentiality primitive). and `mudra/specs/delay.md` claims signal ordering and rate-limiting in its usage section, which is sync's job. mudra keeps the VDF primitive; the ordering semantics go to sync.

## hygiene

- tape houses the prysm dialect catalog (`spec/7-catalog.md`, `molecule.rs`); tape already flags it for migration to prysm. tape keeps the framing substrate and the `(*,k)` dialect-declaration mechanism, not any one dialect's schemas. reconcile the spec-versus-impl vocabulary drift (`type`/`size` versus `sigil`/`render`) in the same pass.
- sync's `store::GSet` is a last-writer-wins set with an `f64` confidence and a clock drift — a float in a field-arithmetic stack. make it a true grow-only set, or update the spec, and remove the float per [[tru/specs/arithmetic|field arithmetic]].
- retire stale roadmaps that describe a border the code already moved past: `hemera/roadmap/erasure-coding.md`, `mudra/.claude/plans/expand-mudra-scope.md`.
- write a boundary section into the CLAUDE.md of [[radio]], [[tape]], and [[foculus]] — they have none, which is how the borders drifted in the first place.
