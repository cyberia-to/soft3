---
title: economy cost — φ* over the burial graphs with genesis weights, 2026-10-17
tags: soft3, chain, audit, economy, tru, benchmark
crystal-type: audit
crystal-domain: cyber
date: 2026-10-17
---

# economy cost — φ* over the burial graphs, 2026-10-17

Evidence for [chain-economy](../../specs/chain-economy.md) §7.2 and §8:
the cost of one cold tri-kernel solve over the full burial graphs with the
genesis weights of chain-economy §4.1, and the shape of those weights.

## setup

- machine: Apple M4 Max, one core, release build, shared machine;
- engine: tru's legacy fixed-point engine (`FocusingGraph::build` +
  `focusing::focusing::iterate` with `derived_steps`, `FocusingParams::default()`,
  `Fx` at 2⁻³²) from the checkout `~/cyber/tru` at `44725bd` (behind
  origin/master by 47 commits; the focusing operators are the ones the
  epoch engine starts from — its own figures replace these when it lands);
- input: `cyberlinks_indexed.csv.gz` of the pussy and bostrom burials
  (`~/.cache/cyber/snapshots/<chain>/`); one tru `Link` per source link,
  `amount = max(1, floor(B(n) / L(n)))` with `B(n)` the author's native
  holdings at halt (`holdings.jsonl` total, the amount genesis credits) and
  `L(n)` the author's link count; `valence 0`, `price = ONE`,
  `Context::none()`; particle ids are a stand-in 32-byte hash of the CID
  string (the cost does not depend on the id function);
- harness: a 60-line binary in `/tmp/econ-spec/phibench` (source below, abridged by the timing print of the derived-step call).

## results

| graph | links | particles | λ_max | steps derived | build | per step | iterate | total | peak RSS |
|---|---|---|---|---|---|---|---|---|---|
| pussy | 29,112 | 15,967 | 10.40 | 24 | 0.141 s | 16.5 ms | 0.395 s | 0.82 s wall | — |
| bostrom | 2,949,732 | 2,914,272 | 2.17 | 19 | 24.3 s | 1.205 s | 22.9 s | 48.1 s wall | 2.78 GB |

No particle received zero focus in either run. `build` includes the
spectral power iterations that estimate λ_max and λ₂ (180 matrix–vector
products).

Projection used by chain-economy §7.2: at a fixed `PHI_STEPS = 48`, one
bostrom solve ≈ 24 + 48 × 1.2 ≈ 82 s; a settlement is two forward solves
and one adjoint, estimated at three solves ≈ 4–5 minutes of one core,
against the 3,600 s of `SETTLE_DELAY` blocks. The adjoint's cost is an
estimate until tru's epoch engine measures it.

## genesis weights

Computed from the same inputs (`/tmp/econ-spec/gw.py`), rule of
chain-economy §4.1:

| chain | links | authors | Σ genesis weight | share of native holdings | frames at the floor of 1 | top author | top ten |
|---|---|---|---|---|---|---|---|
| pussy | 29,112 | 114 | 41,802,483,829,015,337 | 3.51 % | 30 | 15.2 % | 78.6 % |
| bostrom | 2,949,732 | 1,246 | 182,135,071,579,137 | 22.79 % | 278,118 | 39.1 % | 76.6 % |

Concentration of links: the busiest bostrom author made 2,289,526 links
(77.6 %), the top ten 97.0 %; on pussy the top ten made 89.8 %. Resource
tokens: 1,104 of 1,246 bostrom authors and 111 of 114 pussy authors hold
milliampere at halt. Genesis-file counts (bostrom: 2,997,569 `l` frames
including 47,837 passports and 259,435 by legacy authors) differ from the
source-link counts above by the passports; the per-author shares move by
less than a percent.

## harness

```rust
// /tmp/econ-spec/phibench/src/main.rs — Cargo dep: tru = { package = "cyber-tru", path = "~/cyber/tru/rs" }
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::Instant;
use tru::{FocusingGraph, FocusingParams, Context, Link};

fn id(s: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    for k in 0..4u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        k.hash(&mut h); s.hash(&mut h);
        out[(k as usize) * 8..(k as usize) * 8 + 8].copy_from_slice(&h.finish().to_le_bytes());
    }
    out
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (links_csv, bal_csv, denom) = (&a[1], &a[2], &a[3]);
    let mut bal: HashMap<String, u128> = HashMap::new();
    for l in std::fs::read_to_string(bal_csv).unwrap().lines().skip(1) {
        let f: Vec<&str> = l.split(',').collect();
        if f[1] == denom { bal.insert(f[0].to_string(), f[2].parse().unwrap()); }
    }
    let text = std::fs::read_to_string(links_csv).unwrap();
    let mut raw = Vec::new();
    let mut cnt: HashMap<String, u128> = HashMap::new();
    for l in text.lines().skip(1) {
        let f: Vec<&str> = l.split(',').collect();
        *cnt.entry(f[2].to_string()).or_default() += 1;
        raw.push((f[0], f[1], f[2]));
    }
    let links: Vec<Link> = raw.iter().map(|(p, q, n)| {
        let w = (bal.get(*n).copied().unwrap_or(0) / cnt[*n]).max(1);
        Link { neuron: [0; 32], from: id(p), to: id(q), amount: w, valence: 0, price: tru::Fx::ONE }
    }).collect();
    let t0 = Instant::now();
    let g = FocusingGraph::build(links, &Context::none());
    let tb = t0.elapsed();
    let p = FocusingParams::default();
    let steps = tru::focusing::focusing::derived_steps(&g, &p);
    let t2 = Instant::now();
    let r = tru::focusing::focusing::iterate(&g, &p, steps);
    let ti = t2.elapsed();
    let zero = r.focus.iter().filter(|x| **x == tru::Fx::ZERO).count();
    println!("links {} nodes {} lambda_max {:?} steps {} build {:?} iterate {:?} zero-focus {}",
        raw.len(), g.n(), g.lambda_max(), steps, tb, ti, zero);
}
```

Note: the benchmark weights used `balances.csv` (liquid balance) for the
cost run and `holdings.jsonl` totals for the weight table; the cost does not
depend on which, the weight table follows chain-economy §4.1 (the genesis
balance includes returned delegations).

Commands:

```
zcat < ~/.cache/cyber/snapshots/bostrom/cyberlinks_indexed.csv.gz > /tmp/econ-spec/bostrom_links.csv
CARGO_TARGET_DIR=/tmp/econ-spec/target cargo build --release
/usr/bin/time -l /tmp/econ-spec/target/release/phibench /tmp/econ-spec/bostrom_links.csv ~/.cache/cyber/snapshots/bostrom/balances.csv boot
```
