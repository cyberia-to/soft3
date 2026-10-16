---
title: genesis pipeline
tags: cyber, soft3, spec, genesis, bostrom, pussy
crystal-type: spec
crystal-domain: cyber
alias: genesis pipeline, soft3 genesis, genesis builder
---

# genesis pipeline

How the burial snapshots of the bootloader chains become the genesis file of
the phase-1 PoW root chain. The file format, the derivation rules and block 0
are normative in [chain.md §4](chain.md#4-genesis); this page is the
pipeline that implements them (`crate/src/genesis/`, `soft3 genesis`), the
decisions it takes where §4 leaves room, and its debug export. The node
(`chain/genesis.rs`) consumes the file through `soft3::genesis::state::load`
and must reproduce the recorded block-0 root. Companion contracts:
`cybergraph/specs/genesis-replay.md` (bijection), `mudra/specs/bridge.md`
(the legacy bridge), [chain-conformance §10](chain-conformance.md#10-genesis-pipeline)
(the blessed vectors).

## inputs

| chain | manifest | halt | genesis time (§4.3) |
|---|---|---|---|
| pussy | `space-pussy/snapshot/manifest.json` | 19,940,993 · 2026-06-14T08:36:41Z | 1793885820 (2026-11-05T13:37:00Z) |
| bostrom | `bostrom/snapshot/manifest.json` | 25,120,712 · 2026-08-05T08:55:00Z | 1793884962 (2026-11-05T13:22:42Z) |

Every file the manifest pins (`cid`, `sha256`, `bytes`) is fetched from IPFS
into `~/cyber/.cache/snapshots/<chain>/` (never in git) and verified by
sha256 and size on every run, cached or not. Files read: `supply.json`,
`pools.json`, `pubkeys.csv`, `balances.csv`, `delegations.csv`,
`holdings.jsonl` (only the integer `undelegating` per account),
`passports.jsonl`, `cyberlinks_indexed.csv.gz`. Gateway layout: bostrom
`<cid>/<name>`, pussy `<cid>/pussy-snapshot/<name>`; the burial sites are the
fallback. `state_export.json.gz` is not read.

## command

```
soft3 genesis build --chain pussy|bostrom --snapshot DIR --out FILE [--cache DIR] [--export DIR] [--no-root] [--time UNIX]
soft3 genesis verify FILE [--root]
```

`build` writes the genesis file to `FILE` and a report to `FILE.json`
(particle, S0, subsidy, state root, block-0 id, counts, per-token sums,
source pins, gaps). It is deterministic: two builds of the same snapshot
produce byte-identical files and the same particle. `verify` decodes the
file strictly, recomputes the particle and, with `--root`, the block-0 root
and id. `--time` is for rehearsal nets only; the chain's time is in
`Chain::genesis_time`.

## derivation (chain.md §4.2, applied)

Integers only: micro-unit amounts are `u128` in derivation and must fit the
`u64` of a `b` frame. Any non-digit amount is a hard error.

- **owners** — every address the snapshot names (auth accounts, bank rows,
  delegators, unbonding entries, link authors, passport owners), once.
  `secp256k1` key known → `neuron_of(pubkey)`, the key re-derived to its
  address and refused on mismatch; anything else (never signed, multisig,
  contract, module) → `legacy_owner(hrp, account_id)`. Owner ids are checked
  unique.
- **balances** — the bank, every denom; then the integer delegated
  (`delegations.csv`) and unbonding (`holdings.jsonl`) amounts back to their
  accounts out of the staking pools (dust stays on `bonded_tokens_pool`);
  then every LP position dissolved into its pool's reserves (floor, dust to
  the reserve account; bostrom's reserve accounts are found as the one bank
  holder of exactly the pool's reserves; a drained pool dissolves to
  nothing); pool coins dropped. Hard check: Σ per carried denom equals
  `supply.json`. Denoms with a balance or a total above 2^64 − 1 are excluded whole
  and listed in the report (`tokens_excluded_u64`); the native coin may not be.
  TOCYB is carried as `token("tocyb")` unchanged (§10 default 5).
- **S0 and subsidy** — S0 = Σ of the native token after the above (pussy:
  the bank supply at halt; no swap is executed); `subsidy = floor(S0 /
  105,120,000)` written into `p`.
- **particles** — every CID a link names → `hemera(cid bytes)`, the
  black-hole branch of `file::readdress`, uniformly (no bytes fetched); no
  naming links, since the two identities coincide.
- **links** — passports first, one `l` per passport at height 0
  (`owner → hemera("passport:" ‖ nickname)`, authored by the owner); then
  every restored source row in order with the author's owner id, the two
  particles and the old height. Duplicates kept.
- **params** — `version 1`, `name`, `hrp`, `denom`, the chain's time,
  `bits = GENESIS_BITS`, `subsidy`, `checkpoint_key` all-zero (no key set
  yet), `checkpoint_sunset = 525,600`, then the pool record of chain.md
  §11.10 (format 1, depth 40, anchor window 1,000, digest width 64,
  tags digest `d4ea8f2b…816b` against hemera 0.3, 74 bytes). The pool is off at launch; the record is how
  the chain id commits to its formats. `build` and `verify` print it with
  the empty-tree anchor `E_40`, and both refuse any other record.

### block 0

`state::load` applies the file to a fresh `bbg::BbgState` exactly as §4.1
says — `b` → balance entry + `coins` total, `l` → bbg `insert` with
`token = τ_native, amount 0, valence 0` at chain height 0, a neuron record
per distinct owner and author — and `root()` commits once. The header of
§1.1/§4.3 (version 1, height 0, the chain's time, `GENESIS_BITS`, nonce 0,
prev 0, `tx_root = chain`, `state_root`, `chain = particle`) is hashed for
`block0_id`. The pipeline's encoder reproduces the header vectors of
chain-conformance §5 (tested).

The root is bbg dimension version 3 (bbg `specs/state.md` § dimension
versions): every dimension leaf is a keyed-trie root (no size cap), every
neuron record carries `nonce = 0`, a link with `amount 0` writes topology
only (`axons_out`, `axons_in`; no particle record, no balance row), and a
zero balance is an absent row. Both burials commit (pussy and bostrom,
`audit/genesis-dimension-v3-2026-10-16.md`). `root_blocked` is reported only
for a state explicitly pinned to the flat (v1) root, whose dimensions are
capped at the lens Brakedown limit of `2^20` field elements.

## debug export

`--export DIR` writes TSV side tables that explain the file in legacy
terms — `accounts.tsv` (address, owner, kind, key), `balances.tsv` (frame +
address + denom), `particles.tsv` (particle, CID), `links.tsv` (frame + author
address, CIDs, time, tx), `tokens.tsv` (token, denom, Σ). They are not
consensus and never hashed; the `.json` report beside the file is the
record of a build.

## verification

1. source counts: the csv row count must equal the manifest's declared
   restored (pussy) or on-chain (bostrom) link count.
2. conservation: Σ per carried token == `supply.json` (hard check in
   `derive`); S0 and subsidy re-derived by `verify`.
3. bijection: `l` frames = passports + source rows; accounts = auth
   accounts; particles = distinct CIDs; unique addresses and owner ids.
4. determinism: two builds, identical bytes; `verify --root` re-derives
   particle, root and block-0 id from the file alone, and checks the pool
   record byte for byte.
5. spot checks: passport contract dormant under `legacy_owner`, staking-pool
   dust, first link author and height, every neuron owner recomputed from
   its pubkey, the known-answer key's address absent (chain-conformance §10).

Observed results: `audit/genesis-2026-10-15.md`; blessed values:
`conformance/chain.snap`.
