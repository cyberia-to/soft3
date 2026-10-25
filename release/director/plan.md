---
title: the director's plan — phase 1
tags: soft3, release, director, plan, phase 1
crystal-type: process
crystal-domain: cyber
status: open
date: 2026-10-25
serves: release/director/README.md § the goal
---
# the director's plan — phase 1

the goal: a green candidate of soft3 from origin default branches, every gate of `specs/releases.md` green, `release/phase1.toml` pinned to what was built, the owner's promotion the only step left. this page is the director's plan; the owner corrects it by editing it. one session takes one unit: the first open row below, or the reddest gate when the stack is red.

## what phase 1 is

phase 1 is defined in `cyber/launch.md` § phase 1, not here: **the bootloader reborn** — pussy and bostrom as two soft3 proof-of-work chains, genesis 2026-11-05 (bostrom 13:22:42 UTC, pussy 13:37:00 UTC), the cores in consensus at genesis per `specs/chain-economy.md` (decision 2026-10-17), the material hemera 0.4 / profile #15 (decision 2026-10-16), the pool's formats frozen and the pool off (`specs/chain.md` §11). the code spec is `specs/chain.md`, `specs/chain-wire.md`, `specs/chain-conformance.md`, `specs/genesis.md`, `specs/chain-economy.md`.

the calendar that binds this plan (`cyber/launch.md`, decision 2026-10-17):

| date (UTC) | what |
|---|---|
| until 2026-11-02 00:00 | anything may change: specs, formats, parameters, genesis files |
| 2026-11-02 → 11-05 | code and feature freeze: only security and bug fixes with a failing test merge |
| 2026-11-05 | genesis of both chains |

so every row of "the order" below that is not a fix must merge before 2026-11-02 00:00 UTC. what does not fit is cut from phase 1, never squeezed (`cyber/launch.md` § the phases).

## where the stack stands — receipts

the last train run: `Release train` run 40098866000 (2026-10-24 13:36 UTC, `cut=false`), all five legs red. receipts: the run's `candidate-*` artifacts (on agent-a in `~/director/state/run-40098866000/`). stack leg:

| gate | result | first error (from `logs/<gate>.txt`) |
|---|---|---|
| origin-checkouts, source-inputs-unchanged | green | |
| phase1-pins | red | drift on 25: bbg cybergraph evy file foculus glia hemera honeycrisp inf lens mir mudra neuron nox nu optica radio rune soma spark strata tade tok tru zheng |
| release-notes-source | red | `cyb: captured product HEAD has no merged pull request` |
| package-resolution, soft3-tests, soft3-release | red | `failed to load manifest for dependency cyber-lens-gpu … lens/gpu/Cargo.toml: No such file` |
| node-status | blocked | release executable was not built |
| stack-hemera, lens, nox, zheng, tru, tok, mudra, file | green | |
| stack-bbg | red | `failed to select a version for the requirement cyber-lens = "^0.2.0"` |
| stack-cybergraph, stack-foculus | red | `failed to select a version for the requirement bbg = "^0.2"` |
| stack-vault, stack-neuron | red | `failed to select a version for cybergraph` |
| stack-radio | red | `address_lookup::mdns` 2 tests FAILED (93 passed) — multicast on the runner |
| conformance-snapshot | red | `failed to select a version for the requirement cyber-hemera = "^0.4"` |
| stack-chain-genesis | blocked | `no pussy genesis file at ~/.cache/cyber/genesis/pussy.tade (set SOFT3_GENESIS_DIR)` |
| simulation-gate | blocked | `no simulation checkout: pin cyberia-to/simulation in release/phase1.toml` |

the root cause of most of the red: **the 0.4 line lives off the default branches.** soft3 `main` already requires cyber-hemera ^0.4, cyber-lens ^0.2 with `lens/gpu`, nox ^0.3, zheng ^0.4, bbg ^0.3; the train reads default-branch HEADs only (`release/train_sources.py`), and those carry the older line. measured 2026-10-25 through the GitHub API (crate `version` of each manifest, `compare` of line against default):

| repo | default | crate on default | line | crate on line | line ahead / behind |
|---|---|---|---|---|---|
| hemera | main | 0.3.1 | release/0.4 | 0.4.0 | +25 / −2, 72 files |
| lens | master | 0.1.3 (`src/`), no `gpu/` | release/0.2 | 0.2.0, `gpu/` 0.1.1 | +77 / −6, 97 files |
| nox | master | 0.2.0 | release/0.4 | 0.3.0 | +28 / −30, 255 files |
| zheng | master | 0.3.3 | release/0.4 | 0.4.0 | +244 / −12, 300 files |
| joy | main | — | release/0.4 | — | +124 / −3 |
| bbg | master | 0.3.0, wants lens ^0.2 | — | | |
| cybergraph, foculus | master | want bbg ^0.2, hemera ^0.3, lens ^0.1 | — | | |

