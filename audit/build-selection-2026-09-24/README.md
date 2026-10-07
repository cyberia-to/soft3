# Pinned soft3 build consumption and Nu provenance

Engine implementation: [soft3 `838450ae`](https://github.com/cyberia-to/soft3/commit/838450ae972740fd2564319bb23ab869d79d92dd).

`python3 -m unittest discover -s release -p 'test_*.py' -v` passes 34 tests.
These cover immutable build identity, source/checksum substitution, registry/Git
escape by stack-owned crates, default-branch upstream history, product-only
origin capture and inherited RED qualification with otherwise green product
receipts. Synthetic test binaries are fixtures. Actionlint v1.7.12 passes the
shared candidate, release-train and release-engine workflows with
`actionlint -shellcheck= <workflow-paths>`.

## Actual selected build

[selected-build.json](selected-build.json) records the real GitHub asset download
and pinned digest validation for soft3 `candidate-20260924.1`, source
`c5cc2077df510610abb7b5bd69675af6048ed724`. Consumption preserves its RED verdict. The original candidate
assets remain unchanged; the new Nu input belongs to a subsequent soft3 build.
Both product contracts select the same release ID and checksum manifest.

## Actual Nu input

[nu-origin.json](nu-origin.json) records a fresh origin-only checkout of Nushell
`d40e191d8efad32a174d85b2a45f7ba4796cff84` and successful
`cargo metadata --no-deps --locked --format-version 1` in that checkout. All nine
embedded packages are declared at version 0.110.0. This is source/manifest
verification; compilation and runtime acceptance remain release gates.

The local `~/cyber/nu/.git` says
`gitdir: ../../.git/modules/vendor/nushell`. Its surviving Git metadata is at
`~/cyber/cyb/.git/modules/vendor/nushell`; it still names upstream
`https://github.com/nushell/nushell.git` and the same commit. Read-only inspection:

```sh
GIT_OPTIONAL_LOCKS=0 git --git-dir=$HOME/cyber/cyb/.git/modules/vendor/nushell --work-tree=$HOME/cyber/nu log -3 --oneline
GIT_OPTIONAL_LOCKS=0 git --git-dir=$HOME/cyber/cyb/.git/modules/vendor/nushell --work-tree=$HOME/cyber/nu status --short
GIT_OPTIONAL_LOCKS=0 git --git-dir=$HOME/cyber/cyb/.git/modules/vendor/nushell --work-tree=$HOME/cyber/nu diff -- Cargo.toml
```

Observed local changes are a Sugarloaf workspace member and its lock updates,
plus a removed test symlink. Cyb source `6c20559206d90f0474379e48706ba00db9630a37`
links the nine Nu crates from `../../nu/crates/`; its Cargo manifests contain no
Sugarloaf dependency. `git ls-tree HEAD vendor/nushell` has no gitlink, while
`.gitmodules` still advertises the former submodule. The change removes that
stale descriptor from the product branch. Owner working trees remain intact.

The CI command runner is separately pinned at Nu 0.112.2 in the captured
workflow. The embedded libraries are selected by the soft3 assembly.
