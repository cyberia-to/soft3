---
title: filesystem composition
tags: soft3, fs, cyb, storage, architecture
crystal-type: proposal
crystal-domain: cyber
status: draft
---
# Filesystem composition

Build [[fs]] as the filesystem component of [[cyb]], with a reusable library
in the existing `fs` repository. Memory, Brain, Com, `cy` and neuron programs
use one service. Local files work offline; [[cyber]] can host remote services
and replicas. [[bbg]] stores the data, [[radio]] transports it.

This is the proposed composition. Implementation findings are in the
[source audit](../audit/storage/filesystem-composition-2026-09-27/README.md).

Owner contracts: [[soft3/file/specs|File specification]] and
[[soft3/fs/specs|FS specification]]. The
[[soft3/fs/roadmap/extraction|extraction plan]] maps existing code to its owner
and defines when the replaced implementation must be deleted.

## File, FS and Hemera

| Repository | Owns |
|---|---|
| [file](https://github.com/cyberia-to/file) | File, Particle, Kind and format detection |
| [fs](https://github.com/cyberia-to/fs) | Names, paths, versions, patches and channels |
| [hemera](https://github.com/cyberia-to/hemera) | Hash and structural-commitment algorithms |

`file` supplies the shared representation of an individual file:
`File { particle, data }`. It constructs a `Particle` through Hemera and
identifies the data format through `Kind` and `sniff`. Spark consumes these
types to open content. The same file representation serves documents, images,
models and programs, independently of their organization.

`fs` organizes files and records their changes. It resolves names, presents
directories, selects versions and channels, and applies patches. It uses
File's types and Cybergraph's publication/content interfaces. Large files need
a streaming representation in File; Cybergraph/BBG supply the stored bytes.

## File, particle, name, version

These terms follow the [shared vocabulary](../specs/terms.md#identity-and-naming).

| Term | Meaning |
|---|---|
| File | The thing: data together with its particle |
| Particle | The file's immutable 32-byte identity |
| Name | A mutable label pointing to a particle |
| Path | A route to a name within a selected context |
| Version | A recorded state in a document's history |
| Patch | An attributed change with prerequisites |
| Channel | A named selection of patches |

Names and version relations are cyberlinks about files. Name resolution follows
the existing supersession rule: the latest cyberlink by a neuron under a path.
FS applies it within the selected namespace, channel and state; Inf reads the
same view. Latest follows the established supersession order. Incomparable
concurrent changes remain a conflict until explicitly resolved.

For example, the name `landing.cyb` points to file F through particle P:

- Rename to `descent.cyb`: change the name; F and P stay the same.
- Edit its data: create file G with particle Q and record a new version.
- Add another name for F: both names point to P.
- Read an earlier version: open its file; naming history remains available.

A rename preserves the document's version history. Names sharing a particle
can evolve independently through subsequent edits.

A directory manifest is itself a file. Changing an entry's name changes that
manifest's particle while preserving the named file's particle.

Directories, tags, search results and Brain are views of the same files.
Independent patches should compose; incompatible changes remain visible as
conflicts until a resolution patch is applied. The FS patch algebra must define
and verify those rules before concurrent editing ships.

## Ownership and dependency direction

```svgbob
Cyb: Memory, Brain, Now, Com, cy
Neuron programs, scoped SDK
            |
            v
+-----------------------+     +-----------+     +-----------+
| FS                    |---->| File      |---->| Hemera    |
+-----------+-----------+     +-----------+     +-----------+
            |
            v
+-----------------------+     +-----------+     +-----------+
| Cybergraph            |<--->| Foculus   |---->| Radio     |
+-----------+-----------+     +-----------+     +-----------+
            |
            v
+-----------------------+
| BBG                   |
+-----------------------+
```

Arrows show component use through shared interfaces.

| Component | Owns |
|---|---|
| Cybergraph | Validated publication, history and content access |
| BBG | Durable bytes, indexes, retention and reclamation |
| Foculus | Reconciliation and availability mechanisms |
| Radio | Authorized transport through injected sources/sinks |
| Inf | Queries over the selected filesystem state |

FS decides what a change means and how document conflicts resolve. Foculus
finds and transfers missing authorized history/content through shared ports;
the application and host choose replicas and retention policy. Network finality
uses a separately selected consensus profile. Local saves require local
authority and durability.

Ward supplies grants, Vault performs secret operations, and Mudra supplies
cryptography. FS checks current authority at its service boundary. Vault shares
storage and transfer mechanisms while retaining its own writer and recovery
rules; its sealed state remains accessible through the private application port
for unlock and recovery.

## One service in Cyb

Cyb embeds FS through its shared GraphSession. Memory, `cy fs …`, Nu's graph
commands and neuron file acts use the same operation context: subject, namespace,
channel, selected state and grant. A pending operation keeps its captured
context when navigation changes.

GUI and CLI share one database owner, using local IPC when needed. A headless
session can own the same service. Cyber administers hosted services; Radio's CLI
exposes transport diagnostics. Native OS paths remain an explicit import/export
boundary. Restricted programs receive scoped host operations; interactive Nu
keeps its separately granted OS access.

The service resolves names, reads files, lists and watches views, applies
changes, and reports availability. Its core guarantees are:

- Publish a saved version and its required durable content together; preserve
  request identity across retries. An uncertain commit pauses dependent work
  until recovery determines its outcome.
- Keep private content, names and history within their authorized scope.
  Public graph publication is explicit; encrypted discovery needs a defined
  metadata-leakage policy.
- Distinguish local save, retained history and complete remote protection.
  Unlink changes a view; reclamation respects retained versions and active reads.
- Page history and listings at a pinned state; resume watches from a cursor or
  report a gap. Stream bytes and fetch on demand. Bound individual operations;
  total files and versions have no arbitrary cap. Retained indexes/checkpoints
  make recovery practical as history grows.

## Delivery

Continue the existing
[storage roadmap](https://github.com/cyberia-to/soft3/blob/189c8aa9809aed18417023c9b2f8b324dabd5b5b/roadmap/storage/README.md):

1. Specify Name/path resolution, version history and channel/conflict rules in
   `fs`, aligned with the existing terms and Inf views. File defines its
   streaming representation; Hemera owns the underlying identity/proof algorithms.
2. Move catalog semantics into FS while preserving the shared durable index,
   existing identities and receipts. Integrate with Cyb's GraphSession work.
3. Wire Memory, `cy`, Nu and neuron acts to that service. Import existing data
   with provenance, then retire the parallel Cyb, Foculus and Radio writers.
4. Qualify synchronization, retention, restore and patches/channels; complete
   Radio storage removal after consumer and migration parity.

The owner specs now define names, versions, conservative patch/channel behavior
and migration compatibility. The current catalog provides local serialized
history; implementing the wider contract, complete replication and canonical
identity/range-proof qualification remain delivery work.

Acceptance: import a file, rename, edit, inspect earlier versions, retry after
an interrupted response, restart, replicate and restore names plus content after
device loss. Memory, `cy`, Nu and neuron acts must agree on results, permissions
and durability. Exercise concurrent rename/edit, incomplete replicas, both BBG
backends and Vault's independent recovery rules.
