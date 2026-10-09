---
title: soft3 proposals
tags: cyber, soft3, proposal
crystal-type: process
crystal-domain: cyber
alias: proposals, soft3 proposals
---
# proposals

architecture changes that cross repositories are written here before they are built. a proposal fixes the picture, the moves, the acceptance criteria and the order; it closes on the **last merge**, not the first — all tests green in every repository it touches. the page then changes status and stays as the record.

| proposal | status | date | one line |
|---|---|---|---|
| [one proof — the repair of zheng](proof-system-repair.md) | proposed | 2026-10-09 | goal ≤ 64 KB · post-quantum · verify ≤ 1 ms · constant size: one field, one hash, Reed–Solomon + WHIR, Spartan over CCS, ARC accumulation; what was broken and why 2 KB was never real; five phases with acceptance; §9 the Merkle question (why trees are a lower bound for hash-only, the attack on the Merkle-free design, the levers to 15–20 KB); §10 lattices — what they buy (the accumulator) and what they do not (bytes) |
| [the network in planes](network-planes.md) | proposed | 2026-10-07 | foculus → `sync` · `foculus` · `solong` in one workspace, one arrow; radio keeps its fork and hands the reconciliation cores to sync; the epoch certificate stops carrying the payout |
| [tade — one exchange format](tade-one-exchange.md) | proposed | 2026-10-07 | one format from the wire to the screen: payload is the canonical nox noun, tade annotates, spark is the catalog of the render byte, postcard and serde_json leave the stack |
| [vault has its own home](vault-secret-custody.md) | superseded | 2026-09-16 | vault became a repository (now an organ of [[cy]]) |
| [one component, one page](component-page-canon.md) | draft | 2026-08-12 | every component answers at one canonical address |
| [cybergraph + sync + tade](cybergraph-sync-tade-architecture.md) | implemented | 2026-06-07 | cybergraph as the one API, foculus owns structural sync, tade is the wire substrate |

statuses: **proposed** — agreed picture, not built · **draft** — still being argued · **implemented** — closed by its last merge · **superseded** — overtaken by a later decision, kept as the record.

the ladder these proposals shape is the front page of [soft3.org](https://soft3.org); the boundaries they respect are [[soft3/roadmap/component-boundaries|component boundaries]].
