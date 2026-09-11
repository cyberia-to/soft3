---
title: soft3 specifications
tags: cyber, soft3, spec, foundation
crystal-type: spec
crystal-domain: cyber
alias: soft3 specs, stack contracts
---

# soft3 specifications

Start with [warriors, workers and networks](warriors.md), the foundational
execution architecture: reusable VM/OS support, open-ended compatible network
instances, explicit proof profiles and independently placed workers.

| specification | scope |
|---|---|
| [warriors, workers and networks](warriors.md) | normative architecture; implementation coverage is reported separately |
| [terms](terms.md) | shared vocabulary |
| [types](types.md) | data/type model |
| [languages](languages.md) | language contracts |
| [filenames](filenames.md) | portable source-tree paths |

Soft3 owns composition invariants. Trident, warriors, proof systems and node
products own their specialized interfaces. [Foundations](../docs/README.md)
explains the substrate; [chains as plugins](../docs/chains-as-plugins.md)
discusses protocol adapters over it.
