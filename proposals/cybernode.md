---
tags: cyber, soft3, proposal, cybernode, node, network, chain, operations, release, ladder
crystal-type: process
crystal-domain: cyber
alias: cybernode, the seventh rung, the node, rung 7 rename, soft3 node rename, cybernode instance, cybernode.ai
status: proposed
date: 2026-10-17
---
# cybernode — the seventh rung is the node, and cyberia runs on it

the owner, 2026-10-17: the seventh of soft3's seven stack rungs is called *soft3*, and that is an architectural hole — a mixture with no name of its own. rename it **cybernode**, give it the domain cybernode.ai, and make it one thing with two faces: the product that runs the infrastructure, and the instance of that product that cyberia operates — explorer and the rest. cybernode is an integral part of soft3: the node and everything networked. and it must grow until cyberia can run all of its execution infrastructure on it. without this, soft3 cannot start to release systematically and structurally.

this page is the diagnosis, the picture, the decisions, what cybernode must contain, and the order of the moves. it depends on the [[soft3/roadmap/stack-completeness|ladder]], the [[soft3/roadmap/component-boundaries|component boundaries]], the [[soft3/specs/execution-model|execution model]], [[network-planes]] and [[proven-present]].

## 0. the decision in one paragraph

**soft3 is the stack; cybernode is its node.** soft3 names the ten-rung ladder and the repository of its contracts — specs, release train, conformance, SDKs — and it never runs. cybernode is rung 7: one binary that runs any compatible network instance as data, and that owns everything networked — the chain kernel, the wire and sync, routing, mining, proving workers and block aggregation, the HTTP surface, snapshots and tip proofs, the explorer's index, status. its planes stay where [[network-planes]] put them: [[radio]] reaches, sync reconciles, [[foculus]] agrees, solong rewards. **cybernode.ai is the reference instance**: cyberia's own deployment, described as data and run by cybernode. an instance is cybernode plus a manifest; anyone can stand up one of the same shape.

## 1. the hole

one word names six things:

| "soft3" today | what it is |
|---|---|
| the stack | the ten-rung ladder: nox · trident · joy · cybergraph · neural · cy · soft3 · cyb · cyber · cyberia |
| rung 7 | "network — agents connected: files by particle, epochs agreed" (soft3.org ladder), parts radio · sync · foculus · solong |
| the repository | `cyberia-to/soft3`, whose charter says "foundational stack composition contracts and developer interfaces … do NOT implement stack logic here" (`CLAUDE.md`) |
| the crate | `soft3` 0.10.0 on crates.io: "network + real cybergraph node + CLI" |
| the binary | `soft3 node`, `soft3 mine`, `soft3 sync` |
| the site | soft3.org |

the crate breaks the repository's own charter. `crate/src` holds the chain kernel (51 files under `chain/`), the genesis pipeline, the node with its HTTP routes, the wallet, the network glue; `deploy/` holds the service units of a running chain; `specs/` holds chain.md, chain-wire.md, chain-conformance.md, genesis.md, native-node.md and routing.md. that is stack logic and operations living in the contracts repository, under a rung name that is also the stack's name. routing.md says it exactly: "the node is the only place where the three meet" — and the node has no name.

the other half has the name and no software. `cyberia-to/cybernode` is a private operations repository: seven Hetzner servers, go-cyber archive and RPC nodes for bostrom and space-pussy, cyberindex with hasura, an IPFS repo of 3,217,835 pins, lytics, every static site, ERPNext, and since October the soft3 `pussy-rc` chain and miner (`audit/infra-footprint-2026-10-16.md`). it holds secrets in git (`local/.env`, `local/keys`, `local/keyring-test`). cybernode.ai itself is a redirect to a Grafana dashboard; domains.cybernode.ai is the domain register.

software without a name, a name without software. a release train cannot ship a rung whose code lives in the contracts repo under the stack's name, whose specs sit beside the release rules, and whose operators run it from a private repo with another name.

## 2. the picture

| layer | name | is | runs? |
|---|---|---|---|
| the stack | **soft3** | the ladder and its composition contracts: execution model, release train, terms, types, languages, conformance snapshots, SDKs, MCP | never |
| rung 7 | **cybernode** | the node: one binary, many network instances, everything networked | yes |
| its planes | radio · sync · foculus · solong | transport · reconciliation · agreement · rewards ([[network-planes]]) | inside cybernode |
| an instance | **cybernode.ai** (the reference) | cybernode + an instance manifest: which networks, which roles on which hosts, which endpoints, which workers | operated by cyberia |
| operations | private repository | secrets, inventory, runbooks of cyberia's hosts | never public |

the ladder after the change:

| rung | before | after |
|---|---|---|
| 7 | soft3 · network — agents connected: files by particle, epochs agreed | **cybernode** · network — the node: agents connected, files by particle, blocks proven and agreed |

the question of the rung ("how do we coordinate?") and its facet (play) do not change, nor do its parts. what changes is that the rung now names a product somebody installs and an instance somebody visits.

