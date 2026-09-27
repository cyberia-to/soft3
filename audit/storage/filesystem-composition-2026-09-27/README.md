---
title: filesystem composition source audit
tags: soft3, audit, fs, cyb, radio, storage
status: source-review
---
# Filesystem composition source audit

Inspected on 2026-09-27 to answer where filesystem behavior belongs across
soft3, Cyb, Cyber and Radio. Result:
[filesystem composition proposal](../../../proposals/filesystem-composition.md).
This is source and specification inspection. No product build, live-network
probe, migration or durability qualification was performed for this report.

## Scope and method

[sources.json](sources.json) records complete revisions, branches, local paths,
dirty-tree indicators and the acquisition commands. Soft3's baseline is fetched
`origin/main` at `434244e9f3ffd5850188dddb5ac4c5eceb6aa49c`. Other owner repositories
were inspected at their recorded local committed HEADs; this does not establish
that they form a compatible build or the latest remote release. Dirty working
files were excluded from the committed-code findings.

The registry at that soft3 revision contains 45 entries. This count is obtained
with:

```sh
python3 -c 'import tomllib; print(len(tomllib.load(open("release/components.toml", "rb"))["component"]))'
```

Breadth: registry, layers, release pins, composition/terms/languages/execution
contracts, component entrypoints and available root manifests. Depth: FS,
Cyb, Cyber, File, Cybergraph, BBG, Foculus, Radio, Neuron, Vault and Inf paths
in the evidence table. Other components were assessed for their place in this
composition, rather than exhaustively audited line by line.

To reproduce a cited source, use the full revision from `sources.json`:

```sh
git -C ~/cyber/<repo> show <revision>:<path>
git -C ~/cyber/<repo> ls-tree -r --name-only <revision>
```

For soft3 main, additionally read `README.md`, `release/components.toml`,
`release/phase1.toml`, `site/layers/index.html`, `specs/{terms,languages,execution-model,native-node,vault}.md`,
`roadmap/{component-boundaries,stack-completeness}.md`,
`proposals/cybergraph-sync-tade-architecture.md`, `crate/Cargo.toml`,
`crate/src/{lib,node}.rs`, and the CLI/MCP/Python/schema/conformance READMEs.

## Whole-stack coverage

Every registry entry is represented below. Layer placement and labels such as
`live` or `spec` describe the registry, not an executable readiness verdict.

| Components | Consequence for filesystem composition |
|---|---|
| strata, hemera, nox, trident, silicon, joy, lens, zheng, eidos | Arithmetic, identity and execution/proof tools remain reusable; an FS effect captures its content/state context. A proof-library dependency alone supplies no filesystem proof. |
| file, bbg, cybergraph, fs, inf | Separate immutable content, durability, publication, filesystem semantics and queries. File's registry verb `name` refers to content identity; FS owns contextual names. |
| foculus, radio, tade, tru, tok | Separate reconciliation, transport/framing and network convergence/economics. Local document edits need no global finality cycle. |
| neuron, mudra, vault, ward | Preserve subject authority, cryptographic mechanisms, custody and current permission checks at effects; FS has no independent key or permission issuer. |
| cyb, cy, true-cyber, nu, spark, prysm, mir, evy, rune | Product/shell/presentation/runtime adapters consume one service. The existing `cy` provides a terminal home; native OS IO and graph operations need explicit context. |
| soma, glia, honeycrisp, wysm, neural, sigma, kern | Tasks/models/programs consume scoped content and snapshots; execution/rendering/acceleration choices do not acquire storage or signing ownership. |
| soft3, optica, lytics, cybernode, bostrom, pussy | Composition/qualification, publishing, domain applications, deployment and networks consume declared profiles. Publishing a workspace and publishing private replicas require different grants. |

The registry puts FS at layer 5, phase `after`, state `spec`, alongside the
body/product components. Its current delivery phase needs revision when FS
becomes a shipping dependency; the proposal does not change that status by fiat.
The Cyber node product is inspected additionally: the registry's `true-cyber`
entry alone does not describe both packages currently emitting `cyber`.

