---
title: the director's day — 2026-11-01
tags: soft3, release, director, audit
date: 2026-11-01
---
# the director's day — 2026-11-01

per `release/director/README.md` § the measurement. Sessions append. Each number names its run or PR.

## the candidate

**Red.** The newest train run on the defaults is still [41097270082](https://github.com/cyberia-to/soft3/actions/runs/41097270082) (`schedule`, `main`, 2026-10-30). Every leg is red at package-resolution: `failed to read …/sources/lens/gpu/Cargo.toml` (`cyber-lens-gpu`), because lens `master` lacks the 0.2 line until [lens#36](https://github.com/cyberia-to/lens/pull/36) lands (L2). phase1-pins (G9) and release-notes-source (G8) wait on the owner's landings too. The newest train run of any kind is still the stand rehearsal [41201610001](https://github.com/cyberia-to/soft3/actions/runs/41201610001) (2026-10-31 12:33 UTC). It holds on all five legs at soft3 `b6f22dc`, as the 2026-10-31 report records.

## units done

- session 438: closed the 2026-10-31 report (sessions 435–437, plan PRs #528–#531) and opened this one. The tick `20261101T000612Z` is identical to 437's `20261031T234850Z` except for its timestamp (`jq -S 'del(.at)'` diff: empty). The newest owner event is still soma#5, opened 2026-10-31 12:50:53 UTC (`users/mastercyb/events`). Reviews by anyone but the director: 0 on soft3#376, lens#36, hemera#25 and radio#52. The doctrine heads are unchanged: cyberia `b48a145`, cyber `68f1260`, soft3 `main` `3045d54`.
- session 439: idle. Tick `20261101T002401Z` is identical to 438's (`jq -S 'del(.at)'` diff: empty). No new train run, owner event or review; the doctrine heads are unchanged.
- session 440: idle. Tick `20261101T004114Z` is identical to 439's (`jq -S 'del(.at)'` diff: empty). No new train run, owner event or review; the doctrine heads are unchanged.

## PRs merged

Into soft3 `release/phase1`: the plan PRs of sessions 438, 439 and 440. Nothing was merged into a default branch, because every landing waits on the owner.

## plan items left

The same as at the end of 2026-10-31. All of it is the owner's hand (`plan.md` § what only the owner does):

- P3 (b): cyberproxy's `soft3-chain@pussy-rc` unit with its endpoint id, plus § E on deimos and jupiter. Then the director does P3 (c), the rehearsal on the two-host pair.
- The landings: soft3#376, lens#36, hemera#25, nox#43 and radio#52, plus the asks that lapsed 2026-10-29 (L3b, L3c, L3d, C2a, G10b).
- R1, the candidate from origin defaults, follows the landings.

**The code and feature freeze starts 2026-11-02 00:00 UTC, about 24 h from this session.** After that only security and bug fixes that come with a failing test merge. pussy's genesis attempt is 2026-11-05 13:37:00 UTC.

## fable and windows

Fable questions asked: 0. Subscription windows hit: none recorded.
