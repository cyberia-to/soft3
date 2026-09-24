---
title: component boundaries
tags: cyber, soft3, roadmap, architecture
crystal-type: roadmap
crystal-domain: cyber
status: draft
---

# component boundaries — the transport, crypto and consensus tier

[[hemera]], [[mudra]], [[radio]], [[tade]], [[foculus]] and [[bbg]] meet at the execution, persistence and wire boundaries. this roadmap assigns responsibilities and records the moves that get there. the sync repo has merged into [[foculus]], bringing the structural-sync substrate and fork-choice into one crate.

The [storage architecture](../specs/storage.md) governs content persistence and
transport boundaries. Its [delivery project](storage/README.md) owns the shared
identity decision, BBG integration, Radio extraction and consumer migration.
Ordering, VDF and transport-crypto changes remain separate workstreams here.

the layer-protocol view lives in [[foculus/specs/structural-sync|structural-sync.md]] (which signal field belongs to which of the verification layers). this document is the orthogonal view: which component owns which mechanism, and where the same mechanism is implemented twice.

## the diagnosis

Sharing a Hemera primitive does not establish equal file identities or wire
formats. The [storage audit](../audit/file-storage-2026-09-24/README.md) records
the actual paths and incompatibilities. [S1](storage/identity.md) must reconcile
the sponge, structural, BAO and polynomial constructions before transport cutover.

The iroh-derived Radio surface needs a caller-by-caller responsibility inventory:

- `iroh-docs` (range-based set reconciliation) + `iroh-willow` (Willow sync) — these are [[foculus]]'s reconciliation domain.
- Blob persistence, tags, partial-transfer records and GC — migrate to [[bbg]].
- Transport TLS and signatures — preserve the selected Radio transport
  profile; any migration to [[mudra]] mechanisms requires a separate design and
  security qualification.
- Per-protocol postcard / irpc framing — distinguish transport internals from
  application framing before mapping the latter to the shared framing contract.
- `cyber-bao` verified streaming — reconcile identity and proof semantics before
  selecting a canonical Hemera verifier (see below).

the remaining duplications sit on the hemera↔radio, hemera↔foculus, and mudra↔foculus seams.

## clean boundaries

one responsibility per component. the verb is the whole job.

| component | owns | consumes |
|---|---|---|
| [[hemera]] | sponge primitives, canonical identity rules and structural verification chosen by S1 | nebu (field) |
| [[lens]] | polynomial commitments and openings, where selected by S1 | its mathematical/hash dependencies; the file adapter binds a commitment to particle above both Lens and Hemera |
| [[bbg]] | durable file parts, state/history, staging, coverage, retention and recovery metadata | selected local backend and canonical verifier |
| [[mudra]] | confidentiality and key distribution: seal (KEM), stealth (NIKE), veil (FHE), quorum (threshold), the identity definition, and the VDF primitive | hemera (hash), nebu / genies / jali (algebras) |
| [[tade]] | wire framing: marker + sigil + render + varint + payload, plus the minimal stream-control set | bytes only |
| [[radio]] | transmit: existing transport, connections, hole-punching, relay and gossip; bounded content streams | injected source/sink and verification interfaces, selected framing and transport-crypto profile |
| [[foculus]] | the whole reconciliation engine: structural availability and merge (erasure coding / Reed-Solomon, DAS, CRDT reconciliation, layer-2 ordering) *and* the fork-choice that completes the merge on conflict — the τ-threshold rule over φ\*, nullifier double-spend, the epoch beacon. fork-choice is a pluggable strategy, so the availability/merge substrate runs without the tri-kernel for trusted deployments | tru (φ\*, Focus strategy only), hemera (NMT, hash), nebu (RS), mudra (VDF), radio (transport), tade (frames), bbg (state), zheng (proof) |

Target storage orchestration (arrows show calls):

```text
applications → cybergraph → bbg → selected durable backend
                    ↓
                 foculus → radio → remote transport

BBG and host adapters consume canonical identity/proof interfaces.
Foculus persists sync state through BBG; BBG returns local coverage.
```

