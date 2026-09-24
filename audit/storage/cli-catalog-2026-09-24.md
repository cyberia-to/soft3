---
title: BBG-backed CLI files and retained names
tags: soft3, audit, storage, radio, cybergraph
---
# BBG-backed CLI files and retained names

The actual Radio CLI now shares the Cybergraph/BBG owner for file operations and
local name/history commands. Its production dependency closure no longer includes
old blob/docs stores. The old packages themselves remain in Radio; the
[physical removal milestone](../../roadmap/storage/radio-removal.md) is open.

| implementation | source and evidence |
|---|---|
| Radio file CLI and descriptor lookup | [[radio/audit/storage/cli-2026-09-24/README|Commands, pinned sources and logs]] |
| Cybergraph conditional names and retained revisions | [[cybergraph/audit/storage/catalog-2026-09-24/README|Catalog checks and transfer regression]] |
| Remaining stores and consumers | [[radio/audit/storage/extraction-2026-09-24|Extraction inventory]] |

The receipts cover separate CLI processes, same-owner reopen, cross-profile file
transfer, authorization, historical paths, rename/edit competition and request
replay. The catalog growth test retains 4,100 revisions per backend profile with
`cargo test --release --features legacy-redb-migration --test catalog --locked
--offline` at Cybergraph `56680f589727e52bb5803e470b0fb206f0fe550b`; this crosses a
page boundary without replacing history with a whole-catalog snapshot.

This advances S2/S5/S7 and provides partial A4/A8/A11/A12 evidence. Full rows stay
open: signed metadata sync, aliases/channels, content-aware legacy import and
restore, retention release/GC, S1 authenticated ranges and Cyber/Cyb product
assembly remain outstanding. Both software profiles ran on the same local SSD.
Local loopback and process-exit checks do not qualify physical power loss,
external-network reachability or replica failure domains.
