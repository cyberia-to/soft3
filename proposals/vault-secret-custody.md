---
tags: soft3, vault, mudra, neuron, architecture, proposal
crystal-type: process
crystal-domain: security
status: draft
date: 2026-09-16
---
# Vault: secret custody and authorized operations

Vault owns typed secrets and their protected use: seeds, private keys, passwords,
PINs, authentication factors and service credentials. Ordinary consumers request
operations; they cannot extract a root seed. The neuron's derivation operation is
`derive_neuron`. These are the owner's directions. This proposal develops them;
wire formats, platform profiles and recovery factors remain draft.

Composition follows [cyb's anatomy](../../cyb/anatomy.md) and the
[soft3 execution model](../specs/execution-model.md). Current implementation
observations live in [the custody audit](../audit/vault-custody-2026-09-16.md);
the [alignment review](../audit/vault-architecture-alignment-2026-09-16.md)
records the contracts checked and remaining decisions.

## Ownership and deployment

| Component | Responsibility |
|---|---|
| Vault | Typed secret generation/import, derivation, custody, protected operations, lock/unlock, backup/recovery and lifecycle |
| Mudra | Crypto algorithms, public verification and profile-specific encodings; secret operations execute inside Vault's boundary |
| Soul / Ward | Policy configuration / its single authorization authority, including enforcement at Vault's operation boundary |
| Neuron | Subject/binding references, exact action intents and execution through existing host ports |
| Cybergraph / BBG | Canonical history and private application records / their atomic durable storage through the existing database owner |
| Sigma | Assets and neuron attachments; presents spending and subject selection without holding private keys |
| Body / workers | Placement, process supervision and resource limits; no ambient access to secrets |
| Com / cyb | Controlled entry and approval surfaces, scoped metadata and product assembly; ordinary UI/agent context never receives root secrets |

Vault is a reusable component, independent of the GUI and Neuron engine. Its
crate graph points to crypto/profile code, not to a VM/renderer/inference host.
Public clients use a small protocol/client package; host composition supplies
authorization and storage ports. Soft3 owns this composition contract. An organ,
crate, repository and process are different boundaries: this proposal requires
no new organ or separately distributed binary, and leaves repository packaging
open. Headless hosts use the same interfaces as cyb.

One Vault can hold many secrets for many neurons and compatible networks.
`VaultRef` and `SecretRef` identify custody objects, not additional protocol
subjects. Creating a password entry does not create a neuron. Watch-only
attachments need no key or unlocked Vault. Secret metadata and even public-key
associations remain subject to disclosure policy; public-key material is not
automatically public graph data.

Native deployments isolate custody from application/plugin/agent processes,
using authenticated bounded IPC and an explicit OS sandbox/privilege profile.
This need not add a separately distributed binary: a product can launch its
Vault service mode from the same distribution. A mere Rust struct in the caller's
address space supplies an API abstraction, not protection from that process.
Browser/embedded deployments require a declared custody profile; a web worker
alone must not be presented as an equivalent hostile-host boundary.

## Secret classes and public interface

Each record has a versioned kind/schema, opaque reference, purpose and applicable
subject/service/network scope, allowed operations, Ward policy reference,
disclosure rule, custody/recovery profile and lifecycle revision. Payload and
sensitive metadata are encrypted. `KeyRef` is a restricted kind of `SecretRef`.
Kind, permission and custody level are independent: a password is not a seed,
and marking a software-held key non-exportable does not make it hardware-held.

| Secret kind | Protected use | Disclosure boundary |
|---|---|---|
| Root seed / mnemonic | Generate/import, derive purpose-scoped keys and neurons, sealed backup | No raw read/reveal/copy through application APIs |
| Private signing / authority key | Authorize an exact action, approved proof or authentication assertion | Public authorization result; private material stays in custody |
| Discovery / payload / channel / wrapping key | Profile-bound discovery, agreement, open/seal or key wrapping | Only declared results; internal storage keys never serve generic decryption |
| Password / PIN | Deliver to the bound service/device or use as an external authenticator | Recipient necessarily receives the value; trusted reveal/export requires a separate explicit grant |
| TOTP / HOTP authenticator | Produce a code under a fixed algorithm/parameter profile | Codes only during normal use; enrollment-seed transfer needs an explicit sealed migration profile |
| Recovery codes | Select/reserve one code for its bound provider | One authorized code release, with durable use state |
| API token / service credential | Authenticate through a bound adapter to a declared audience/origin | Delivery to that recipient; no implicit return to an agent/tool context |
| Passkey credential | Assertion bound to the relying party, challenge and supported profile | Assertion only; portability depends on the credential's custody profile |

"2FA" is a role in an authentication policy, not one storage format: OTP seeds,
passkeys and recovery codes have different operations and lifecycles. A stored
service password/PIN is also distinct from a Vault unlock factor. A low-entropy
PIN is not root-key entropy; its unlock profile must provide enforceable attempt
limits and resistance to offline guessing. Storing a TOTP seed beside a password
does not provide two independent factors against compromise of that Vault.

There is no generic secret dump, arbitrary derivation callback or unrestricted
sign/decrypt oracle. New kinds require a reviewed schema and operation profile;
unknown kinds fail closed. Retagging a seed as a password cannot enable reveal.
The interface returns descriptors, references and explicitly permitted results:

```text
create/import through trusted Vault ceremony -> SecretRef + permitted descriptor
derive_neuron(KeyRef, derivation_profile, scope, selector, grant, request_id)
    -> KeyRef + SubjectRef + public descriptor
authorize_action(KeyRef, canonical intent, grant, request_id) -> authorization
open_message(KeyRef, bound encrypted message, grant, request_id) -> permitted content
prove_authority(KeyRef, approved relation, public inputs, grant, request_id) -> proof
otp_code(SecretRef, bound OTP request, grant, request_id) -> code
deliver_credential(SecretRef, bound destination, grant, request_id) -> delivery receipt
reveal_entry(SecretRef, trusted output surface, grant, request_id) -> permitted entry
backup(VaultRef, recovery policy, grant, request_id) -> sealed recovery capsule
lock / revoke / rotate under their authenticated lifecycle policies
```

These signatures are design sketches, not a frozen wire ABI. Public descriptors
declare actual algorithm, purpose, custody level and allowed operation set.
Possession of a reference grants no authority. `reveal_entry` is permitted only
for explicitly revealable kinds under their policy; it cannot reveal seeds or
private keys. Purpose-scoped handles retain derived secret material. Payload
decryption never doubles as opening a vault backup. Credential delivery binds
the authenticated recipient and disclosed fields; redirects or origin changes
require renewed checks. Its receipt proves no external login succeeded.

`derive_neuron` deterministically resolves key material and its subject under an
explicit existing profile. Repeating the same root/profile/scope/selector returns
the same key and subject association. Native profiles return `Native(NeuronId)`;
a supported foreign profile retains `Foreign(domain, address_bytes)`, never a
hash or cast invented to fit a native ID. Derivation creates no grant, attachment,
active selection or ledger record. Sigma/host composition manages attachments.
Network, derivation domain, endpoint and display format are separate inputs;
changing an endpoint or worker cannot silently derive a different neuron.

Seed generation, mnemonic parsing and HD derivation move behind this boundary.
BIP-39/BIP-32 arithmetic may remain a low-level crypto implementation detail;
application use of `mudra::seed -> SigningKey` and raw-key getters ends. Existing
Cosmos paths, domain derivations, native IDs and signature bytes stay unchanged.
Moving custody does not silently introduce a new identity or hash profile.
Mudra's [programmable authority](../../mudra/specs/identity.md) remains the
strategic proof-based interface; current signature compatibility and future
private-proof profiles use the same custody boundary. Rotation cannot preserve
a key-derived subject ID without a separately specified authority transition.

Trusted import/recovery input goes directly to Vault's controlled entry surface,
outside generic chat, terminal history, argv/environment and ordinary UI state.
Com routes an explicit protected-entry mode before capture by history or Soma;
pattern matching arbitrary chat is not a guarantee of secret interception.
Revealable passwords/codes use a narrowly authorized output surface, not root-key
UI state or model context. Clipboard use is a separate disclosure permission.
Existing external mnemonic copies are a pre-existing exposure, not something
the import operation can retroactively revoke.

## Authorization protocol

Vault receives enough canonical intent to reconstruct the exact approved
statement: secret/purpose, caller, operation, payload, policy revision,
nonce/expiry, requested disclosures and applicable limits. Neuron actions also
bind subject/binding revision, key profile, network/genesis and program;
credential operations bind their service/origin. Local secret management does
not invent a network or neuron merely to fit the action format.
A caller-supplied digest plus `approved=true` is insufficient. Profile adapters
must define which action semantics and transaction fields Vault verifies.

Ward remains the sole permission authority derived from Soul. Vault authenticates
the caller and enforces that authority at the secret-operation boundary: either
the shared Ward evaluator runs there, or an authenticated Ward decision binds
the exact request with a defined freshness/revocation protocol. This is not a
second grant issuer or an independently edited permission database. Custody
constraints can further restrict a grant, never widen it. Grants bind caller,
secret/purpose, applicable network/service, operations, limits, generation and
lifetime. A caller cannot mint one by filling a JSON object.
Ward policy revisions, decision credentials and trusted confirmations need the
selected profile's protection; an authenticated decision from a compromised
ordinary application does not establish independent user authorization.
Headless automation uses explicit bounded grants; unattended operation does not
require broad export permissions or a prompt for every signature.

Ward owns grant/revocation state; Vault durably tracks protected uses and local
quota reservations under it. Policy freshness must be established before use;
offline automation needs an explicitly authorized lease and revocation bound.
A request ID binds the complete intent; conflicting reuse is rejected. Reserve
permission/quota before a protected operation and durably record its outcome
before replying.
Crash recovery preserves spent/reserved exposure and exact retry meaning. The
profile defines handling of uncertain hardware/prover completion.

OTP and recovery codes need their own retry rules. TOTP may repeat within a time
window; local generation cannot promise one-time acceptance at the provider.
The OTP profile fixes parameters and time/counter authority; an untrusted caller
cannot choose arbitrary future time steps or rewind a counter.
HOTP counter reservations are atomic and durable before release; recovery cannot
silently rewind them. Recovery codes distinguish available, reserved, consumed
and uncertain use. A timeout or backup restore cannot make a released code unused.
Concurrent bodies require one authorized writer or a qualified coordination
profile for counters, use state and quotas; copying encrypted backups is not one.

Signing has an explicit linearization point against revocation. Revocation can
stop later operations; it cannot retract an already delivered signature. Neuron
still checks permission at effect dispatch, and the receiving protocol enforces
its own current-root, expiry and nonce rules. Vault signing is not a replacement
for ledger admission or consensus verification.

## Proofs and private computation

Proved execution uses soft3's five selections: machine, environment, proof,
network and executor. Custody limits permissible placements and disclosure;
it is not a sixth identity axis or a new warrior. Reuse the host's Body/worker
composition and approved proof adapters. One warrior can serve many compatible
networks and workers; a GPU switch creates neither a warrior nor a neuron.
Network-specific context remains bound to each job. Local password/OTP use and
native crypto operations need no VM or proof worker merely to use Vault.

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

Ciphertext and custody receipts use Cybergraph's private application/history
ports over the existing BBG database owner. Cybergraph owns history; BBG supplies
atomic storage and durability, not another policy or history owner. Vault opens
no competing writer and introduces no canonical WAL/database. The store receives
ciphertext, never plaintext keys. Private namespaces are not publication grants;
public hashes, receipts, subject associations and lookup patterns may also leak.
Bootstrap/open/unlock must work through an authorized local private storage port
before a signing neuron is available: loading the seed cannot require signing
with that same locked seed. Public graph admission remains independently gated.

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

Vault recovery restores secret custody, not the whole robot. Cyb's resurrection
contract remains name + Soul + Vault recovery material + Log. A seed reconstructs
only deterministic derivations: imported passwords, OTP enrollments, tokens and
use counters also require their encrypted records and authenticated freshness.
The independent recovery factor and capsule locator must survive loss of the
original device; a credential available only inside that lost Vault cannot
provide its recovery path. Device-bound keys need provider re-enrollment or an
explicit alternate authority path.

Mudra's [private recovery](../../mudra/specs/private-recovery.md) still owns
discovery/coverage and spend-state requirements. Vault retains its purpose-scoped
keys; Neuron and authorized workers perform the allowed recovery computation.
Neither a Vault backup nor a derived seed replaces retained payment ciphertexts,
verified history or available openings. This proposal changes no UTXO/account
model and promises no faster scan by itself.

## Cutover and acceptance

1. Freeze typed record/operation schemas, threat/custody levels, recovery policy,
   Ward freshness semantics and versioned sealed storage. Implement the
   service/client boundary with test secrets of every initially supported kind.
2. Import existing secrets through a bounded migration ceremony. Verify public
   identities and exact derivation/signing vectors, durable reopen and independent
   recovery before disabling the old readers. Envelope changes use an explicit
   versioned migration; existing private record/profile bytes remain readable.
   Do not automatically delete the user's original recovery material or claim
   erasure from SSDs/backups.
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
witness. Include secret-kind confusion/relabeling, credential destination changes,
protected-entry bypass, OTP/counter crashes, recovery-code uncertainty and two
bodies attempting the same reserved use. Successful restore must reproduce the
expected subjects and preserve imported records and protected-use state.

Before implementation, settle the first platform's isolation level, the recovery
factor/capsule format, and which typed operation profiles are allowed initially.
Neither an encrypted file alone nor the present KeyVault adapter satisfies this
complete custody contract.
