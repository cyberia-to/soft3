---
title: Release train
tags: soft3, release, spec
---

# Release train

Soft3 is the versioned stack assembly. Cyber and Cyb each select one concrete
soft3 build, inherit its component revisions and qualification, then compile
and qualify their own product. `release/phase1.toml` owns stack source pins;
`release/train.py` implements this contract.

## selecting a build

Each product commits `release/soft3.toml`:

| field | binding |
|---|---|
| `repository` | `cyberia-to/soft3` |
| `version`, `revision` | expected soft3 version and full source commit |
| `build` | exact candidate or release identity |
| `release_id` | GitHub release containing that build |
| `checksums_sha256` | digest of its `SHA256SUMS` asset |

The digest authenticates the build's original `candidate.json`, `sources.json`,
`release-validation.json` and `soft3-dependencies.json`. A release name locates
a build; its pinned checksum manifest fixes the bytes. Missing evidence,
changed bytes or an identity mismatch stops consumption. A RED build remains
RED for every consumer, including when all product tests pass.

The workflow implementation has its own immutable Git revision. Updating the
qualification engine preserves the selected stack build. Changing the product's
stack build requires a reviewed change to its contract.

## source assembly

A soft3 cut captures its own origin/default branch and the components in
`phase1.toml`. Cyber and Cyb are downstream consumers. Each product cut captures
only its own default-branch revision and imports stack sources from its selected
build. Advancing a component's origin HEAD leaves an existing build unchanged.

Sources are fetched at those exact commits into a clean sibling layout. Cargo
path dependencies resolve inside this assembly. Products compile the selected
SDK sources with their platform/features and committed Cargo lockfile; soft3's
CLI executable is a separate artifact. The package inventory verifies that
stack-owned crates resolve from the selected sources. Registry/Git substitution
of a stack-owned crate makes `component-bindings` red. External dependencies
remain governed by Cargo.lock.

Owned component pins are compared with origin/default HEAD when soft3 cuts a
new build. An upstream dependency marked `source = "upstream"` instead fixes a
reviewed commit in its upstream default-branch history. Nu is maintained in
`cyberia-to/nu` and follows the owned-component rule: soft3 supplies the pinned
Nushell crates embedded by Cyb. CI's Nushell command runner is an independently
pinned build tool.

A working directory, unpublished source or changed source tree cannot become a
release input. Product builds preserve the stack's original source evidence;
their own source capture records the inherited build contract.

## qualification and evidence

Soft3 runs shared component tests, conformance, its CLI build and boot/status
checks. Products reuse that recorded qualification and run their own integration
and platform gates. Each product receipt includes `soft3-build.json` and
`soft3-build.tar.gz`, carrying the selected pin and original upstream evidence.

| owner | gates |
|---|---|
| shared | `origin-checkouts`, `phase1-pins`, `release-notes-source`, `package-resolution`, `component-bindings`, `source-inputs-unchanged` |
| soft3 | `stack-{hemera,bbg,lens,nox,zheng,cybergraph,foculus,tru,tok,mudra,vault,neuron,file,radio,nu}` |
| soft3 | `soft3-manager`, `conformance-snapshot`, `soft3-tests`, `soft3-release`, `node-status` |
| cyber, cyb | `soft3-dependency`: contract, version and source match; `soft3-build`: inherited qualification |
| cyber | `cyber-format`, `cyber-tests`, `cyber-release`, `cyber-acceptance`, `optica-build`, `protocol-graph` |
| cyb | locked Cargo check/tests, fleet, macOS DMG, Linux release build, Android APK/signature |

Rust warnings fail their gate. Failed prerequisites remain blocked. Every
candidate includes `SHA256SUMS`, `sources.json`, `candidate.json`,
`release-validation.json`, `soft3-dependencies.json` and `soft3-dependencies.md`.
Component declarations, package resolution and gate logs retain distinct roles.
Collection verifies platform hashes and source identities and preserves the
inherited stack verdict. Missing required platforms or binaries keeps it RED.

## operation and authority

Friday 12:00 UTC or manual dispatch cuts `candidate-YYYYMMDD.N`. Products use
the build already selected in their contract. Choosing a newer stack is an
explicit pin update. `cut=false` retains rehearsal evidence. Reruns use a new
candidate number; existing build assets remain unchanged.

The native matrix is macOS ARM64/x64 and Linux ARM64/x64; Cyb adds Android
ARM64. Public soft3 releases are readable by product workflows. A cross-repo
draft requires `SOFT3_READ_TOKEN` with read access to soft3, passed as the
reusable workflow's optional `soft3_read_token` secret. Missing access fails
closed; automation never substitutes a different build.

Verify the engine with
`python3 -m unittest discover -s release -p 'test_*.py' -v`.
Inspect a selected build with
`python3 release/train.py consume --contract <product>/release/soft3.toml --output <new-directory>`.
The command reports the authenticated upstream verdict, including RED.

The owner promotes candidates, pushes version tags, merges version bumps and
publishes packages. Agents create drafts and evidence. Default branches remain
frozen between cut and verdict; fixes go to `release/<date>` and receipts to
`audit/release-<date>/`. Version and build-pin changes follow the reviewed bump
process in [[cyberia/dev]].
