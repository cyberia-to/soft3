---
title: soft3 on hemera 0.4 — the phase-1 root chain re-derived under profile #15
tags: cyber, soft3, audit, chain, hemera, genesis, conformance
crystal-type: audit
crystal-domain: cyber
---

# soft3 on hemera 0.4 — the phase-1 root chain re-derived under profile #15

Date: 2026-10-16. Sweep part 1b of hemera 0.4 (profile #15, merged on
hemera `release/0.4` `7237a23`; receipt `hemera/audit/profile-0.4-2026-10.md`
§5 names what soft3 must move). Branch soft3 `feat/hemera-0.4` on
`origin/main` `2d55fdb`. Contracts: `specs/chain.md`,
`specs/chain-conformance.md`, `specs/genesis.md`.

Stand `~/cyber/.stands/S1B`, detached worktrees: soft3 `origin/main`,
hemera `origin/release/0.4` `7237a23`, bbg `origin/master` `eb152fa`, zheng
`origin/release/0.4` `09308d7`, lens `origin/release/0.2` `e11eaa8`, nox /
joy / trident `origin/release/0.4`, cybergraph `feat/durable-applications`
`0eae7ae`, mudra `refactor/spell` `4bfe3e8`, foculus
`feat/device-profile-choice` `b28f0a2`, honeycrisp `origin/main` (lens
`gpu/`'s `aruminium` path), tade / strata / tru / tok / radio / neuron / inf
/ file defaults. Machine: M4 Max, 48 GB, shared with other agents' builds
throughout (1-minute load 20–90). Build: `cargo build --release
--no-default-features` — the CPU miner only; the GPU miner follows lens
(agent S1a, lens `feat/hemera-0.4`, not yet on `release/0.2` when this ran).

## 1. the dependency

- `crate/Cargo.toml`, `conformance/rs/Cargo.toml`: `cyber-hemera` `0.3` → `0.4`
  (path kept); both lockfiles move the path entry to 0.4.0. radio keeps its
  own git-pinned `cyber-hemera` 0.3.1 (`d5f0a09`) inside the wire crates —
  radio's sweep, not soft3's; the chain's topic ids are computed by soft3's
  hemera.
- `release/phase1.toml`: hemera pin `main` `23f3bbc` → `release/0.4`
  `7237a23afad38913a143da48010f3912f50d7684`, `updated = "2026-10-16"`.
- `conformance/rs/Cargo.toml` also moves bbg `0.2` → `0.3`, nox `0.2` →
  `0.3`, zheng `0.3` → `0.4`, lens `0.1.3` → `0.2`: the committed requirements
  did not resolve against any of the pinned siblings.
- **Every sibling still requires `cyber-hemera = "0.3"` with its path dep**
  (bbg `rs/Cargo.toml`, cybergraph, mudra, foculus, lens workspace, nox,
  joy, trident, tru, tok, zheng). Cargo refuses a 0.4.0 path crate for a
  `^0.3` requirement, so each was bumped to `"0.4"` *in the stand only*
  (uncommitted). bbg master does not "follow hemera automatically" until
  its one-line bump lands; the same line is owed by every repo above
  (hemera receipt §5, first paragraph). Nothing in soft3's dependency
  graph uses the removed API (`Goldilocks::inv`, `PartialSbox`,
  `Profile::new`, `RoundVisitor`); the `.inv()` calls in foculus, nox, lens
  and strata are on nebu's field, not hemera's.

## 2. the vectors (`specs/chain-conformance.md`)

Every pinned digest in `crate/src/chain/{sigs,header,pow,tree,genesis,
net/codec}.rs`, `genesis/tests.rs`, `wallet/tests.rs`, `mine/{mod,bench,
target}.rs` and §1, §5, §6, §7, §12 of the spec was re-derived through the
chain's own code (a throwaway printer linking the crate; the tests then
assert the same values). The definitions did not move.

