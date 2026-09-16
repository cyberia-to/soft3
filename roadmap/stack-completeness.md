---
title: stack completeness
tags: cyber, soft3, roadmap, architecture
crystal-type: roadmap
crystal-domain: cyber
status: draft
---

# stack completeness — coverage, gaps, and the deep overlaps

A map of component ownership and the remaining integration work. The
2026-09-13 update reflects the implemented neuron/prog model, shared graph host
and local ward/vault/Soma profiles. The [execution contract](../specs/execution-model.md)
defines compatibility and [neuron](../specs/neuron.md) defines subject identity.
The [implementation ledger](../audit/neuron-cell/implementation.md) carries
test evidence and pending acceptance gates; architectural coverage alone does
not establish a release, network deployment or Hermes parity.

## the layered model

the stack is layered, not a flat list of repos. each layer reduces toward the one below it, and the three through-lines (one field, one proof, one focus) run vertically through all of them.

```text
app          cyb (embody)             lytics (measure)
value        tok (pay)
cognition    soma (think)
capability   ward (authorize)
subject      neuron (act: identity + optional durable progs)
custody      vault (keep; host-bound secret access)
present      mir (render world)        prysm (paint UI)
network      mudra · radio · tade · foculus
dynamics     tru (converge φ*)         foculus (agree)
graph        cybergraph · bbg · fs
proof        zheng (execution)         eidos (theorem)
runtime      nox · wysm · glia · wgpu  (the proof-contract ladder)
language     trident · neural · rune · inf     (+ cybermark, nu host)
commit       hemera (hash)             lens (commit)
math         strata — nebu 𝔽_p · genies 𝔽_q · jali R_q
substrate    honeycrisp (accelerate)
```

The rows describe roles; authority, custody and evidence cross several rows.
Zheng proves a declared execution statement; eidos checks theorem terms; mir
renders the graph and prysm renders an interface. Tru computes focus while
foculus owns declared ordering/finality rules. Cyb is the named robot and its
bodies. Its GraphSession hosts multi-neuron graph state over cybergraph/BBG.
Lytics is a domain-scoped consumer with its own signed event and analytics
contract. Its historical deployment is traffic evidence; current replay and
identity compatibility are recorded in the [consumer audit](../audit/neuron-cell/consumers.md).

## the proof-contract ladder

Execution families expose different evidence. The family name alone cannot
select a proof contract; machine, proof profile, worker and network must match:

| runtime | compiler | what runs | proof contract |
|---|---|---|---|
| [[soft3/nox|nox]] | trident | .nox programs and supported jets | Declared trace/statement/verifier profile |
| [[glia]] | model loader | .model inference | Pinned model and host observation; current local profile |
| [[wysm]] | external | WASM programs | Declared metering/isolation and host evidence |
| wgpu | external | GPU compute shaders | Declared host result and bounds |

Soul retains policy and configuration. Progs and invocations carry executable
state and work. The current neuron engine reuses Rune and records correlated
host observations, reservations and continuations. That durable observation can
be an input to a later proof, but does not prove the remote effect or model
execution by itself. Lowering more work through Trident/nox is a separate profile
implementation, not an automatic consequence of installing a prog.

## completeness verdict

The architecture assigns owners from silicon to application. Its three
through-lines are design constraints whose concrete profiles require evidence:

- one form — compatible field and commitment primitives support composition; exact codecs, subject domains and native/foreign wire identities remain explicit.
- one proof — verified statements name their semantics and verifier; native receipts and trusted host observations keep their declared evidence tier.
- one focus — tru computes the attention field; network rewards/finality and local execution budget units each retain their own accounting and validation contract.

The convergence now has executable owners: neuron for subject/prog lifecycle,
cyb Host/Registry for binding and custody, Rune/worker for admitted work,
cybergraph/BBG for history, and Soma for tasks. Identity-only consumers remain
independent of the runtime. [Ward](../../cyb/specs/runtime-authority.md) and
[vault](../../cyb/specs/private-vault.md) have local host implementations;
standalone packaging is a separate question.