Local provenance gaps: `nu/.git` points to an unavailable Git directory;
`git -C ~/cyber/nu rev-parse HEAD` fails. The source directory exists, but this
scan cannot give it a committed local revision. `sigma` and `kern` have no local
repository at the expected paths. These are local inspection limits, with no
inference about whether the projects exist publicly. Nu's missing pin in this
soft3 snapshot is release metadata, not evidence that the fork is private.

## Findings with source evidence

| Finding | Committed source and implication |
|---|---|
| FS has a domain home and no Rust implementation in the inspected tree | `fs@75d4380`, `git ls-tree -r --name-only`: four Markdown files. `patch/spec.md` describes independent patches, channels, graph paths and first-class conflicts, explicitly pre-implementation. Scaffold the library here. |
| Cyb already owns the personal filesystem experience | `cyb@46a05d2`, `parts/{fs,memory,now,com,name}.md`, `cli/src/main.rs`, `cli/Cargo.toml`, `core/src/lib.rs`: Memory/Brain/Com share a robot context; `cy` is the existing headless CLI. Cyb's token/robot name resolver and FS path resolution are separate scopes. |
| Committed Cyb still has parallel persistence | Same Cyb revision, `shell/src/worlds/content.rs`: `~/cyb/particles.jsonl`, whole-file read and best-effort appends; `core/src/cell.rs`: `graph.log`, replay and ignored write/flush errors; `shell/src/worlds/mod.rs`: shared Cell with fallback to ephemeral on open failure. These need the common storage migration, not another wrapper around successful-looking writes. |
| Ward presentation exceeds actual enforcement | Same Cyb revision, `anatomy.md` labels Ward live; `parts/ward.md` says unbuilt. `shell/src/worlds/com/mod.rs` has ungated emit and stub host effects. Its Nu initialization installs the normal shell context and OS HOME, not a graph FS provider. |
| Cyber owns the network product lifecycle | `cyber@cfdc0e5`, `specs/node-product.md`, `Cargo.toml`, `src/main.rs`: native node and service/configuration ownership; proposed GraphSession composition is separate from network attachment. Personal file commands have no architectural obligation to live here. |
| Neuron convergence already names the shared integration path | `neuron@c875411`, `specs/integration.md`, `node/src/lib.rs`: GraphSession, common Host/Registry, explicit source migration, `Graph::from_database`. These contracts describe convergence beyond the older committed Cyb snapshot; this audit does not claim that Cyb already runs it. |
| Two packages emit the same binary name | `cyber@cfdc0e5` and `true-cyber@b58589f`, `Cargo.toml`/READMEs: network launcher versus headless native client. CLI spelling cannot be used as a reliable architectural boundary. |
| File supplies immutable value types | `file@fc666a5`, `src/lib.rs`, `README.md`: Particle, File, Kind and sniffing, with `Vec<u8>` payload. Streaming service and path/channel semantics are separate work. |
| Shared publication exists independently of FS | `cybergraph@0eae7ae`, `specs/applications.md`, `src/application.rs`; `bbg@c75b685`, storage/application code. Application publication validates and conditionally commits history/receipts in the shared database. This alone establishes neither a native Signal nor a public consensus/query proof. |
| Useful catalog implementation has overly broad ownership | Feature `cybergraph@1571ed9`, `src/catalog.rs`, `src/catalog/{codec,tree}.rs`, `specs/catalog.md`: local path grammar, stable bindings, edit/rename policy and revision encoding are FS semantics. Keep the durable radix index, CAS and retention mechanisms; adapt domain policy into FS. Current namespace-wide linear history does not implement patch/channel merge. |
| Radio CLI currently performs storage composition | Feature `radio@a2fda22`, `radio-cli/src/{files,names}.rs`: database/backend/namespace selection and opening Cybergraph/BBG. Rename of the executable alone leaves this assembly mistake intact. |
| Radio has an injected content path but extraction remains incomplete | Feature `radio@a2fda22`, file-stream protocol and `scripts/check-cli-storage.py`; `cybergraph@1571ed9`, `radio/`: Provider/Source/Sink over shared BBG. Old blob/docs/Willow implementations still exist outside that CLI closure. |
| Sync ownership documents conflict | Soft3 main `roadmap/component-boundaries.md` records Sync merged into Foculus; historical `proposals/cybergraph-sync-tade-architecture.md` carries a supersession note. `foculus@9e32aa6`, `specs/structural-sync.md` retains the older narrower split. Prefer an explicit reconciliation of contracts over creating a second Cyb sync engine. |
| Foculus has useful mechanisms, with application-specific limits | Same Foculus revision, `src/{reconcile,fork,conflict,erasure,das,nmt}.rs`: pluggable fork choice and reusable availability algorithms. Conflict keys identify native author/step equivocation. These are not FS edit conflicts. `Cargo.toml` unconditionally includes tru/zheng/tok; a lightweight application substrate is proposed work. |
| Foculus also owns duplicate storage | Same Foculus revision, `src/{store,node,vdisk}.rs`: directory-backed ChunkStore, filename LWW GSet, `registry.json`, shard writes and `secret.key`; network uses registry iroh. `store.rs` declares `MAX_REGISTRY_ENTRIES = 100_000`. The same BBG/Vault/Radio extraction must cover these paths. The value is read with `git show 9e32aa688e0f03ffa8fd9a3c099bc7e50cc8c744:src/store.rs`. |
| Inf has a path-resolution contract to reconcile | `inf@bf37987`, `specs/relations.md`: latest assertion under neuron/path wins; FS draft instead has channels and concurrent patches. Bind FS query relations to the same selected profile/state. The document explicitly says current BbgSource is `provable() == false`. |
| Vault requires stronger admission than file merge | `vault@f3fe2e4`, `specs/synchronization.md`: private application history, complete ciphertext closure and writer/freshness rules; filesystem LWW cannot select a Vault head. `soft3/specs/vault.md` additionally requires unlock bootstrap independent of the locked key. |

