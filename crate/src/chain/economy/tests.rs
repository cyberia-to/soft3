//! chain-conformance §14 / chain-economy §10: the identities and integer
//! vectors, and the reference engine's contract.

use super::focus::{FocusEngine, FocusGraph, LN_BITS, PHI_BITS, Reference, ln_q40, log2_q40};
use super::settle::{Earner, Record, RECORD_BYTES, divide};
use super::wide::W;
use super::*;
use crate::chain::sigs::token;

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

const PUSSY_S0: u64 = 1_099_389_810_279_114_036;
const PUSSY_SUBSIDY: u64 = 10_458_426_657;

#[test]
fn identities_of_section_14() {
    let card = card_id(&[0x66; 32], &token("boot"), &token("pussy"));
    assert_eq!(hex(&card), "74f0a211d177f290e9c95a6e7edb65cd8705ea96d9713a841d5a086f6476b5b4");
    assert_eq!(hex(&stake_key(&card)), "f7e0cbf8e5cfc501b7e3098b238bce8451b3b0a7fa2e42b69724801115c6b08a");
    assert_eq!(hex(&focus_key(&token("pussy"))), "0fae8671376d879515b2736c2d6d51eafdd9ff40055aa84aaff6e17a0beed4b9");
    let leaf = payout_leaf(&card, &[0x77; 32], 1_000);
    assert_eq!(hex(&leaf), "63c3867a76af023aa6417e5a945612289906e8352ea69fe4aea259ce5b152115");
    assert_eq!(hex(&payout_node(&leaf, &leaf)), "240a852c1bd8f641f65d6b3746a72a08c0930e5d1bd1feafe4b9cdab914a6ec2");
    assert_eq!(hex(&phi_digest(&[1 << 59, 1 << 59])), "3351bfa40d6eaa24c8e2b8755d900dbcd3c3015730a5efcfdf9b6e14c7019a8a");
    // the payout tree: odd node promoted, empty is zeros
    assert_eq!(payouts_root(&[]), [0; 32]);
    assert_eq!(payouts_root(&[leaf]), leaf);
    assert_eq!(payouts_root(&[leaf, leaf, leaf]), payout_node(&payout_node(&leaf, &leaf), &leaf));
    // raw keys are 53-byte preimages: no owner ‖ token (64 bytes) collides
    assert_ne!(stake_key(&card), crate::wallet::tx::balance_key(&card, &token("pussy")));
}

#[test]
fn integers_of_section_14() {
    assert_eq!(
        [0, PUSSY_S0 / 10_000, PUSSY_S0 / 100, PUSSY_S0 / 4, PUSSY_S0].map(|a| focus_bps(a, PUSSY_S0)),
        [0, 99, 999, 5_000, 5_000]
    );
    assert_eq!(cut(PUSSY_SUBSIDY, 1_000), 1_045_842_665);
    assert_eq!(PUSSY_SUBSIDY - cut(PUSSY_SUBSIDY, 1_000), 9_412_583_992);
    assert_eq!(cut(PUSSY_SUBSIDY, 5_000), 5_229_213_328);
    assert_eq!(PUSSY_SUBSIDY - cut(PUSSY_SUBSIDY, 5_000), 5_229_213_329);
    assert_eq!(decay_mark(1_000_000, 0), 998_077);
    let mut h = 1_000_000;
    for _ in 0..360 {
        h = decay_mark(h, 0);
    }
    assert_eq!(h, 500_047);
    assert_eq!(decay_mark(10, 25), 25, "a new high replaces the mark");
    let e = |x: u64, cap: u64| Earner { card: [0; 32], owner: [0; 32], x: W::from_u64(x), cap };
    let pays = divide(1_000, &[e(1, u64::MAX), e(1, u64::MAX), e(1, u64::MAX)]);
    assert_eq!(pays, vec![333, 333, 333], "1 stays in the pool");
    let caps = [1_000_000u64, 1_000_000_000].map(|ds| ((ds as u128 * crate::chain::params::R_MAX_PPM as u128) / 1_000_000) as u64);
    assert_eq!(caps, [10_000, 10_000_000]);
    let pays = divide(1_000_000, &[e(3, caps[0]), e(1, caps[1])]);
    assert_eq!(pays, vec![10_000, 250_000], "740,000 stays");
    assert_eq!(divide(5, &[e(0, 9)]), vec![0], "X = 0 pays nothing");
}

