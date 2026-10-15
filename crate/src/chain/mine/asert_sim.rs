//! ASERT (chain.md §1.5) under a 100× hash-rate step and collapse — the
//! miner's audit question, simulated on the kernel's integer `daa::asert`
//! (`audit/chain/mine-2026-10-15.md` §3). No randomness, no floats.

#![cfg(test)]

use super::target::{pow2, Words};
use crate::chain::daa::{asert, Anchor};
use crate::chain::header::encode_bits;
use crate::chain::u256::U256;

fn anchor(g: &Words) -> Anchor {
    Anchor {
        target: U256(*g),
        time: 0,
    }
}

/// chain-conformance §4, behavioral: a chain at T-spacing holds bits;
/// T/2 blocks for 20 blocks halve the target (factor in [1.9, 2.1])
/// after 600 s of block time; a 100× slowdown recovers the T interval
/// within 70 minutes of simulated wall time. Hash rate is simulated
/// deterministically: a block takes `expected_hashes / rate` seconds
/// (integer seconds, rounded to nearest), where
/// `expected_hashes = 2^256/(target+1)`; no randomness, no floats.
fn simulate(genesis_target: &Words, steps: &[(u64, u64)]) -> Vec<(u64, i64, Words)> {
    // steps: (blocks, hash rate in H/s) phases; returns (height, time, target of the next block)
    let mut out = Vec::new();
    let (mut h, mut t) = (0u64, 0i64);
    let mut target = *genesis_target;
    for &(blocks, rate) in steps {
        for _ in 0..blocks {
            // seconds a block at `target` and `rate`: 2^256 / (target+1) / rate, nearest
            let exp = expected_hashes(&target);
            let secs = ((exp + rate as u128 / 2) / rate as u128) as i64;
            t += secs.max(1);
            h += 1;
            let a = asert(anchor(genesis_target), h, t as u64);
            target = a.target.0;
            out.push((h, t, target));
        }
    }
    out
}

/// `2^256 / (target + 1)` for targets of the form `2^k` or close: via
/// the leading 128 bits (test accuracy, not consensus).
fn expected_hashes(target: &Words) -> u128 {
    // work ≈ 2^256 / target: shift so the target fits 64 bits
    let lz = {
        let mut z = 0;
        for w in target.iter().rev() {
            if *w == 0 {
                z += 64
            } else {
                z += w.leading_zeros();
                break;
            }
        }
        z
    };
    let bits = 256 - lz; // target in [2^(bits-1), 2^bits)
    let top = if bits > 64 {
        U256(*target).shr(bits - 64).0[0]
    } else {
        target[0]
    };
    // 2^256/target = 2^(256-(bits-64)) / top = 2^(320-bits)/top
    let num_shift = 320 - bits;
    if num_shift >= 128 {
        ((1u128 << 127) / top as u128) << (num_shift - 127)
    } else {
        (1u128 << num_shift) / top as u128
    }
}

#[test]
fn asert_behaviour_and_hundredfold_step() {
    let g = pow2(228);
    let base_rate: u64 = (expected_hashes(&g) / 30) as u64; // exactly T-spacing at 2^228
                                                            // 1. T-spacing holds bits
    let steady = simulate(&g, &[(200, base_rate)]);
    for (h, _, target) in &steady {
        assert_eq!(encode_bits(U256(*target)), 0x1d10_0000, "height {h}");
    }
    // 2. blocks at T/2 spacing: after 600 s of block time (40 blocks at
    //    15 s, the chain 600 s = one half-life ahead) the target has
    //    halved — factor in [1.9, 2.1]
    let a = asert(anchor(&g), 40, 600);
    let ratio_x10 = expected_hashes(&a.target.0) * 10 / expected_hashes(&g);
    assert!(
        (19..=21).contains(&ratio_x10),
        "work ratio ×10 = {ratio_x10}"
    );
    assert_eq!(a.bits, 0x1d08_0000);
    // 3. a 100× step: how long until the interval is back near T, and
    //    after the farm leaves (÷100), how long until blocks flow again.
    //    Lead grows at most 30 s a block, absorbing 100× needs
    //    log2(100) half-lives ≈ 3,986 s of lead, so > 133 blocks by
    //    construction; the collapse recovers in H·log2(100) ≈ 66 min
    //    of wall time whether or not blocks arrive (chain.md §1.5).
    let run = simulate(&g, &[(1_000, 100 * base_rate)]);
    let settled = run
        .windows(2)
        .position(|w| w[1].1 - w[0].1 >= 27)
        .expect("settles");
    let (h_set, t_set, _) = run[settled + 1];
    eprintln!(
        "asert 100x step: interval ≥ 27 s from block {h_set}, after {t_set}s wall ({} min)",
        t_set / 60
    );
    assert!(
        h_set <= 300 && t_set <= 3 * 3600,
        "100x absorbed in {h_set} blocks / {t_set}s"
    );
    let tail = run[999].1 - run[998].1;
    assert!(
        (27..=33).contains(&tail),
        "interval after 1000 blocks at 100x: {tail}s"
    );
    // the collapse: steady at 100× (continue from the settled state), then ÷100
    let both = simulate(&g, &[(1_000, 100 * base_rate), (200, base_rate)]);
    let t_leave = both[999].1;
    let first_slow = both[1000].1 - both[999].1;
    let back = both[1000..]
        .windows(2)
        .find(|w| w[1].1 - w[0].1 <= 60)
        .map(|w| w[1].1 - t_leave)
        .expect("recovers");
    eprintln!("asert 100x collapse: first block {first_slow}s ({} min), blocks ≤ 60 s again after {back}s ({} min)", first_slow / 60, back / 60);
    assert!(back <= 70 * 60, "recovery took {back}s");
    // 4. the same step from the real genesis target 2^232
    let g2 = pow2(232);
    let r2: u64 = (expected_hashes(&g2) / 30) as u64;
    let run2 = simulate(&g2, &[(20, r2), (1_000, 100 * r2)]);
    let first_fast = run2[20].1 - run2[19].1;
    let settled2 = run2[20..]
        .windows(2)
        .position(|w| w[1].1 - w[0].1 >= 27)
        .expect("settles");
    let (h2, t2, _) = run2[20 + settled2 + 1];
    let last = run2.last().unwrap();
    eprintln!(
            "asert 100x step @2^232: first block {first_fast}s, interval ≥ 27 s from block {h2} ({}s wall), bits after 1000 blocks {:#x}",
            t2 - run2[19].1,
            encode_bits(U256(last.2))
        );
}
