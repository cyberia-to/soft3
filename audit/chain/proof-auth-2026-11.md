---
tags: soft3, chain, audit, zheng, veil, signatures, pussy
crystal-type: audit
crystal-domain: crypto
date: 2026-11-01
serves: release/director/plan.md row P5 (decision D16)
---
# transactions without signatures — the measurement, 2026-11-01

The owner, 2026-11-01 (plan D16): pussy's tru and foculus must be "fully on
proofs, without signatures". Today every non-coinbase transaction carries
NSIG1 (`specs/chain.md` §2.7: `"NSIG1" ‖ pubkey[33] ‖ ecdsa[64]`, 102 bytes,
secp256k1). This page measures the replacement the plan names: a
zero-knowledge proof that the sender knows the secret behind the account's
key, bound to the transaction it authorizes. It measures and names. It does
not change the spec. The §2.7 amendment is the owner's hand (D6) and is
drafted separately with this page as its receipt.

## 1. what was measured

- **Stand:** zheng `diag/p5-proof-auth` at
  [`a2f6641`](https://github.com/cyberia-to/zheng/commit/a2f664189f7114f49ae9c317ac0b20a12bfa3333),
  cut from the stand's zheng `166a14e`. It was never merged and is never in
  `release/pussy-stand.toml` (D16 (3)). Siblings are pinned by SHA to the
  stand: hemera `8559b52` (0.4, profile #15), lens `d6dcc3f`, nox `41ba24f`.
  strata, tade and honeycrisp are at their 2026-11-01 heads.
- **Test:** `rs/tests/proof_auth.rs`,
  `proof_authorized_transaction_binds_the_digest_and_is_measured`, `--release`.
- **Statement:** "I know `k = (k0, k1, k2, k3)`, four Goldilocks limbs, with
  `hemera(k0, k1, k2, k3, 0, 0, 0, 0) = K`."
  - `K` (the account's key) is the only public output.
  - The proof is veil (envelope profile 2, `prove_zk`).
  - The transaction digest `hemera(tx_bytes)` rides in the envelope
    `context`. `prove_zk` absorbs the context into the transcript
    (`veil::prove_bound` over `zk_statement_bytes`, zheng
    `rs/src/envelope/mod.rs:174-197`), so the binding needs no new
    construction.
  - The formula is checked three ways: the secret-input program, the same
    hash over public inputs (`certify_execution`), and native nox.
- **Asserted on every leg:**
  - the same proof fails against another transaction's digest, cold and with
    a cached verifying key;
  - forging any one limb of `K` fails;
  - of 64 spread single-bit flips of the envelope, 0 are accepted;
  - no limb of `k` appears in the statement bytes or the envelope bytes.
- **Gate:** zheng run
  [41289914355](https://github.com/cyberia-to/zheng/actions/runs/41289914355),
  5 of 5 legs `success`, each `test result: ok. 1 passed; 0 failed`. The logs
  are on agent-a in `~/director/state/zheng-run-41289914355/<leg>/`.

The proof parameters, from `veil::report`, are identical on all legs:

| vars | log m | degree | matrices | committed entries | k | data rows | log n | queries | opening bits | IOP bits |
|---|---|---|---|---|---|---|---|---|---|---|
| 13 | 12 | 8 | 5 | 8,633 | 256 | 34 | 14 | 123 | 128.24 | 184.86 |

Proven security is the minimum of the two: **128.2 bits**.

## 2. results

The proof size is the median of 11 proofs (min … max in parentheses). The
envelope is one proof. Prove time is the median of 11. Verify times are
medians of 21: "cold" is `Envelope::verify`, and "cached" is
`veil::verify_bound` with a cached `VerifyingKey`.

| leg | backend | proof bytes | envelope bytes | prove ms | verify cold ms | verify cached ms | flips accepted | log sha256 |
|---|---|---|---|---|---|---|---|---|
| linux x86_64 (`ubuntu-24.04`) | cpu | 98,797 (96,989 … 99,053) | 99,044 | 672.5 | 88.5 | 23.1 | 0/64 | `cc76a8140e00be06…` |
| linux aarch64 (`ubuntu-24.04-arm`) | cpu | 97,997 (97,381 … 99,213) | 98,244 | 223.3 | 47.5 | 12.7 | 0/64 | `76f7fa6a4682566e…` |
| macOS aarch64, all threads | cpu | 98,149 (97,477 … 99,213) | 98,396 | 299.6 | 67.4 | 13.7 | 0/64 | `7650887a21b3a3f1…` |
| macOS aarch64, `ZHENG_THREADS=1` | metal | 98,533 (97,437 … 99,693) | 98,780 | 314.9 | 93.9 | 17.9 | 0/64 | `c20bcfbc2e457748…` |
| macOS aarch64, all threads | metal | 98,221 (97,477 … 99,309) | 98,468 | 208.4 | 67.8 | 15.7 | 0/64 | `c8f3052961caf702…` |

**The Metal rows show the path runs, not Metal's speed.**

- The device was `Apple Paravirtual device`, the runner VM's adapter (as in
  plan row G16a).
- A zheng test proves on the CPU backend unless a backend is installed (lens
  `rspcs/src/backend.rs:627`). The Metal legs therefore install one
  explicitly (`ZHENG_BACKEND=metal`), and the log line `backend: metal`
  confirms it.
- Derived from the code, not measured: the encode's `coset_ntt` reaches the
  backend (n = 16,384 ≥ `PAR_MIN`). Its rows of 512 coefficients are below
  Metal's `MIN_NTT` 2^12, so they stay on the CPU. The Merkle leaves and the
  grinding reach the device. Verification runs on the CPU only.
- A bare-metal Apple Silicon number needs the owner's machine or build-a. No
  such number is claimed here.

## 3. against the 2026-10-09 baseline

zheng `audit/zk-profile-2026-10.md` §3 measured veil on hash.tri's hash with
**one** secret limb, `hemera(x, 0⁷)`:

| | 10-09 baseline | this run |
|---|---|---|
| committed entries | 2,417 | 8,633 (3.6×) |
| commitment `k`, data rows, `log n` | 128, 19, 13 | 256, 34, 14 |
| proven bits | 128.2 | 128.2 |
| proof bytes, median | 63,861 | 97,997 – 98,797 (1.55×) |
| prove | 142.9 ms | 208.4 – 672.5 ms |
| verify, cached key | 4.1 ms | 12.7 – 23.1 ms |

- **Sizes compare directly.** Four secret limbs and a bound context grow the
  masked relation 3.6×, and the hiding commitment moves to `k = 256`. The
  proof grows 1.55×.
- **Times do not compare.** The baseline ran on an Apple M4 Max shared with
  other agents (load 22–27). This run used GitHub-hosted runners.

## 4. what this means for §2.7

The figures below come from `specs/chain.md` §0 and §1.6 (rule set 1) and the
table above.

| | NSIG1 today | proof-authorized, as measured |
|---|---|---|
| authorization bytes per tx | 102 | ~99,044 (envelope) |
| ratio | 1× | 971× NSIG1; 1,544× a bare 64-byte ECDSA signature |
| a transfer's bytes (2 + body 120 + authorization) | 224 | ~99,166 |
| against `MAX_TX_BYTES` 512 | fits | **193× over** |
| transactions per block under `MAX_BLOCK_BYTES` 262,144 (header 168, count 4, coinbase 82) | 1,023 (the `MAX_BLOCK_TXS` cap binds) | **2** (261,890 / 99,166 = 2.64) |
| sustained rate at T = 30 s | ~34 tx/s (1,023 / 30 s; §1.6's rationale rounds to 40) | **0.067 tx/s** |
| fee at `MIN_FEE_PER_BYTE` 1 | 224 units | ~99,166 units (443×) |
| verifier work per tx (cached key) | one ECDSA verify | 12.7 – 23.1 ms |

So per-transaction proofs at today's size cannot ride rule set 1's envelope.
The §2.7 draft has to choose among these options. Each is named with what is
measured and what is not:

1. **Raise the limits for the proof-authorized kind.** This is a spec change
   only. Throughput stays at 2 transactions per 256 KiB block (0.067 tx/s).
   That is still above bostrom's historical rate (≈ 0.02 tx/s, §1.6), but
   §1.6's 40 tx/s design rationale no longer holds.
2. **One proof per block over all of its authorizations.** The aggregation
   is recursive: each sender's proof folds into a block proof. That is
   proven-present S1's territory (plan row P8).
   - The relation is not measured.
   - The nearest receipt is zheng's recursion chain: `c_wrap_chain`'s final
     proof is 74,525 B (plan row L3b, zheng run 40272079660), over
     `specs/recursion.md`'s hard 64 KiB gate.
   - Per-transaction bytes would fall to the body. The block would carry
     one proof of that order.
3. **A smaller per-transaction proof.** The proof is 96 % opening (§3 of the
   baseline: opening 61,693 of 64,309 B). Fewer queries or a different
   commitment trade proven bits or prover time for bytes. Not measured.

## 5. legacy accounts — the claim's options

A preimage proof authenticates nothing for a legacy account. In §2.8, the id
of every snapshot account with a known key is
`neuron_of(pubkey) = hemera(compressed pubkey)`, and that **pubkey is public**:

- `pubkeys.csv` lists all 61,675 bostrom accounts;
- the ones that ever signed a transaction carry a key.

Anyone can prove knowledge of a preimage that is already public. For those
accounts, and for kind 3's address-only accounts (8,390 wallets; §2.8:
`ripemd160(sha256(sig.pubkey)) == account_id`), authorization has to prove
knowledge of the **secp256k1 secret key**. The options:

| option | what it costs | measured? |
|---|---|---|
| (a) secp256k1 proven in-circuit (ECDSA verify, or knowledge of `sk` with `sk·G = pubkey`, over Goldilocks with non-native 256-bit arithmetic) | no circuit exists to measure: trident `lib/std/crypto/secp256k1.tri` (`master` `8f5c1a7`) is a stub (`scalar_mul` returns the identity, `verify_ecdsa` returns false, modular inversion TODO), and zheng `release/0.4` (`ecf2491`) and nox `release/0.4` (`41ba24f`) have no secp256k1 file in their trees | **no** — a cost needs the circuit first |
| (b) NSIG1 kept for the one-time move only: a legacy-keyed or address-only holder signs **one** transaction (an NSIG1 transfer, or kind 3) that moves everything to a native, proof-authorized neuron, and from then on that neuron uses proofs | 102 B once per account that moves. secp256k1 stays in the rule set as a classical hole (§8: "no post-quantum claim") for the balances not yet moved | the NSIG1 path is rule set 1 as it runs today (conformance, soft3-tests) |

Option (b) is the only one buildable before 2026-11-05. Option (a) is a work
item of its own: write the circuit, then measure it.

## 6. not done

- Bare-metal Metal and the owner's M-series numbers (§2).
- Options 4.2 and 4.3, and 5 (a): unmeasured, as stated in their rows.
- The 518,720-flip exhaustive scan the baseline ran. This run does 64
  spread flips per leg.
- The §2.7 amendment itself. It is drafted as its own `decision` PR, with
  this page as its receipt (plan row P5).