## Feature work and unresolved interfaces

Storage work is on pushed feature branches, distinct from the main baseline:

| Repository | Recorded source |
|---|---|
| soft3 | [189c8aa](https://github.com/cyberia-to/soft3/tree/189c8aa9809aed18417023c9b2f8b324dabd5b5b) |
| bbg | [b1b273d](https://github.com/cyberia-to/bbg/tree/b1b273dbbc784fc4f49814294b7cc7c825389aa2) |
| cybergraph | [1571ed9](https://github.com/cyberia-to/cybergraph/tree/1571ed98a7299d513091631100e71c37178773b1) |
| radio | [a2fda22](https://github.com/cyberia-to/radio/tree/a2fda22560b1c00bee231146808bac2600c06c8d) |

Read `soft3/specs/storage.md` and `roadmap/storage/{README,identity,radio-removal}.md`
at that feature revision. They already assign FS semantics to FS, persistence
to BBG, and require consumer/import parity before physical Radio deletion.
Existing check receipts remain in those branches' audits; they were not rerun
or promoted by this architectural scan.

Remaining interfaces are substantive: canonical file identity and range proof
binding; FS patch/frontier semantics; current grants on local/remote effects;
complete application-history synchronization; retention release, read protection
and GC; private metadata/discovery; checkpoints and recoverable names/content.
The sealed-content feature currently retains indefinitely. A proposed replica
or proof API does not fill any of those implementation gaps.

The owner soft3 checkout is a different `docs/vault-spell` snapshot recorded
under `additional_snapshot` in sources.json. Its neuron/signed-adapter work and
the newer Neuron/Cyber contracts must converge with the storage feature before
product integration. The proposal deliberately preserves that work rather than
starting a competing Cell or GraphSession design.

## Review conclusion

Move filesystem semantics into FS and compose it into Cyb's common runtime.
Keep Cyber as an optional host and Radio as transport. The cleanup must include
Cyb's parallel writers and Foculus's storage/custody paths, while keeping Vault
admission, public network finality and document merge as distinct contracts.
Implementing these boundaries is a follow-up; this audit certifies source
observations only.

## Documentation validation

On the proposal tree based on soft3 `434244e9f3ffd5850188dddb5ac4c5eceb6aa49c`:
`git diff --cached --check` and `python3 site/check-components.py` passed.
Local Markdown destinations in the changed pages resolved, and the coverage
table/source inventory matched every registry component. A separate read-only
review corrected the distinction between proposed shared interfaces and current
Cyb behavior, and made Nu's ambient host authority explicit. Runtime acceptance
remains the implementation work described in the proposal.
