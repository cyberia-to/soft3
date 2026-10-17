---
title: chain-conformance — acceptance vectors of the phase-1 root chain
tags: cyber, soft3, spec, chain, conformance, hemera
crystal-type: spec
crystal-domain: cyber
status: accepted
date: 2026-10-15
alias: chain conformance, chain vectors, chain acceptance tests
---

# chain-conformance — acceptance vectors of the phase-1 root chain

The tests the conformance harness (`soft3/conformance`, snapshot files per
the stack's `.snap` discipline) must carry for [chain.md](chain.md) and
[chain-wire.md](chain-wire.md). Every vector below is either computed here
with the stack's own primitives and quoted exactly, or marked `blessed` —
produced once by the reference implementation, reviewed, and frozen into
`conformance/chain.snap`. A blessed vector that moves is a consensus change.

Hemera vectors were computed on 2026-10-15 with `cyber-hemera` at
`~/cyber/hemera/rs` (the revision pinned in `release/phase1.toml`, `hemera`
row) through `hemera::hash`, the flat sponge with no domain tag. All hex is
lowercase, bytes in wire order.

## 1. hemera anchors

| input | `hemera(input)` |
|---|---|
| empty | `9e89f2487be432e48c27e24e6945adffe6918c8b5d6a62061e70e0bebd657326` |
| `"boot"` (= `token("boot")`) | `1d443e1103abbd0aebe4abaf3fadc203160a471e9accb9b6e0a73804ea785b3b` |
| `"pussy"` (= `token("pussy")`) | `87a70ff9ed0638fc9fafb1a30f57c5ad2ebe5155c48702c50bdbb2ece87f6a71` |
| 168 zero bytes | `355fd03315a5ceb056cda53d11ff7b9755167182d59ff9e31134d69d33bdf0f6` |

The harness fails if any of these moves: it means hemera itself changed,
and every id below with it.

## 2. compact target

Decode and canonical-encode (chain.md §1.3):

| bits | target | verdict |
|---|---|---|
| `0x1e010000` | 2^232 | canonical; `GENESIS_BITS` |
| `0x1e100000` | 2^236 | canonical; `POW_LIMIT` |
| `0x1d100000` | 2^228 | canonical |
| `0x1d200000` | 2^229 | canonical |
| `0x1d080000` | 2^227 | canonical |
| `0x1d084860` | `0x0848600000…` (target = 0x084860 · 256^26) | canonical |
| `0x03123456` | `0x123456` | canonical |
| `0x1e001000` | 2^228 | reject: non-canonical (canon `0x1d100000`) |
| `0x04001234` | `0x123400` | reject: non-canonical (canon `0x03123400`) |
| `0x01003456` | 0 | reject: target 0 |
| `0x1d800000` | — | reject: sign bit |
| `0x00000000` | 0 | reject: target 0 |
| `0x1f010000` | 2^240 | reject: above `POW_LIMIT` |

## 3. work

`work(target) = floor(2^256 / (target + 1))`:

| target | work |
|---|---|
| 2^228 | 268,435,455 |
| 2^232 | 16,777,215 |
| 2^236 | 1,048,575 |
| 1 | 2^255 |

Chainwork is the u256 sum; the harness checks a 3-block chain at
`GENESIS_BITS`: `3 · 16,777,215 = 50,331,645`.

## 4. ASERT

Test anchor `genesis_target = 2^228` (a test net value; the real chains
anchor at `GENESIS_BITS` = 2^232 and the formula is the same), `T = 30`,
`H = 600`, `genesis.time = 0` (chain.md §1.5). Inputs are the parent's
height and time; outputs the exact intermediate values and the next block's
bits.

| parent height | parent time | e | shifts | frac | factor | next target | next bits |
|---|---|---|---|---|---|---|---|
| 0 | 0 | 0 | 0 | 0 | 65536 | 2^228 | `0x1d100000` |
| 19 | 570 | 0 | 0 | 0 | 65536 | 2^228 | `0x1d100000` |
| 19 | 1170 | 65536 | 1 | 0 | 65536 | 2^229 | `0x1d200000` |
| 100 | 2400 | −65536 | −1 | 0 | 65536 | 2^227 | `0x1d080000` |
| 19 | 0 | −62260 | −1 | 3276 | 67852 | `0x084860 · 256^26` | `0x1d084860` |
| 5 | 1,000,000 | 109210282 | 1666 | 27306 | 87473 | 2^236 (clamped) | `0x1e100000` |

Row 5 pins floor division toward −∞ (`e = −62260`, not −62259) and the
cubic (`factor = 67852`); row 6 pins the clamp at `POW_LIMIT` = 2^236 under
a shift far beyond 256. An implementation using floating point will fail row 5 at
`factor`.

Behavioral checks (simulated, deterministic): a chain mined at exactly
T-spacing holds `bits` constant; blocks arriving at T/2 for 20 blocks
produce a target below 2^228 by a factor in `[1.9, 2.1]` after 600 s of
block time; a 100× slowdown recovers the T interval within 70 minutes of
simulated wall time.

## 5. header

Test header (not a chain block; `chain = 0x44 × 32`):

```
version 1, height 1, time 1762349000, bits 0x1d100000, nonce 0,
prev 0x11×32, tx_root 0x22×32, state_root 0x33×32, chain 0x44×32
```

bytes:

```
01000000 0100000000000000 c84f0b6900000000 0000101d
00000000000000000000000000000000
1111…11 (32) 2222…22 (32) 3333…33 (32) 4444…44 (32)
```

`block_id = bdd7f2ef3a5a1c573b8d84a7a2aeedc95c6f8aa2139eea5c83946b33c4180c31`.
It does not satisfy its own target (byte 31 is `0x31`), which the harness
asserts: `valid_pow == false`.

PoW-valid header at `GENESIS_BITS`: the header above with
`bits = 0x1e010000`, `time = 1793885850`, re-mined 2026-10-16 under hemera
0.4 by a linear nonce search from 0 with the chain's own path (`Header::id`
= `hemera::hash` over the 168 bytes, `valid_pow`): 934,637 hashes, 10.4 s
at 90.3 kH/s on one M4 Max core of a shared machine — the measurement
behind chain.md §1.2 (the 0.3 vector was nonce 40,165,933: 623 s at
64.4 kH/s):

