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
| empty | `a67a71b221e6bdd6442a20432bf5d74c885d89e5dfbeec3ec4e334cb806d563c` |
| `"boot"` (= `token("boot")`) | `10275cf6703891a515f64ada465137cd36f2a5d3f0b99a8dafc62e6b95ad1269` |
| `"pussy"` (= `token("pussy")`) | `8cffbea0ac348d13febcee5b2848f916f501237d3ed8966167a31bafddf18ffa` |
| 168 zero bytes | `ba560d1fd2ed380e09ef8e526e8817504cce6e71363c12081537751e4f761754` |

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

`block_id = bdfbbff94e6e17eec9169932276825a0f4957f6f3a6af9030439979eacaeb58d`.
It does not satisfy its own target (byte 31 is `0x8d`), which the harness
asserts: `valid_pow == false`.

PoW-valid header at `GENESIS_BITS`: the header above with
`bits = 0x1e010000`, `time = 1793885850`, mined 2026-10-15 by a linear nonce
search from 0 (40,165,933 hashes, 623 s at 64.4 kH/s on one M4 Max core —
the measurement behind chain.md §1.2):

```
nonce (u128 LE) = 40165933 = 2de2640200000000 0000000000000000
header = 01000000 0100000000000000 9a86ec6a00000000 0000011e 2de264020000000000000000000000000000 1111…11 2222…22 3333…33 4444…44
block_id = d1eec40d1db983bb3df0946890538bec4ba9712fb26b82316efc6b162f000000
```

`valid_pow == true` (bytes 29–31 are zero, so the LE integer is below
2^232); the harness also asserts the same header is rejected at
`bits = 0x1d100000` (2^228 needs byte 28 below 0x10; it is `0x2f`), and that
nonce 40,165,932 fails.

Decode rules: 167 or 169 bytes → error; `version & 0xffff != 1` → error;
`chain` mismatch → error (wrong chain, not malformed).

## 6. transactions

Chain id for all tx vectors: `0x44 × 32`.

Coinbase (kind 0), height 1, `to = 0x55 × 32`, amount 10,000, extra zero:

```
tx    = 0100 0100000000000000 5555…55 (32) 1027000000000000 0000…00 (32)   (82 bytes)
txid  = d7662bdbf363831e4f2efd77967f684db927cf04576f7e82bb8f1872e333cc51
leaf  = c2cb6adb74ed69c5672c495ccfb659e12b8f33db3be8caf6167a0d50b41893e6
```

`leaf` is also the `tx_root` of a block containing only this coinbase.
`node(leaf, leaf) = 3685e0a5b8bce069a4ef54da15f56393cd5e64dc6d10701d247a4663b7d922c1`
(the harness uses it to pin the node domain; a real block never has two
equal leaves).

Transfer (kind 1), unsigned body: `from = 0x66 × 32`, nonce 0,
`token = token("pussy")`, amount 5, `to = 0x77 × 32`, fee 224:

```
tx_without_sig = 0101 6666…66 (32) 0000000000000000 8cffbea0…8ffa (32) 0500000000000000 7777…77 (32) e000000000000000   (122 bytes)
statement      = c7e4a223871018bb9f3f64466b45df3a2962ff344776df2960600dfc49cf4a6f
```

Signed transfer: `blessed` — signed with the mudra known-answer key (the
all-`abandon` BIP-39 phrase, path `m/44'/118'/0'/0/0`, pubkey
`024f4e2ad99c34d60b9ba6283c9431a8418af8673212961f97a77b6377fcd05b62`), with
`from = neuron_of(pubkey)`:

```
neuron_of(pubkey) = 31284576301bacb9c2cad6379244f856dc32c751c38c3b0af4c5c886fd0ed741
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
legacy_owner(pussy, account_id) = cca6f468fe834711fc1e672c21ba95ee7028ed9af6915fbc7b95b449ef0cdbc0
```

Reference for the derivation function:
`legacy_owner("pussy", 0x99 × 20) = d5e54c499456ee690e5c82a3d2119a8180a2e13f832df5254757a41f5e6d509d`.
The harness asserts a claim signed by that key moves every token of the
dormant owner to `neuron_of(pubkey)` and that a claim for an `account_id`
the signing key does not derive is rejected.

Encoding rejections, every kind: trailing byte, missing byte, `version = 2`,
`kind = 4` under rule set 1.