product and instance are one thing, not two products: the execution model already makes a network instance "data supplied as configuration to a compatible warrior" (§1). cybernode extends the same rule from networks to deployments — an instance manifest is data, and cybernode.ai is manifest number one. the explorer, the public endpoints and the snapshot service are not a second codebase beside the node; they are roles of the node, switched on in the manifest.

## 3. decisions

1. **rung 7 is cybernode.** every ladder, registry and page that names rung 7 says cybernode: soft3.org ladder, `status.md`, `roadmap/stack-completeness.md`, `roadmap/terms-map.md`, the cyber wiki.
2. **soft3 means the stack and nothing else.** the soft3 repository returns to its charter: contracts, interfaces, the release train. no node, chain, wallet or deploy code stays in it.
3. **one binary, many instances, as data.** `cybernode` runs pussy, bostrom and later the CYB chain from configuration ([[chain]] §4.4 already says "two chains, one binary"); an instance manifest selects networks and roles.
4. **the code moves unchanged.** `crate/src/{chain,genesis,node,node.rs,network.rs,wallet,bin}` move to the cybernode repository as the crate `cybernode` (the name is free on crates.io, checked 2026-10-17). the move changes paths and names only; the chain conformance vectors and the chain fleet prove that no behaviour changed.
5. **two repositories, split by secrecy.** `cyberia-to/cybernode` becomes the public component: code, specs, instance manifests without secrets, audits. the current private operations repository is renamed (proposed `cyberia-to/cybernode-ops`) and keeps secrets, inventory and runbooks. no secret crosses into the public repository, and the tracked secrets of the private one are moved out of git as part of the rename (§6.2).
6. **the node's specs move with it, and moved means moved.** chain.md, chain-wire.md, chain-conformance.md, genesis.md, native-node.md and routing.md go to `cybernode/specs/`; the soft3 copies are deleted, every link is updated, the proposals that cite them ([[proven-present]], [[privacy-frozen]], [[steps-blocks-epochs]], [[goldilocks-parano1d]]) point at the new paths. soft3/specs keeps the execution model, releases, the release manifest, terms, types, languages, filenames, the vault and kadek compositions.
7. **the verbs move to the node's binary.** `cybernode run | mine | prove | sync | wallet | genesis | status`. the `soft3` binary keeps forwarding shims for the operational verbs for one release, then drops them. `soft3` stays resolvable on crates.io — never yanked — and its next version is the stack facade that depends on cybernode under the `stack` feature.
8. **cybernode.ai is the instance's front, not a redirect.** the home page shows the instance as the node reports it — networks, heights, tips, peers, proofs, endpoints — from the nodes' own status; the explorer, the endpoints list, snapshots and tip proofs, the domain register and the metrics dashboards are its sections.
9. **cybernode rides the release train as a sibling.** it is pinned in `release/phase1.toml`; the soft3 candidate ships the cybernode binary; the `node-status` gate checks it. deploy units become `cybernode@<network>.service`.
10. **the legacy belongs to the reference instance until it is retired.** the go-cyber nodes of bostrom and space-pussy run on cybernode.ai's hosts until their balances are claimed ([[proven-present]] §2.9: PUSSY and BOOT live with ECDSA until then); cyberindex and hasura run until the explorer index over bbg replaces them; Kubo until particles are served by radio blobs. each retirement is a dated line in the instance's audit, never a silent switch-off.

## 4. what cybernode must contain for cyberia to run on it

the test the owner set: cyberia runs all of its execution infrastructure on cybernode. the capabilities, where each lives today, and where it goes:

