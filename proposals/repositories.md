---
tags: cyber, soft3, proposal, repositories, organization, github, monorepo, lab, products, projects
crystal-type: process
crystal-domain: cyber
alias: repositories, the shape of the repositories, repository shape, monorepo proposal, the lab, lab regime
status: draft
date: 2026-10-17
---
# the repositories — what merges into what, and the lab

the owner, 2026-10-17: a hundred repositories, many of them parts of products; publish parts from folders; keep the warriors apart; put kernel and core on different footings; and until a project has become a product, keep it with the others under loose rules, because a product and a non-product live by very different laws. this page is the picture, the rule, the table of moves, and the order. evidence: `cyberia/audit/repositories-2026-10-17.md`; the decision page: `cyberia/tasks/d7-repository-shape.md`.

## 0. the decision in one paragraph

**a repository is the door of something that stays, and everything else lives in the lab.** a product — one per rung or one per warrior's target — gets a repository; its parts are folders of that repository's workspace and publish to crates.io from there. a project that has not become a product is a folder of one repository, `lab`, where the rules are loose on purpose. the kernel stays three repositories because strangers arrive by name; the core merges by product; the neural dialects stay apart because each is a product-sized project of its own.

## 1. the hole

118 repositories, 75 public. on github three of them are visited — cyb, cyber, trident — and 68 of 72 public repositories had fewer than five visitors in two weeks. one boundary change took sixteen pull requests across seventeen repositories; the release train pins eight sibling revisions only because the parts live apart. twelve public repositories are empty seeds. and every repository, product or sketch, carries the same expectations: a readme, a license, a ci, a release — so the sketches look like abandoned products and the products look like sketches.

## 2. the rule

two regimes, and a repository belongs to exactly one:

| | product | lab |
|---|---|---|
| what | a goal kept alive: one rung, one warrior's target, one network | a project that may become a product, or may not |
| repository | its own, the door | a folder of `lab` (public) or `den` (private) |
| release | the train; semver; crates.io from folders | none promised; a tag when useful |
| quality | the gates of its lane; ci green; review by another vendor | builds, or says it does not |
| communication | the product page with dots; changelog; issues as inbox | one readme paragraph and a status line |
| maintenance | kept alive; a stale page is a bug | none promised; a quarter without a push is fine |
| agents | worktree per task, pull request, merge gate | commit to main is allowed |
| history | kept forever | kept; extracted with `git filter-repo` on graduation |

**graduation**: a folder leaves the lab for its own repository when it has a page with at least one green dot, a door — a site, a crate, a binary — and a first release. **the other way**: a product nobody kept alive for two quarters is archived, not demoted; archived keeps its stars and its address.

a part earns its own repository only with its own audience under its own name; the list of such parts is a decision on this page, never a drift.

## 3. the moves

