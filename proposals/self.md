---
title: self — the private state machine that acts
tags: cyber, soft3, proposal, self, cy, neuron, research
crystal-type: process
crystal-domain: cyber
kind: proposal
status: draft
date: 2026-10-16
alias: private self, autonomous private neuron, network self, self — a private neuron that acts
depends-on: soft3/proposals/privacy-frozen
evidence: open
---

# self — the private state machine that acts

## 0. decision

The first reason to build self is to make the complete private protocol work.
Privacy over contributions held by many neurons needs a persistent private
state machine that holds distributed secrets, advances their lifecycle and
uses or reveals them only under fixed rules. That machine is the core of self.
The network's first self instance runs its own privacy service: admitting
protected contributions, computing aggregates, authorizing disclosure and
preserving those obligations across replacement of executors.

Self and [privacy](privacy-frozen.md#01-mutual-dependency-with-self) are mutually
dependent. Privacy supplies the cryptographic protection and proof mechanisms
that make self's state private. Self supplies the persistent rules and accepted
history that make the network's use of those mechanisms a complete private
protocol. The two proposals specify different responsibilities in one composed
machine; both and their joint behavior must be qualified before that machine
is frozen and launched.

The same core supports a private persistent [[neuron]] that receives
observations, maintains memory, decides and performs bounded actions. A fixed
transition program is enough for the privacy service. Models, learning and
objectives extend its applications. An instance's identity and accepted history
continue across replacement of its physical executors, and its authority stays
within its declared law and budgets.

Select existing component homes. [[neuron]] owns subject/runtime records and
durable execution; [[cy]] owns the agent's declaration, decisions and permission
semantics. The network deployment composes them through existing execution and
privacy services. No additional repository is required for this proposal or
its initial implementation. §7 assigns the work without adding a second owner
of identities, state, cryptography or consensus.

This is a draft research and composition decision. Its exact private execution
profile, authority relation and failure protocol require implementation and
qualification. The privacy service and self's secret-state kernel share the
closing gate S0/G6; an autonomous agent instance also closes S1–S6. The root
chain's separate activation schedule remains recorded in privacy §1.13. The
complete private protocol includes the joint-computation machine described here.

## 1. why privacy needs self, and self needs privacy

The proposed aggregate service must remember the accepted input set, function,
key generation, disclosure history and remaining release budget. It must carry
that state through failures and changes of executors. Otherwise a committee can
be asked to decrypt an individual input, repeat an exhausted query or release
conflicting results. Encryption and a proof of arithmetic supply only part of
this functionality; the accepted state transition supplies its operational law.

| direction | required capability |
|---|---|
| privacy → self core | a persistent private state machine for input admission, secret lifecycle, authorized evaluation/release, replay protection, recovery and failure handling |
| self core → privacy | hiding commitments, protected state and access, authenticated private inputs, qualified PQ joint computation, proofs and distributed secret operations |
| agent applications → composed core | private memory, standing orders, bounded decisions and effects over the same accepted state and authority rules |

Privacy's aggregate machine therefore instantiates self's core. The monetary
conservation relation stays independently enforced. Owner-held private spends
use the pool's own proof path; the requested complete machine additionally
includes private computation over many independent neurons' contributions.
Its completeness gate includes the self core. Model-driven decisions in [[cy]]
consume that core through a separate application interface.

### 1.1 the first self instance is the privacy service

Its conceptual state, with exact encodings still to be specified, is:

```text
privacy_service_state = (domain, fixed_law, privacy_profile,
                         accepted_predecessor, key_generation,
                         role_assignment, protected_state_binding,
                         closed_input_set, approved_function,
                         release_history, allowances, pending_obligations)

step(state, admitted_event, cryptographic_evidence)
    -> (next_state, permitted_secret_operations, authorized_disclosures)
```

Public commitments authenticate the history; secret shares and protected
records are distributed according to the privacy profile. Each field's public
or hidden representation belongs in the joint leakage model. The state machine
admits contributions, closes an input set, evaluates the approved function,
authorizes the exact release after finality, and records that release before
another request can consume its allowance. Resharing, erasure, executor
replacement, failure and retirement are transitions under the same fixed law.
Each secret domain has its own scope and compromise analysis.

An honest threshold releases only on evidence binding the domain, predecessor,
key generation, input set, function and final decision. The proof establishes
correctness; the threshold protocol and its corruption assumptions establish
confidentiality. A colluding reconstruction threshold can disclose secrets
outside the law. That failure remains explicit in the threat model, together
with mobile corruption, erasure assumptions and availability. The machine's
rules do not turn threshold custody into an unconditional secrecy guarantee.

[Can a Public Blockchain Keep a Secret?](https://eprint.iacr.org/2020/464)
studies long-term secret custody with specified use/release conditions and
resharing across changing hidden committees under a mobile-adversary model.
[YOSO](https://eprint.iacr.org/2021/210) separates protocol roles from their
physical assignment. These motivate the separation of a continuing logical
service from replaceable executors here; a qualified PQ construction and its
composition with this stack remain research obligations.

### 1.2 ownership and bootstrap

| concern | privacy owns | self owns |
|---|---|---|
| secrecy | leakage/adversary model and concrete PQ construction | the state, events and allowed disclosures of the service and its applications |
| state | hiding bindings, encrypted persistence and private access mechanisms | canonical predecessor, accepted continuation, release history and allowances |
| computation | authenticated inputs and verifiable bounded private evaluation | the fixed transition law, admitted input set and permitted secret operations |
| disclosure | cryptographic enforcement and safe threshold use | authorization against accepted state, finality, prior releases and remaining budget |
| continuity | DKG, resharing, erasure and protected recovery mechanisms | when generations change, which history continues and how pending obligations survive |
| consensus | privacy of admission/settlement and the joint security argument | state/effect binding to the admitted finality certificate |
| failure | cryptographic/network fault bounds and failure evidence | suspension, recovery, exhaustion and retirement transitions |

The reciprocal dependency is a composition contract. Implementation starts
from explicit bootstrap inputs: public parameters, an admitted initial state,
a qualified role-assignment/authentication profile, recipient channels and the
selected key-generation protocol. These establish the first protected state.
That state then advances using the cryptographic mechanisms. Agent objectives
enter through the higher application interface. No step may require a secret
or aggregate that the same unfinished step is meant to create.

The shared interface binds network, subject, program, policy, predecessor,
key generation and input set. It covers protected reads, bounded evaluation,
finalized release, secret succession, recovery, failure evidence and resource
charges. Consensus admission uses the declared bootstrap or prior accepted
state; any private feedback into later consensus has a separately specified
ordering. S0/G6 must close this schedule and failure behavior before launch.
The two proposals share this interface and its conformance fixtures; component
modules implement it through the existing ownership boundaries.

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

The [ICP comparison](https://github.com/cyberia-to/soft3/blob/764d90f71e4fd800532548d76ec61829a791d970/audit/self-2026-10-16/icp-comparison.md) records a useful
deployed reference for distributed authority and key delivery, while separating
those capabilities from general private state execution. It supplies neither
a qualified PQ implementation nor a performance claim for this design.

## 6. conformance to privacy and joint freeze

| shared obligation | privacy contract/gate | composition evidence required |
|---|---|---|
| privacy service implemented by self core | §1.7 joint secrets, G4–G6 ↔ S0 | complete admitted-input → evaluation → authorized-release lifecycle, bootstrap and succession under one fixed law |
| private subject and action authority | §2 authority relation, §6.4 predicates; G3–G5 | wrong subject/domain/program/policy/predecessor rejected; linkability matches the declared profile |
| persistent private memory and execution | §1.7 joint evaluation; G1, G2, G4–G6 | actual protected transition, correct successor, private reads and qualified proof/evaluation composition |
| autonomous action and disclosure | §1.7 authorized release; G4, G6 | only an admitted final transition can authorize the exact output/effect; no individual-input decryption oracle |
| continuity through replacement | G6 key generation, resharing, erasure and failure | replacement preserves state, pending effects, key domains and authorized history without reconstructing a full service secret |
| private messages and recovery | §1.2 delivery, §6.6 recovery; G7 | declared identity/traffic leakage, complete encrypted history and access-private recovery |
| sustainable operation | §6.2 measured budgets, G6 collective costs | complete lifecycle receipt including computation, storage, succession, transport and recovery costs |

The shared core gate and the additional agent gates:

| gate | closing evidence |
|---|---|
| S0: privacy service core | joint with G6: exact state and fixed law, bootstrap, binding of inputs/functions/results, stateful release accounting, finality, key succession/erasure, recovery and resource/failure outcomes; privacy's G4/G5 qualify the leakage and cryptographic composition; required for the complete private protocol |
| S1: subject and law | exact profile/record definitions, authority and disclosure semantics, fixed invariants and accepted migration rules |
| S2: autonomous private transition | creator offline; authenticated input/timer wakes bounded computation; accepted private state and permitted action agree with the reference relation |
| S3: effect confinement | forged/expanded grants, wrong domains, stale state and unfinalized output cannot dispatch; native duplicates and external uncertain outcomes follow their stated protocols |
| S4: complete lifecycle | creation, funding, concurrent candidates, worker loss, succession, partition recovery, exhaustion, pause and retirement preserve history, allowances and obligations |
| S5: learning boundary | any claimed private model workload stays within its declared boundary and costs; adversarial proposals cannot widen authority or disclosure |
| S6: end-to-end closure | privacy gates and S0–S5 pass on exact committed implementations; security/composition review, adversarial fixtures and measured product budgets cover the advertised behavior |

The first integration fixture is the privacy service itself: independent
neurons contribute encrypted inputs; a fixed aggregate is evaluated over the
closed set; only its admitted result is released. Continue through executor
replacement while retaining release history and pending obligations. Reject
individual-input decryption, wrong functions or key generations, stale and
unfinalized results, repeated/exhausted releases and rollback after disclosure.
Exercise missing roles, interrupted resharing and bootstrap/consensus stalls.
The G4 analysis states what even an authorized exact aggregate reveals.
This single fixture belongs to both S0 and G6; all advertised functions still
need their own leakage, correctness, failure and cost evidence.

The next fixture exercises an autonomous private neuron: it keeps
a secret preference and finite allowance, consumes an authorized collective
aggregate, decides whether to emit a cyberlink or make a bounded native payment,
records the outcome and continues after executor replacement. The creator stays
offline. Inject duplicate messages, stale results, forbidden disclosures,
exhaustion and crashes around commit/dispatch. This is a proposed experiment;
no result or benchmark is claimed here.

These fixtures establish a test path, with the complete declared workload set
still required for acceptance. Privacy and self retain separate responsibilities
and share the S0/G6 receipt. The complete private protocol closes that joint
receipt before its immutable launch; an autonomous agent launch additionally
closes S1–S6. Neither proposal can mark the shared private machine complete
while the other's core obligations remain open.

## 7. implementation homes and repository decision

| owner | work assigned by this proposal |
|---|---|
| soft3 | reciprocal privacy/self contract, profile/conformance matrix and shared S0/G6 acceptance receipt |
| neuron | secret-state kernel and privacy-service lifecycle, existing subject/program model, durable invocation/continuation/effect records and network execution profile |
| cy | soul/ward decision semantics, objectives, observation handling, standing orders and agent-level lifecycle composition |
| mudra | privacy cryptography, key distribution, encrypted computation/delivery and secret operations invoked by the self core under its accepted law |
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
[research receipt](https://github.com/cyberia-to/soft3/blob/764d90f71e4fd800532548d76ec61829a791d970/audit/self-2026-10-16/README.md).
That receipt describes the original split on PR #190. This restored draft
points to the reconciled `privacy-frozen.md` and its current gate numbering;
the older receipt's one-way dependency, privacy links and genesis wording remain
historical. This draft makes the shared secret-state kernel explicit in both
proposals and assigns its acceptance to S0/G6.
Existing accepted specs,
component registry, release pins and implementation claims are unchanged.
