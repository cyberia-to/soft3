---
title: privacy freeze research receipt
tags: soft3, audit, privacy, research
crystal-type: report
crystal-domain: cyber
date: 2026-10-16
status: findings
---

# privacy freeze research receipt

Companion to [the proposal](../../proposals/privacy.md). This is source research
and algebraic counterexample reproduction. It is not a production cryptographic
audit, a complete implementation review, a new prover benchmark, or an exploit
against a deployed network.

## provenance

Soft3 base: `8525b174fa227f7def0e368d6ea64e135fdf063a`, fetched `origin/main`.
Worktree: `/Users/master/cyber/.worktrees/soft3-privacy-freeze`, branch
`research/privacy-freeze-20261016`. The existing soft3 checkout was dirty and
was left untouched. Other component checkouts were read only; remote refs were
fetched for the component comparisons recorded below.

[sources.json](sources.json) records each inspected source's repository,
revision, path, content SHA-256, source kind and reproduction command. It also
records fetched default-branch heads. Working files that match their recorded
commit can be reproduced with the supplied `git show` command. The inspected
`tru/specs/rewards.md` contains concurrent changes: its checksum identifies the
working snapshot; it is contextual evidence, not a claim about that HEAD or an
accepted release. No conclusion here relies on its numerical estimates.

Search commands used `rg --files` for privacy/identity/proof/harness material and
`rg -l -i 'privacy|приватност|shielded|anonymity|mixnet|private.*consensus'` over
the stack's Markdown files, followed by direct reads. The core comparisons
use exact Git objects; local research branches are labeled as such. The work
does not claim to have read every file below `~/cyber`.

## observations that decide the proposal

### the accepted chain and the requested freeze differ

Command:

```text
git -C /Users/master/cyber/soft3 show 8525b174fa227f7def0e368d6ea64e135fdf063a:specs/chain.md
```

The introduction, signatures section, shadow-mode activation section and
security model specify a public chain with classical secp256k1 authorization.
The transfer/cyberlink wire carries the originating neuron, amount and public
destination/payload. Later scheduled activation supplies privacy. The same
revision's `crate/src/wallet/tx.rs` and `crate/src/wallet/mod.rs` implement
those public encodings and NSIG1 signing. This is a direct conflict with
freezing private PQ currency at its initial genesis, rather than a small
missing prover optimization.

The earlier corrected P1 intent is independently located with:

```text
git -C /Users/master/cyber/cyber show db51aa9bb6dd7fa3dc877123e06b8c636f6ab990:launch.md
```

The privacy core explicitly makes the cyberlink public and author private.
This permits public assertion content, while protected weights and reports
still require collective computation. Hidden valence and marginal settlement
are mandatory parts of the requested machine's genesis scope.

### the private prover exists, with narrower evidence than the headline

Command:

```text
git -C /Users/master/cyber/zheng show 09308d7134914e940a41474ad12a772f6b1e92d7:specs/soundness.md
```

This is `origin/release/0.4` at inspection, not the fetched default branch
`5733dfdcc15cbc22fce9f5877b37515d005b8c44`. The ledger describes the masked
Spartan/hiding-opening implementation and explicitly says it is not the VEIL
paper construction. Its Fiat–Shamir argument uses the classical ROM. Its
separate private MITH row explicitly excludes a QROM claim. The recursion
composition row records a conjecture; honest tests do not close it.

The zero-knowledge row gives an interactive statistical-distance bound with
`epsilon_v <= types * log(m) / p`; its example is `50/p`, approximately
`2^-58.4`. These figures are quoted from that exact source by the command
above. They are an upper bound in an argument, not a demonstrated leak or
an attack costing that many operations. They do show that the stated argument
does not establish an unrestricted 128-bit lifetime hiding claim. The same
row's PRF/Fiat–Shamir assumptions require separate compositional treatment.

The hash row records the generic quantum collision scale of the current
four-limb output and experimental inverse-round assumptions. The standalone
soundness and grinding numbers are therefore not a certification of the
whole deployed post-quantum protocol.

Prover randomness was inspected with:

```text
git -C /Users/master/cyber/zheng show 09308d7134914e940a41474ad12a772f6b1e92d7:rs/src/execution/veil/coins.rs
git -C /Users/master/cyber/zheng show 09308d7134914e940a41474ad12a772f6b1e92d7:audit/veil-coins-2026-10.md
```

The production constructor uses fresh OS entropy; deterministic seeds are
identified for test/differential use. Its parallel counter-mode generator has
an ideal-permutation argument. This review neither found seed reuse nor
certifies that argument in the quantum setting.

### use the final latency follow-up, not the superseded timing table

Command:

```text
git -C /Users/master/cyber/soft3 show 8525b174fa227f7def0e368d6ea64e135fdf063a:audit/proof-latency-2026-10.md
```

