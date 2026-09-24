---
title: storage architecture
tags: soft3, storage, bbg, radio, foundation, spec
status: accepted
---
# storage architecture

This is the stack-wide ownership and composition contract for [[file]] storage.
It applies to public files, private application data, [[vault]] records, program
artifacts and retained history. Accepted architecture is an implementation
obligation; qualification is tracked in the [storage project](../roadmap/storage/README.md).
The exact particle construction remains an explicit [decision gate](../roadmap/storage/identity.md).

## ownership

| component | responsibility | interface boundary |
|---|---|---|
| [[hemera]] | Sponge primitives, canonical identity rules and structural verification algorithms | Identity and proof algorithms with shared test vectors |
| [[lens]] | Polynomial commitments and openings where the selected construction requires them | Commitment/proof primitives with an explicit binding to the canonical particle |
| [[file]] | Immutable file abstraction, content interpretation and composition of the selected identity primitives | Descriptors, streams and ranges |
| [[fs]] | Names, paths, filesystem views, channels and patch semantics over the graph | Contextual resolution, namespace edits and historical views |
| [[bbg]] | All durable content, parts, indexes, retention roots, staging and recovery records | Local storage handles under the shared Database owner |
| [[cybergraph]] | Validated application history, content references and conditional publication | Authorized application operations over BBG |
| [[foculus]] | Reconciliation, missing-content scheduling, replication and provider acknowledgements | Local BBG operations and Radio transport sessions |
| [[radio]] | Connections, discovery, relay and transmission | Bounded streams; injected content source/sink capabilities |
| [[mudra]], [[vault]], Ward | Cryptographic mechanisms, secret custody and authorization respectively | Ciphertext and narrowly authorized operations |
| [[cyber]], [[cyb]], [[soft3]] adapters | Assembly and product interaction | The same content/history services used by every application |

Radio evolves in its existing repository. Its storage and reconciliation
responsibilities move behind these interfaces as their replacements qualify.
Transport protocol internals remain Radio's responsibility. Cryptographic
handshake changes require their own qualification.

If the selected file construction combines Lens with Hemera, its adapter
composes their interfaces above both libraries. Hemera's primitive layer stays
independent of Lens, preserving an acyclic implementation dependency graph.

## one persistence authority

Every durable byte and every record needed to resume, serve, retain or recover
it MUST pass through BBG. This includes transfer progress, partial content,
proof side data, catalogs and replication jobs. Radio has zero independently
owned databases, blob directories or persistent spools in the target system.
Transient transport buffers have explicit memory and flow-control budgets.
Disk-backed buffering uses a BBG staging capability.
Persistent transport credentials are supplied through the host's custody
interface. Session identity, content access and neuron authority have explicit
separate bindings.

The host selects a Database profile and path: Fjall for the SSD profile, redb
for the HDD profile. Content, application history and graph state on that store
share its owner, locks, transaction boundary and failure state. Radio receives
capabilities rather than database paths or backend constructors. Separate
devices are explicit BBG stores; their coordination uses durable receipts,
with atomicity scoped to each owner.

BBG performs local storage operations. A missing range is returned to the
caller; Foculus orchestrates retrieval through Radio. Network waits MUST occur
outside database transactions. Restarting Radio preserves all acknowledged
durable state and resumable transfers held by BBG.

## one file identity

A [[particle]] is the canonical identity of a file across local storage,
application references, synchronization and transport. All components MUST
consume the same identity contract. An internal part key, tree root, proof
commitment or transfer identifier has an explicit subordinate role; none may
silently substitute for the file's particle.

The contract MUST define content interpretation, length binding, ordering,
canonical encoding, structural invariants and their domain separation.
An opaque byte file binds its exact bytes. A semantic adapter such as `.cyb`
must declare which representation differences preserve identity and how exact
source bytes are retained when required. [Types](types.md) governs the
distinction between semantic structure and incidental wire framing.

Physical backend pages, compression, transfer frame boundaries and the choice
of provider MUST leave particle unchanged. Canonical logical segmentation,
if used by identity, is specified by Hemera and is independent of physical
layout. Every accepted range proof MUST bind its bytes, position and total
length to the requested particle. An auxiliary commitment requires a verified
binding to that particle before it can authenticate a range.

The decision gate reconciles the sponge, structural, BAO and polynomial paths.
Until it closes, new storage machinery can use an injected identity verifier;
canonical format freeze and replacement of the public transfer path wait for
the agreed construction and vectors. Identity compatibility and cryptographic
security qualification are separate release obligations.

## names, filesystem views and patches

[[cyb/parts/fs|The filesystem model]] exposes the graph through names, paths
and patch-based views. [Shared terms](terms.md) separates a file's content
identity from names and metadata expressed as cyberlinks.
[[fs/patch/spec|Patch semantics]] defines channels and changes to these views. FS owns their meaning;
Cybergraph validates their publication and BBG persists their data and indexes.