| value | hemera 0.3 | hemera 0.4 |
|---|---|---|
| `hemera("")` | `a67a71b2…563c` | `9e89f2487be432e48c27e24e6945adffe6918c8b5d6a62061e70e0bebd657326` |
| `token("boot")` | `10275cf6…1269` | `1d443e1103abbd0aebe4abaf3fadc203160a471e9accb9b6e0a73804ea785b3b` |
| `token("pussy")` | `8cffbea0…8ffa` | `87a70ff9ed0638fc9fafb1a30f57c5ad2ebe5155c48702c50bdbb2ece87f6a71` |
| `hemera(168 zero bytes)` | `ba560d1f…1754` | `355fd03315a5ceb056cda53d11ff7b9755167182d59ff9e31134d69d33bdf0f6` |
| §5 test header id | `bdfbbff9…b58d` (byte 31 `0x8d`) | `bdd7f2ef3a5a1c573b8d84a7a2aeedc95c6f8aa2139eea5c83946b33c4180c31` (byte 31 `0x31`) |
| coinbase txid / leaf / node | `d7662bdb…` / `c2cb6adb…` / `3685e0a5…` | `6cf0e4d2…c3f9` / `514d5d57…fe9c` / `9d68ac9b…595b` |
| transfer statement | `c7e4a223…4a6f` | `41e14955…12a7` |
| `neuron_of(abandon pubkey)` | `31284576…d741` | `55c76c9f172b850eaa82e4c6defe13b1d80d983c29d81526eb94d2c8199b0fd2` |
| `legacy_owner(pussy, account_id)` / `(pussy, 0x99×20)` | `cca6f468…` / `d5e54c49…` | `4382c7f5…e373` / `5384913f…9f22` |
| `topic(0x44, "headers")` | `1f003a48…8b8b` | `598b5696…d7a8` |
| §12 tags digest (64 B) | `d4ea8f2b…816b` | `0056e480f212c69488b71289b7c3e170f015c6411267b915a1596fae4f9fc4a6` `4faaecefa7d0f0a8b75c0d76902da823b6c45ca46e8358f6578848e415dff0b1` |
| §12 `E_40` (genesis pool anchor) | `9a47bbbc…3aa5` | `3ce6bca0…d1eb` `8bcf4448…b395` |
| `T(token("pussy"))` | `ddd4a6ce…4ae1` | `4b41f4e2…1b05` |

The full §12 table (owner, rhos, commitments, nullifier, roots, spend id,
vk set, statement id) is in the spec; `chain.snap` carries their
fingerprints.

### the mined header, re-mined