The final follow-up records a node-tree private membership proof at depth 32
with 96 ms warm proving, 0.18 s cold and 165–181 KB. These are historical
reported measurements from the release-line fast-path work, not reruns by
this research. The source names the component branches/PRs and references
`zheng/audit/fast-path-2-2026-10.md`; the same file's earlier table is explicitly
superseded for that workload. It still records the 64 KB size goal as unmet.
These figures support taking the new prover seriously, while giving no
complete private payment/recovery/settlement performance guarantee.

### the current hash has an unresolved production argument

Commands:

```text
git -C /Users/master/cyber/hemera show d5f0a09539360934fb32db9b2e15a91995607db6:rs/src/params.rs
git -C /Users/master/cyber/hemera show d5f0a09539360934fb32db9b2e15a91995607db6:specs/README.md
git -C /Users/master/cyber/hemera show 2bb9bb623cd97fb685acd7b9882518a2901723af:research/permutation-decision.md
```

The first two are fetched default-branch evidence; the last is a local research
branch. The latter withdraws the old stacked-degree justification and explicitly
leaves full-round security unresolved. Its reduced-round observations are not
full-round attacks. This audit does not assert that Hemera is broken. It rejects
using unqualified security claims as the basis of an irreversible freeze.

### current settlement publishes what P3 asks to protect

Command:

```text
git -C /Users/master/cyber/foculus show 4ae1a79b55791e56e742bb34732212af7042ef25:src/marginal_cert.rs
```

`replay_marginals` compares the recomputed vector with `ticket.marginals`;
the verifier consumes public base/contributor data. Making only the final
certificate succinct does not conceal those public inputs. A hidden-witness
statement and the origin/availability of its complete witness remain separate
obligations.

### the actual aggregate requires a collective protocol

Commands:

```text
git -C /Users/master/cyber/tru show 6b25e4e93defe22a08168c46224bc2bf44cef1a8:specs/attention.md
cat /Users/master/cyber/tru/specs/rewards.md
```

The attention source matches that revision. Its effective adjacency is the
sum of per-contribution attention times karma times the market factor; focus
is the tri-kernel fixed point over that aggregate. The proposed execution cut
therefore has a concrete target: prove each locally available product, encrypt
it, sum encrypted contributions, release the authorized adjacency, then compute
public focus. This is algebraic decomposition, with no throughput claim.

The rewards working snapshot, identified by its existing checksum in
[sources.json](sources.json), explicitly distinguishes standalone propose from
joint epoch settlement. BTS surprise depends on the crowd; Shapley evaluates
coalitions of grouped contributions. Its current conservation correction also
recognizes a nonlinear normalized fixed point. These observations support a
joint hidden-input computation requirement. They do not certify the draft's
economic claims, estimates, or consistency with all accepted component specs.

The original research commit `ea413d43e5f3e4b52e11c589e41d2600f8c2ee25`
placed private collective computation behind a conditional lane while its
recommendation foregrounded owner-only proving and public assertions. That
scope was too narrow for the owner's requested machine. The revised proposal
makes encrypted aggregation and protected joint settlement mandatory genesis
requirements, and separates additive preprocessing from general hidden-input
evaluation. ZK input validity alone supplies neither computation nor disclosure
authority over other neurons' contributions.