## the gaps

The earlier gap list has changed. Current scope and remaining work:

1. Local capability enforcement is implemented: admission/dispatch bind subject, network, revision, grant and worker generation. Publication holds the current authority guard through commit. Broader OS/device/remote adapters still need their own declared permission, fencing, receipt and isolation profiles; a device label alone proves no attestation.
2. eidos settles nothing yet. its zheng bridge is a stub (`certificate.rs`: `stark_bytes` empty, zheng and bbg crates unlinked). the CIC kernel checks terms, but a checked theorem does not yet become a settled, memoized cyberlink. the theorem-proof half of the proof layer is not closed.
3. The evidence ladder remains partial. Soma/glia observations are correlated and durable, with actual local model evidence. General proofs of inference or arbitrary host effects need their own witness/verifier integration. Trusted Nu and cooperative GPU execution do not imply hard preemption or remote sandboxing.
4. The shared runtime contract is implemented for the supported Rune/worker path: bounded admission, per-prog CAS, continuation, children/budgets, current-grant dispatch and unknown-outcome recovery. Additional compatible execution families must satisfy the [execution model](../specs/execution-model.md), rather than add another subject hierarchy or VM.
5. The original cyb-core `Cell` has become GraphSession; the standalone runtime has moved to neuron. The [convergence plan](neuron-cell-convergence.md) still owns complete workspace acceptance, legacy operator rehearsals and packaging/context evidence. Immutable original `cell/*/1` bytes remain reader provenance. The [Soma audit](../../soma/audit/neuron-composition.md) establishes the current local task profile; full provider/tool/channel parity is separately staged.

## the deep overlaps

these are places where two components touch the same noun. each is either a clean complement to keep, or a seam to clarify.

| overlap | reading | verdict |
|---|---|---|
| two nox on-ramps — trident (.tri / .mir) and wysm-trident (.wasm) | both emit trident LIR, the single waist | complement — keep; guard that LIR stays the sole lowering target |
| two prove verbs — zheng (execution) / eidos (theorem) | different claims over the same kernel: ran-correctly vs is-well-typed | complement — orthogonal, both belong on the proof layer |
| two render verbs — mir (3D world from the graph) / prysm (UI from a particle dialect) | different input and output: spatial graph viz vs symbolic chunk stream | complement — distinct, name the border so neither grows into the other |
| three terminal surfaces — rune (provable eval → nox) / neu (neural command-as-sentence) / nu (host OS shell) | provable vs semantic vs host; neu may displace nu over time | clarify — three languages, one prompt; document which is which |
| the field stack — strata (algebra API) / nebu (portable 𝔽_p, NTT) / acpu (accelerated) | one API, one portable impl, one silicon impl | clean once nebu is named; acpu is the fast path nebu dispatches to |
| local-state assembly — neuron execution / cyb-core / foculus / cybergraph | runtime history and multi-neuron graph hosting were conflated | neuron owns program execution; cybergraph/bbg own graph durability; foculus owns protocol order/finality |
| ward vs Rune capability values | language values refer to host authority; host retains current grants and custody | Current binding/grant checks at dispatch; an artifact or mutable `~caps` value cannot grant itself authority |

## closing the gaps

Extend the working ownership boundaries with evidence for each added profile:

- reuse neuron/Rune/worker admission and history when adding another tool or execution family; keep subject/network/budget and current authority explicit (gaps 1 and 4).
- close the witness ladder (gap 3) — specify what a host observation proves, and implement the additional witness/verifier work needed for stronger claims.
- link eidos to zheng (gap 2) — fill the certificate stub so a checked theorem settles as a memoized cyberlink.
- finish and audit all gates of the neuron/cell convergence plan (gap 5), with
  legacy preservation, release source provenance and generated-context checks.
  Record broader agent parity and remote network work under their own scopes.
