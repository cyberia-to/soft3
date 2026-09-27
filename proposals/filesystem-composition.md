---
title: filesystem composition
tags: soft3, fs, cyb, storage, architecture
crystal-type: proposal
crystal-domain: cyber
status: draft
---
# Filesystem composition

Proposed decision: [[fs]] owns the reusable filesystem model and service.
[[cyb]] embeds that service and owns the personal filesystem experience.
[[cyber]] can host an explicitly enabled service or encrypted replica.
[[bbg]] owns durable storage; [[radio]] carries authorized traffic.

This proposal connects the existing FS patch design to the storage work and
the neuron runtime. Its [source audit](../audit/storage/filesystem-composition-2026-09-27/README.md)
separates implemented paths, feature branches and design intent. Acceptance of
this proposal would amend the owner contracts listed below; its publication
alone changes no protocol, release pin or implementation status.

## Product, component, deployment

The existing [Cyb filesystem doctrine](https://github.com/cyberia-to/cyb/blob/46a05d2f5dac6cf31ef14e937e8bae7b57f77b87/parts/fs.md)
and [FS patch design](https://github.com/cyberia-to/fs/blob/75d438014eded7b71ad7d2411bcc36205d30e94d/patch/spec.md)
give the filesystem a clear home. In the proposed composition, Memory presents
files, Brain presents their relations, Com and `cy` operate on them, and Now
selects the current context. These adapters will consume the same FS state and
authority through the shared runtime. Today's `cy` still exposes graph commands;
the FS service and its common enforcement remain integration work.

Keep domain code in the existing `fs` repository. Start with a reusable Rust
library, separating the model, service and adapters in modules/features as
needed. Cyb composes it through its shared runtime. A headless consumer can use
the same library without Bevy, rendering or inference. No additional product
binary or mandatory daemon is needed to establish this boundary.

| Deployment | Composition | Connection requirement |
|---|---|---|
| Personal Cyb | FS + GraphSession + Cybergraph + one BBG owner; Memory/Com/`cy` are adapters | Local read, edit, rename and history work offline |
| Headless personal session | Same runtime and FS service, through `cy` or an SDK | GUI optional |
| Connected Cyb | Personal service plus selected peer/replica capabilities | Remote loss leaves retained local work usable |
| Cyber node | Optional FS hosting, shared content serving or encrypted replication, according to granted role | Node lifecycle and service administration belong to Cyber |

One database owner means one owner for each configured database, with explicit
namespaces and isolation. Several devices have separate replicas. A robot can
attach several neurons and networks; a directory, worker or storage process
does not create a signing subject. The [execution model](../specs/execution-model.md)
continues to govern worker placement and proof profiles.

## Ownership and dependency direction

```svgbob
+------------------------------------------------------------------+
| Cyb: Memory, Brain, Now, Com, cy                                 |
| Neuron file acts, scoped SDK calls                               |
+---------------------------------+--------------------------------+
                                  |
                                  v
+------------------------------------------------------------------+
| FS: names, channels, patches, revisions                          |
| Current authority: Ward grants, Vault operations, Mudra          |
+------------------+---------------------------------+-------------+
                   |                                 |
                   v                                 v
+------------------------------------+  +--------------------------+
| Cybergraph                         |  | Foculus                  |
| validate, publish, history         |  | reconcile, availability  |
| query, Inf views at pinned state   |<-+ shared content ports     |
+------------------------------------+  +--------------------------+
                   |                                 |
                   v                                 v
+------------------------------------+  +--------------------------+
| BBG                                |  | Radio                    |
| one database owner per store       |  | authorized transport     |
| content, indexes, retention        |  | injected sources, sinks  |
+------------------------------------+  +--------------------------+
```

Arrows show service use, not a requirement for each module to depend on every
crate below it. Host composition injects capabilities and owns lifecycle.

| Owner | Responsibility |
|---|---|
| Hemera / File | Canonical content identity, representation and verification; streaming interfaces for large content |
| FS | Contextual bindings, paths, directories, aliases, channels, revision/patch semantics, conflicts and retention intent |
| Cybergraph | Validated application publication, history, content access, request deduplication, head preconditions and shared query/sync ports |
| BBG | Atomic durable records, content parts, derived indexes, retention, recovery and reclamation on the selected Fjall/SSD or redb/HDD profile |
| Foculus | Reusable reconciliation, availability and transfer scheduling mechanisms; separately selected network finality profiles |
| Radio / Tade | Transport / framing through injected capabilities |
| Ward / Vault / Mudra | Current authorization / secret custody / cryptographic mechanisms |
| Cyb / Neuron / Soma | Product context and interfaces / subject-bound execution / task and tool orchestration |
| Inf | Query evaluation over a scoped, pinned source; declared proof support |
| Spark / Prysm / Mir / Evy | Open and render supplied content and views |
| Soft3 / Cyber | Shared composition and qualification / network product lifecycle and advertised services |

FS depends downward on the shared publication/content interfaces. Cybergraph
and BBG remain usable by Vault, Neuron and other applications directly. A Vault
credential, a model artifact and a document may share physical storage while
keeping their own admission and recovery rules. Generated exports, diagnostics
and temporary OS files keep their explicit host roles; canonical application
state has the BBG durability owner.

FS application history can remain private and locally committed. Its bindings
and patches produce scoped graph/query views; publishing an attributed change
into a public network uses that network's admission path explicitly. An
ApplicationGraph commit alone must not be reported as a finalized native Signal.

## The filesystem model

The filesystem exposes graph state through useful, contextual organization:

| Object | Meaning |
|---|---|
| File / particle | Immutable content / its identity under the selected canonical construction |
| Binding | An attributed association within a workspace; stable across an ordinary rename |
| Path | A deterministic directory/name projection within a namespace, channel and selected state |
| Revision | A retained content change with provenance and dependencies |
| Patch | An attributed change to bindings/content relations, with explicit prerequisites |
| Channel | A named selection of patches and its derived state/frontier |
| Conflict | Retained competing changes awaiting an explicit resolution |

For example, `mission/main:plans/landing.cyb` can select binding B at state H
and content P. Rename to `plans/descent.cyb` preserves B and P while creating
new namespace history. Editing creates content Q and a new revision of B.
Two bindings can refer to P and later evolve independently. A directory manifest
that includes its entries' names has its own particle, which changes on rename.

Directories are one navigable projection; tags, relations, saved queries and
Brain views refer to the same files. Ranked search and ambiguous graph labels
remain distinct from exact path resolution. Robot names in Cyb's `name` organ
retain their identity/token resolver; FS resolves the path inside the selected
workspace. The spelling above illustrates context, without freezing an address
grammar or replacing cybermark.

Independent patches should commute under a specified algebra. Concurrent
rename/edit must address the binding and dependencies, allowing both changes
when compatible; incompatible rename/rename or edit/edit preserves a conflict.
The existing patch draft's confluence claims require executable vectors and
proof obligations. Until that profile qualifies, explicitly advertise the local
serialized profile and reject stale writes. Wall-clock ordering or a hash-based
winner must never silently stand in for document merge semantics.

The target history is a dependency graph. Materialized indexes/checkpoints
accelerate selected views with verifiable provenance. The existing catalog's
radix index, conditional updates and retained history are useful implementation
material; its namespace-wide linear CAS is the initial local adapter, with a
declared concurrency limit. Preserve its identities and receipts during an
explicit import. Avoid presenting that adapter as completed patch/channel FS.

## One service contract for every interface

Capture the operation context before dispatch: acting neuron or authorized
reader, workspace/namespace, channel, selected state, visibility and current
grant. Capture a network binding when the operation uses one. Navigation after
dispatch must not retarget a pending write or a Soma task.

Proposed semantic surface, prior to Rust and wire syntax:

| Group | Operations and result |
|---|---|
| Resolve/read | Resolve particle/path/binding; stat, list, read/range, history and watch at an explicit state/cursor |
| Change | Create, edit, rename, unlink, apply patch, select channel and resolve conflict; exact request identity and relevant preconditions |
| Content | Stream import/export, acquire a retained read, report missing content, request fetch or protection |
| Availability | Describe local durability, retained closure, replication evidence and recovery status separately |

Reads and listings are paged against a pinned state. Watches have resumable
cursors and report a gap requiring resynchronization. Writes return the
committed state/revision and durable request receipt. Bytes may stage over many
transactions; the visible binding/head and its complete required retention
closure publish atomically after verification. An uncertain commit fences
dependent work until recovery establishes its outcome.

The service checks authority at the effect/publication boundary regardless of
whether the caller is a UI, Nu command, program or remote client. A read grant
also scopes enumeration and metadata. Ward's product prompts and grant issuance
compose with this enforcement; merely rendering a permission prompt establishes
no server-side control.

GUI and CLI share a live owner through local IPC, or an exclusive headless owner
opens the database when the host is absent. They must not independently open
and replay competing writable copies. Use the existing GraphSession convergence
work for composition and legacy imports instead of inventing another runtime.

## CLI and native filesystem integration

Make `cy fs …` the proposed terminal surface of Cyb. The embedded Nu adapter
offers structured `fs …` operations over the same service and reports the active
graph context. This names a command family, not a second filesystem engine.
Candidate examples are `cy fs ls`, `cy fs history`, `cy fs rename` and
`cy fs import`; finalize exact syntax in Cyb after the semantic contract.

Keep host paths and graph addresses typed and explicit. Nu's current OS
`open`/`ls`/`mv`/`rm` behavior continues within its native context. Import/export
cross the boundary deliberately. Optional OS mounts translate through FS with
declared write/flush/conflict semantics; FUSE and complete POSIX behavior can
follow the native API.

Interactive native Nu remains a trusted host shell with its declared OS access.
Adding `fs` commands does not sandbox `open`, `save` or external processes.
Capability-limited programs receive a restricted host surface, with process/OS
isolation where required, instead of inheriting that ambient shell. Qualification
must show that such a program cannot open the database or invoke unrestricted
OS commands to bypass its FS grant.

Cyber's CLI manages node/services, peers, retention policy and diagnostics.
Radio's CLI manages transport and protocol diagnostics. Move file/name product
commands out of Radio after the `cy` adapter passes parity gates. Keep temporary
compatibility calls thin and explicit; they acquire the service, never construct
their own database. The existing `true-cyber` headless client and Cyber node both
use the binary name `cyber`; resolve that packaging collision separately from
FS domain ownership.

## Synchronization and privacy

The [component-boundary roadmap](../roadmap/component-boundaries.md) records the
merge of Sync into Foculus. Reuse that substrate with a clear split:

| Decision | Owner |
|---|---|
| Which changes mean the same file state; how conflicts resolve | FS patch/channel profile |
| Which records a peer can submit or observe | Domain validator and current host authority |
| Discover missing authorized history/content; resume, transfer, erasure/DAS where selected | Foculus mechanisms over shared Cybergraph/BBG ports and Radio |
| Which peers, budgets, replicas and retention obligations to request | Application policy and host configuration |
| Which public network state becomes final | Selected Foculus consensus profile |

An offline save needs local authority and durability. Replication can proceed
under an authorized application profile without network consensus. Foculus's
current native Signal reconciler needs an application-facing seam; its
equivocation winner cannot implement FS conflicts or Vault writer selection.
Keep its erasure and reconciliation algorithms while replacing independent
chunk/registry writers with injected BBG ports and transport with Radio.

Private workspaces keep encrypted content and protected names/history outside
public aggregates. Publication is an explicit authorized operation. A content
hash supplies integrity; access policy and encryption supply confidentiality.
Deterministic plaintext identities and inventories can disclose equality or
enable guessing, so private discovery must declare its leakage and avoid public
announcements by default. Metadata, size, timing and replica visibility belong
in the qualification threat model.

Vault uses the same storage/transfer mechanisms and its own stricter lineage,
writer fencing, counters and recovery rules. FS can present an authorized Vault
view. Bootstrap reads its sealed state directly through the private application
port, so unlocking the filesystem never requires a key trapped behind that
same unlock. Generic patch merge cannot choose a Vault head.

Local save, retained history, complete remote copy and restore-tested protection
are distinct facts in every interface. Unlink changes a view; reclamation follows
retention roots and active readers. No arbitrary total file, library or revision
cap is introduced: bound requests, buffers, pages and background work, report
physical capacity and configured quotas, and test growth without full-history
materialization. Sparse clients can synchronize authorized metadata and fetch
content on demand: a visible binding with missing bytes reports that state.
Only the declared complete retained closure qualifies for a protection receipt.
Fast recovery needs retained indexes/checkpoints as well as
history; a promise of unlimited history does not make replay cost disappear.

## Delivery and decisions

Continue the existing
[storage project](https://github.com/cyberia-to/soft3/blob/189c8aa9809aed18417023c9b2f8b324dabd5b5b/roadmap/storage/README.md)
under its S work packages. The linked revision is a feature-branch baseline.

| Order | Work package | Concrete completion condition |
|---|---|---|
| 1 | S2: reconcile contracts | FS owns path/patch/channel semantics; Inf queries those views at the same state; Foculus generic mechanisms and consensus profiles have distinct ports |
| 2 | S2/S7: FS library and Cyb composition | Adapt the catalog into FS over shared application/content APIs; reuse GraphSession; headless offline create/edit/rename/history survives restart |
| 3 | S7: product adapters and import | Memory, `cy`, Nu and neuron acts share results/authority; import JSONL/catalog sources with preserved provenance; retire parallel writers after verification |
| 4 | S4/S5/S6: retention and sync | Move Foculus/Radio persistence into BBG; qualify closure, resume, replica receipts and retention/GC; retain useful network behavior |
| 5 | S7/S8: channels and recovery | Qualify patch conflicts/concurrency and checkpoints; recover files with names/history; pass Vault's independent recovery/fencing gates |
| 6 | S9: extraction and release | Delete superseded Radio storage/docs paths after live consumers migrate; qualify coherent pinned builds of Cyb/Cyber against soft3 |

S1 canonical identity proceeds alongside this work and gates public protocol
cutover. FS delegates identity to File/Hemera. Existing exact-byte Blob identity
supports local development; sponge primitives, structural roots and BAO roots
need an explicit compatibility decision before claiming one public particle.

Contract decisions to close in owner specs before their dependent implementation:

- `fs`: path normalization/case, binding identity, aliases, channel frontier,
  concurrent rename/edit and conflict resolution; exact publication/retention closure.
- `inf` and `soft3/specs/terms.md`: replace unconditional “latest assertion under
  a path wins” with the selected FS state/profile; preserve scoped query semantics.
- `foculus`: reconcile the older structural-sync ownership text with the Sync
  merger; expose application admission/merge ports and a lightweight build profile.
- `cyb`: common service lifecycle, captured context, `cy`/Nu vocabulary and effect
  grants. Align with the in-flight neuron/GraphSession migration.
- `soft3`: composition gates and registry/release closure. FS moves from `after/spec`
  to the shipping profile only with a pinned implementation and qualification.

The cross-interface acceptance case is one document: import binary content,
rename, edit, inspect earlier state, retry after a lost response, restart,
replicate, lose the primary device and restore names plus bytes. Run through
Memory, `cy`, Nu and a neuron act; require identical identities, histories,
permission failures and durability receipts. Add concurrent edit/rename,
revocation during transfer, incomplete replicas, private discovery, both
backends, and an interrupted migration with retained source data.

Proof qualification is separately named: application-head durability, range
integrity, query completeness and consensus finality establish different claims.
Current local application roots and Inf sources require their own authenticated
proof bindings before remote clients can treat them as proven filesystem views.