```
nonce (u128 LE) = 934636 = ec420e0000000000 0000000000000000
header = 01000000 0100000000000000 9a86ec6a00000000 0000011e ec420e000000000000000000000000000000 1111…11 2222…22 3333…33 4444…44
block_id = a3dc0cdb7c69e1d4986c388effe1612613b1b71f2c2ca49084bc0730da000000
```

`valid_pow == true` (bytes 29–31 are zero, so the LE integer is below
2^232); the harness also asserts the same header is rejected at
`bits = 0x1d100000` (2^228 needs byte 28 below 0x10; it is `0xda`), and that
nonce 934,635 fails.

Decode rules: 167 or 169 bytes → error; `version & 0xffff != 1` → error;
`chain` mismatch → error (wrong chain, not malformed).

## 6. transactions

Chain id for all tx vectors: `0x44 × 32`.

Coinbase (kind 0), height 1, `to = 0x55 × 32`, amount 10,000, extra zero:

```
tx    = 0100 0100000000000000 5555…55 (32) 1027000000000000 0000…00 (32)   (82 bytes)
txid  = 6cf0e4d280e6d5cbb5c7fef566d28913b17249cb4a6fdecf9c59ad3f0995c3f9
leaf  = 514d5d57388046e2cd534c69a789bf09e329678cedbabcff4cc9a439e8c0fe9c
```

