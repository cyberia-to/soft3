---
title: the director's report — 2026-10-26
tags: soft3, release, director, report, phase 1
crystal-type: measurement
crystal-domain: cyber
date: 2026-10-26
serves: release/director/README.md § the measurement
---
# the director's report — 2026-10-26

Opened in session 39 (08:00 UTC, plan row G11) from `release/director/plan.md`'s log and the receipts it names. Each later session's plan PR updates it.

## the candidate: red

No run from origin defaults today. The 12:00 daily cut (G3) takes effect only after the owner approves `release/phase1 → main`.

The last rehearsal read in full is **40280626840** (06:34–07:19, stack leg `receipt/release-validation.json`, 0.4 tier overridden). 15 gates are red and 2 are blocked. The cause of each is the first error of its log in `receipt/logs/`:

| gate | cause | row |
|---|---|---|
| package-resolution, soft3-tests, soft3-release, node-status | soft3 depends on `cybergraph` with feature `local-storage`, which exists only in the owner's `decision` chain bbg#9 → foculus#5 → cybergraph#3 | L4, owner's decision by 10-29 12:00 |
| stack-vault, stack-neuron | `failed to select a version for cybergraph` (the same feature) | L4 |
| stack-cybergraph, stack-foculus, conformance-snapshot | `bbg = "^0.2"` unresolvable against bbg's 0.4 line | L4 consumers, G6 |
| stack-mudra | `cyber-hemera = "^0.3"`: mudra#30 is unmerged, and on its own CI it fails on the owner-frozen domain-v1 key vectors | L3c, owner's decision by 10-29 12:00 |
| stack-zheng | timeout after 900.002 s at zheng `release/0.4`; zheng#73 (hemera 0.4) is unmerged because of its 64 KiB final-proof limit | L3b, owner's decision by 10-29 12:00 |
| stack-radio | `iroh-blobs` 140 passed, 1 failed (535 s). Not a receipt for radio#47: radio moved mid-run. **Green in rehearsal 40286177469** at #47's head (695.3 s), merged | G4 done |
| source-inputs-unchanged | `changed: [radio]`; green in 40286177469 | G4 done |
| release-notes-source | cyb `master` `ab12f38` was a direct push | G8, owner approves a cyb PR |
| phase1-pins | pins move after the landings | G9 |
| stack-chain-genesis (blocked) | `no pussy genesis file … (set SOFT3_GENESIS_DIR)` | G5 |
| simulation-gate (blocked) | `no simulation checkout` (#242 merged after this run, `464f5ea`) | G7 |

What moved green today, each at its PR head in a rehearsal: stack-hemera (40246955916), stack-lens (40256961023, 848.1 s; 40258751198, 713.7 s), stack-nox (40258751198), stack-bbg (40267704972, 320 s), and stack-tru, stack-tok, stack-file (40280626840). stack-radio passed every test it reached in 40273566327 and was red only by the 900 s clock.

## units done

| row | receipt |
|---|---|
| L2a WHIR soundness | not a hole; lens#38 `c73e1e6`; run 40254402090, rspcs 43/43 |
| L2b lens fmt/clippy | lens#39 `b2f72c1`; run 40256487269 |
| G10 stack-lens budget | lens#40 `41ef114`; rehearsal 40256961023 stack-lens 848.1 s / 900 s |
| L2 lens | lens#37 `d1a88cc` into `release/0.2`; landing lens#36 to the owner |
| G8 release-notes-source | decided: the gate stays, the input is the owner's |
| L3a zheng fmt/clippy | zheng#74 into #73's branch; run 40263964809 |
| L4 (to the owner) | ask on cybergraph#3 (comment 6408330160), D10 |
| L5 superseded PRs | lens#14, zheng#33 and nox#17 closed with receipts |
| L3c (in part) | tru#43 `464104c`, plumb#23 `8266e9f`, file#39 `4957848`, rune#8 `a2820e6` on each `release/phase1`; mudra#30 to the owner |
| G7 simulation pin | soft3#242 `464f5ea`; drift run 40286588394, test run 40286586961 |
| L3 nox, bbg (in part) | nox#31 `2c67aa9` into `release/0.4`; bbg#52 `3a8b823` into `release/phase1` |
| L5a (in part) | zheng#75 `0ad4c3e`, nox#41 `c25acf4`; zheng#76 open |
| G4 stack-radio (session 40) | radio#47 `e98bc69` into `release/phase1`; rehearsal 40286177469 stack-radio green 695.3 s, 657 passed, 0 failed |
| G10b stack-zheng budget (session 41, doing) | cause: unoptimized test profile (rehearsal 40286177469 `logs/stack-zheng.txt`); `fix/g10b-test-profile` `166a14e` under measurement: rehearsal 40293725274, zheng CI 40293728603 |
| G5 stack-chain-genesis (session 42, doing) | soft3#288: space-pussy pinned at `bdcfdaf` (manifest sha256 `b46cb327…` = blessed), the gate builds then checks; unit tests 31 OK; rehearsal waits on 40293725274 starting |
| C1 soft3 PR queue (session 43, done) | 18 open soft3 PRs triaged against the plan; none merges into phase 1 (#234 waits on the owner's economy spec, #243 is a spec, #212 needs owner host steps, #167 #169 #170 #245 are outside phase 1); reasons in plan row C1 |
| C2 proven-present S0 (session 44, doing) | soft3#291 (draft): corpus writer, verdict codes, invalid-block generator, replay, test, `node --corpus`; not compiled: no stand compiles soft3 (diagnostic 40297730975: bbg#9 fails on zheng 0.4, `unresolved import zheng::LookOpening`) |
| G10b measured (session 44) | zheng CI 40293728603 at opt-level 3: 324 passed, 1 failed (L3b 64 KiB), 1077.95 s, over 900 s |
| G5 rehearsal (session 45) | Release train 40299054024 on #288's branch `50ef027`, same overrides as 40293725274, which it replaced (`cancelled`); pending |
| G7 simulation-gate (session 46, done) | rehearsal 40293230892: simulation-gate runs, red at resolution (`mudra` → `cyber-hemera ^0.3`, L3c) |
| L5a zheng#76 (session 47) | 40288589957 `test` hit its 90-min timeout before the cli tests; diagnostic `diag/l5a-76-cli` `fbf3bb8` (+ `166a14e`), zheng CI 40301875126 pending |
| G5 stack-chain-genesis (session 49, done) | rehearsal 40299054024: the gate runs (red on L4 `cybergraph` `local-storage`, 0.265 s); soft3#288 merged `893d9e0` into `release/phase1` |
| G10b on the stand (session 49) | 40299054024 at `166a14e`: 323 ok, 1 failed (L3b), timeout 900.002 s |
| L5a (session 48, no final line) | zheng 40304510063 at `24b4439`: test, lockfile, fmt, clippy success; #76 merge next |
| L5a (session 50, done) | zheng#76 reviewed and merged into `ci/hemera-0.4-sweep` as `7d3fc4e` on receipt 40304510063 (cli 2/2, clippy); debug `test` 40304422158 cancelled |
| G10b budget (session 51) | the soundness review `a_state_unbound_from_the_last_public_input_is_refused` takes about 985 s alone (zheng CI 40293728603 job log); soft3#298 gives stack-zheng 1800 s (others 900 s); rehearsal 40307972588 on the branch |
| L3c plumb CI (session 52) | plumb#24 `2b3bdce` into `release/phase1`: `TRU_REF` = `release/phase1`; CI 40310274101 84 passed, tru at `464104c` |
| L3 nox sync (session 53, doing) | nox#42 `730f6b0` → `release/0.4`: master `ec052bb` merged in (19 conflicts, launch fixes kept, hash cost 61 kept), first CI; run 40313369006 in progress |
| G10b budget merged (session 54) | rehearsal 40307972588: stack-zheng 987.2 s of 1800 s, 324 passed, 1 failed (L3b 64 KiB), soundness review ok; soft3#298 merged `b764f7a`; profile `166a14e` onto #73's branch remains |
| G10b profile PR (session 55) | zheng#77 `7471376` → `ci/hemera-0.4-sweep` (draft): the profile rebased onto `7d3fc4e`; diagnostic zheng 40316171643 (`--no-fail-fast`, every `rs/` binary) in progress; nox CI 40313369006 (nox#42) green |
| L3d split out (session 57) | joy and trident are on hemera 0.3 (joy `e89e4e3`, trident `master`). trident#129 `b29aabc` is the owner's 0.4 draft, and its blockers lens#37 and nox#31 are cleared. `cyberia-apparatus` has `push=false` on both, which are not in team `apparatus`. The access ask is on trident#129 (comment 6415437829), deadline 10-29 12:00; it does not gate R1 |
| C3 verified built (session 58) | registry rows 26/27/41 → `economy/vectors.rs:263-274` + chain.snap `two epochs: …`; `integers_of_section_14`, `coinbase_split_…`; `two_nodes_agree_on_every_epoch_…`; all under soft3-tests, which does not compile until L4 (rehearsal 40307972588) |
| G10a budgets (session 59) | stack-lens green 615.7–856.0 s (six rehearsals on `release/0.2`), stack-radio 605.3–779.8 s; soft3#307 `STACK_TEST_BUDGET` lens 1200, radio 1200, gate `test` green (run 40323982690), merged `a04bfce`; the stack-leg sum waits on L4 |
| L3 nox merged (session 60) | rehearsal 40316574923 stack leg: stack-nox green 96.2 s, 292 passed 0 failed at nox#42 `730f6b0`; nox#42 merged into `release/0.4` `41ba24f`; landing nox#43 `release/0.4 → master` opened for the owner. stack-radio and source-inputs-unchanged red in the same run, radio's (`e98bc69` unchanged, `iroh-blobs` proptest) → G4a |
| G4a cause and fix up (session 61) | the flake is tokio 1.49.0's mpsc `len()` underflow (tokio #8062). radio#49 moves the lock to tokio 1.51.5 (lock from radio run 40325343061). The same run's 50× repeat: 1 failed at 1.49.0, 0 at 1.51.5. Rehearsal 40325821696 (`radio=pull/49/head`) decides the merge |
| G4a train side merged (session 62) | soft3#311 `9f78716`: a stack test that changes its own tree is red on that gate (`dirtied`, `logs/stack-<owner>.dirtied.txt`), and the checkout is restored to its recorded revision; gate `test` run 40327523244, 35 tests OK |
| G6 measured (session 63) | waits on L4: rehearsal 40316574923 `logs/conformance-snapshot.txt` exits 101 at resolution, cybergraph 0.1.1 (the harness's path dependency, `conformance/rs/Cargo.toml:33`) requires `bbg ^0.2`; the cybergraph surface stays in the snapshot |
| L3b round 3 read (session 65) | zheng 40287758728: control `9f:26:5` 56:50, FINAL 71,653 B (> 64 KiB), final level 1,740.7 s; `10f:32:5` and `9f:36:5` cancelled at 355 min, so (i)'s final level takes > ~19,600 s (> 11×); posted on zheng#73 (comment 6417716289); round 4 with timing: zheng 40334403053 |
| G4a done (session 66) | rehearsal 40325821696: stack-radio green 787.0 s at radio#49 `2a0ee79` (tokio 1.51.5), 657 passed 0 failed, proptests `_fs`/`_mem` ok, `source-inputs-unchanged` green; own CI reds = #47's (deny, clippy, wasm, codespell, fmt, same findings); radio#49 merged into `release/phase1` as `7452833` |
| L4 accept branch drafted (session 68) | foculus#86 `9e34693`, cybergraph#23 `91e1fd5` (zheng `legacy` dev-dependency for #3's refusal tests; 8 `verify_state_execution` calls disabled with reason), over bbg#53 (CI 40333694101 green); rehearsal 40341228019 dispatched with the chain overridden; nothing merged (D10, the owner's) |
| G10b decided (session 70) | zheng 40316171643 (`--no-fail-fast`, cancelled at 180 min): lib 1,075.7 s, `succinct_bitflip` 5,097.7 s, `wrap` 1,463.3 s, `recursion` 1,112.1 s, `wrap_review` > 1,460 s unfinished; ≥ ~10,860 s vs 1,800 s. fable #5: zheng `#[ignore]` + `heavy` CI + the train requires that check; the soundness half asked of the owner on zheng#77 (comment 6419556782) |

## PRs merged

45 PRs by 08:00 (`gh search prs --owner cyberia-to --merged-at 2026-10-26`). 14 are outside soft3: lens#31, #37, #38, #39 and #40; nox#31 and #41; zheng#74 and #75; bbg#52; tru#43; plumb#23; rune#8; file#39. 31 are in soft3: #242 on `release/phase1` and the plan's 30 PRs from #254 to #284 (#283 was not merged). Session 40 added radio#47 (`e98bc69`, 08:34). Later: soft3#298 (`b764f7a`, session 54); soft3#288 (`893d9e0`, session 49) and zheng#76 (`7d3fc4e`, session 50), plumb#24 (`2b3bdce`, session 52), soft3#307 (`a04bfce`, session 59), nox#42 (`41ba24f`, session 60). Session 62: soft3#311 (`9f78716`).

## plan items left

Open: G6, G9 and R1; G10a doing (session 59: budgets merged, the stack-leg sum waits on L4); C3 built, acceptance waits on L4 (session 58) (C2 doing since session 44, soft3#291). G6 waits on L4 (conformance-snapshot resolves `cybergraph` → `bbg ^0.2`). Doing: L1, L3, L3b, L3c and G10b (session 41; soft3#298 under rehearsal 40307972588, session 51). Done since 08:00: G11 (session 39), G4 (session 40), C1 (session 43), G7 (session 46: simulation-gate runs; red owned by L3c). G5 (session 49), L5a (session 50). G4a done in session 66 (radio#49 merged into radio `release/phase1` as `7452833` on rehearsal 40325821696). Session 64: L4's accept branch prepared as draft bbg#53 (not merged; CI 40333694101). With the owner: L4 (D10), L3b, mudra#30 (L3c), L3d (access to joy and trident, session 57), and the landings hemera#25, lens#36 and nox#43 (session 60).

## fable

4 questions of `FABLE_DAILY` 20 by 08:00 (`state/fable-2026-10-26`, `state/fable.log`): D10 storage chain (04:34), zheng limits (05:07), mudra keys (06:31, 07:16). Session 70: #5, zheng's stack-gate budget (G10b), answered "A: `#[ignore]` + `heavy` job + a train check; the soundness half is the owner's".

## subscription windows

One hit: `state/tick.log` line 282, `08:02:16 subscription window hit; sleeping 30 min` (corrected in session 47; the earlier text said none). 34 sessions started by 08:00 (`state/session-20261026T*.log`). **4 ended without a final line** (`NEXT: ?` at 05:29, 05:48, 06:35 and 06:53 in `state/tick.log`). Two of them opened L3c twice, which session 30 consolidated.
