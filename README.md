---
title: soft3
tags: cyber, soft3, sdk
crystal-type: spec
crystal-domain: cyber
icon: "👙"
alias: soft3 stack, cyb stack, software stack, proof pipeline
---
soft3 is the substrate for planetary superintelligence: signed staked [[cyberlink]]s between [[particles]], authenticated state in [[bbg]], proofs via [[zheng]] / [[nox]]. the product chaosnet is **spacepussy-test**.

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

[[cyber]] · [[cyb]] · [[soft3/stack]] · [[install]] · [[soft3/docs/launch|launch]]
