//! chain-conformance §14 blessed scenarios (chain-economy §10), produced
//! by the reference implementation for `conformance/chain.snap`.
//!
//! The scenarios run the block transition itself (`apply::transition`),
//! not the miner: block ids are `hemera(D_BLOCK ‖ height ‖ prev ‖ root)`,
//! there is no header and no proof of work, so the binary can bless them
//! (`soft3 conformance bless chain`) without the test-only PoW limit. Every
//! economy rule on the path — coinbase split, settlement at
//! `boundary + D`, snapshots keyed by boundary id, the journal that a
//! reorganization reverts — is the kernel's own code.

use std::sync::Arc;

use crate::genesis::file::{Balance, GenesisFile, Link};

use super::super::apply::{self, Decoded, Locked, Rules};
use super::super::bbg_state::BbgBacked;
use super::super::genesis::{Genesis, Params};
use super::super::params::{EpochClock, GENESIS_BITS};
use super::super::sigs::{self, neuron_of, token, txid};
use super::super::state::{Id, StateBackend, Undo};
use super::super::tx::{Body, Tx};
use super::focus::{FocusEngine, FocusGraph, Tru};
use super::run::Economy;
use super::settle::Outcome;
use super::weights::GenesisWeights;
use super::{card_id, focus_key, phi_digest};

/// The economy scenarios' clock: L = 10, D = 5, MIN_LOCK = 15.
pub const CLOCK: EpochClock = EpochClock { length: 10, delay: 5, min_lock: 15 };
const D_BLOCK: &[u8] = b"soft3/conformance/block/v1\0";
const FUNDS: u64 = 1_000_000_000_000_000;
const FEE: u64 = 257;

/// The three signers: the §6 known-answer key and two fixed secrets.
pub fn keys() -> Vec<mudra::SigningKey> {
    vec![
        super::super::conformance::known_answer_key(),
        mudra::SigningKey::from_bytes((&[2u8; 32]).into()).expect("scalar"),
        mudra::SigningKey::from_bytes((&[3u8; 32]).into()).expect("scalar"),
    ]
}

pub fn neuron(k: &mudra::SigningKey) -> Id {
    neuron_of(&mudra::cosmos::compressed(k.verifying_key()))
}

/// The economy conformance net: the three signers funded with 10^15 pussy,
/// and four genesis links among `boot`, `pussy` and `cyber` (genesis
/// weights: each author's balance spread over its links).
pub fn genesis() -> Genesis {
    let ks = keys();
    let n: Vec<Id> = ks.iter().map(neuron).collect();
    let (boot, pussy, cyber) = (token("boot"), token("pussy"), token("cyber"));
    let mut balances: Vec<Balance> = n.iter().map(|o| Balance { owner: *o, token: pussy, amount: FUNDS }).collect();
    balances.sort();
    let s0 = FUNDS * 3;
    Genesis::from_file(&GenesisFile {
        params: Params {
            version: 1,
            name: "conformance-economy".into(),
            hrp: "pussy".into(),
            denom: "pussy".into(),
            time: 0,
            bits: GENESIS_BITS,
            subsidy: super::super::params::subsidy(s0),
            checkpoint_key: [0; 33],
            checkpoint_sunset: 0,
            pool: super::super::pool::Pool::genesis(),
        },
        balances,
        links: vec![
            Link { neuron: n[0], p: boot, q: pussy, height: 1 },
            Link { neuron: n[1], p: pussy, q: cyber, height: 2 },
            Link { neuron: n[1], p: pussy, q: boot, height: 3 },
            Link { neuron: n[2], p: cyber, q: boot, height: 4 },
        ],
    })
    .expect("the economy conformance genesis is canonical")
}

/// A chain driven block by block through the transition.
pub struct Driver {
    pub genesis: Genesis,
    pub rules: Rules,
    pub state: BbgBacked,
    pub economy: Economy,
    /// block id per height, 0 = genesis
    pub ids: Vec<Id>,
    pub roots: Vec<Id>,
    undos: Vec<Undo>,
    pub outcomes: Vec<Option<Outcome>>,
    keys: Vec<mudra::SigningKey>,
    pub nonces: Vec<u64>,
}

impl Driver {
    pub fn new(engine: Arc<dyn FocusEngine>) -> Driver {
        let genesis = genesis();
        let rules = Rules {
            chain: genesis.chain_id,
            native: genesis.native_token(),
            hrp: genesis.params().hrp.clone(),
            subsidy: genesis.subsidy(),
            s0: genesis.s0(),
            clock: CLOCK,
        };
        let mut state = genesis.state();
        let root = state.root();
        let id0 = genesis.block0(root).id();
        let mut economy = Economy::new(GenesisWeights::from_file(&genesis.file), engine, CLOCK, None);
        economy.background = false;
        economy.on_boundary(&state, 0, id0, root, rules.s0, None);
        Driver { genesis, rules, state, economy, ids: vec![id0], roots: vec![root], undos: vec![], outcomes: vec![None], keys: keys(), nonces: vec![0; 3] }
    }

