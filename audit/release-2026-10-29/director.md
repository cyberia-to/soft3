---
title: the director's report — 2026-10-29
tags: soft3, release, director, report, phase 1
crystal-type: measurement
crystal-domain: cyber
date: 2026-10-29
serves: release/director/README.md § the measurement
---
# the director's report — 2026-10-29

Opened in session 268 (00:13 UTC) from `release/director/plan.md`'s log and the receipts it names. Each later session's plan PR updates it.

## the candidate: red

The newest train run is still rehearsal 40688607756 (2026-10-28 12:32 UTC, `failure`; tick `20261029T001342Z` `last_run`). Its reds are in `state/red-40656164861.json`, the baseline since session 217:

| gate | cause | row |
|---|---|---|
| phase1-pins | rehearsal drift on the overridden siblings (23 drift in the tick); the 0.4 tier and its ports move after the landings | G9 |
| release-notes-source | cyb `master` `ab12f38` was a direct push | G8, the owner's input |
| stack-zheng, stack-mudra, stack-neuron | the 64 KiB gate (L3b) and the keep-or-rotate decision (L3c), due from the owner by **2026-10-29 12:00 UTC** | L3b, L3c, L4 |

stack-radio went green in 40688607756 (S1(b), session 233).

## units done

| row | receipt |
|---|---|
| this report (session 268) | opened on `plan/idle-20261028-s235b` (soft3#470) |

## PRs merged

None yet.

## plan items left

Waiting on the owner's approval of the landing PRs: L1 hemera#25, L2 lens#36, L3 nox#43, R0 soft3#376, S1 radio#52. Each has 0 reviews (REST, 00:13 UTC). Due from the owner today at 12:00 UTC: the decisions L3b, L3c, L3d and C2a (soft3#350, `clean`). C4 waits on cybernode#1. S1 also waits on simulation#3. G9 finishes after those, and R1 comes last. The code freeze starts 2026-11-02 00:00 UTC.

## fable and windows

Fable questions asked: 0. Subscription windows hit: none.
