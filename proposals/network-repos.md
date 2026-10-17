---
tags: cyber, soft3, proposal, cybernode, genesis, network, bostrom, pussy, cyber, plumb, tok, release
crystal-type: process
crystal-domain: cyber
alias: network repos, network repositories, a network is a repository, network config, launch configurator, network manifest
status: proposed
date: 2026-10-17
---
# a network is a repository — cybernode takes it as input

the owner, 2026-10-17: every network has a configuration — a set of contracts (plumb), the genesis distribution, the starting parameters, the starting graph: a configurator of the launch. there must be three such repositories — **cyber** (true-cyber), **bostrom** and **space-pussy** — and **cybernode**, the software that takes that repository as its input. today it is not so, and it is not thought through to the end.

this page fixes what a network repository is, what moves into the three, what cybernode does with one, and the order. it continues [[cybernode]] (which left true-cyber open, §7.2 — this page closes it) and uses the [[soft3/specs/execution-model|execution model]], where a network instance is already "genesis/domain identity, rule versions, trust configuration and endpoint locations" supplied as data.

## 0. the decision in one paragraph

**a network is defined by one repository, and nothing about a particular network lives anywhere else.** the repository holds four things — the contracts the network carries (plumb programs), the genesis distribution (its sources and its derivation policy), the starting parameters (identity, rule sets, consensus values, authorization policy) and the starting graph — plus a lock: the particle of the genesis file it builds, which is the chain id. there are three: `true-cyber` defines **cyber** (CYB), `bostrom` defines **bostrom** (BOOT), `space-pussy` defines **pussy** (PUSSY). cybernode is generic: it reads a network repository at a pinned revision, builds the genesis deterministically, checks it against the lock, and runs the network. the binary knows no network by name.

## 1. what is wrong today

the definition of each network is spread over four places, and the third network has none:

| piece of a network | where it lives today | problem |
|---|---|---|
| burial sources (snapshot pins) | `bostrom/snapshot/manifest.json`, `space-pussy/snapshot/manifest.json` | right place — the seed of this proposal |
| derivation policy (denoms carried, pools dissolved, passports, legacy owners, particles re-addressed, the u64 filter, pussy supply, TOCYB) | `soft3/specs/chain.md` §4.2, `soft3/specs/genesis.md` | per-network policy written into the stack's spec |
| starting parameters — genesis time, bits, subsidy, checkpoint key and sunset | the `p` frame, computed by `soft3 genesis build`; times written in chain.md §4.3 and genesis.md | per-network values inside a generic pipeline and a spec |
| consensus values — block time, half-life, finality, reorg limit, checkpoint interval, budgets | `soft3/crate/src/chain/params.rs`, global constants | one set for every network; a second network with other values needs a code change |
| defaults per network — ports, endpoints | `match chain { "pussy" => …, "bostrom" => … }` in `crate/src/bin/soft3.rs` | the binary knows networks by name |
| contracts | none: phase-1 chains have native transfers only; plumb (Coin, Card, pay · lock · update · mint · burn) is not declared by any network | the CYB claim and the later move of PUSSY and BOOT into contracts have nowhere to be declared |
| the cyber network | nowhere. `true-cyber` is a CLI client (`cyber`, crate 0.8); "TOCYB × 666 → CYB belongs to $CYB genesis" (chain.md §4.2) points at a genesis that has no home | the network that matters most is the one without a definition |

and two of the three repositories carry other things: `space-pussy` is the go-cyber codebase of the old chain (`app`, `x`, `proto`, `cmd`, `go.mod`) plus a landing plus the snapshot tooling; `bostrom` is the bostrom wiki (`root/`, `heroes/`, essays) plus the snapshot tooling.

## 2. the network repository

one layout for all three:

```text
<network>/
  network.toml          identity and rules
  genesis/
    sources.toml        pinned inputs: the burial manifest (CID, sha256, bytes) — or none
    distribution.toml   the derivation policy, written as rules from a fixed vocabulary
    graph.toml          the starting graph: link sources, passports, naming links, particle policy
  contracts/            plumb programs carried at genesis: trident sources and their digests
  endpoints.toml        bootstrap peers and public endpoints, signed by the network's key
  genesis.lock          the output: genesis file particle (= chain id), block-0 id, state root, size
  audit/                build receipts: two independent byte-identical builds per lock
  …                     what the repository already holds (wiki, landing, snapshot tooling) stays beside
```

`network.toml` carries, for each network:

