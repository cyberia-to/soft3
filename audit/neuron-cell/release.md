---
title: neuron convergence release audit
date: 2026-09-13
status: passed-for-declared-local-profiles
---
# Neuron convergence release audit

This audits [the accepted P00–P13 plan](../../roadmap/neuron-cell-convergence.md),
including the legacy data migration, against the implemented workspace. The
chronological [ledger](implementation.md) retains intermediate failures and their
repairs. This document describes the resulting source/profile set; a local build
is distinct from a published package or a production deployment.

P00–P13 are implemented and G01–G14 pass for the declared local profiles below.
The [validation manifest](validation-manifest.json) binds this review to the
[source manifest](source-manifest.json), [122 final dispositions](final-repositories.md),
actual feature/toolchain invocations, checksummed logs and verified artifacts.
Source discovery records 116 Git roots/markers plus six non-Git groups; 56 selected
source roots supply 12,902 file hashes and all 87 required local build manifests.
Historical observations below remain distinguishable from final integration runs.

## Resulting model

One protocol subject, neuron, carries native or qualified foreign identity.
A robot attaches subjects across keys, networks and devices. Programs, tasks,
invocations, operations and checkpoints identify work/data. Several installed
progs and concurrent jobs share one neuron's finite resource account. Subject
selection never supplies a later asynchronous action's identity or grant.

Cybergraph and BBG own shared history, native/application records and durability;
GraphSession supplies the multi-author host view. Log renders history. Ward and
Vault roles bind current authority and custody through the existing owners.
Rune supplies bounded execution and continuations; Soma supplies durable agent
composition and actual local model/tool turns. The former cross-scale cell
ladder now states the separate service, book, shard and node-mode duties.

## Plan closure matrix

