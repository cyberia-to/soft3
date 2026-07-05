---
title: component boundaries
tags: cyber, soft3, roadmap, architecture
crystal-type: roadmap
crystal-domain: cyber
status: draft
---

# component boundaries — the transport, crypto and consensus tier

five repos sit between [[soft3/nox|nox]] execution and the wire: [[hemera]], [[mudra]], [[radio]], [[tape]], [[foculus]]. they grew independently, and the borders blurred — most of the blur is concentrated in one place. this roadmap fixes each component to a single responsibility and records the moves that get there. (the sync repo has since merged into [[foculus]] — the structural-sync substrate and the fork-choice that completes its merge are now one crate, which resolves the sync↔foculus ownership drift this document originally flagged.)

the layer-protocol view lives in [[foculus/specs/structural-sync|structural-sync.md]] (which signal field belongs to which of the verification layers). this document is the orthogonal view: which component owns which mechanism, and where the same mechanism is implemented twice.

## the diagnosis

the hashing fear is unfounded. there is one hash home — [[hemera]] (Poseidon2 over Goldilocks). no sibling reimplements blake3/sha/keccak; the blake3 in [[radio]] is vendored upstream iroh, and radio's own `cyber-bao` delegates every hash to `hemera::tree`. the mess is structural, and it has one root.

[[radio]] is a wholesale fork of iroh. forking the whole P2P stack to swap one hash dragged four other components' responsibilities into radio:

- `iroh-docs` (range-based set reconciliation) + `iroh-willow` (Willow sync) — these are [[foculus]]'s reconciliation domain.
- vendored `rustls` + `ring` TLS and `ed25519-dalek` — these are [[mudra]]'s key-agreement and signature domain (and classical, which strains the post-quantum claim).
- per-protocol postcard / irpc framing — this is [[tape]]'s framing domain.
- `cyber-bao` verified streaming — a reimplementation of `hemera::stream` (see below).

the remaining duplications sit on the hemera↔radio, hemera↔foculus, and mudra↔foculus seams.

## clean boundaries

one responsibility per component. the verb is the whole job.

| component | owns | consumes |
|---|---|---|
| [[hemera]] | content identity and its proof: Poseidon2 sponge, Merkle / NMT / sparse trees, content-defined chunking, and the verified-streaming codec (`stream` / `stream_async` / `async_io`) | nebu (field) |
| [[mudra]] | confidentiality and key distribution: seal (KEM), stealth (NIKE), veil (FHE), quorum (threshold), the identity definition, and the VDF primitive | hemera (hash), nebu / genies / jali (algebras) |
| [[tape]] | wire framing: marker + sigil + render + varint + payload, plus the minimal stream-control set | bytes only |
| [[radio]] | transmit: iroh transport (QUIC, hole-punching, relay, gossip), piping a hemera-encoded verified stream over the wire | hemera (streaming), tape (framing), mudra (transport crypto) |
| [[foculus]] | the whole reconciliation engine: structural availability and merge (erasure coding / Reed-Solomon, DAS, CRDT reconciliation, layer-2 ordering) *and* the fork-choice that completes the merge on conflict — the τ-threshold rule over φ\*, nullifier double-spend, the epoch beacon. fork-choice is a pluggable strategy, so the availability/merge substrate runs without the tri-kernel for trusted deployments | tru (φ\*, Focus strategy only), hemera (NMT, hash), nebu (RS), mudra (VDF), radio (transport), tape (frames), bbg (state), zheng (proof) |

dependency order is a clean DAG, bottom up:

```text
hemera   tape          (leaves)
   │       │
 mudra     │
   │       │
 radio ────┘
   │
foculus                 (substrate + fork-choice, one crate)
```

## verified streaming belongs to hemera

`hemera/rs/src/stream.rs` already implements the full combined pre-order BAO codec — `encode`, `decode`, `outboard`, incremental verification against a root — with an async twin in `stream_async.rs` and `async_io.rs`. this is hemera's proof-of-content path: given a root, prove the bytes match it.

