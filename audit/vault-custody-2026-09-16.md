---
tags: soft3, vault, mudra, neuron, audit
date: 2026-09-16
---
# Existing secret-custody boundary

Read-only source inspection supporting the
[Vault proposal](../proposals/vault-secret-custody.md). No real mnemonic/key file
was opened, no signing operation was requested, and no custody migration ran.

Observed bases: Mudra `e2d1a63bccd760b96ba55f448ef338b7ab027c37`, Neuron
`34bbff9dbf36421a45c5c283e6c1336dcafc997a`, cyb
`49cf94c035a09cb45f7f489ddba0e94b30017dde`. Mudra/Neuron include the working-tree
changes identified in the [readiness assessment](../../mudra/audit/readiness-2026-09-16.md).
Cyb was on master with only Cargo.lock modified; do not substitute older dirty
P13 source for this observed checkout. This is a source review, not fresh testing.

| Source | Observed boundary |
|---|---|
| [Mudra seed](../../mudra/src/seed.rs) | `seed()` returns a raw 64-byte seed; `signing_key()`/`cosmos_key()` return a SigningKey; mnemonic generation returns String |
| [Mudra domain](../../mudra/src/domain.rs) | `domain_scalar()` returns a secret scalar; `DomainKey::signing_key()` exposes the private signing object |
| [Neuron authority](../../neuron/node/src/authority.rs) | KeyVault holds a caller-supplied SigningKey in the host process; SigningVault narrows runtime access but supplies no isolated custody/store |
| [Neuron CLI](../../neuron/cli/src/main.rs) | `key()` reads a raw 32-byte scalar; `keygen()` writes one to a mode-0600 file and syncs it |
| [Cyb vault store](../../cyb/shell/src/worlds/vault/store.rs) | XChaCha20-Poly1305 seals entries, using SHA256(domain || BIP39 seed) derived from a separately plaintext mnemonic file; `key()` returns raw key bytes |
| [Cyb Vault UI](../../cyb/shell/src/worlds/vault/mod.rs) | The root mnemonic is loaded into a String entry in UI state; generic reveal/copy handling includes that entry; TOTP generates codes, not a vault-unlock factor |
| [Cyb identity](../../cyb/shell/src/worlds/identity.rs) | GUI holds an Arc<SigningKey>; load_or_mint can create a new mnemonic on read failure and has an ultimate fixed-test-vector fallback on derivation/generation failure |
| [Cyb CLI identity](../../cyb/cli/src/main.rs) | Reads/creates the plaintext mnemonic independently and derives a key outside a custody service |

These sources implement useful cryptographic helpers, local authorization and
encrypted entries. They do not implement a single owner for non-exportable seeds.
Mode 0600 and a private Rust field are different boundaries from an isolated,
policy-enforcing Vault. Encrypting entries under a plaintext root stored beside
them does not protect the root from a reader of that storage.

The store's temp-write/rename sequence contains no file/directory sync; permissions
are changed after rename with errors ignored. Source inspection therefore does
not establish power-loss durability or strict access-mode handling. The current
generic entry type is Clone+Debug with plaintext values. These observations
motivate explicit storage/memory contracts rather than certifying an exploit.

The preceding readiness assessment's passing Neuron/Mudra tests concern local
execution/cryptographic helpers. They do not qualify production custody, hardware
non-extraction, protected import or recovery ceremonies. Vault is a distinct
mission-critical implementation requirement.
