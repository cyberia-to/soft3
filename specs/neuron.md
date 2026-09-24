---
title: neuron subject and program execution
tags: soft3, neuron, architecture, spec
crystal-type: spec
crystal-domain: cyber
status: accepted
---
# Neuron subject and program execution

A neuron is the subject of protocol actions and may execute installed progs.
Robot attaches neurons; a prog supplies code/state and invocation records.
There is one subject ID, NeuronId. Program, task, operation and checkpoint IDs
identify data, not another level of actors with keys.

[Cyb architecture](../../cyb/specs/architecture.md) defines robot/organ meanings.
[Execution model](execution-model.md) defines independent machine, environment,
proof, network and executor choices. A neuron is independent of a worker/device.

## Identity profiles

| Profile | Native identifier | Authentication / compatibility |
|---|---|---|
| cyber-secp256k1-hemera-v1 | Hemera of the compressed secp256k1 public key | Existing mudra claim/domain derivation and ADR-036 operations; preserve exact bytes |
| Foreign protocol account | Original domain-qualified address bytes | Its declared protocol adapter verifies access; no implicit native cyber ID |
| Proof-native subject | Defined only by an explicit versioned profile | No substitution of H(secret) for existing H(pubkey) identities; unsupported profiles reject writes |

An ID identifies a subject; it alone proves no authority. Import/observe remains
available without signing keys or a runtime. Native NeuronId stays a 32-byte
wire type owned by neuron-id and re-exported by foundational APIs. Implementations
preserve native domains and distinguish destination network from key derivation
scope, transport endpoint and display encoding.

## Action and state boundary

An admitted action pins subject, binding revision, prog/invocation/native caller,
network, payload, grant, worker contract and budget. Ward/vault enforce current
permissions at dispatch. Keys and live grants are never VM input or checkpoint
authority. Selection changes cannot retarget pending operations.

Independent progs/jobs can progress concurrently. Authoritative state commits use
revision/CAS checks, and shared prog state follows a declared conflict policy.
Invocations preserve inputs, context, continuation, usage and pending effects.
Graph namespaces and prog bindings introduce no new signing subject.

Cybergraph and BBG store authoritative history; foculus orders protocol signals;
log renders it. GraphSession hosts many neurons. Local commits and finality remain
different: this contract neither upgrades unsigned acceptance to authentication
nor treats local checkpoints as network consensus.

## Compatibility

Immutable cell v1 schema bytes, IDs, claims and author evidence remain readable.
Migration maps legacy origins to authorized neuron/prog bindings and preserves
all live and terminal effects/resources. No legacy birth hash is cast into a
verified NeuronId. Upstream, Rust memory, DAS/grid, WASM and biological cells
retain their meanings; domain modules/shards/UI targets receive precise names.

The [roadmap](../roadmap/neuron-cell-convergence.md) and
[implementation ledger](../audit/neuron-cell/implementation.md) distinguish
requirements, completed work and release evidence.