`leaf` is also the `tx_root` of a block containing only this coinbase.
`node(leaf, leaf) = 9d68ac9b03293ba25059ceb5d601e3c6ddae700e05343c190fdd4c8eea6c595b`
(the harness uses it to pin the node domain; a real block never has two
equal leaves).

Transfer (kind 1), unsigned body: `from = 0x66 × 32`, nonce 0,
`token = token("pussy")`, amount 5, `to = 0x77 × 32`, fee 224:

```
tx_without_sig = 0101 6666…66 (32) 0000000000000000 87a70ff9…6a71 (32) 0500000000000000 7777…77 (32) e000000000000000   (122 bytes)
statement      = 41e149559c98af20b62db3e02ac2dfd4c62568f695d92986f9c274a7d62d12a7
```

Signed transfer: `blessed` — signed with the mudra known-answer key (the
all-`abandon` BIP-39 phrase, path `m/44'/118'/0'/0/0`, pubkey
`024f4e2ad99c34d60b9ba6283c9431a8418af8673212961f97a77b6377fcd05b62`), with
`from = neuron_of(pubkey)`:

```
neuron_of(pubkey) = 55c76c9f172b850eaa82e4c6defe13b1d80d983c29d81526eb94d2c8199b0fd2
```

The harness asserts `verify_statement(from, statement, sig) == true`, that
flipping any byte of `sig`, `statement` or `from` fails, that the 101- and
103-byte truncations fail, and that the same tx with `chain = 0x45 × 32`
fails (replay across chains).

Cyberlink (kind 2): `blessed` — `p = token("boot")`, `q = token("pussy")`,
amount 0, valence +1, fee 257; the harness checks the 257-byte length, the
`valence = 2` rejection, the `p == q` rejection.

Claim (kind 3): the known-answer key's legacy account:

```
account_id        = 28ff5c6d57d8cfd492b6fb42614536ed648e01fd      (ripemd160(sha256(pubkey)); bech32-decodes from the address)
pussy address     = pussy19rl4cm2hmr8afy4kldpxz3fka4jguq0a4l8rzm   (mudra tests/vectors.rs)
legacy_owner(pussy, account_id) = 4382c7f5d008648955902bb95ef7a6a3cdce9e9c9181a553b9858a73341ce373
```

Reference for the derivation function:
`legacy_owner("pussy", 0x99 × 20) = 5384913f850b453826161f40676a431d912c38c112d944ed85aab22c59889f22`.
The harness asserts a claim signed by that key moves every token of the
dormant owner to `neuron_of(pubkey)` and that a claim for an `account_id`
the signing key does not derive is rejected.

Encoding rejections, every kind: trailing byte, missing byte, `version = 2`,
`kind = 4` under rule set 1.

## 7. identities

| derivation | value |
|---|---|
| `neuron_of(abandon pubkey)` | `55c76c9f172b850eaa82e4c6defe13b1d80d983c29d81526eb94d2c8199b0fd2` (hemera of the 33 pubkey bytes) |
| `token("boot")` | `1d443e11…5b3b` (§1) |
| `token("pussy")` | `87a70ff9…6a71` (§1) |
| `topic(chain = 0x44 × 32, "headers")` | `598b569691c3dc0cc6ee22e1cc7a595ee747e33c1b49defc69bc26c831bbd7a8` |
| `axon(p, q)` | bbg `axon_id` vectors, inherited from bbg's conformance |
| `balance_key(owner, token)` | bbg `balance_key` vectors, inherited |

## 8. state

`blessed`, produced by the reference implementation over bbg at the pinned
revision, with `NeuronRecord.nonce` and `DIMENSION_VERSION = 3`:

