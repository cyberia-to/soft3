# File storage: what actually retains the bytes

Source audit, 2026-09-24. Local repositories and committed test inputs are pinned
in [sources.json](sources.json). This is an implementation audit, not a release
or cryptographic qualification. No live content, credentials or Vault data were
opened. Runtime code and specifications were left unchanged.
The local soft3 checkout is `9784d92`, while the report branch starts at
origin/main `434244e`; both route tables were inspected. Current Cyb has local UI
edits, but its audited JSONL module is identical to committed `6c205592`.
Findings describe these sources, not an assertion about every published binary.

## Verdict

The stack has usable storage building blocks, but the inspected Cyber/Cyb paths
do not compose them into one reliable file service. Graph persistence preserves
identities and relations; it does not automatically retain the bytes identified
by every particle.

There are three distinct implementations:

| Path | Bytes actually retained | Physical location | Current boundary |
|---|---|---|---|
| Cyb particle/file resolver | Plain UTF-8 text and a declared particle | `~/cyb/particles.jsonl`; whole-file RAM index | Product path, best-effort writes, no verified network fallback |
| Cybergraph ApplicationGraph → BBG | Verified Blob/Data records, application history, receipts | Shared BBG database; default SSD/Fjall directory | Used by Neuron/Vault adapters; TextArchive exists but Cyb's resolver does not call it |
| Radio `iroh-blobs::FsStore` | Arbitrary binary data, BAO verification data, tags | `<root>/blobs.db` (redb), `data/`, `temp/` | Library implementation with persistence tests; no generic file-service wiring found in the inspected Cyber/Cyb entry points |

`file::File` is an in-memory `(particle, Vec<u8>)` value. It supplies identity and
format detection, with no disk store, replication or ownership policy.

```text
Cyb remember(text) ──> particles.jsonl ──> lookup ──> File::bind ──> Spark

Neuron / Vault ──> ApplicationGraph ──> BBG Database ──> Fjall
TextArchive ──────> ApplicationGraph           (Cyb integration missing)

Radio FsStore ──> redb + owned binary files + BAO metadata
                 (product integration / identity binding missing)

Cyber graph API ──> native history + graph records in BBG
                   (generic put/get/fetch/retain file API missing)
```

## Findings, in implementation order

### 1. Cyb can retain a link while losing or substituting its text — high

The active resolver hashes text, appends a hand-built JSON line, then loads all
lines into a `HashMap`. Duplicate identities keep the last record. Directory,
open and write errors are ignored; no file/directory sync barrier or
cross-process transaction joins this operation to graph publication. A caller
can continue publishing cyberlinks after a failed content write.

Reading trusts the particle from the line. `Now::load_file` then calls
`File::bind`, whose explicit contract requires the caller to have verified the
bytes. The resolver does not do that. The synthetic probe substituted text under
an existing particle and the reader returned it as that particle.

The same probe demonstrated a swallowed write-to-directory error, a panic on
malformed UTF-8 in the hex field, and incorrect decoding of a standard JSON tab
escape. Corrupt/skipped lines and read failures can appear as absent content.
The writer also emits raw control characters other than newline. Whole-file
loading and duplicate append growth make this unsuitable as a general binary
store. Cache invalidation uses file length; a same-length external rewrite can
remain invisible until another invalidation.

