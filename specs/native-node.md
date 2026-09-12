---
tags: cyber, soft3, spec, storage, node
crystal-type: spec
crystal-domain: cyber
status: implementation
---

# native node adapter

Soft3 owns HTTP parsing, label normalization and response formatting. The
Cybergraph native coordinator owns signal ordering, economics, finalization,
request receipts and recovery over the shared BBG database. The cyber product
owns configuration, the home lock and explicit storage import commands.

## acceptance

The adapter MUST return success after the coordinator commits the complete
operation durably. One accepted signal advances one block. Intents participate
in the same durable request transaction and keep their existing height rule.
Their retained count is exposed as `intents` in status; the current BBG root
profile commits its existing dimensions while intents are recovered from the
durable operation history.
Native batches contain at most 64 events and commit atomically; any invalid
event rejects the whole batch. This replaces the old partial batch behavior.

JSON link/pay and native frame ingress MUST share economics and recovery.
An operation rejected before commit MUST preserve the graph, chain position,
balances and exported history. An ambiguous storage failure MUST stop further
acceptance and make health unavailable until the database is reopened and
validated. HTTP storage failures use 503; request conflicts use 409; malformed
or rejected operations use 400; unsupported representations use 422.

Clients MAY supply a request identity in the `Idempotency-Key` header or JSON
`request_id` field. It contains 1–128 visible ASCII bytes. If both are present,
they MUST match. A 64-digit hex identity directly names the 32-byte key
returned in receipts. Other identities are hashed with the domain
`soft3/native-request/v1\0` into a 32-byte key. The coordinator binds that key
to the canonical operation. A matching retry returns its original receipt,
including original height, root and payment balance, after later commits or
restart. Reusing a key for another operation fails with 409. Timestamp is
observability metadata and does not change request identity.

An omitted identity creates a new operation on each submission. Existing cyb
clients remain compatible; callers requiring safe retries must retain a key.

## wire and limits

`POST /v1/frame` accepts complete legacy foculus frames through its strict
decoder. `POST /v2/frame` accepts one complete versioned foculus signal codec
envelope, retaining network and proof fields. Parsing MUST consume the complete
body and reject invalid encodings before acceptance. Native v1 batches have
the same atomic acceptance and request identity contract as JSON submissions.

Headers are limited to 64 KiB and bodies to 8 MiB. The coordinator separately
bounds canonical operations at
4 MiB and enforces its transaction limits. A rejected limit returns 400 and
leaves the node healthy. POST requires one valid
Content-Length. Duplicate lengths, transfer encoding, truncated bodies and
ambiguous request framing fail before state mutation. JSON requests reject
unknown fields and invalid types; link defaults are token `0`, amount `1`
and valence `0`. Valence must fit i8. Label/hex normalization preserves the
existing optional lowercase `0x` prefix and left-padding of 1–64 hex digits.

`GET /log?from=N` retains its byte-offset legacy wire contract. Bounded pages
end at complete frames so existing cyb replication cursors can advance by the
returned byte length. Imported bytes remain an identical prefix. A history
containing signals that v1 cannot represent MUST report the representation
error without silently dropping network or proof data. The complete export is
`GET /v2/history?after=POSITION&limit=N`, with an exclusive operation cursor and
1–64 entries. Its `cyber/native-history/v1` JSON has `entries`, `next` and
`encoding: CGOP-v1-hex`; each entry contains the original `receipt` and hex
canonical `operation` bytes. `next` is the last returned position or null.

Explorer lists return at most 500 blocks. Block data, time, supply, weight and
root come from the committed coordinator state. Imported timestamps are zero
when the source log did not record them.

## home and recovery

The native database is `$home/bbg`. Genesis is strict JSON with the supported
chain, engine, protocol and integer genesis time. The adapter pins canonical
JSON bytes in the database and preserves the user's genesis file. Invalid
existing genesis fails startup. A database whose genesis file is missing also
fails startup.

An existing `$home/log` requires explicit `cyber storage import-legacy` before
normal startup. Import uses the existing validated genesis and complete source
log, preserves both files, and marks the imported source in the database.
Subsequent startup validates that the retained source still matches that
marker. Malformed or rejected legacy events fail import. Incomplete import
destinations remain unavailable. The host bounds the import input at 64 MiB;
larger imports require a separately specified streaming interface.

The old `block_meta` sidecar remains preserved historical input. Its wall-clock
observations do not establish graph state or root validity. Recovery replays
the committed operations and verifies stored roots before readiness.

The product remains an unsigned local chaosnet bridge. Authentication, peer
sync, consensus and graceful shutdown follow their owning product contracts.