| scenario | frozen value |
|---|---|
| empty state root | `root(BbgState::new())` |
| genesis of the conformance net: one `b` frame (`owner = 0x66 × 32`, `token("pussy")`, 1,000,000), no links | genesis file bytes, its particle (chain id), block 0 id, state root |
| after block 1: coinbase to `0x55 × 32`, the transfer of §6 | state root, balances of `0x55`, `0x66`, `0x77`, nonce of `0x66` = 1, `coins[token("pussy")].total_supply = 1,000,000 + SUBSIDY` |
| after a cyberlink (amount 0: chain.md §10 item 2, decided 2026-10-15) | state root; the axon lists hold `axon(p, q)` and no particle record is written (a conviction-free link writes topology only, bbg dimension version 3), author balance down by the fee, nonce 2 |
| a cyberlink with amount 7, native token | accepted since 2026-10-17: the card `card(from, p, q)` is created and 7 moves to its stake key (chain-economy §3.3); the `TxError::Box(7)` vector is retired. With a non-native token: rejected |
| conservation | Σ native balances == S0 + h · SUBSIDY at every height of a 200-block simulated chain with random valid txs, including a 3-block reorg |
| coinbase maturity | spending block 1's subsidy at height 100 is rejected, at height 101 accepted |

## 9. consensus

Simulated with a deterministic miner (nonce search at a trivial target on
a private test genesis):

- heaviest chain: two forks of equal length, the one with lower targets
  (more work) wins; equal work keeps the first seen;
- reorg limit: a fork from `tip − 120` with more work is rejected; from
  `tip − 119` it is accepted;
- checkpoints: a signed checkpoint at height 120 makes a competing
  heavier chain excluding it rejected; a checkpoint with height
  `> CHECKPOINT_SUNSET` is ignored; a checkpoint signed by another key is
  dropped and scored;
- validation order: a block with bad PoW and a bad body reports the PoW
  error and never touches the body; a block with valid PoW and a wrong
  `state_root` marks the header invalid;
- timestamps: `time == MTP` rejected, `MTP + 1` accepted; `now + 121 s`
  held then accepted once the clock passes;
- orphan pool: 1,025 orphan headers evict the lightest.

## 10. genesis pipeline

`blessed`, the highest-value vector of the set:

- the pussy genesis file: bytes, particle (chain id), block 0 id, state
  root, S0, `SUBSIDY`, frame counts (`b`, `l`) and the per-token sums
  against `supply.json` after decomposition;
- two independent runs of the pipeline (two machines, the pinned inputs by
  sha256) must produce byte-identical files; the harness compares the
  particle;
- the bostrom genesis file: the same, produced once row 43 lets the replay
  finish; until then the bostrom entry is red evidence, not a placeholder;
- the known-answer key's pussy address holds no balance in either snapshot
  (the harness asserts the absence, so a test key never becomes a funded
  account by accident).

## 11. wire

- tade: every message frame round-trips through the tade reference decoder;
  a stream whose first frame is not `(*, k) urn:cyberia:chain:1` is closed;
- status frame: 32 + 2 + 8 + 32 + 32 + 32 = 138 bytes; chain mismatch →
  `x` code 4 and disconnect;
- locator: for a chain of height 1,000 the locator has 20 entries
  (heights 1000…991, 989, 985, 977, 961, 929, 865, 737, 481, 0 and the
  stop id); the harness freezes the height list;
- get-range: `count = 129` → `x` code 3; a pruned height → `x` code 2;
- gossip: a body pushed on any topic is a protocol error (+100);
- rate limits: 65 headers in one second from one peer → the 65th is dropped
  and scored +1;
- HTTP: `/status` parses as cybermark with the fields of chain-wire §7;
  `POST /tx` with a bad signature → `400`; `/v2/history` entries carry
  `height`, `block`, `txid`; legacy ingress → `410`.

## 12. the pool

