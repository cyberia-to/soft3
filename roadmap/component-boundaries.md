---
title: component boundaries
tags: cyber, soft3, roadmap, architecture
crystal-type: roadmap
crystal-domain: cyber
status: executed 2026-10-07
---

# component boundaries — the transport, crypto and consensus tier

five repos sit between [[soft3/nox|nox]] execution and the wire: [[hemera]], [[mudra]], [[radio]], [[tade]], [[foculus]], with [[neuron]] as the subject they act for. they grew independently and the borders blurred — most of the blur in one place. this page fixed each component to a single responsibility, recorded the moves, and on 2026-10-07 the moves were executed: every row below says what was done, what was reversed on contact with the code, and what stays open. (the sync repo had already merged into [[foculus]]; that resolved the sync↔foculus drift this page originally flagged.)

the layer-protocol view lives in [[foculus/specs/structural-sync|structural-sync.md]] (which signal field belongs to which verification layer). this document is the orthogonal view: which component owns which mechanism, and where the same mechanism was implemented twice.

## the diagnosis

the hashing fear was unfounded. there is one hash home — [[hemera]] (Poseidon2 over Goldilocks). no sibling reimplements blake3/sha/keccak; the blake3 in [[radio]] is vendored upstream iroh, and radio's own `cyber-bao` delegates every hash to `hemera::tree`. the mess was structural, with one root:

[[radio]] is a wholesale fork of iroh. forking the whole P2P stack to swap one hash dragged other components' responsibilities into radio — `iroh-docs` / `iroh-willow` reconciliation engines (foculus's domain; still there, see the ledger), vendored rustls + ring + ed25519 (mudra's domain, and classical), per-protocol postcard framing (tade's), and `cyber-bao` (a second copy of hemera's verified-streaming format).

the second root was [[mudra]]: a confidentiality repo that had grown an identity implementation (the only code it had), two specs about time and position, and a module index advertising an `order` page that did not exist.

## clean boundaries

one responsibility per component. the verb is the whole job.

| component | owns | consumes |
|---|---|---|
| [[hemera]] | content identity and its proof: Poseidon2 sponge, Merkle / NMT / sparse trees, content-defined chunking, and the verified-streaming codec (`stream` / `stream_async` / `async_io`) | nebu (field) |
| [[neuron]] | the subject: `NeuronId`, proof-based authority ([[neuron/specs/proof-authority|spec]]), the implemented secp256k1 profile and NSIG1 envelope, domain-scoped keys, the legacy-key bridge — crate `neuron-auth` | hemera (hash), neuron-id |
| [[mudra]] | confidentiality and key distribution: seal (KEM), stealth (NIKE), veil (FHE), quorum (threshold), private recovery. **specification only** — no code | hemera (hash), nebu / genies / jali (algebras) |
| [[tade]] | wire framing: marker + sigil + render + varint + payload, plus the minimal stream-control set | bytes only |
| [[radio]] | transmit: iroh transport (QUIC, hole-punching, relay, gossip), piping a hemera-encoded verified stream over the wire | hemera (streaming), tade (framing) |
| [[foculus]] | the whole reconciliation engine: ordering (hash chain, step, equivocation, the VDF — [[foculus/specs/delay|delay]]), availability (erasure coding, DAS — [[foculus/specs/erasure|erasure]]), CRDT merge, position ([[foculus/specs/place|place]]) *and* the fork-choice that completes the merge — the τ-threshold rule over φ\*, nullifier double-spend, the epoch beacon. fork-choice is pluggable, so the substrate runs without the tri-kernel for trusted deployments | tru (φ\*, Focus strategy only), hemera (NMT node rule, hash), nebu (RS), radio (transport), tade (frames), bbg (state), zheng (proof) |

dependency order is a clean DAG, bottom up:

```text
hemera   tade          (leaves)
   │       │
neuron-auth│           (identity over hemera)
   │       │
 radio ────┘           (transport; classical TLS until mudra's seal exists)
   │
foculus                (substrate + fork-choice, one crate)

mudra                  (specs; implemented by nobody yet — the seal row below waits on it)
```

## the duplication ledger — executed

each row is one mechanism that was implemented twice. the status column is what actually happened.

| mechanism | second home | canonical home | status |
|---|---|---|---|
| verified-streaming codec | `radio/cyber-bao` | `hemera::stream` / `stream_async` | **re-scoped.** the plan said "delete cyber-bao, wrap hemera". on contact: iroh-blobs needs tree *geometry* (block sizes, chunk ranges, partial encodings, slices, an async FSM) that hemera's whole-tree API has no reason to carry, and carrying it would pull `iroh-io` + `genawaiter` into a zero-dependency crate. what was duplicated is the *format*, and that is now one: `cyber-bao/tests/hemera_format.rs` pins root, outboard pairs and the combined stream byte-identical to `hemera::stream` at the whole-tree block size (verified across 0 B … 1 MB). cyber-bao stays as radio's ranged adapter over hemera's format; radio's README and CLAUDE.md say so |
| NMT node hashing | `foculus/src/nmt.rs` (String / hex, own `nmt_node:` preimage) | `hemera::tree::hash_node_nmt` (field, capacity-committed bounds) | **done.** foculus's NMT is rebuilt over hemera's leaf and NMT-parent rules with `u64` namespaces and 32-byte digests; a test asserts the two-leaf root equals `hash_node_nmt` directly |
| VDF | ledger said: `foculus/src/vdf.rs` → consume `mudra::delay` | **reversed:** foculus | mudra never implemented delay; foculus had the working code. `mudra/specs/delay.md` moved to `foculus/specs/delay.md` with its status told honestly (Goldilocks squaring is sequential work, not a VDF; the unknown-order construction is the spec) and the audit's equivocation correction folded in. the advertised-but-absent `order` page is gone: ordering *is* `foculus/src/chain.rs` |
| erasure / DAS | `hemera/roadmap/erasure-coding.md` (claim) | `foculus/src/{erasure,das}.rs` | **done.** the roadmap's substance became `foculus/specs/erasure.md` over the code that exists; hemera's roadmap index points there. `das::confidence` no longer returns an `f64` — it returns the exponent (`confidence_bits`) |
| CRDT reconciliation | `radio/iroh-docs` (range-based set reconciliation — `ranger.rs` — replicas, QUIC sync session), `radio/iroh-willow` (3D range reconciliation, meadowcap) | `foculus` | **open — a refactor, not a file move.** the first execution pass deleted the two engines because nothing in the stack called them; that was wrong and was reverted the same day: they are the set-reconciliation substrate that closes [[foculus/specs/structural-sync|structural-sync]] layers 3–5 (compare fingerprints over ranges, exchange the difference), and foculus's `reconcile.rs` resolves *conflicts between signals*, it does not reconcile *sets* over a wire. the move splits each crate at the transport line: `iroh-docs/src/ranger.rs` and willow's `proto/` (the algorithms, transport-independent) become foculus modules; the sessions over QUIC and the blob/gossip plumbing stay in radio and call foculus. until then the engines stay in radio, documented as foculus's tenants (`radio/README.md` § boundary, `radio/CLAUDE.md`) |
| φ\* / tri-kernel | `foculus/specs/provable-consensus.md` re-derived the kernel, its contraction rate and a 1.4B-constraint circuit | [[tru]] | **moved, not linked.** the circuit — operators, per-iteration constraints, convergence, algebraic-NMT reads, capacity, prover/verifier times — is `tru/specs/proving.md`; foculus's page keeps only what the proof does to a network (what it replaces, what remains, recursion, phases). the "624 million" closing figure that contradicted the page's own total is gone |
| identity | `mudra/src/*` + `mudra/specs/{identity,neuron-auth,bridge,bridge-proof,neuron-measures}.md` | [[neuron]] | **done.** crate `neuron-auth` (`neuron/auth`), specs under `neuron/specs/` (`proof-authority.md`, `local-authority.md` § NSIG1 envelope, `bridge.md`, `bridge-proof.md`, `measures.md`). ten dependents switched (`cyber`, `cyb`, `soma`, `vault`, `lytics`, `cyberia-my`, `neuron`, …); the `cyber-mudra` crate no longer exists |
| classical transport crypto | radio rustls / ring / ed25519 | [[mudra]] seal / stealth | **open, with a condition.** mudra has specified seal and stealth and implemented neither; a post-quantum QUIC handshake cannot be routed through a primitive that does not exist. the row moves when `seal` has code. until then radio's README names the classical placeholder for what it is |

## ownership decisions — settled

**layer-2 ordering.** the code (`Signal`, `SignalChain`, `VdfProof`, equivocation) lives in `foculus/src/chain.rs` and `vdf.rs`; [[cybergraph]] re-exports it and calls it at its [[cybergraph/specs/order|order]] verb. `structural-sync.md` said cybergraph owned layer 2 — it now says foculus, as cybergraph's own specs already did. the stale comments pointing at a non-existent `cybergraph/src/vdf.rs` are gone.

**mudra scope.** executed above: delay and place to foculus, identity to neuron, mudra keeps confidentiality. `mudra/.claude/plans/expand-mudra-scope.md` is deleted.

**layer 4 and the local merge.** `structural-sync.md` still named a `cyb/sync` repo for availability and device-level merge; that repo merged into foculus. the table now reads foculus for layers 2, 4 and 5 and radio for transport only.

## hygiene

- `foculus::store::GSet` — the float the earlier draft flagged was not in `store.rs` (the set carries no confidence); the `f64` was `das::confidence`, fixed above. `cli.rs` still formats fixed-point values as floats for display, which is display, not decision.
- retired: `hemera/roadmap/erasure-coding.md`, `mudra/.claude/plans/expand-mudra-scope.md`.
- boundary sections written: `radio/CLAUDE.md`, `foculus/CLAUDE.md` (new); `mudra/CLAUDE.md` rewritten to four modules and no code.
- **open:** tade houses the prysm dialect catalog (`spec/7-catalog.md`, `molecule.rs`) that tade itself flags for migration to [[prysm]], and the spec-versus-impl vocabulary drift (`type`/`size` vs `sigil`/`render`). not touched in this pass; `tade/CLAUDE.md` still does not exist.
- **open:** `soft3` PR #3 (docs/vault-spell) adds a `cyber-mudra` dependency to `soft3/crate` and `audit/neuron-cell/implementation.md`; on rebase it must take `neuron-auth` instead.

## what this changed on the ladder

[soft3.org/layers](https://soft3.org/layers): `mudra` moved from rung 5 (language) to rung 4 (cybergraph) beside `neuron` — a primitive lives where its noun is born, and the neuron is born in the book. rung 6 (cy) now shows the agent as one [[cy/specs/chroma|chroma]] — the eight organs that show — plus the eight that work unseen; `time` (log ← now → plan) and `name` moved from the robot to the agent.
