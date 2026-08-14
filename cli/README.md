---
tags: cyber, soft3, cli
crystal-type: spec
crystal-domain: cyber
---
# soft3 CLI

`soft3` — command-line interface to the soft3 stack.

## commands

```
soft3 link <from> <to> [--token <particle>] [--amount <n>] [--valence +1|0|-1]
    construct and submit a cyberlink signal

soft3 query particle <particle> [--root <bbg_root>]
    query particle state with BBG proof

soft3 query neuron <particle> [--root <bbg_root>]
    query neuron state with BBG proof

soft3 query axons <particle> [--out|--in] [--root <bbg_root>]
    query outgoing or incoming axons for a particle

soft3 verify <proof-file>
    verify a BBG Lens opening proof against a root

soft3 particle <file>
    compute the particle (hemera hash) of a file

soft3 sync / soft3 network
    product probe and presets — default spacepussy-test @ :7780
    (shipped in soft3 0.4)

soft3 node
    start the product soft3-node for spacepussy-test
    (planned — milestone S6; see soft3/docs/launch.md)

soft3 status [--node <addr>]
    show node status, BBG root, block height
```

## launch

full operator guide: [[soft3/docs/launch|launch spacepussy-test]].

summary:

- product network is **spacepussy-test**, not cosmos space-pussy on cybernode
- today: local [[cybergraph]] + [[foculus]] components; `cyber sync` probes :7780
- target: `soft3 node` binds rpc/lcd/index on 7780–7782 and peers over [[radio]]

## implementation

Rust binary in `crate/`. network presets and `sync` are live.

remaining CLI surface (link / query / verify / node) waits on:

- BBG proof serialisation (serde for `QueryProof`)
- query wire protocol (`schema/`)
- soft3-node wiring (cybergraph + bbg + radio + foculus + soma)