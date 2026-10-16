---
title: soft3 proposals
tags: cyber, soft3, proposal
crystal-type: process
crystal-domain: cyber
alias: proposals, soft3 proposals
---
# proposals

architecture changes that cross repositories are written here before they are built. a proposal fixes the picture, the moves, the acceptance criteria and the order; it closes on the **last merge**, not the first — all tests green in every repository it touches. the page then changes status and stays as the record.

<style>
.page-content .proposal-index { overflow-x: auto; }
.page-content .proposal-index table { display: table; width: 100%; }
.page-content .proposal-index th,
.page-content .proposal-index td { white-space: nowrap; overflow-wrap: normal; }
</style>

<div class="proposal-index">

| proposal | status | date | summary |
|---|---|---|---|
| [rooted civilization](rooted-civilization.md) | proposed | 2026-10-13 | remaining stack contracts |
| [one proof](proof-system-repair.md) | proposed | 2026-10-09 | compact post-quantum proofs |
| [term](term.md) | proposed | 2026-10-13 | programmable trading rights |
| [goldilocks chain](goldilocks-parano1d.md) | proposed | 2026-10-13 | proof-gated chain runtime |
| [consensus clock](steps-blocks-epochs.md) | proposed | 2026-10-12 | steps, blocks and epochs |
| [network planes](network-planes.md) | proposed | 2026-10-07 | sync, consensus and rewards |
| [tade](tade-one-exchange.md) | proposed | 2026-10-07 | one exchange format |
| [vault](vault-secret-custody.md) | superseded | 2026-09-16 | dedicated secret custody |
| [component pages](component-page-canon.md) | draft | 2026-08-12 | one canonical page each |
| [graph, sync, tade](cybergraph-sync-tade-architecture.md) | implemented | 2026-06-07 | graph and network boundaries |

</div>

statuses: **proposed** — agreed picture, not built · **draft** — still being argued · **implemented** — closed by its last merge · **superseded** — overtaken by a later decision, kept as the record.

the ladder these proposals shape is the front page of [soft3.org](https://soft3.org); the boundaries they respect are [[soft3/roadmap/component-boundaries|component boundaries]].
