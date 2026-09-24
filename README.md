---
title: soft3
tags: cyber, soft3, sdk
crystal-type: spec
crystal-domain: cyber
icon: "👙"
alias: soft3 stack, cyb stack, software stack, proof pipeline
---
soft3 is the substrate for planetary superintelligence: signed staked [[cyberlinks]] between [[files]], authenticated state in [[bbg]], proofs via [[zheng]] / [[nox]]. the product chaosnet is **spacepussy-test**.

## first storage use case: vault

[Vault](https://github.com/cyberia-to/vault) is the first end-to-end use case for
safe storage and private synchronization: retain typed secrets, replicate their
encrypted state and recover after device loss. Its product and specifications
have a dedicated repository; soft3 owns the [composition contract](specs/vault.md).
This is the next implementation target, not an already shipped custody service.

## architectural foundation

[Execution model — the foundational contract](specs/execution-model.md).

A warrior supports a VM/OS family across an open-ended set of compatible
networks. Workers instantiate its capabilities inside a node, in a separate
process, or remotely. CPU/GPU selection changes the backend, not the warrior.
Machine, environment, proof profile, network instance and executor are
separate selections with explicit compatibility rules.

[All specifications](specs/README.md) · [Shared vocabulary](specs/terms.md)

[Neuron + cell convergence roadmap](roadmap/neuron-cell-convergence.md) —
one subject identity, durable program execution, and explicit legacy migration.
The local implementation now uses [neuron](specs/neuron.md), progs and a shared
GraphSession. The [implementation ledger](audit/neuron-cell/implementation.md)
records completed packages and remaining acceptance gates; the
[inventory](audit/neuron-cell/README.md) preserves the original scan. This is
source-checkout evidence, separate from published releases and deployed nodes.

## releases

[Release train](specs/releases.md) owns stack qualification for soft3, cyber and
cyb. [GitHub Releases](https://github.com/cyberia-to/soft3/releases) holds draft
candidates with source inventories, platform binaries and gate receipts.

## install

Build the migration from compatible sibling checkouts with Rust 1.95+:

```nu
cargo install --path ~/cyber/soft3/crate --locked
```

Registry releases (`cargo install soft3` / `cargo install true-cyber`) retain
their published contents. They do not acquire these workspace changes until a
new release. The [headless client](../true-cyber/README.md) and
[product node launcher](../cyber/specs/cli.md) both name their binary `cyber`;
select the intended source explicitly when building or invoking them.

## run a node

For a fresh local home, activate authenticated publication explicitly before
starting the node:

```nu
soft3 auth enable --home ~/.spacepussy-test
soft3 node --home ~/.spacepussy-test --bind 127.0.0.1:7780 --moniker dev-1
```

The engine is cybergraph + BBG. Stop old writers before upgrading a legacy home;
use `soft3 auth enable --home DIR --import-legacy` to import retained log bytes
and activate the monotonic authenticated generation. Original genesis/log
history remains available. Activation creates no neuron key or attachment.

In another terminal, inspect `http://127.0.0.1:7780/capabilities` and compare the
reported network with the activation output or trusted genesis. `NETWORK_HEX`
below is that complete 64-hex native ID. Use a separate client home:

```nu
cargo run --manifest-path ~/cyber/true-cyber/Cargo.toml --locked -- --home ~/.cyber/native-client neuron create desktop NETWORK_HEX
cargo run --manifest-path ~/cyber/true-cyber/Cargo.toml --locked -- --home ~/.cyber/native-client link topic answer --network NETWORK_HEX --rpc http://127.0.0.1:7780
```

`neuron attach KEY_FILE NETWORK_HEX` reuses existing custody. Submission uses
`POST /v3/action`; receipt lookup and bounded history use `/v3/receipt/...` and
`/v3/history`. Retain the printed request ID for exact retry or reconciliation.
Endpoint acceptance is distinct from consensus finality. Authenticated clients
require matching capabilities and never fall back to author-label writes.
See the [signed adapter](specs/signed-native-adapter.md),
[consumer boundaries](specs/identity-consumers.md) and
[launcher evidence](../cyber/audit/neuron-launcher.md).

The configured public chaosnet endpoint is `https://cyb.ai/spacepussy-test`
(cyberproxy). Its deployed profile must be observed separately; local migration
tests do not establish a remote upgrade. `soft3 sync` probes status; the native
client's explicit `sync --network NETWORK_HEX` mirrors the declared history.

launch manual: [[soft3/docs/launch|launch spacepussy-test]].

cosmos **space-pussy** / **bostrom** on cybernode are bootloader chains — different substrate. see [[bootloader]].

[[cyber]] · [[cyb]] · [[soft3/status]] · [[install]] · [[soft3/docs/launch|launch]]
