---
title: self — a private neuron that acts
tags: cyber, soft3, proposal, self, cy, neuron, research
crystal-type: process
crystal-domain: cyber
kind: proposal
status: draft
date: 2026-10-16
alias: private self, autonomous private neuron, network self
depends-on: soft3/proposals/privacy
evidence: open
---

# self — a private neuron that acts

## 0. decision

Make self a private persistent [[neuron]] whose fixed transition law gives it
authority to receive observations, maintain memory, decide and perform bounded
actions. Its identity and accepted history continue across replacement of its
physical executors. The network may instantiate such a neuron to act on its own
behalf; its authority is limited by the same declared law and budgets as any
other instance. No implicit authority over other neurons follows from that role.

Self depends on [privacy](privacy.md). Privacy supplies anonymous participation,
protected state, joint computation and distributed use of secrets. Self supplies
the meaning of that state, which events advance it, what it may decide, and which
effects it may authorize. This proposal owns the complete private agent process.
The privacy proposal owns the complete private protocol it consumes.

Select existing component homes. [[neuron]] owns subject/runtime records and
durable execution; [[cy]] owns the agent's declaration, decisions and permission
semantics. The network deployment composes them through existing execution and
privacy services. No additional repository is required for this proposal or
its initial implementation. §7 assigns the work without adding a second owner
of identities, state, cryptography or consensus.

This is a draft research and composition decision. Its exact private execution
profile, authority relation and failure protocol require implementation and
qualification before launch. The split creates independent questions and
acceptance receipts, with one combined gate for launching a private self.
It creates no post-genesis completion stage.

## 1. the boundary between the proposals

| concern | privacy owns | self owns |
|---|---|---|
| secrecy | leakage/adversary model and concrete PQ construction | which application observations, memory and decisions must be protected |
| state | hiding bindings, encrypted persistence, private access and recovery | subject/program identity, logical predecessor and accepted continuation |
| computation | authenticated inputs and verifiable bounded private evaluation | the transition law, trigger, decision procedure and permitted state changes |
| disclosure | enforcement of admitted release relations and safe threshold use | the instance's permitted disclosures and actions within those relations |
| continuity | succession of shares/keys and protected data under changing executors | continuing the same neuron, its tasks, allowances and unresolved effects |
| consensus | private admission/settlement and their finality dependencies | binding a decision and effect to an accepted state and finality certificate |
| failure | cryptographic/network fault bounds and failure evidence | suspension, reconciliation, retirement and resource exhaustion semantics |

