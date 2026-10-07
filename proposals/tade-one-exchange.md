---
tags: cyber, soft3, architecture, proposal, tade
crystal-type: process
crystal-domain: cyber
status: proposed
date: 2026-10-07
---
# tade — one exchange format, from the wire to the screen

> builds on [[soft3/proposals/cybergraph-sync-tade-architecture|cybergraph + sync + tade]] (implemented 2026-06), which made tade the wire substrate under radio and foculus. this proposal finishes the thought: tade is the **one** format data takes on every boundary of the stack — wire, the payload of a link inside a signal, a CLI's machine channel, the durable log, the stream a painter reads — and the serialization libraries the stack borrowed along the way go.

## the claim

the stack has one data model already: the nox noun (atom / pair), provable and canonical; [[neuron/specs/data|neuron data]] is encoded in it. tade today is a second structural format where it carries kv / tables, and postcard / serde_json are a third and fourth wherever a crate needed a quick codec. four truths for one value.

the resolution is a division of labour, not a winner:

```
noun   (nox)    computes      — the value, provable, canonical bytes
tade   (frame)  carries       — (sigil · render) + payload, text-safe, streamable, nestable
spark  (dialect) shows        — the catalog of the render byte: what a (sigil, render) pair renders to, in cells or pixels
prysm  (paint)  draws         — pixels / world units / terminal cells from spark's answer
```

one value, four roles, four components. a tade frame's structural payload **is** a canonical noun; tade adds exactly what the noun lacks — a type and a way to show — and nothing else.

## what changes

| # | step | where | size |
|---|---|---|---|
| 1 | **canon.** one value, one byte sequence: ordered keys (no `HashMap` on the read or write side), minimal LEB128, fixed child order, conformance vectors that pin every rule | tade spec + crate | one spec page, ~50 lines, vectors |
| 2 | **payload = noun.** the structural payload of a frame is the canonical nox noun encoding; tade frames of structured data and nox data are the same bytes under a 2-byte annotation | tade spec (one page on the binding), nox reference cross-link | one page |
| 3 | **catalog → spark.** the `(sigil, render)` vocabulary staged in `tade/spec/7-catalog.md` and `molecule.rs` is spark's: spark is the catalog of the render byte. prysm reads spark; it does not hold a vocabulary of its own | tade, spark, prysm | move + links |
| 4 | **proofs own their bytes.** zheng and lens stop encoding proofs and commitments with postcard — the bytes a verifier hashes must be defined by zheng's spec, not by a crates.io version. a proof on any boundary is a tade frame whose payload is zheng's own layout | zheng (12 postcard call sites), lens (6) | the real work of this proposal |
| 5 | **CLI: tty → ANSI, pipe → tade.** every CLI follows zheng's `tade_out`: styled text on a terminal, a tade chunk stream when piped, ending in a `STATUS` chunk. serde_json leaves the machine channel (≈130 call sites across bbg, foculus, neuron, cybergraph, soft3, cyb, lens, zheng, hemera); serde derives (≈60) go with it | all CLIs | mechanical, repo by repo |
| 6 | **ladder.** tade's chip moves from rung 7 (network) to rung 4 (cybergraph), beside `file` and `signal`: it is the bytes of a file and the payload of a link on any boundary. **done 2026-10-07** — the page already reads so | soft3.org/layers | — |

radio's upstream iroh messages (QUIC session chatter) stay postcard: that is transport noise, not stack data. the line is: **what has meaning is tade; what only moves bytes between endpoints is iroh's business.**

## why not the alternatives

- *noun everywhere, no tade.* a noun stream is not text-safe and not self-delimiting; it cannot be `cat`-ed into a terminal next to plain output or rendered incrementally. tade's `0x1F` marker never occurs in valid UTF-8 — that property is the whole reason a typed stream can live inside a terminal.
- *tade everywhere, no noun.* then provability and canon would have to be re-derived for a second structural format, and nox would compute over one thing while the graph stored another.
- *postcard / serde as the canon.* a borrowed codec's version becomes the definition of a proof's identity. unacceptable for anything a verifier hashes.

## acceptance

this proposal closes only when all of the following hold — not when the pages are written:

1. tade conformance vectors exist and pass in every crate that encodes or decodes frames (tade, foculus, cyb, prysm, rune, zheng cli, bbg cli);
2. `rg postcard` in zheng and lens returns nothing outside the vendored transport;
3. no CLI in the stack writes serde_json to a pipe; each has the tty / pipe split;
4. `cargo test` is green in **every** repository touched, and every PR is merged — the proposal is closed by the last merge, not by the first.

## sequencing

specification first (steps 1–3): small, and it fixes what the implementation must match. implementation (4–5) after the harness work in flight is finished — not before, and not interleaved with it. the chip move (6) is already live.

[[tade]] · [[nox]] · [[spark]] · [[prysm]] · [[cybergraph]] · [[zheng]] · [[lens]]
