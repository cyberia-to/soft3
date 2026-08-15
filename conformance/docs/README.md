---
tags: cyber, soft3, conformance
crystal-type: entity
crystal-domain: cyber
alias: conformance documentation, why conformance
---
# why conformance

[[soft3]] is fourteen repos that all serialize, hash, prove, and verify each other's values. one repo silently changes an encoding; every snapshot, proof, and signal produced before that change becomes unverifiable. the harness catches the change at the moment it happens, not months later at a downstream verifier.

## the federated drift problem

an encoding change surfaces in three places at different times:

- the author who changes the encoding sees green tests — local tests pass because the encoding stays internally consistent
- a downstream verifier rejects previously valid commitments weeks later
- protocol governance learns afterwards, from the rejection

each link in this chain is invisible to the others. silent drift across repo boundaries is the failure mode of any federated stack.

## the harness model

a single rule: every value whose encoding the protocol depends on registers a [[hemera]] fingerprint as a snapshot. every mechanism whose output the protocol depends on registers a fingerprint of its output on a fixed input. CI compares each snapshot to the registered fingerprint. drift fails the build. the snapshot moves only through a tier-appropriate bless ceremony.

## why hemera makes it cheap

a snapshot stores 32 bytes — the [[hemera]] fingerprint of the canonical encoding. no test corpus, no fixture files, no encoded payloads on disk. one `conformance/encoding.snap` file per crate, ~100 lines, covers every stable type.

## why nox makes mechanisms work

a mechanism snapshot is the hemera fingerprint of a [[nox]] program's output on a fixed input. nox executes deterministically by construction, so the scenario is its own simulator. mechanism conformance reduces to encoding conformance applied to a deterministic VM trace. same input, same nox version, same output, same fingerprint.

this also dissolves the "deterministic simulation runtime" gap separately: nox-as-simulator is stronger than a swappable async scheduler, because the trace is provable as well as reproducible.

## the provable manifest

the conformance/ directory of every soft3 repo is itself a value. its canonical encoding has a hemera fingerprint. the union across all soft3 repos is the protocol's stability manifest at a given git revision. a [[zheng]] proof can be produced over this manifest root, attesting "this protocol revision conforms to manifest M" to any verifier without re-running the harness.

a field-native fingerprint stack makes the manifest a witness, not just a hash. snapshots become proof inputs.

## relationship to [[Commonware]]'s conformance primitive

Commonware introduced the idea of a conformance crate: encoded forms and mechanism outputs that should stay stable. their primitive uses traditional hashes; the snapshot is a fixture, the harness is a test.

cyber's version inherits the discipline and gains provability for free, because every fingerprint is already a Goldilocks-field element. the harness produces witnesses that ride the same proof rails as everything else in [[soft3]].
