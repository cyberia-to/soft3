---
title: the director's day — 2026-10-31
tags: soft3, release, director, audit
date: 2026-10-31
---
# the director's day — 2026-10-31

per `release/director/README.md` § the measurement. Sessions append. Each number names its run or PR.

## the candidate

**Red.** The newest train run on the defaults is [41097270082](https://github.com/cyberia-to/soft3/actions/runs/41097270082) (`schedule`, `main`, 2026-10-30 17:15 UTC). Every leg is red at package-resolution: `cyber-lens-gpu` fails with `failed to read …/sources/lens/gpu/Cargo.toml`, because lens `master` lacks the 0.2 line until [lens#36](https://github.com/cyberia-to/lens/pull/36) lands, and that landing is the owner's approval. The other landing PRs are also the owner's: hemera#25, nox#43, radio#52, soft3#376.

On pussy's pinned stand (`release/pussy-stand.toml`), rehearsal [41041110233](https://github.com/cyberia-to/soft3/actions/runs/41041110233) gives these results:

| leg | soft3-tests | soft3-release | node-status | reds |
|---|---|---|---|---|
| stack | green | green | green | phase1-pins, release-notes-source, stack-zheng (L3b), stack-mudra (L3c), stack-neuron (L3c) |
| x86_64-unknown-linux-gnu | green | green | green | phase1-pins, release-notes-source |
| aarch64-unknown-linux-gnu | green | green | green | phase1-pins, release-notes-source |
| aarch64-apple-darwin | green | green | green | phase1-pins, release-notes-source |
| x86_64-apple-darwin | green | green | green | phase1-pins, release-notes-source |

On the stack leg, stack-chain-genesis, conformance-snapshot and simulation-gate are also green.

## units done

- session 394: **P1** done. The rehearsal's overrides equal the stand file, 16 of 16, and its acceptance gates are green on all five legs. The remaining reds have rows (G8, G9, L3b, L3c).

## PRs merged

Into soft3 `release/phase1`: this session's plan PR.

## plan items left

P2: both diagnostics of #479 ended `failure`, including the green one, [41042360873](https://github.com/cyberia-to/soft3/actions/runs/41042360873). Also left: P2a, the owner's landings and asks (`plan.md` § what only the owner does), and R1.

## fable and windows

Fable questions asked: 0. Subscription windows hit: none.
