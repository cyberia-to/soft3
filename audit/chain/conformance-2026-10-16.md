---
title: chain conformance — kernel reference run 2026-10-16
tags: cyber, soft3, audit, chain, conformance
crystal-type: audit
crystal-domain: cyber
date: 2026-10-16
---

# chain conformance — kernel reference run 2026-10-16

Receipt of the first run of chain-conformance.md on the consensus kernel
(`crate/src/chain/`, branch `feat/chain-kernel`, revision `1243ef2` plus
the engine split committed with this file). Per the audit doctrine:
revision, command, observed result; the requirements stay in the specs.

## commands

```
cd soft3/crate
cargo test                                   # 168 lib + 11 bin tests, 0 failed (212 s debug)
cargo test --lib chain::                     # 66 chain tests (19 engine scenarios)
cargo build --release                        # 0 soft3 warnings (3 pre-existing in bbg's vendored fjall)
soft3 conformance check chain                # chain.snap: 9 entries, no drift
```

Stand: `~/cyber/.stands/N1` — soft3 `feat/chain-kernel` over main `8525b17`
(wallet #184, genesis #183), siblings at the make.sh integration base
(bbg `1d39975` 0.3.0, cybergraph `feat/durable-applications`, lens
`release/0.2`, nox `release/0.4`, zheng `feat/public-certificate-v3`, mudra
`refactor/spell`, hemera/strata/tade/tru/tok/radio/foculus defaults).
Genesis file: `~/cyber/.cache/genesis/pussy.tade` (3,249,582 bytes) from
`soft3 genesis build`.

## results by section

| section | result | where |
|---|---|---|
| §1 hemera anchors | green — empty, `"boot"`, `"pussy"`, 168 zero bytes | `chain::sigs::tests` |
| §2 compact target | green — 7 canonical, 6 rejections | `chain::header::tests` |
| §3 work | green — 2^228/2^232/2^236/1, 3-block sum 50,331,645 | `chain::u256::tests`, `chain::pow::tests` |
| §4 ASERT | green — all 6 rows with `e`, `shifts`, `frac`, `factor`, bits (row 5 pins floor toward −∞ and the cubic; row 6 the clamp); T-spacing holds bits; T/2 for 600 s halves the target; 100× slowdown back at T within 70 min | `chain::daa::tests` |
| §5 header | green — bytes, id `bdfbbff9…`, 167/169 bytes, version 2, wrong chain; mined header `d1eec40d…` valid at 2^232, rejected at 2^228, nonce −1 fails | `chain::header::tests`, `chain::pow::tests` |
| §6 transactions | green — coinbase bytes/txid/leaf/node, transfer statement `c7e4a223…`, signed transfer verifies and fails on every byte flip / 101 / 103 bytes / other chain; cyberlink 257 B, valence 2 and p == q rejected; claim account `28ff5c6d…`, legacy owners `cca6f468…` and `d5e54c49…`; encoding rejections every kind | `chain::sigs::tests`, `chain::tx::tests`, `chain::tree::tests`, `chain::tests::rules` |
| §7 identities | green — `neuron_of`, tokens, topic `1f003a48…` | `chain::sigs::tests` |
| §8 state | green, blessed — empty root, conformance genesis (file, root, block 0), after block 1 and after a cyberlink in the engine, maturity 100 rejected / 101 accepted, conservation at every height of a 200-block random chain through a 3-block reorg | `conformance/chain.snap`, `chain::tests::{rules,simulate}` |
| §9 consensus | green — heaviest chain, first-seen ties, reorg from tip−120 rejected / tip−119 taken, checkpoint vetoes a heavier fork, foreign signer and past-sunset checkpoints dropped, PoW error before the body, wrong state root marks the header invalid, `time == MTP` rejected / `MTP + 1` accepted, `now + 121 s` held then accepted; three stores converge | `chain::tests::forks` |
| §10 genesis | green, blessed — pussy particle `2a83e54f…`, state root `f2296834…`, block-0 id `b863ce28…` reproduced by the kernel's own path (`Genesis::state` → `genesis::state::load`); bostrom stays red (lens commitment limit, audit/genesis-2026-10-15.md) | `chain::genesis::tests`, `conformance/chain.snap` |
| §11 wire | not this kernel — PR #187 (`chain/net`) | — |
| orphan pool | not this kernel — the net layer's pool | — |

Blessed entries written by `soft3 conformance bless chain` (9): the three
genesis lines unchanged from the hand-written file, plus `chain::tx@v1`
signed transfer / cyberlink fingerprints and `chain::state@v1` empty root,
conformance genesis file / root / block 0.

## caveats, stated plainly

- Nonce placement: bbg at the integration base has no `NeuronRecord.nonce`;
  the kernel keeps the nonce in a side map beside the state (feature
  `bbg-nonce` off) — every rule holds, the nonce is not yet under
  `state_root`. bbg branch `feat/chain-dims` (`97c6272`, 56 tests green
  against bbg master's siblings) carries the field as the fourth neurons
  leaf; when it lands with DIMENSION_VERSION 3, flip the feature default
  and re-bless `chain.snap` in the same PR (every root moves).
- Conservation is checked from per-token running sums the state backend
  maintains through its own trait (`Σ balances == supply` for every token,
  native supply `== S0 + h·SUBSIDY`), not by `tok`: tok's `MintLedger`
  owns settle-mint conservation over its own balance map and cannot sum
  bbg balances; chain.md §3.4's "tok owns the arithmetic" is not met
  literally.
- Chainwork of block 0 is 0 (genesis is not mined); chainwork(h) sums the
  work of blocks 1..h.
- The 200-block simulation and the 130-block reorg scenarios run at a test
  target of 2^255 through a `cfg(test)`-only override of `POW_LIMIT`; no
  release path has that hook.
- `cargo test` of the crate takes ~210 s in debug: the pussy genesis root
  (35 s) and the fork scenarios dominate.
- The pool (chain.md §11, in flight on `specs/pool-formats`) will move the
  genesis `p` params and every blessed root; rule set 1 rejects every kind
  ≥ 4 already and the activation table names no rule set 2 yet.
