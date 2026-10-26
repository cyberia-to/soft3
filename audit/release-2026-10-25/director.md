---
title: the director's report — 2026-10-25
tags: soft3, release, director, report, phase 1
crystal-type: measurement
crystal-domain: cyber
date: 2026-10-25
serves: release/director/README.md § the measurement
---
# the director's report — 2026-10-25

Written after the day, in session 39 (2026-10-26, plan row G11), from `release/director/plan.md`'s log and the receipts it names. The director's first session started at 23:11 UTC (`state/tick.log`).

## the candidate: red

No cut and no green leg. The day's last train run on a default branch is `Release train` run **40098866000** (2026-10-24 13:36 UTC, `cut=false`), and all five legs are red. The four platform legs each fail 6 gates (`phase1-pins`, `release-notes-source`, `soft3-tests`, `soft3-release`, `node-status`, `package-resolution`), and the stack leg fails 15 (`state/red-40098866000.json`). Why: package-resolution cannot resolve `lens/gpu` and `cyber-hemera ^0.4` from the default branches, because the 0.4 line lives on `release/*` branches (plan rows L1–L4).

Rehearsals dispatched from `release/phase1` (`cut=false`), both concluded `failure`:

| run | override | what it showed |
|---|---|---|
| 40246391619 (23:48) | `lens=release/0.2` | `lens/gpu` resolves, so package-resolution moves to `cyber-hemera ^0.4` (only 0.3.1 on hemera `main`), which confirms L1 before L2 |
| 40246955916 (23:56) | `hemera=4f267d1 lens=release/0.2` | stack-hemera green, so package-resolution moves to `bbg 0.3.0` requiring `cyber-hemera ^0.3` (L3); read 2026-10-26 |

## units done

| row | receipt |
|---|---|
| G0 the plan | soft3#248 merged on `release/phase1` |
| B0 the org ruleset | lifted by the owner by 23:36; #248 `mergeStateStatus: CLEAN` |
| G1 lane labels | labels in 26 repositories; 19/19 open soft3 PRs laned (soft3#249) |
| G2 sibling rehearsal | soft3#250 `a177b0a`; `Release engine / test` run 40246344403, 27 tests OK; rehearsal 40246391619 records the override |
| G3 daily train | soft3#246 `4fdd704` on `release/phase1`; `Release engine / test` run 40096560433 |
| L1 hemera (begun) | hemera#24 merged into `release/0.4` `8559b52` (CI run 40246904637); landing hemera#25 to the owner |

## PRs merged

8 PRs: soft3#246, #248, #249, #250, #251, #252, #253, and hemera#24 (`gh search prs --owner cyberia-to --merged-at 2026-10-25`).

## plan items left at the day's end

L1 (landing to the owner), L2, L3, L4, L5, G4–G9, C1–C3 and R1.

## fable

1 question of `FABLE_DAILY` 20 (`state/fable-2026-10-25`): D1 at 23:14 UTC.

## subscription windows

None hit. `state/tick.log` has no usage-limit line. 7 sessions ran (`state/session-20261025T*.log`).
