---
title: the director's day — 2026-11-01
tags: soft3, release, director, audit
date: 2026-11-01
---
# the director's day — 2026-11-01

per `release/director/README.md` § the measurement. Sessions append. Each number names its run or PR.

## the candidate

**Red at the day's close.** No train ran on the defaults on 2026-11-01; the newest is still [41097270082](https://github.com/cyberia-to/soft3/actions/runs/41097270082) (`schedule`, `main`, 2026-10-30), red at package-resolution on every leg until [lens#36](https://github.com/cyberia-to/lens/pull/36) lands (L2). The day's rehearsals ran from `release/phase1` and PR heads (`gh run list -w release-train.yml`): [41354213746](https://github.com/cyberia-to/soft3/actions/runs/41354213746) (`release/phase1`, 21:52 UTC, C4) and [41344713007](https://github.com/cyberia-to/soft3/actions/runs/41344713007) (G18) are red only on phase1-pins (G9) and release-notes-source (G8) on every leg, and on stack-zheng, stack-mudra and stack-neuron on the stack leg (L3b, L3c: the owner's), as `state/red-41344713007.json` records. The other five dispatches were cancelled or superseded (41315563611, 41326426601, 41333224819, 41344227676).

## units done

- session 438: closed the 2026-10-31 report (sessions 435–437, plan PRs #528–#531) and opened this one. The tick `20261101T000612Z` is identical to 437's `20261031T234850Z` except for its timestamp (`jq -S 'del(.at)'` diff: empty). The newest owner event is still soma#5, opened 2026-10-31 12:50:53 UTC (`users/mastercyb/events`). Reviews by anyone but the director: 0 on soft3#376, lens#36, hemera#25 and radio#52. The doctrine heads are unchanged: cyberia `b48a145`, cyber `68f1260`, soft3 `main` `3045d54`.
- session 439: idle. Tick `20261101T002401Z` is identical to 438's (`jq -S 'del(.at)'` diff: empty). No new train run, owner event or review; the doctrine heads are unchanged.
- session 440: idle. Tick `20261101T004114Z` is identical to 439's (`jq -S 'del(.at)'` diff: empty). No new train run, owner event or review; the doctrine heads are unchanged.
- session 441: the loop handed sessions a stale red file (`red-41097270082.json`, not the last run 41201610001). Repaired in state: the four missing legs were downloaded and `red-41201610001.json` written; it equals session 408's reading. The fix is [soft3#535](https://github.com/cyberia-to/soft3/pull/535), stacked on #384 and waiting for the owner; gate `test` green (run 41259397268).
- session 442: idle. The tick changed only by soft3#535, which the director opened itself. No new train run, owner event or review, and the doctrine heads are unchanged. The session was handed the right red file (`red-41201610001.json`) because of session 441's state repair; #535 is not installed.

- sessions 443–464: idle. Each tick was identical to the one before it apart from its timestamp (folded into one plan log row by session 462).
- session 465 onward, after the owner's D16 ([soft3#560](https://github.com/cyberia-to/soft3/pull/560), [cyberia#10](https://github.com/cyberia-to/cyberia/pull/10): pussy first). The units, each with its receipt in `plan.md` § the order:
  - **P4** (sessions 465, 475, 483): [soft3#562](https://github.com/cyberia-to/soft3/pull/562) merged as `e11d8a5` after re-gate [41296572291](https://github.com/cyberia-to/soft3/actions/runs/41296572291).
  - **P3a** (466, 478): [soft3#565](https://github.com/cyberia-to/soft3/pull/565) merged as `d2ad0c7` (glibc 2.35 floor).
  - **P3b** (467, 479, 485–489): [soft3#567](https://github.com/cyberia-to/soft3/pull/567) merged as `29022f7` and [#589](https://github.com/cyberia-to/soft3/pull/589) as `249eb95` (the `pussy-rc.tade` pins), from [41308714022](https://github.com/cyberia-to/soft3/actions/runs/41308714022).
  - **P5** (468–471): the receipt `audit/chain/proof-auth-2026-11.md` ([#570](https://github.com/cyberia-to/soft3/pull/570), [#574](https://github.com/cyberia-to/soft3/pull/574); zheng [41289914355](https://github.com/cyberia-to/zheng/actions/runs/41289914355), [41291667421](https://github.com/cyberia-to/zheng/actions/runs/41291667421)). The §2.9 draft is [soft3#572](https://github.com/cyberia-to/soft3/pull/572), `decision`, the owner's.
  - **P6** (472–474, 484, 490–491, 495–500, 508): [foculus#86](https://github.com/cyberia-to/foculus/pull/86), [#88](https://github.com/cyberia-to/foculus/pull/88) and [#89](https://github.com/cyberia-to/foculus/pull/89) merged on foculus `release/phase1`.
  - **P7** (477): [tru#44](https://github.com/cyberia-to/tru/pull/44) merged as `cbd9025`; D18.
  - **P8** (482): `audit/chain/block-relation-2026-11.md` (census [41311666610](https://github.com/cyberia-to/soft3/actions/runs/41311666610)).
  - **G17** (492, 501): [soft3#593](https://github.com/cyberia-to/soft3/pull/593) merged as `c3347f6` (aarch64-apple-darwin on macos-15).
  - **G18** (502–504, 506, 508): [soft3#609](https://github.com/cyberia-to/soft3/pull/609) merged as `a465b7d`; the stack leg runs on build-a ([41344713007](https://github.com/cyberia-to/soft3/actions/runs/41344713007)).
  - **C4** (505, 507, 509–512): [soft3#407](https://github.com/cyberia-to/soft3/pull/407) and [#615](https://github.com/cyberia-to/soft3/pull/615) (`95c3790`) merged, and cybernode#2–#4, #6 and #7 on cybernode `release/phase1`. The owner gave the go-ahead (session 505).

## PRs merged

96 across `cyberia-to` (`gh search prs --owner cyberia-to --merged-at 2026-11-01`). By repository:

| repository | PRs | |
|---|---|---|
| soft3 | 86 | 74 plan PRs (sessions 438–512) and 12 work PRs: #406 (the owner's, landed through #407), #407, #562, #565, #567, #570, #574, #589, #593, #606 (the owner's), #609, #615 |
| cybernode | 5 | #2, #3, #4, #6, #7 |
| foculus | 3 | #86, #88, #89 |
| tru | 1 | #44 |
| cyberia | 1 | #10 (the owner's) |

All of them went into release branches or were merged by the owner. The director merged nothing into a default branch.

## plan items left at the close

- **The owner's:** the landings (soft3#376, lens#36, hemera#25, nox#43, radio#52) and the asks that lapsed on 2026-10-29 (L3b, L3c, L3d, C2a, G10b; D10: silence cuts nothing). Also soft3#572 (P5), P3 (b) (done by the owner at 19:47 UTC, read on 2026-11-02 in session 522), and [soft3#479](https://github.com/cyberia-to/soft3/pull/479)'s halt sentence.
- **The director's, carried into 2026-11-02:** P4a ([soft3#595](https://github.com/cyberia-to/soft3/pull/595), rehearsal [41367974273](https://github.com/cyberia-to/soft3/actions/runs/41367974273)) and C4a (simulation#2's lock).
- R1 follows the landings.

The code and feature freeze began at 2026-11-02 00:00 UTC.

## fable and windows

Fable questions asked: 2 (`state/fable-2026-11-01`): P5's relation (A) and D18. Sessions: 80 logs (`state/session-20261101T*.log`). Subscription windows hit: none (`state/tick.log` has no `window hit` line for the day).