the branch name of a line is not its crate version: nox `release/0.4` carries crate 0.3.0, which is what soft3 asks for.

PR gates today: soft3 runs `release-engine.yml` and `components.yml` on `pull_request`; hemera, lens, tru and radio have a `ci.yml`; nox, zheng, joy, bbg, cybergraph, foculus, plumb, mudra, vault, neuron, file, tade, strata have none. nothing checks a sibling PR against the candidate graph.

## decisions

| # | decision | reason |
|---|---|---|
| D1 | land each release line on its default branch by an ordinary merge PR (release line → default), in dependency order; never switch a repository's default branch, never teach the train to read a non-default branch. a landing PR into a default branch is merged only after the owner approves it (branch protection on the 30 public stack defaults, owner 2026-10-25, `release/director/README.md` § the powers); the director opens it with the receipts and lists it under § what only the owner does | the doctrine defines a candidate's inputs as default-branch HEADs (`~/cyber/AGENTS.md` § release train); a merge is reversible by revert and is inside the director's power; switching defaults retargets every open PR and strands the commits only on the default. asked fable 2026-10-25 (`ask-fable`, #1 of the day): same verdict |
| D2 | before each landing PR, merge the default into the line on a task branch and resolve the conflicts there, so the landing PR itself is conflict-free | conflicts are paid once, on the line, where the line's owner work lives |
| D3 | "gates green" for a landing or bump PR means: the repository's own build and tests on the PR head, and a train rehearsal (`cut=false`) in which that sibling is taken at the PR head and every gate it touches is green on all five legs | a per-repository green proves nothing about the candidate graph; the rehearsal is a qualification, never a candidate, and refuses `cut=true` |
| D4 | the old owner version PRs zheng#33, lens#14, nox#17 are superseded by the landings and are closed with a comment naming what replaced them, after checking their fixes are present on the line | they are conflicting alternative lines of the same versions |
| D5 | the proposals that are not in phase 1 are not worked on before 2026-11-02 (table below) | `cyber/launch.md`: what does not fit the date is cut, never squeezed |
| D6 | lane of every PR by the files it touches (`cyberia/processes/deliver.md` § lanes): contained → green checks; wide (a contract, a shared crate, a release line landing) → green checks and a clean trajectory; a PR that changes a specification of rungs 1–7 (`specs/` of soft3 or of a sibling) is wide and passes the owner's hand: labelled `decision`, not merged by the director; implementation of an accepted spec is not a spec change; irreversible → the owner. labels `contained`, `wide`, `irreversible` beside the queue labels | the lane is computed, never declared; deliver.md § lanes |
| D7 | pin and version bumps are merged by the director when green — into `release/phase1`; they reach a default branch only inside the owner-approved `release/phase1 → <default>` PR | `~/cyber/AGENTS.md` § release train, the product director (owner 2026-10-25): "the director merges any PR whose gates are green, bumps and pins included" — this supersedes the older "owner merges" wording in #242 and #195 |
| D8 | the freeze: once a candidate is cut green, nothing merges into the soft3, cyb and cyber defaults until the owner's verdict; fixes go to `release/<date>` as `fix:` commits; a red candidate freezes nothing. so R1's cut is taken only when the phase-1 rows above it are done | `~/cyber/AGENTS.md` § release train |
| D9 | soft3 work integrates on soft3 `release/phase1`: the director's soft3 PRs (G2, G5, G6, C2, C3) and the bumps (#242, #195's successor) target `release/phase1`, are merged there when green, and reach `main` by one `release/phase1 → main` PR with the receipts, approved by the owner. PRs already opened against `main` (#242, #246) are retargeted to `release/phase1` after review | `main` is protected; the director's power is the release branch (`release/director/README.md` § the powers). the train still reads defaults, so the rehearsal of G2 is what makes the release-branch state checkable before the owner's approval |

## the proposals against phase 1

| proposal | phase 1 needs | built | remains for phase 1 |
|---|---|---|---|
| [proof-system-repair](../../proposals/proof-system-repair.md) | the material only: hemera 0.4 (row 2h decided) and the zheng/lens 0.4 lines on defaults so the stack resolves; full nodes validate natively (launch decision 2026-10-15), so repair phases 1–5 and soft3 0.11 leave the critical path | lines on `release/*` | L1–L4 |
| [privacy-frozen](../../proposals/privacy-frozen.md) | the pool's formats frozen in the genesis params, the pool off (rule set 2) | `specs/chain.md` §11, `audit/chain/pool-formats-2026-10-16.md` | covered by conformance (G6) |
| [proven-present](../../proposals/proven-present.md) | S0 only: the node writes the conformance corpus from height 0, with a generator of invalid blocks | not verified this session | C2 |
| [cybernode](../../proposals/cybernode.md) | provisionally nothing: §6 targets C0–C3 at the 2026-10-30 candidate and ships genesis from the soft3 crate unchanged if C2 is not green by then; the owner chooses (§7.3). this plan does not start C0–C3 before the stack is green, so unless the owner says otherwise the trigger will be met | — | the owner's choice at the 10-30 candidate |
| [network-repos](../../proposals/network-repos.md) | provisionally nothing: §5 targets N1–N3 at the 2026-10-30 candidate; if they are not green, genesis ships from today's pipeline and N1–N3 record the same particle afterwards; the owner chooses | — | the owner's choice at the 10-30 candidate |
| [metabolic-oracle](../../proposals/metabolic-oracle.md) | its §5 step 3 would put a `metabolic` epoch record and the weights vote in `chain-economy.md` at launch with genesis w = (0,1,0); `chain-economy.md` on `main` has no such record today, and the proposal (2026-10-23) is `proposed`, not accepted; moving it into rule set 1 is a spec of rung 7 and the owner's hand | — | not started by this plan; enters phase 1 only if the owner accepts it before 2026-11-02 |
| [steps-blocks-epochs](../../proposals/steps-blocks-epochs.md), [goldilocks-parano1d](../../proposals/goldilocks-parano1d.md) | nothing new: their phase-1 descendant is `specs/chain.md` (launch decision 2026-10-15) | chain on soft3 main | — |
| [rooted-civilization](../../proposals/rooted-civilization.md) | nothing for the candidate: phase 0 is the site, A–C follow the chain | — | after genesis |
| [term](../../proposals/term.md), [network-planes](../../proposals/network-planes.md), [tade-one-exchange](../../proposals/tade-one-exchange.md), [component-page-canon](../../proposals/component-page-canon.md) | nothing | — | after genesis |
| [vault-secret-custody](../../proposals/vault-secret-custody.md) (superseded), [cybergraph-sync-tade-architecture](../../proposals/cybergraph-sync-tade-architecture.md) (implemented) | — | — | — |

## the order

status: `open → doing → done`, or `closed`. each row is one unit; its acceptance is a receipt.

| # | unit | repo · lane | acceptance | status |
|---|---|---|---|---|
| G0 | this plan | soft3 · contained | #248 merged on `release/phase1` | done 2026-10-25 |
| B0 | merging into `release/phase1` was refused by an organization ruleset that required a review the director cannot give itself (first session). by 23:36 UTC the owner had lifted it: #248 `mergeStateStatus: CLEAN`, `reviewDecision` empty, `rules/branches/release/phase1` `[]` | org · irreversible lane (access) | #248 merges | done 2026-10-25 |
| G1 | lane labels `contained`, `wide`, `irreversible` in every repository of `release/components.toml` that has open PRs; every open soft3 PR carries its lane | all · contained | `gh api repos/<o>/<r>/labels` lists them; no open soft3 PR without a lane label | done 2026-10-25 — labels in 26 repositories (bbg cyb cybergraph evy file foculus glia hemera honeycrisp lens mir mudra neuron nox nu optica prysm radio rune soft3 soma spark tru vault zheng); 19/19 open soft3 PRs laned (contained: #167 #170 #200 #220 #234 #245; wide: #3 #19 #30 #34 #169 #181 #195 #212 #217 #242 #243 #246 #247); trident, wysm, cyberia-my, eidos, lytics, cybernode refuse the write (`permissions.push=false`) — see § what only the owner does |
| G2 | the sibling rehearsal (D3; if `specs/releases.md` must change, that part waits for the owner, D6): `release-train.yml` dispatch input `override` (`<sibling>=<ref>`, repeatable), honoured only with `cut=false`, recorded in `sources.json` as `override`; unit test that `cut=true` with an override fails | soft3 · wide | `python3 -m unittest discover -s release -p 'test_*.py' -v` green on the PR (`release-engine.yml`); a dispatched rehearsal with `lens=release/0.2` whose `sources.json` names the override | done 2026-10-25 — #250 merged on `release/phase1` (`a177b0a`), `Release engine / test` run 40246344403: 27 tests OK (9 new `RehearsalTests`); rehearsal `Release train` run 40246391619 (`release/phase1`, `cut=false`, `override=lens=release/0.2`): `sources.json` has `rehearsal: true`, lens `override: {ref: release/0.2, revision: b1040c2, default_branch: master, default_revision: 8b26cef}`; the `releases.md` sentence on rehearsal overrides is left to the owner (D6) |
| G3 | the daily train: review #246 (cron `0 12 * * *`, owner 2026-10-24), retarget to `release/phase1` (D9), merge there; it takes effect when the `release/phase1 → main` PR is approved, since scheduled workflows run from the default branch | soft3 · wide | merged on `release/phase1`; after the owner's approval, the next 12:00 UTC run appears in `gh run list --workflow "Release train"` | done on `release/phase1` 2026-10-25 — #246 reviewed (cron `0 12 * * 5` → `0 12 * * *` only; `git merge-tree` with `ce703e6` clean; scheduled runs keep `cut=true`, no override), retargeted by REST, `Release engine / test` SUCCESS (run 40096560433), merged `4fdd704`; workflow `active`, no invalid-file run on the push. the 12:00 run itself waits on the owner's approval of `release/phase1 → main`. `~/cyber/AGENTS.md:14` still says friday (owner's doctrine) |
| L1 | hemera: default into `release/0.4`, then `release/0.4` → `main` | hemera · wide | hemera `ci.yml` green on the PR; rehearsal with `hemera=<PR head>`: stack-hemera green, no new red; merged | open |
| L2 | lens: default into `release/0.2` (the lens#36 branch), lens#31 first if the rehearsal needs it, then lens#36 → master | lens · wide | lens `ci.yml` green; rehearsal with `hemera`, `lens` at heads: package-resolution no longer fails on `lens/gpu`, stack-bbg resolves; merged | open |
| L3 | nox, zheng, joy: default into each `release/0.4` (nox#31, zheng#69 first if required), then line → default; a minimal `pull_request` CI (`cargo test --locked`) shipped inside each landing PR | nox, zheng, joy · wide | each CI green on its PR; rehearsal with the three at heads: stack-nox, stack-zheng green, conformance-snapshot resolves | open |
| L4 | consumers onto the landed lines: cybergraph, foculus (bbg ^0.3, hemera ^0.4, lens ^0.2), then mudra, vault, neuron re-resolve; lockfiles committed | cybergraph, foculus, mudra, vault, neuron · wide | stack-cybergraph, foculus, mudra, vault, neuron green in a rehearsal | open |
| L5 | close zheng#33, lens#14, nox#17 as superseded (D4) | zheng, lens, nox · contained | each closed with a comment naming its replacement and the commits checked | open |
| G4 | stack-radio: the two `address_lookup::mdns` tests need multicast the runners lack; gate them on an environment the train sets, or mark them ignored in CI with the reason, in radio | radio · contained | stack-radio green on all legs that run it | open |
| G5 | stack-chain-genesis: the train builds or fetches the pussy and bostrom genesis files into `SOFT3_GENESIS_DIR` from the pinned burial sources, checked against the particles of `conformance/chain-genesis.toml` | soft3 · wide | stack-chain-genesis green, not blocked | open |
| G6 | conformance-snapshot green on the landed lines; drift blessed in the PR that causes it (`conformance/specs/README.md`) | soft3 · wide | conformance-snapshot green | open |
| G7 | simulation-gate: review #242 (pin cyberia-to/simulation; D7), retarget to `release/phase1` (D9), merge there | soft3 · wide | simulation-gate runs; its expected reds are named, not failures (`specs/releases.md`) | open |
| G8 | release-notes-source: find why cyb's HEAD has no merged PR (a direct push) and fix the gate or the input | soft3 or cyb · contained | release-notes-source green | open |
| G9 | phase1-pins: after L1–L4, a version bump is one PR per component (`chore: <component> <version>`, AGENTS.md); pins that move without a version change go in one `release/phase1.toml` PR to the qualified HEADs; supersedes #195 (D7) | soft3 · wide | phase1-pins green; #195 closed | open |
| C1 | the soft3 PR queue: #234 (economy findings red until the owner's spec decisions), #167, #169, #170, #212, #243, #245 triaged against this plan — merged when green and in phase 1, labelled and left otherwise | soft3 · per PR | every open soft3 PR has a lane and a line here or in the log | open |
| C2 | proven-present S0: verify the node writes the conformance corpus from height 0 and generates invalid blocks; if not, build it | soft3 · wide | a test that writes the corpus for a mined chain and replays every generated invalid block to its native verdict | open |
| C3 | the economy at genesis: the conformance vectors of registry rows 26 (passive stake earns 0), 27 (θ split), 41 (epoch record agreed) run in soft3-tests | soft3 · wide | the named vectors pass in soft3-tests on all legs | open |
| R1 | the candidate: a dispatched `cut=false` run green on all five legs from origin defaults, then the 12:00 cut of the same inputs | soft3 | `release-validation.json` green on every leg; the draft `candidate-YYYYMMDD.N` exists; the owner's promotion the only step left | open |

reading the order: G2 makes "green" checkable for every sibling PR; L1 → L2 → L3 → L4 is the dependency order of the 0.4 line (leaves first); G4–G8 are independent and may interleave whenever a gate is the reddest; G9 comes after the landings; R1 closes the plan.

## what only the owner does

approve each `release/* → <default>` PR the director opens with receipts (the landings L1–L3 and the `release/phase1 → main` PR of soft3; none open yet) — this is the rule working, not a blocker; promote a candidate, push a tag, publish to crates.io, run `make ship`; the irreversible lane (production chain, keys, data, money); a change to the charter; access: `cyberia-apparatus` has no push on trident (a component of rung 2, 3 open PRs), wysm, cyberia-my, eidos, lytics (cybernode is read-only by design) — the director cannot label or merge there until the owner grants it, and does not need to before a phase-1 row touches them. open owner questions this plan meets but does not answer: cybernode §7.3 and network-repos §5 (before or after genesis — decided at the 10-30 candidate; this plan does not start them first), whether the metabolic oracle enters rule set 1, the `chain-economy.md` §12 questions behind #234.

## log

| date | session | moved |
|---|---|---|
| 2026-10-25 | first director session | plan written from the doctrine, the proposals, `cyber/launch.md` and run 40098866000; D1 asked of fable (#1); PR #248 opened, gate `test` green, merge refused by the org's required-review ruleset (B0); second-model review returned five defects (owner questions settled early, metabolic-oracle misread, the owner's hand on specs, the bump-merge clause, the freeze) — fixed as D6–D8 and the proposal rows |
| 2026-10-25 23:36 | director session 2 | B0 found lifted (#248 `CLEAN`); review of #248 against the 2026-10-25 README: G0's acceptance named `main` instead of `release/phase1`; D1, D7 and G3/G7 assumed the director merges into protected defaults — added D9 (soft3 integrates on `release/phase1`, defaults by owner-approved PR), retargeting of #242/#246, the owner's approvals under § what only the owner does; #248 merged. next: G1 |
| 2026-10-25 23:38 | director session 3 | G1 done: lane labels created by REST in 26 repositories; the 19 open soft3 PRs laned by the files they touch (D6) — specs/pins/workflow/doctrine/deck/deploy wide, proposals/audit/tests-only contained; no push on trident, wysm, cyberia-my, eidos, lytics, cybernode (recorded under the owner). next: G2 |
| 2026-10-25 23:44 | director session 4 | G2 done: `train.py snapshot --override NAME=REF` (branch, `pull/N/head`, commit) + `--cut`; refused with `cut=true`, never drafted, notes marked REHEARSAL; dispatch input `override` on `release-train.yml` → `candidate.yml`; #250 merged after review, rehearsal 40246391619 recorded the lens override. Since 23:48 the rehearsal's five legs have been qualifying lens at `release/0.2` against the defaults; L2 reads its receipts (expect `lens/gpu` found; hemera ^0.4 still off the default). How to dispatch from now on: `gh workflow run release-train.yml --ref release/phase1 -f cut=false -f candidate=candidate-YYYYMMDD.9N -f override='<sibling>=<ref> …'`. next: G3 |
| 2026-10-25 23:52 | director session 5 | G3 done on `release/phase1`: #246 (the owner's daily cron) reviewed, retargeted from `main` (D9), merged `4fdd704`. rehearsal 40246391619 (lens=`release/0.2`) still in progress — its receipts feed L1/L2. next: L1 (hemera `main` into `release/0.4`, then the landing PR for the owner) |
