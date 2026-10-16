---
title: privacy — can the network keep its secret
tags: soft3, audit, privacy, research, mudra, foculus
crystal-type: report
crystal-domain: cyber
date: 2026-10-16
status: findings
evidence: open
---

# privacy — can the network keep its secret

Companion to [privacy](../../proposals/privacy.md), especially §6.4–6.6.
The owner requires the entire working protocol to be complete before freeze.
This literature review identifies a concrete secret-custody architecture and
its composition obligations. It supplies no implemented protocol or benchmark.

## finding

The likely intended paper is [Can a Public Blockchain Keep a Secret?](https://eprint.iacr.org/2020/464),
Benhamouda, Gentry, Gorbunov, Halevi, Krawczyk, Lin, Rabin and Reyzin, TCC 2020.
Its ECPSS construction sends refreshed shares to hidden successor committees.
It addresses targeted compromise of small committees through concealed
membership and erasure before public action. Its applications explicitly
include FHE-key custody and consensus-controlled result decryption.

The useful conclusion is architectural: secret custody can be an ongoing
network protocol. Treating a visible static committee as the only possibility
misses the paper's contribution. Security still depends on its corruption,
communication and erasure assumptions. The selected research direction is a
complete PQ implementation of this lifecycle composed with collective execution
and consensus, with all components present at genesis.

## lineage and its different contributions

These are related construction lines, not a claim that every row is a direct
descendant or that their implementations can be composed without new analysis.
The linked primary sources were opened on the receipt date. Exact revision
dates and inspection depth are in [lineage-sources.json](lineage-sources.json).

| source | contribution relevant here | qualification boundary |
|---|---|---|
| [CHURP, 2019](https://eprint.iacr.org/2019/017) | proactive resharing with changing committees | committee honesty must be justified separately; original efficient polynomial commitments do not furnish our PQ profile |
| [Can a Public Blockchain Keep a Secret?, 2020](https://eprint.iacr.org/2020/464) | selection and state handoff while successor holders remain hidden | synchronous authenticated broadcast, mobile-corruption and erasure model; not a qualified implementation for this stack |
| [YOSO, 2021](https://eprint.iacr.org/2021/210) | general secure computation with one-message roles, including guaranteed output delivery in its synchronous honest-majority model | role assignment is abstracted out; a secure protocol for roles is not yet a secure deployment on physical machines |
| [YOLO YOSO, 2022](https://eprint.iacr.org/2022/242) | efficient encrypted delivery and verifiable resharing to anonymous random recipients | useful interface and cost reference; exact instantiated assumptions must be checked |
| [Encryption to the Future, 2022](https://eprint.iacr.org/2021/1423) | makes delivery to a future unknown committee a separate primitive | near-future handoff differs from unrestricted far-future encryption; auxiliary committees remain part of a construction |
| [Constant-Round YOSO MPC Without Setup](https://eprint.iacr.org/2022/187) | setup generation and verifiable state propagation can themselves be executed by ephemeral roles | bootstrap must instantiate its assumed recipient channels and proof system; the title is not a certificate for our genesis |
| [Taming Adaptivity, 2023](https://eprint.iacr.org/2023/1077) | composes role protocols with assignment to physical machines under adaptive corruption | use its end-to-end modeling discipline; audit the exact PQ substitutions and composition |
| [Packed YOSO MPC, 2025](https://eprint.iacr.org/2025/635) | separates preprocessing from online cost and treats fail-stop failures explicitly | amortized communication improvements depend on circuit structure, preprocessing and the role model; no imported soft3 latency claim |
| [Asynchronous YOSO à la Paillier, 2025](https://eprint.iacr.org/2025/128) | combines MPC, role assignment and total-order broadcast in one adaptively secure asynchronous construction | Paillier is outside the PQ baseline; asynchronous adaptive security does not by itself establish perpetual mobile-corruption security |
| [Practical Committee Sizes, 2026](https://eprint.iacr.org/2026/919) | fixed-size selection, tighter analysis and accountable nonparticipation | claimed improvements also use time-lock puzzles and penalties; cryptoeconomic assumptions must not replace confidentiality assumptions silently |

The research should therefore implement a whole role lifecycle, including
assignment and communication. Appending a refresh timer to an ordinary
interactive threshold server is insufficient for the selected adversary.

## the PQ branch is real, but the composition remains open

[Practical PVSS with Thousands of Parties](https://eprint.iacr.org/2021/1397)
combines LWE encryption with discrete-log Bulletproofs. Its encryption primitive
alone cannot make the complete verifiable-sharing protocol post-quantum. Its
published implementation is a useful classical comparison, not the selected
frozen security profile.

[A Generic Framework for Practical Lattice-Based Non-interactive PVSS](https://eprint.iacr.org/2025/901)
proposes fully lattice-based verification, tailored encryption and proofs of
smallness. It is a concrete candidate for the PQ sharing work package. This
review inspected its primary abstract and revision metadata; its complete
adaptive-security argument and implementation were not audited.

[PVSS: Generic Constructions and Lattice-Based Instantiations in the Standard Model](https://arxiv.org/html/2504.14381v1)
provides another LWE-based construction. Its concrete syntax includes trusted
parameter setup, and its introduction explicitly discusses an efficiency
tradeoff. It is useful evidence of PQ feasibility, not a transparent setup
certificate or a ready YOSO instantiation. Adaptive soundness of a NIZK is a
different statement from adaptive corruption security of the whole service.

The proposed PQ composition needs all of the following simultaneously:

- anonymous recipient encryption secure for the actual selective-opening game;
- verifiable resharing and decryption, including malicious key/input generation;
- hidden and resource-weighted role assignment with an unbiased randomness rule;
- proof of correct state propagation and any simulated corruptions in the
  end-to-end argument;
- exact lattice key/noise distributions compatible with the selected HE scheme;
- PQ authentication, transcripts and proof-system parameters throughout.

The references do not establish that replacing each classical primitive with
a similarly named lattice primitive preserves the protocol theorem.

## erasure and network timing are explicit assumptions

[Proactive Secret Sharing without Erasures](https://eprint.iacr.org/2026/1072),
revised before this review, obtains its result using quantum computation,
one-shot signatures and post-quantum extractable witness encryption. It is
not software for ordinary classical hosts. It does not remove the erasure
obligation from the selected practical baseline.

The newer [Asynchronous PSS possibility/impossibility study](https://eprint.iacr.org/2026/2018)
distinguishes mobile-corruption models when there is no global clock and proves
model-dependent limits and matching protocols. Only its abstract was inspected
here; no numerical resilience bound from it is asserted. It identifies the
precise issue a partial-synchrony integration must resolve.

For our proposed service, the following are engineering/security implications,
not additional theorems claimed from either paper:

- refresh epochs must account for real exposure during stalled agreement;
  stopping block production does not stop an attacker;
- honest role state must disappear from RAM, swap, crash dumps, snapshots and
  backups according to the accepted host model;
- ordinary wallet restoration must never restore retired service shares;
- crash recovery after erasure may resend a prepared public message, but must
  not reconstruct and rerun the one-time secret role;
- if a role crashes before publishing, redundancy and handoff rules must
  preserve progress within the specified fault budget;
- an arbitrary unbounded partition plus unrestricted continuing compromise is
  not the same promise as recovery within a bounded corruption model.

A complete protocol implements its declared fault model before launch. It also
states where that model ends. Permanent halt is not an adequate substitute for
required liveness under faults the protocol claims to tolerate.

## binding this to the actual harness

Local source commands:

```text
git -C /Users/master/cyber/foculus show 9e32aa688e0f03ffa8fd9a3c099bc7e50cc8c744:specs/protocol.md
git -C /Users/master/cyber/mudra show 8b19098fd394c459ba01dd98cb6081d469dc27c7:specs/quorum.md
```

Both inspected files match these local revisions; they are not identified as
origin release inputs. Checksums are in [sources.json](sources.json).

The foculus source specifies partial synchrony and domain-local finality,
without the synchronized total-order board assumed by the original ECPSS
construction. Its focus-weighted conflict and finality rules are a concrete
integration obligation. If determining the canonical batch needs a new private
aggregate while revealing that aggregate needs that same batch to be finalized,
the combined design is circular.

The proposal selects the previous finalized public state as the control context
for the next private computation. Eligibility, input closure and disclosure
authorization bind to that context; aggregate/payout finalization then advances
it atomically. This is an architectural decision requiring a new combined
consensus proof and implementation. A DAG/local-finality implementation must
realize the necessary canonical certificates explicitly.

The mudra quorum source describes Shamir, DKG and threshold-decryption
interfaces. Its hash-of-coefficients sketch supplies no complete verifiable
sharing relation, hidden-role assignment or proactive lifecycle. It cannot be
treated as an already implemented ECPSS service. The existing zheng proof work
may help prove input and handoff relations; the complete statements, proof
costs and PQ composition still have to be established.

## complete freeze evidence to produce

The existing proposal's gates are extended to cover this whole lifecycle:

| required artifact | what it must settle before genesis |
|---|---|
| single functionality and adversary model | permitted outputs, adaptive/mobile corruptions, role versus network thresholds, corruption windows, timing, erasure, lifetime failure budget |
| executable initialization | parameter generation, first hidden roles, distributed service-key generation, first randomness and no circular dependency |
| executable normal cycle | private inputs, accepted batch, encrypted aggregate, nonlinear reward circuit, verifiable authorized release, private payouts, successor handoff |
| proof of composition | role assignment, selective opening, PQ proofs/encryption, custody, computation and consensus in the same model |
| adversarial recovery fixtures | hidden-recipient exposure, malformed shares, targeted withholding, crashes around erasure/publication, reboot, reorg, overlapping release attempts and partitions |
| end-to-end cost receipt | actual node/device time, peak memory, ciphertext and proof bytes, all handoff/preprocessing traffic, latency tails, recovery and rejection costs |

The research verdict is specific: pursue PQ ECPSS/YOSO-style secret custody
with threshold encrypted evaluation and policy-bound release. The literature
supports this direction and supplies components; it does not yet close the
entire soft3 protocol. All remaining work is pre-genesis work. No reduced
protocol is designated as the launch version.
