---
title: self proposal research receipt
tags: soft3, audit, self, privacy, cy, neuron, research
crystal-type: report
crystal-domain: cyber
date: 2026-10-16
status: findings
---

# self proposal research receipt

Companion to [self](../../proposals/self.md) and its dependency on
[privacy](../../proposals/privacy.md). This change separates the private
protocol from the private neuron's ability to maintain identity, decide and
act. It introduces no production implementation or cryptographic qualification.

## provenance and scope

Soft3 source: `10f72625ecb8b30215619e93cc195b71576d699a` on
`research/privacy-freeze-20261016`, in the existing isolated worktree
`/Users/master/cyber/.worktrees/soft3-privacy-freeze`. The proposal split was
made on that source. Other repositories were read only.

[sources.json](sources.json) records inspected repository paths, revisions,
branches, content checksums and reproduction commands. A committed snapshot
is evidence of that revision, not proof of inclusion in a release or even in
the remote default branch. Mudra and BBG were inspected on the local branches
named in the ledger. Cy's anatomy page had working changes; its checksum
identifies the inspected context and it is not a release input.

The review read the existing soft3 execution/ownership contracts, cy's agent
declaration/permission pages, neuron's identity/execution/effect/history
contracts, and mudra's authority/confidentiality separation. It did not run
the neuron engine, inspect every implementation path or benchmark a private
model. The claims below distinguish documented local capabilities from the
new network profile's obligations.

## observations and decisions

| observation | exact local source in the ledger | decision |
|---|---|---|
| soft3 composes VM/OS, proof, network and executor selections | `soft3/specs/execution-model.md` | self uses the existing composition contract; worker replacement is a placement change |
| cy describes a mind over a neuron's book, with soul/ward and network-independent startup | `cy/README.md`, `cy/specs/agent.md`, `cy/specs/ward.md` | cy owns agent decisions and permission semantics; the distributed deployment must preserve the local boundary |
| neuron already owns a subject with separate program state and a durable execution workspace | `neuron/README.md`, `neuron/Cargo.toml`, `neuron/specs/execution.md` | extend neuron for the private network profile instead of creating another subject/runtime owner |
| local authority uses a classical profile and local worker fencing explicitly excludes partition-safe remote takeover | `neuron/specs/identity.md`, `neuron/specs/worker-dispatch.md` | existing local behavior does not qualify private PQ network authority, finality or secret succession |
| cybergraph/BBG own history and atomic persistence; local records and consensus evidence differ | `neuron/specs/history.md`, `bbg/specs/native-state.md` | reuse shared storage with an explicit private/network admission relation; no new canonical agent database |
| mudra separates proof-based authority from encryption, private computation and key distribution | `mudra/README.md`, `mudra/specs/identity.md` | privacy remains the cryptographic dependency; self supplies application authority/disclosure semantics |

The inspected anatomy overview describes ward as live, while its detailed ward
page still lists implementation stages. This review does not resolve that drift
or infer a qualified permission boundary from either status label. Self requires
enforcement inside its verified transition and at the actual effect boundary.

Some neuron pages link to soft3 convergence documents absent from this soft3
base. The proposal relies on the ledger's inspected neuron contracts and the
present soft3 execution model, without importing an uninspected convergence
decision or altering the default checkout to manufacture one.

Repository decision: use existing homes. Neuron owns subject/runtime semantics,
cy owns the agent process, and privacy's components supply protected execution
and secret operations. A new `self` repository would duplicate those owners
before an independent release boundary or distinct responsibility is established.

## split ledger

| previous material | destination and disposition |
|---|---|
| privacy §6.7: private autonomous state, identity, authority, learning and continuity | developed in `proposals/self.md`; removed from the privacy proposal |
| privacy's private notes, encrypted programs, aggregate/settlement functions and key lifecycle | retained in privacy because they are required for the private protocol itself |
| coupling between application meaning and cryptographic mechanisms | privacy §1.1 owns the service contract; self §§1 and 6 consume it and define composition evidence |
| `audit/privacy-2026-10-16/icp-comparison.md` | moved to [icp-comparison.md](icp-comparison.md), with the former wiki path preserved as an alias; source access dates and inspection scope retained |
| the privacy receipt's earlier autonomous-state finding | kept as historical evidence and redirected to self; the prior source commit preserves the original section |

The dependency is one way: self depends on privacy. Privacy's native aggregates
and consensus do not depend on self's decisions. Both proposals remain drafts;
the requested private self needs both completed protocols and their composition
qualified before genesis. No implementation work is deferred past launch by
this documentation split.

## external evidence

The [ICP comparison](icp-comparison.md) retains the primary-source review from
the earlier research. This split adds repository analysis and a proposed
composition; it introduces no new claims about a deployed external system.

## validation

Executed from the research worktree on
`10f72625ecb8b30215619e93cc195b71576d699a` plus this documentation change:

```text
python3 -m unittest discover -s release -p 'test_*.py' -v
git diff --check
python3 scripts/check-filenames.py --root . --fs
```

The release-engine suite passed all 15 tests. Its RED/BLOCKED lines are expected
negative fixtures. Diff and filename checks passed. One-off Python assertions
also checked local Markdown destinations and heading anchors, balanced fences,
reference definitions, source-ledger uniqueness and checksums, proposal registry
entries, the dependency link, removal of the old actor section and preservation
of privacy's aggregate/settlement and key-lifecycle gates. The moved comparison
retains its old wiki path as an alias.

These are documentation and existing regression checks. No private self was
implemented, benchmarked or security-certified by this change.
