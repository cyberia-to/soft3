---
tags: soft3, audit, release
crystal-type: report
crystal-domain: cyber
date: 2026-10-10
---
# release receipts — 2026-10-10, proof-system repair

Three pieces of evidence for the release that ships the proof-system repair (`proposals/proof-system-repair.md`): a merge-readiness pass over every open proof-repair pull request, a build of every repository's stack top against its siblings' stack tops (launch row 39), and a train dry-run from origin default branches after the merges. This is a dry run: no draft release was created, nothing was tagged, published or promoted. Bump proposal: soft3#169 (`chore: soft3 0.11.0`, draft).

Machine: Apple M4 Max, macOS 26, rustc 1.95.0, shared with other agents.

## 1. train dry-run from origin default branches — RED

Commands, from a clean worktree of soft3 `origin/main` d9c225b:

```
python3 release/train.py snapshot --manager . --component soft3 --candidate candidate-20261010.1 --output <snap>
python3 release/train.py build --snapshot <snap>/sources.json --checkout <checkout> --target stack --stack --output <out>
```

Receipts: `train-stack/` (`sources.json`, `release-validation.json`, `candidate.json`, `soft3-dependencies.*`, `SHA256SUMS`, `logs/`), the snapshot as `train-stack/snapshot-sources.json`. Archive `soft3-candidate-20261010.1-stack.tar.gz` sha256 `143abc2dac1b8d3f30d66ffb59bbfecc185411ada12584661cb8ed90186d316f` (not committed; same files).

| gate | result | evidence (first error in the log) |
|---|---|---|
| origin-checkouts | green | 30 repositories fetched at their default-branch HEAD |
| phase1-pins | red | 15 siblings drifted from `release/phase1.toml` (bbg, cybergraph, evy, glia, hemera, honeycrisp, inf, mudra, nox, nu, radio, spark, strata, tade, tru); soft3#169 moves the pins |
| release-notes-source | red | cyb: captured product HEAD has no merged pull request |
| stack-hemera, stack-lens, stack-file | red | no committed `Cargo.lock`; `--locked` cannot create one |
| stack-bbg, stack-nox, stack-tru | red | committed `Cargo.lock` out of date against the sibling default branches |
| stack-zheng | red | workspace member `cli` reads `../tape/impl/rust` — a dead path on zheng master (0.3.3) |
| stack-foculus | red | dependency `cyber-tape` manifest missing — same dead path on foculus master |
| stack-cybergraph, stack-mudra | red | `cyber-nox = "^0.1.x"` against nox master 0.2.0 |
| stack-neuron, stack-vault | red | cannot select a version of `cybergraph` |
| stack-radio | red | output filename collision warnings; `cannot find module or crate radio` in an example |
| stack-tok | green | 8 passed |
| conformance-snapshot | red | `cargo conformance` does not exist |
| soft3-tests, soft3-release | red | cannot select a version of `cybergraph` |
| node-status | blocked | no release executable |

Reading: the default branches do not build together (launch row 39), and none of the repair is on them — zheng master is still 0.3.3 with the legacy fold as its public API. The repair lives on integration lines (§3).

## 2. stand REL — every stack top against its siblings' stack tops

Detached worktrees at `~/cyber/.stands/REL` (deleted after this run), one `CARGO_TARGET_DIR`, `cargo test --release`; `--locked` wherever the repository commits a lockfile. Revisions:

zheng 75f3846 (`feat/accumulation`, #51) · joy 639bcea (`feat/accumulation`, #37) merged locally with `feat/no-legacy` (#34) · nox 2f09ca3, trident a3cef15f (`release/0.4`) · lens 3cf2aaa (#19) · bbg 48153b2 (#37) · hemera cf28a64 (#17) · strata 81da2f6 (#14) · foculus c07bc15 (#63) · cybergraph a3b1232 (#15) · mudra 5b1b95f (#20; 9bca68c = #19 merged on top for its run) · inf a1fa041 (#8) · soft3 f19bef4 (#167) · cyb 26efc5b0 (#1407) · neuron fd24e76 · tade c7d404a, tok e27da0d, file fc666a5, radio 258724bd, tru 4cec250 (defaults) · honeycrisp 9d74126 (`feat/tip5`).

| repo | command | result | passed / failed |
|---|---|---|---|
| strata | `cargo test --release --workspace --no-fail-fast` (no lockfile) | red, pre-existing | 509 / 10 — all 10 in `nebu-wgsl --test gpu` (WGSL `vec2<u32>` → `u32` validation error); the same 10 fail on strata `main` |
| hemera | `cargo test --release --workspace` (no lockfile) | green | 306 / 0 |
| lens | `cargo test --release --workspace` (no lockfile) | green | 154 / 0 |
| nox | `cargo test --release --locked --workspace` | green | 244 / 0 |
| zheng | `cargo test --release --locked --workspace` | green | 275 / 0, 5 ignored |
| joy (#37 alone) | `cargo test --release --locked --workspace` | green | 224 / 0 |
| joy (#37 + #34, the tree now on `release/0.4`) | `cargo test --release --locked --workspace` | green | 215 / 0 |
| tok | `cargo test --release --locked --workspace` | green | 8 / 0 |
| foculus | `cargo test --release --locked` | green | 231 / 0 |
| foculus | `… --features accumulated-settlement` | green | 235 / 0 |
| inf | `cargo test --release --workspace --features inf-lower/prove` (no lockfile) | green | 102 / 0 |
| bbg | `--locked` | red | lockfile out of date against the sibling tops |
| bbg | unlocked | green | 88 / 0 |
| mudra | `--locked --features prove` | red | lockfile out of date |
| mudra | unlocked, `--features prove` | green | 50 / 0; with #19 merged: 51 / 0 |
| cybergraph | `--locked` | red | lockfile out of date |
| cybergraph | unlocked, `--workspace` | green | 59 / 0 |
| soft3 crate | `--locked` | red | lockfile out of date |
| soft3 crate | unlocked | green | 90 / 0 |
| cyb | `cargo test --release -p cyb-core` | red | the workspace needs `nu`, which has no usable git checkout to put in a stand (`../../.git/modules/vendor/nushell` missing) |

Per-repository result lines: `stand-rel/*.results.txt`; run order and exit codes: `stand-rel/summary*.txt`.

## 3. merge readiness and what was merged

Most PRs carry no CI; the stand of §2 is their build-and-test evidence. hemera's CI fails on `main` and on every PR the same way (the runner has no sibling checkouts for path dependencies); lens#14's likewise; cyb's Netlify checks fail and `master` requires an approving review, which the PR's own author cannot give.

Merge style: a merge commit everywhere except soft3 (squash), as in each repository's history. Stacked PRs were retargeted as their bases merged; no branch was force-pushed.

| PR | base | state 2026-10-10 | merge commit / why open |
|---|---|---|---|
| zheng#45 … #51 | `release/0.4` | merged | 3f9b8a8c, b77cc6cb, 3edf8e77, 9fed4743, 2ce706ca, 18f32a8e, ec82a4ba |
| zheng#52, #53 | retargeted to `release/0.4` | open, clean | held: the wrap step on #53 is in progress |
| zheng#54 | `feat/ivc` | open, conflicting with its base | held with #53 |
| joy#32 … #37 | `release/0.4` | merged | e007cacf, 222f6779, e826994d, e62e7eb8, 4ea20502, 3fc831cd — #36 and #37 conflicted with #34 in `rs/warrior.rs` (they patched legacy functions #34 deletes); resolved on their branches by merging `release/0.4` and keeping the deletion; #37's resolved tree is identical to the tested tree of §2 |
| joy#38 | retargeted to `release/0.4` | open, clean | needs zheng#53/#54 |
| lens#14 | master | open | owner decision; a version bump (cyber-lens 0.1.x → 0.2.0) |
| lens#17, #18, #19, #20 | the head of lens#14 | open, clean; green in §2 | merging would change the content of the owner's decision PR lens#14 |
| foculus#61, #62, #63 | the head of foculus#5 | open, clean; green in §2 | same, foculus#5 |
| cybergraph#15 | the head of cybergraph#3 | open, clean; green in §2 (unlocked) | same, cybergraph#3 |
| mudra#20 | the head of mudra#2 | open, clean; green in §2 (unlocked) | same, mudra#2 |
| soft3#167 | main | open | "merge after cybergraph#15" (its tests need it) |
| cyb#1407 | master | open, blocked | review required; consumes foculus#61 |
| bbg#37 | `chore/coordinated-release-20260916` | merged | ccda7a2e |
| bbg#36, #38 | master | merged | cdba7525, e502ce32 |
| nox#27, #28 | master | merged | ba1d4c86, 90288aad (#28's code lines are comments) |
| trident#124, #125 | master | merged | ee871350, ec986d8e |
| hemera#17, #18 | main | merged | d94390d7, 71acf488 |
| strata#14 | main | merged | 4d359726 |
| mudra#19 | master | merged | 4381b9e8 |
| inf#8 | main | merged | 5054c939 |
| cybergraph#16 | master | merged | 94fb982f |
| evy#2, #3 · cybics#1, #2 · glia#9 | main | merged | 598e426c, a194311a · f873aeaf, d4caa832 · 477d91c0 |
| radio#21 | main | merged | fd90677c |
| cyber#114, #115 | master | merged | c6fefa90, bb6ebd0e |
| soft3#162, #168 | main | merged (squash) | 677efd25, d9c225b9 — #162 conflicted with main's #163–#165 in the proposal; resolved on its branch by keeping the compact rewrite and folding in main's points (20 KB small-class gate, the hash row and phase 2h, full prototypes in the bake-off, the scale table, the SmallWood estimate, the LUNA+ qualifier) |

## 4. what stands between this and a green candidate

1. The 0.4 line reaches the default branches: zheng `release/0.4` (0.4.0; zheng#33 is the owner's decision PR for 0.4.0), nox `release/0.4` (0.3.0), joy and trident `release/0.4`. Owner.
2. The decision PRs that carry the integration heads: lens#14, foculus#5, cybergraph#3, mudra#2; bbg's `chore/coordinated-release-20260916` line has no PR. Owner. The green stacks of §3 then retarget and merge.
3. Lockfiles: hemera, lens, strata, file commit none; bbg, nox, tru, mudra, cybergraph, soft3 drift against siblings. One pin PR per repository after (1)–(2).
4. `cargo conformance` does not exist; the conformance gate cannot pass until it does.
5. `nu` needs a reproducible checkout before cyb can build in a stand.
6. The proof-size goal: profile 5 is constant (282–285 KB from 33 to 1,572,850 cycles) but over 64 KB and over 1 ms; the wrap step on zheng#53 is the open work.

## 5. rerun after the gate fixes — RED, 10 of 14 stack gates green

Same commands, from a clean worktree of soft3 `origin/main` d2cfe5f (after soft3#171), candidate `candidate-20261010.2`, `CARGO_TARGET_DIR` shared across the stack (so `soft3-release` would put its binary outside `crate/target`; moot here, the crate does not resolve). Receipts: `train-stack-rerun/` (snapshot as `snapshot-sources.json`). Archive `soft3-candidate-20261010.2-stack.tar.gz` sha256 `97a72e74ac0b36b84aa4780d99ddceef8747a5ace29abfcee8335f6bf7cf9a4f` (not committed).

Fix pull requests, each tested on a stand of origin default branches (`.stands/G2`, deleted afterwards) with the gate's own command, `cargo test --locked`, and the gate's zero-warning rule:

| PR | change | state |
|---|---|---|
| zheng#55 | cli `cyber-tape` at dead `../../tape` → `tade` (same alias as `release/0.4`, which needs nothing); stale lock | merged 5733dfd |
| foculus#64 | same for foculus; dead test bindings (warnings) | merged 8a935e1 |
| cybergraph#17 | `cyber-nox` 0.1 → 0.2; `stack_*` tests on nox 0.2 names (#10's test diff) and `Statement.bbg_root`; lock | merged 42fc251 |
| mudra#21 | `cyber-nox` 0.1 → 0.2, `zheng` 0.1 → 0.3; `bbg_root` no-state sentinel; lock | merged bff8dce |
| bbg#39, nox#29 | lockfile refresh (path crates only) | merged 441e3c9, 0b1407a |
| tru#31 | lockfile refresh; 16 example/test warnings | merged 2f3a255 |
| hemera#19, file#35 | commit `Cargo.lock` (was gitignored; the spec says committed lockfiles); hemera duplicate import | merged d5f0a09, e29b4c7 |
| lens#22 | commit `Cargo.lock` | open: green only together with lens#16 |
| strata#15 | nebu-wgsl: `gl_double` was called with a `vec2<u32>` in `fp3_norm`/`fp3_inv`; naga rejected the module, so all 10 GPU tests died at shader creation — a code bug, not the environment | merged 494df7a; workspace 519/0, gpu 11/0 |
| soft3#171 | `conformance-snapshot` builds `cargo-conformance` from soft3's `conformance/rs`; blocked (never green) while that crate is a scaffold | merged d2cfe5f |

| gate | before (§1) | after |
|---|---|---|
| origin-checkouts | green | green |
| phase1-pins | red, 15 drifted | red, 18 drifted (the fixes moved bbg, cybergraph, file, foculus, hemera, mudra, nox, strata, tru, zheng); `nu` has a remote now (cyberia-to/nu, main 1e58416) but no `rev` in the manifest; soft3#169 must re-pin |
| release-notes-source | red (cyb) | red (cyb: HEAD has no merged PR) |
| stack-hemera, stack-file | red, no lockfile | **green** |
| stack-bbg, stack-nox, stack-tru | red, stale lock | **green** |
| stack-zheng, stack-foculus | red, dead `../tape` | **green** |
| stack-cybergraph, stack-mudra | red, nox ^0.1 | **green** |
| stack-tok | green | green |
| stack-lens | red, no lockfile | red, no lockfile until lens#22; with #22 and lens#16 (or its duplicate #10, the porphyry `squeeze_field` panic for Fq) the stand gives 98/0 |
| stack-vault, stack-neuron | red | red: they need cybergraph features/modules (`local-storage`, `legacy-redb-migration`, `cybergraph::{application,content,native}`) that exist only on the owner's decision PR cybergraph#3 |
| stack-radio | red | red: `iroh-bench` names the crate `radio`; `iroh-docs`/`iroh-blobs` mix registry `iroh-base` 0.96 with the forked one; two `transfer` examples collide. radio#5 + #6 merged locally still leave `iroh-blobs` examples broken: fork maintenance, not a pin |
| conformance-snapshot | red, `no such command` | blocked: harness is a scaffold (soft3#171) |
| soft3-tests, soft3-release, package-resolution | red | red: soft3 main needs cybergraph#3 (`cybergraph::native`, `local-storage`) and foculus#5 (`signal_codec`, `decode_events_strict`); the lock refresh waits for those |
| node-status | blocked | blocked |

Owner items left: cybergraph#3 and foculus#5 (unblock neuron, vault, soft3 and its lockfile); merge lens#16 then lens#22 (refresh one lock line after); radio fork repair; a `nu` rev pin and the cyb release-notes PR; the conformance harness itself.

## 6. second rerun after the re-pin and soft3#172 — RED, 10 of 14 stack gates green, conformance green

Same commands, from a clean worktree of soft3 `origin/main` 18fd6b2 (after soft3#172, the conformance harness), candidate `candidate-20261010.3`, `CARGO_TARGET_DIR` shared across the stack, stand `.stands/G3` (deleted afterwards). Wall time 551 s. Receipts: `train-stack-rerun2/` (snapshot as `snapshot-sources.json`). Archive `soft3-candidate-20261010.3-stack.tar.gz` sha256 `2160dc6e3473bbbb4ac5ac8c3ddf74d1a036e9fcd41b3b8dced3eaa1d61309f2` (not committed).

Changes since §5: soft3#169 re-pinned (ab01cc3 on `chore/soft3-0.11.0`, not merged — bump PRs are the owner's): every sibling at its origin default head, `nu` pinned at cyberia-to/nu `main` 1e58416. radio#35 opened (the workspace build and test repair, carrying radio#5 and #6); not merged, see the radio row.

| gate | §5 | §6 |
|---|---|---|
| origin-checkouts | green | green |
| phase1-pins | red, 18 drifted | red from main's manifest, 18 drifted (bbg, cybergraph, evy, file, foculus, glia, hemera, honeycrisp, inf, mudra, nox, nu, radio, spark, strata, tade, tru, zheng). With soft3#169's manifest (snapshot from `origin/chore/soft3-0.11.0` ab01cc3, `snapshot-sources-pr169.json`): **0 drifted** — green once the owner merges #169 |
| release-notes-source | red (cyb) | red (cyb: captured product HEAD has no merged pull request) |
| stack-hemera, -bbg, -nox, -zheng, -cybergraph, -foculus, -tru, -tok, -mudra, -file | green | green |
| conformance-snapshot | blocked (scaffold) | **green** (soft3#172) |
| stack-lens | red, no lockfile | red, no lockfile (lens#22 + lens#16 open) |
| stack-vault, stack-neuron | red | red: `cybergraph` without `local-storage` — needs cybergraph#3 |
| stack-radio | red | red on main 344ac162: `iroh-bench` names the crate `radio`, `iroh-docs` mixes registry `iroh-base`, `transfer` examples collide. radio#35 fixes all three plus the `iroh-blobs` 64-byte-hash leftovers (`Hash::EMPTY` was a zero placeholder; 27 library failures and hangs before): on its branch `cargo test --locked` has 0 warnings and 578 passed / 3 failed with `--no-fail-fast`. The 3 are `address_lookup::mdns` timeouts — upstream iroh 0.96.1 fails them identically on this machine and 224.0.0.251 routes into the VPN tunnel `utun4` — so the gate stays red on this Mac until mDNS reaches the LAN (`radio/audit/2026-10-10-workspace-gate/` on the PR branch) |
| soft3-tests, soft3-release, package-resolution | red | red: `cybergraph` has no `local-storage` (cybergraph#3), foculus#5 |
| node-status | blocked | blocked: no release executable |
| source-inputs-unchanged | green | green |

Owner items left: merge soft3#169 (pins); cybergraph#3 and foculus#5 (neuron, vault, soft3 and its lockfile); lens#16 then lens#22; radio#35 (and a decision on mDNS on the train machine); the cyb release-notes PR.
