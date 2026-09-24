---
title: Release train
tags: soft3, release, spec
---

# Release train

Soft3 owns qualification of the shared stack. Cyber and Cyb consume its pinned
release contract and add product acceptance. The source manifest is
`release/phase1.toml`; the executable contract is `release/train.py` and the
reusable workflow is `.github/workflows/candidate.yml`.

Each product's `release/soft3.toml` pins the soft3 version and full source
revision. Its workflow references the same immutable revision. This is a release
composition dependency; changes to Rust API dependencies remain with component
integration work. A green product requires a green stack and product gates.

## source contract

Capture the default-branch HEAD of every input from origin, then fetch those
exact revisions into new directories. Compare each sibling with the manifest
pin. A missing remote, pin drift, failed package resolution, modified source or
missing inventory makes the candidate red. Cargo uses committed lockfiles.
Working directories and unpublished feature branches cannot supply release inputs.

`sources.json` records repository URLs, actual default branches and revisions,
expected pins and source availability. `soft3-dependencies.json` includes every
platform's resolved local packages, versions and vendored paths; unresolved
packages retain their error and separately labelled manifest declarations.
`soft3-dependencies.md` makes the inventory readable in GitHub release notes.

`candidate.json` binds the candidate to all three product versions, source
revisions, manager revision and available binaries. `release-validation.json`
contains commands, results and log hashes. Each platform archive contains the
full receipts and logs, covered by `SHA256SUMS`. Collection rejects mixed source
sets, altered artifacts, incomplete inventories and missing platform receipts.

## executable gates

| owner | gates |
|---|---|
| soft3 | `origin-checkouts`, `phase1-pins`, `release-notes-source`, `package-resolution`, `source-inputs-unchanged` |
| soft3 | `stack-{hemera,bbg,lens,nox,zheng,cybergraph,foculus,tru,tok,mudra,vault,neuron,file,radio}`: `cargo test --locked` in each component |
| soft3 | `conformance-snapshot`: `cargo conformance --check`; snapshot validation must be implemented for this gate to pass |
| soft3 | `soft3-tests`, `soft3-release`: locked Cargo tests/build; `node-status`: fresh-home boot and HTTP status |
| cyber, cyb | `soft3-dependency`: source revision, qualifier revision and version match the pinned stack contract |
| cyber | `cyber-tests`, `cyber-release`: locked tests and `nu scripts/release.nu --locked-sources`; `optica-build`, `protocol-graph` |
| cyb | `cyb-check`, `cyb-tests`, `cyb-fleet`: locked check/tests and `make fleet`; platform build gates |
| cyb macOS | `wasm-toolchain`, `trunk-tool`, `cyb-dmg`: install build tools and `make dmg` |
| cyb Linux | `cyb-release`: locked release build |
| cyb Android | `android-build`, `android-signature`: `make android` and APK signature verification |

Warnings from Rust commands make their gate red. A failed prerequisite records
blocked gates. The collector requires every platform and a binary per product
platform; a red stack cannot be hidden by successful product compilation.

## cadence and authority

Friday 12:00 UTC, `Release train` cuts `candidate-YYYYMMDD.N` as a GitHub draft
prerelease in each product repository. Manual dispatch supports an explicit
candidate name and `cut=false` for a rehearsal. Reruns use a new candidate number;
an existing candidate is never overwritten.

The matrix is macOS ARM64/x64 and Linux ARM64/x64. Cyb adds Android ARM64 APK.
Red candidates retain available binaries and failure receipts, including missing
platforms. Binaries are attached inside platform archives. The owner promotes a
candidate, pushes version tags, merges version bumps and publishes packages.
Automation only creates drafts.

From the cut until the owner's verdict, the three default branches are frozen.
Fixes and audit receipts go on `release/<date>`; receipts belong in
`audit/release-<date>/`, and Cyber's launch log records each candidate. Version or
pin changes follow the coordinated bump PR rule in [[cyberia/dev]].

## operation

Run `python3 -m unittest discover -s release -p 'test_*.py' -v` to verify receipt
integrity. Dispatch `.github/workflows/release-train.yml` on the default branch
to qualify origin inputs. Download `assembled-candidate` for the complete local
receipt set. GitHub Releases shows drafts to repository collaborators; publishing
one is an owner action.