#[test]
fn isqrt_is_the_floor_root() {
    for n in [0u128, 1, 2, 3, 4, 8, 9, 99, 100, 10_000, 99_999_999, 100_000_000, u64::MAX as u128, u128::MAX] {
        let r = isqrt(n);
        assert!(r * r <= n, "{n}");
        assert!((r + 1).checked_mul(r + 1).is_none_or(|sq| sq > n), "{n}");
    }
}

#[test]
fn record_is_198_bytes() {
    let r = Record { syntropy: -5, delta_syntropy: 7, focus_bps: 999, ..Default::default() };
    assert_eq!(r.encode().len(), RECORD_BYTES);
    assert_ne!(r.digest(), Record::default().digest());
}

#[test]
fn fixed_point_logarithms() {
    let one = 1u128 << PHI_BITS;
    assert_eq!(log2_q40(one, PHI_BITS), 0);
    assert_eq!(log2_q40(2 * one, PHI_BITS), 1 << LN_BITS);
    assert_eq!(log2_q40(one / 4, PHI_BITS), -(2 << LN_BITS));
    // ln 2 at 2^40 ≈ 762,123,384,786
    assert!((ln_q40(2 * one, PHI_BITS) - 762_123_384_786).abs() <= 1);
    // log2(3) = 1.584962500721156…
    let l3 = log2_q40(3 * one, PHI_BITS);
    assert!((l3 - 1_742_684_699_131).abs() <= 2, "{l3}");
}

fn star(n: u8, w: u128) -> Vec<(Id, Id, u128)> {
    // every leaf links to the hub [0; 32]
    let mut v: Vec<(Id, Id, u128)> = (1..=n).map(|i| ([i; 32], [0; 32], w)).collect();
    v.sort();
    v
}

#[test]
fn reference_engine_is_a_distribution_and_ranks_the_hub() {
    let g = FocusGraph::build(&star(5, 10));
    assert_eq!(g.n(), 6);
    assert_eq!(g.particles[0], [0; 32]);
    let f = Reference::default().focus(&g);
    let mass: u128 = f.phi.iter().map(|x| *x as u128).sum();
    let one = 1u128 << PHI_BITS;
    assert!(mass <= one && one - mass < 1 << 20, "mass {mass}");
    assert!(f.phi[0] > 3 * f.phi[1], "the hub holds the focus");
    assert!(f.syntropy > 0, "a star is more concentrated than uniform");
    // determinism: the same bits twice
    assert_eq!(Reference::default().focus(&g), f);
    assert_eq!(Reference::default().focus(&FocusGraph::default()).phi, Vec::<u64>::new());
}

#[test]
fn reference_gradient_agrees_with_a_finite_difference() {
    // a chain 1 → 2 → 3 plus 4 → 1: which edge raises syntropy?
    let mut axons: Vec<(Id, Id, u128)> = vec![([1; 32], [2; 32], 1_000), ([2; 32], [3; 32], 1_000), ([4; 32], [1; 32], 1_000), ([3; 32], [4; 32], 1_000), ([1; 32], [3; 32], 1_000)];
    axons.sort();
    let e = Reference::default();
    let base = FocusGraph::build(&axons);
    let grad = e.gradient(&base);
    let j0 = e.focus(&base).syntropy;
    for (i, g) in grad.iter().enumerate() {
        let mut bumped = axons.clone();
        bumped[i].2 += 100;
        let dj = e.focus(&FocusGraph::build(&bumped)).syntropy - j0;
        // g is ∂J/∂A at 2^100 per token; J at 2^60 → predicted ΔJ = g · 100 / 2^40
        let predicted = (g * 100) >> 40;
        assert_eq!(dj.signum(), predicted.signum(), "edge {i}: finite difference {dj}, gradient {predicted}");
        let err = (dj - predicted).abs();
        assert!(err * 10 <= dj.abs().max(1 << 20), "edge {i}: finite difference {dj}, gradient {predicted}");
    }
}

