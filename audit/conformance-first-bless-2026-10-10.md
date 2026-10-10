---
tags: soft3, conformance, audit
crystal-type: entity
crystal-domain: cyber
---
# conformance — first bless, 2026-10-10

The first stack snapshot of `cyber-conformance` 0.2 (contract:
[conformance/specs](../conformance/specs/README.md)). It was generated from the
origin default branches. Their revisions are recorded in
[conformance/snapshots/provenance.toml](../conformance/snapshots/provenance.toml):
hemera `d5f0a09`, strata `494df7a`, nox `0b1407a`, lens `392abe7`, zheng `5733dfd`,
tade `c7d404a`, bbg `441e3c9`, foculus `8a935e1`, cybergraph `42fc251`, inf `5054c93`,
tru `2f3a255`, tok `e27da0d`.

## result

- 24 encodings, 127 mechanisms, manifest root
  `h64353c453d8db5101b7dfe6d084da4ea72bfc2e5869ccca47598ec29f6d05586`.
- `--check` ran twice in a row and both runs reproduced the files byte for
  byte. The zheng prover is deterministic.
- The release-train gate (`train_gates.conformance_gate`) ran on a fresh
  target directory (aarch64-apple-darwin): green in 15.4 s, no warnings.
  The Linux targets were not run locally. The first Friday candidate gives
  the cross-platform receipt.
- Harness self-tests: 10 integration tests and 11 unit tests passed. They
  detect a tampered fingerprint, an edited encoding, a removed line,
  hand-edited bytes, a hand-edited manifest root, a tampered proof fixture
  and a stale fixture. They also check that drift at beta tier is only
  reported, that `--bless` restores the files and prints the diff, and that
  a delta-tier entry is refused unless `--tier delta` is given.

## findings

1. **The `tree` vectors published in hemera are the fixed-chunk tree.**
   `hemera/vectors/hemera.json` `tree.4k_zeros` (`66a2b0f7…`) equals
   `tree::fixed_chunk_root`. It does not equal `tree::root_hash`
   (`70dd43dc…`). Since the content-defined chunking change, `root_hash`
   matches the fixed-chunk tree only below about 2 KiB.
   hemera's `rs/tests/vectors.rs` tests just `empty` and `hello`, so the
   divergence never failed there. The harness checks the `tree` section
   against `fixed_chunk_root` and snapshots both trees. Follow-up for
   hemera: label the section, or add a `cdc_tree` section.
2. **zheng's default branch has no `ZHENGPF1` profiles.** Profile 0
   (public), profile 3 (state-public) and the other profiles are on the
   0.4 integration line (zheng#33 and the `release/0.4` line). The zheng
   surface covers what `master` exposes:
   - `commit`/`verify` of trace proofs: three quote-only programs and the
     hash-binding proof `[15 [1 42]]`;
   - the Brakedown PCS `open`/`verify_eval`.
   Each profile becomes a scenario pair when it lands on `master`. That
   change moves zheng's entries and requires a bless in the same PR.
3. **All 45 single-bit tamperings of the five committed proof fixtures
   are rejected.** The tamper points are k/8 of the length for k = 0..7,
   plus the last byte. For the trace proofs, 8 of 9 tamperings fail in
   the verifier and the last-byte flip fails in postcard decoding. For
   `hash-42` and `pcs-eval`, all 9 fail in the verifier. The snapshot pins
   these verdicts.
4. **The nox pattern-spec test vectors hold on `master`.** These are
   add (3, cost 3), quote (7, cost 1), cons ([1 2], cost 3), branch (200,
   cost 5), inv(2), inv(0) = InvZero, and (p−1)² = 1. They are now
   fixture invariants.
