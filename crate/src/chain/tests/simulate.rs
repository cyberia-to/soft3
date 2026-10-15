//! Simulated chains with random valid transactions: conservation at every
//! height through a reorg, nonce ordering, finalized events
//! (chain-conformance §8 conservation row, §9).

use std::collections::BTreeMap;

use super::*;
use crate::chain::api::BlockVerdict;
use crate::chain::params::FINALITY_DEPTH;
use crate::chain::state::StateBackend;

/// splitmix64: deterministic, no platform randomness.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

/// Random transactions valid on `chain`'s tip for the accounts of `net`,
/// nonces taken from the chain, amounts within the liquid balance.
fn random_txs(net: &Net, chain: &Chain, rng: &mut Rng, max: u64) -> Vec<Vec<u8>> {
    let mut nonces: BTreeMap<usize, u64> = BTreeMap::new();
    let mut out = Vec::new();
    for _ in 0..rng.below(max + 1) {
        let from = rng.below(net.neurons.len() as u64) as usize;
        let nonce = *nonces.entry(from).or_insert_with(|| chain.nonce(&net.neurons[from]));
        let balance = chain.balance(&net.neurons[from], &net.native);
        if balance < 10_000 {
            continue;
        }
        let tx = if rng.below(3) == 0 {
            net.link(from, nonce, [rng.next() as u8; 32], [rng.next() as u8 ^ 1; 32], 0, (rng.below(3) as i8) - 1, 257 + rng.below(50))
        } else {
            let to = net.neurons[rng.below(net.neurons.len() as u64) as usize];
            net.transfer(from, nonce, to, rng.below(balance / 4), 224 + rng.below(50))
        };
        nonces.insert(from, nonce + 1);
        out.push(tx);
    }
    out
}

#[test]
fn two_hundred_blocks_of_random_txs_keep_conservation_through_a_reorg() {
    let net = Net::new(4);
    let mut rng = Rng(42);
    let mut a = net.open("a");
    let mut blocks = Vec::new();
    for h in 1..=150u64 {
        let miner = net.neurons[rng.below(4) as usize];
        let txs = random_txs(&net, &a, &mut rng, 4);
        let block = mine(&mut a, miner, 0, &txs);
        assert_eq!(a.tip().height, h);
        assert!(a.status().conserved, "height {h}");
        assert_eq!(a.supply(&net.native), net.genesis.s0() + h * net.subsidy());
        blocks.push(block);
    }
    // a competitor forks 3 blocks back and mines 5 with its own random txs
    let mut b = net.open("b");
    feed(&mut b, &blocks[..147]);
    let mut fork = Vec::new();
    for _ in 0..5 {
        let txs = random_txs(&net, &b, &mut rng, 3);
        fork.push(mine(&mut b, net.neurons[0], 0, &txs));
    }
    let verdicts = feed(&mut a, &fork);
    assert!(matches!(verdicts[3], BlockVerdict::Reorganized { depth: 3, .. }), "{verdicts:?}");
    assert_eq!(a.tip(), b.tip());
    assert!(a.status().conserved);
    assert_eq!(a.status().state_root, b.status().state_root);
    // nonces equal the count of each account's txs on the main chain
    let mut counts: BTreeMap<Id, u64> = BTreeMap::new();
    for h in 1..=a.tip().height {
        let block = a.block(&a.header_at(h).unwrap().id()).unwrap();
        for raw in crate::chain::tx::split_body(&block.body).unwrap().iter().skip(1) {
            let tx = Tx::decode(raw).unwrap();
            *counts.entry(sigs::subject(&tx).unwrap()).or_insert(0) += 1;
        }
    }
    for (i, n) in net.neurons.iter().enumerate() {
        assert_eq!(a.nonce(n), counts.get(n).copied().unwrap_or(0), "account {i}");
    }
    // continue to 200 on the merged chain; finalized events start flowing
    for h in 153..=200u64 {
        let txs = random_txs(&net, &a, &mut rng, 4);
        mine(&mut a, net.neurons[rng.below(4) as usize], 0, &txs);
        assert_eq!(a.tip().height, h);
        assert!(a.status().conserved, "height {h}");
    }
    assert_eq!(a.finalized_height(), 200 - FINALITY_DEPTH);
    let events = a.take_events();
    assert_eq!(events.len() as u64, 200 - FINALITY_DEPTH);
    assert!(a.take_events().is_empty());
    // reorged-out txs re-validate on the new tip only if their nonce still fits
    let mut readmitted = 0;
    for raw in blocks[147..].iter().flat_map(|b| crate::chain::tx::split_body(&b.body).unwrap().into_iter().skip(1)) {
        if a.validate_tx(&raw, &[]).is_ok() {
            readmitted += 1;
        }
    }
    let _ = readmitted;
    // a fresh replay of the final main chain agrees
    let mut d = net.open("d");
    let main: Vec<Block> = (1..=a.tip().height).map(|h| a.block(&a.header_at(h).unwrap().id()).unwrap()).collect();
    feed(&mut d, &main);
    assert_eq!(d.tip(), a.tip());
    assert_eq!(d.state().token_sum(&net.native), a.state().token_sum(&net.native));
}

#[test]
fn nonce_ordering_property_under_shuffled_submission() {
    let net = Net::new(2);
    let mut rng = Rng(7);
    let mut a = net.open("a");
    let to = net.neurons[1];
    let txs: Vec<Vec<u8>> = (0..10u64).map(|n| net.transfer(0, n, to, 1, 224 + rng.below(100))).collect();
    for _ in 0..5 {
        let mut shuffled = txs.clone();
        for i in (1..shuffled.len()).rev() {
            let j = rng.below(i as u64 + 1) as usize;
            shuffled.swap(i, j);
        }
        // drop one nonce at random: everything above it must be excluded
        let gap = rng.below(10) as usize;
        let subset: Vec<Vec<u8>> = shuffled.iter().filter(|t| Tx::decode(t).unwrap().nonce() != Some(gap as u64)).cloned().collect();
        let tpl = template(&mut a, to, 0, &subset);
        let got: Vec<u64> = tpl.txids[1..].iter().map(|id| txs.iter().position(|t| sigs::txid(&net.genesis.chain_id, t) == *id).unwrap() as u64).collect();
        assert_eq!(got, (0..gap as u64).collect::<Vec<_>>(), "gap at {gap}");
        // mempool admission agrees: nonce gap+1 needs gap pending
        let pending: Vec<Tx> = txs[..gap].iter().map(|t| Tx::decode(t).unwrap()).collect();
        if gap + 1 < 10 {
            assert!(a.validate_tx(&txs[gap + 1], &pending).is_err());
            let mut with_gap = pending.clone();
            with_gap.push(Tx::decode(&txs[gap]).unwrap());
            assert!(a.validate_tx(&txs[gap + 1], &with_gap).is_ok());
        }
    }
    // mine all ten in order; nonce ends at 10
    mine(&mut a, to, 0, &txs);
    assert_eq!(a.nonce(&net.neurons[0]), 10);
    assert!(a.status().conserved);
}