| section | what |
|---|---|
| identity | name, hrp, native denom and token ids, genesis time |
| rule sets | the activation table (chain §7.1): which rule set from which height, and for each the transaction kinds it admits |
| consensus | block time, half-life, finality depth, reorg limit, checkpoint interval, key and sunset, budgets, subsidy policy — values that today are global constants become per network, bounded by what the rule set allows |
| authorization | which signature schemes the network admits and until when: NSIG1 (secp256k1) for pussy and bostrom until their balances are claimed; hash-only for cyber from genesis ([[proven-present]] §2.9) |
| proof | the block-proof profile and its activation ([[proven-present]] §2.8, S4) |

`distribution.toml` and `graph.toml` are rules, not code: "carry every denom as its own token", "dissolve LP positions at halt reserves", "secp256k1 key → `neuron_of`, otherwise `legacy_owner(hrp, id)`", "one passport link per nickname", "re-address a CID as `hemera(bytes)` or the black hole `hemera(cid_string)`". the vocabulary of rules is cybernode's (§3.3); which rules a network applies, with which values, is the network's.

the three, filled in:

| | pussy (`space-pussy`) | bostrom (`bostrom`) | cyber (`true-cyber`) |
|---|---|---|---|
| sources | space-pussy burial, halt 19,940,993 | bostrom burial, halt 25,120,712 | the proven states of pussy and bostrom (claims), no burial |
| distribution | every denom carried; S0 = bank supply at halt | every denom carried; passport contract balance dormant under `legacy_owner` | CYB minted only by claims; nothing distributed at genesis except what the network.toml names |
| starting graph | pussy's cyberlinks | bostrom's 2,949,732 cyberlinks + 47,837 passport links | open (§6.1) |
| contracts | none in rule set 1; balances move into contracts later | none in rule set 1; balances move into contracts later | the CYB coin and the claim program — a claim proves, against pussy's or bostrom's proven state, the lock or burn that names a CYB neuron |
| authorization | NSIG1 until claimed | NSIG1 until claimed | hash-only, no ECDSA anywhere |
| rule set 1 | transfer · cyberlink · claim (chain §2) | the same | claims only, no transfers |
| genesis | 2026-11-05 13:37:00 UTC | 2026-11-05 13:22:42 UTC | when the proving and the encryption reach a stable construction |

## 3. decisions

