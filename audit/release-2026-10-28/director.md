---
title: the director's report — 2026-10-28
tags: soft3, release, director, report, phase 1
crystal-type: measurement
crystal-domain: cyber
date: 2026-10-28
serves: release/director/README.md § the measurement
---
# the director's report — 2026-10-28

Opened in session 183 (00:40 UTC) from `release/director/plan.md`'s log and the receipts it names. Each later session's plan PR updates it.

## the candidate: red

No train run is newer than 40570117323 (2026-10-27 19:33 UTC, `failure`; `gh run list --workflow "Release train" -L 2`). Its reds, in `state/red-40570117323.json`, belong to `main` before the landings:

| gate | cause | row |
|---|---|---|
| package-resolution, soft3-tests, soft3-release, node-status | `lens/gpu/Cargo.toml` missing on lens `master`; it clears when lens#36 lands (the owner's approval) | L2, R0 |
| phase1-pins | the 0.4 tier, its ports and neuron move after the landings | G9 |
| release-notes-source | cyb `master` `ab12f38` was a direct push | G8, the owner's input |
| stack-zheng, stack-mudra, stack-neuron, stack consumers | the 64 KiB gate (L3b) and the keep-or-rotate decision (L3c), both due from the owner by 2026-10-29 12:00 UTC | L3b, L3c, L4 |

On `release/phase1`, the latest full read is still rehearsal 40446879208 (see the 2026-10-27 report).

## units done

| row | receipt |
|---|---|
| G9b, the leaves after the owner's `AGENTS.md` sweep (session 183) | soft3#440 → `2521fc6`: strata `5ef0c84`, soma `1d3fe69`, honeycrisp `595ce26`, evy `14c8d3e`. Each is a fast-forward of renames only (REST `compare`). PR gates `drift` 40608864456 ("no drift") and `test` 40608864377 are green |

## PRs merged

soft3#440 (pins) and this plan PR, both into `release/phase1`.

## plan items left

L1, L2, L3 and R0 wait on the owner's approval of the landing PRs (hemera#25, lens#36, nox#43, soft3#376). L3b, L3c and L3d wait on the owner's decisions due 2026-10-29 12:00 UTC. C2a (soft3#350) is due the same day. C4 waits on cybernode#1. G9 finishes after those, and R1 comes last.

## fable and windows

Fable questions asked: 0. Subscription windows hit: none.
