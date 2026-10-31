---
title: the director's day — 2026-10-31
tags: soft3, release, director, audit
date: 2026-10-31
---
# the director's day — 2026-10-31

per `release/director/README.md` § the measurement. Sessions append. Each number names its run or PR.

## the candidate

**Red.** The newest train run on the defaults is [41097270082](https://github.com/cyberia-to/soft3/actions/runs/41097270082) (`schedule`, `main`, 2026-10-30 17:15 UTC). Every leg is red at package-resolution: `cyber-lens-gpu` fails with `failed to read …/sources/lens/gpu/Cargo.toml`, because lens `master` lacks the 0.2 line until [lens#36](https://github.com/cyberia-to/lens/pull/36) lands, and that landing is the owner's approval. The other landing PRs are also the owner's: hemera#25, nox#43, radio#52, soft3#376.

On pussy's pinned stand (`release/pussy-stand.toml`), rehearsal [41041110233](https://github.com/cyberia-to/soft3/actions/runs/41041110233) gives these results:

| leg | soft3-tests | soft3-release | node-status | reds |
|---|---|---|---|---|
| stack | green | green | green | phase1-pins, release-notes-source, stack-zheng (L3b), stack-mudra (L3c), stack-neuron (L3c) |
| x86_64-unknown-linux-gnu | green | green | green | phase1-pins, release-notes-source |
| aarch64-unknown-linux-gnu | green | green | green | phase1-pins, release-notes-source |
| aarch64-apple-darwin | green | green | green | phase1-pins, release-notes-source |
| x86_64-apple-darwin | green | green | green | phase1-pins, release-notes-source |

On the stack leg, stack-chain-genesis, conformance-snapshot and simulation-gate are also green.

Since session 400 the stand is pinned at soft3 `b6f22dc` (P2 and P2a, [#489](https://github.com/cyberia-to/soft3/pull/489)). Rehearsal [41201610001](https://github.com/cyberia-to/soft3/actions/runs/41201610001) on that stand holds on all five legs. Its overrides equal the stand 16 of 16, and soft3-tests, soft3-release and node-status are green on every leg. The platform legs are red only at phase1-pins (G9) and release-notes-source (G8). The stack leg is red at those two and at stack-zheng (L3b; at this revision it also times out at 1800 s), stack-mudra and stack-neuron (L3c), the same set as P2's baseline 41192673416 (sessions 402, 405, 408).

## units done

- session 394: **P1** done. The rehearsal's overrides equal the stand file, 16 of 16, and its acceptance gates are green on all five legs. The remaining reds have rows (G8, G9, L3b, L3c).

- session 395: **P2**: soft3#479 merged into `release/phase1` as `bce863b`. Red diagnostic 41042358656 fails on the hole, and green 41042360873 has lib 294/0 with `warnings: 0`. The all-legs rehearsal [41192673416](https://github.com/cyberia-to/soft3/actions/runs/41192673416) is dispatched.

- session 396: **P2a**: release `overflow-checks` diagnostic [41192950342](https://github.com/cyberia-to/soft3/actions/runs/41192950342) dispatched (baseline and overflow legs on pussy's stand). P2's rehearsal is still running.

- session 397: **P2a**: diagnostic 41192950342 green on both legs (soft3 4/4, tru 1/1 with the flag). [soft3#484](https://github.com/cyberia-to/soft3/pull/484) opened with the release profile.

- session 398: **P2a** done: soft3#484 merged into `release/phase1` as `b6f22dc` after gate [41194300832](https://github.com/cyberia-to/soft3/actions/runs/41194300832) went green.

- session 399: **P2**: the four platform legs of rehearsal [41192673416](https://github.com/cyberia-to/soft3/actions/runs/41192673416) read. `a_settlement_panic_is_a_named_halt` passes and soft3-tests, soft3-release and node-status are green on all four. The only reds are phase1-pins and release-notes-source. The stack leg is still qualifying.

- session 400: **P2** done. The stack leg of [41192673416](https://github.com/cyberia-to/soft3/actions/runs/41192673416) is green on soft3-tests, soft3-release, node-status and simulation-gate, `a_settlement_panic_is_a_named_halt` passes, and the reds are the stand's known set. [#489](https://github.com/cyberia-to/soft3/pull/489) bumped the stand's soft3 to `b6f22dc` (merged as `89d9c24`).

- session 401: **P2b**: no scheduled train runs today, because `main` schedules Fridays only (`0 12 * * 5`). So the stand rehearsal [41201610001](https://github.com/cyberia-to/soft3/actions/runs/41201610001) was dispatched on the stand as merged.

- session 402: the owner's plan correction [#488](https://github.com/cyberia-to/soft3/pull/488) (P3, the peers leave Hetzner) landed through [#492](https://github.com/cyberia-to/soft3/pull/492), which resolved its conflict and kept the owner's commit `f6c9fc5`. **P2b**: x86_64-linux, aarch64-linux and aarch64-darwin are green, red only at G8 and G9.

- session 403: **P3** doing: [#493](https://github.com/cyberia-to/soft3/pull/493) merged as `feb3d18`. It adds the two-host stand (cyberproxy plus agent-a as a dial-out peer and miner), the agent-a tmux supervisor, `soft3-chain-check` without systemd, and README § E. Review fixed one defect: the miner was not signalled. Measured: agent-a is behind NAT and has no SSH to the Hetzner hosts, so cyberproxy's unit and endpoint id and § E are the owner's.

- session 404: **P3 (a)** done. agent-a's binary is installed from 41201610001's `candidate-x86_64-unknown-linux-gnu` (soft3 `b6f22dc`, executable sha256 `b5f85028…`, equal to the receipt's `SHA256SUMS`). `soft3-chain-check --pre pussy-rc` returned rc 0, and nothing is started. Finding P3d: the check passed an empty peer id.

- session 405: **P2b**: the x86_64-apple-darwin leg is green, red only at G8 and G9. P2a's release `overflow-checks` is now green on all four platform legs.

- session 406: **P3d** done: [#497](https://github.com/cyberia-to/soft3/pull/497) merged as `c16fa90`. On agent-a the check now says `DEGRADED … skipped (empty id): cyberproxy`, which is the true state until P3 (b). A sonnet subagent implemented it.

- session 408: **P2b** done. The stack leg of 41201610001 is read (see the candidate above), and the stand at `b6f22dc` holds on all five legs. Session 407 (14:11 UTC) ended with no plan PR.

- session 409: **P3e** done: [#500](https://github.com/cyberia-to/soft3/pull/500) merged as `5580cb7`. The probe is DOWN for every peers file `parse_peers` refuses. It was checked against a 20-case matrix on agent-a, and the merged copy is installed there. A sonnet subagent wrote `687f86b`, and review fixed one defect (`981776a`). The 14:46 session ended with no plan PR.

- session 410: idle, on an unchanged tick. Every open row waits on the owner.

- session 411: **C4** measured and held to after pussy's genesis, decision D15. [#407](https://github.com/cyberia-to/soft3/pull/407) has 8 conflicts against `release/phase1` `523a84d`, and resolving them means porting the node into cybernode, which is read-only to the director. pussy runs the qualified `b6f22dc`. Relabelled `queue` → `blocked`. Fable answered B, and the director agreed.

- session 412: this report brought current through session 411. The tick `20261031T162256Z` is unchanged since 16:03 UTC. REST at about 16:25 UTC shows 0 reviews and no new comment since 15:40 UTC on soft3#376, #350, #398, #384, lens#36, hemera#25, radio#52, cybernode#1, simulation#2 or simulation#3.

- sessions 413–416: idle, on an unchanged tick (`20261031T164125Z`, `165956Z`, `171801Z`, `173536Z`; `diff` without `at`: empty). No owner review or comment on the four landings or the held PRs. In 414 the director found 18 commits on `release/phase1` dated 2026-10-31 that carry a `Co-Authored-By` trailer, which `AGENTS.md` § git forbids; the director's commits omit it from that session on.

## PRs merged

Into soft3 `release/phase1`: session 394's plan PR #481; [#479](https://github.com/cyberia-to/soft3/pull/479) (P2, `bce863b`); session 395's plan PR #482; [#484](https://github.com/cyberia-to/soft3/pull/484) (P2a, `b6f22dc`); the plan PRs #483, #485, #486 (sessions 396–398), session 399's #487; [#489](https://github.com/cyberia-to/soft3/pull/489) (P2, the stand, `89d9c24`); session 400's plan PR #490; session 401's #491; the owner's [#488](https://github.com/cyberia-to/soft3/pull/488) (P3) through session 402's #492; [#493](https://github.com/cyberia-to/soft3/pull/493) (P3, `feb3d18`); the plan PRs #494 and #495 (sessions 403 and 404) and #496 (405); [#497](https://github.com/cyberia-to/soft3/pull/497) (P3d, `c16fa90`); #498 (406); #499 (408); [#500](https://github.com/cyberia-to/soft3/pull/500) (P3e, `5580cb7`); #501 (409); #502 (410); #503 (411). No PR was merged into any default branch. Every landing waits on the owner.

## plan items left

P2, P2a, P2b, P3 (a), P3d and P3e are done. C4 is held to after pussy's genesis (D15). What is left is the owner's hand (`plan.md` § what only the owner does):

- P3 (b): cyberproxy's `soft3-chain@pussy-rc` unit with its endpoint id, plus § E on deimos and jupiter. Then the director does P3 (c), the rehearsal on the two-host pair.
- The landings: soft3#376, lens#36, hemera#25, nox#43 and radio#52. The asks that lapsed on 2026-10-29: L3b, L3c, L3d, C2a and G10b.
- R1, the candidate from origin defaults, follows the landings.

The freeze starts 2026-11-02 00:00 UTC.

## fable and windows

Fable questions asked: 1, in session 411 on C4 (`state/fable-2026-10-31`). Subscription windows hit: none recorded.