`radio/cyber-bao` reimplements the identical format on top of the same `hemera::tree` primitives. the move: delete `cyber-bao`, expose a thin async wrapper over `hemera::stream_async`, and let radio carry the hemera-encoded stream rather than re-encode it. radio then neither hashes, nor builds the tree, nor encodes the stream — it dials, holepunches, relays, gossips, and pipes.

## the duplication ledger

each row is one mechanism implemented twice. the move makes it one.

| mechanism | second home (to remove) | canonical home | move |
|---|---|---|---|
| verified-streaming codec | `radio/cyber-bao` | `hemera::stream` / `stream_async` | radio wraps hemera; drop the fork |
| NMT node hashing | `foculus/src/nmt.rs` (String / hex) | `hemera::tree::hash_node_nmt` (field) | foculus calls hemera |
| VDF | `foculus/src/vdf.rs` (impl) | `mudra::delay` (primitive) | foculus consumes mudra's `vdf_prove` / `vdf_verify` |
| erasure / DAS | `hemera/roadmap/erasure-coding.md` (claim) | `foculus/src/{erasure,das}.rs` | retire the hemera roadmap; foculus owns it |
| CRDT reconciliation | `radio` iroh-docs / iroh-willow | `foculus` reconciliation | move the engines to foculus; radio transmits |
| φ\* / tri-kernel | `foculus/specs/provable-consensus.md` (re-derived) | [[tru]] | foculus references φ\*, never derives the kernel |
| classical transport crypto | radio rustls / ed25519 | [[mudra]] seal / stealth | aspirational: route the QUIC handshake through mudra |

## ownership decisions to settle

two borders need a call before the moves land.

layer-2 ordering (hash chain, VDF, equivocation detection). the code has `Signal`, `SignalChain`, and equivocation in `foculus/src/chain.rs`, and [[soft3/cybergraph|cybergraph]] re-exports them. [[foculus/specs/structural-sync|structural-sync.md]] says cybergraph owns layer 2. one of the two is the source of truth; pick it, move the code or rewrite the spec to match, and fix the stale comments that point at a non-existent `cybergraph/src/vdf.rs`.

mudra scope creep. `mudra/specs/place.md` (location via RTT and MDS) proves position through network geometry — structural, closer to [[foculus]] than to confidentiality (network geometry, not a confidentiality primitive). and `mudra/specs/delay.md` claims signal ordering and rate-limiting in its usage section, which is foculus's job. mudra keeps the VDF primitive; the ordering semantics go to foculus.

## hygiene

- tape houses the prysm dialect catalog (`spec/7-catalog.md`, `molecule.rs`); tape already flags it for migration to prysm. tape keeps the framing substrate and the `(*,k)` dialect-declaration mechanism, not any one dialect's schemas. reconcile the spec-versus-impl vocabulary drift (`type`/`size` versus `sigil`/`render`) in the same pass.
- float audit (done): the flagged `f64` in `store::GSet` is gone — `FileEntry` is last-writer-wins by integer `u64` timestamp, no float. the whole consensus path (fork-choice, finality, φ*, support-switching, beacon, settlement) is `tru::Fx` fixed-point end to end. the `f64` that remains — `vdisk` rebalancing ratios, `das::confidence` (a `1 − 2⁻ᵏ` report, test-only), byte-formatting display — is off the consensus/proof path (local storage heuristics + availability reporting + display), the mandate's allowed boundary. `store::GSet` LWW is the *virtual-filesystem* file merge, a different layer from the signal-consensus merge — which foculus implements as conflict-detection + fork-choice, not LWW, so there is no LWW-vs-`vec.md`-P1 divergence to reconcile.
- retire stale roadmaps that describe a border the code already moved past: `hemera/roadmap/erasure-coding.md`, `mudra/.claude/plans/expand-mudra-scope.md`.
- write a boundary section into the CLAUDE.md of [[radio]], [[tape]], and [[foculus]] — they have none, which is how the borders drifted in the first place.
