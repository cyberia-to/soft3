//! The focus economy on the engine (chain-economy §2–§6, conformance §14):
//! staking with valence, unlock and payout owner, settlement across
//! epochs, the new-high rule, the cap, spam, determinism across nodes,
//! reorgs around a boundary, and settlement off the block path.

use std::sync::Arc;
use std::time::Duration;

use super::*;
use crate::chain::api::{BlockVerdict, ChainApi};
use crate::chain::apply::TxError;
use crate::chain::economy::focus::{Focus, FocusEngine, FocusGraph, Reference};
use crate::chain::economy::{card_id, focus_key, stake_key};
use crate::chain::state::StateBackend;

/// L = 10, D = 5, MIN_LOCK = 15: settle(E) = 10E + 5.
pub const CLOCK: EpochClock = EpochClock { length: 10, delay: 5, min_lock: 15 };
/// Funds per account large enough that the subsidy (S0 / 105,120,000)
/// and the pool are not dust: S0 ≈ 4·10^15, SUBSIDY ≈ 3.8·10^7.
pub const RICH: u64 = 1_000_000_000_000_000;
const FEE: u64 = 257;

fn econ_net(accounts: u8) -> Net {
    let mut net = Net::with_funds(accounts, RICH);
    net.clock = Some(CLOCK);
    net
}

fn p(i: u8) -> Id {
    [i; 32]
}

/// Mine until the tip is at `height`, empty blocks to `miner`.
fn mine_to(c: &mut Chain, miner: Id, height: u64) -> Vec<Block> {
    let mut out = Vec::new();
    while c.tip().height < height {
        out.push(mine(c, miner, 0, &[]));
    }
    out
}

fn pool(c: &Chain, net: &Net) -> u64 {
    c.state().raw_balance(&focus_key(&net.native))
}

fn stake(c: &Chain, author: Id, p: Id, q: Id) -> u64 {
    c.state().raw_balance(&stake_key(&card_id(&author, &p, &q)))
}

fn outcome_at(c: &Chain, height: u64) -> Arc<crate::chain::economy::settle::Outcome> {
    let id = c.header_at(height).unwrap().id();
    c.economy.outcome(&id).expect("a settlement outcome at this height")
}

/// `x(c)` of a card in the settlement carried at `height` (main chain).
fn x_at(c: &mut Chain, height: u64, card: &Id) -> crate::chain::economy::wide::W {
    let e = CLOCK.settles_at(height).expect("a settlement height");
    let prev = c.header_at(CLOCK.boundary(e - 1)).unwrap().id();
    let cur = c.header_at(CLOCK.boundary(e)).unwrap().id();
    let r = c.economy.result(&prev, &cur).unwrap();
    r.earners.iter().find(|x| x.card == *card).map(|x| x.x).unwrap_or_default()
}

fn assert_economy_conserved(c: &Chain) {
    assert!(c.status().conserved, "conservation at {}", c.tip().height);
    crate::chain::apply::check_stake_total(c.state()).unwrap();
}

