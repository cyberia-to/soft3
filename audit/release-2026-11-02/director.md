---
title: the director's day — 2026-11-02
tags: soft3, release, director, audit
date: 2026-11-02
---
# the director's day — 2026-11-02

Written per `release/director/README.md` § the measurement. Sessions append to it, and each number names its run or PR.

**The code and feature freeze holds from 00:00 UTC** (`cyber/launch.md`; `plan.md` § what phase 1 is). Until pussy's genesis attempt on 2026-11-05 13:37:00 UTC, only security and bug fixes that come with a failing test merge. Gate and test work on `release/phase1` and its siblings' release branches counts as such; nothing new is started.

## the candidate

**Red.** No train has run on the defaults since [41097270082](https://github.com/cyberia-to/soft3/actions/runs/41097270082) (`schedule`, `main`, 2026-10-30), which is red at package-resolution until [lens#36](https://github.com/cyberia-to/lens/pull/36) lands (L2). The newest rehearsal is [41367974273](https://github.com/cyberia-to/soft3/actions/runs/41367974273) (`feat/p4a-cached-boot`, 00:35 UTC, conclusion `failure`). It is red only on phase1-pins (G9) and release-notes-source (G8) on all five legs, and is no worse than [41354213746](https://github.com/cyberia-to/soft3/actions/runs/41354213746) (sessions 517, 518, 520). On its stack leg radio and simulation-gate are green, `stack-pussy-identity` boots the built `pussy.tade`, and `stack-chain-genesis` reproduces `pussy-rc.tade` byte for byte (sha256 `d20bab4b…76f5`). G8 and G9 wait on the owner's landings.

**The stand** (pussy-rc, agent-a's peer, `http://127.0.0.1:7783/status` at 02:42 UTC, session 524): height 2097 = `best-header` 2097, `finalized` 1977, `epoch` 9, `last-settled` 8 (7 at 02:14), `peers` 1 / `outbound` 1, `conserved` `settlement-ok` `stake-keys-match` all true, `genesis` `5614749e…b713` = `[pussy-rc]`. 8 records in `state/pussy-rc/epochs`.

## units done

- session 513: **C4a done.** simulation#2 `c312ab9` is green under `--locked` in diagnostic [41362852433](https://github.com/cyberia-to/soft3/actions/runs/41362852433). Merging it is the owner's, after pussy's genesis (D15).
- sessions 514–515: P4a rewritten against cybernode `rs/`. Rehearsal 41365724425 was cancelled because of a short-SHA override, and was re-dispatched as [41367974273](https://github.com/cyberia-to/soft3/actions/runs/41367974273). G19 opened.
- session 516: **G19 done.** [soft3#623](https://github.com/cyberia-to/soft3/pull/623) merged as `2e3a050` (gate 41369692758).
- sessions 517–519: 41367974273 read leg by leg, no worse than the baseline ([#595 comments 6566719540, 6566922925](https://github.com/cyberia-to/soft3/pull/595)).
- session 520: **P4a done.** cybernode#8 merged as `645ac7f`, [soft3#595](https://github.com/cyberia-to/soft3/pull/595) as `ef69e2f`, and the pin [soft3#628](https://github.com/cyberia-to/soft3/pull/628) as `b2694da`.
- session 521: **P3b (a′) done.** The rehearsal file reproduces under the pin ([#589 comment 6567411837](https://github.com/cyberia-to/soft3/pull/589#issuecomment-6567411837)).
- session 522: **P3b (b) and P3a recorded done.** The owner's 2026-11-01 20:11 UTC report on [#493](https://github.com/cyberia-to/soft3/pull/493) was verified on agent-a; sessions 513–521 had missed it. P3c opened.
- session 523: **P3c done.** cybernode#9 merged as `f1065eb` (red [41377369646](https://github.com/cyberia-to/cybernode/actions/runs/41377369646) → green [41377422279](https://github.com/cyberia-to/cybernode/actions/runs/41377422279)), and the pin [soft3#632](https://github.com/cyberia-to/soft3/pull/632) as `8870bb5`.
- session 524: **G11 upkeep.** This report had not been opened, and the 2026-11-01 report stopped at session 442. That report is now closed from the plan's log and `gh search prs`. No other row had director-side work (see below).
- session 525: **idle, recorded.** The tick is unchanged (`changed=0`), and the owner has not commented or reviewed since 02:30 UTC. Every open PR into `release/phase1` is the owner's decision. The stand reads height 2139, `conserved`, `settlement-ok` at 03:02 UTC.
- session 526: **S1 (e) doing.** [soft3#637](https://github.com/cyberia-to/soft3/pull/637) was opened (rustls 0.23.46, GHSA-vp88-798v-h67f, lockfile only), and rehearsal [41381272502](https://github.com/cyberia-to/soft3/actions/runs/41381272502) was dispatched as its gate.
- session 527: **waiting on 41381272502.** All five legs were `in_progress` at 03:23 UTC. The owner has been silent since 03:00 UTC, and edge-a is still unreachable (`000`). The stand reads height 2178, epoch 10, `last-settled` 8, `conserved`, `settlement-ok` at 03:24 UTC.
- session 528: **#637 reviewed, three of five legs read.** The three finished legs of [41381272502](https://github.com/cyberia-to/soft3/actions/runs/41381272502), which built soft3 `4f8a667`, are red only on the baseline pair, and soft3-tests is green. The intel leg and the stack leg are still running, so #637 waits on them.
- session 529: **still waiting on 41381272502.** The intel leg and the stack leg were in `Qualify stack and product` at 03:58 UTC, and neither had failed. The stand reads height 2254, epoch 10, `last-settled` 8, `conserved`, `settlement-ok`. The epoch-9 record has been on disk since 03:12 UTC, and the chain applies it at `settle(9)` = 9·240 + 120 = height 2280, so the lag is the settle delay (`params.rs:133`).
- session 530: **S1 (e) done.** [soft3#637](https://github.com/cyberia-to/soft3/pull/637) merged into `release/phase1` as `ca86451` (rustls 0.23.46, lockfile only). The intel and stack legs of [41381272502](https://github.com/cyberia-to/soft3/actions/runs/41381272502) were compared gate by gate with 41367974273: intel 8 of 8 and stack 26 of 26 are equal, and the only reds are the known ones. Dependabot alert #6 closes when R0 brings the lock to `main`.
- sessions 531–536: **idle, recorded** (plan log).
- session 537: **C2a carried to cybernode**: [cybernode#10](https://github.com/cyberia-to/cybernode/pull/10) opened, rehearsal [41400293113](https://github.com/cyberia-to/soft3/actions/runs/41400293113) dispatched as its gate.
- session 538: **P5 carried to cybernode**: [cybernode#11](https://github.com/cyberia-to/cybernode/pull/11) opened.
- sessions 539–540: **C2a**: the Linux and macOS aarch64 legs of 41400293113 were read onto cybernode#10, and it holds on all three (+1 test, `body_malleation_never_invalidates_the_honest_header`).
- session 541: **idle**, waiting on 41400293113's last two legs.
- session 542: **P9 spec half**: [cybernode#12](https://github.com/cyberia-to/cybernode/pull/12), revert by release (`specs/chain.md` §5.6 draft, `decision` `wide`, for the owner; D19). Recommended: (b), a revert table compiled into the release. Rejected: (a) miner signals and (c) a header era.
- session 543: **C2a to the owner.** All five legs of [41400293113](https://github.com/cyberia-to/soft3/actions/runs/41400293113) were read onto [cybernode#10](https://github.com/cyberia-to/cybernode/pull/10): 323 / 0 against the baseline's 322 / 0, and the red sets are unchanged.
- session 544: **P9b opened**: [cybernode#13](https://github.com/cyberia-to/cybernode/pull/13), the revert-by-release machinery with an empty table (`wide`, sonnet subagent, reviewed by the director). Rehearsal 41411831611 was dispatched.
- session 545: **41411831611 void**: it ran with a lone `cybernode` override, so soft3 `main` and lens `master` were built and every leg died in package-resolution. It was re-dispatched with the stand's full override set as [41414017957](https://github.com/cyberia-to/soft3/actions/runs/41414017957).
- session 546: **41414017957, leg 1 of 5 (Linux x86_64)**: [cybernode#13](https://github.com/cyberia-to/cybernode/pull/13) holds: 336 / 0 against the baseline's 323 / 0, the new revert tests are green, and the reds are only the baseline pair. The other four legs are still running.
- sessions 547–557: see the plan's log. **P9b done**: cybernode#13 merged as `1d126da` (gate 41423930605). **P9c done**: cybernode#15 merged as `599cd85` (gate 41448682465).
- session 558: **P9d doing**: [cybernode#16](https://github.com/cyberia-to/cybernode/pull/16) (the owner's key in pussy's and pussy-rc's genesis) opened, and measuring rehearsal [41464171130](https://github.com/cyberia-to/soft3/actions/runs/41464171130) dispatched.

## PRs merged

So far 25 across `cyberia-to` (`gh search prs --owner cyberia-to --merged-at 2026-11-02`, 04:30 UTC), plus this session's plan PR:

| repository | PRs | |
|---|---|---|
| soft3 | 23 | 18 plan PRs (#620–#622, #624–#627, #629–#631, #633–#636, #638–#641) and 5 work PRs: #595, #623, #628, #632, #637 |
| cybernode | 2 | #8, #9 |

All went into release branches. Nothing was merged into a default branch.

## plan items left

Every open row now waits on the owner (`plan.md` § what only the owner does):

- **P9** (D19, revert in the protocol): the owner decides [cybernode#12](https://github.com/cyberia-to/cybernode/pull/12). The director's next unit is P9b, the machinery with an empty table, which does not wait on that decision.

- **P3b (c)**, epoch records from both peers for 24 h, closing **2026-11-02 19:47 UTC**. Edge-a's half is not reachable from agent-a: `cyb.ai/pussy-rc/` serves the site, and edge-a's HTTP is bound to loopback.
- **P5**: soft3#572, the §2.9 draft. **P2**: soft3#479, the halt sentence.
- **The landings**: soft3#376, lens#36, hemera#25, nox#43, radio#52. Also the asks that lapsed on 2026-10-29 (L3b, L3c, L3d, L4, C2a, G10b). Under D10 their silence cuts nothing, and none of them is a freeze-eligible fix the director can take.
- **R1** follows the landings. **G8** and **G9** turn green with them.

Session 537 found both of the director's owner-facing PRs into soft3 `release/phase1`, #350 (C2a) and #572 (P5), `dirty`: C4 moved the chain code and `specs/chain.md` to cybernode. C2a is carried as [cybernode#10](https://github.com/cyberia-to/cybernode/pull/10) with its blobs identical to #350, and gated by rehearsal [41400293113](https://github.com/cyberia-to/soft3/actions/runs/41400293113). #572 is next.

## fable and windows

Fable questions asked: 1 (session 542, P9: agreed with (b) and added five rules). Sessions: 34 logs so far, this one included (`state/session-20261102T*.log`). Subscription windows hit: none.
