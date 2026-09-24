---
title: storage acceptance matrix
tags: soft3, storage, conformance, roadmap
status: planned
---
# storage acceptance matrix

These are required gates to implement, with no implied passing result.
The [project](README.md) tracks owners and dependencies; the
[baseline audit](../../audit/file-storage-2026-09-24/README.md) records existing
behavior separately. Each future receipt gives the command, complete source
revisions, backend configuration, inputs and observed outcome.

## invariants

| ID | exercise | required result |
|---|---|---|
| A1 | Same inputs through File, Cybergraph, BBG and Radio; empty/binary/boundary files; adversarial ranges and proofs | Canonical particles agree; altered, truncated or misplaced bytes fail; verified coverage is exact |
| A2 | Upload and read files larger than a backend value or transaction; slow/cancelled readers and writers | Bounded memory and backpressure, resumable progress, exact output; no size-derived whole-file allocation |
| A3 | Process crash at each write/seal/publish/ack boundary; backend I/O failure, disk full and ambiguous commit | Acknowledged durable heads retain complete closure; partial work stays resumable/unpublished; uncertainty freezes dependent operations |
| A4 | Restart the existing Radio during transfer and after acknowledgement; offline reopen with Radio stopped | BBG retains content and progress; transfer resumes without a Radio database or blob directory |
| A5 | Concurrent publish, retain, release, read and interrupted GC | Every live obligation survives; released unreachable data is reclaimable; no stale readiness handle can publish reclaimed content |
| A6 | Replication over actual transport to distinct configured stores; dropped/duplicated replies, partitions, loss of writer and a replica | Idempotent protected revisions, complete surviving copy under the policy; count only authenticated durable acknowledgements and disclose failure-domain assumptions |
| A7 | Restore stale/omitted history, conflicting writers and expired permissions; spoof remote receipts | Report incomplete/unknown/stale status; preserve writer fencing, protected-use state and authorization; transport receipt alone cannot mark protected |
| A8 | Increase file count, file size and revision depth independently; page/restart scans and cross-device migration | No hard total collection cap or full-history allocation; small operations avoid total-history rewrites; interruptions preserve acknowledged content |
| A9 | Private Vault/application data beside public files in one owner; unauthorized enumeration, dedup probes, logs and public projections | Namespace authorization holds; private storage facts do not enter public indexes/advertisements; document and review remaining size/timing/access-pattern leakage |
| A10 | Import legacy JSONL, application blobs and Radio stores on copies; restart imports, inject mismatched identities/missing bytes | Validate source content, preserve provenance and recoverable mappings, report incomplete inputs; no silent relabeling or deletion; one authoritative writer after cutover |
| A11 | All consumer classes below through SDK/CLI/node boundaries | Same storage/retention semantics and truthful status; no alternate product database or hidden fallback writer |

Run every applicable local gate on both Fjall/SSD and redb/HDD profiles. Include
cross-profile replication and migration. Separate simulated/process-failure
results from physical power-loss qualification; clean shutdown/reopen alone
cannot qualify crash durability.

## consumer coverage

| consumer | fixture and required workflow | additional contract |
|---|---|---|
| Vault | Synthetic typed secrets, encrypted revisions and protected-use records; offline changes, replica repair and device-loss restore | [Vault composition](../../specs/vault.md) and [[vault/specs/conformance]] |
| Neuron / Cybergraph | Application history and native action receipts sharing the owner with content; conditional publication and retry | [[cybergraph/specs/applications]] and [native node](../../specs/native-node.md) |
| Cyb | Public text, private documents and arbitrary binary attachments; streaming fetch, offline reopen and explicit retention | File identity and storage contracts |
| Programs / model artifacts | Large immutable binary content, sparse authenticated reads and interrupted download | Content coverage and active-read retention |
| Structured files | `.cyb` plus an independently defined structured consumer; semantic chunks, extraction and exact-byte export where promised | S1 canonicalization and adapter vectors |
| Graph history / archives | Many revisions, paged traversal, retention change and SSD-to-HDD movement | BBG per-owner atomicity and explicit archive policy |

Vault is the first full delivery slice. Qualification covers all rows before
claiming a general storage system. Use synthetic fixtures, never live custody
records, for adversarial tests and published evidence.

## measurements and completion

Collect peak working memory, persistent bytes and write amplification, startup
and recovery work, time to first verified range, complete transfer time, proof
cost, GC progress and small-operation cost as each scale axis grows. Separate
cryptography, backend and network contributions. Specify load, hardware,
concurrency and cache state; set explicit product latency/resource budgets
before a performance qualification run.

Retention and total capacity are independent: budgets may slow or reject new
work visibly but cannot silently cap a catalog, truncate history or release
protected data. Every test receipt links its S package and A rows. Soft3's
release validation references these receipts for the exact pinned component
revisions; unimplemented or failing rows remain visible.