The additional primary references opened on the receipt date were
[multiparty BFV](https://eprint.iacr.org/2020/304) and
[OpenFHE security notes](https://openfhe-development.readthedocs.io/en/latest/sphinx_rsts/intro/security.html).
The former explicitly states a semi-honest model; the latter documents active
security limits and unintended packed-slot disclosure. They motivate a concrete
research baseline and its missing obligations. No malicious-secure PQ threshold
implementation, parameter selection or aggregate benchmark was produced here.

### a network-held secret needs a complete lifecycle

The owner's literature lead was resolved to
[Can a Public Blockchain Keep a Secret?](https://eprint.iacr.org/2020/464).
The [lineage review](network-secret-lineage.md) follows ECPSS, YOSO, anonymous
handoff, adaptive composition, PQ PVSS and later timing/erasure work. Its
[source ledger](lineage-sources.json) records access dates, source revisions and
which texts were inspected only at abstract level.

The resulting proposal selects hidden evolving roles with proactive secret
state propagation. It adds explicit genesis, erasure, key retirement,
partition-recovery and computation/consensus dependency obligations. The
freeze unit is the complete implemented and validated protocol. None of these
is designated for completion after launch.

### distributed authority and private execution have different boundaries

The [ICP comparison](../self-2026-10-16/icp-comparison.md) records deployed chain-key signatures,
proactive key resharing and mainnet vetKey derivation from primary sources.
It distinguishes policy-controlled key delivery from private computation over
arbitrary persistent state, and application immutability from the underlying
protocol's upgrade authority. Its source table records inspection scope and
the access date. The comparison adds no execution benchmark or security audit.

This analysis originally accompanied privacy §6.7. It now belongs to the
separate [self proposal](../../proposals/self.md) and its research receipt.
Privacy retains the generic confidential computation/custody/release contract
needed by notes and aggregates. Self consumes that contract for identity,
decisions, bounded authority and a continuing agent lifecycle. The separation
preserves both complete pre-genesis obligations and removes the agent semantics
from the privacy proposal.

## reproduced counterexamples

Source under analysis:

```text
git -C /Users/master/cyber/bbg show c75b68574dd8291311a9a6e9a61fe96b1f5781e0:specs/privacy.md
```

Command executed from the research worktree:

```text
python3 audit/privacy-2026-10-16/counterexamples.py
```

Program revision: Git blob `6526a5ef2492f23cff585a2ceb7b97719e5e09d0`;
SHA-256 `b6a384ee742b4a92a863290fb5883091773a173252976c9b85dbfc394e0bb2e3`.
Soft3 base is the revision recorded above. [counterexamples.json](counterexamples.json)
contains the output and the same script checksum. Every numerical example
below comes from that command/program, using the displayed Goldilocks modulus.

| sketch | reproduced result | scope |
|---|---|---|
| BBG's displayed ciphertext pair | summing its components cancels `a*rho`, leaving the message plus the displayed noise; the illustrative zero-noise value is 7 | identifies the flaw in the formula, not a break of standard RLWE encryption |
| conservation through `norm(S) < q/2` | the constant polynomial 1 has a small norm and passes despite a nonzero imbalance | disproves that test as an exact conservation predicate |
| integer money checked only modulo the field | the positive u64 value `p` is zero in the field | motivates canonical integer/limb conservation constraints |
| public aggregate change | a change from 100 to 107 recovers contribution 7; known summands recover the remaining 17 | inherent output leakage, independent of encryption |
| live-only nullifier reset | an already spent identifier is absent from the cleared live table | counterexample to reset alone; a correctly enforced lifetime accumulator can prevent it |
| nullifier reduced to one field element | distinct integers 0 and `p` alias | full identifier encoding must survive authentication |

All assertions passed. These small exact examples carry no throughput,
cryptanalytic work-factor or production exploitation claim.

## validation of this change

Run on soft3 base `8525b174fa227f7def0e368d6ea64e135fdf063a` plus the
proposal/receipt changes, with release-engine source unchanged:

```text
python3 -m unittest discover -s release -p 'test_*.py' -v
```

Result: 15 tests passed. Printed RED/BLOCKED fixture messages are expected
negative cases within the passing suite, not release results for this branch.

```text
git diff --check
python3 scripts/check-filenames.py --root . --fs
python3 audit/privacy-2026-10-16/counterexamples.py
```

All exited successfully. Local Markdown destinations, balanced code fences,
proposal indexing, JSON source identifiers and the script/output checksum were
also checked. This validates the research artifacts and existing release-engine
tests; it does not exercise or qualify the proposed cryptographic implementation.

The collective-computation correction was validated with the same commands
on `ea413d43e5f3e4b52e11c589e41d2600f8c2ee25` plus its documentation changes.
The release-engine suite again passed all 15 tests. The counterexample script,
filename check, diff check, local Markdown links/fences and source-record
uniqueness passed; both inspected tru files still matched their recorded
checksums. These checks add no cryptographic performance or security evidence.

The network-secret lineage addition was checked on
`b0110398e6279593515cb24271d9b5ebaa972a8a` plus its documentation changes:
the same release-engine command passed all 15 tests, and the diff/filename
checks passed. Local Markdown links/fences, both source ledgers' identifier
uniqueness, the new lifecycle-gate link and the recorded foculus/mudra checksums
were checked with Python assertions. The release-engine and counterexample
program sources were unchanged. These are artifact checks, not ECPSS tests.

The autonomous-state/ICP addition was checked on
`ee3bd0ddab2f1f7682412192e89937758f6f515c` plus its documentation changes.
The same `python3 -m unittest discover -s release -p 'test_*.py' -v` command
passed all 15 tests. `git diff --check` and
`python3 scripts/check-filenames.py --root . --fs` passed. Python assertions
checked local Markdown destinations, balanced fences, ICP reference definitions
and the new proposal section target. No cryptographic program source changed.

## external evidence and limits

Primary-source pages for WHIR, Arc, VEIL, mutator sets, anonymous PQ encryption,
secure aggregation, Loopix, the anonymity trilemma, BHT, FIPS 202 and FIPS 203
were opened. The proposal links each at the claim it supports. This was an
architectural literature review; it did not reproduce their security proofs.
In particular, a paper's random-oracle claim or its use of hashes does not
automatically certify our complete protocol in the QROM.

No native source, accepted contract, cryptographic parameter, release pin,
working-tree change belonging to another task, or default branch was modified.
The proposal and this receipt preserve the distinction between a selected
research direction and an implemented, independently reviewed production system.