#[test]
fn settle_result_round_trips_through_the_cache_file() {
    use super::settle::{FocusOut, SettleResult};
    let r = SettleResult {
        epoch: 3,
        prev_block: [1; 32],
        boundary: 720,
        beacon: [2; 32],
        boundary_root: [3; 32],
        particles: 9,
        axons: 8,
        phi: [4; 32],
        syntropy: -77,
        prev_syntropy: 1 << 100,
        focus_bps: 999,
        earners: vec![Earner { card: [5; 32], owner: [6; 32], x: W([1, 2, 3, 4, 5, 6, 7, 8]), cap: 10 }],
        marks: vec![([7; 32], Mark { pos: 1, neg: 2 })],
        focus: std::sync::Arc::new(FocusOut { particles: vec![], phi: vec![], syntropy: -77, digest: [4; 32] }),
    };
    assert_eq!(super::run::round_trip(&r), Some(r));
}

#[test]
fn the_new_high_rule_pays_only_above_the_decayed_mark() {
    use super::settle::{Position, Snapshot, compute};
    use super::weights::GenesisWeights;
    use std::collections::BTreeMap;
    // a fixed graph: genesis weight on a → b and b → c; a +1 stake of 100
    // on b → c (which concentrates focus on c: ∂J/∂A > 0) appears in the epoch
    let (a, b, cc) = ([1u8; 32], [2u8; 32], [3u8; 32]);
    let weights = GenesisWeights { axons: vec![(a, b, 1_000), (b, cc, 1_000)], total: 2_000, at_floor: 0 };
    let axon = axon_of(&b, &cc);
    let card = card_id(&[9; 32], &b, &cc);
    let snap = |epoch: u64, stake: u64, mark: Option<Mark>| Snapshot {
        epoch,
        height: epoch * 10,
        block_id: [epoch as u8; 32],
        state_root: [0; 32],
        positions: if stake > 0 { vec![Position { card, axon, p: b, q: cc, owner: [9; 32], valence: 1, stake }] } else { vec![] },
        marks: mark.map(|m| BTreeMap::from([(axon, (b, cc, m))])).unwrap_or_default(),
        active: stake,
        supply: 1_000_000,
    };
    let e = Reference::default();
    let x_of = |mark: Option<Mark>| -> W {
        let r = compute(&snap(1, 0, None), &snap(2, 100, mark), &weights, &e, None);
        r.earners.iter().find(|x| x.card == card).map(|x| x.x).unwrap_or(W::ZERO)
    };
    let fresh = x_of(None);
    assert!(!fresh.is_zero(), "weight on b → c concentrates focus on c");
    // the mark at the old height: nothing new, nothing paid
    assert!(x_of(Some(Mark { pos: 100, neg: 0 })).is_zero());
    // one epoch of decay: 99 of 100 is old height, 1 is new
    let decayed = decay_mark(100, 0);
    assert_eq!(decayed, 99);
    assert_eq!(x_of(Some(Mark { pos: decayed, neg: 0 })), fresh.div_u64(100));
    // a mark on the other side does not limit this one
    assert_eq!(x_of(Some(Mark { pos: 0, neg: 1_000 })), fresh);
    // the marks the settlement leaves: the new height, decayed from there
    let r = compute(&snap(1, 0, None), &snap(2, 100, Some(Mark { pos: 250, neg: 0 })), &weights, &e, None);
    assert_eq!(r.marks, vec![(axon, Mark { pos: decay_mark(250, 100), neg: 0 })]);
    // an axon whose sides are empty and marks spent is dropped
    let r = compute(&snap(1, 0, None), &snap(2, 0, Some(Mark { pos: 1, neg: 0 })), &weights, &e, None);
    assert!(r.marks.is_empty());
}

