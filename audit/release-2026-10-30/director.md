---
title: the director's day — 2026-10-30
tags: soft3, release, director, audit
date: 2026-10-30
---
# the director's day — 2026-10-30

per `release/director/README.md` § the measurement. Sessions append. Each number names its run or PR.

## the candidate

**Red.** The newest train run is still [40886405395](https://github.com/cyberia-to/soft3/actions/runs/40886405395) (2026-10-29 13:55 UTC, `cut=false`, defaults, no overrides). Every leg is red at package-resolution: `cyber-lens-gpu`, because lens `master` lacks the 0.2 line until [lens#36](https://github.com/cyberia-to/lens/pull/36) lands, which is the owner's approval. The landing PRs are the owner's: hemera#25, lens#36, nox#43, radio#52, soft3#376. The 12:00 UTC daily cron runs from `main`, so it stays on Fridays until #376 lands.

## units done

- session 389: the owner's asks in `plan.md` carry their links ([cyberia#9](https://github.com/cyberia-to/cyberia/pull/9)).
- session 391: plan re-scoped to [cyber#154](https://github.com/cyberia-to/cyber/pull/154) (pussy 2026-11-05, bostrom by readiness); D13, rows P1 and P2. [soft3#476](https://github.com/cyberia-to/soft3/pull/476) merged as `61d4616`.
- session 392: **P1** pussy's pinned stand, `release/pussy-stand.toml` (16 sibling commits, code-identical to rehearsal [40688607756](https://github.com/cyberia-to/soft3/actions/runs/40688607756)). [soft3#477](https://github.com/cyberia-to/soft3/pull/477) merged as `9064844`, gate run 41041033175 green. Rehearsal [41041110233](https://github.com/cyberia-to/soft3/actions/runs/41041110233) dispatched with exactly its overrides.
- session 393: **P2** decided D14 (the settlement panic stays a halt, named and state-clean). [soft3#479](https://github.com/cyberia-to/soft3/pull/479) opened, with diagnostics red [41042358656](https://github.com/cyberia-to/soft3/actions/runs/41042358656) and green [41042360873](https://github.com/cyberia-to/soft3/actions/runs/41042360873) running. New row P2a (release `overflow-checks`, measure first).

## PRs merged

Into soft3 `release/phase1`: #476, #477, plus this session's plan PR.

## plan items left

P1 (read 41041110233), P2 (read the diagnostics, merge #479), P2a, the owner's landings and asks (`plan.md` § what only the owner does), R1.

## fable and windows

Fable questions asked: 1 (session 393, P2: answered B, keep the halt). Subscription windows hit: none.
