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

The latest rehearsal is 40446879208 (`release/phase1` `5ca1d64`, `soft3=release/phase1`, 40437685274's overrides), all five legs read (sessions 120, 122, 124). On all four building legs (x86_64/aarch64 linux, aarch64/x86_64 mac), soft3-tests, soft3-release and node-status are **green** and soft3-tests shows `warnings: false`. Each leg has two reds:

| gate | cause | row |
|---|---|---|
| phase1-pins | the leaves are pinned (soft3#373, session 123); the 0.4 tier, its ports and neuron move after the landings | G9 |
| release-notes-source | cyb `master` `ab12f38` was a direct push | G8, the owner's input |

The stack leg of rehearsal 40446879208 (job 122000222218, `release/phase1` `5ca1d64`, read in session 124) is the first to finish with a verdict: 108.4 min of its 210-min cap, 24 gates summing to 6478.0 s. It is green except phase1-pins (G9), release-notes-source (G8), stack-zheng (the 64 KiB final-proof gate, L3b), stack-mudra (`domain_compat` legacy address bytes, L3c) and stack-neuron (lock not updated under `--locked`, L3c). simulation-gate is **green** (961.7 s, 16 scenarios, 0 failures; economy's two reds are the named expected ones), and so are conformance-snapshot, chain-genesis, soft3-tests, soft3-release and node-status.

## units done

| row | receipt |
|---|---|
| G16 Intel mac `mine_e2e` (session 107) | 40422683385 `candidate-x86_64-apple-darwin`: soft3-tests exit 0, 887.8 s of 1800, `warnings: false`; `mine_e2e` 1/0 in 25.75 s |
| G15 rehearsals measure `release/phase1` (session 107) | 40422683385 `sources.json` soft3 override `21d963d`; soft3-tests green on all four building legs |
| G16a(2) the CPU probe rate (sessions 108–112) | soft3#358 merged as `063b6d9`; 40427084706 ubuntu green, 40430771555 pass 1 Intel green (exit 0, `warnings: 0`) |
| G16a(1) forced WebGPU after an abandoned dispatch, named (session 115) | 40430771555 pass 2 and `stall-first` step 8: `every_backend_agrees_and_verifies` hangs to the 1800 s timeout after `mine_e2e`'s abandoned probe, 7/7; the gate's order green; Auto already refuses the VM adapter, so no code change |
| G6 built: `conformance/rs` on zheng 0.4's `ZHENGPF1`, blessed on hemera 0.4 (sessions 113–114) | soft3#362 merged as `53e341d`; diagnostic 40437312746: bless 0 changes, root `h6032aa33…`, `--check` ×2 green, 21/21 tests, 0 warnings; acceptance under rehearsal 40437685274 |
| 40437685274 Intel leg read (session 117) | `candidate-x86_64-apple-darwin`: soft3-tests exit 0, `warnings: false`, 1216.5 s of 1800 (lib 293/0, `mine_e2e` 1/0 in 36.75 s); soft3-release green; node-status green; red only phase1-pins, release-notes-source |
| 40437685274 stack leg read (session 118) | conformance-snapshot **green** (exit 0, 43.3 s, root `h6032aa33…`): G6 done. Red: stack-zheng (L3b), stack-mudra and stack-neuron (L3c), phase1-pins (G9), release-notes-source (G8). The leg was cancelled at its 90-min cap inside simulation-gate; the gates sum to 4752.7 s, a full leg ≈ 105 min (G10a, next) |
| G10a the stack leg's cap (session 119) | soft3#368 merged as `5ca1d64`: the stack row gets 210 min, platform legs keep 90; gate 40446801267 36 tests OK; acceptance rehearsal 40446879208 |
| 40446879208 platform legs read (session 120) | soft3 built at `release/phase1` `5ca1d64` (`sources.json` override). x86_64 linux, aarch64 linux, aarch64 mac: soft3-tests green (exit 0, `warnings: false`), soft3-release and node-status green; red only phase1-pins (G9), release-notes-source (G8). The stack and Intel legs were still running at 04:20 |
| 40446879208 Intel leg read (session 122) | `candidate-x86_64-apple-darwin` (soft3 `release/phase1` `5ca1d64`, `sources.json` override): soft3-tests green, exit 0, `warnings: false`, 1057.8 s of 1800 (lib 293/0, `every_backend_agrees_and_verifies` ok, `mine_e2e` 1/0 in 36.07 s; log sha256 `d39343b6…`); soft3-release green 959.2 s; node-status green; red only phase1-pins (G9), release-notes-source (G8). All four building legs at `5ca1d64` now agree; the stack leg was still running at 04:46 |
| G9a, the leaves (session 123) | soft3#373 → `a3ca9b7`: strata, glia, honeycrisp, inf, mir, soma, spark, tade, evy, optica pinned to the default HEADs 40446879208 captured without an override, and nu pinned for the first time (`7c498b3`). All fast-forward, no version change. PR gates `drift` 40453611829 and `test` 40453611855 green |
| G10a accepted, 40446879208 stack leg read (session 124) | job 122000222218: `Qualify stack and product` 03:56:13–05:44:34, 6501 s of the 210-min cap; `Report the verdict` ran, `assembled-candidate` produced; receipt sha256 `8c6d71d5…`, 24 gates, sum 6478.0 s; simulation-gate green 961.7 s; reds only G8, G9, L3b (stack-zheng 324/1), L3c (stack-mudra, stack-neuron) |
| R0 soft3's landing opened (session 125) | **soft3#376** `release/phase1 → main` (`cd15055`, 274 ahead, 0 behind, no `specs/`), `decision`/`wide`. It waits on the owner's approval and must land before or with lens#36 (G13); decision D11 |
| no unit (sessions 126–131, 06:25–07:50) | state checked only, six sessions: each tick from `20261027T062108Z` to `20261027T074807Z` equals the last except for `at` (`jq -S 'del(.at)'`); no PR or issue activity in `cyberia-to` except the director's own #376, lens#36 and plan PRs; soft3#376, hemera#25 and lens#36 `BLOCKED`/`REVIEW_REQUIRED`, 0 reviews; every red of 40446879208 has an owner row (G8, G9, L3b, L3c); every row not done waits on the owner or the landings. Session 130 added D12 (no merge into a landing's head once it is approved). Session 131 found that sessions 127–130 had not updated this row and folded them in |
| the idle churn's cause (session 132) | no phase unit could move (tick `20261027T080510Z` equals the last except `at`; the owner's newest `cyberia-to` event `2026-10-26T07:35:34Z`). **soft3#384** into the owner's `feat/soft3-director`, not merged: `tick.sh` gains `owner_activity`, `loop.sh` skips a session on an unchanged tick after an idle one (3 h heartbeat). Harness (a)–(f) and the live `gh api` expression as receipts |
| no unit (sessions 133–134, 08:26–08:43) | state checked only: the tick's one change is #384 itself (session 134: tick `20261027T084307Z` equals `20261027T082552Z` except `at`, `tick.log` `changed=0`); no owner event since `2026-10-26T07:35:34Z`; no train run since 40446879208; soft3#376, hemera#25, lens#36 and nox#43 `BLOCKED`, 0 reviews |

## PRs merged

soft3#358 into `release/phase1` (`063b6d9`, session 112), soft3#362 (`53e341d`, session 114), soft3#368 (`5ca1d64`, session 119), soft3#373 (`a3ca9b7`, session 123: G9a, the 11 leaves pinned to the qualified default HEADs of 40446879208), and the plan PRs. Opened, the owner's to merge: soft3#376 (R0), soft3#384 (session 132).

## plan items left

Open: R1. Doing: G9 (G9a, the leaves, done in session 123 by soft3#373; the rest waits on the landings). Done: G6 (session 118, conformance-snapshot green in 40437685274). Done: G10a (session 124), the stack leg finished in 108.4 min of its 210-min cap with a full verdict (40446879208). Interim (sessions 116–117, from `state/run-40437685274/candidate-*/receipt/release-validation.json`): on all four platform legs (aarch64 linux, x86_64 linux, aarch64 mac, x86_64 mac) only phase1-pins (G9) and release-notes-source (G8) are red. G9 waits on the landings L1–L4. Waiting on the owner (by 2026-10-29 12:00 UTC): L3b, L3c (mudra), L3d, L4 (D10), C2a (#350). Waiting on L4: C3. See `release/director/plan.md` § the order.

## fable questions

0 of `FABLE_DAILY` 20 (no `state/fable-2026-10-27`).

## subscription windows

None hit so far (session 108). Session 106 (00:09) ended without a final line (`NEXT: ?` in `state/tick.log` at 00:10:28). Session 113 (≈01:54) ended without a record, after pushing G6's port and its bless diagnostic; session 114 finished the row.
