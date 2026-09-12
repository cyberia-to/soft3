---
title: soft3 specifications
tags: cyber, soft3, spec, foundation
crystal-type: spec
crystal-domain: cyber
alias: soft3 specs, stack contracts
---

# soft3 specifications

Start with the [execution model](execution-model.md), the foundational
execution architecture: reusable VM/OS support, open-ended compatible network
instances, explicit proof profiles and independently placed workers.

| specification | scope |
|---|---|
| [execution model](execution-model.md) | normative architecture; implementation coverage is reported separately |
| [native node adapter](native-node.md) | durable acceptance, request retry, HTTP bounds and legacy import |
| [terms](terms.md) | shared vocabulary |
| [types](types.md) | data/type model |
| [languages](languages.md) | language contracts |
| [filenames](filenames.md) | portable source-tree paths |

Soft3 owns composition invariants. Trident, warriors, proof systems and node
products own their specialized interfaces. [Foundations](../docs/README.md)
explains the substrate; [chains as plugins](../docs/chains-as-plugins.md)
discusses protocol adapters over it.

## report placement across the stack

Each repository keeps audits, implementation status reports, benchmark results
and release validation in its root `audit/` directory, grouped by subsystem
when needed. `specs/` and `reference/` define contracts, invariants, formats and
acceptance criteria. Mixed documents keep their contract here and move observed
results into an audit report, linked from the relevant contract or index.

Reports retain their checked revisions, dates, evidence and limitations.
Relocation updates relative links and graph paths together. A report's results
describe the recorded check; a directory move does not revalidate them.
