# Soft3 candidate qualification — 2026-09-24

Verdict: RED. The full macOS/Linux ARM64/x64 matrix and stack qualification ran
in [Actions run 35975316739](https://github.com/cyberia-to/soft3/actions/runs/35975316739).
[GitHub draft candidate](https://github.com/cyberia-to/soft3/releases)
retains the generated assets. No executable was produced; no version tag or
public promotion was performed.

Source and qualifier: `c5cc2077df510610abb7b5bd69675af6048ed724`. All product versions and
sibling revisions are in `candidate.json`. `release-validation.json` records
each command and result. The platform archives preserve logs and their hashes.

Observed blockers include unavailable Nu origin, absent or stale committed
Cargo lockfiles, cybergraph's missing `local-storage` feature, and the absent
`cargo conformance --check` implementation. See command logs for each component;
the audit does not treat these failures as successful qualification.

The engine's 13 integrity tests passed at the same source revision:
`python3 -m unittest discover -s release -p 'test_*.py' -v`.
Output is retained in `engine-tests.log`.

Acquisition and draft creation:

```sh
gh workflow run release-train.yml --repo cyberia-to/soft3 --ref main -f candidate=candidate-20260924.1 -f cut=false
gh run download 35975316739 --repo cyberia-to/soft3 --name assembled-candidate --dir /tmp/soft3-final-assembled
python3 release/train.py draft --output /tmp/soft3-final-assembled
```

The last command used the same qualifier from a clean origin checkout.
`SHA256SUMS` is the generated release asset inventory; this README and local
engine test log are additional audit context. All recorded asset hashes were
verified. The default branches are frozen pending owner verdict; this receipt
is committed only on `release/2026-09-24`.

## Component presentation trace

`release-page.md` is the current draft presentation. `component-inputs.json`
traces the product's own manifest declarations from the exact captured Git
objects. It retains package names, workspace versions, optional/target/test
conditions, registry requirements, manifest hashes and source links. Registry
ownership is not a resolved dependency version. Missing declarations remain
explicit; the original candidate assets and qualification results are unchanged.

Generator: [soft3 `712af9f3`](https://github.com/cyberia-to/soft3/commit/712af9f302b999ccd848f3467bf471b25caa306d).
The generator's 18 integrity and declaration tests passed with:
`python3 -m unittest discover -s release -p 'test_*.py' -v`.

Reproduce using that generator revision and the captured origin source trees:

```sh
python3 /tmp/cyber-train-20260924/audit-soft3/release/train_components.py --sources /tmp/cyber-train-20260924/audit-soft3/audit/release-2026-09-24/sources.json --checkout /tmp/cyb-origin-candidate/sources --output /tmp/cyber-train-20260924/audit-soft3/audit/release-2026-09-24/component-inputs.json
python3 /tmp/cyber-train-20260924/audit-soft3/release/train_notes.py --directory /tmp/cyber-train-20260924/audit-soft3/audit/release-2026-09-24 --audit-url https://github.com/cyberia-to/soft3/blob/release/2026-09-24/audit/release-2026-09-24 --output /tmp/cyber-train-20260924/audit-soft3/audit/release-2026-09-24/release-page.md --run-url https://github.com/cyberia-to/soft3/actions/runs/35975316739
```

`presentation-verification.json` records the subsequent GitHub Markdown render,
draft update and unchanged asset digests. The full train inventory remains in
`soft3-dependencies.json`; the release page scopes its component table to this
product. Soft3's table covers its phase-1 qualification inventory and missing
inputs referenced by its manifests, excluding the downstream products.
