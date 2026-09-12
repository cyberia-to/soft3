---
tags: soft3, audit, release
crystal-type: entity
crystal-domain: engineering
date: 2026-09-12
---
# Thread publication audit

Scope: completed work from the Cyber node/storage, execution-model,
Eidos/Hemera and Mudra research thread. Remote refs and GitHub releases were
queried directly. Unrelated local projects and active source changes from
parallel threads are outside the publication snapshot. The latest account/UTXO
brainstorming remains discussion-only.

## Published and reviewable work

| Repository | Published revision / review | Result |
|---|---|---|
| Mudra | `348c46195388ac009667144987511a0e33bdc859` on master | documentation/research prerelease, source archive and SHA-256 |
| Inf | `a92eddff1a92c818dc58c9a19090486a26dcaa1e` on main | complete-input coverage contract pushed |
| Eidos | `585354c1fb9caec5757961fe660f29a2a4973ffa`, [PR 1](https://github.com/cyberia-to/eidos/pull/1) | strict checking and arithmetic certificates pushed; research prerelease and draft integration PR |
| Hemera | `fef673fd8e1bee37e8267d4bac43b88db0e57cfa`, [PR 2](https://github.com/cyberia-to/hemera/pull/2) | structural commitments, models and proofs pushed; research prerelease and draft integration PR |
| BBG | `a25039d54d0671c70c11aed2c9efc8266c371e8b`, [PR 9](https://github.com/cyberia-to/bbg/pull/9) | 13 storage commits pushed; draft coordinated integration |
| Cybergraph | `fa289ea46e7bafe46daea9319ad6531b31ace2cd`, [PR 3](https://github.com/cyberia-to/cybergraph/pull/3) | 4 durable-application/private-retrieval commits pushed; draft |
| Foculus | `27e284cc3ddedb5c10a18166d1e7cf4bc2339bc3`, [PR 5](https://github.com/cyberia-to/foculus/pull/5) | 3 codec/dependency commits pushed; draft |
| Joy | `d065814258f4d2fd86c3132c0435b5a2c99f9727`, [PR 1](https://github.com/cyberia-to/joy/pull/1) | 9 execution/CLI/warrior commits pushed; draft |
| Cyber | `c13607275f8c8a8cd868ed9a4b892f92a36acf22` | already on origin/master, includes the private-retrieval roadmap |
| soft3 | `45b05b9410e72dfda3b9d2caddca39f3b5ef325d` before this audit | already on origin/main, includes execution-model and native acceptance work |
| warriors | `27872352a6e692d5fdf5ffeb53445b6b22ab9ba0` | already on origin/main |

[Mudra specs-2026-09-12](https://github.com/cyberia-to/mudra/releases/tag/specs-2026-09-12)
is a prerelease of specifications and evidence. Production library code,
ordinary tests and Cargo files remain identical to v0.1.0. The release record
links pinned companion specifications and states the remaining qualification
work. It does not release a new recovery implementation or the concurrent
neuron-auth code.

## Research releases and fresh verification

[Eidos research-2026-09-12](https://github.com/cyberia-to/eidos/releases/tag/research-2026-09-12)
ships a source bundle with pinned nox/prysm proof fixtures and an Apple Silicon
CLI. All 165 tests passed both in the isolated worktree and again in the
extracted source bundle. Formatting, strict Clippy and rustdoc passed; the
packaged binary checked both strict examples. The documented public source API
change is boxed `ElabError::TypeMismatch` payloads. General inductive validation,
bounded normalization and universal machine-word refinement remain open.

[Hemera research-2026-09-12](https://github.com/cyberia-to/hemera/releases/tag/research-2026-09-12)
ships source, models, proof evidence and the tested Cargo dependency lock.
Fresh CPU verification passed 273 unit/integration tests and one doctest
(one doctest ignored), strict Clippy, rustdoc and formatting. Eidos checked
21 explicit theorems, rejected 13 negative controls and checked two concrete
Rust multiplication certificates. Model controls and nine manually translated
Z3 integer obligations passed. GPU execution/performance and a new full-profile
attack campaign were outside this snapshot.

Pre-publication review caught and fixed a regression introduced by loop cleanup:
`permute_with_constants` must consume exactly 16 internal constants even when
the public API receives an oversized slice. Two new regression tests fail on
the intermediate commit and pass on the published revision. Source fingerprints
and all CPU/model/Eidos checks were refreshed after the fix.

The RF8/RP16 inverse profile's required security remains unresolved, as does a
full-round break. These prereleases preserve checked functional/research work;
they do not certify cryptographic security or bump/publish Cargo package versions.
Each release contains provenance and SHA-256 checksums. Published tag targets
were verified against the exact commits above; assets were downloaded from
GitHub and checked against their published checksum files.

| Published archive | SHA-256 |
|---|---|
| Mudra specs source | `1585deec3e0d5fbde271723759de5d244b58dc9492eb7ba32ef11e17d63f433e` |
| Eidos source with fixtures | `3fe81f70f53d31a7b982af2b25003c0a697f8d1ade752da1fa7b70d34cab6ef6` |
| Eidos macOS arm64 CLI | `5150ca5521db194bd06253e7af22d4eb12c897a4b43f778524ed926d139ed41e` |
| Hemera source | `ab58ea25d2e29ff086cf478de70b07911f928e0bbaaf063051105d2e2511c44e` |

## Coordinated product release remains open

The current committed repositories do not form the source closure used by the
existing local binaries and earlier integration tests:

1. Cybergraph requires BBG 0.3, Zheng 0.4 and Nox 0.3. Their corresponding
   committed heads still declare 0.2.1, 0.3.3 and 0.2.0; the coordinated
   version/implementation changes are in active working trees.
2. BBG's vendored Fjall repair must survive distribution. Its path dependency
   retains the upstream `fjall` name/version; ordinary registry publication
   resolves the crates.io package and loses the local repair. A complete source
   distribution can carry the vendor; registry distribution needs an explicit
   solution.
3. The committed Inf source adapter requires BBG ^0.1. It does not resolve
   against either the current committed BBG 0.2.1 or the active 0.3.0 tree.
4. The existing `cyber/dist` binary has SHA-256
   `79881675117208d6741e9ec46d983b5f47a2dbb6400f544fa972232585d9c4a5`.
   Its build record includes dirty BBG, Hemera, Lens, Nox, Tok, Tru, Neuron,
   soft3, Cyber and Zheng inputs. A top-level Git tag cannot reproduce it.
5. The coordinated Trident/Trisha/Joy release has separate frozen-candidate
   receipts and later source changes. Its owner must reconcile final source,
   versions, proof formats and installed macOS/Linux artifacts. Old binaries
   cannot be relabeled as builds of newer commits.

The release completion sequence is: settle active source changes with their
owning threads; commit the compatible dependency closure; preserve patched
dependencies in the selected distribution; build and test from that immutable
closure; publish matching source/binaries, checksums and versioned notes.

## Workspace preservation and cleanup

Active changes in Neuron, Mudra neuron-auth, BBG/Cybergraph, Joy/Trident/Trisha,
Lens/Nox/Zheng and soft3 were kept separate from the completed publication
commits. This audit does not claim those working trees are clean.
Inf, Eidos, Hemera, Foculus, Cyber and warriors were clean at the final snapshot;
their active branch heads were published. All six new integration PRs are drafts.

Four obsolete clean documentation worktrees and their merged local branches
were removed after checking remote containment: Cyber private retrieval,
Cybergraph private retrieval, Inf private retrieval and Mudra's older signature
comparison. Main working trees and unpublished legacy branches were preserved.

Older unrelated local commits remain in Cybergraph `docs/foculus-links`,
Foculus and soft3 histories. They were not silently merged or deleted during
this thread-scoped publication pass.
