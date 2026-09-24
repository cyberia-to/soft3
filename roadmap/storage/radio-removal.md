---
title: remove Radio-owned storage
tags: soft3, roadmap, storage, radio
status: implementing
---
# remove Radio-owned storage

Completion means the Radio repository and supported runtime dependency closure
contain no independently owned file or metadata store. Every supported file
workflow uses the shared BBG owner through its authorized capabilities.
The [storage project](README.md) owns S1–S9; this milestone closes the extraction
part of S5/S7. [[radio/audit/storage/extraction-2026-09-24|The inventory]] records
the legacy surface and its actual consumers.
The [CLI/catalog receipt](../../audit/storage/cli-catalog-2026-09-24.md) links the
implemented local cutover and names/history checks. The final milestone remains
open while any later step lacks its required evidence.

| step | status | exit evidence |
|---|---|---|
| Replace the CLI's fresh MemStore instances | implementing | Separate add/list/serve/get/export processes share durable BBG data; both backend profiles, empty/binary files, access denial, retry and offline reopen |
| Provide retained names and revisions | implementing | [[cybergraph/specs/catalog|Catalog]] supplies conditional create/edit/rename/remove, immutable historical views and bounded pages; rename preserves payload identity and stale operations conflict |
| Replace scoped metadata synchronization | planned | Authorized sharing, exact/prefix queries, policy, reconnect and concurrent changes survive through FS/Cybergraph/Foculus; secrets follow Vault custody |
| Import old stores | planned | Read-only legacy import preserves surviving records, names, deletion markers, exact bytes and old-to-new identity provenance; interruption/retry works and incomplete inputs remain explicit |
| Close retention and transfer parity | planned | Tags/aliases, temporary protection, release/read leases/GC, multi-provider scheduling and required collections have qualified successors |
| Remove old implementations and dependencies | planned | Delete local blobs/docs stores and engines, replace Willow payload/storage coupling, adapt or retire dormant FFI surfaces; port useful integration scenarios and verify the actual dependency closure |
| Qualify node/product cutover | planned | Cyber/Cyb assembly exercises the shared path; A4, A10, A11 and applicable A12 pass for pinned sources |

The final removal commit includes a source/dependency absence check and the
passing behavior/migration receipts. Merely excluding packages from workspace
builds, moving their database constructors or leaving registry storage engines
in the runtime closure does not meet the exit condition. QUIC, relay, discovery,
gossip and endpoint authentication retain their transport responsibilities.

File naming and content revision are distinct. Renaming changes the contextual
binding and preserves its target particle; editing publishes new bytes and a new
revision. Historical names and payloads remain recoverable under retention.
Local conditional catalog history provides the first foundation; signed remote
patches, aliases, channels and merges must meet the wider FS contract before
claiming full filesystem parity.

Legacy Docs keeps surviving latest records and prefix deletion semantics; it
does not retain all past versions or provide atomic rename. Import preserves
what exists and records unavailable history explicitly. Identity conversion
verifies both constructions; a BAO root is never silently relabeled as a Blob
particle. S1 remains the gate for canonical structural identity and authenticated
ranges.