The formats of chain.md §11, frozen at genesis although rule set 2 is off.
Computed on 2026-10-16 against hemera 0.4 (`cyber-hemera` 0.4.0 at
`7237a23`, `release/0.4`, profile #15 — the profile the chain freezes; the
0.3 values of the same day were re-blessed through the same code, the
definitions did not move) through the hemera API alone —
`Hasher::finalize_xof` for `W` (chain.md §11.1) and the tree node (§11.4),
`hemera::hash` for the narrow values — by `crate/src/chain/pool/` (tests
`chain::pool::tests`). They are blessed
as `chain::pool@v1` entries of `conformance/chain.snap`; a snapshot line
holds 32 bytes, so a wide value enters it as `hemera(value)`, its
fingerprint, and the full values are quoted here. A moved value is a
consensus change once the profile is final.

The sample: `chain = 0x44 × 32`, `sk = 0x01 × 32`. The input record was
deposited by txid `0x33 × 32`: `token("pussy")`, value 5,
`owner = W(D_OWNER ‖ sk)`, `rho = W(D_RHO ‖ txid ‖ 00)`, `r = 0x03 × 32`,
`data = 0x04 × 32`, predicate zero. The output is output 0 of the spend:
the same with value 4, `rho = W(D_RHO ‖ nf_0 ‖ 00)`, `r = 0x06 × 32`. The
spend: anchor = the root of the tree holding the input's commitment at
position 0, the input's nullifier, the output's commitment, one cyberlink
`(token("boot"), token("pussy"), weight 1, payout = owner)`, no
withdrawal, fee 1 in `token("pussy")`, two zero predicates, no notes.

| value | 64 bytes, hex |
|---|---|
| `owner = W(D_OWNER ‖ sk)` | `1b152c6b13df0965934007b4e01922e8548bdfb9478a7e239d8776104bda98d8` `759483fbf9c9f7f9f172f08e84b31285c82d13c14e7a44a0353a793a8a70de8c` |
| input `rho = W(D_RHO ‖ 0x33 × 32 ‖ 00)` | `4b41345cf71d27c2907681bc91e6737e0feb3a817fd286d294b222a817aa2829` `e6718d726e7655796551025cdd501f8d8690f24182711167bb38a7b156a505ea` |
| input commitment | `c15665f95e939dd50433ff7a0846cf1c45a1c82c5ca6e93c035c71bb69f24bea` `9b50809ac156774cce868a6abef0e43ba2fb70a05d25d1c04242e707eb4c1335` |
| `nf_0 = W(D_NULL ‖ chain ‖ sk ‖ rho)` | `102418fa5d3f7c688831761a6b557385b5bf1d4b82f6f9d6b0a552a09ca8d623` `e043c8ab759266a3d73af105e68decd7f8721509b555fbb9ed9a07dbfa42cbb2` |
| output `rho = W(D_RHO ‖ nf_0 ‖ 00)` | `0d0a220ddad70be95ceb63ee54cc70ceab8690a7105da0da4eb2236911f1526b` `f101c36b81c84fc8a88e39f93915d7152cdf54f87ca32081c877af003e3328e9` |
| output commitment | `10f51d4500eee5a93367eaecc8f3b00dda2341194ffb1707e5dd42e66d6e66f5` `7c14fc707a71cb14d0ebea02dfd54f77056526176e18ff90747137e0b7ab18cd` |
| `EMPTY = W(D_EMPTY)` | `7017af618fecaeb35debc7ae9ef02014ecc9116f7a797cae6166a31fa0b9f471` `df5bb8d91bfeed06bf4c760ef647df0b19070677f80862cc810f2169b0648ae4` |
| `E_1 = node(EMPTY, EMPTY)` | `31c36c435178289e28716d82881c1bd9ba54ae37db1c37af9801c99a6defc12a` `e13fa04bcbc572cffb0d231ed63e149a120ba04d5687230f116b326d804e503b` |
| `E_40`, the empty tree root (anchor of block 0) | `3ce6bca0d485365765a1c527968a71ee75acb3dfffbd9d4c2c76a8f11655d1eb` `8bcf444881f1ac6bd8424eb9a32b938476aa75f02f89dd48e9719656c47cb395` |
| root, input commitment at position 0 | `3f8737ef321a6b0fefffa72c226b620e18061661c8bef868f59ca9ea6fca72b5` `73205fa34f9e51f1f695e094c6ac8842cfe0997fd47b4683833a07f757ad2f9b` |
| root, input then output commitment at 0, 1 | `1325e823ef4fe668bf597ace789dcdf7aba7be517d46200a050503344672f854` `d7d1000058f1f1d87722e46cc7524a1710024f76ba5b5e178e14dd74a66df72b` |
| `spend_id = W(D_SPEND ‖ spend_public)` | `3a17d36f50f813867cc95ab0ce30382d074f0e1c306096172d3238e5e7486578` `eac26a7f677cc7436b2f0f4e01a4296a8e8ef2d9038186347dcf1fe7015ebe74` |
| `vk_set` of no keys, `W(D_VK)` | `6156b6f616bba60fded9c30ba7efd1d8854edc649481263d4bc09321acd16866` `567519e480d8eca65c5e994a6ef79aa8511cbb79e4c221c7a8d74a9ddb4d47ee` |
| `statement_id`, `chain = 0x44 × 32`, height 1, that `vk_set`, the one spend | `ec3ea0eb3e864ea5a08c3263f82ea9026aecad2ec34cb672ea1f6d0bae633c99` `af996c5038dee2acd11244d1602852253ba55be31bc5dfded5eeca339abd0395` |
| tags digest (chain.md §11.9) | `0056e480f212c69488b71289b7c3e170f015c6411267b915a1596fae4f9fc4a6` `4faaecefa7d0f0a8b75c0d76902da823b6c45ca46e8358f6578848e415dff0b1` |

