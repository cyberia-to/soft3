---
title: soft3
tags: cyber, soft3, sdk
crystal-type: spec
crystal-domain: cyber
icon: "👙"
alias: soft3 stack, cyb stack, software stack, proof pipeline
---
start with the deal the current web offers you. everything you know lives on someone else's servers: you write — the platform owns it. you teach an AI — the company sells it back to you. and when the machine answers, there is no way to check where the answer came from. give your knowledge away, then trust it and rent it back — that is the deal.

soft3 is the software stack built to break it. underneath everything sits one shared graph: whatever you say becomes a signed, staked [[cyberlink]] between [[particles]] of content, and whatever the system computes comes back with a [[cryptographic proof]] attached. one trick makes this work — in soft3, data, identity, state and proof are the same mathematical object, so proving is not an extra step bolted on top. it is simply how the machine runs.

that one trick changes what you can own. your knowledge stops being a post on a platform and becomes property: nobody can delete it, it composes with everyone else's, and when the attention of others flows through your links, you earn. your agents become the second half of your mind — they read, link and prove in your graph while you sleep, and instead of trusting them you verify every step they took. and truth itself gets an economy: claims are staked, so being right compounds and lying burns.

because the whole stack is just a dependency, you are never merely a user in it. import soft3 and build your own world — a game, a city, a market, a science guild — and it ships with identity, memory, money and truth out of the box, connecting to every other world by a link instead of an API. [[cyberia]] is the first world built this way. yours is next.

the first step takes a minute:

```bash
cargo install soft3
soft3 sync   # → space-pussy @ https://rpc.space-pussy.cybernode.ai
```

this puts a live slice of the graph under your fingers. space-pussy is a chaosnet — everything on it is test, [[rewards]] included — so play fearlessly. full guide: [[install]]

when you are ready to go deeper: the stack is seven triads of components, each carrying one verb — hash, link, store, transmit, converge, prove, embody. the full registry lives at [[soft3/stack]]; the ideas behind it — one mind, many languages, open world — at [[soft3/docs|the whitepaper]]. the language you speak by linking is [[neural]]. specs: [[soft3/specs/languages|languages]] · [[soft3/specs/types|types]] · [[soft3/specs/terms|terms]] — roadmap: [[component-boundaries]] · [[stack-completeness]] · [[terms-map]] — SDKs: [js/](js/) · [cli/](cli/) · [mcp/](mcp/) · [py/](py/) · [schema/](schema/)
