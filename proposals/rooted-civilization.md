---
title: rooted civilization
tags: cyber, soft3, proposal, stack, civilization, lytics, vault, tok, foculus, mudra
crystal-type: process
crystal-domain: cyber
status: proposed
date: 2026-10-13
alias: rooted civilization, civilizational software, stack upgrade, what where when
---
# rooted civilization — the properties the stack still owes

> soft3.org sells a machine: collective, private, provable computing, thirty features, seven verbs. the market it actually enters is **rooted civilizational software** — the institutions a civilization can stand on when the cloud, the state and the vendor are gone. that market does not buy sumcheck. it buys identity that survives a death, a unit of account, a record a phone can check, a clock everyone shares, an upgrade that does not split the tribe, and boring operations. this page fixes the picture, names what the stack already holds at the same weight as the wall, brings the seven "critical and out of scope" questions of [[soft3/status]] into scope, and schedules the upgrade: **what, where, when.** it closes on the last merge, not the first.

the ladder these moves respect is the front of [soft3.org](https://soft3.org). the seats they land in are [[soft3/status|the registry]] and [[stack-completeness]]. they do not reopen [[proof-system-repair]], [[term]], [[goldilocks-parano1d]], or [[steps-blocks-epochs]] — those are the machine. this is the constitution the machine still lacks.

## 1. the picture

a **rooted** stack is grounded in physics (joules, light, silicon), in people (keys, names, inheritance), in a record that is readable in a hundred years, and in the right to leave (a fork). civilizational software is what a village, a ship, a city can run when the captured stack is dark.

civilization already runs on a captured stack: DNS and PKI and KYC for who; Visa and SWIFT for value; Linux and iOS and AWS for compute; Google and GitHub and Wikipedia for knowledge; TCP and SMTP for speech; NTP and GPS for time and place; closed models for mind. beside it sit the rooted products that already won a piece: Bitcoin, Linux, Git, Unicode, GPS/UTC, IPFS, Tor, Nostr, OSM, Nix, seL4. Urbit aimed at the same market and died on onboarding and isolation.

the buyer is not "a zkVM developer". the buyer is an **operator of an institution**: a citizen, a node, a town, an archive, a wallet, a school. the questions that buyer asks, in order:

1. it works when the other party is hostile
2. it outlives its authors
3. one person can run it
4. it speaks to the old world
5. there is a unit of account, and a reason to hold the bytes
6. operations are boring — uptime, backup, upgrade
7. a name a human can say
8. a key can be rotated, given, buried
9. a partition leaves local life intact
10. a fork is a constitution, not a catastrophe
11. outsiders can audit it
12. a child can learn it

Bitcoin closed 1–3 and part of 6; 8 is still weak. Linux closed 2–4 and 11. Unicode closed 7 forever. soft3 aims at all twelve, plus a mind and a state. that is the right scale. it is also the checklist this upgrade is scored against.

## 2. what the shop window already names

thirty tiles, one loop. unique as software features. honest on the home page (every number says measured or target). the `/layers/` line "~2 KB one proof" is older copy; [[proof-system-repair]] is the live goal (≤ 64 KB, ≤ 20 KB for a small statement). do not repeat the 2 KB claim.

| loop | tiles | what they are as software |
|---|---|---|
| compute | frozen foundations · post-quantum · deterministic · no halting · native silicon | a floor that will not move: one field, one hash, every program ends, the same byte on every device, jets on the chip |
| build | compact · trinity · cross-verified | a corpus one person can read; zk, FHE and nets as one arithmetic; three writers of every primitive |
| prove | succinct proofs · incremental proving · private by default · trusting trust | the proof rides with the run; the person is not in the block; the compiler proves it compiled itself |
| remember | memoization · provable reads · graph not ledger · home books · planet scale | the state is the graph; an answer carries a proof; a home book registers into the one |
| learn | explainable · aligned | weights are signed links; alignment is a distance of foci |
| link | names · referral · the wire · agentic | a name is a token; a route is graph content; an agent is a neuron |
| focus | collective focus · truth staking · resource markets · useful mining · fast finality · location proofs · interplanetary | φ* instead of a vote; honesty pays; miners sell order and agreement; where is the speed of light; facts commute across light-minutes |

measured today, among these: nox frozen-candidate, hash-only proofs live (full witness), deterministic replay of Bostrom's 2 949 732 links, bounded execution, honeycrisp on macOS + Android, 275k lines, differential harness by call tree, the wire on three nodes, κ = 0.74 on Bostrom. still specified or proposed: succinctness, incremental proving, privacy, self-host, provable reads, home books, planet-scale sampling, explainable/aligned models, referral, resource markets, useful mining, fast finality, location, interplanetary. foculus has never chosen a fork.

this wall is the **physics of a shared mind**. it is not the **institutions a civilization stands on**. the rest of this page is that second list.

## 3. properties the stack must match

not competitor features. the invariants of what already became civilizational.

| invariant | who already holds it | the match in this stack |
|---|---|---|
| physics | Bitcoin in joules; GPS in light | VDF, location proofs, useful mining, honeycrisp — and a **joule as a unit**, still unnamed |
| identity that outlives a body | passports, poorly; Bitcoin keys, poorly | [[vault]] is the first storage use-case; rotation, two-devices-one-neuron, death, guardianship — unbuilt |
| a record a phone can check | Git, Bitcoin | provable reads / 32-byte root — *building* |
| a small readable corpus | Linux against the LLVM wall | compact, 275k — and **several independent full nodes**, still one Rust line |
| names a human says | DNS, Unicode | names; 47 837 passports port into genesis |
| shared time | UTC, GPS | VDF is a clock of work. a human calendar, and a civil order of events that survives a blackout, are specified as interplanetary and not shipped as a daily product |
| the right to leave | Linux, Bitcoin | foculus parameters are derived; the procedure for *changing* them on a live network does not exist |
| a bridge into the old world | SMTP over UUCP; Bitcoin over USD; Linux over POSIX | `cargo install`, chaosnet, a Cosmos bootloader. HTTP, mail, BTC, cash, paper — no seat |
| boredom | every network that lived | [[lytics]] is live; operations as protocol citizens are not |
| several implementations | Bitcoin, TLS | rust + trident agree on primitives; a second full node is not a product |

## 4. already in the stack, equal weight to the wall

these are not missing. they are **not on the shop window**, and the market buys them as the goods.

| already held | where | why it weighs with succinct proofs |
|---|---|---|
| **measurement** | [[lytics]], live | [[soft3/status]]: the strongest single-node evidence — cybergraph + bbg + hemera + mudra + inf under adversarial traffic. cybernetics without measurement is decoration |
| **the bootloader memory** | [[bootloader]], [[bostrom]] snapshot | 2.9M links, 3.1M particles, 47k names. rooted means *already rooted in a past*. a white-sheet ontology is a different product |
| **the money** | [[$CYB]], [[best-money]] | origin, scarce 18m, private, named, light, inference as the fourth resource. civilizational software without a unit of account is a library. soft3.org almost does not sell it |
| **publishing** | [[optica]] | a book that cannot be read as a site does not enter a culture. cyber.page is the web of the graph |
| **warriors** | [[erga]], zoya, mona, xena | energy turned into useful work. the money/PoW market understands this better than a *proposed* tile |
| **recovery after device loss** | [[vault]], spec + repo | for rooted identity this outranks a 64 KB proof. absent from the thirty |
| **conformance / kelvin** | [[soft3/specs/releases]], frozen foundations | a release is a frozen snapshot. that is how Nix and Bitcoin-consensus became civilizational |
| **the wire as graph content** | [[cy]] `wire`, [[radio]] | live, three nodes. closer to a civilizational TCP than trinity is. on the wall, drowned among twenty-nine neighbours |
| **silicon** | [[honeycrisp]] | a type-2 stack that does not touch the chip is a cloud. this is soil, not an optimisation |
| **land and deed** | [[cyberia]], hak sewa | the state layer exists beside the machine and is not woven into the machine's features. the market includes land and money |

acceptance for this section is a shop-window change, not new crates: these ten are named on [soft3.org](https://soft3.org) at the same rank as the thirty, or the thirty are re-cut so these are among them. the `/layers/` 2 KB line is retired.

## 5. forgotten — now in scope

[[soft3/status]] listed seven questions no roadmap owned. this proposal owns them. they stay critical. they stop being out of scope.

| # | question | what | where | when |
|---|---|---|---|---|
| 1 | economics of writing | who may write, at what cost; an open graph with free writes drowns on day one | [[tok]] live economy; the [[bostrom]] → genesis bridge; lytics' 0.042 s PoW is one app's local answer, not the network's | A |
| 2 | key lifecycle | recover, rotate, revoke; two devices are one neuron; a lost seed is not a lost citizen | [[vault]] recovery; [[mudra]] rotation/revocation; neuron as a person across devices | A |
| 3 | protocol upgrade | the procedure for changing foculus parameters on a running network; a hard fork of a φ*-converged graph is defined behaviour | [[foculus]] upgrade spec; a fork as a user right (replay of a home book, exit of a domain) | B |
| 4 | information is legal | there are no illegal bytes; the record keeps everything, a node chooses what it holds and serves, liability sits with the actor | a settled stance written as a page with an owner; no `censor`, no `tombstone` verb — ever; replication follows focus (#5) is the only seat | settled · page in B |
| 5 | storage economics | a reason to hold the bytes; the filecoin-shaped hole | [[bbg]] × [[tok]] duration × popularity; replication-follows-focus as a paying market | A |
| 6 | adversarial audit | someone paid to break it, before real value moves | budget and a named campaign against mudra, the truth-serum, the consensus theorem | B, starts when A money moves |
| 7 | operations | health, metrics, alerting as protocol citizens | [[lytics]] extended from visitors to nodes; a `health` signal on the wire | A |

further questions the status page did not ask, and the market still asks:

| property | what | where | when |
|---|---|---|---|
| capture of φ* | focus is a government. truth staking is incentive-compatible against unilateral deviation; collusion is open. name anti-capture as a feature: sybil on links, purchase of attention, censorship through stake | [[tru]] / [[foculus]] | B |
| joule as a unit | type-2 is a civilization of energy. VDF and mining exist. a balance in watts, "this answer cost this much light", does not | [[solong]], [[honeycrisp]], a resource dimension | C |
| hardware plurality | native silicon is live on macOS + Android. a civilizational stack that will not build on RISC-V / ordinary Linux without AMX is rooted in one vendor | [[honeycrisp]] ports; [[nox]] jets without AMX | C |
| a bridge into the captured world | names, money, proofs enter email, a browser, BTC, a bank, paper | a `bridge` seat; optica already publishes; money and identity still stop at chaosnet | C |
| compaction of history | 10¹⁵ particles without forgetting cannot be held. archive + compaction + "cold and still proven" | [[bbg]] storage tiers, already specified; a paying forget | C |
| a second full node | durable money in [[best-money]] wants independent implementations | a second language on the conformance snapshot | C |
| guardianship | machines, animals, children, organisations are in the cyberia thesis and not in the features: guardian, quorum, spend limits | [[vault]], [[neuron]], [[term]] | B, with [[term]] |
| a human entrance | `cargo install soft3` is an entrance into a compiler. civilizational software installs as an app, a key, a name, a book | [[cyb]] shell speaking `cy` with mudra keys — already named as the next milestone after the wire | A |
| human languages | i18n, literacy, a child's path | docs and cyb, not a new crate | C |
| a deed a court can read | a Goldilocks proof is not a notarial act | [[cyberia]] legal layer; a rendering of a graph into an act | C |
| offline of a day, not of Mars | interplanetary is specified. a pocket without a network for a day is a product that must be boring and live | home books + radio store-and-forward | B |
| fork as a button | exit right, replay of one's book, leaving a domain | [[foculus]] + home books | B |

## 6. the upgrade — three constitutions

four phases. each phase is a last-merge: the tests of every repository it names are green, and the shop window says measured only of what is measured.

```text
   shop window          A person               B network              C world
   (docs, site)         (key, write, hold)     (upgrade, capture,    (joule, iron,
                                                fork, law)             bridge, court)
         │                    │                      │                      │
         ▼                    ▼                      ▼                      ▼
   the ten of §4      status 1,2,5,7          status 3,4,6            plurality,
   on the wall        vault · tok · bbg       foculus · tru           honeycrisp
                      cyb entrance            sigma/tombstone         bridges · compaction
                                              term guardianship       second node
```

### phase 0 — the visor (now)

**what.** the shop window matches the stack. the ten of §4 sit at the same rank as the thirty, or the thirty are recut to include them. `/layers/` drops the 2 KB line. every number on the site already tagged measured/target stays that way.

**where.** `soft3/site/` (`index.html`, `layers/index.html`); this proposal; a pointer from [[soft3/status]] § critical.

**when.** this merge. no runtime.

**acceptance.** a reader of soft3.org can name measurement, the bootloader memory, the money, publishing, vault, and the wire without opening GitHub. the 2 KB claim is gone.

### phase A — the constitution of the person

the network may not take real value until a person can lose a phone and remain a person, and until writing and holding cost something.

**what.**

1. vault recovery after device loss, as specified in the vault repo, wired through [[mudra]]
2. key rotation and revocation; one person, two devices, one neuron
3. live write pricing (status #1) — tok in the loop, genesis bridge from Bostrom
4. live storage market (status #5) — paying holders; replication follows focus
5. node health as a signal (status #7)
6. the cyb shell speaking `cy` with mudra keys (already the named next milestone of the wire)

**where.** [[vault]] · [[mudra]] · [[neuron]] · [[tok]] · [[bbg]] · [[lytics]] · [[cyb]]

**when.** in parallel with [[steps-blocks-epochs]] P1 (the chain machine) and [[term]]'s first settlement, because write-price and rights share tok. not blocked on succinctness.

**acceptance.**

- lose the device, recover the neuron, the name and the home book
- two devices sign as one neuron
- an unpriced write is rejected
- a holder of un-focused bytes is paid, or the bytes leave
- a sick node is visible on the wire as a particle, not only as a launchd log
- a non-compiler install produces a name and a book

### phase B — the constitution of the network

**what.**

1. upgrade procedure for foculus parameters; a fork of a φ*-converged network is defined (status #3)
2. fork as a user right: replay a home book, leave a domain
3. the stance "information is legal" written as a page with an owner (status #4): no censor or tombstone verb, a node's choice of what it holds is local, liability belongs to the actor
4. anti-capture of φ* as a specified property; collusion in truth staking has a next proof or a named bound
5. adversarial audit budget and campaign (status #6)
6. guardianship (with [[term]]): quorum, spend limits, animals and organisations as neurons under a guardian
7. offline of a day: home book lives, store-and-forward catches up

**where.** [[foculus]] · [[tru]] · [[sigma]] · [[term]] · [[vault]] · [[radio]]

**when.** after A's write-price and vault, because an upgrade of a valueless graph is theatre, and after [[steps-blocks-epochs]] has a clock, because "change the parameters" needs a height to change at.

**acceptance.**

- a parameter change has a document, a height, and a replay
- a neuron leaves a domain with its book intact
- the information-is-legal page exists with an owner; no deletion verb ships, ever
- an audit report exists from someone paid to break mudra / BTS / the consensus theorem
- a guardian neuron can spend, and can be removed, under a term
- a pocket dark for 24 h converges without manual surgery

### phase C — the constitution of the world

**what.** joule as a unit; honeycrisp/nox on ordinary Linux and RISC-V; bridges into HTTP, mail, BTC, paper; compaction of history that stays provable; a second full node on the conformance snapshot; i18n; a deed a court can read.

**where.** [[honeycrisp]] · [[solong]] · [[optica]] · [[bbg]] · [[soft3/conformance]] · [[cyberia]]

**when.** after B's upgrade rule (a second implementation is a fork-risk until forks are defined) and after [[proof-system-repair]] has a stable proof format (bridges and light courts verify something frozen).

**acceptance.**

- a proof, a name and a payment are each expressible in one old-world channel
- a node builds and proves on a machine without AMX
- an energy receipt exists for a computation
- a cold tier forgets bytes and still answers a root
- a second implementation verifies the same conformance snapshot
- a graph fragment renders as a deed

## 7. order against the other proposals

```text
now     phase 0 visor
        │
        ├─[[proof-system-repair]] ──────────── (machine: the proof)
        ├─[[steps-blocks-epochs]] ──────────── (machine: the clock)
        ├─[[term]] ─────────────────────────── (machine: rights)
        ├─[[goldilocks-parano1d]] ──────────── (machine: the chain port)
        │
A       person: vault · tok · storage market · health · cyb entrance
        │         (shares tok with term; does not wait for 64 KB)
        ▼
B       network: upgrade · fork-right · tombstone · anti-capture · audit · guardian · day-offline
        │         (needs A's value and the clock's height)
        ▼
C       world: joule · iron plurality · bridges · compaction · second node · court
              (needs B's fork rule and a frozen proof format)
```

phase 0 is documentation. A is the first runtime this proposal owes. B and C are not a licence to grow new repos for their own sake: every row in §5 names a seat that already exists, except the optional `bridge` seat in C, which starts as optica + a money/identity adapter, not as a blank organisation.

## 8. what this page does not do

- it does not change the thirty's physics. frozen, PQ, deterministic, no-halt, silicon, compact, trinity, cross-verified, proofs, privacy, memoization, graph, names, wire, φ* stay.
- it does not claim a 2 KB proof, a live foculus fork-choice, or a shipped vault.
- it does not replace [[best-money]]; it demands that the money appear on the machine's visor.
- it does not design the tombstone, the joule ticker, or the RISC-V port — it seats them and dates them.

## 9. the verdict

the thirty tiles correctly sell a rare machine. the market of rooted civilization also buys **inheritance, upgrade, boredom, a bridge, a joule, and the right to leave**. some of that already sits in the stack (lytics, the bootloader, CYB, optica, vault, the wire) at the same weight as the wall, and is invisible on the visor — phase 0. some of it (a key after death, the economics of writing, capture of focus, iron that is not Apple, ops) is absent from both the visor and the protocol. without it soft3 remains a great compiler. with it, it is a root.

[[soft3/status]] · [[stack-completeness]] · [[vault]] · [[best-money]] · [soft3.org](https://soft3.org)