| capability | today | in cybernode | status |
|---|---|---|---|
| node — validation, state, chain kernel | soft3 `crate/src/chain`, `node` | the core | built (phase-1 chain, soft3#189) |
| wire, sync, discovery | soft3 node + radio + foculus; chain-wire.md | the network plane | built for the chain (soft3#187) |
| routing — the wire obeys the graph | routing.md | node-owned, as routing.md says | specified |
| mining | `soft3 mine` (CPU, Metal, WebGPU) | `cybernode mine` | built |
| proving workers and block aggregation | joy, zheng; not in any node | worker roles per the execution model: step provers, aggregators, the block merge of [[proven-present]] §2.5 | not built |
| HTTP surface | native-node.md, `node/routes.rs` | the node's API, one versioned surface for cyb, true-cyber, the explorer | built |
| explorer and index | cyberindex + hasura + postgres over go-cyber (Deimos, Jupiter) | an index role over bbg through [[inf]]: blocks, transactions, neurons, links, particles, supply | not built |
| snapshots and tip proofs | none | `(snapshot, tip proof)` serving, [[proven-present]] S5 | not built |
| archive and pruning | phase-1 nodes are archival (chain §3.6) | archive role, prune role | partly built |
| particles and files | Kubo on Io, 3.2 M pins | radio blobs, verified streaming (routing.md, "fetch a file by particle") | specified |
| status and metrics | Grafana and Prometheus on Mimas; cybernode.ai redirects there | the node's own status and metrics; the instance front aggregates them | partly built |
| supervision and deploy | soft3 `deploy/` units, cybernode `servers/` | `cybernode@<network>` units, the instance manifest, one deploy command per host | partly built |
| upgrades | activation tables in the release (chain §7.1) | the release train ships the binary; the manifest pins its version per host | specified |
| operator keys | payout address, checkpoint key | keys held through [[vault]] / [[mudra]], never in a manifest | not built |
| legacy bootloaders | go-cyber bostrom and space-pussy | roles of the reference instance until claims close | running, to retire |

hosted applications are not the node: lytics, the static sites, ERPNext run on the instance's hosts and are listed in its manifest as services, but their code stays in their own repositories.

## 5. where things live afterwards

| repository | holds |
|---|---|
| `soft3` | the ladder pages, proposals, the execution model, the release train and its manifest, terms, types, languages, filenames, compositions (vault, kadek), conformance snapshots, SDKs (`js/`, `py/`), `mcp/`, `schema/`, the soft3.org site, the stack facade crate |
| `cybernode` (public) | `rs/` the crate and binary; `specs/` chain, wire, conformance, genesis, native node, routing, the instance manifest format; `deploy/` units and templates; `instances/cybernode.ai/` the reference manifest without secrets; `audit/` |
| `cybernode-ops` (private, renamed) | secrets, host inventory, runbooks, recovery notes |
| radio, foculus (with sync), tru, … | unchanged; cybernode depends on them |

## 6. the plan

each phase closes on its last merge with a receipt in `audit/`.

| phase | what | acceptance |
|---|---|---|
| **C0 — the name** | this page; rung 7 renamed on the ladder, in `status.md`, `stack-completeness.md`, `terms-map.md`, the soft3.org ladder and the cyber wiki; no code moves | every page that names rung 7 says cybernode; `stale` grep for "rung 7 soft3" is empty |
| **C1 — the repositories** | rename the private operations repository (owner action, §7.1); create the public `cyberia-to/cybernode`; move the tracked secrets of the private repository out of git into the operators' vault | public repo exists with a README and the charter of §5; `git ls-files` of the private repo lists no key, keyring or `.env` |
| **C2 — the code** | move `crate/src/{chain,genesis,node,wallet,network,bin}` to `cybernode/rs` with history; crate and binary `cybernode`; soft3 becomes the facade with forwarding shims | `cargo test` in cybernode equals the soft3 crate's test count before the move; chain conformance vectors green; chain fleet green; `cargo install cybernode` from a clean checkout runs a pussy node |
| **C3 — specs and deploy** | move the six node specs and `deploy/`; update every link in soft3, cyber, cyb, the proposals and the launch tracker; soft3 keeps no node code | link check green on the cyber.page build (zero unresolved links into moved pages); a guard in soft3 CI fails if `crate/src/chain` reappears |
| **C4 — the train** | cybernode pinned in `release/phase1.toml`; the candidate ships the cybernode binary; `node-status` gate on it; hosts switch `soft3-chain@` units to `cybernode@` at a quiet height | a Friday candidate with the cybernode binary and green gates; the running pussy nodes report the cybernode version in `/status` |
| **C5 — the instance front** | cybernode.ai home from the nodes' status; endpoints; explorer v0 over the node API (blocks, transactions, neurons, links); the domain register and metrics as sections | cybernode.ai serves the page, every number on it read from a live node, no placeholder |
| **C6 — the rest of section 4** | proving workers and aggregation, the index over bbg, snapshots and tip proofs, particles over radio blobs, operator keys; each legacy service retired when its replacement is live | each capability row of §4 reaches "built" with its receipt; each retirement dated in the instance audit |

timing against genesis (2026-11-05): C0–C3 are a rename and a move with zero behaviour change, gated by the conformance vectors and the fleet. the target is the candidate of 2026-10-30, so that the chains launch from the binary that carries the rung's name. if C2 is not green by then, genesis ships from the soft3 crate unchanged and C2 lands on the first candidate after genesis; the owner chooses (§7.3).

## 7. open, and who decides

1. **the private repository's new name** — `cybernode-ops` proposed. renaming on GitHub keeps a redirect until a repository of the old name is created, which C1 does deliberately; every script that clones the old name is updated in the same phase. owner.
2. **true-cyber and its `cyber` binary.** today the thin product client ("product face") beside the node. either it becomes `cybernode`'s client mode, or it stays the product face of rung 9. it is never yanked either way. owner.
3. **before or after genesis** for C2 (§6). owner.
4. **the explorer's face.** the index is a node role; whether its pages are served by the node, rendered in cyb, or both. cybernode with cyb.
5. **the label of the rung** — "network" kept, or "node". owner.

## 8. sources

- soft3: `CLAUDE.md` (charter), `crate/Cargo.toml` (0.10.0), `crate/src/` tree, `deploy/`, `specs/README.md`, `specs/execution-model.md` §1, `specs/routing.md`, `specs/chain.md` §3.6, §4.4, §7.1, `release/phase1.toml`, the soft3.org ladder (`site/layers/index.html`), [[network-planes]], [[proven-present]].
- cybernode (private): `README.md`, `audit/infra-footprint-2026-10-16.md`, `servers/`.
- cybernode.ai and domains.cybernode.ai, read 2026-10-17; crates.io `cybernode`: "crate does not exist", 2026-10-17.