    pub fn height(&self) -> u64 {
        self.ids.len() as u64 - 1
    }

    pub fn neuron(&self, i: usize) -> Id {
        neuron(&self.keys[i])
    }

    fn sign(&mut self, i: usize, body: Body) -> Vec<u8> {
        let mut tx = Tx { body, sig: vec![] };
        sigs::sign(&self.rules.chain, &mut tx, &self.keys[i]).expect("own key");
        self.nonces[i] += 1;
        tx.encode()
    }

    pub fn lock(&mut self, i: usize, p: Id, q: Id, amount: u64, valence: i8) -> Vec<u8> {
        let (from, nonce, token) = (self.neuron(i), self.nonces[i], self.rules.native);
        self.sign(i, Body::Cyberlink { from, nonce, token, amount, p, q, valence, fee: FEE })
    }

    pub fn unlock(&mut self, i: usize, p: Id, q: Id, amount: u64) -> Vec<u8> {
        let (from, nonce) = (self.neuron(i), self.nonces[i]);
        self.sign(i, Body::Unlock { from, nonce, p, q, amount, fee: 224 })
    }

    pub fn payout(&mut self, i: usize, p: Id, q: Id, payout: Id) -> Vec<u8> {
        let (from, nonce) = (self.neuron(i), self.nonces[i]);
        self.sign(i, Body::Payout { from, nonce, p, q, payout, fee: 248 })
    }

    pub fn transfer(&mut self, i: usize, to: Id, amount: u64) -> Vec<u8> {
        let (from, nonce, token) = (self.neuron(i), self.nonces[i], self.rules.native);
        self.sign(i, Body::Transfer { from, nonce, token, amount, to, fee: 224 })
    }

    /// Does `raw` execute on the current state (as the next block would see it)?
    pub fn valid(&mut self, raw: &[u8]) -> bool {
        let Ok(tx) = Tx::decode(raw) else { return false };
        let h = self.height() + 1;
        self.state.begin();
        let ok = apply::execute_tx(&self.rules, &tx, raw.len(), &mut self.state, &Locked::default(), h).is_ok();
        let undo = self.state.commit();
        self.state.revert(&undo);
        ok
    }

    /// Apply the next block with `txs` (coinbase to `0x55 × 32` prepended).
    pub fn block(&mut self, txs: &[Vec<u8>]) -> Result<(), apply::BlockError> {
        let h = self.height() + 1;
        let fees: u64 = txs.iter().map(|t| Tx::decode(t).map(|t| t.fee()).unwrap_or(0)).sum();
        let (miner, _) = apply::subsidy_split(&self.rules, &self.state);
        let cb = Tx { body: Body::Coinbase { height: h, to: [0x55; 32], amount: miner + fees, extra: [0; 32] }, sig: vec![] }.encode();
        let decoded: Vec<Decoded> = std::iter::once(cb)
            .chain(txs.iter().cloned())
            .map(|raw| Decoded { tx: Tx::decode(&raw).expect("well-formed"), txid: txid(&self.rules.chain, &raw), raw })
            .collect();
        let settle = self.rules.clock.settles_at(h).and_then(|e| {
            let (prev, cur) = (self.ids[CLOCK.boundary(e - 1) as usize], self.ids[CLOCK.boundary(e) as usize]);
            self.economy.result(&prev, &cur)
        });
        let applied = apply::transition(&self.rules, h, &decoded, &mut self.state, &Locked::default(), settle.as_deref())?;
        let mut buf = D_BLOCK.to_vec();
        buf.extend_from_slice(&h.to_le_bytes());
        buf.extend_from_slice(self.ids.last().expect("genesis"));
        buf.extend_from_slice(&applied.root);
        let id = *hemera::hash(&buf).as_bytes();
        self.ids.push(id);
        self.roots.push(applied.root);
        self.undos.push(applied.undo);
        self.outcomes.push(applied.settled);
        if CLOCK.is_boundary(h) {
            let e = CLOCK.epoch(h);
            let prev = self.ids[CLOCK.boundary(e - 1) as usize];
            let supply = self.rules.supply_at(h);
            self.economy.on_boundary(&self.state, e, id, applied.root, supply, Some(prev));
        }
        Ok(())
    }