The single service contract is [privacy §1.1](privacy.md#11-service-boundary-for-dependent-programs).
Self consumes it; changing self's model or objectives cannot change that
contract. Privacy's aggregate and monetary rules remain executable without
loading self or consulting its decisions. An application admission policy is
data to the protocol's fixed verifier, with no reverse dependency on cy.

The [execution model](../specs/execution-model.md) remains the composition rule:
machine, environment, proof, network and executor are separate selections.
Replacing a worker changes placement; it creates neither a new subject nor
permission to observe protected state. A network profile is a deployment choice.
Local cy retains its ability to start and work without a reachable network.
Generic runtime ports carry state, policy and evidence; they do not import cy's
objectives or model implementation. The network deployment supplies the adapters
that enforce the selected private profile.

## 2. identity, private state and fixed law

Use the existing neuron/program distinction: one subject may run several
programs with separate state and budgets. A program and a worker are identified
as data; their creation does not silently create another authority subject.
The current local authentication profile is classical. Its bytes must not be
reinterpreted as qualified private PQ authority; admission of this new profile
requires the explicit binding and migration rules from privacy.

Conceptual record and transition, pending exact encodings:

```text
manifest = (network_domain, neuron_binding, program,
            fixed_law, authority_policy, privacy_profile, resource_class)

protected_state = (memory, retained_observations, continuations,
                   capabilities, allowances, inbox, outbox, history)

(state_next, disclosures, proposed_acts)
    = step(fixed_law, protected_state, admitted_event)

accepted_transition = (manifest_binding, predecessor, event_binding,
                       successor, disclosure_binding, act_binding,
                       resource_charge, proof, admission_evidence)
```

Bindings use privacy's qualified commitments and exact canonical encoding.
Low-entropy memory needs hiding randomness. The statement authenticates every
field used for authorization and binds the exact prior state and input. The
accepted predecessor and revision rule prevent competing workers from adopting
two continuations of the same exclusive state. Generic concurrent evaluation
may still produce candidates which never become authoritative transitions.

Protected continuations, observations, prompts and history remain encrypted in
storage, recovery material and executor traffic. Existing local record/log
codecs need an explicit private profile before reuse. Recoverable application
memory and ephemeral key-share state have different retention rules: a portable
checkpoint must never resurrect shares that privacy requires a role to erase.

The subject remains stable internally. Publishing a permanent subject or
program handle intentionally makes its visible activity linkable. Private
actions use privacy's anonymous authority relation and scoped replay markers;
public service identity is an explicit disclosure. Stable handles, inbox
addresses, state-change frequency and action timing belong in the same leakage
analysis. A claim of private memory alone cannot imply anonymous behavior.

The fixed law contains the limits on capabilities, spending, code/model changes,
disclosure, recovery and retirement. It may permit changing model weights or
application code as state, but every successor remains constrained by that law.
The enclosing monetary and privacy invariants remain enforced independently.
Creation fixes who, if anyone, can request pause, delegation or recovery, and
what those operations can do. An unrestricted administrative extraction path
would become part of the declared authority and privacy model.

## 3. a complete life as an acting neuron

| transition | required behavior |
|---|---|
| create and fund | bind the law/profile, establish protected initial state and authority through qualified privacy services, reserve execution/storage resources and declare recovery/retirement rules |
| receive | admit authenticated encrypted messages or committed public observations under bounded queues, eligibility and anti-replay rules |
| wake | accept a message, recorded outcome or authenticated network-time event according to the standing rule; any eligible executor may service it |
| decide | run the bounded private transition with retained code/model/input context; verify resource use and all proposed acts |
| commit | atomically accept the successor, consumed event, resource charge and outbox obligations under the network's finality rule |
| act and reconcile | release only the authorized effect, retain evidence of dispatch/outcome, and resume its continuation using that correlated observation |
| change executors or recover | rebind work to the accepted state and generation, preserve pending obligations and use privacy's secret/data succession protocol |
| pause, exhaust or retire | apply the fixed permission and resource rules; preserve outstanding effects, asset obligations and the authorized recovery closure |

Autonomy means progress under the admitted rule while the initiating person
and original executor are offline. An external caller need not approve every
step. The schedule, funding and fault assumptions must make that progress
possible. The creator's continuing consent is required only where the creation
law explicitly requires it.

Clock, randomness, model choices and observations that affect state enter
through authenticated or explicitly classified inputs. Replaying a continuation
uses the retained inputs. A worker's local clock or a fresh inference cannot
replace an earlier admitted observation. Information from outside the network
retains its evidence/trust classification; a proof of processing does not
establish the truth of that information.

Resource accounting includes computation, ciphertext/proof traffic, private
reads, storage retention, scheduled wakeups and effects. Reserve the amount
needed for the declared operation and recovery rule before accepting it.
An unfunded neuron follows its fixed exhaustion policy. Indefinite operation
requires continuing resources under a specified funding mechanism.

## 4. permission, effects and external reality

[[cy/specs/soul|Soul]] declares the agent policy and
[[cy/specs/ward|ward]] is its permission boundary. For this network profile,
the permission check must be in the accepted transition relation and enforced
at the effect boundary. A local prompt or an honest worker's promise is
insufficient. The mind proposes acts; the fixed law admits their exact scope.
Delegation attenuates capabilities and binds expiry, revision and operation.

Native acts use the qualified proof of authority for the exact action and
network. The kernel requires no universal signing master key. Signature-based
external protocols need their own reviewed adapter and cryptographic profile;
a classical destination cannot be counted as an end-to-end PQ integration.
Separate secret domains limit authority, and independently generated keys are
required wherever master-key compromise must not cross those domains.
An owner funds or delegates to this neuron through an explicit authorized
transition. Other private notes keep their owners' spending authority; running
the service supplies no implicit access to them.

An outbox entry commits the destination, payload binding, operation identity,
budget and authorization context before dispatch. Dispatch authority is derived
only from the accepted transition and sufficient finality. A copied proof or
historical worker descriptor cannot create a fresh permit. Privacy releases
only the approved effect or scoped key operation, with no raw master-key path.

For native state changes, atomic acceptance and nullifiers enforce the selected
once-only relation. For an external service, a crash after dispatch may leave
an unknown outcome. Require an idempotency/status/receipt protocol adequate to
reconcile that destination; retrying blindly can repeat the effect. Delivery,
destination acceptance and completion are separate facts. An advertised effect
whose required failure/recovery behavior cannot be implemented remains blocked
before freeze; an indefinite unknown state cannot satisfy its completion claim.

Neuron-to-neuron calls use authenticated asynchronous messages and correlation
identities, with bounded outstanding requests and explicit timeout/refund rules.
They preserve capabilities across the call and do not assume a distributed
atomic transaction with another neuron or an external service.

## 5. memory, learning and the limits of self

The operational kernel can run a deterministic decision program. An intelligent
instance additionally needs an objective, a model, observations and a qualified
learning/inference process. [[cy]], [[glia]] and [[seer]] already own those
concerns. They use the same authority boundary as simpler programs.

Protected inference/training must execute inside the accepted privacy boundary.
Giving plaintext memory to an ordinary remote model crosses it. Opaque model
output may enter as an explicitly untrusted observation and still be checked
against action constraints; that does not establish confidential or correct
model execution. Every promised private model workload needs its actual bounded
circuit, numeric semantics and measured cost before launch.

Memory and permitted model parameters can evolve under a fixed law. Learning
cannot mint authority, alter monetary conservation or loosen the disclosure
relation. Outputs, selected actions and timing can expose private information;
the release analysis covers their sequence. Arbitrary evaluation capability
does not authorize arbitrary plaintext answers about protected state.

The cryptographic limits remain those of privacy: bounded corruption, erasure,
availability and a qualified security profile. Already disclosed information
can be copied, and compromised threshold custody cannot be repaired by merely
renaming the neuron. Classical state has no physical unclonability guarantee.
Accepted identity/history depends on finality and domain binding. Intelligence,
truthful inputs, useful learning and resistance to poisoning require separate
evidence from correct execution.

The [ICP comparison](../audit/self-2026-10-16/icp-comparison.md) records a useful
deployed reference for distributed authority and key delivery, while separating
those capabilities from general private state execution. It supplies neither
a qualified PQ implementation nor a performance claim for this design.

## 6. conformance to privacy and joint freeze

| self obligation | privacy contract/gate consumed | composition evidence required |
|---|---|---|
| private subject and action authority | §1.1 context binding; G1–G4 | wrong subject/domain/program/policy/predecessor rejected; linkability matches the declared profile |
| persistent private memory and execution | protected state/evaluation; G2, G4, G5 | actual protected transition, correct successor, private reads and qualified proof/evaluation composition |
| autonomous action and disclosure | authorized release; G1, G5 | only an admitted final transition can authorize the exact output/effect; no individual-input decryption oracle |
| continuity through replacement | secret continuity/failure evidence; G5a | replacement preserves state, pending effects, key domains and authorized history without reconstructing a full service secret |
| private messages and recovery | private delivery/protected state; G6 | declared identity/traffic leakage, complete encrypted history and access-private recovery |
| sustainable operation | resource classes; G7 | complete lifecycle receipt including computation, storage, succession, transport and recovery costs |

Own closing gates:

| gate | closing evidence |
|---|---|
| S1: subject and law | exact profile/record definitions, authority and disclosure semantics, fixed invariants and accepted migration rules |
| S2: autonomous private transition | creator offline; authenticated input/timer wakes bounded computation; accepted private state and permitted action agree with the reference relation |
| S3: effect confinement | forged/expanded grants, wrong domains, stale state and unfinalized output cannot dispatch; native duplicates and external uncertain outcomes follow their stated protocols |
| S4: complete lifecycle | creation, funding, concurrent candidates, worker loss, succession, partition recovery, exhaustion, pause and retirement preserve history, allowances and obligations |
| S5: learning boundary | any claimed private model workload stays within its declared boundary and costs; adversarial proposals cannot widen authority or disclosure |
| S6: end-to-end closure | privacy gates and S1–S5 pass on exact committed implementations; security/composition review, adversarial fixtures and measured product budgets cover the advertised behavior |

Use a small autonomous private neuron as the first integration fixture: it keeps
a secret preference and finite allowance, consumes an authorized collective
aggregate, decides whether to emit a cyberlink or make a bounded native payment,
records the outcome and continues after executor replacement. The creator stays
offline. Inject duplicate messages, stale results, forbidden disclosures,
exhaustion and crashes around commit/dispatch. This is a proposed experiment;
no result or benchmark is claimed here.

That fixture establishes a test path, with the complete declared workload set
still required for acceptance. Private aggregate/reward computation remains in
privacy's own fixtures. Each proposal has its own evidence; the requested
private self launches only when both and their composition are qualified.
Unresolved rows stay open before genesis.

## 7. implementation homes and repository decision

| owner | work assigned by this proposal |
|---|---|
| soft3 | cross-component contract, privacy dependency, profile/conformance matrix and joint acceptance receipts |
| neuron | existing subject/program model, private authority bindings, durable invocation/continuation/effect records and network execution profile |
| cy | soul/ward decision semantics, objectives, observation handling, standing orders and agent-level lifecycle composition |
| mudra | privacy-owned key distribution, encrypted computation/delivery and scoped secret operations consumed by self |
| cybergraph, bbg | shared authenticated history, encrypted records and atomic state publication; no second canonical agent database |
| foculus | network admission/finality and canonical continuation; consume the privacy protocol without depending on the agent's objectives |
| nox, trident, joy, zheng, lens | existing execution/compiler/proof contracts and qualified protected backend integration |
| radio, tade | existing transport/framing homes with privacy's metadata and delivery profile |
| glia, seer and the owning model kernels | model execution/proposals and their separately qualified protected workloads |
| vault | local credential custody and its adapters; distributed service secrets use the privacy protocol's distinct custody profile |

No new repository now. The inspected neuron workspace already has identity,
model, engine and node/CLI composition packages; cy has the agent contracts.
Extend those boundaries with a reviewed network/private profile. Module/crate
extraction inside an owner is justified by an actual independent interface and
dependency need; a separate repository would need an owner, independent release
boundary and a demonstrated consumer that the existing homes cannot serve.
This proposal creates none of those requirements by naming self.

The source snapshots and the distinction between existing local behavior and
proposed network guarantees are recorded in the
[research receipt](../audit/self-2026-10-16/README.md). Existing accepted specs,
component registry, release pins and implementation claims are unchanged.
