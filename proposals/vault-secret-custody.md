---
tags: soft3, vault, mudra, neuron, architecture, proposal
crystal-type: process
crystal-domain: security
status: draft
date: 2026-09-16
---
# Vault: secret custody and authorized operations

The owner's direction is explicit: Vault owns seed custody and prevents ordinary
consumers from extracting it. This proposal develops that direction; concrete
wire formats, platform profiles and recovery factors are not yet accepted.
Current implementation observations live in
[the custody audit](../audit/vault-custody-2026-09-16.md).

## Ownership and deployment

| Component | Responsibility |
|---|---|
| Vault | Secret generation/import, derivation, custody, protected operations, lock/unlock, backup/recovery and key lifecycle |
| Mudra | Crypto algorithms, public verification and profile-specific encodings; secret operations execute inside Vault's boundary |
| Ward | Permission/policy semantics; their enforcement also runs at Vault's operation boundary |
| Neuron | Subject/binding references, exact intents, durable execution and operation receipts |
| BBG storage | Durable private ciphertext records and transactions, through its existing database owner |
| cyb | Public key/permission status and approved interaction; ordinary UI/agent context never receives root secrets |

Vault is a reusable component, independent of the GUI and Neuron engine. Its
crate graph points to crypto/profile code, not to a VM/renderer/inference host.
Public clients use a small protocol/client package. A dedicated repository/crate
is the proposed home; Soft3 owns the cross-component contract.

Native deployments isolate custody from application/plugin/agent processes,
using authenticated bounded IPC and an explicit OS sandbox/privilege profile.
This need not add a separately distributed binary: a product can launch its
Vault service mode from the same distribution. A mere Rust struct in the caller's
address space supplies an API abstraction, not protection from that process.
Browser/embedded deployments require a declared custody profile; a web worker
alone must not be presented as an equivalent hostile-host boundary.

## Secret classes and public interface

Root seeds, private signing keys, discovery keys and storage/wrapping keys are
custodial objects with purpose restrictions. Exportable password-manager entries
are a different class. An imported entry tagged "seed" cannot acquire a generic
copy/reveal operation through the password UI.

The application interface has no `get_seed`, `get_private_key`, secret-bearing
serialization, arbitrary derivation callback or unrestricted signing oracle.
It returns public descriptors, references and policy-approved results:

```text
create/import through trusted Vault ceremony -> KeyRef + public descriptor
derive_identity(KeyRef, approved derivation profile) -> KeyRef + public identity
authorize_action(KeyRef, canonical intent, grant, request_id) -> authorization
open_message(KeyRef, bound encrypted message, grant) -> permitted message content
prove_authority(KeyRef, approved relation, public inputs, grant) -> proof
backup(VaultRef, recovery policy) -> sealed recovery capsule
lock / revoke / rotate under their authenticated lifecycle policies
```

These signatures are design sketches, not a frozen wire ABI. Public descriptors
declare actual algorithm, key purpose, custody level and allowed operation set.
An opaque KeyRef names a key; possession of it grants no authority. Root and
storage keys are unavailable to general signing/decryption operations. Channel
and derived secret material stays behind purpose-scoped handles when required
by the profile. Payload decryption never doubles as opening a vault backup.

Seed generation, mnemonic parsing and HD derivation move behind this boundary.
BIP-39/BIP-32 arithmetic may remain a low-level crypto implementation detail;
application use of `mudra::seed -> SigningKey` and raw-key getters ends. Existing
Cosmos paths, domain derivations, native IDs and signature bytes stay unchanged.
Moving custody does not silently introduce a new identity or hash profile.

Trusted import/recovery input goes directly to Vault's controlled entry surface,
outside generic chat, terminal history, argv/environment and ordinary UI state.
Existing external mnemonic copies are a pre-existing exposure, not something
the import operation can retroactively revoke.

## Authorization protocol

Vault receives enough canonical intent to reconstruct the exact approved
statement: subject/key profile, network/genesis, operation/program, payload,
policy revision, nonce/expiry, requested disclosures and applicable limits.
A caller-supplied digest plus `approved=true` is insufficient. Profile adapters
must define which action semantics and transaction fields Vault verifies.

Vault authenticates the caller and checks current Ward policy inside its trust
boundary. Grants bind caller, key/purpose, network, operations, limits, policy
generation and lifetime. A caller cannot mint one by filling a JSON object.
Headless automation uses explicit bounded grants; unattended operation does not
require broad export permissions or a prompt for every signature.

Grant revocation, quotas and request identity have durable state. A request ID
binds the complete intent; conflicting reuse is rejected. Reserve permission/
quota before a protected operation and durably record its outcome before replying.
Crash recovery preserves spent/reserved exposure and exact retry meaning. The
profile defines handling of uncertain hardware/prover completion.

