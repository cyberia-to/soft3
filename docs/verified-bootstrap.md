---
title: verified bootstrap on soft3
tags: cyber, soft3, bootstrap
status: accepted
milestone-status: open
---

# Development and bootstrap on soft3

The accepted design starts from our reviewed native [[nox]] executor, establishes
[[trident]], then establishes a native [[rs]] toolchain through an interpreter of
restricted Rs written in Trident. These routes build the Rust and Trident
implementations of the complete declared stack. Implementation and acceptance
remain open. The ceremony below describes future work, not commands already run.

This page owns the cross-component order and trust boundary. Component work and
acceptance checks live in the
[Trident VB0–VB8 plan](https://github.com/cyberia-to/trident/blob/docs/0.4-dual-implementation-bootstrap/roadmap/verified-bootstrap.md)
and the
[Rs RS0–RS9 plan](https://github.com/cyberia-to/rs/blob/docs/verified-bootstrap-rs/roadmap/verified-bootstrap.md).
The links name the coordinating feature branches until their owner merges them.
Specifications and implementations stay with each component owner; future
receipts belong in each owner's `audit/verified-bootstrap/`.

## Decisions and claim boundaries

| Decision | Status and consequence |
|---|---|
| Native nox seed retained | Accepted design; native bytes, readable instruction mapping, loader and I/O contract receive direct review |
| Restricted Rs interpreter on Trident | Accepted design; the first native Rs comes from our own source-processing route |
| Own Rs frontend | Required development; the present rustc driver/plugin cannot close this route |
| Critical implementations in Rust/Rs and Trident | Mandatory for compilers, interpreters, Eidos, nox, full scoped Zheng producer/verifier, complete scoped Joy and dependencies |
| Rust compatibility | A product obligation of Rs; the compiler's implementation language can remain smaller than its accepted language |
| Initial root platform | Linux AArch64 design baseline; macOS AArch64 and other root platforms need separately reviewed images and host contracts |
| All VB, Eidos and Rs acceptance gates | Open; identifiers describe checks to implement |

The existing Trident SH0–SH8 receipts cover the frozen compiler's self-compilation
on nox through Joy and its disclosed public proof profile. Those measured results
keep their original sources, names and scope. VB adds independently established
source correspondence and semantic obligations. It requires all private/public
profiles in the frozen delivery, including the complete producer and Joy.

The common reviewed root is an explicit shared assumption. Separately authored
implementations supply diversity above it; source translation or running one
algorithm through two executors cannot count as an independent implementation
pair. Record shared encoders, generators, libraries and specification ancestry.

## Two language domains in Rs

Call the compiler implementation profile B. Its grammar, types, ownership,
arithmetic, evaluation order, effects and resource behavior are frozen by RS1.
Every source program executed during initial construction must fit B, including
the build driver, dependencies, expansion/generation tools and native emitter.
Unsupported features fail admission. A strict lint over an upstream Rust build
does not establish that the actual source closure belongs to B.

Call the language accepted by the completed compiler L. Its eventual Rust
compatibility contract remains a separate required deliverable. A compiler
written in B can implement L: the initial interpreter only executes that
compiler's B source, while the compiler processes L source as data. Early B
support and coverage of the frozen stack are intermediate milestones; neither
closes the full compatibility obligation.

Rs owns independently authored B interpreters and compiler implementations in
Rust/Rs and Trident. Interpreting the Rust/Rs compiler source with the Trident
interpreter establishes another execution/provenance route for that compiler;
the separately authored Trident compiler remains required for the compiler pair.

## Artifact names and execution environments

Names below belong to the future ceremony. Historical SH artifacts retain their
original C1/C2/C3 names. Every named artifact has exact source, configuration,
dependency and target identities in the future manifest.

| Name | Meaning | Executes on |
|---|---|---|
| N_seed | Reviewed native nox executor | Initial ISA/OS platform |
| K_seed | Directly reviewed minimal Eidos checking image | N_seed |
| I_tri | Admitted Trident source interpreter, a `.nox` program | N_seed |
| T_root | Trident compiler obtained by source interpretation | N_seed |
| I_rs | Admitted interpreter of B, written in Trident and compiled to `.nox` | N_seed |
| R_first | First native Rs compiler/toolchain obtained by interpreting its B source | Initial ISA/OS platform |
| R_rebuilt | Native Rs rebuilt by R_first from the same frozen source recipe | Initial ISA/OS platform |
| Stack_tri | Complete scoped Trident implementations compiled to `.nox` | Accepted nox/Joy; root replay uses N_seed where required |
| Stack_rs | Complete scoped native Rust/Rs implementations built by accepted Rs | Declared native platforms |

An executor's platform and its output format are separate. I_rs runs on nox and
can emit native executable bytes. A full nox written in Trident is itself a nox
program; running it still needs an underlying executor. A native nox built by
Rs is a checked descendant. N_seed remains the retained origin of its provenance.

## Ceremony: what executes and what gets built

Each row requires the preceding accepted artifacts, the relevant owner
specifications and the stated root-checked evidence. Proof producers can be
ordinary development tools; the accepted checker independently validates their
complete output against the externally selected statement and exact files.

| Step | Executor and inputs | Output | Acceptance boundary |
|---|---|---|---|
| 0 | Review native instructions, reconstruct their bytes, check loading and input/output | N_seed | VB1: readable source/byte mapping, resource/failure rules, independent reconstruction/readback and explicit hardware/OS assumptions |
| 1 | N_seed executes the directly reviewed kernel image on closed terms and rejection fixtures | K_seed available as root checker | E0/VB1: reviewed logical rules, decoding, premises and exact expected-statement binding |
| 2 | K_seed checks the semantics and exact `.nox` translation of the Trident source interpreter | I_tri | VB1: admitted parser/evaluator, complete import closure, bounds and executable provenance |
| 3 | N_seed runs I_tri, interpreting parent compiler source P_tri with compiler source A_tri as its input | T_root | VB2: compare with the artifact attributed to that exact parent/source/recipe; VB3 separately checks compiler semantics or complete delivered translations |
| 4 | N_seed runs accepted T_root on the Trident source of the B interpreter and its dependencies | I_rs | RS1/RS2: root checks B semantics and the exact emitted image; compilation success alone leaves these obligations open |
| 5 | N_seed runs I_rs to interpret the B compiler/build-driver source with its own complete source bundle, target and recipe as input | R_first and its required runtime/build artifacts | RS3–RS5: source closure, native encoding/linking, ISA/ABI semantics and artifact refinement checked before relying on R_first |
| 6 | The host executes accepted R_first on the same frozen source bundle, target and recipe | R_rebuilt | RS5: exact executable and behavior-affecting metadata comparison; root-checked source/native obligations remain separate |
| 7 | Accepted Rs compiles the frozen Rust/Rs stack sources; accepted T_root compiles the paired Trident sources | Stack_rs and Stack_tri | RS6–RS8, VB3–VB6 and Eidos obligations for every artifact, dependency, proof profile and host effect |
| 8 | Fresh environments consume the complete retained package and check proofs and interoperability | Independently replayable delivery evidence | RS9, E4, VB7/VB8: all mandatory pairs/profiles/platform claims, negative controls and original failures accounted for |

In step 3, direct interpretation executes P_tri on A_tri once; it adds no
intermediate compiler generation. For the frozen S1 experiment P_tri = A_tri =
S1 and the comparison target is historical C2. In later generations P_tri can
differ from A_tri, so both identities are mandatory. The alternative compiled
DDC construction and its two generations are specified in the Trident VB2 plan.

Step 5 interprets the compiler as a program and supplies its source bundle as
that program's input. The job includes all build operations needed to produce a
usable native compiler: libraries, generated inputs, object emission, linking,
runtime and builtins. Any helper executing outside B must first have its own
admitted source-to-artifact route. The initial recipe uses B throughout its
software build closure, avoiding such a hidden prerequisite.

Step 6 compares outputs of the same compiler source algorithm and deterministic
recipe through two execution routes. Agreement alone proves reproducibility.
Source correspondence depends on the admitted interpreter, inputs, executor and
comparison; native behavior depends on the ISA/ABI refinement obligations.
Different independent compiler algorithms can emit different layouts; their
semantic comparison is separate from this exact self-build comparison.

## Root availability and Eidos product acceptance

The minimal logic, K_seed image, N_seed and physical execution contract are
reviewed foundational assumptions. Downstream formal claims are conditional on
them. The source/statement decoder, comparison and result readback are inside
this boundary; a producer-controlled manifest cannot choose the theorem or
declare acceptance.

Root expressiveness and model availability precede every certificate check.
Before step 2, the admitted root must express the Trident interpreter statement;
before steps 4–5 it must express B and native ISA/ABI/refinement statements. A
root too weak for a claim keeps that step open. Conservative checker extensions
can be certified by the accepted checker; stronger logical rules require a
separately reviewed root version and an updated assumption ledger.

Eidos E1–E3 construction and semantic models may therefore proceed from E0 and
reviewed root extensions while final compiled-pair acceptance remains open.
Full Eidos product acceptance E4 waits for the established compiler/executor
routes, native toolchain acceptance and complete proof corpus. A later compiled
Eidos cannot be the sole authority for the earlier artifacts that built it.

RS8 toolchain acceptance uses this already admitted root. RS9 replays the
toolchain independently and feeds VB8. E4 may depend on RS8 for native Eidos
provenance; RS8/RS9 never depend on E4 or VB8 to establish their own prerequisites.

## Development order and reviewable deliveries

Work is packaged by component and useful acceptance slice. Prototype code and
proof production may proceed in parallel; acceptance follows these dependencies.

| Delivery | Work | Evidence needed before proceeding to the dependent accepted step |
|---|---|---|
| D0: inventory | VB0 + RS0: pin sources, binaries, language domains, all pairs/profiles/platforms, proof needs, tools and assumptions | Complete operation/claim matrix; missing input, hidden helper and mismatched profile are rejected |
| D1: root experiment | E0/VB1: minimal nox profile, readable native seed, initial kernel and one small real source/artifact claim | Reconstructed bytes; valid and corrupted inputs; exact statements; explicit memory/work exhaustion; measured cost |
| D2: Trident origin | I_tri admission, frozen compiler correspondence and compiler semantic/translation obligations | VB2/VB3 for the exact declared compiler and translations; preserve historical SH evidence |
| D3: Rs foundation | RS1–RS4: B contract/interpreters, own frontend, native emission/linking and complete B self-build closure | Root-checkable B and ISA models, real source-to-native examples and complete self-build inventory |
| D4: native Rs origin | RS5: interpreted first build and native rebuild | Root-derived artifacts, exact recipe/output comparison and native refinement; absent old stage0/rustc/LLVM on root replay |
| D5: complete implementations | RS6–RS8, E1–E3 and VB4–VB6: accepted-language coverage, dependency/build closure, both implementations of the full stack | Every operation/profile has source/artifact/refinement evidence and cross-implementation checks |
| D6: final replay | RS9 + E4 + VB7/VB8: full package, adversarial fixtures, platform consumption and original receipts | Reproduction by a fresh environment; all mandatory gates pass for one manifest |

Start with D0 and a small D1 experiment. Size later work from measured execution,
memory and root checking costs in sessions/pomodoros. No calendar estimate or
full-stack proof-performance claim has been established. Keep an independently
useful result and its evidence in each feature PR.

Trident/Joy/Trisha deliveries target `release/0.4` through feature PRs, preserving
their default branches. Rs and soft3 currently have no `release/0.4` integration
branch; their documentation uses separate feature PRs to their existing default
branches for owner review. This grants no merge, version, tag or publication
authority. A future implementation integration branch is recorded before its
first delivery. Existing release-train policies still govern actual candidates.

## Platform and resource contracts

Root execution, native output targets and package-consumption platforms have
separate manifest rows. Linux AArch64 is the initial root design target. A
macOS AArch64 root has a separate syscall/loader image and direct review.
Cross-compiling for another platform requires that target's codegen/linker
correctness; executing it adds that platform's explicit physical/OS assumptions.

Retain the existing Trident/Joy/Trisha macOS/Linux/Windows ARM64/x64 delivery
claims and their exact evidence. Rs's current macOS/Linux product plan and
soft3's release-train matrix retain their own scopes. A final VB package claiming
any native Windows artifact also needs an admitted Windows-producing build
route; a Linux-only pilot leaves that row open. No matrix silently overrides
another component's claimed delivery.

The root profile fixes representation, field/word arithmetic, Hemera identity,
decoding, errors and logical charge. Every required operation is implemented;
unsupported witness/state operations fail explicitly. Jets, cached answers and
host semantic helpers require separate justification before enabling them.
Memory management, alias/lifetime behavior and allocation failure are semantic
obligations. Fuel limits the work of an attempt; adequate fuel and storage for
successful self-build require separate evidence.

Seed-size projection, status open: for a compact deterministic Linux AArch64
executor, the discussion estimated 1,600–3,400 instructions, roughly 8–16 KiB of
code/constants, with a planning allowance up to 32 KiB. ELF packaging was
estimated at 16–32 KiB and Mach-O at 32–64 KiB. These are unmeasured design
estimates, assuming bounded storage, simple I/O, scalar arithmetic and Hemera,
and excluding the separately loaded Eidos/compiler programs and proof producer.
They impose no acceptance threshold. D1 must measure actual code/data/file size,
peak memory and execution time on its exact revision; memory reclamation and
real compiler workloads can invalidate these estimates.

## Final evidence and rejection conditions

The future manifest records complete source and artifact identities, grammar
profiles, imports/features, target triples, recipe/environment, expected
statements, root/rule versions, proof dependencies, resource caps, host effects,
comparison rules, commands, original results and assumptions. Every measured
number names its command and revision. Retain all files required for offline
replay; pin every generator and generated result together.

The accepted root replay operates with the old Rustc/mrustc/LLVM toolchain and
sealed stage0 absent. Those tools may assist development and propose proofs;
their success cannot replace root-checked obligations. The OS/loader/firmware
boundary remains explicit. C/C++-free build tools do not establish that physical
boundary or logical soundness by themselves.

Reject missing imports, unrecorded tools or prebuilt libraries, altered seed
bytes, incorrect arithmetic, inconsistent memory behavior, unsupported B syntax,
omitted compilation units, unsafe capability escapes, changed proof rules,
statement swaps, source-visible semantic violations, inherited binary additions,
false verification and resource exhaustion presented as success. Preserve
positive controls so a blanket refusal cannot pass.

Whole-stack closure includes all scoped Zheng producers/verifiers, public and
private profiles, complete Joy decisions/adapters and dependency algorithms.
Joy stays on soft3/cyber; Trisha owns Triton/Neptune. This milestone claims no
Triton self-hosting. Proof search can be fallible; the root checks the resulting
closed evidence, and required producer correctness/privacy obligations remain.

A completed milestone publishes the exact conditional claims: source
correspondence, admitted semantic preservation, prover completeness, verifier
soundness, privacy assumptions and delivery coverage. Hardware correctness,
unmodeled leakage and unrestricted absence of malicious intent retain their
explicit boundaries.