1. **one network, one repository.** everything particular to a network — sources, policy, parameters, contracts, starting graph, endpoints, lock — lives in its repository and nowhere else. the three are `true-cyber`, `bostrom`, `space-pussy`.
2. **true-cyber becomes the cyber network.** the repository keeps its name, because `cyberia-to/cyber` is the wiki behind cyber.page; the network's name is cyber and its coin CYB. its client code (the `cyber` binary: neuron, sync, link, relay, receipt) leaves it and becomes cybernode's client commands. the crate `true-cyber` stays resolvable on crates.io and is never yanked; its last version points at cybernode.
3. **cybernode owns the format and the vocabulary; the network owns the values.** the network manifest format and the rule vocabulary are a spec of cybernode (`specs/network.md`, written in soft3 until [[cybernode]] C3 moves the node's specs). a network can only use rules the vocabulary has; a new kind of rule is a cybernode change, reviewed like code.
4. **the genesis is a function of the repository.** `genesis = build(network repo @ rev, pinned sources)`, deterministic, byte-identical across independent builds; the result is pinned in `genesis.lock`, and the chain id is the particle of the file (chain §4.1). a node refuses a genesis whose particle differs from the lock of the revision it was given.
5. **per-network policy leaves the stack spec.** chain.md keeps the generic: the genesis file format (§4.1), the generic derivation operations, block 0 (§4.3). the bostrom and pussy decisions of §4.2 and the genesis times of §4.3 move into their repositories' `distribution.toml`, `graph.toml` and `network.toml`, with their rationale in each repository's docs; genesis.md keeps the pipeline and its command.
6. **consensus values become network data within rule-set bounds.** what is a constant in `params.rs` today becomes a value the network declares; the rule set defines the admissible range (a network cannot declare a block time the proof cannot keep, or a finality depth of zero). the two phase-1 networks declare today's values unchanged.
7. **contracts are declared, not implied.** a network lists the plumb programs it carries at genesis and from which rule set others activate; their digests enter the genesis. pussy and bostrom carry none in rule set 1. cyber carries the CYB coin and the claim program from its genesis.
8. **the binary knows no network by name.** cybernode runs `--network <repo path | revision | chain id>`; ports and endpoints come from `endpoints.toml` and the instance manifest ([[cybernode]] §2), not from a `match` on a name.
9. **network repositories ride the release train.** they are pinned in `release/phase1.toml` like siblings; a cybernode release embeds the `genesis.lock` of every network it ships for, so a node started on a known chain checks its genesis offline.
10. **what else a repository holds stays beside, clearly apart.** the bostrom wiki and the pussy landing stay; the snapshot tooling becomes the sources tooling of `genesis/`; the go-cyber codebase of space-pussy is the old chain, not the network's definition — it moves to `legacy/` or to an archived repository (§6.3).

## 4. what cybernode does with a network repository

```text
cybernode genesis build --network <repo>[@rev]     → genesis file + report, checked against genesis.lock
cybernode genesis verify --network <repo>[@rev]    → rebuild, compare particle, block-0 id, root
cybernode run --network <repo | chain id>          → a node of that network (roles from the instance manifest)
cybernode network check <repo>                     → the manifest parses, every rule is in the vocabulary, every source pin resolves
```

today's `soft3 genesis build --chain pussy|bostrom --snapshot DIR` becomes the first of these with the chain's policy read from its repository instead of from code and specs.

## 5. the plan

each phase closes on its last merge with a receipt in `audit/`.

| phase | what | acceptance |
|---|---|---|
| **N0 — the format** | `specs/network.md`: the layout of §2, the `network.toml` schema, the rule vocabulary, the lock | the spec merged; `cybernode network check` (still `soft3 network check`) validates a manifest against it |
| **N1 — pussy and bostrom as data** | write `network.toml`, `genesis/{sources,distribution,graph}.toml`, `endpoints.toml` in `space-pussy` and `bostrom` from today's chain.md §4.2–4.3, genesis.md and `params.rs`; the builder reads them | the genesis built from each repository is byte-identical to the one today's pipeline builds — same particle, same block-0 id, same root |
| **N2 — the binary forgets names** | builder and node read the repository; the per-name defaults in `bin/soft3.rs` and the global consensus constants become network data; chain.md §4.2–4.3 slimmed to the generic | conformance vectors and the chain fleet green; `git grep '"pussy"\|"bostrom"'` in the node's non-test code is empty |
| **N3 — locks and the train** | `genesis.lock` committed in both repositories with two independent build receipts; both repositories pinned in `release/phase1.toml`; the binary embeds the locks | the Friday candidate carries both locks; a node refuses a genesis that does not match its network's lock |
| **N4 — cyber** | true-cyber becomes the cyber network repository: client code moves to cybernode's client commands; `network.toml` with rule set 1 = claims only, hash-only authorization, the CYB coin and the claim program declared | `cybernode network check true-cyber` green; the claim program's relation specified against pussy's and bostrom's proven state ([[proven-present]] §2.9) |
| **N5 — contracts** | plumb programs as declared contracts: trident sources and digests in `contracts/`, compiled and proven like any nox program; the activation of contracts on pussy and bostrom as a rule set | a test network whose genesis carries a declared contract and a transaction that runs it |
| **N6 — cyber genesis** | when the proving and the encryption are stable: the cyber genesis built from its repository, locked, shipped | the lock published; two independent builds identical |

timing: N1–N3 touch the genesis of 2026-11-05. their acceptance is byte-identity with today's genesis, so they change no outcome; the target is the candidate of 2026-10-30. if they are not green by then, the genesis ships from today's pipeline and N1–N3 record the same particle in the repositories right after; the owner chooses.

## 6. open, and who decides

1. **cyber's starting graph.** empty, the union of the bootloader graphs as proven by pussy's and bostrom's states, or a curated graph. owner.
2. **where the `cyber` client commands land.** cybernode's client commands (decision 2), or cyb's headless mode, or both. owner, with cyb.
3. **space-pussy's go-cyber code.** a `legacy/` directory in the repository, or a separate archived repository (for example `go-space-pussy`), so that the repository root is the network's definition. owner.
4. **the network key.** who signs `endpoints.toml` and the lock: the foundation key of chain §5.3 for pussy and bostrom, and for cyber a key that is not ECDSA. owner.
5. **bounds of the rule set.** which consensus values a network may change within a rule set, and which only by a new rule set. cybernode spec.
6. **conflict with the repositories draft (soft3 PR #220, open, 2026-10-17).** that draft folds `true-cyber` into the `cyber` repository and archives `bostrom` and `space-pussy` as monuments of the bootloader. this page, on the owner's instruction of the same day, keeps all three as the repositories that define their networks; only the go-cyber code inside `space-pussy` is a monument (decision 10). the two must be reconciled before either merges its moves. owner.

## 7. sources

- soft3: `specs/chain.md` §0, §4, §7.1; `specs/genesis.md`; `crate/src/chain/params.rs`; `crate/src/bin/soft3.rs`; `specs/execution-model.md` §1–2; [[cybernode]]; [[proven-present]] §2.9.
- `bostrom/snapshot/manifest.json`, `space-pussy/snapshot/manifest.json`; the repositories' trees, read 2026-10-17.
- true-cyber `README.md` (the `cyber` client, 0.8); plumb `README.md` (tok: Coin, Card, PLUMB).
