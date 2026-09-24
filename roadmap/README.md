---
title: soft3 roadmap
tags: soft3, roadmap
status: active
---
# soft3 roadmap

Cross-component delivery projects live here. Contracts belong in `specs/`;
measured behavior, commands and source revisions belong in `audit/`.

| project | outcome |
|---|---|
| [Storage](storage/README.md) | One file identity, BBG-owned persistence, Radio transport and qualified recovery across consumers |
| [Component boundaries](component-boundaries.md) | Reconciliation, framing and cryptographic responsibilities across components |
| [Stack completeness](stack-completeness.md) | Coverage of the stack's wider architectural responsibilities |
| [Vocabulary and identity migration](migration.md) | Coordinated changes to the value model and names |

The storage project owns the implementation order for content persistence and
transport integration. Related roadmaps link to it rather than maintaining
independent completion claims for the same work.
