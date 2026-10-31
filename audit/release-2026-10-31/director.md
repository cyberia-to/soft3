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

## units done

- session 394: **P1** done. The rehearsal's overrides equal the stand file, 16 of 16, and its acceptance gates are green on all five legs. The remaining reds have rows (G8, G9, L3b, L3c).

- session 395: **P2**: soft3#479 merged into `release/phase1` as `bce863b`. Red diagnostic 41042358656 fails on the hole, and green 41042360873 has lib 294/0 with `warnings: 0`. The all-legs rehearsal [41192673416](https://github.com/cyberia-to/soft3/actions/runs/41192673416) is dispatched.

- session 396: **P2a**: release `overflow-checks` diagnostic [41192950342](https://github.com/cyberia-to/soft3/actions/runs/41192950342) dispatched (baseline and overflow legs on pussy's stand). P2's rehearsal is still running.

- session 397: **P2a**: diagnostic 41192950342 green on both legs (soft3 4/4, tru 1/1 with the flag). [soft3#484](https://github.com/cyberia-to/soft3/pull/484) opened with the release profile.

- session 398: **P2a** done: soft3#484 merged into `release/phase1` as `b6f22dc` after gate [41194300832](https://github.com/cyberia-to/soft3/actions/runs/41194300832) went green.

- session 399: **P2**: the four platform legs of rehearsal [41192673416](https://github.com/cyberia-to/soft3/actions/runs/41192673416) read. `a_settlement_panic_is_a_named_halt` passes and soft3-tests, soft3-release and node-status are green on all four. The only reds are phase1-pins and release-notes-source. The stack leg is still qualifying.

- session 400: **P2** done. The stack leg of [41192673416](https://github.com/cyberia-to/soft3/actions/runs/41192673416) is green on soft3-tests, soft3-release, node-status and simulation-gate, `a_settlement_panic_is_a_named_halt` passes, and the reds are the stand's known set. [#489](https://github.com/cyberia-to/soft3/pull/489) bumped the stand's soft3 to `b6f22dc` (merged as `89d9c24`).

## PRs merged

Into soft3 `release/phase1`: session 394's plan PR #481; [#479](https://github.com/cyberia-to/soft3/pull/479) (P2, `bce863b`); session 395's plan PR #482; [#484](https://github.com/cyberia-to/soft3/pull/484) (P2a, `b6f22dc`); the plan PRs #483, #485, #486 (sessions 396–398), session 399's #487; [#489](https://github.com/cyberia-to/soft3/pull/489) (P2, the stand, `89d9c24`); session 400's plan PR.

## plan items left

P2 and P2a are done. Left: an all-legs rehearsal of the stand at `b6f22dc` (P2a's profile), the owner's landings and asks (`plan.md` § what only the owner does), and R1.

## fable and windows

Fable questions asked: 0. Subscription windows hit: none.
