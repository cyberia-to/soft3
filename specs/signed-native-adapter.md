---
title: Authenticated local native HTTP profile
status: accepted
---
# Authenticated native adapter

This profile adds authenticated submission and exact retry to the local native
coordinator. Its receipts describe endpoint acceptance. Consensus, authenticated
peer membership and proof of network finality remain separate contracts.

A network is pinned by H(the exact canonical genesis bytes accepted by NativeNode).
The existing genesis stays unchanged on upgrade. Activating the profile requires
an explicit local host operation with the database writer lock available; it
promotes a persistent BBG reader-generation guard. A fresh compatible reader
recognizes the guard and requires authentication for all HTTP mutation. Earlier
executables reject opening the upgraded store. Promotion is monotonic and is
retained across later neuron imports; an import cannot downgrade authentication.

Pre-BBG executables can ignore all database markers and silently start a fresh
in-memory graph. Activation therefore also retires their mandatory flat
`genesis.json` reader path through cybergraph LegacyFile. The original JSON bytes
remain in `genesis.json/source`, with a verified manifest; its canonical parsed
genesis and network hash do not change. Such old executables fail their genesis
read. New readers require the original authenticated database when this path is
retired. Promotion precedes retirement; interruption resumes retirement before
the new HTTP listener starts. Old uncooperative writers must be stopped before
upgrading, as with all LegacyFile retirement. No profile claims to revoke an
already running old process's file descriptors or in-memory state.

`GET /capabilities` returns schema, profile, network ID, supported submission kinds
and the request/envelope bounds. A client must match these against its configured
network/profile before sending. Network names and endpoint ordering cannot choose
an action's destination. HTTP/TLS endpoint trust does not establish consensus.

## Requests and journal

`POST /v3/action` accepts the canonical neuron-model SignedAction envelope specified
in neuron/specs/action-envelope.md. The adapter validates size/codec/context,
requires a native subject/network equal to the pinned network, and verifies mudra
NSIG1 over H(the common action statement bytes). It then validates the exact
operation payload. Supported kinds:

| Kind | Payload | Native publication |
|---|---|---|
| native/signal | Complete canonical foculus signal codec bytes | The same single Signal, preserving author, network, step, prev and proof |
| native/pay | Canonical JSON `{to: [32 bytes], amount: u64}` with positive amount | Existing Native Operation::Pay semantics for the verified subject |

The pay envelope authorizes exact recipient/amount/network; the server assigns
its existing native chain position. Clients cannot invent a remote Signal step
from their unrelated local private history. The signed request remains durable
authorization evidence for that native operation. This preserves the original
Pay encoding and economics without relabeling old unsigned activity as signed.

A robot that concurrently creates local signals owns that subject's sequence
locally: its payment is a complete `native/signal` containing delta_pi. This
keeps payment, linking and queued work in one ordered author chain. The separate
`native/pay` convenience kind serves clients that delegate sequence assignment
to this endpoint. Mixing both sequencing authorities for one subject requires
explicit synchronization; the adapter never repairs such a fork by renumbering.

A derived request key commits to the profile domain, network, native subject and
caller's request ID. The complete verified envelope is pinned in ApplicationGraph
before native acceptance, using an isolated request journal namespace. Its same
request key with different context/payload/evidence is a conflict. NativeNode's
existing durable request receipts provide at-most-once local economic mutation.
For a newly accepted signed action the owner also requires sufficient funds for
every payment leg. Legacy replay's skipped insufficient leg semantics stay
unchanged, but cannot produce a successful signed payment receipt. Existing
request resolution precedes this check so a retry after spending still returns
the original receipt.
Journal-before-native failure resumes the same operation; native-before-response
failure resolves its original receipt. No source graph or sidecar is another
balance/history authority.

Receipts carry the original action request, derived native request, subject,
network, envelope commitment and the native receipt. `GET /v3/receipt/...` resolves
that exact tuple. A client cannot acknowledge another author/network/request or a
partial subset of links. Unknown HTTP results remain retained until resolution;
identical retransmission is allowed only by this declared idempotency contract
and a current local grant. It cannot change context or become a new user action.

When authentication is active, `/v1/link`, `/v1/pay`, `/v1/frame` and `/v2/frame`
reject unsigned mutation. Read/history routes remain available. The old unsigned
chaosnet profile remains explicit for legacy inspection/tests and advertises no
authenticated capability. No authenticated client silently falls back to it.

## Lineage and relay

Native graph chains currently index by NeuronId. Multiple compatible network
bindings do not create separate chains for one native subject. A destination
missing required predecessors rejects a relayed Signal. The relay preserves the
original step/prev and never fills a gap by broadcasting private history or
rewriting authorship. Independent native histories use distinct key identities;
foreign network identities retain their domain/address-specific semantics.

A relay outbox records one whole Signal and its explicit binding/network/profile,
exact request/envelope and disposition. Durable cursors are scoped by network and
author and advance only after a correlated receipt for that entire signal.
Legacy `relayed` marks are retained provenance, not proof that all links or a
signature were accepted. Their import must not infer missing remote receipts.

Balance observations carry subject, network and query generation. A late result
can update that observation but cannot replace the currently selected account's
view. The UI distinguishes endpoint receipts from verified network finality.

## Exact journal keys and receipt lookup

Hashes use the unchanged hemera H(bytes), with serde compact JSON byte arrays.
The native request is H(`["neuron/native-request/1", network, subject, request]`).
Its journal namespace is H(`["neuron/native-journal/1", native_request]`).
The single index-0 head commits to the complete envelope Blob. The receipt path
is `/v3/receipt/{subject-64-hex}/{request-64-hex}`. The response schema is
`neuron/native-receipt/1`, with profile, network, subject, request, native_request,
envelope, state and receipt. State `prepared` has a null receipt; `accepted` holds
the existing native receipt (including its exact request_id). It is not a proof
that another replica or consensus has finalized the operation.

`GET /v3/history?after=POSITION&limit=N` exposes the owner's complete native
history view (default 4, maximum 16 entries, maximum response 32 MiB). It retains
receipt, original CGOP operation hex and full canonical signal hex, including
network/proof. The page declares schema `neuron/native-history/1`, profile,
network, requested after, entries and last returned position (next, null for an
empty page). A missing after begins at native position 1. The complete operation is
provenance; host LocalCredit events must never be executed by a peer mirror.

Cyb mirrors each native source entry into its existing ApplicationGraph with
network, original receipt, operation Blob, complete signal Blobs and previous
observation. Its cursor is the last fully projected source position, scoped by
network ID. It validates a whole bounded page before writes; each entry applies
only complete Signals, then stores provenance and cursor. A crash between these
steps retries exact already-held signals, preserving the cursor until all are
stored. Duplicate/equivocation checks remain in GraphSession. Old name/byte-offset
`synced` files are retained historical provenance and cannot seed these cursors.
These are endpoint observations; verified consensus remains a separate view.
