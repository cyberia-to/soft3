---
tags: soft3, vault, mudra, neuron, architecture, audit
date: 2026-09-16
---
# Vault architecture alignment

Review of the [custody proposal](../proposals/vault-secret-custody.md) against
stack ownership and the owner's requests for typed secrets and `derive_neuron`.
The revised proposal fits the composition model. It remains a design draft,
not an implementation or security qualification. No secret migration ran.

## Sources and scope

The proposal baseline is soft3 `842f1f4cf59eb4ce665f72833b7008101f132903`.
The [source custody audit](vault-custody-2026-09-16.md) records implementation
bases and gaps; this review does not overwrite those findings.

Reviewed local contracts on 2026-09-16:

- [Soft3 execution model](../specs/execution-model.md), especially sections 1–6.
- [Cyb anatomy](../../cyb/anatomy.md), [Vault](../../cyb/parts/vault.md),
  [Ward](../../cyb/parts/ward.md), [Sigma](../../cyb/parts/sigma.md),
  [Com](../../cyb/parts/com.md) and [Body](../../cyb/parts/body.md).
- [Neuron integration](../../neuron/specs/integration.md), plus the local
  `neuron/specs/identity.md`, `soft3/specs/neuron.md` and
  `soft3/specs/identity-consumers.md` contracts.
- [Mudra identity](../../mudra/specs/identity.md) and
  [private recovery](../../mudra/specs/private-recovery.md).

The three named subject/consumer files above were untracked working-tree
contracts marked accepted; Neuron integration also has local changes. They
inform compatibility here but are not represented as published by this change.
`cyb/specs/architecture.md`, referenced by those contracts, was absent in this
checkout. Cyb anatomy and its part pages were inspected directly instead.
Older implementation-status prose in those pages is not evidence of present
runtime behavior; their ownership rules are the relevant source.

## Findings incorporated into the proposal

| Contract / invariant | Issue in the first draft | Correction |
|---|---|---|
| Vault owns secrets, not only signing keys | Passwords were merely a separate unspecified class; PIN/OTP/service lifecycles were absent | Typed records and an operation/disclosure matrix; root keys cannot inherit password reveal/copy |
| Neuron is the protocol subject | `derive_identity` was ambiguous about identity creation and attachment | `derive_neuron` preserves profile bytes and native/foreign subject semantics; derivation grants no authority or attachment |
| Soul configures, Ward authorizes, Vault protects use | An internal policy check could imply a second policy owner | One Ward authority; exact, fresh decisions enforced at Vault; custody rules only narrow permission |
| Sigma manages assets and neuron attachments | Cyb was represented as one undifferentiated client | Sigma selection, Com protected input and Body placement have their existing roles |
| One organism does not mean one crate/process | A dedicated repository was prematurely prescribed | Independent custody/client boundary; repository layout remains open; service mode may ship in the same binary |
| Cybergraph owns history, BBG owns durable storage | Direct reference to BBG omitted the history owner | Existing private Cybergraph/application ports over the shared BBG owner; no parallel database/history |
| Five execution selections and many networks/workers | Private proving had no explicit execution-model mapping | Custody constrains executor placement and disclosure; reuse workers, isolate each job; no new subject on CPU/GPU/device changes |
| Mudra separates private authority, discovery and spending | Current raw-key compatibility could be mistaken for the final authority model | Current signature bytes remain compatible; approved private-proof relations stay behind custody; no substitute crypto selected |
| Private recovery needs retained data and complete verified history | A seed/capsule could sound like complete recovery | Imported entries/counters need backed-up state; payment recovery retains its own history/coverage contract |
| Robot resurrection includes name, Soul, Vault and Log | Vault backup scope was not explicit | Capsule restores custody; robot and payment history remain with their existing owners |

## Consequences for the first implementation

Three distinctions must survive the implementation:

1. **Use versus disclosure.** A password can be delivered to its approved service;
   that service learns it. An OTP operation returns a code, not its seed. Root
   seeds and private keys have no ordinary read/reveal/copy path. An agent gets
   the allowed result, not an unrestricted credential getter.
2. **Entry versus unlock factor.** A stored PIN/password/OTP is not automatically
   suitable for unlocking Vault. Low-entropy factors need a qualified protection
   profile; co-storing a password and OTP does not make their custody independent.
3. **Keys versus state.** Deterministic derivation can recover keys. Imported
   secrets, OTP counters, recovery-code use, quotas and revocations need durable
   records and freshness. Restore must not make a consumed capability unused.

Implementation decisions still to pin are the first platform's isolation and
trusted input/confirmation surface; Ward decision freshness and offline leases;
the versioned envelope and independent recovery factor/locator; and supported
operation profiles. Begin with one authorized writer for mutable custody state;
multi-body writes require coordination, not just replicated encrypted files.
These are engineering contracts to complete, not reasons to choose a different
identity/signature scheme.

Document checks passed: 29 local link targets, Markdown table column counts,
`derive_neuron` naming in the active proposal, `git diff --check`, and the
repository's NTFS filename check over `~/cyber`. Runtime, fault-injection and
security tests in the proposal are acceptance requirements, not passing results.