Sources: [Cyb content resolver](https://github.com/cyberia-to/cyb/blob/6c20559206d90f0474379e48706ba00db9630a37/shell/src/worlds/content.rs#L30), [committed File resolver](https://github.com/cyberia-to/cyb/blob/6c20559206d90f0474379e48706ba00db9630a37/shell/src/now.rs#L72), [File::bind](https://github.com/cyberia-to/file/blob/fc666a5880ff874adae655150414b3a45832baf0/src/lib.rs#L103). The content module is unchanged from
the captured Cyb commit; the owner tree has unrelated UI edits, recorded in the
manifest. Both committed and local `Now::load_file` use the same binding path.

### 2. Particle and Radio blob identities do not agree — high

| Consumer | Computation |
|---|---|
| `file::Particle`, Cybergraph `Codec::Blob`, BBG `fetch_content` | `hemera::hash(bytes)` — sponge digest |
| Radio `Hash::new` | `PreOrderMemOutboard::create(bytes, IROH_BLOCK_SIZE).root` — BAO tree commitment |
| Hemera semantic tree API | Separate CDC tree construction |
| Polynomial-particle design pages | Hemera over a Lens commitment and domain |

The probe confirmed different File/Radio identities for identical deterministic
inputs of 0, 5, 4096, 4097 and 65537 bytes. These are two constructions, not a
hex/base32 presentation difference. `Hash::from_bytes(particle)` merely relabels
the digest; it cannot supply the missing proof binding. BBG's network-content
check expects the sponge digest too.

Before wiring Radio, select the canonical file identity and its relationship to
transport commitments. Retaining the current identity needs an authenticated
binding to the BAO root; adopting a different identity needs explicit treatment
of existing references. This audit does not choose or migrate that contract.

Sources: [Particle::hash](https://github.com/cyberia-to/file/blob/fc666a5880ff874adae655150414b3a45832baf0/src/lib.rs#L33), [Content codecs and verified read](https://github.com/cyberia-to/cybergraph/blob/0eae7ae9fa116ae51fc4ec6d2cd0941c12e86212/src/content.rs#L29), [Radio tree identity](https://github.com/cyberia-to/radio/blob/db1d62e2cd1e4b2f309fa19bcc158bd29b44753d/iroh-blobs/src/hash.rs#L13), [Hemera sponge](https://github.com/cyberia-to/hemera/blob/2bb9bb623cd97fb685acd7b9882518a2901723af/rs/src/lib.rs#L88), [tree and CDC constructors](https://github.com/cyberia-to/hemera/blob/2bb9bb623cd97fb685acd7b9882518a2901723af/rs/src/tree.rs#L126), [verified content fetch](https://github.com/cyberia-to/bbg/blob/c75b68574dd8291311a9a6e9a61fe96b1f5781e0/rs/src/storage/tiered.rs#L101).

### 3. A durable application store exists; its scope is narrower than arbitrary files

`ApplicationGraph` validates identities, referenced content and expected heads.
BBG atomically writes immutable content, history, heads, claims and request
receipts. The default `ApplicationStore::open` selects Fjall; `from_database`
shares an existing owner's backend, process lock and failure state. Fjall uses
`PersistMode::SyncAll`; ambiguous commit errors stop dependent operations.
Reads reconstruct the codec and rehash the stored bytes. Content lives in the
application content table, not inside every graph-particle record.
General Blob data is stored as supplied, with a codec prefix. Encryption belongs
to its caller: Vault encrypts before this boundary, while Cyb text is plaintext.

Neuron stores execution records/artifacts through this path. Vault stores
encrypted packets through it. Local soft3's signed-action journal also retains
exact envelope bytes this way; that does not ingest the arbitrary files named
inside an action. Local soft3 and origin/main differ, and both were inspected.

TextArchive adds verified text, observation history and strict legacy JSONL
import/retirement. It is not wired into the inspected Cyb content resolver. Its
current implementation also reconstructs the selected text projection during
`remember`, so directly substituting it does not settle large-library scaling.

| Existing budget | Scope, from code |
|---|---|
| 16 MiB | Cybergraph Content payload limit; application publication separately bounds the sum of stored rows including codec bytes to 16 MiB |
| 8 MiB | TextArchive individual UTF-8 text |
| 64 MiB | TextArchive total unique-text projection |
| 1,000,000 | TextArchive observation count bound |

These are inspected constants, not benchmark results or proposed product
limits. The first is an operation budget; the TextArchive projection/history
bounds are real archive-capacity constraints. ApplicationGraph is not a
streaming large-file/chunk store. Vault's catalog has no equivalent configured
total-entry/revision ceiling; its packet limits apply per operation.

Sources: [ApplicationGraph](https://github.com/cyberia-to/cybergraph/blob/0eae7ae9fa116ae51fc4ec6d2cd0941c12e86212/src/application.rs#L62), [ApplicationStore](https://github.com/cyberia-to/bbg/blob/c75b68574dd8291311a9a6e9a61fe96b1f5781e0/rs/src/storage/application.rs#L63), [Fjall commit barrier](https://github.com/cyberia-to/bbg/blob/c75b68574dd8291311a9a6e9a61fe96b1f5781e0/rs/src/storage/database/backend_fjall.rs#L128), [TextArchive](https://github.com/cyberia-to/cybergraph/blob/0eae7ae9fa116ae51fc4ec6d2cd0941c12e86212/src/text_archive.rs#L1), [Neuron graph adapter](https://github.com/cyberia-to/neuron/blob/c87541120caae4fb5d5bb401653d1cbcbeffeb7d/node/src/lib.rs#L24), [Vault graph adapter](https://github.com/cyberia-to/vault/blob/f3fe2e4ae3168a3de8441285430f3552218eb7fb/src/graph.rs#L1),
[local signed-envelope retention](https://github.com/cyberia-to/soft3/blob/9784d92eb507207f96faa886fcc44886707e0db2/crate/src/node/signed.rs#L157). Relevant contracts: [application storage contract](https://github.com/cyberia-to/bbg/blob/c75b68574dd8291311a9a6e9a61fe96b1f5781e0/specs/application-storage.md) and [Vault local profile](https://github.com/cyberia-to/vault/blob/f3fe2e4ae3168a3de8441285430f3552218eb7fb/specs/local-profile.md).

### 4. Radio has a real disk engine, but verified partial import needs repair — high

FsStore keeps small complete payloads and small outboards inline in redb; the
default threshold for each is 16 KiB. Larger owned content uses
`data/<hash>.data`, with `.obao4`, `.sizes4` and `.bitfield` sidecars as needed.
`temp/` must share a device with `data/` for atomic moves. `TryReference` import
can retain an external path instead of owning a copy, so moving/deleting that
original is an availability dependency. `Copy` imports into owned storage.

Its redb is Radio's own metadata/payload backend, independent of BBG's SSD/HDD
selector. Selecting BBG/Fjall does not change Radio's redb usage.

The selected full-byte import → clean shutdown → reopen → verified export test
passes. The existing two-stage BAO persistence test fails at empty input during
import (`failed to import size=0: inner error: Error::Io`), before it can establish
resume behavior for the remaining sizes. A separate nonempty range test panics
at `store/util.rs:227` while persisting the partial-file bitfield: it copies a
32-byte checksum into a 64-byte slice. `partial_mem_storage.rs:41–42` separately
retains the same wrong child-hash width in its outboard write path (static finding). The task failure leaves the test pending; a bounded rerun captures
the backtrace. The one-byte BAO import test passes. These results isolate actual
partial-transfer defects rather than treating the whole library as absent. See
the exact test results below.

The engine explicitly documents that dropping without `shutdown` can lose recent
writes. Metadata commands are batched, and the code replies from handlers before
the enclosing transaction commit. It therefore needs an explicit durable receipt
boundary before a product can call a file safely stored. Successful orderly
reopen is not power-cut qualification.

Sources: [FsStore lifecycle](https://github.com/cyberia-to/radio/blob/db1d62e2cd1e4b2f309fa19bcc158bd29b44753d/iroh-blobs/src/store/fs.rs#L39), [FsStore layout and options](https://github.com/cyberia-to/radio/blob/db1d62e2cd1e4b2f309fa19bcc158bd29b44753d/iroh-blobs/src/store/fs/options.rs#L10), [copy versus external reference](https://github.com/cyberia-to/radio/blob/db1d62e2cd1e4b2f309fa19bcc158bd29b44753d/iroh-blobs/src/store/fs/import.rs#L440), [batched metadata commit](https://github.com/cyberia-to/radio/blob/db1d62e2cd1e4b2f309fa19bcc158bd29b44753d/iroh-blobs/src/store/fs/meta.rs#L786).
Partial-file failure: [checksum writer](https://github.com/cyberia-to/radio/blob/db1d62e2cd1e4b2f309fa19bcc158bd29b44753d/iroh-blobs/src/store/util.rs#L216), [outboard buffer](https://github.com/cyberia-to/radio/blob/db1d62e2cd1e4b2f309fa19bcc158bd29b44753d/iroh-blobs/src/store/util/partial_mem_storage.rs#L32).

### 5. Graph membership, file retention and remote replicas are disconnected — high

BBG's `FileRecord` contains `available` and `chunk_count`. A proof over that
record authenticates the recorded fields, not a successful fetch or the physical
existence of independent copies. Application content also has a separate local
commitment boundary from BBG polynomial state.

`NetworkStore` and a hash-checking `TieredStore::fetch_content` exist. Searches
of BBG/Cybergraph/Cyber/Cyb and the workspace found only the test implementation
of this trait; no production Radio adapter or node bootstrap wiring was found.
The two inspected soft3 route tables expose graph/history/actions, not generic
file upload/download/retention. Cyb's local miss ends without fetching bytes.
The source comment claiming IPFS and two Cyb/Soma JSONL writers is stale: no
IPFS fetch path was found in these runtime trees, and the current Soma kernel
explicitly has no independent persistence writer.

Radio GC, when enabled, marks persistent tags, temporary tags and HashSeq
references; a callback can add protected hashes. Default FsStore options disable
periodic GC. There is no discovered bridge from Cybergraph/Vault retention
obligations to these roots. Application history currently retains immutable
content without a general history/content GC service. Thus neither graph links
nor focus automatically mean files are pinned or replicated.

Vault does have ciphertext copy-and-readback to configured local replica stores.
Its implementation and spec call this local evidence: authenticated remote
receipts and physical failure-domain independence are separate requirements.
Application archive/transfer APIs support sealed local migration; they are not
an automatic network backup/repair service. Restoring the graph alone cannot
reconstruct missing file bytes.

Sources: [availability record](https://github.com/cyberia-to/bbg/blob/c75b68574dd8291311a9a6e9a61fe96b1f5781e0/rs/src/types.rs#L72), [network interface](https://github.com/cyberia-to/bbg/blob/c75b68574dd8291311a9a6e9a61fe96b1f5781e0/rs/src/storage/network.rs#L14), [verified content fetch](https://github.com/cyberia-to/bbg/blob/c75b68574dd8291311a9a6e9a61fe96b1f5781e0/rs/src/storage/tiered.rs#L101), [local route table](https://github.com/cyberia-to/soft3/blob/9784d92eb507207f96faa886fcc44886707e0db2/crate/src/node/routes.rs#L23), [origin/main route table](https://github.com/cyberia-to/soft3/blob/434244e9f3ffd5850188dddb5ac4c5eceb6aa49c/crate/src/node/routes.rs#L23), [GC roots](https://github.com/cyberia-to/radio/blob/db1d62e2cd1e4b2f309fa19bcc158bd29b44753d/iroh-blobs/src/store/gc.rs#L35),
[local replica copying](https://github.com/cyberia-to/vault/blob/f3fe2e4ae3168a3de8441285430f3552218eb7fb/src/replication.rs#L1), [Vault synchronization contract](https://github.com/cyberia-to/vault/blob/f3fe2e4ae3168a3de8441285430f3552218eb7fb/specs/synchronization.md), [sealed archive transfer](https://github.com/cyberia-to/cybergraph/blob/0eae7ae9fa116ae51fc4ec6d2cd0941c12e86212/src/application/transfer.rs#L1), [Soma kernel boundary](https://github.com/cyberia-to/soma/blob/b0d5d4203af2a1ba40f1f209ee5089f54e259a7b/kernel/src/lib.rs#L15).

## Where the specs diverge

| Document | Statement | Inspected implementation |
|---|---|---|
| Cyb particle reference | SQLite → Radio store → P2P fetch → hash verification | JSONL-only text resolver; no such fallback chain |
| Cyber file / particle pages; BBG storage design | Sponge identity in one page, Lens-based identity/content polynomials in others | Sponge for File/Blob; BAO tree in Radio; no composed polynomial file service |
| Cybergraph particle spec / Radio blob docs | 64-byte identity | Current relevant Rust types use 32 bytes |
| BBG storage and FS sync designs | Focus-weighted replication, availability proofs, erasure-coded device sync | Architectural targets; not established by the inspected product path |

References: [Cyb particle reference](https://github.com/cyberia-to/cyb/blob/6c20559206d90f0474379e48706ba00db9630a37/reference/particle.md), [file page](https://github.com/cyberia-to/cyber/blob/d7eeff42db1a27cb8ceabcc82c459448a35a7267/file.md), [particle page](https://github.com/cyberia-to/cyber/blob/d7eeff42db1a27cb8ceabcc82c459448a35a7267/particle.md), [BBG storage design](https://github.com/cyberia-to/bbg/blob/c75b68574dd8291311a9a6e9a61fe96b1f5781e0/specs/storage.md), [Cybergraph particle spec](https://github.com/cyberia-to/cybergraph/blob/0eae7ae9fa116ae51fc4ec6d2cd0941c12e86212/specs/particle.md),
[Radio blob document](https://github.com/cyberia-to/radio/blob/db1d62e2cd1e4b2f309fa19bcc158bd29b44753d/docs/blob.md), [FS synchronization target](https://github.com/cyberia-to/fs/blob/75d438014eded7b71ad7d2411bcc36205d30e94d/sync.md). The FS sync page already explicitly marks its multi-layer
service as a target. `fs/` currently contains design documents rather than a
working general file-store implementation.

## Recommended next work

1. Resolve file identity versus BAO/Lens commitment binding; publish cross-crate
   vectors before moving persistent identities.
2. Provide a node file-service contract: verified put/get, durable retention,
   range reads, explicit cache/owned/external-reference status, and recoverable
   content publication. Cybergraph owns application semantics; BBG owns local
   persistence; Radio transports and verifies byte ranges. Reuse these components.
3. Route Cyb through that service, import JSONL with validation, and propagate
   persistence failures. Use TextArchive for its defined text role while
   addressing its capacity/replay constraints; large binary data needs its own
   chunk/manifest integration. Define content-before-reference and orphan recovery.
4. Repair/qualify Radio partial import and connect graph/application retention
   roots to blob GC. Define who keeps bytes, for how long, and when space can be
   reclaimed without violating an acknowledged obligation.
5. Add authenticated replication receipts, loss repair and self-contained restore.
   Accept against a fresh process/device: add binary → commit → restart → fetch
   from a replica → verify exact original bytes; reject corrupt content, disk
   failure and incomplete recovery explicitly.

This requires composition and a clear ownership contract more than another
database. Product success must distinguish locally durable, replicated,
verified, recoverable and merely cached content.

## Reproduction and evidence

| Check | Result |
|---|---|
| ApplicationGraph atomicity / corruption / shared owner | Pass: 6 tests |
| TextArchive identity / reopen / legacy import | Pass: 3 tests |
| TextArchive boundaries | Pass: 3 debug cases and the separate optimized capacity case |
| Radio full-byte store → shutdown → reopen → export | Pass |
| Radio one-byte BAO import | Pass |
| Radio two-stage BAO persistence | Fail on empty input |
| Radio nonempty partial range | Panic in checksum persistence; test remained pending |
| File/Radio identity and Cyb resolver probe | Negative findings reproduced |

Counts refer to the commands and revisions in [checks.json](checks.json), with
full logs attached. All 13 selected Cybergraph cases passed across the two runs.
The debug capacity case was restarted under the optimized profile; the debug
interruption is retained as evidence, not counted as a pass.

See [checks.json](checks.json) for exact commands, revisions, exit codes and log
paths. [reproduce.py](reproduce.py) builds the focused probe in a fresh directory;
its only adaptation of the Cyb module changes `HOME` lookup to an audit-only
fixture variable. It never touches the real content store. Expected negative
observations are assertions, not suppressed failures. [probe.rs](probe.rs) and
[probe.lock](probe.lock) retain the probe and resolved dependencies.

All tests use committed detached component worktrees listed in `sources.json`.
Warnings from vendored Fjall and cyber-bao remain in the logs. No full Cyb UI
build, hostile network experiment, power-cut experiment, cryptographic proof,
throughput benchmark or loss-of-device qualification is claimed.