#[test]
fn staked_cyberlink_rules_kinds_6_and_7() {
    let net = econ_net(3);
    let mut c = net.open("stake-rules");
    let miner = net.neurons[2];
    let (a, b) = (net.neurons[0], net.neurons[1]);
    let (x, y) = (p(10), p(11));
    // a non-native stake is invalid; a native one locks
    assert_eq!(c.validate_tx(&net.link_token(0, 0, token("other"), x, y, 7, 1, FEE), &[]), Err(TxError::StakeToken));
    assert!(c.validate_tx(&net.link_token(0, 0, token("other"), x, y, 0, 1, FEE), &[]).is_ok(), "a bare link names any token");
    // kinds 6 and 7 need the card
    assert_eq!(c.validate_tx(&net.unlock(0, 0, x, y, 1, 224), &[]), Err(TxError::NoCard));
    assert_eq!(c.validate_tx(&net.payout(0, 0, x, y, b, 248), &[]), Err(TxError::NoCard));
    mine(&mut c, miner, 0, &[net.link(0, 0, x, y, 7, 1, FEE)]);
    let card = card_id(&a, &x, &y);
    let since = c.tip().height;
    assert_eq!(stake(&c, a, x, y), 7);
    assert_eq!(c.balance(&a, &net.native), RICH - 7 - FEE);
    let rec = c.state().card(&card).unwrap();
    assert_eq!((rec.owner, rec.author, rec.valence, rec.since), (a, a, 1, since));
    assert_eq!(rec.particle, crate::chain::economy::axon_of(&x, &y));
    assert_economy_conserved(&c);
    // the opposite valence on a live card is rejected; the same adds and restarts the lock
    assert_eq!(c.validate_tx(&net.link(0, 1, x, y, 3, -1, FEE), &[]), Err(TxError::Valence { card: 1, tx: -1 }));
    assert_eq!(c.validate_tx(&net.link(0, 1, x, y, 3, 0, FEE), &[]), Err(TxError::Valence { card: 1, tx: 0 }));
    mine_to(&mut c, miner, since + 3);
    mine(&mut c, miner, 0, &[net.link(0, 1, x, y, 3, 1, FEE)]);
    let since = c.tip().height;
    assert_eq!(stake(&c, a, x, y), 10);
    assert_eq!(c.state().card(&card).unwrap().since, since);
    // kind 7 by the author re-points the owner; by anyone else there is no card
    assert_eq!(c.validate_tx(&net.payout(1, 0, x, y, b, 248), &[]), Err(TxError::NoCard));
    mine(&mut c, miner, 0, &[net.payout(0, 2, x, y, b, 248)]);
    assert_eq!(c.state().card(&card).unwrap().owner, b);
    // kind 6: before since + MIN_LOCK rejected, at it accepted; a non-author has no card
    mine_to(&mut c, miner, since + CLOCK.min_lock - 2);
    assert_eq!(c.validate_tx(&net.unlock(0, 3, x, y, 4, 224), &[]), Err(TxError::Locked { until: since + CLOCK.min_lock }));
    assert_eq!(c.validate_tx(&net.unlock(1, 0, x, y, 4, 224), &[]), Err(TxError::NoCard));
    mine(&mut c, miner, 0, &[]);
    assert_eq!(c.tip().height + 1, since + CLOCK.min_lock);
    assert_eq!(c.validate_tx(&net.unlock(0, 3, x, y, 11, 224), &[]), Err(TxError::UnlockAmount { stake: 10, amount: 11 }));
    let before = c.balance(&a, &net.native);
    mine(&mut c, miner, 0, &[net.unlock(0, 3, x, y, 4, 224)]);
    assert_eq!(stake(&c, a, x, y), 6, "partial unlock");
    assert_eq!(c.balance(&a, &net.native), before + 4 - 224);
    assert!(c.state().card(&card).is_some());
    mine(&mut c, miner, 0, &[net.unlock(0, 4, x, y, 6, 224)]);
    assert_eq!(stake(&c, a, x, y), 0);
    assert!(c.state().card(&card).is_none(), "the card is deleted at zero");
    assert_eq!(c.state().scalars().total_stake, 0);
    // once empty, the other side is open
    mine(&mut c, miner, 0, &[net.link(0, 5, x, y, 2, -1, FEE)]);
    assert_eq!(c.state().card(&card).unwrap().valence, -1);
    assert_economy_conserved(&c);
}

#[test]
fn an_unlock_pays_its_fee_from_what_it_returns() {
    let net = econ_net(3);
    let mut c = net.open("unlock-fee");
    let miner = net.neurons[2];
    let (x, y) = (p(20), p(21));
    // a's whole liquid balance goes into the stake but for the link fee
    let a = net.neurons[0];
    mine(&mut c, miner, 0, &[net.link(0, 0, x, y, RICH - FEE, 1, FEE)]);
    assert_eq!(c.balance(&a, &net.native), 0);
    mine_to(&mut c, miner, 1 + CLOCK.min_lock - 1);
    mine(&mut c, miner, 0, &[net.unlock(0, 1, x, y, 1_000, 224)]);
    assert_eq!(c.balance(&a, &net.native), 1_000 - 224);
    assert_economy_conserved(&c);
}