The §5 PoW-valid header (`bits = 0x1e010000`, `time = 1793885850`, the test
header's other fields) was re-mined, not replaced: a linear nonce search
from 0 through `Header::id` (= `hemera::hash` over the 168 bytes) and
`valid_pow` — the kernel's own path, no miner backend.

```
nonce    934636 = ec420e0000000000 0000000000000000        (0.3: 40165933)
block_id a3dc0cdb7c69e1d4986c388effe1612613b1b71f2c2ca49084bc0730da000000
```

934,637 hashes; one thread 10.4 s = **90.3 kH/s on one core** (0.3:
64.4 kH/s, 623 s for its 40.2 M hashes) — 1.40× for the sponge over 168
bytes, against 2.2× for the bare permutation (hemera receipt §3: the
sponge's four permutations are latency-bound like the single row). Twelve
threads found the same nonce (least index, 1.52 M hashes, 2.2 s). Byte 28
is `0xda`, so the harness's rejection at `bits = 0x1d100000` (needs byte 28
below `0x10`) holds; nonce 934,635 fails. `chain.md` §1.2, §1.3 and §8
carry the new rate (order 0.9 MH/s per ten cores, floor block ≈ 12 s on
one core); the `mine bench` vector check and the backend-agreement test
search from base 934,000 for index 636.

The CPU miner (`permute_batch`, 16 lanes) measured 91–104 kH/s per thread
inside the fleet below on the loaded machine; hemera's receipt measures 0.4
batched at 0.6× of 0.3 batched, so the CPU miner is not faster on 0.4 even
though single hashing is. The miner's sponge view (`mine/job.rs`) is
unchanged: same rate, same limbs, same length lane.

## 3. genesis

`soft3 genesis build --chain <c> --snapshot ~/cyber/<bootloader>/snapshot
--out ~/cyber/.cache/genesis/<c>.tade` (cache `~/cyber/.cache/snapshots/<c>`,
pinned inputs by sha256; bbg dimension version 3). `derive.rs` re-derives
every particle (`hemera(cid)`, the black hole of launch row 20 — 74,514
CIDs have no bytes on the cybernode) and every owner id.

| | pussy | bostrom |
|---|---|---|
| particle (chain id) | `bae85f40d9f508664b5214107551be95d3f188d9585a899178c23e80886720e5` (0.3: `1bb7d6aa…`) | `f78c60edd6aea30af0da261233334e7549e63ece912a82c8d5b9e4df3cadb425` (0.3: `d984145f…`) |
| file | 3,249,657 B, sha256 `ec7aee793fe3019355a5ead95652640731dc9c632b6d400d7288fee7e6ee2d72` | 330,841,520 B, sha256 `ffa3d6dd89839224192f3fa268651fbb87923435f66de872b0f14c833e446443` |
| state root | `a360798987fb821f3bb7e07c66947e91e1eacb4278a834972c32ea8b3054b8b2` (0.3: `93e4660f…`) | `c50a6d8438ed8b0ef72450e7f83cab81e78bfd4e833106add9d62a0831bcdf13` (0.3: `ce3052f4…`) |
| block-0 id | `bcb1e9b0826283172993108dc22c52b457e1582f8148adf035046d976288092f` (0.3: `5669e96e…`) | `9ab9fc49967caa7c5324f877512254969fcb56fada55b57cf0ba0b2187d3aa67` (0.3: `be6f8900…`) |
| S0 / SUBSIDY / time | 1,099,389,810,279,114,036 / 10,458,426,657 / 1793885820 (unchanged) | 482,287,925,778,234 / 4,587,974 / 1793884962 (unchanged) |
| frames | b 1,385, l 29,113, 23 tokens (unchanged) | b 93,472, l 2,997,569, 49 coins (unchanged) |
| particles derived | 16,271 | 3,143,650 |
| wall / user / memory | 4.2 s / 0.85 s / 73 MB RSS | **327 s wall / 110.5 s user / 5.9 GB max RSS (7.35 GB peak footprint)**, root 236 s |
| bbg records | 0 particles, 6,685 / 15,061 axons out/in, 26,805 axons, 547 neurons | 0 particles, 1,701,082 / 1,772,963, 2,990,817 axons, 60,452 neurons |

The file sizes, counts and sums are byte-for-byte the 0.3 shape; only the
ids moved. Previous run (0.3, same day): `audit/genesis-dimension-v3-2026-10-16.md`
(bostrom 204–268 s, 7.5 GB) — this run was slower on a machine at load
70–90, not from 0.4 (user time 110 s against 189 s).

## 4. conformance

- `soft3 conformance bless chain --genesis ~/cyber/.cache/genesis/pussy.tade`:
  **30 of 30 entries moved** (every entry is a hemera value); `soft3
  conformance check chain` green against the rebuilt file;
  `conformance/chain-genesis.toml` rewritten with both chains' values and
  the hemera pin.
- `cargo test --release --no-default-features --features chain-conformance
  --locked`: 26 of 215 lib tests failed on the old vectors before the sweep
  (every one a pinned digest, none a logic failure); after it **215 lib +
  12 wallet + mine e2e + adversary pass** (sections 1–9, 11, 12; §10
  through the cached pussy file). The chain tests do not depend on zheng
  proofs: no `zheng` reference in `crate/`.
- **The stack conformance harness (`conformance/rs`, snapshots
  `mechanism.snap` / `encoding.snap` / `manifest.snap` / `provenance.toml`,
  `surfaces/hemera.rs`) could not be re-blessed**: it links zheng, whose
  `release/0.4` head `09308d7` does not compile against hemera 0.4
  (`RoundVisitor::partial_round` takes `[x², x³]` now — the gadget in
  `rs/src/execution/hash.rs`, hemera receipt §5) once its `legacy` surface is
  on, and the harness's `surfaces/zheng.rs` uses only that retired API
  (`commit`/`open`/`verify_eval`/`row_hash`, gated behind `legacy` in 0.4).
  Waits on S2 (zheng `feat/hemera-0.4`), then on a zheng-surface rewrite.
  nox `release/0.4`'s hash pattern is still the 24-round 0.3 trace
  (`rs/patterns/hash.rs`); its surface entries move again with nox's sweep.
  The hemera surface's doc line and `conformance/specs/README.md` name 0.4;
  the `.snap` files are untouched (still the 2026-10-10 bless).

## 5. the fleet

`nu scripts/chain-fleet.nu --threads 1 --keep --http-base 17980 --radio-base
17990` (the flags are new: the default range 17780/17790 was held by another
session's SSH tunnel and `pussy-rc` node, and the script only kills what it
spawned). Run 1 (20 s window, load 70–90, cargo builds running): peers
connected, 2 blocks in 20 s — RED on the `≥ 3` check: 2^20 expected hashes
per floor block at 91–104 kH/s per thread is ≈ 10 s per miner, 5 s per
block for two miners, and the window was short on a loaded machine (the
script's comment "a block per 4 s" is a 0.3-batched figure). Run 2 (40 s
window, load still 70–90): **GREEN** — peers connected (2 each); heights
agree at 5 across 3 nodes, block 1 `aea953a4…` identical everywhere; the
late joiner IBDs to height 5 by ranges; a tx submitted to node 1 is mined
at height 8 by node 2's miner and bob holds 5 on node 3; a fork of depth 3
from height 8 (main 11, fork 13) reorgs every node to the heavier chain;
node 3 killed at height 13 restarts from disk, replays, syncs to 17,
conserved. Log: `~/cyber/.stands/S1B/fleet-2.log` (stand deleted after
the sweep; the run is reproducible with the command below).

## 6. what waits

- **S1a** (lens `feat/hemera-0.4` on `release/0.2`): the Metal / WebGPU
  miner (`gpu/`: `ginv`, `RC[144]`, the shared inversion). This branch
  builds `--no-default-features`; `every_backend_agrees_and_verifies` and
  `mine bench` run the CPU backend only. When lens lands, the default
  features build again and the GPU backends must find index 636 from base
  934,000.
- **S2** (zheng `feat/hemera-0.4`): the stack conformance re-bless (§4), the
  zheng proof fixtures under `conformance/fixtures/`, and `surfaces/zheng.rs`.
- **sibling bumps** (`cyber-hemera = "0.4"`): bbg, cybergraph, mudra,
  foculus, lens, nox, joy, trident, tru, tok, zheng — one line each; until
  bbg's lands, soft3 main does not resolve against hemera `release/0.4`
  without a stand-local edit.
- `cyber/launch.md` row 14 carries the new ids (cyber PR).

## reproduce

```
cargo build --release --no-default-features                      # crate/
cargo test --release --no-default-features --features chain-conformance --locked
soft3 genesis build --chain pussy   --snapshot ~/cyber/space-pussy/snapshot --out ~/cyber/.cache/genesis/pussy.tade
soft3 genesis build --chain bostrom --snapshot ~/cyber/bostrom/snapshot     --out ~/cyber/.cache/genesis/bostrom.tade
soft3 conformance check chain --genesis ~/cyber/.cache/genesis/pussy.tade
nu scripts/chain-fleet.nu --threads 1 --secs 40 --http-base 17980 --radio-base 17990
```
