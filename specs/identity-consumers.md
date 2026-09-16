# Identity at consumer boundaries

The [neuron contract](../../neuron/specs/identity.md) and
[cyb architecture](../../cyb/specs/architecture.md) define one subject with
explicit domain/network/binding. Neither a process nor a prog is a second
signing subject. This document fixes consumer compatibility, not a new wire
protocol or an automatic cross-chain bridge.

| Consumer | Meaning retained | Boundary |
|---|---|---|
| mudra, lytics-event | domain KDF, secp256k1, H(compressed pubkey), ADR-036 address/signature | Domain and HRP are pinned inputs; no derivation or signed-byte change |
| both cyberia-my checkouts | `cyberia.my` / `cyber`, six-decimal signal body, local browser sequence | Same profile and browser storage keys; neither checkout is generated from the other |
| lytics ingestion | verified event public key and its original address | New retained provenance permits exact native replay; old rows without keys remain unresolved legacy observations |
| inf | native subject as unchanged Hash bytes in its declared schema | No authority from a Hash; execution schemas are projected by neuron-node, never implemented in the query language |
| true-cyber, cyb, soft3 native | explicit native network and complete signed action | Common custody/registry/coordinator; endpoint identity cannot replace subject or network |
| trisha / Neptune | native upstream wallet, receiving address, selected network | Foreign domain/address/network; no cast of Neptune address to native H(pubkey) |
| soft3/js, go-cyber, space-pussy, bostrom, bootloader | Cosmos chain/account/HRP and protobuf/amino/JSON bytes | SDK messages, historical genesis/snapshots and addresses remain their original protocol |
| cybernet | chain + contract + netuid + UID, hotkey/coldkey roles | UID identifies a subnet slot and may be reassigned; it is not a native NeuronId |
| hub-skills | entry owner, neuron, network, protocol, endpoint | Entry ID is data; discovery entry alone does not prove custody or authorize endpoint actions |
| portal cyber-std | chain-scoped neuron address, bandwidth, program/contract types | Existing CosmWasm messages/storage stay unchanged |
| midao contracts | chain ID, EVM address, token/contract roles | Legal collective membership, contract authority and agent execution are separate |
| radio | endpoint public key, transport session, opaque content identifiers | Transport authentication does not authorize a neuron action; preserve existing iroh/CID profiles |
| tape | frames and dialect-selected reference presentation | `@name` is presentation, not verified authority; row cells stay row cells |

An adapter may observe a foreign address as
`SubjectRef::Foreign { domain, address: original_bytes }`, paired with its
explicit `NetworkRef::Foreign`. It must retain the protocol's canonical address
encoding, chain/contract scope and validation rules. A display name, same HRP,
numeric UID or successful TCP/QUIC connection cannot establish equivalence with
a native subject. The current generic robot foreign attachment is watch-only;
control requires an implemented domain verifier and custody contract.

Changing key material changes H(pubkey) identity in the implemented native
profile. Recovery/rotation under a future stable-subject proof policy requires
that policy's separately versioned verifier; a migration cannot rewrite past
authors or synthesize missing signatures. These rules also apply to imported
rows from earlier unsigned local graph tools.

`neuron-id`, `neuron-model` without default features, mudra without `prove`,
lytics-event and inf's value/source interfaces must stay free of execution,
storage, GUI and inference transitive dependencies when used only for identity.
Opting into a graph/worker adapter is a different build profile. Actual builds,
frozen compatibility vectors and source revisions belong in
[the consumer audit](../audit/neuron-cell/consumers.md).