Resolution has an explicit context: neuron namespace, selected channel/view,
path and committed state. It returns a bound particle, absence or an explicit
unresolved conflict under the selected resolution policy. The resulting
particle is then read through the content service. A mutable path used for an
authorized operation MUST be bound to the checked state/particle so a concurrent
rename or edit cannot silently redirect that operation.

- Renaming or moving a path changes its graph binding while preserving the
  target file's particle. Several names may refer to the same content.
- Editing creates new content and updates the selected binding through an
  attributed patch. Earlier content remains addressable by particle, with
  physical availability governed by retention obligations.
- A directory manifest or `.cyb` container that includes entry names commits
  those names as its own content. Renaming an entry changes that enclosing
  object's particle while preserving an unchanged child's particle.
- Channels select views over patch history and share immutable content.
  Concurrent changes preserve the information needed to represent and resolve
  conflicts; synchronization cannot silently erase a competing name binding.
- Removing a name changes its view. Content reclamation preserves every retained
  namespace revision, channel and other live obligation. Removing one alias
  cannot release content protected by another obligation.

A protected filesystem revision includes its namespace bindings, channel/patch
state, required history and referenced content closure. Restore MUST recover
that view with its bytes. Name updates, selected heads, retention transitions
and request receipts share the conditional publication boundary below.

Private paths, directory membership, channel names and patch metadata follow
the same disclosure policy as private content. A BBG authorization namespace
and an FS path namespace have explicit mappings; a path prefix alone grants
no storage access. Local indexes and caches remain paged BBG-owned projections.

S2 specifies path encoding/normalization, case rules, alias behavior,
rename/edit concurrency and conflict resolution; S7 integrates the FS consumer.
The [acceptance matrix](../roadmap/storage/acceptance.md) qualifies this layer
alongside payload storage. Existing FS blob-store and identity descriptions
must be reconciled with this contract during those work packages.

## local write and publication

The lifecycle is staged parts → verified complete content → atomic publication
→ optional replication. Each stage has durable retry and cancellation semantics.

1. Admit a write under an explicit namespace, authorization and resource budget.
   Stream parts into BBG in bounded transactions; persist resumable progress.
2. Verify identity, length and required content closure. Incomplete or unverified
   parts remain staging data. Sparse verified reads declare their exact coverage.
3. Publish the application head, retention root and idempotency receipt in one
   conditional transaction after the complete content is durably available.
   Large content is prepared across batches; publication refers to a validated,
   protected closure rather than rewriting every part in one transaction.
4. Resolve an ambiguous commit after reopen before dependent effects or retries.
   Success means the selected backend's durability boundary has completed.

Pure caches may retain sparse ranges. A protected application revision MUST
have its declared complete closure. A graph reference can also describe remote
or absent content; its presence alone supplies no local durability claim.

## replication and retention

Foculus compares required and available coverage, requests missing ranges and
resumes through BBG records. The receiver verifies and durably retains the
required closure before issuing a protection receipt. A receipt binds the
provider, namespace, particle/revision, closure, retention obligation and
request identity under the selected authenticated profile.

Policy distinguishes local durability, verified coverage, retention, remote
acknowledgement, present availability and restore freshness. Products expose
these facts separately. Receipts count toward a replica policy only under
explicit failure-domain assumptions; copying into several local directories
does not establish independent replicas.

Retention roots cover selected heads, required history, recovery points,
in-flight publication/replication and active read leases. Reclamation MUST
preserve every live obligation and be safe under concurrent publication,
restart and partial failure. Focus-driven cache eviction operates on releasable
data. History removal requires an explicit retention decision and keeps its
consequences visible to recovery.

## privacy and growth

Private data reaches storage and replication as ciphertext with permitted
metadata. BBG namespaces enforce separation of authorization, enumeration,
deduplication and accounting. Private presence or retention MUST NOT become
a public lookup, aggregate, provider advertisement or cross-namespace dedup
oracle. Public graph projections require a separately reviewed disclosure
contract. Sharing a physical backend grants no publication rights.

Storage interfaces expose leakage such as size, timing and request patterns
for the selected privacy profile to address. Encrypted persistence alone
does not qualify private retrieval or the existing public aggregate design.

File count, library size and revision count have no arbitrary protocol-wide
or library-wide ceiling. Capacity grows with provisioned storage. Streaming,
paged indexes, incremental validation and resumable maintenance keep working
memory bounded independently of collection size. Per-operation budgets and
operator quotas provide backpressure and explicit errors. They MUST NOT become
fixed total-history limits, silent truncation or automatic loss of retained data.

## specialized contracts

[[bbg/specs/content-storage|BBG content storage]] defines local handles,
publication, retention and crash recovery. [Vault composition](vault.md) adds
custody, protected-use state and writer fencing. Other applications use the
same storage interfaces with their own semantics. The [acceptance matrix](../roadmap/storage/acceptance.md)
qualifies the composition across consumers and both durable backend profiles.
