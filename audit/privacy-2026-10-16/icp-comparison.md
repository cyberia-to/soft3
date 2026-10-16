---
title: network secrets, autonomous state and ICP
tags: soft3, audit, privacy, research, icp
crystal-type: report
crystal-domain: cyber
date: 2026-10-16
status: findings
---

# network secrets, autonomous state and ICP

Companion to [privacy §6.7](../../proposals/privacy.md#67-persistent-private-state-and-autonomous-authority).
This is a primary-source comparison and architectural analysis, performed on
soft3 `ee3bd0ddab2f1f7682412192e89937758f6f515c` plus this documentation change.
No ICP node was run, no deployed canister was audited, and no performance or
cryptographic security experiment was conducted.

## what is deployed

| capability | source-backed finding | relevance to the proposed machine |
|---|---|---|
| network-held signing authority | chain-key protocols distribute a signing key across a subnet and support threshold signatures without reconstructing it at an individual executor [1] | a program can control external assets without an individual operator holding its full signing key |
| continuity across changing holders | Groth's NI-DKG/resharing paper describes public-key-preserving resharing, forward-secure encryption and proactive security, with deployment for ICP's BLS keys [2] | ICP already addresses key succession; describing it as only a static multisignature would be inaccurate |
| policy-controlled key derivation | vetKD is documented as live on mainnet; a canister authorizes derivation and delivery encrypted to a recipient's transport key [3] | the recipient obtains the raw derived key locally; this operation has a different disclosure boundary from computing while keeping all inputs secret |
| encrypted storage and conditional access | vetKeys supports encrypted application data and policy-controlled access to its keys [3] | useful for private files and recovery, with security depending on authentication and the release policy |
| ordinary canister computation | ICP states that operators of standard application subnets can read canister memory; its SEV-SNP mitigation is described as being rolled out [4] | the documented key service does not establish arbitrary confidential execution over canister plaintext; hardware protection adds a distinct trust assumption |
| application immutability | controller removal/blackholing can make a canister immutable; the documentation retains an exceptional NNS uninstall path [5] | application immutability exists, while the underlying protocol remains governed and upgradeable [6] |

The paper [vetKeys: How a Blockchain Can Keep Many Secrets][7] introduces
verifiably encrypted threshold key derivation and derives applications from a
shared root secret. Its abstract was inspected, not its full proof or deployed
implementation. This is a closely relevant parallel line of work to
[ECPSS/YOSO](network-secret-lineage.md); this review establishes no historical
derivation of one construction from the other.

## boundaries that matter for soft3

The cited chain-key signature families use BLS, ECDSA and Schnorr [1], and the
documented vetKD curve is BLS12-381 [3]. These discrete-logarithm-based choices
do not meet the proposal's post-quantum requirement. This is a classification
of the cited constructions, not a claim about every current or future ICP
cryptographic component. NI-DKG's pairing-based construction [2] likewise
cannot be imported as a PQ implementation.

Subnet membership and creation are governed through registered nodes and NNS
proposals [6]. That deployment model does not supply the hidden ephemeral
role-assignment guarantee selected in §6.4. Proactive refresh and anonymity of
the next physical executor address different attacks; neither replaces the
other's assumptions.

Controller authority and network upgrade authority also differ. An application
controller with unrestricted code upgrades can change a release policy. Removing
that controller constrains the application; NNS-governed replica upgrades [6]
still differ from freezing the complete underlying protocol at genesis.

The architectural conclusion is that ICP demonstrates deployed distributed
authority and conditional secret delivery. Combining these with general private
state transitions, the selected adaptive adversary model, post-quantum security
and permanent protocol rules remains a separate qualification task for soft3.
No impossibility of implementing additional cryptography on ICP is asserted.

## the operational meaning of self

For this proposal, an autonomous service needs a durable identity, an authorized
state-transition rule, continuity of its state, and the capacity to act under
that rule. A network secret can give it authority and private memory that no
individual honest executor possesses in full. The service's goals, learning
algorithm and inputs determine its behavior; possession of a secret does not
establish intelligence or the truth of an observation. Public-state programs
can also act autonomously.

The relevant distinction is between holding a secret, applying a permitted
operation to it, and disclosing a permitted result. For example, a network may
sign an authorized payment without releasing its key. A private auction may
publish the winning price while concealing other bids. A private learning
process may update protected state while exposing selected predictions. Each
case needs a complete functionality and its own leakage argument; a common
key-custody interface alone proves none of them.

Some boundaries follow directly from the requested functionality:

- Publishing an arbitrary function of secret state includes publishing the
  identity function. Private computation therefore needs an explicit permitted
  release relation even when the evaluator supports arbitrary bounded programs.
- A released fact or key can be copied by its recipient. Revoking future access
  cannot make the recipient forget the earlier disclosure.
- Enough cooperating holders can copy classical shares or reconstruct a secret.
  Stable cryptographic identity is therefore distinct from physical
  unclonability. Consensus determines which state history is accepted under
  its fault assumptions; external verifiers must enforce the relevant domain.
- Persistent state can learn while its transition law stays fixed. Giving the
  learner authority to rewrite that law also gives it any extraction powers
  expressible by the replacement law. Monetary and disclosure constraints need
  to remain outside that authority.

The remaining costs include malicious-secure computation, private memory
access, share succession, availability, cover traffic and output-leakage
control. Holding a secret does not make those operations free. This review
supplies no measurements or claim that protected training fits the harness.

## sources and inspection scope

All URLs below were opened with `web.run` on 2026-10-16. For reproduction, use
`open: [{"ref_id": "<URL>"}]` for the linked URL. These are live documentation
views, not pinned source revisions; publication metadata is recorded for the
two papers. No document's full security proof was verified.

| id | primary source | inspected scope |
|---|---|---|
| 1 | [Chain-key cryptography][1] | threshold signatures, key derivation and membership-change sections |
| 2 | [Non-interactive distributed key generation and key resharing][2] | abstract and metadata; received 2021-03-17 |
| 3 | [VetKeys][3] | protocol flow, API/curve, applications and mainnet status |
| 4 | [Security model][4] | execution, memory confidentiality and controller boundaries |
| 5 | [Trust in canisters][5] | controller removal, blackholing and self-upgrade caveat |
| 6 | [Evolution & scaling][6] | recovery, subnet creation and protocol upgrades |
| 7 | [vetKeys: How a Blockchain Can Keep Many Secrets][7] | abstract and metadata; received 2023-04-30 |

[1]: https://docs.internetcomputer.org/concepts/chain-key-cryptography/
[2]: https://eprint.iacr.org/2021/339
[3]: https://docs.internetcomputer.org/concepts/vetkeys/
[4]: https://docs.internetcomputer.org/concepts/security/
[5]: https://docs.internetcomputer.org/guides/canister-management/trust-in-canisters/
[6]: https://docs.internetcomputer.org/concepts/evolution-scaling/
[7]: https://eprint.iacr.org/2023/616