#[test]
fn attribution_is_linear_in_stake_and_valence_cancels() {
    use super::settle::{Position, Snapshot, compute};
    use super::weights::GenesisWeights;
    let (a, b, cc) = ([1u8; 32], [2u8; 32], [3u8; 32]);
    let weights = GenesisWeights { axons: vec![(a, b, 1_000), (b, cc, 1_000)], total: 2_000, at_floor: 0 };
    let axon = axon_of(&b, &cc);
    let pos = |author: u8, stake: u64, valence: i8| Position { card: card_id(&[author; 32], &b, &cc), axon, p: b, q: cc, owner: [author; 32], valence, stake };
    let pos_on = |author: u8, p: Id, q: Id, stake: u64, valence: i8| Position { card: card_id(&[author; 32], &p, &q), axon: axon_of(&p, &q), p, q, owner: [author; 32], valence, stake };
    let snap = |epoch: u64, positions: Vec<Position>| {
        let mut positions = positions;
        positions.sort_by(|x, y| x.card.cmp(&y.card));
        let active = positions.iter().filter(|p| p.valence != 0).map(|p| p.stake).sum();
        Snapshot { epoch, height: epoch * 10, block_id: [epoch as u8; 32], state_root: [0; 32], positions, marks: Default::default(), active, supply: 1_000_000 }
    };
    let e = Reference::default();
    // one author with 100, or two with 60 and 40: the same total weight (Sybil-neutral)
    let one = compute(&snap(1, vec![]), &snap(2, vec![pos(7, 100, 1)]), &weights, &e, None);
    let two = compute(&snap(1, vec![]), &snap(2, vec![pos(7, 60, 1), pos(8, 40, 1)]), &weights, &e, None);
    let sum_two = two.earners.iter().fold(W::ZERO, |acc, x| acc.checked_add(&x.x).unwrap());
    let one_x = one.earners[0].x;
    let (hi, lo) = if one_x > sum_two { (one_x, sum_two) } else { (sum_two, one_x) };
    assert!(hi <= lo.checked_add(&W::from_u64(2)).unwrap(), "split changes only rounding: {one_x:?} vs {sum_two:?}");
    // passive stake moves the graph, earns nothing
    let passive = compute(&snap(1, vec![]), &snap(2, vec![pos(7, 100, 0)]), &weights, &e, None);
    assert!(passive.earners.is_empty());
    assert_ne!(passive.phi, compute(&snap(1, vec![]), &snap(2, vec![]), &weights, &e, None).phi, "passive stake moves φ*");
    // −1 on an axon with no weight does nothing: A is clipped at zero
    let against = compute(&snap(1, vec![]), &snap(2, vec![pos_on(7, cc, a, 100, -1)]), &weights, &e, None);
    assert!(against.earners.is_empty(), "nothing to subtract from: zero gradient");
    // −1 against a → b (whose weight spreads focus, ∂J/∂A < 0) earns
    let minus = compute(&snap(1, vec![]), &snap(2, vec![pos_on(7, a, b, 100, -1)]), &weights, &e, None);
    assert_eq!(minus.earners.len(), 1, "subtracting a spreading edge raises syntropy");
    // +1 on the same edge earns nothing: its gradient is negative
    let plus = compute(&snap(1, vec![]), &snap(2, vec![pos_on(7, a, b, 100, 1)]), &weights, &e, None);
    assert!(plus.earners.is_empty());
    // +1 and −1 of equal size on one axon (§9 hedging): A is unchanged, the
    // attributions are opposite; only the side whose a(c) > 0 earns, so the
    // pair costs twice the capital for one side's credit
    let hedge = compute(&snap(1, vec![]), &snap(2, vec![pos(7, 100, 1), pos(8, 100, -1)]), &weights, &e, None);
    assert_eq!(hedge.earners.len(), 1, "{:?}", hedge.earners);
    assert_eq!(hedge.earners[0].card, card_id(&[7; 32], &b, &cc));
}