## verified streaming belongs to hemera

Hemera owns the canonical cryptographic verification relation. Radio invokes
that verifier through its transfer adapter. The existing `hemera::stream` and
`radio/cyber-bao` paths are inputs to S1, with no assumed identity or format
equivalence. A wrapper swap alone cannot establish agreement with File or
Cybergraph particles.

S5 upgrades Radio after S1 supplies the construction, vectors and migration
rules. It removes the competing BAO address and obsolete implementation once
the replacement transfer path passes range, resume and import qualification.
Retained proof side data lives in BBG throughout the target architecture.

## the duplication ledger

Each row identifies a boundary to reconcile. Verify live callers and protocol
requirements before classifying a dependency as removable duplication.

| mechanism | second home (to remove) | canonical home | move |
|---|---|---|---|
| verified-streaming identity/proof | `radio/cyber-bao` | Hemera construction selected by S1 | S5 migrates validated content and uses the canonical verifier |
| content persistence / transfer recovery / GC | Radio blob database and files | BBG content service | S3–S5 move durable records behind injected interfaces; no Radio-owned store |
| NMT node hashing | `foculus/src/nmt.rs` (String / hex) | `hemera::tree::hash_node_nmt` (field) | foculus calls hemera |
| VDF | `foculus/src/vdf.rs` (impl) | `mudra::delay` (primitive) | foculus consumes mudra's `vdf_prove` / `vdf_verify` |
| erasure / DAS | `hemera/roadmap/erasure-coding.md` (claim) | `foculus/src/{erasure,das}.rs` | retire the hemera roadmap; foculus owns it |
| CRDT reconciliation | `radio` iroh-docs / iroh-willow | `foculus` reconciliation | move the engines to foculus; radio transmits |
| φ\* / tri-kernel | `foculus/specs/provable-consensus.md` (re-derived) | [[tru]] | foculus references φ\*, never derives the kernel |
| transport crypto | Radio's selected TLS/signature profile | Radio integration, cryptographic mechanisms in their owning libraries | separate qualified migration proposal; preserve working transport during storage extraction |

## ownership decisions to settle

two borders need a call before the moves land.

layer-2 ordering (hash chain, VDF, equivocation detection). the code has `Signal`, `SignalChain`, and equivocation in `foculus/src/chain.rs`, and [[soft3/cybergraph|cybergraph]] re-exports them. [[foculus/specs/structural-sync|structural-sync.md]] says cybergraph owns layer 2. one of the two is the source of truth; pick it, move the code or rewrite the spec to match, and fix the stale comments that point at a non-existent `cybergraph/src/vdf.rs`.

mudra scope creep. `mudra/specs/place.md` (location via RTT and MDS) proves position through network geometry — structural, closer to [[foculus]] than to confidentiality (network geometry, not a confidentiality primitive). and `mudra/specs/delay.md` claims signal ordering and rate-limiting in its usage section, which is foculus's job. mudra keeps the VDF primitive; the ordering semantics go to foculus.

## hygiene

- tade houses the prysm dialect catalog (`spec/7-catalog.md`, `molecule.rs`); tade already flags it for migration to prysm. tade keeps the framing substrate and the `(*,k)` dialect-declaration mechanism, not any one dialect's schemas. reconcile the spec-versus-impl vocabulary drift (`type`/`size` versus `sigil`/`render`) in the same pass.
- foculus's `store::GSet` is a last-writer-wins set with an `f64` confidence and a clock drift — a float in a field-arithmetic stack. make it a true grow-only set, or update the spec, and remove the float per [[tru/specs/arithmetic|field arithmetic]].
- retire stale roadmaps that describe a border the code already moved past: `hemera/roadmap/erasure-coding.md`, `mudra/.claude/plans/expand-mudra-scope.md`.
- write a boundary section into the CLAUDE.md of [[radio]], [[tade]], and [[foculus]] — they have none, which is how the borders drifted in the first place.
