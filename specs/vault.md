---
title: vault composition and first storage use case
tags: soft3, vault, storage, synchronization, recovery, spec
status: draft
version: 0.1
---
# Vault composition

[Vault](https://github.com/cyberia-to/vault) is a fundamental independent stack
component. Its [product](https://github.com/cyberia-to/vault/blob/main/README.md)
and [component specifications](https://github.com/cyberia-to/vault/blob/main/specs/README.md)
have one home in that repository. Soft3 owns the composition invariants here.
This is a specification contract, not a claim of implemented private sync.
The [storage architecture](storage.md) owns the common content, persistence and
transport boundaries. The [storage project](../roadmap/storage/README.md)
qualifies those interfaces across applications, with Vault as the first full
recovery slice.

## First end-to-end use case

The stack MUST safely retain, privately synchronize and recover Vault records
after loss of the primary device. Imported passwords, external authenticators,
tokens and their use state require durable records; a neuron seed cannot
regenerate them. Retention and recovery are primary product requirements.

The initial profile composes one active custody writer, a durable local store
and two encrypted replicas in separately configured failure domains. Restore
from a surviving complete copy requires independent recovery material and a
qualified freshness/fencing path before new authority is activated.

Local commit, complete-copy acknowledgement, current availability, recovery
freshness and consensus finality MUST remain distinct. The product reports
the exact locally saved, protected and restore-checked revisions. A proof or
receipt does not recreate missing data or force future provider availability.

## Owners and integration invariants

| Owner | Stack obligation |
|---|---|
| Vault | Typed secret custody, protected operations, sealed-record semantics and replica/recovery policy |
| Soul / Ward | Single policy/permission authority; current exact decisions enforced at custody and effect boundaries |
| Mudra | Cryptographic algorithms, key derivation and authentication/verification profiles |
| Cybergraph | Canonical private application history and validated storage/synchronization entry points |
| BBG | Shared physical Database owner, conditional atomic publication and actual backend durability |
| Selected sync / Foculus profile | Authenticated ordering/fencing or consensus when the selected deployment requires it |
| Tape / Radio | Qualified framing and transport; no plaintext secret or implicit custody authority |
| Neuron / Sigma | Subject/action bindings and execution / attachment and asset management |
| Body / workers / cyb | Placement and isolation / qualified execution / product assembly and protected interaction |

1. Applications receive opaque references and authorized results, not root keys.
   Vault MUST NOT implement a second permission issuer, canonical history owner,
   database writer or general transport/consensus engine.
2. Store only ciphertext and permitted metadata outside custody. Private local
   namespaces, public-key descriptors and record hashes are not publication grants.
3. Complete content closure, selected head, protected-use reservations and request
   receipts cross the shared durability boundary consistently. CommitUnknown
   stops dependent dispatch until reopen and reconciliation.
4. Replication is bounded and idempotent. A receipt counted toward protection
   covers a complete durable revision under explicit retention/failure assumptions.
5. Device copy, device enrollment and writer promotion are different operations.
   Local database locks do not fence writers across machines; stale histories
   and conflicts cannot silently win by timestamp.
6. Restore cannot reset revocations, OTP counters, code consumption or quotas.
   It needs a complete selected revision and surviving freshness evidence;
   otherwise the result stays incomplete or read-only.
7. One Vault serves many neurons/networks. `derive_neuron` preserves existing
   profile bytes and native/foreign references. Custody IDs are not new subjects.
8. The [five execution selections](execution-model.md) still govern proved jobs.
   Custody restricts placement/disclosure; changing CPU/GPU/device creates no
   warrior or neuron and supplies no right to a private witness.
9. Unlock/recovery must bootstrap through private local storage without needing
   a signature from the locked seed. Headless and GUI hosts share the contract.

The [Vault conformance matrix](https://github.com/cyberia-to/vault/blob/main/specs/conformance.md)
is the cross-stack acceptance gate, including simultaneous loss of the writer
and one replica, stale/omitted history, partitioned handover, restore and retry
faults. Wire profiles and platform qualification belong to its implementation
packages. Detailed Vault requirements MUST be maintained there, not copied here.