#[test]
fn coinbase_split_follows_the_active_stake_of_the_last_boundary() {
    let net = econ_net(3);
    let mut c = net.open("split");
    let miner = net.neurons[2];
    let subsidy = net.subsidy();
    // epoch 1: bps(0) = 0, the miner takes the whole subsidy
    mine(&mut c, miner, 0, &[net.link(0, 0, p(30), p(31), RICH / 100, 1, FEE), net.link(1, 0, p(30), p(32), RICH / 100, 0, FEE)]);
    mine_to(&mut c, miner, 10);
    assert_eq!(pool(&c, &net), 0);
    // boundary 10: active = RICH/100 (the passive stake does not count)
    let supply = net.genesis.s0() + 10 * subsidy;
    let bps = crate::chain::economy::focus_bps(RICH / 100, supply);
    assert_eq!(c.state().scalars().focus_bps, bps);
    assert!(bps > 0 && bps < 5_000);
    let before = c.balance(&miner, &net.native);
    mine(&mut c, miner, 0, &[]);
    let cut = crate::chain::economy::cut(subsidy, bps);
    assert_eq!(c.balance(&miner, &net.native) - before, subsidy - cut);
    assert_eq!(pool(&c, &net), cut);
    assert_eq!(c.supply(&net.native), net.genesis.s0() + 11 * subsidy);
    assert_economy_conserved(&c);
}