Narrow values: `T(token("pussy")) = hemera(D_TURNSTILE ‖ token("pussy"))`
= `4b41f4e28348237bdcb43058670b7b6e901a87bf4782b88ec7e65a1589061b05`; the
genesis pool record (74 bytes) is
`0100000028e803000040` ‖ the tags digest.

Record bytes (296; chain.md §11.2 in order): `token("pussy")` ‖
`0500000000000000` ‖ owner ‖ input rho ‖ `03 × 32` ‖ `04 × 32` ‖ `00 × 64`.
Spend public inputs (533 bytes; chain.md §11.6 in order): `44 × 32` ‖
anchor (the one-leaf root) ‖ `01` ‖ `nf_0` ‖ `01` ‖ output commitment ‖
`01` ‖ `token("boot")` ‖ `token("pussy")` ‖ `0100000000000000` ‖ owner ‖
`00` ‖ `token("pussy")` ‖ `0100000000000000` ‖ `00 × 128` ‖ `00`. The
second spend: the same input, output 0 of value 2, no cyberlink, a
withdrawal of 2 `token("pussy")` to `0x77 × 32`, fee 1, one note
`ab ab ab` (474 bytes, ending `… 01 0300 ababab`). Their fingerprints are
the `chain::pool@v1` lines "record", "spend public" and "spend public,
withdrawal and note".

Rejections the harness carries: a 295-byte record; a limb `≥ p` in any
identifier field of a record (token, owner, rho, r, data, predicate) or of
a spend (anchor, a payout); a pool record differing in format, depth,
window, digest width or tags (the genesis file is refused); a params frame
without the pool record; a spend with 0 or 5 inputs, a repeated nullifier
or commitment, `p = q`, a withdrawal flag 2, a withdrawal of 0, a note of
2,049 bytes, more notes than outputs, a predicate missing, a trailing or a
missing byte. `W(x)` is the first two 32-byte squeezes of hemera's
extendable output and begins with `hemera(x)`; the tree node is
`W(D_NODE ‖ L ‖ R)` and depends on the order of its children; paths of the two-leaf tree verify from either side and
a wrong side fails. A value of `u64::MAX` is a valid record value (values
are integers, not limbs). The longest canonical spend encodes to 9,981
bytes; a deposit is 384 bytes. Kinds 4 and 5 are rejected by the
rule-set-1 decoder (§6).

