# soft3 clean-checkout build, 2026-09-23

property #39 — "every phase-1 component builds and tests from a clean checkout of
its default branch against the default branches of its siblings". lane B (releases).

soft3 is not among the repos the 2026-09-22/23 sweep found broken (bbg,
cybergraph, foculus, mudra, neuron, nox, prysm, zheng, true-cyber, glia,
honeycrisp, rune, vault, radio). this measurement checks it directly.

## revision

`crate/Cargo.toml` at commit `5272f6c7110eecaa3f4acf95503214fb1ae7c76b`
(`origin/main`), in a worktree with the ten sibling path/git dependencies
resolved against `/Users/master/cyber/<repo>` (see table below).

## commands and results

```
$ cargo check --tests
   Checking soft3 v0.10.0 (/private/tmp/launch-work-b/soft3/crate)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.78s

$ cargo test
test result: ok. 76 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 17.25s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests soft3
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

green: 87 tests, no failures, no dead path dependencies, no version pin
behind a checked-out sibling.

## what this does not prove

the ten siblings resolved above are the checkouts at
`/Users/master/cyber/<repo>`, not fresh clones of `origin`'s default
branch tip. none carries uncommitted changes (verified with `git status
--porcelain`, two false positives from stray self-referential symlinks
excluded), but seven of the ten sit behind their own `origin` default
branch, because today's merge wave (bbg #15, the seven soft3 coverage
PRs, etc.) landed after these checkouts were last fetched-and-merged
locally:

| sibling | local HEAD | origin default HEAD | at tip |
|---|---|---|---|
| tru | 8941629 | 8941629 | yes |
| inf | bf37987 | bf37987 | yes |
| cybergraph | 0eae7ae | aa374ce | no |
| foculus | 9e32aa6 | 177fad4 | no |
| bbg | c75b685 | a3c9a0c | no |
| zheng | 3db5904 | 44bd5bf | no |
| nox | bf83009 | 2612673 | no |
| tade | c1d81c7 | b1f1c2c | no |
| lens | b921e9e | 392abe7 | no |
| tok (repo `plumb`, local dir `tok`) | e27da0d | e27da0d | yes |

a true clean-checkout gate needs fresh worktrees of every sibling at its
own `origin` default-branch tip, not the owner's last-fetched local
checkout. that is the CI gate this property still needs; today's result
is evidence soft3 itself carries no defect, not evidence the gate is
built.

## remains

- build the clean-checkout CI gate itself (fresh clone of soft3 +
  fresh clones of every path-dependency sibling at its `origin` default
  branch tip, run `cargo check --tests && cargo test`) — this is the
  piece property #39 is still open on for every repo, soft3 included
- re-run this measurement once the seven lagging siblings above have
  merged their pending `launch #39`/`launch #40` PRs, to confirm soft3
  still resolves clean against their new tips
