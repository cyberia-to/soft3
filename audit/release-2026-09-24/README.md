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