/// A deterministic random graph of `n` particles and `m` axons.
fn random_axons(n: u64, m: u64, seed: u64) -> Vec<(Id, Id, u128)> {
    let mut s = seed;
    let mut next = || {
        s = s.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = s;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    };
    let id = |i: u64| -> Id { *hemera::hash(&i.to_le_bytes()).as_bytes() };
    let mut map = std::collections::BTreeMap::new();
    for _ in 0..m {
        let (a, b) = (next() % n, next() % n);
        if a != b {
            *map.entry((id(a), id(b))).or_insert(0u128) += 1 + (next() % 1_000_000) as u128;
        }
    }
    map.into_iter().map(|((p, q), w)| (p, q, w)).collect()
}

#[test]
fn tru_engine_digest_threads_and_scale() {
    use super::focus::Tru;
    let axons = random_axons(300, 1_500, 7);
    let g = FocusGraph::build(&axons);
    let one = Tru::new(1);
    let f = one.focus(&g);
    // the chain's digest is hemera over tru's preimage
    let tru_axons: Vec<tru::focusing::epoch::Axon> = axons.iter().map(|(p, q, w)| tru::focusing::epoch::Axon { from: *p, to: *q, weight: *w }).collect();
    let t = tru::focusing::epoch::phi(&tru_axons, &one.params);
    assert_eq!(t.particles, g.particles);
    assert_eq!(phi_digest(&f.phi), *hemera::hash(&t.preimage()).as_bytes());
    let mass: u128 = f.phi.iter().map(|x| *x as u128).sum();
    assert!(mass.abs_diff(1 << 60) < 1 << 40, "φ is a distribution at 2^60: {mass}");
    // thread count never changes a bit
    let eight = Tru::new(8);
    assert_eq!(eight.focus(&g), f);
    assert_eq!(eight.gradient(&g), one.gradient(&g));
}

#[test]
fn quadrature_is_a_parameter_and_one_node_is_the_midpoint() {
    use super::focus::Tru;
    use super::settle::{Position, Snapshot, compute_with};
    use super::weights::GenesisWeights;
    let axons = random_axons(60, 200, 11);
    let weights = GenesisWeights { axons: axons.iter().map(|(p, q, w)| (*p, *q, *w as u64)).collect(), total: 0, at_floor: 0 };
    let snap = |epoch: u64, positions: Vec<Position>| Snapshot { epoch, height: epoch * 10, block_id: [epoch as u8; 32], state_root: [0; 32], active: positions.iter().map(|p| p.stake).sum(), positions, marks: Default::default(), supply: 1 << 40 };
    let positions: Vec<Position> = {
        let mut v: Vec<Position> = axons
            .iter()
            .take(20)
            .enumerate()
            .map(|(i, (p, q, _))| Position { card: card_id(&[i as u8; 32], p, q), axon: axon_of(p, q), p: *p, q: *q, owner: [i as u8; 32], valence: if i % 2 == 0 { 1 } else { -1 }, stake: 500_000 })
            .collect();
        v.sort_by(|a, b| a.card.cmp(&b.card));
        v
    };
    let e = Tru::new(2);
    let one = compute_with(&snap(1, vec![]), &snap(2, positions.clone()), &weights, &e, None, 1);
    let two = compute_with(&snap(1, vec![]), &snap(2, positions.clone()), &weights, &e, None, 2);
    assert!(!one.earners.is_empty());
    assert_eq!(one.phi, two.phi, "φ*_E does not depend on the quadrature");
    // the two rules agree on who earns for most positions on a smooth path
    let who = |r: &super::settle::SettleResult| r.earners.iter().map(|x| x.card).collect::<std::collections::BTreeSet<_>>();
    let common = who(&one).intersection(&who(&two)).count();
    assert!(common * 4 >= who(&one).len() * 3, "{} of {}", common, who(&one).len());
}