/// The whole cycle over several epochs: +1, 0 and −1 stake, zero-stake
/// spam, a payout owner change, partial and full unlocks, the high-water
/// recycling attempt, and a self-cycle held by the cap.
#[test]
fn multi_epoch_settlement_end_to_end() {
    let net = econ_net(6);
    let mut c = net.open("e2e");
    let miner = net.neurons[5];
    let n = &net.neurons;
    let (g1, g2) = (p(1), p(2)); // the genesis link of Net (author n[0])
    let hub = p(40);
    let mut nonce = [0u64; 6];
    let mut tx = |i: usize, f: &dyn Fn(u64) -> Vec<u8>| -> Vec<u8> {
        let t = f(nonce[i]);
        nonce[i] += 1;
        t
    };
    let big = RICH / 50;

    // epoch 1 (blocks 1–10). Under J = KL(φ ‖ u) stake earns where it
    // concentrates focus: +1 on the dominant genesis edge g1 → g2, −1
    // against the minor edge 42 → hub (weighted by a passive lock); a +1
    // ring and zero-stake spam ride along
    let e1 = vec![
        tx(1, &|k| net.link(1, k, g1, g2, big, 1, FEE)),
        tx(2, &|k| net.link(2, k, p(42), hub, big, 0, FEE)),
        tx(4, &|k| net.link(4, k, p(50), p(51), big, 1, FEE)),
        tx(4, &|k| net.link(4, k, p(51), p(50), big, 1, FEE)),
    ];
    let spam: Vec<Vec<u8>> = (0..20u8).map(|i| tx(0, &|k| net.link(0, k, p(60 + i), hub, 0, 1, FEE))).collect();
    mine(&mut c, miner, 0, &e1);
    mine(&mut c, miner, 0, &spam);
    mine(&mut c, miner, 0, &[tx(3, &|k| net.link(3, k, p(42), hub, big / 4, -1, FEE))]);
    mine_to(&mut c, miner, 10);
    let boundary1 = c.tip().id;
    assert!(c.economy.snapshot(&boundary1).is_some(), "the boundary state is frozen");
    // the payout owner of n[1]'s position moves to n[0] before settlement:
    // too late for epoch 1's snapshot (it reads G_1), in time for epoch 2
    mine(&mut c, miner, 0, &[tx(1, &|k| net.payout(1, k, g1, g2, n[0], 248))]);
    mine_to(&mut c, miner, 14);
    let pool_before = pool(&c, &net);
    let paid_before: Vec<u64> = n.iter().map(|x| c.balance(x, &net.native)).collect();
    mine(&mut c, miner, 0, &[]); // 15 = settle(1)
    assert_economy_conserved(&c);
    let o1 = outcome_at(&c, 15);
    let r = &o1.record;
    assert_eq!((r.epoch, r.boundary, r.beacon), (1, 10, boundary1));
    assert_eq!(r.boundary_root, c.header_at(10).unwrap().state_root);
    assert_eq!(c.state().time(15), Some(r.digest()), "time[settle(1)] commits to the record");
    assert!(r.budget > 0, "the pool accrued from block 11 on");
    assert_eq!(r.budget, (pool_before + crate::chain::economy::cut(net.subsidy(), c.state().scalars().focus_bps)) / 4);
    assert!(r.paid <= r.budget);
    assert_eq!(r.pool_after, pool(&c, &net));
    let cards_paid: Vec<Id> = o1.payouts.iter().map(|(card, _, _)| *card).collect();
    // the passive position and the spammer earn nothing
    assert!(!cards_paid.contains(&card_id(&n[2], &p(42), &hub)), "v = 0 never earns");
    for (_, owner, _) in &o1.payouts {
        assert_ne!(*owner, n[2]);
    }
    assert!(o1.payouts.iter().all(|(card, _, _)| *card != card_id(&n[0], &p(60), &hub)), "no card, no reward");
    // whatever was paid went to the payout owners of G_1: the authors (the
    // owner change of block 11 came after the boundary)
    let authors = [
        (card_id(&n[1], &g1, &g2), n[1]),
        (card_id(&n[3], &p(42), &hub), n[3]),
        (card_id(&n[4], &p(50), &p(51)), n[4]),
        (card_id(&n[4], &p(51), &p(50)), n[4]),
    ];
    for (card, owner, amount) in &o1.payouts {
        let author = authors.iter().find(|(c, _)| c == card).map(|(_, a)| *a).expect("only active positions earn");
        assert_eq!(*owner, author);
        assert!(*amount > 0);
    }
    // the ring is bounded by the cap: at most 1 % of the stake each side added
    for (card, _, amount) in &o1.payouts {
        if *card == card_id(&n[4], &p(50), &p(51)) || *card == card_id(&n[4], &p(51), &p(50)) {
            assert!(*amount <= big / 100, "ring paid {amount} > 1 % of {big}");
        }
    }
    let gained: Vec<i128> = n.iter().zip(&paid_before).map(|(x, b)| c.balance(x, &net.native) as i128 - *b as i128).collect();
    assert_eq!(gained.iter().sum::<i128>() - gained[5], r.paid as i128, "payouts are the only credits of a quiet block but the miner's");
    let earned: Vec<Id> = o1.payouts.iter().map(|x| x.0).collect();
    assert!(earned.contains(&card_id(&n[1], &g1, &g2)), "+1 on the dominant edge raised syntropy: {r:?}");
    assert!(earned.contains(&card_id(&n[3], &p(42), &hub)), "−1 against a spreading edge raised syntropy: {r:?}");

    // epoch 2: n[1] adds to its position (paid to n[0] now), nothing else moves
    mine(&mut c, miner, 0, &[tx(1, &|k| net.link(1, k, g1, g2, big, 1, FEE))]);
    mine_to(&mut c, miner, 25);
    assert_economy_conserved(&c);
    let o2 = outcome_at(&c, 25);
    let hub_card = card_id(&n[1], &g1, &g2);
    let (_, owner, _) = o2.payouts.iter().find(|(card, _, _)| *card == hub_card).expect("the increase earns again");
    assert_eq!(*owner, n[0], "the payout owner of G_2");
    // positions that did not add stake earn nothing in epoch 2
    assert!(o2.payouts.iter().all(|(card, _, _)| *card == hub_card), "{:?}", o2.payouts);

    // the high-water rule: n[3] unlocks its −1 stake after MIN_LOCK and relocks the same amount
    let since = c.state().card(&card_id(&n[3], &p(42), &hub)).unwrap().since;
    mine_to(&mut c, miner, since + CLOCK.min_lock - 1);
    mine(&mut c, miner, 0, &[tx(3, &|k| net.unlock(3, k, p(42), hub, big / 4, 224))]);
    assert!(c.state().card(&card_id(&n[3], &p(42), &hub)).is_none());
    // the next settlement writes the decayed mark of the emptied side
    let next_settle = CLOCK.settle(CLOCK.epoch(c.tip().height));
    mine_to(&mut c, miner, next_settle);
    let axon = crate::chain::economy::axon_of(&p(42), &hub);
    let mark = c.state().marks().into_iter().find(|(a, _)| *a == axon).map(|(_, m)| m).unwrap();
    assert!(mark.neg > 0 && mark.neg < big / 4, "the mark decays from the old height: {mark:?}");
    mine(&mut c, miner, 0, &[tx(3, &|k| net.link(3, k, p(42), hub, big / 4, -1, FEE))]);
    let relock_epoch = CLOCK.epoch(c.tip().height);
    mine_to(&mut c, miner, CLOCK.settle(relock_epoch));
    let o = outcome_at(&c, CLOCK.settle(relock_epoch));
    let relock = card_id(&n[3], &p(42), &hub);
    let first = x_at(&mut c, 15, &relock);
    let again = x_at(&mut c, CLOCK.settle(relock_epoch), &relock);
    let _ = o;
    // the relock climbs back under the decayed mark: N/R is the few parts
    // in a thousand the mark decayed, against 1 for the first lock (the
    // exact arithmetic is pinned by the unit vectors; the gradient moves
    // between the epochs, so this compares orders of magnitude)
    assert!(!first.is_zero());
    assert!(again.checked_mul(&crate::chain::economy::wide::W::from_u64(20)).unwrap() < first, "recycling earns the decay only: first {first:?}, again {again:?}");
    assert_economy_conserved(&c);
}