## 13. running

`cargo test -p soft3 --features chain-conformance` runs sections 1–9, 11,
12 and 14 on every commit; section 10 runs in the release train (`stack-chain-genesis`
gate) because it needs the pinned snapshot inputs. Blessed values are
written by `soft3 conformance bless chain` into `conformance/chain.snap`
(and checked by `soft3 conformance check chain`); the genesis entries of
§10 need the genesis file (`--genesis FILE`, default
`~/.cache/cyber/genesis/pussy.tade`) and are kept as they are when it is
absent. They are reviewed in the PR that moves them. The report of each run goes to `soft3/audit/chain/conformance-<date>.md`
with the revision and the command, per the audit doctrine.

## 14. the focus economy

The vectors of [chain-economy](chain-economy.md) §10, computed on
2026-10-17 with `cyber-hemera` 0.4 at `7237a23` through `hemera::hash` and
with Python integers; the rest `blessed`.

```
card(0x66 × 32, token("boot"), token("pussy")) = 74f0a211d177f290e9c95a6e7edb65cd8705ea96d9713a841d5a086f6476b5b4
stake_key(that card)                             = f7e0cbf8e5cfc501b7e3098b238bce8451b3b0a7fa2e42b69724801115c6b08a
focus_key(token("pussy"))                        = 0fae8671376d879515b2736c2d6d51eafdd9ff40055aa84aaff6e17a0beed4b9
payout_leaf(that card, 0x77 × 32, 1,000)         = 63c3867a76af023aa6417e5a945612289906e8352ea69fe4aea259ce5b152115
payout_node(leaf, leaf)                          = 240a852c1bd8f641f65d6b3746a72a08c0930e5d1bd1feafe4b9cdab914a6ec2
phi digest, n = 2, φ = [2^59, 2^59]              = 3351bfa40d6eaa24c8e2b8755d900dbcd3c3015730a5efcfdf9b6e14c7019a8a
phi digest, tru's conformance graph              = e0ff79997aa697bced38e2cdedb823b189892982bae411933ec623007de18fc8
```

The conformance graph, its φ*, J, gradient and 4-node path gradient are
defined in chain-economy §10 (amended 2026-10-17: `GROUND` 1/4,
`ATTR_NODES` 4).

| case (pussy `S0`, `SUBSIDY`) | value |
|---|---|
| `focus_bps` at active = 0, S0/10⁴, S0/100, S0/4, S0 | 0, 99, 999, 5,000, 5,000 |
| `cut` at 1,000 bps; miner share | 1,045,842,665; 9,412,583,992 + fees |
| `cut` at 5,000 bps; miner share | 5,229,213,328; 5,229,213,329 + fees |
| mark decay, H = 1,000,000, S = 0: after 1 epoch, after 360 | 998,077; 500,047 |
| budget 1,000, x = [1, 1, 1] | pay [333, 333, 333]; 1 stays in the pool |
| budget 10⁶, x = [3, 1], Δs = [10⁶, 10⁹] | caps [10,000, 10⁷]; pay [10,000, 250,000]; 740,000 stays |
| `epoch(240)`, `epoch(241)`, `settle(1)` | 1, 2, 360 |

`blessed`: the transaction vectors of kinds 2 (with stake), 6 and 7; the
genesis weights of the conformance genesis and of pussy's genesis file;
φ*_0 of pussy's genesis single-threaded and with 8 threads (equal); a
2-epoch chain with a +1, a 0 and a −1 lock (epoch record, payouts, pool,
root); the new-high relock; the reorg pair (a reorg above the boundary
reproduces the record, a reorg of the boundary changes it); conservation
over 2,000 simulated blocks including settlement blocks. The list and
the scenarios are chain-economy §10.
