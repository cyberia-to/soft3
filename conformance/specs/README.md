---
tags: cyber, soft3, conformance, cip
crystal-type: entity
crystal-domain: cyber
alias: conformance specification, conformance how, conformance reference
---
# conformance specification

| field    | value           |
|----------|-----------------|
| version  | 0.1 (scaffold)  |
| status   | draft           |
| authors  | mastercyb       |
| date     | June 2026       |

## canonical encoding

every `Conformant` type defines exactly one byte sequence as its canonical encoding. canonicity rules:

1. fixed-width integers in little-endian
2. variable-length values prefixed by length (varint)
3. sets and maps in lexicographic order of canonical key encoding
4. floats forbidden in any `Conformant` type
5. nested `Conformant` values inlined under the same rules

a `Conformant` impl that admits two encodings of the same value is a bug.

## fingerprint

```
fingerprint(value) = hemera(canonical_encoding(value))
```

32 bytes, Goldilocks-field-aligned, STARK-friendly. the fingerprint is the snapshot.

## encoding snapshot file

each crate carries `conformance/encoding.snap`:

```
# generated; do not edit by hand
# crate: cyber-mudra v0.3.1
# blessed: 2026-06-07 commit a7f3...
mudra::quorum::Share@v1            tier=delta    h1f3a...c2d
mudra::quorum::Commitment@v1       tier=delta    h89bc...e41
mudra::seal::Ciphertext@v2         tier=gamma    hd4e1...0a7
```

one line per type. one fingerprint per line. tier inline. `@vN` suffix permits parallel encodings during migration.

## mechanism snapshot file

each crate that owns a deterministic mechanism carries `conformance/mechanism.snap`:

```
# generated; do not edit by hand
# crate: cyber-foculus v0.2.0
foculus::convergence@v1 :: "3-node-partition"   tier=epsilon  h7d2e...8af
nox::execute@v1         :: "hello-world.nox"    tier=gamma    h891c...233
zheng::stark@v1         :: "merkle-512"         tier=delta    h442b...91e
```

scenario name is the lookup key. fingerprint binds the [[nox]] trace output for the named scenario.

## tiers

| tier    | meaning   | drift in CI    | bless ceremony                                    |
|---------|-----------|----------------|---------------------------------------------------|
| alpha   | mutable   | silent rewrite | none                                              |
| beta    | tracked   | warning        | commit message notes change                       |
| gamma   | enforced  | fails build    | `cargo conformance bless --tier=gamma`            |
| delta   | locked    | fails build    | bless + commit tag `[conformance:delta]`          |
| epsilon | governed  | fails build    | bless + detached signature appended to .snap file |

promotion is one-way: alpha → beta → gamma → delta → epsilon. demotion requires the same ceremony as breaking a snapshot at the target tier.

## tooling

```
cargo conformance check        # compare; CI invokes this
cargo conformance bless        # regenerate snapshots (respects tier policy)
cargo conformance show <name>  # print fingerprint and tier
cargo conformance manifest     # union of all crate snapshots as one hemera root
```

`cargo conformance manifest` returns the protocol stability root for the current workspace revision. a [[zheng]] proof can be produced over this root.

## manifest construction

```
manifest_root = hemera(
  canonical_encoding(
    sort_by_name(all_encoding_snapshots ∪ all_mechanism_snapshots)
  )
)
```

the manifest binds the entire conformance state of the workspace into a single 32-byte root. a verifier that holds the root and a zheng proof learns the workspace's stability fingerprint at that git revision.

## dependencies

| crate          | role                                              |
|----------------|---------------------------------------------------|
| [[hemera]]     | fingerprint function                              |
| [[nox]]        | mechanism simulator (deterministic VM)            |
| [[zheng]]      | proves a manifest; manifest root is public input  |

conformance depends downward only. [[soft3]] re-exports `verify_conformance(root, proof)` for clients that check a manifest without running the harness.

## non-goals

conformance asserts that encodings and mechanism outputs do not drift. it does not assert correctness — a stable-but-wrong encoding still passes conformance. correctness lives in each crate's own test suite. conformance is the layer beneath correctness: it guarantees that yesterday's correct answer and today's are the same answer.

## status

trait surface drafted in [rs/src/lib.rs](../rs/src/lib.rs). harness implementation lands after:

1. [[hemera]] output stable (currently flagged "may change before stable release")
2. `cargo conformance` subcommand scaffolded
3. one reference crate ([[mudra]] candidate) migrates its stable types to `Conformant`

once those land, every soft3 crate adopts the trait at the pace of its own tier promotion.