Signing has an explicit linearization point against revocation. Revocation can
stop later operations; it cannot retract an already delivered signature. Neuron
still checks permission at effect dispatch, and the receiving protocol enforces
its own current-root, expiry and nonce rules. Vault signing is not a replacement
for ledger admission or consensus verification.

## Proofs and private computation

ZK hides a witness from the verifier, not from the prover. A seed-derived private
witness stays inside the declared custody/prover boundary. General workers,
remote proving services, GPU buffers and model context do not automatically gain
access by implementing a prove method.

Only approved relations and public-output schemas may use custodial secrets;
an arbitrary circuit could simply output them. Prover versions, memory handling,
error paths and output validation therefore belong to the trusted profile.
Offload public work freely within its contract. Private distributed proving/MPC
would require a separately qualified protocol. A backend unable to execute an
operation at the requested custody level returns Unsupported; no key export
fallback is allowed.

## Storage, isolation and recovery

Use a random storage-encryption key, independent of the identity seed. Versioned
authenticated encryption protects secrets and sensitive metadata. Envelopes bind
vault/object identity, key purpose, profile and generation to prevent substitution.
Existing AEAD can be retained under a specified envelope; a new primitive is not
required merely to separate custody.

Wrap that storage key using an explicitly selected device/unlock factor and an
independent recovery path. A passphrase profile requires a salted memory-hard KDF
and concrete measured parameters; a hardware profile declares its supported key
and operation types. The only means to unlock a stored seed cannot be that same
stored seed.

Ciphertext and custody receipts use the existing BBG private storage path through
its database owner. Vault does not open a competing writer on an exclusively
owned database. The store receives ciphertext, never plaintext keys. Keep records
outside public graph publication; a public content hash or lookup pattern may
also disclose metadata. No new canonical WAL/database is proposed.

Acknowledged updates require the actual backend durability barrier. Atomic
record/wrapped-key/generation updates, backups and restoration need crash tests.
AEAD does not stop an adversary restoring an older valid database. Policy rollback
protection requires a declared monotonic hardware anchor, authenticated external
state or another explicit freshness mechanism. Restore never silently resets
revocations, signing quotas or ledger replay state.

Software Vault unseals secrets in its protected process memory while operating.
It limits copies, zeroizes buffers, suppresses secret debug/serialization/errors
and applies available dump/swap protections. Those measures do not claim secrecy
against an attacker controlling that process or its OS.

Hardware-native keys can have a stronger non-extraction property only for
supported operations. Apple's documented Secure Enclave interface supports P-256
and does not import arbitrary existing private keys; it is not a generic
secp256k1/CSIDH/proof processor. A hardware key wrapping our software seed protects
the sealed storage path, while the unwrapped seed still enters software memory.
[Apple documentation](https://developer.apple.com/documentation/security/protecting-keys-with-the-secure-enclave).
Android also separates extraction prevention from authorization and requires
hardware support for each algorithm/mode combination.
[Android Keystore](https://developer.android.com/privacy-and-security/keystore).

The proposed default backup is a sealed capsule restored directly by another
authorized Vault using an independent recovery factor/device policy. It includes
key derivation/version/epoch metadata and the means to locate required backups.
Portable recovery and device-bound non-exportable keys need separate profiles;
do not promise seed restoration of hardware keys that have no export/import path.
The recovery factor has custody-level authority and needs its own protection.
An optional raw-mnemonic ceremony would explicitly weaken the strict no-export
profile and is not enabled by this proposal.

## Cutover and acceptance

1. Freeze threat/custody levels, recovery policy, intent/grant semantics and
   versioned sealed storage. Implement the service/client boundary with test keys.
2. Import existing secrets through a bounded migration ceremony. Verify public
   identities and exact derivation/signing vectors, durable reopen and independent
   recovery before disabling the old readers. Do not automatically delete the
   user's original recovery material or claim erasure from SSDs/backups.
3. Replace Neuron KeyVault, cyb identity loading and UI seed-copy paths with
   Vault clients. Missing/locked/corrupt custody fails closed or opens watch-only;
   it never silently creates a replacement identity or uses a fixed test key.
4. Add concrete hardware and private-prover profiles without lowering the
   operation's required custody guarantee.

Required adversarial checks: unknown caller/copied handle; wrong network or
purpose; altered intent/grant; revoke-vs-sign race; exact/conflicting retries;
crash at each reservation/persistence/reply boundary; rollback and restore;
backup loss/corruption; unsupported hardware operations; secret leakage through
serialization/logs/errors/UI; a malicious proof program trying to reveal its
witness. Successful restore must reproduce the expected public identity.

Before implementation, settle the first platform's isolation level, the recovery
factor/capsule format, and which typed operation profiles are allowed initially.
Neither an encrypted file alone nor the present KeyVault adapter satisfies this
complete custody contract.
