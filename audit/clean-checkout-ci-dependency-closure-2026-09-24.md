# soft3 clean-checkout CI: the sibling dependency closure

date: 2026-09-24 · revision: 5272f6c7110eecaa3f4acf95503214fb1ae7c76b (origin/main)

## why

three launch #39 PRs on this row (soft3#20, soft3#21, soft3#22) each verified
soft3 against the *local* sibling checkouts under `~/cyber` and named the
same next step: a CI job that clones every sibling fresh at its origin tip
before running `cargo check --tests` / `cargo test`, closing property #39
for real instead of against a working tree that can drift. none of the
three built it, because the job needs the exact list of repos to check out
and none had captured it. this audit captures that list.

## method

```
$ cd crate && cargo metadata --format-version 1 > /tmp/soft3-metadata.json
```

run against the launch worker's mirror (`/tmp/launch-work-b/`, a symlink
farm over `~/cyber`, resolved so every path dependency is visible), then
every package whose `manifest_path` falls under that mirror was grouped by
its top-level directory — one group per sibling git repository — with
`cargo`'s default feature resolution (no `--all-features`), matching the
`cargo check --tests` / `cargo test` invocation soft3's own PRs verify
with.

## the closure

13 sibling repositories, beyond soft3 itself, resolve into the default
build:

| repo | packages it provides |
|---|---|
| bbg | bbg, fjall (vendored) |
| cybergraph | cybergraph |
| foculus | foculus |
| hemera | cyber-hemera |
| inf | inf-ast, inf-eval, inf-lex, inf-parse, inf-plan, inf-source, inf-value |
| lens | cyber-lens, cyber-lens-assayer, cyber-lens-binius, cyber-lens-brakedown, cyber-lens-core, cyber-lens-ikat, cyber-lens-porphyry |
| neuron | neuron-id |
| nox | cyber-nox |
| strata | strata-compute, strata-core, strata-ext, strata-genies, strata-jali, strata-kuro, strata-nebu, strata-proof, strata-trop |
| tade | tade |
| tok | cyber-tok |
| tru | cyber-tru |
| zheng | zheng |

mudra, neuron-model and cyb do not appear: they sit behind the `stack`
feature (`dep:cyb`) and other non-default feature gates in
`crate/Cargo.toml`, so a default-feature clean-checkout gate does not need
them. a future `--features stack` or `--features prove` gate would add
mudra, neuron-model and cyb to this table.

## what a CI job needs

each repo above checked out at its own `origin/<default-branch>` HEAD (not
a pin — this row proves the *current* tips build together) into the
sibling layout soft3's `Cargo.toml` path dependencies expect: `soft3/` and
the 13 repos as siblings under one workspace root, since `crate/Cargo.toml`
reaches them via `../../<repo>[/<subpath>]` (e.g. `../../hemera/rs`,
`../../lens/brakedown`). `actions/checkout@v4` supports this directly, one
step per repo with a matching `path:` and `repository: cyberia-to/<repo>`.

## remains

- the workflow file itself (`.github/workflows/clean-checkout.yml`): 14
  checkout steps, a toolchain step, then `cd crate && cargo check --tests
  && cargo test`
- deciding whether it runs on every push/PR (cheap, ~14 shallow clones) or
  on a schedule, given it duplicates work already done by each sibling's
  own gate
- the same closure recomputed for `--features stack` and `--features
  prove` if those gates are wanted too

## verified

```
$ cd crate && cargo metadata --format-version 1 > /tmp/soft3-metadata.json
$ cargo check --tests
    Finished `dev` profile [unoptimized + debuginfo] target(s)
$ cargo test
test result: ok. 76 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