## 7. identities

| derivation | value |
|---|---|
| `neuron_of(abandon pubkey)` | `31284576301bacb9c2cad6379244f856dc32c751c38c3b0af4c5c886fd0ed741` (hemera of the 33 pubkey bytes) |
| `token("boot")` | `10275cf6…1269` (§1) |
| `token("pussy")` | `8cffbea0…8ffa` (§1) |
| `topic(chain = 0x44 × 32, "headers")` | `1f003a48b0506c0ea85cb893abc83c6b6a2603adea7e62f9fe6997692f9f8b8b` |
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
| after a cyberlink (amount 0: chain.md §10 item 2, decided 2026-10-15) | state root; `particles[q]`, `particles[axon]` and the axon lists exist at zero, author balance down by the fee, nonce 2 |
| a cyberlink with amount 7 | rejected (`TxError::Box(7)`) — no boxes in rule set 1 |
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
Computed on 2026-10-16 with `cyber-hemera` at `d5f0a09` (origin `main`)
through `hemera::hash` and `hemera::tree::hash_node`, by
`crate/src/chain/pool/`, and blessed as `chain::pool@v1` entries of
`conformance/chain.snap`. A moved value is a consensus change.

The sample: `sk = 0x01 × 32`; input record `token("pussy")`, value 5,
`owner = hemera(D_OWNER ‖ sk)`, `rho = 0x02 × 32`, `r = 0x03 × 32`,
`data = 0x04 × 32`, predicate zero; output record the same with value 4,
`rho = 0x05 × 32`, `r = 0x06 × 32`. The spend: `chain = 0x44 × 32`, anchor
= the root of the tree holding the input's commitment at position 0, the
input's nullifier, the output's commitment, one cyberlink
`(token("boot"), token("pussy"), weight 1, payout = owner)`, no withdrawal,
fee 1 in `token("pussy")`, two zero predicates, no notes.

Record bytes (200; fields of chain.md §11.1 in order):

```
8cffbea0ac348d13febcee5b2848f916f501237d3ed8966167a31bafddf18ffa   token("pussy")
0500000000000000                                                   value 5
4424760142c6837ead0542693ce6ad7d827c8e2c836bea4fa5bab44f7c22c192   owner
0202020202020202020202020202020202020202020202020202020202020202   rho
0303030303030303030303030303030303030303030303030303030303030303   r
0404040404040404040404040404040404040404040404040404040404040404   data
0000000000000000000000000000000000000000000000000000000000000000   predicate
```

Spend public inputs (341 bytes; chain.md §11.5 in order):

```
4444444444444444444444444444444444444444444444444444444444444444   chain
74c97bcd357ccf5b5680b5a39824fa90f72c59afa5590651ed1e65ad3a517eb9   anchor
01 f5b99cad398b929c0fa25774dc4e1433c108950454aa72330407184340e6c5bb   n = 1, nullifier
01 c2c8aacad99b98ff322b8d9cea52c5f7b87b4735f2df09d748921187b60b04e7   m = 1, commitment
01 10275cf6703891a515f64ada465137cd36f2a5d3f0b99a8dafc62e6b95ad1269   k = 1, p = token("boot")
   8cffbea0ac348d13febcee5b2848f916f501237d3ed8966167a31bafddf18ffa   q = token("pussy")
   0100000000000000                                                 weight 1
   4424760142c6837ead0542693ce6ad7d827c8e2c836bea4fa5bab44f7c22c192   payout
00                                                                  w = 0
8cffbea0ac348d13febcee5b2848f916f501237d3ed8966167a31bafddf18ffa   fee token
0100000000000000                                                   fee 1
0000000000000000000000000000000000000000000000000000000000000000   predicate of the input
0000000000000000000000000000000000000000000000000000000000000000   predicate of the output
00                                                                  c = 0
```

The second spend: the same input, one output of value 2 (`rho = 0x05 × 32`,
`r = 0x06 × 32`), no cyberlink, a withdrawal of 2 `token("pussy")` to
`0x77 × 32`, fee 1, one note `ab ab ab` (314 bytes, ending
`… 01 0300 ababab`).

