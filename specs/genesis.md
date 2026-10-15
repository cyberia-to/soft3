---
title: genesis manifest
tags: cyber, soft3, spec, genesis, bostrom, pussy
crystal-type: spec
crystal-domain: cyber
alias: genesis pipeline, genesis manifest, soft3 genesis
---

# genesis manifest

The contract between the burial snapshots of the bootloader chains and the
phase-1 PoW root chain (owner decision 2026-10-15: pure-PoW root chain on
soft3, pussy first, bostrom when its state commits). The pipeline is
`soft3 genesis build`; the artifact is a directory of canonical tables plus a
hashed `manifest.json`; the node loads that directory through
`soft3::genesis::state::load` and must reproduce the recorded state root.
Companion contracts: `cybergraph/specs/genesis-replay.md` (bijection),
`mudra/specs/bridge.md` (claims), `specs/chain.md` (the chain that consumes
this). Implementation: `crate/src/genesis/`.

## inputs

One source manifest per chain, in its repository:

| chain | manifest | halt | links restored |
|---|---|---|---|
| pussy | `space-pussy/snapshot/manifest.json` | 19,940,993 · 2026-06-14T08:36:41Z | 29,112 of 48,370 (19,258 unrecoverable) |
| bostrom | `bostrom/snapshot/manifest.json` | 25,120,712 · 2026-08-05T08:55:00Z | 2,949,732 of 2,949,732 |

Every file the manifest pins (`cid`, `sha256`, `bytes`) is fetched from IPFS
into `~/cyber/.cache/snapshots/<chain>/` (never in git) and verified by
sha256 and size on every run, cached or not. Files read: `supply.json`,
`pubkeys.csv`, `balances.csv`, `delegations.csv`, `holdings.jsonl`,
`passports.jsonl`, `cyberlinks_indexed.csv.gz`. Gateway layout: bostrom
`<cid>/<name>`, pussy `<cid>/pussy-snapshot/<name>`; the burial sites are the
fallback. `state_export.json.gz` is not read.

## command

```
soft3 genesis build --chain pussy|bostrom --snapshot DIR --out DIR [--cache DIR] [--no-root] [--time UNIX]
soft3 genesis verify DIR [--root]
```

`build` is deterministic: two builds of the same snapshot produce
byte-identical tables, the same `manifest.json` and the same hash. `verify`
recomputes every table hash, the manifest hash and (with `--root`) the state
root from the directory alone.

## output

```
<out>/
  accounts.tsv            address · owner · kind · pubkey_type · pubkey
  balances.tsv            address · owner · denom · amount          (loaded into bbg)
  balances-deferred.tsv   address · denom · amount                  (carried, not loaded)
  particles.tsv           particle · cid
  cyberlinks.tsv          from · to · neuron · height · time · tx · address   (snapshot order)
  passports.tsv           card · owner · particle · token_id · nickname · avatar · address
  manifest.json           canonical JSON, sorted keys, no whitespace
  genesis.hash            sha256(manifest.json)
  header.json             genesis block header stub (height 0)
```

Tables are UTF-8, tab-separated, `\n`-terminated, header line first, one row
per record, 32-byte ids as lowercase hex, amounts as decimal integers, empty
optional fields as `-`. Sort orders: accounts by address; balances and
deferred balances by (address, denom); particles by CID; cyberlinks and
passports in snapshot order.

`manifest.json` carries: `format` (`soft3/genesis/v1`), chain, hrp, native
denom, bootloader chain id; `source` (the source manifest's sha256, halt
height and time, and every pinned file's cid/sha256/bytes); `tables` (sha256,
rows, bytes of each table); `counts`; `supply` per loaded denom; `decisions`;
`gaps`; and, once the state is committed, `state_root`, `state_counts` and
`state_fields` (or `root_blocked`, see below).

**The genesis hash is `sha256(manifest.json)`**; it commits to every table
through their hashes and to the snapshot through the source manifest's hash
and CIDs. `header.json` is `{chain, protocol, height: 0, time, time_iso,
prev: 0x00…, genesis_hash, state_root}` — the time is `--time` (default
2026-11-05T00:00:00Z = 1793836800) and is outside the manifest hash.

## derivation

Integers only. Micro-unit amounts are parsed as `u128`, summed as `u128`, and
must fit `u64` before they reach bbg. Any non-digit amount is a hard error.

### accounts

Every address the snapshot names (auth accounts, bank rows, delegators,
unbonding entries, link authors, passport owners) is one account, once.

- **neuron** — the snapshot holds a `/cosmos.crypto.secp256k1.PubKey` for the
  address: `owner = mudra::claim::neuron_of(pubkey) = hemera(compressed pubkey)`.
  The builder recomputes `bech32(hrp, ripemd160(sha256(pubkey)))` and refuses
  a key that does not derive its address.
- **claimable** — no usable key (never signed; multisig; absent from auth):
  `owner = hemera("soft3/genesis/claimable/v1\0" ‖ u64le(len) ‖ address)`.
  The balance sits under that id until the holder binds a neuron through the
  legacy bridge (`mudra/specs/bridge.md`): a verified ADR-036 claim moves the
  claimable balance to `neuron_of(pubkey)` — the claim verifies that the
  pubkey derives the claimable address, so no second table is needed. Module
  and contract accounts are claimable by construction and are expected to
  stay unclaimed; their disposition is a launch decision outside this spec.

