---
tags: soft3, vault, mudra, neuron, architecture
status: superseded
date: 2026-09-16
---
# Vault has its own home

Vault is now an independent component in
[cyberia-to/vault](https://github.com/cyberia-to/vault). The owner selected a
dedicated repository and made safe storage, private synchronization and recovery
the first end-to-end soft3 use case.

The canonical product and component contracts are:

- [Product README](https://github.com/cyberia-to/vault/blob/main/README.md).
- [Specification map](https://github.com/cyberia-to/vault/blob/main/specs/README.md).
- [Storage](https://github.com/cyberia-to/vault/blob/main/specs/storage.md),
  [synchronization](https://github.com/cyberia-to/vault/blob/main/specs/synchronization.md)
  and [recovery](https://github.com/cyberia-to/vault/blob/main/specs/recovery.md).
- [Conformance](https://github.com/cyberia-to/vault/blob/main/specs/conformance.md)
  and [implementation order](https://github.com/cyberia-to/vault/blob/main/roadmap/README.md).

Soft3 retains the [cross-component contract](../specs/vault.md). Detailed secret
schemas and custody/recovery requirements have one owner in Vault rather than
two competing specifications.

The original proposal is preserved at
[soft3 4ddfab4](https://github.com/cyberia-to/soft3/blob/4ddfab48f81f5df9cb3dcb0ed9b5da3d0001d7de/proposals/vault-secret-custody.md).
The [source custody audit](../audit/vault-custody-2026-09-16.md) and
[alignment review](../audit/vault-architecture-alignment-2026-09-16.md) remain
dated evidence of the earlier design and implementation. Creating the repository
does not implement or qualify the new Vault runtime.
