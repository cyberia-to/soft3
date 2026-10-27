---
title: the director's report — 2026-10-27
tags: soft3, release, director, report, phase 1
crystal-type: measurement
crystal-domain: cyber
date: 2026-10-27
serves: release/director/README.md § the measurement
---
# the director's report — 2026-10-27

Opened in session 107 (00:30 UTC) from `release/director/plan.md`'s log and the receipts it names. Each later session's plan PR updates it.

## the candidate: red

The latest rehearsal is 40422683385 (`release/phase1` `21d963d`, `soft3=release/phase1`, 40389629921's overrides). On all four building legs (x86_64/aarch64 linux, aarch64/x86_64 mac), soft3-tests, soft3-release and node-status are **green** and soft3-tests shows `warnings: false`. Each leg has two reds:

| gate | cause | row |
|---|---|---|
| phase1-pins | pins move after the landings | G9 |
| release-notes-source | cyb `master` `ab12f38` was a direct push | G8, the owner's input |

The stack leg (job 121920831788) was still running at 00:30 UTC.

## units done

| row | receipt |
|---|---|
| G16 Intel mac `mine_e2e` (session 107) | 40422683385 `candidate-x86_64-apple-darwin`: soft3-tests exit 0, 887.8 s of 1800, `warnings: false`; `mine_e2e` 1/0 in 25.75 s |
| G15 rehearsals measure `release/phase1` (session 107) | 40422683385 `sources.json` soft3 override `21d963d`; soft3-tests green on all four building legs |

## PRs merged

Only this plan PR.

## plan items left

Open: G9, G16a, R1. Waiting on the owner (by 2026-10-29 12:00 UTC): L3b, L3c (mudra), L3d, L4 (D10), C2a (#350). Waiting on L4: G6, C3. See `release/director/plan.md` § the order.

## fable questions

0 of `FABLE_DAILY` 20 (no `state/fable-2026-10-27`).

## subscription windows

None hit so far. Session 106 (00:09) ended without a final line (`NEXT: ?` in `state/tick.log` at 00:10:28).