    pub fn empty_to(&mut self, h: u64) {
        while self.height() < h {
            self.block(&[]).expect("an empty block applies");
        }
    }

    /// Undo the tip block (a reorganization's first half).
    pub fn retract(&mut self) {
        let undo = self.undos.pop().expect("above genesis");
        self.state.revert(&undo);
        self.ids.pop();
        self.roots.pop();
        self.outcomes.pop();
    }

    pub fn outcome(&self, h: u64) -> &Outcome {
        self.outcomes[h as usize].as_ref().expect("a settlement block")
    }
}

fn h256(b: &[u8]) -> Id {
    *hemera::hash(b).as_bytes()
}

fn u64s(vals: &[u64]) -> Id {
    let mut out = [0u8; 32];
    for (i, v) in vals.iter().take(4).enumerate() {
        out[i * 8..i * 8 + 8].copy_from_slice(&v.to_le_bytes());
    }
    out
}

/// `(scenario, value)` pairs for `chain::economy@v1`.
pub fn entries() -> Vec<(String, Id)> {
    let mut out: Vec<(String, Id)> = Vec::new();
    let (boot, pussy, cyber) = (token("boot"), token("pussy"), token("cyber"));
    let tru: Arc<dyn FocusEngine> = Arc::new(Tru::new(1));

    // transactions of kinds 2 (with stake), 6 and 7, §6 key, chain 0x44 × 32
    let chain = [0x44u8; 32];
    let key = super::super::conformance::known_answer_key();
    let from = neuron(&key);
    let signed = |body: Body| {
        let mut tx = Tx { body, sig: vec![] };
        sigs::sign(&chain, &mut tx, &key).expect("own key");
        tx.encode()
    };
    let stake = signed(Body::Cyberlink { from, nonce: 0, token: pussy, amount: 7, p: boot, q: pussy, valence: 1, fee: 257 });
    let unlock = signed(Body::Unlock { from, nonce: 1, p: boot, q: pussy, amount: 7, fee: 224 });
    let payout = signed(Body::Payout { from, nonce: 2, p: boot, q: pussy, payout: [0x77; 32], fee: 248 });
    assert_eq!((stake.len(), unlock.len(), payout.len()), (257, 224, 248));
    out.push(("signed stake cyberlink (amount 7)".into(), h256(&stake)));
    out.push(("signed unlock (kind 6)".into(), h256(&unlock)));
    out.push(("signed payout (kind 7)".into(), h256(&payout)));

    // genesis weights and φ*_0 of the economy conformance net
    let g = genesis();
    let w = GenesisWeights::from_file(&g.file);
    out.push(("conformance genesis weights digest".into(), w.digest()));
    out.push(("conformance genesis weights total, at floor".into(), u64s(&[w.total as u64, w.at_floor])));
    let adjacency: Vec<(Id, Id, u128)> = w.axons.iter().map(|(p, q, g)| (*p, *q, *g as u128)).collect();
    let f0 = tru.focus(&FocusGraph::build(&adjacency));
    out.push(("conformance phi*_0 digest".into(), phi_digest(&f0.phi)));
    out.push(("conformance phi*_0 syntropy".into(), u64s(&[f0.syntropy as u64, (f0.syntropy >> 64) as u64])));

    // a 2-epoch chain: +1, 0 and −1 locks in epoch 1
    let mut d = Driver::new(tru.clone());
    let txs = vec![d.lock(0, boot, pussy, FUNDS / 1_000, 1), d.lock(1, pussy, cyber, FUNDS / 1_000, 0), d.lock(2, pussy, boot, FUNDS / 4_000, -1)];
    d.block(&txs).expect("locks apply");
    d.empty_to(25);
    for e in [1u64, 2] {
        let o = d.outcome(CLOCK.settle(e)).clone();
        out.push((format!("two epochs: epoch {e} record"), o.record.digest()));
        out.push((format!("two epochs: epoch {e} payouts"), o.record.payouts));
        out.push((format!("two epochs: epoch {e} budget, paid, pool after"), u64s(&[o.record.budget, o.record.paid, o.record.pool_after])));
    }
    out.push(("two epochs: state root at 25".into(), d.roots[25]));
    let passive = card_id(&d.neuron(1), &pussy, &cyber);
    assert!(d.outcome(15).payouts.iter().all(|(c, _, _)| *c != passive), "the passive position earns nothing");

    // reorganizations: three blocks above the boundary reproduce epoch 1's
    // record; replacing the boundary block changes it
    let main = d.outcome(15).record.clone();
    let mut r = Driver::new(tru.clone());
    let txs = vec![r.lock(0, boot, pussy, FUNDS / 1_000, 1), r.lock(1, pussy, cyber, FUNDS / 1_000, 0), r.lock(2, pussy, boot, FUNDS / 4_000, -1)];
    r.block(&txs).expect("locks apply");
    r.empty_to(15);
    for _ in 0..3 {
        r.retract();
    }
    let t = r.transfer(0, [0x77; 32], 5);
    r.block(&[t]).expect("a transfer applies");
    r.empty_to(15);
    assert_eq!(r.outcome(15).record, main, "a reorg above the boundary reproduces the record");
    out.push(("reorg above the boundary: epoch 1 record".into(), r.outcome(15).record.digest()));
    while r.height() > 9 {
        r.retract();
    }
    let extra = r.lock(1, cyber, boot, FUNDS / 2_000, 1);
    r.block(&[extra]).expect("a lock in the new boundary block");
    r.empty_to(15);
    assert_ne!(r.outcome(15).record, main, "a new boundary block is a new snapshot");
    out.push(("reorg of the boundary block: epoch 1 record".into(), r.outcome(15).record.digest()));

    // the new-high rule: lock, unlock after MIN_LOCK, relock one epoch later
    let mut n = Driver::new(tru.clone());
    let amount = FUNDS / 1_000;
    let t = n.lock(0, boot, pussy, amount, 1);
    n.block(&[t]).expect("lock");
    n.empty_to(1 + CLOCK.min_lock - 1);
    let t = n.unlock(0, boot, pussy, amount);
    n.block(&[t]).expect("unlock at since + MIN_LOCK");
    n.empty_to(30);
    let t = n.lock(0, boot, pussy, amount, 1);
    n.block(&[t]).expect("relock");
    n.empty_to(45);
    let card = card_id(&n.neuron(0), &boot, &pussy);
    let first = n.outcome(15).payouts.iter().find(|x| x.0 == card).map(|x| x.2).unwrap_or(0);
    let again = n.outcome(45).payouts.iter().find(|x| x.0 == card).map(|x| x.2).unwrap_or(0);
    out.push(("new high: first lock and relock paid".into(), u64s(&[first, again])));
    out.push(("new high: epoch 4 record".into(), n.outcome(45).record.digest()));

    // conservation over 2,000 blocks of random locks, unlocks and payout changes
    let mut c = Driver::new(tru);
    let mut s = 0x2000_u64;
    let mut next = || {
        s = s.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = s;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    };
    let tokens = [boot, pussy, cyber, token("tru"), token("hemera")];
    for _ in 1..=2_000u64 {
        let mut txs = Vec::new();
        for _ in 0..(next() % 3) {
            let i = (next() % 3) as usize;
            let (p, q) = (tokens[(next() % 5) as usize], tokens[(next() % 5) as usize]);
            if p == q {
                continue;
            }
            let held = c.state.card(&card_id(&c.neuron(i), &p, &q));
            let raw = match next() % 4 {
                0 | 1 => {
                    let v = held.map(|x| x.valence).unwrap_or((next() % 3) as i8 - 1);
                    c.lock(i, p, q, 1 + next() % (FUNDS / 10_000), v)
                }
                2 => c.unlock(i, p, q, 1 + next() % (FUNDS / 10_000)),
                _ => {
                    let to = c.neuron((next() % 3) as usize);
                    c.payout(i, p, q, to)
                }
            };
            // a tx that would fail is dropped and its nonce given back
            let mut trial = txs.clone();
            trial.push(raw.clone());
            if c.valid_sequence(&trial) {
                txs.push(raw);
            } else {
                c.nonces[i] -= 1;
            }
        }
        c.block(&txs).expect("only valid transactions");
        apply::check_conservation(&c.rules, &c.state, c.height()).expect("conserved at every height");
        apply::check_stake_total(&c.state).expect("stake keys equal the running total");
    }
    let pool = c.state.raw_balance(&focus_key(&c.rules.native));
    out.push(("random 2,000 blocks: state root".into(), c.roots[2_000]));
    out.push(("random 2,000 blocks: pool, total stake".into(), u64s(&[pool, c.state.scalars().total_stake])));
    out
}

impl Driver {
    /// Do these transactions execute in order on the current state?
    pub fn valid_sequence(&mut self, raws: &[Vec<u8>]) -> bool {
        let h = self.height() + 1;
        self.state.begin();
        let mut ok = true;
        for raw in raws {
            let Ok(tx) = Tx::decode(raw) else {
                ok = false;
                break;
            };
            if apply::execute_tx(&self.rules, &tx, raw.len(), &mut self.state, &Locked::default(), h).is_err() {
                ok = false;
                break;
            }
        }
        let undo = self.state.commit();
        self.state.revert(&undo);
        ok
    }
}
