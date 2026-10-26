//! Measurement only (plan row L4b, director session 75) — copied into a foculus
//! checkout by `.github/workflows/diag-l4b-withholding.yml`, never merged.
//!
//! The same computation as foculus `tests/withholding_bias.rs`, over 32 beacons
//! (seed 0 is the test's own beacon). Per seed and q it prints the pool size N,
//! the tickets the adversary is assigned A (its realized share A/N), the
//! tickets it withholds W, the bias, the test's nominal bound
//! `range * q / (1 - q)` and the realized-share bound `range * A / (N - A)`.
//!
//! `bias = sum_W (mean - x) / (N - W) <= range * W / (N - W) <= range * A / (N - A)`
//! holds for every pool; the nominal bound holds only when A/N <= q (roughly).

use foculus::settlement::Contribution;
use foculus::tickets::{grind_settlement, self_fold, SettlementTicket};
use tru::{Context, FocusingParams, Fx, Link};

fn h(b: u8) -> [u8; 32] {
    let mut x = [0u8; 32];
    x[0] = b;
    x
}

fn base() -> Vec<Link> {
    vec![
        Link::stake(h(1), h(2), 100),
        Link::stake(h(2), h(3), 100),
        Link::stake(h(3), h(1), 100),
    ]
}

fn cluster() -> Vec<Contribution> {
    (0..6)
        .map(|i| Contribution {
            neuron: h((16 + i) as u8),
            links: vec![Link::stake(h(2), h(1), 100 + i as u128 * 900)],
            surprise: Fx::ONE,
        })
        .collect()
}

fn assign_to_adversary(nonce: u64, q: f64) -> bool {
    let mut x = nonce ^ 0x9E37_79B9_7F4A_7C15;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    (x % 1_000_000) as f64 / 1_000_000.0 < q
}

#[test]
fn withholding_sweep() {
    let c = cluster();
    let neurons: Vec<[u8; 32]> = c.iter().map(|c| c.neuron).collect();
    let (mut nominal_red, mut realized_red, mut monotone_red, mut runs) = (0, 0, 0, 0);
    for seed in 0u8..32 {
        let mut beacon = h(0xBE);
        beacon[1] = seed;
        let tickets: Vec<SettlementTicket> = grind_settlement(
            &base(), &c, &Context::none(), &FocusingParams::default(),
            &beacon, &h(0xC1), &h(0x91), 0, 4_000, 400, u64::MAX >> 3,
        );
        let n = tickets.len();
        let sum = tickets.iter().fold(Fx::ZERO, |a, t| a + t.marginals[0]);
        let mean = sum.div(Fx::from_int(n as i64));
        let min = tickets.iter().map(|t| t.marginals[0]).reduce(|a, b| if b < a { b } else { a }).unwrap();
        let range = mean - min;
        let below = tickets.iter().filter(|t| t.marginals[0] < mean).count();
        let mut prev = Fx::ZERO;
        for &q in &[0.05, 0.1, 0.25, 0.5] {
            let a = tickets.iter().filter(|t| assign_to_adversary(t.nonce, q)).count();
            let published: Vec<SettlementTicket> = tickets.iter()
                .filter(|t| !assign_to_adversary(t.nonce, q) || t.marginals[0] >= mean)
                .cloned().collect();
            let w = n - published.len();
            let biased = self_fold(6, &published).mean_shares(&neurons)[0].1;
            let bias = biased - mean;
            let nominal = range * Fx::from_ratio((q * 1000.0) as i64, (1000.0 * (1.0 - q)) as i64);
            let realized = range * Fx::from_ratio(a as i64, (n - a) as i64);
            let (nok, rok, mok) = (bias <= nominal, bias <= realized, bias >= prev);
            runs += 1;
            nominal_red += !nok as u32;
            realized_red += !rok as u32;
            monotone_red += !mok as u32;
            println!(
                "[sweep] seed={seed:2} q={q:.2} N={n} below_mean={below} A={a} A/N={:.4} W={w} bias={:.6} nominal={:.6} realized={:.6} nominal_ok={nok} realized_ok={rok} monotone_ok={mok}",
                a as f64 / n as f64, bias.to_f64(), nominal.to_f64(), realized.to_f64()
            );
            prev = bias;
        }
    }
    println!("[sweep-summary] runs={runs} nominal_red={nominal_red} realized_red={realized_red} monotone_red={monotone_red}");
}