#[test]
fn two_nodes_agree_on_every_epoch_and_a_restart_replays_it() {
    let net = econ_net(4);
    let mut a = net.open("det-a");
    let miner = net.neurons[3];
    let mut blocks = vec![mine(&mut a, miner, 0, &[net.link(0, 0, p(70), p(71), RICH / 20, 1, FEE), net.link(1, 0, p(71), p(72), RICH / 30, -1, FEE)])];
    blocks.extend(mine_to(&mut a, miner, 12));
    blocks.push(mine(&mut a, miner, 0, &[net.link(2, 0, p(72), p(70), RICH / 40, 1, FEE)]));
    blocks.extend(mine_to(&mut a, miner, 36));
    let mut b = net.open("det-b");
    feed(&mut b, &blocks);
    assert_eq!(a.tip(), b.tip());
    for e in 1..=3 {
        assert_eq!(outcome_at(&a, CLOCK.settle(e)).record, outcome_at(&b, CLOCK.settle(e)).record, "epoch {e}");
    }
    // a restart replays from disk, through the settlement cache
    drop(a);
    let a2 = net.reopen(0);
    assert_eq!(a2.tip(), b.tip());
    assert_eq!(a2.status().state_root, b.status().state_root);
    assert_eq!(outcome_at(&a2, 25).record, outcome_at(&b, 25).record);
}

#[test]
fn reorg_above_the_boundary_reproduces_the_record_and_of_the_boundary_changes_it() {
    let net = econ_net(4);
    let miner = net.neurons[3];
    let mut a = net.open("reorg-a");
    let shared = {
        let mut v = vec![mine(&mut a, miner, 0, &[net.link(0, 0, p(80), p(81), RICH / 20, 1, FEE)])];
        v.extend(mine_to(&mut a, miner, 8));
        v
    };
    // a: a lock in block 9, boundary 10, up to settle(1) = 15
    let mut a_tail = vec![mine(&mut a, miner, 0, &[net.link(1, 0, p(81), p(82), RICH / 20, 1, FEE)])];
    a_tail.extend(mine_to(&mut a, miner, 15));
    let rec_a = outcome_at(&a, 15).record.clone();

    // b: same blocks to 12, then a 3-block fork ending at settle(1) and beyond
    let mut b = net.open("reorg-b");
    feed(&mut b, &shared);
    feed(&mut b, &a_tail[..4]); // heights 9..=12
    let fork: Vec<Block> = (0..4).map(|_| mine(&mut b, net.neurons[2], 0, &[])).collect(); // 13..=16
    let verdicts = feed(&mut a, &fork);
    assert!(verdicts.iter().any(|v| matches!(v, BlockVerdict::Reorganized { depth: 3, .. })), "{verdicts:?}");
    assert_eq!(a.tip(), b.tip());
    assert_eq!(outcome_at(&a, 15).record, rec_a, "a reorg above the boundary reproduces the record");
    assert_eq!(outcome_at(&b, 15).record, rec_a);

    // c: forks at 8 (below the boundary) without block 9's lock and wins:
    // the boundary block is replaced, the snapshot and the record change
    let mut c = net.open("reorg-c");
    feed(&mut c, &shared);
    let fork_c: Vec<Block> = (0..10).map(|_| mine(&mut c, net.neurons[1], 0, &[])).collect(); // 9..=18
    let verdicts = feed(&mut a, &fork_c);
    assert!(verdicts.iter().any(|v| matches!(v, BlockVerdict::Reorganized { .. })), "{verdicts:?}");
    assert_eq!(a.tip(), c.tip());
    let rec_c = outcome_at(&a, 15).record.clone();
    assert_ne!(rec_c.beacon, rec_a.beacon, "another boundary block");
    assert_ne!(rec_c, rec_a);
    assert_eq!(outcome_at(&c, 15).record, rec_c);
    assert_eq!(a.status().state_root, c.status().state_root);
    assert_economy_conserved(&a);
}