### balances

For the native denom (`pussy`; `boot`):

```
genesis(a) = liquid(a) + delegated(a) + unbonding(a)
bonded_tokens_pool      ← bonded + not_bonded − Σ delegated − Σ unbonding   (dust)
not_bonded_tokens_pool  ← 0
Σ genesis == bank supply of the denom at halt                                 (hard check)
```

`delegated(a)` is the integer balance column of `delegations.csv`;
`unbonding(a)` the integer `undelegating` of `holdings.jsonl` (the extractor's
sum of the state export's unbonding entries). Delegations become liquid at
genesis; the validator set is not carried (pure PoW). Module accounts
(`distribution`, fee collector, pools, ibc escrow) are carried under their
bech32 module address so the sum equals the bank supply exactly.

bostrom additionally mints `cyb`: every TOCYB becomes 666 CYB per account,
`Σ cyb == tocyb supply × 666 == supply_audit.tocyb_to_cyb.genesis_cyb` (hard
check). Every other bank denom (ibc vouchers, pool coins, hydrogen, ampere,
volt, liquidpussy, factory tokens) goes to `balances-deferred.tsv` raw and is
not converted — rates are pool-derived and belong to a later, separately
audited decision.

In bbg: `coins[hemera(denom)] = total_supply`,
`balances[balance_key(owner, hemera(denom))] = amount`.

### particles

Every CID a restored cyberlink names becomes
`particle = hemera(cid bytes)` — the no-bytes branch of `file::readdress`
(launch row 20) — uniformly, because genesis fetches no content. The
`files` dimension marks every genesis particle `available = false`; the
content particle `hemera(bytes)` sparks when the bytes surface, and
`particles.tsv` is the CID → particle table that names it. This keeps the
black-hole and present-bytes identities on one rule until row 19 settles the
particle definition; when it moves, every particle here moves with it.

### cyberlinks

One row per restored snapshot row, in snapshot order (bijection: row *i* of
the source is row *i* of `cyberlinks.tsv`), carrying the original neuron
(owner id of the author address), height, block time (unix seconds) and tx
hash. Loading: `amount 0, valence +1, token 0`; consecutive rows with the same
(author, height, tx) form one `bbg::Signal`; `signals[position]` records
neuron, `network = hemera("soft3/genesis/network/v1\0" ‖ u64le(len) ‖ chain)`,
link count, block height, `proof_hash = 0`. Duplicate (from, to) pairs
replay as the engine replays them (one axon, weight unchanged); the counts
table reports both links and axons.

### passports

One card per passport: `card = hemera("soft3/genesis/passport/v1\0" ‖ chain ‖
token id)`, `owner` = the owner's genesis id, `particle = hemera(nickname)`.
The avatar CID and nickname are carried in `passports.tsv`; no naming
cyberlink is minted at genesis (it would break the links bijection).
`holdings.jsonl`, `pools.json`, `prices.json`, `validators.json` are derived
views, not genesis records.

### state root

`soft3::genesis::state::load` builds a `bbg::BbgState` in memory — neurons,
coins, balances, particles + files, signals via `BbgState::insert`, cards —
with `height 0` and empty `time`/`locations`, then `root()` commits every
dimension once. This is the bulk-build path of launch row 43: no per-batch
recommit, so the pussy root is seconds, not days.

**Commitment limit.** lens Brakedown commits one dimension as a multilinear
polynomial of at most `2^20` field elements (`MAX_VARIABLES = 20`). A
dimension serializes to `3 + Σ(8 + values)` fields (`bbg::dim`). When any
dimension exceeds the limit the builder records `root_blocked` (dimension →
fields and the variables it needs) instead of a root; the manifest and its
hash are still produced. bostrom's particles dimension needs ~2^27 — see
`audit/genesis-2026-10-15.md` — so its root waits on bbg/lens, not on this
pipeline.

## verification

1. source counts: the csv row count must equal the manifest's declared
   restored (pussy) or on-chain (bostrom) link count, else the snapshot
   changed and the build stops.
2. conservation: per loaded denom, Σ genesis == supply (hard check in
   `derive`).
3. bijection: `counts` carry both source rows and genesis rows per table; the
   test asserts links == rows, accounts == auth accounts, particles ==
   distinct CIDs, passports == rows, unique addresses and unique owner ids.
4. determinism: two builds, identical bytes; `verify --root` re-derives the
   root from the directory alone.
5. spot checks: passport contract is claimable, staking-pool dust after
   delegations became liquid, first link author and height, every neuron
   owner recomputed through mudra from its pubkey.

Observed results live in `audit/genesis-2026-10-15.md`, never here.

## open

- `specs/chain.md` fixes the chain id, genesis time and how the node consumes
  this directory; `state::load` is the loading rule it must call.
- the claim flow that moves a claimable balance to a neuron at runtime
  (`soft3 claim`, launch row 15) reads `accounts.tsv`.
- deferred denoms, module-account disposition, the row-19 particle
  definition, and the lens commitment limit for bostrom.