| vector | value |
|---|---|
| `owner = hemera(D_OWNER ‖ sk)` | `4424760142c6837ead0542693ce6ad7d827c8e2c836bea4fa5bab44f7c22c192` |
| tags digest (chain.md §11.8) | `362fdac7f17abfc2d54699a6c6fcd3ad14e6c6edb6ce8534eac796bc22783a67` |
| genesis pool record (41 bytes, chain.md §11.9) | `0100000028e8030000362fdac7f17abfc2d54699a6c6fcd3ad14e6c6edb6ce8534eac796bc22783a67` |
| `EMPTY = hemera(D_EMPTY)` | `ddba4580f44954d39ca84d40621f45c67865e630f98367b4afe012335af4d697` |
| `E_1 = hash_node(EMPTY, EMPTY, false)` | `9b2a1622a8177d0b1a431e283f8191f749e533c92be6d364e31b4f3220c896ab` |
| `E_39` | `df534e8d094f51b75ee6268c532a25afa0b6436f8b568114f270dd2954192734` |
| `E_40`, the empty tree root (anchor of block 0) | `4d8681dd3449327fa76ce217e8bed764067d1c3bad428aac8fcfb11e7ecc8499` |
| `T(token("pussy"))` (chain.md §11.4) | `ddd4a6ce619bc990793a74eb6be61d0765390bc68f9e0ecef7db46597e6f4ae1` |
| `hemera(record bytes)` | `7b39ede9082e3ea2dfc5cb24b72914000127ac2eb42971f06e9d8731e911a06d` |
| commitment of the input | `983d0b892ca90b8ab19526e9a3f4c87349cdadbf956b97187755f8af5cee756f` |
| nullifier of the input | `f5b99cad398b929c0fa25774dc4e1433c108950454aa72330407184340e6c5bb` |
| commitment of the output | `c2c8aacad99b98ff322b8d9cea52c5f7b87b4735f2df09d748921187b60b04e7` |
| root, input commitment at position 0 | `74c97bcd357ccf5b5680b5a39824fa90f72c59afa5590651ed1e65ad3a517eb9` |
| root, input then output commitment at 0, 1 | `dea3054b23f666a3d60b94c4588d37a52705177c12992f2f70120b09f2b7d3ef` |
| `hemera(spend public bytes)` | `970073e4562d9bf2e0692b3b625c5a0b1e059f119aca4e60bb0b059cf4990c1e` |
| `spend_id = hemera(D_SPEND ‖ spend public)` | `c4135697aeb219f4ff1e6cd2e18ae387fb5ffb1ce4d6e34809b80968186eb1f7` |
| `hemera(second spend bytes)` | `349b43b37496e9dd7f4d471c0df8ebaef6dc7cd1324202ee319a07b29e15b95b` |
| `vk_set` of no keys, `hemera(D_VK)` | `49bb5c21ebf22311045270ccd6e7888c6addaaba1a298b830387f6b58f232708` |
| `statement_id`, `chain = 0x44 × 32`, height 1, that `vk_set`, the one spend | `c006302fb2c77b9fa4ef645ff3ede7a7dc18685d33f7aee7e1467514f8ac8853` |

Rejections the harness carries: a 199- or 201-byte record; a pool record
differing in format, depth, window or tags (the genesis file is refused); a
params frame without the pool record; a spend with 0 or 5 inputs, 5
outputs or 5 cyberlinks, a repeated nullifier or commitment, `p = q`, a
withdrawal flag 2, a withdrawal of 0, a note of 2,049 bytes, more notes
than outputs, a trailing or a missing byte. Merkle paths of the two-leaf
tree verify under `hemera::merkle_verify_path` and a wrong sibling fails.
The longest canonical spend encodes to 9,309 bytes; a deposit is 352
bytes. Kinds 4 and 5 are rejected by the rule-set-1 decoder (§6).

## 13. running

`cargo test -p soft3 --features chain-conformance` runs sections 1–9, 11
and 12 on every commit; section 10 runs in the release train (`stack-chain-genesis`
gate) because it needs the pinned snapshot inputs. Blessed values are
written by `soft3 conformance bless chain` into `conformance/chain.snap`
(and checked by `soft3 conformance check chain`); the genesis entries of
§10 need the genesis file (`--genesis FILE`, default
`~/cyber/.cache/genesis/pussy.tade`) and are kept as they are when it is
absent. They are reviewed in the PR that moves them. The report of each run goes to `soft3/audit/chain/conformance-<date>.md`
with the revision and the command, per the audit doctrine.