/// An engine that takes its time, to show the node keeps validating while
/// a settlement computes (chain-economy §2.5).
struct Slow(Reference, Duration);

impl FocusEngine for Slow {
    fn id(&self) -> &'static str {
        "slow-reference"
    }
    fn focus(&self, g: &FocusGraph) -> Focus {
        std::thread::sleep(self.1);
        self.0.focus(g)
    }
    fn gradient(&self, g: &FocusGraph) -> Vec<i128> {
        self.0.gradient(g)
    }
}

#[test]
fn settlement_runs_off_the_block_path() {
    let mut net = econ_net(3);
    net.engine = Some(Arc::new(Slow(Reference::default(), Duration::from_millis(400))));
    let mut c = net.open("offpath");
    let miner = net.neurons[2];
    mine(&mut c, miner, 0, &[net.link(0, 0, p(90), p(91), RICH / 20, 1, FEE)]);
    mine_to(&mut c, miner, 10);
    let b0 = c.header_at(0).unwrap().id();
    let b10 = c.tip().id;
    let t0 = std::time::Instant::now();
    mine_to(&mut c, miner, 14);
    let busy = !c.economy.ready(&b0, &b10);
    assert!(busy, "four blocks were validated while the settlement worker ran ({:?})", t0.elapsed());
    mine(&mut c, miner, 0, &[]); // settle(1): joins the worker
    assert!(c.economy.ready(&b0, &b10));
    assert_eq!(outcome_at(&c, 15).record.epoch, 1);
    assert_economy_conserved(&c);
}

/// Conservation at every height of a long random chain with locks,
/// unlocks and payout changes, settlement blocks included (§10).
#[test]
fn random_economy_keeps_conservation_for_2000_blocks() {
    let net = econ_net(5);
    let mut c = net.open("random");
    let mut rng = 0x5eed_u64;
    let mut next = || {
        rng = rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    };
    let mut paid_total = 0u64;
    for h in 1..=2_000u64 {
        let mut txs = Vec::new();
        for _ in 0..(next() % 3) {
            let i = (next() % 4) as usize;
            let nonce = c.nonce(&net.neurons[i]) + txs.iter().filter(|t: &&Vec<u8>| Tx::decode(t).unwrap().from() == Some(net.neurons[i])).count() as u64;
            let (x, y) = (p(100 + (next() % 6) as u8), p(110 + (next() % 3) as u8));
            let valence = (next() % 3) as i8 - 1;
            let author = net.neurons[i];
            let card = c.state().card(&card_id(&author, &x, &y));
            let t = match next() % 4 {
                0 | 1 => {
                    let v = card.map(|cd| cd.valence).unwrap_or(valence);
                    net.link(i, nonce, x, y, 1 + next() % (RICH / 1_000), v, FEE)
                }
                2 => {
                    let have = c.state().raw_balance(&stake_key(&card_id(&author, &x, &y)));
                    net.unlock(i, nonce, x, y, 1 + next() % have.max(1), 224)
                }
                _ => net.payout(i, nonce, x, y, net.neurons[(next() % 5) as usize], 248),
            };
            if c.validate_tx(&t, &txs.iter().map(|r| Tx::decode(r).unwrap()).collect::<Vec<_>>()).is_ok() {
                txs.push(t);
            }
        }
        mine(&mut c, net.neurons[4], 0, &txs);
        assert_eq!(c.tip().height, h);
        assert_economy_conserved(&c);
        assert_eq!(c.supply(&net.native), net.genesis.s0() + h * net.subsidy());
        if let Some(e) = CLOCK.settles_at(h) {
            let o = outcome_at(&c, h);
            assert_eq!(o.record.epoch, e);
            assert!(o.record.paid <= o.record.budget);
            paid_total += o.record.paid;
        }
    }
    assert!(paid_total > 0, "2,000 blocks of random stake paid something");
}

