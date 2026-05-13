---
tags: cyber, soft3, cli
crystal-type: spec
crystal-domain: cyber
---
# soft3 CLI

`soft3` — command-line interface to the soft3 stack.

## commands

```
soft3 link <from> <to> [--token <cid>] [--amount <n>] [--valence +1|0|-1]
    construct and submit a cyberlink signal

soft3 query particle <cid> [--root <bbg_root>]
    query particle state with BBG proof

soft3 query neuron <cid> [--root <bbg_root>]
    query neuron state with BBG proof

soft3 query axons <cid> [--out|--in] [--root <bbg_root>]
    query outgoing or incoming axons for a particle

soft3 verify <proof-file>
    verify a BBG Lens opening proof against a root

soft3 particle <file>
    compute the CID (hemera hash) of a file

soft3 node
    start a local development node

soft3 status [--node <addr>]
    show node status, BBG root, block height
```

## implementation

Rust binary. depends on:
- `hemera` (particle / CID computation)
- `bbg` (proof verification)
- `lens` (Lens opening verification)

not yet implemented — scaffold only. blocked on:
- BBG proof serialisation (serde for `QueryProof`)
- query wire protocol (`schema/`)
- network transport (`radio`)
