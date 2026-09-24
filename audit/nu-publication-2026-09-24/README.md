# Public Nu source — 2026-09-24

[cyberia-to/nu](https://github.com/cyberia-to/nu) is public. Its default `main`
revision is [`f3962d5fb`](https://github.com/cyberia-to/nu/commit/f3962d5fbfc25e3a4cbfb673d9d1356730b4700e).
`release/phase1.toml` selects that revision as an owned component.
Cyb receives it through the selected soft3 assembly. This update belongs to
release-train PR #30; existing candidate evidence remains unchanged.

The base is Nushell 0.110.0 at `d40e191d8efad32a174d85b2a45f7ba4796cff84`.
Git history, MIT license, Rust source and Cargo lockfile are preserved. The
published change adds Cyb/soft3 context and replaces upstream release automation
with embedded-library CI for Linux and macOS. The upstream runtime has no patch.

The local workspace additionally contains an uncommitted Sugarloaf renderer,
its workspace/lockfile edits, and a missing test-fixture symlink. Cyb's committed
shell manifest does not consume Sugarloaf. Publication uses the clean upstream
base and preserves the symlink. The original working directories remain intact.

## Verification

Commands, exact source revisions, toolchains and output summaries are in
[checks.json](checks.json). On the published source:

| Check | Result |
| --- | --- |
| Anonymous GitHub API and source download | Public, default branch `main` |
| soft3 origin resolution and clean materialization | Pin matches; all nine embedded libraries resolve at 0.110.0 |
| Locked build and shell smoke test | Passed |
| Upstream formatting and Clippy commands | Passed, no warnings |
| Embedded library tests | 714 passed |
| Full selected-package tests, including integration and documentation | 3,359 passed, 44 upstream ignores |
| Release-engine regression suite, changes committed with this receipt | 35 passed |

## Test prerequisites fixed in soft3

`nu-test-support` launches `target/debug/nu`; a fresh checkout must build that
executable before the library tests. The complete upstream CLI integration
fixtures also expect `plugin-path`, so their test profile enables `nu-cli/plugin`.
The initial runs exposed both prerequisites. The final full-suite command in
`checks.json` passes with them present. Runtime dependency features remain owned
by Cyb's manifest.

`stack-nu-build` now records the executable build. A failed build leaves
`stack-nu` blocked and the receipt red. The regression test verifies this
failure path. The standalone library CI continues to test the default library
feature set; full upstream integration coverage is a separate soft3 gate.