#[test]
fn the_node_surface_reads_epochs_focus_and_cards() {
    let net = econ_net(3);
    let mut c = net.open("surface");
    let miner = net.neurons[2];
    mine(&mut c, miner, 0, &[net.link(0, 0, p(1), p(2), RICH / 20, 1, FEE)]);
    mine_to(&mut c, miner, 25);
    let e1 = c.epoch_json(1).unwrap();
    assert_eq!(e1["epoch"], 1);
    assert_eq!(e1["settled_at"], 15);
    assert_eq!(e1["digest"].as_str().unwrap().len(), 64);
    assert_eq!(e1["record"].as_str().unwrap().len(), 2 * crate::chain::economy::settle::RECORD_BYTES);
    assert!(c.epoch_json(3).is_none(), "not settled yet");
    assert_eq!(c.epochs_json(0, 10)["epochs"].as_array().unwrap().len(), 2);
    let top = c.focus_top_json(3).unwrap();
    assert_eq!(top["epoch"], 2);
    assert_eq!(top["top"].as_array().unwrap().len(), 2, "the graph is p(1) → p(2)");
    let f = c.focus_json(&p(2)).unwrap();
    assert!(f["rank"].as_u64().unwrap() >= 1);
    let card = card_id(&net.neurons[0], &p(1), &p(2));
    let cj = c.card_json(&card).unwrap();
    assert_eq!(cj["stake"], RICH / 20);
    assert_eq!(cj["unlock_at"], 1 + CLOCK.min_lock);
    assert_eq!(c.cards_json(&net.neurons[0])["cards"].as_array().unwrap().len(), 1);
    let st = c.economy_status();
    assert_eq!(st["last_settled"], 2);
    assert_eq!(st["next_boundary"], 30);
    assert_eq!(st["stake_keys_match"], true);
    assert_eq!(st["settlement_ok"], true);
    assert_eq!(st["total_stake"], RICH / 20);
}

/// An engine whose every solve panics, to show the halt is named.
struct Panics;

impl FocusEngine for Panics {
    fn id(&self) -> &'static str {
        "panics-test"
    }
    fn focus(&self, _g: &FocusGraph) -> Focus {
        panic!("injected settlement panic")
    }
    fn gradient(&self, _g: &FocusGraph) -> Vec<i128> {
        panic!("injected settlement panic")
    }
}

fn panic_text(payload: Box<dyn std::any::Any + Send>) -> String {
    match payload.downcast::<String>() {
        Ok(s) => *s,
        Err(p) => p.downcast_ref::<&str>().map(|s| s.to_string()).unwrap_or_default(),
    }
}

/// A panic in a settlement is a halt (chain-economy §2.5): named by epoch
/// and height, leaving the state untouched, the same on a replay.
#[test]
fn a_settlement_panic_is_a_named_halt() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    let mut net = econ_net(3);
    net.engine = Some(Arc::new(Panics));
    let miner = net.neurons[2];
    let mut c = net.open("halt-a");
    let mut blocks = vec![mine(&mut c, miner, 0, &[net.link(0, 0, p(90), p(91), RICH / 20, 1, FEE)])];
    blocks.extend(mine_to(&mut c, miner, CLOCK.settle(1) - 1));
    assert_eq!(c.tip().height, 14);
    let (pool0, settled0, root0, tip0) = (pool(&c, &net), c.state().scalars().last_settled, c.status().state_root, c.tip().id);

    let msg = panic_text(catch_unwind(AssertUnwindSafe(|| mine(&mut c, miner, 0, &[]))).err().expect("the settlement block halts"));
    assert!(msg.contains("settlement of epoch 1"), "{msg}");
    assert!(msg.contains("at height 15"), "{msg}");
    assert!(msg.contains("injected settlement panic"), "{msg}");

    assert_eq!(c.tip().height, 14);
    assert_eq!(c.tip().id, tip0);
    assert_eq!(pool(&c, &net), pool0);
    assert_eq!(c.state().scalars().last_settled, settled0);
    assert_eq!(c.status().state_root, root0);

    // a retry halts at the same height with the same message
    let again = panic_text(catch_unwind(AssertUnwindSafe(|| mine(&mut c, miner, 0, &[]))).err().expect("still halted"));
    assert_eq!(again, msg);

    // replay: a second node fed the same 14 blocks halts at the same block
    let mut d = net.open("halt-b");
    feed(&mut d, &blocks);
    assert_eq!(d.tip().height, 14);
    assert_eq!(d.status().state_root, root0);
    let replayed = panic_text(catch_unwind(AssertUnwindSafe(|| mine(&mut d, miner, 0, &[]))).err().expect("the replay halts"));
    assert_eq!(replayed, msg);
    assert_eq!(d.tip().height, 14);
    assert_eq!(d.status().state_root, root0);
}