| Packages / gate | Implemented requirement and executed evidence |
|---|---|
| P00 / G01 | Passed. The final scan independently discovers repositories/worktrees and non-Git sources. Every discovered entry receives a disposition; the original cell entry follows its Git-preserving move to neuron. Generated markers, independent worktrees, foreign/vendor code and unavailable Omi blobs are explicit. The source/disposition manifests preserve the original baseline separately. |
| P01 / G02 | cyb/specs/neuron.md, soft3/specs/neuron.md and neuron/specs/identity.md use one subject. cyb anatomy and the robot registry distinguish keys/networks/devices from progs and tasks; current neuron/cyb/Crystal/Prysm/Soma documents agree. Registry and captured-command scenarios exercise this model. |
| P02 / G03 | Accepted neuron specs cover model/data/lifecycle/execution/history/authority/migration/navigation/action records and worker dispatch. Original cell/*/1 manifests/hashes remain immutable; new neuron/prog records have bounded codecs, validation and explicit migration. Model, reader-corruption, original-binary and process suites pass. |
| P03 / G04 | neuron-id remains dependency-free; model identity/bindings/routes can compile without runtime records. Seven isolated WASM roots independently compile and reject VM/GUI/BBG transitive dependencies. Existing native derivation/domain/signature fixtures and both browser consumers pass. |
| P04 / G05 | One Database composes native/application domains, strict legacy log import and durable GraphSession. Failed writes/uncertain barriers produce errors or frozen views, never invented success. Eight in-memory Signal atomicity regressions also pass. Full BBG and cyb-core suites plus real TextArchive capacity/provenance/race/reopen cases pass. |
| P05 / G06 | Multiple progs/jobs, stale-state conflicts, persistent resource accounting, parent/child allowances, pause/cancel/retire/upgrade/archive and unknown-effect recovery are implemented. Neuron all-feature suite and Soma task/child/control/recovery scenarios pass. An unknown tool does not block an independent program. |
| P06 / G07 | Supported NSIG1 authorization binds subject/network/policy/current grant to exact operations. Publication and physical dispatch recheck authority; worker placement generations and one-use attempt claims fence stale execution. Registry races, revocation/wrong-network/selection changes, independent shared adapters, custody and original-writer fence tests pass. |
| P07 / G08 | Same-store semantic import, separate-store staging, keyless sealed inspection and logical export preserve the source. Three original origins map into one authenticated neuron with separate progs and original states/history/claims/charges/reservations/unknown attempts. Exact retries, process kills at four transfer boundaries, closure corruption, cursor atomicity and storage faults pass. See [cutover audit](legacy-cutover.md) and [operator guide](../../../neuron/docs/legacy-cutover.md). |
| P08 / G09 | Robot registry, headless commands, captured shell actions, typed destinations, foreign watch attachments and device bindings use the same model. Vault/private notes survive restart with ownership. Final Fleet: 32 checks passed across three real GUI bodies, signed native node, readonly beacon and blackhole; all owned processes stopped. |
| P09 / G10 | Soma agent composition supports actual Rune/worker suspend/resume, bounded tool schemas/adapters, children/joins, controls, context/model changes and stable scheduling/recovery. The local pinned model executes through both actual CLI and Bevy bodies; their durable terminal records reopen without another inference and preserve the original attachment. Actual answers were four and seven; the Bevy screenshot was inspected. See [Soma audit](../../../soma/audit/neuron-composition.md). |
| P10 / G11 | [Consumer audit](consumers.md) covers Mudra/lytics, both cyberia-my roots, Trisha, soft3/js, true-cyber, Inf and foreign adapters. Native and foreign derivation/signing/storage bytes retain their existing profiles. The lytics live/replay mismatch was repaired; missing historical key provenance is reported without invented identities. |
| P11 / G12 | Compact cyber/specs/domain-ladder.md and canonical cyber/AOS/Cyberia/MiDAO/Crystal/cyb/Prysm pages preserve shard split, issuer/ledger authority, service governance and full/partial/light responsibilities. Foculus NodeMode::Partial preserves its former variant's branch duties and ordinal; five owner tests pass. Biological, VM, storage and table cells retain their meanings. |
| P12 / G13 | Passed. Source directory, package/API paths, standalone CLI registry, facades, typed URI compatibility, service/build docs and both sites/context have been migrated. The remote build closure resolves 24 sibling repos/82 local packages/87 manifests; local dry-run makes zero transport calls. [Compatibility allowlist](compatibility.json) records immutable readers, exact landing aliases, dated snapshots and the two API shim removal releases. Context output and all 43 selected source hashes match; both sites rebuilt. DMG and APK contents, versions and packaged binaries verify. |
| P13 / G14 | Passed for the declared profiles. Exact dirty source hashes, Cargo locks/declarations, actual activated feature/toolchain profiles, owner-test logs and final artifacts are bound by the validation manifest. Every preceding gate has executed evidence below. Binary dependency files also confirm no newer or missing compiled inputs for the five final native executables; this is a freshness check, not a reproducible-build proof. |

## Executed verification

These are final named suites, rather than a sum of every chronological rerun.
Harness totals include subprocess helper cases where the owner uses them.

| Final run | Result |
|---|---|
| Neuron workspace, all features | 48 passed; 0 failed/ignored |
| Neuron Clippy, workspace/all targets/all features/no-deps, warnings denied | Passed; upstream Fjall warnings remain outside no-deps scope |
| BBG, all features | 162 passed; 0 failed/ignored; includes storage fault and migration process helpers |
| cyb-core | 52 passed; 0 failed/ignored |
| Cybergraph repaired API + historical stack targets | 57 default / 93 all-features passed; subsequent TextArchive changes separately verified below |
| TextArchive final capacity/preflight/CAS/provenance suite | 4 boundary + 3 existing + 5 GraphSession integration tests passed |
| Rune interpreter after driver rename | 84 passed; 0 failed/ignored |
| cy CLI after tool registry/dispatch migration | 16 passed; 0 failed/ignored |
| Foculus live mode rename | 5 passed; 139 unrelated cases intentionally filtered |
| Remote build-script closure/refusal checks | 8 passed; real dry-run used no SSH/rsync/scp |
| Seven isolated identity-only WASM roots | All compiled; dependency-weight checks passed |
| Fleet, fresh cy/cyb/soft3 binaries | 32 passed; 0 failed; all owned processes stopped |
| Actual CLI and Bevy local model tasks | Both passed; reopened terminal task state unchanged; inspected screenshot shows seven |
| Cyber product launcher, fresh release executable | Six real process scenarios + one unit test passed |
| Local protocol/project-site builds | Both passed; current source aliases and neuron subgraph included |
| Web apps | Trunk release build passed |
| Android | Source-fresh Rust library + release APK passed; ZIP integrity, packaged arm64 library equality, signature and ai.cyb.app version 0.13.1 verified |
| macOS | Release and DMG passed; readonly mounted binary/web asset equality, image checksum and bundle version 0.13.1 verified; owned mount detached |

Primary final logs are under [implementation-baseline](implementation-baseline/):
`p13-neuron-all.log`, `p13-neuron-clippy.log`, `p13-bbg-all.log`,
`p13-cyb-core-all.log`, `p13-cybergraph-stack-{default-final,all-features}.log`,
`p04-text-archive-boundaries*.log`, `p12-rune-events.log`,
`p12-cyb-cli-final.log`, `p13-identity-consumers.log`, `p13-fleet-source-final.log`,
`p13-real-tasks-source-final.log` and `p13-cyber-launcher-release-tests.log`.
The last task run uses the actual release Bevy binary and fresh debug CLI.
Selected non-secret graphical/task evidence is retained in
[p13-bodies](implementation-baseline/p13-bodies/).

The [artifact manifest](artifact-manifest.json) identifies the macOS binary/DMG,
Android APK/staged library and web entry point by SHA-256. Packaging fixes normalize
Trunk's NO_COLOR value, allocate DMG filesystem headroom and derive both macOS
bundle versions from Cargo. No application source was replaced by a mock body.

## Declared profiles and practical boundaries

- Native subject: existing secp256k1 compressed-pubkey hash identity, NSIG1 local
  authority, fenced writer and exact signed native adapter. Endpoint acceptance
  remains separate from consensus finality. Cross-device remote multi-writer
  coordination requires its own protocol evidence.
- Durable execution: local bounded Rune worker, supported task/model/tool
  adapters, retained continuations and explicit unknown outcomes. Trusted native
  console and cooperative GPU cancellation provide their documented local
  guarantees; this audit asserts no hostile-process confinement or hard GPU
  preemption.
- Legacy transfer: application-only SSD source, up to 256 namespaces, 1M inspected
  rows and 8 GiB logical bytes; bounded pages and explicit target mapping. Redb
  conversion is separate and tested. Disk headroom is observed, not reserved.
  No production store was silently reassigned to a new owner or discarded.
- Proof execution: supported public DirectProof authenticates complete BBG reads
  at the selected root and reveals its declared witness. Recursive TensorMerkle
  openings remain explicitly unsupported by zheng. Shard/finality/governance
  specifications retain their obligations without claiming this local build
  implements every distributed proof profile.
- Consumer compatibility: existing Cosmos/CosmWasm/Cybernet/Neptune/EVM contracts
  remain foreign profiles. Native adapters do not reinterpret their IDs/keys or
  rewrite deployed storage. The 12,397 unavailable Omi index blobs are a named
  source gap; locally available sparse blobs were scanned.
- Delivery: current macOS desktop, Android build, web apps and identity-only
  WASM profiles are the executed local targets. Linux/Windows remote packaging
  has validated closure/command/failure handling but no remote build/deploy was
  performed. No published-version, remote clone availability or production
  rollout claim is inferred from the local workspace.
- Agent scope follows plan section11: the converged execution and local agent
  composition are implemented. Full Hermes provider/channel/tool breadth and
  comparative performance gates belong to the subsequent agent roadmap.

The existing soft3 conformance scaffold returns a placeholder hash and supplies
no release evidence here. Source/log SHA-256 checksums describe reproducibility;
protocol commitments and immutable-byte checks come from their actual owners.
The [residual review](residual-review.md) records why remaining cell terms and
parallel worktrees are preserved rather than globally replaced.