| rung · shelf | repository | absorbs (today's repositories) | stays apart |
|---|---|---|---|
| 1 machine | **nox** | strata, hemera | honeycrisp — its own audience, Apple Silicon |
| 2 weapon | **trident** | — | — |
| 3 warrior | **joy** | lens, zheng, bbg | — |
| 4 brain | **cybergraph** | file, neuron, plumb (tok), tru | — |
| 5 language | **neural** | the twelve seeds → `lab` | rs, nu, inf, eidos, rune, mudra, spark, fs — product-sized each (owner, 2026-10-17) |
| 6 agent | **cy** | glia, soma, seer, sigma, vault; the cy organs still in `cyb` | — |
| 7 network | **cybernode** | the node code out of `soft3/crate` ([[cybernode]]), radio, foculus | — |
| the stack | **soft3** | tade, soft3.js (the SDK face) | — |
| 8 robot | **cyb** | evy, kadek, kern, lytics, mir, prysm, wysm | optica — `graph render for any repo`, its own audience · **owner decides** |
| 9 superintelligence | **cyber** | true-cyber, ctx, uhash | bostrom, space-pussy (archived) — networks |
| 10 state | **cyberia** | cyberia.to (as `site/`), cx, cyberia-meme | blog, cyber-valley, cyberstates — doors kept alive · **see § 4** |
| warriors | one per target | — | all fifteen stay |
| bootloader | — | — | go-cyber, chaingear, space-pussy and the rest: archived monuments, 442 stars kept |
| the lab | **lab** | arc, bt, ren, dif, sym, bel, seq, ten, sec, cloud-forest, cybervalley, cybervalley-io, simulation, li, hub, pure | — |
| the den | **den** (private) | wav, qu, opt, cyberia-capital, cybland, superhuman, mimi, organiq, oxytocin, my | cve, midao, cybernode-ops, the family repositories — private products and the house |

the kernel/core boundary becomes versioned: cybergraph, cy and cybernode depend on `cyber-nox`, `trident-lang`, `cyber-joy` from crates.io, never by path; `release/phase1.toml` loses strata, hemera, lens, nox, zheng, bbg, radio and foculus to their new homes and is reduced to the siblings that remain repositories.

rungs 1–7 go from 45 repositories to 17; the organization's public page from 75 to about 45, of which the bootloader shelf is archived. the number is smaller than a monorepo would give because the dialects and the warriors are products, and products keep their doors.

## 4. cyberia — the one shelf still open

the pages of the state (`cyberia`), its chronicle (`blog`), its genesis city (`cyber-valley`), its sovereignty terminal (`cyberstates`) and its landing (`cyberia.to`) are five public repositories with five deploys. the proposal above keeps three doors — blog, cyber-valley, cyberstates — because each has a live site and a build of its own, and folds the landing and the century index into `cyberia`. the alternative is one repository `cyberia` with `sites/` and one deploy keyed by folder; it needs the optica build to mount folders the way it mounts repositories today (`cyberia-blog/subgraphs.toml`). **owner decides** between the two; the private estate — cve, midao — is untouched either way.

## 5. how a fold is done

1. `git filter-repo` on the absorbed repository to a subdirectory; `git fetch` + `merge --allow-unrelated-histories` into the product; blame survives. cyb first drops its 27 MB `worker.*.js.map` blobs from history (942 MB packed) so every clone stops paying for them.
2. the workspace gains the crate; `cargo publish -p <crate>` from the folder; the crate's `repository` field updates on the next publish.
3. ci filters by path (`paths:`), so a page edit never builds Metal on the Mac runner.
4. the absorbed repository is archived with a one-line readme pointing at the folder; its open issues are transferred to the product (github moves issues inside an organization, not pull requests — open pull requests are closed with a pointer or re-opened against the folder).
5. the ladder file, the registry groups, the subgraph mounts and the sibling pins are updated in the same pull request.

## 6. the order

1. `joy` ← lens, zheng, bbg: the proof, one day, build green, three archives.
2. `nox` ← strata, hemera.
3. `lab` and `den` created; the seeds and the sketches moved in one sweep — the cheapest and most visible cleanup.
4. `cy` ← glia, soma, seer, sigma, vault, and the organs out of cyb.
5. `cybernode` per its own proposal, then ← radio, foculus.
6. `cybergraph` ← file, neuron, plumb, tru; `cyber` ← true-cyber, ctx, uhash.
7. `cyb` ← its parts, history slimmed.
8. the cyberia shelf, after the decision in § 4.

each step is one week of the train at most; a step that leaves a build red is reverted the same day.

## 7. acceptance

- every public repository is a product, a warrior, a network, an archived monument, `lab` or `.github`; `nu registry/registry.nu lint` reports nothing unsorted.
- no `path = "../…"` crosses a repository boundary; kernel crates are consumed from crates.io.
- `release/phase1.toml` names only repositories that still exist.
- every archived repository's readme points at the folder that replaced it; every crate's `repository` field resolves.
- the lab's readme lists its folders with one status line each.

## 8. open, and who decides

| question | who |
|---|---|
| optica: a part of cyb, or its own door | owner |
| the cyberia shelf: three doors or one repository with `sites/` | owner |
| the names `lab` and `den` | owner |
| whether `tade` belongs to the stack (contracts) or to the node | this page says the stack; cybernode's proposal may claim it |
