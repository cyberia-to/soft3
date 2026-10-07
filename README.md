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

## releases

[Release train](specs/releases.md) qualifies concrete soft3 builds. Cyber and
Cyb pin one build and its checksum manifest, inherit its component revisions,
and add product acceptance. [GitHub Releases](https://github.com/cyberia-to/soft3/releases) holds draft
candidates with source inventories, platform binaries and gate receipts.

## install

```bash
cargo install soft3          # stack CLI + real node
cargo install true-cyber     # product face (binary: cyber)
```

## run a node

```bash
soft3 node --home ~/.spacepussy-test --bind 127.0.0.1:7780 --moniker dev-1
```

engine: **cybergraph + bbg**. not a status stub.

```bash
# submit a cyberlink (labels hemera-hashed, or hex ids)
curl -sS -X POST http://127.0.0.1:7780/v1/link \
  -H 'content-type: application/json' \
  -d '{"neuron":"01","from":"0a","to":"0b","amount":1}'

soft3 sync                   # public edge on cybernode
cyber sync                   # same via true-cyber
```

public chaosnet: `https://cyb.ai/spacepussy-test` (cyberproxy).

launch manual: [[soft3/docs/launch|launch spacepussy-test]].

cosmos **space-pussy** / **bostrom** on cybernode are bootloader chains — different substrate. see [[bootloader]].

[[cyber]] · [[cyb]] · [[soft3/status]] · [[install]] · [[soft3/docs/launch|launch]]
