//! [`StateBackend`] over `bbg::BbgState` (§3.2–§3.3).
//!
//! Balances, nonces, coins and the graph live in the bbg dimensions and
//! therefore under `state_root`. The per-token running sums are derived
//! bookkeeping for the conservation check (§3.4), never committed.
//!
//! Nonce placement: with the `bbg-nonce` feature the nonce is
//! `NeuronRecord.nonce` (bbg DIMENSION_VERSION 3, the change requested in
//! §3.2). Without it — until the bbg PR carrying the field lands — the
//! nonce sits in a side map beside the state, outside the root; the journal
//! and every rule are identical, only the root differs.

use std::collections::BTreeMap;

use bbg::state::{axon_id, balance_key};
use bbg::types::{CoinRecord, NeuronRecord, ParticleRecord};
use bbg::{BbgState, Cyberlink, Signal};

use super::state::{Id, StateBackend, StateError, Undo, UndoEntry};

pub struct BbgBacked {
    pub bbg: BbgState,
    sums: BTreeMap<Id, u64>,
    #[cfg(not(feature = "bbg-nonce"))]
    nonces: BTreeMap<Id, u64>,
    journal: Vec<UndoEntry>,
    open: bool,
}

impl Default for BbgBacked {
    fn default() -> Self {
        Self::new()
    }
}

fn neuron_image(r: &NeuronRecord) -> [u64; 4] {
    #[cfg(feature = "bbg-nonce")]
    let nonce = r.nonce;
    #[cfg(not(feature = "bbg-nonce"))]
    let nonce = 0;
    [r.focus, r.karma, r.stake, nonce]
}

fn neuron_from(img: [u64; 4]) -> NeuronRecord {
    #[cfg(feature = "bbg-nonce")]
    return NeuronRecord { focus: img[0], karma: img[1], stake: img[2], nonce: img[3] };
    #[cfg(not(feature = "bbg-nonce"))]
    NeuronRecord { focus: img[0], karma: img[1], stake: img[2] }
}

fn particle_image(r: &ParticleRecord) -> [u64; 6] {
    [r.energy, r.pi_star, r.weight, r.s_yes, r.s_no, r.meta_score]
}

fn particle_from(i: [u64; 6]) -> ParticleRecord {
    ParticleRecord { energy: i[0], pi_star: i[1], weight: i[2], s_yes: i[3], s_no: i[4], meta_score: i[5] }
}

impl BbgBacked {
    pub fn new() -> Self {
        Self {
            bbg: BbgState::new(),
            sums: BTreeMap::new(),
            #[cfg(not(feature = "bbg-nonce"))]
            nonces: BTreeMap::new(),
            journal: Vec::new(),
            open: false,
        }
    }

    /// Wrap a loaded state (the genesis loader's) with its per-token sums,
    /// given as `(token, amount)` contributions in any order.
    pub fn from_bbg(bbg: BbgState, contributions: impl IntoIterator<Item = (Id, u64)>) -> Self {
        let mut s = Self::new();
        s.bbg = bbg;
        for (token, amount) in contributions {
            let e = s.sums.entry(token).or_insert(0);
            *e = e.checked_add(amount).expect("genesis sums fit u64 (checked by the pipeline)");
        }
        s
    }

    fn note(&mut self, e: UndoEntry) {
        if self.open {
            self.journal.push(e);
        }
    }

    fn note_balance(&mut self, key: Id) {
        let before = self.bbg.balances.get(&key).copied();
        self.note(UndoEntry::Balance { key, before });
    }

    fn note_neuron(&mut self, id: Id) {
        let before = self.bbg.neurons.get(&id).map(neuron_image);
        self.note(UndoEntry::Neuron { id, before });
    }

    fn note_particle(&mut self, id: Id) {
        let before = self.bbg.particles.get(&id).map(particle_image);
        self.note(UndoEntry::Particle { id, before });
    }

    fn note_sum(&mut self, token: Id) {
        let before = self.sums.get(&token).copied();
        self.note(UndoEntry::TokenSum { token, before });
    }

    #[cfg(not(feature = "bbg-nonce"))]
    fn nonce_image(&self, id: &Id) -> Option<[u64; 4]> {
        let n = self.nonces.get(id).copied();
        let r = self.bbg.neurons.get(id).map(neuron_image);
        match (r, n) {
            (None, None) => None,
            (r, n) => {
                let mut img = r.unwrap_or([0; 4]);
                img[3] = n.unwrap_or(0);
                Some(img)
            }
        }
    }

    /// the neuron record image with the nonce wherever it lives
    fn full_neuron(&self, id: &Id) -> Option<[u64; 4]> {
        #[cfg(feature = "bbg-nonce")]
        return self.bbg.neurons.get(id).map(neuron_image);
        #[cfg(not(feature = "bbg-nonce"))]
        self.nonce_image(id)
    }

    fn write_neuron(&mut self, id: Id, img: Option<[u64; 4]>) {
        match img {
            None => {
                self.bbg.neurons.remove(&id);
                #[cfg(not(feature = "bbg-nonce"))]
                self.nonces.remove(&id);
            }
            Some(img) => {
                self.bbg.neurons.insert(id, neuron_from(img));
                #[cfg(not(feature = "bbg-nonce"))]
                self.nonces.insert(id, img[3]);
            }
        }
    }

    fn set_balance(&mut self, key: Id, value: Option<u64>) {
        match value {
            None => self.bbg.balances.remove(&key),
            Some(v) => self.bbg.balances.insert(key, v),
        };
    }

    fn dirty(&mut self) {
        // any journaled write invalidates bbg's cached root; bbg exposes no
        // public "mark dirty", so refresh lazily in `root()` instead
        let _ = &self.bbg;
    }
}

impl StateBackend for BbgBacked {
    fn height(&self) -> u64 {
        self.bbg.height
    }

    fn balance(&self, owner: &Id, token: &Id) -> u64 {
        self.bbg.balances.get(&balance_key(owner, token)).copied().unwrap_or(0)
    }

    fn nonce(&self, id: &Id) -> u64 {
        self.full_neuron(id).map_or(0, |i| i[3])
    }

    fn has_neuron(&self, id: &Id) -> bool {
        self.bbg.neurons.contains_key(id)
    }

    fn supply(&self, token: &Id) -> u64 {
        self.bbg.coins.get(token).map_or(0, |c| c.total_supply)
    }

    fn tokens(&self) -> Vec<Id> {
        self.bbg.coins.keys().copied().collect()
    }

    fn token_sum(&self, token: &Id) -> u64 {
        self.sums.get(token).copied().unwrap_or(0)
    }

    fn neuron_count(&self) -> u64 {
        self.bbg.neurons.len() as u64
    }

    fn particle_count(&self) -> u64 {
        self.bbg.particles.len() as u64
    }

    fn axon_count(&self) -> u64 {
        self.bbg.axon_edges.len() as u64
    }

    fn root(&mut self) -> Id {
        self.bbg.refresh_root()
    }

    fn set_height(&mut self, height: u64) {
        let before = self.bbg.height;
        self.note(UndoEntry::Height { before });
        self.bbg.height = height;
    }

    fn credit(&mut self, owner: &Id, token: &Id, amount: u64) -> Result<(), StateError> {
        let key = balance_key(owner, token);
        let have = self.bbg.balances.get(&key).copied().unwrap_or(0);
        let next = have.checked_add(amount).ok_or(StateError::Overflow)?;
        let sum = self.sums.get(token).copied().unwrap_or(0).checked_add(amount).ok_or(StateError::Overflow)?;
        self.note_balance(key);
        self.note_sum(*token);
        self.bbg.balances.insert(key, next);
        self.sums.insert(*token, sum);
        self.dirty();
        Ok(())
    }

    fn debit(&mut self, owner: &Id, token: &Id, amount: u64) -> Result<(), StateError> {
        let key = balance_key(owner, token);
        let have = self.bbg.balances.get(&key).copied().unwrap_or(0);
        if have < amount {
            return Err(StateError::Insufficient { have, need: amount });
        }
        self.note_balance(key);
        self.note_sum(*token);
        self.bbg.balances.insert(key, have - amount);
        let sum = self.sums.get(token).copied().unwrap_or(0);
        self.sums.insert(*token, sum - amount);
        self.dirty();
        Ok(())
    }

    fn ensure_neuron(&mut self, id: &Id) {
        if self.full_neuron(id).is_some() {
            return;
        }
        self.note_neuron(*id);
        self.write_neuron(*id, Some([0; 4]));
        self.dirty();
    }

    fn bump_nonce(&mut self, id: &Id) -> Result<(), StateError> {
        let mut img = self.full_neuron(id).unwrap_or([0; 4]);
        img[3] = img[3].checked_add(1).ok_or(StateError::Overflow)?;
        let before = self.full_neuron(id);
        self.note(UndoEntry::Neuron { id: *id, before });
        self.write_neuron(*id, Some(img));
        self.dirty();
        Ok(())
    }

    fn mint(&mut self, token: &Id, amount: u64) -> Result<(), StateError> {
        let before = self.bbg.coins.get(token).map(|c| c.total_supply);
        let next = before.unwrap_or(0).checked_add(amount).ok_or(StateError::Overflow)?;
        self.note(UndoEntry::Coin { token: *token, before });
        self.bbg.coins.insert(*token, CoinRecord { total_supply: next });
        self.dirty();
        Ok(())
    }

    fn link(&mut self, from: &Id, p: &Id, q: &Id, token: &Id, amount: u64, valence: i8, height: u64) -> Result<(), StateError> {
        assert_eq!(amount, 0, "rule set 1 carries no boxes (chain.md §10 item 2)");
        let axon = axon_id(p, q);
        // before-images of everything bbg's insert touches for one link
        self.note_particle(axon);
        self.note_particle(*q);
        let out_before = self.bbg.axons_out.get(p).map(|v| v.len() as u64);
        self.note(UndoEntry::AxonsOut { p: *p, before: out_before });
        let in_before = self.bbg.axons_in.get(q).map(|v| v.len() as u64);
        self.note(UndoEntry::AxonsIn { q: *q, before: in_before });
        let existed = self.bbg.axon_edges.contains_key(&axon);
        self.note(UndoEntry::AxonEdge { axon, existed });
        self.note_balance(balance_key(q, token));
        self.note_balance(balance_key(p, token));
        self.note_neuron(*from);
        let signal = Signal {
            neuron: *from,
            links: vec![Cyberlink { from: *p, to: *q, token: *token, amount, valence }],
            box_moves: Vec::new(),
            height,
        };
        self.bbg.insert(&signal).map_err(|_| StateError::Overflow)?;
        Ok(())
    }

    fn begin(&mut self) {
        self.journal.clear();
        self.open = true;
    }

    fn commit(&mut self) -> Undo {
        self.open = false;
        Undo(std::mem::take(&mut self.journal))
    }

    fn revert(&mut self, undo: &Undo) {
        for e in undo.0.iter().rev() {
            match e {
                UndoEntry::Balance { key, before } => self.set_balance(*key, *before),
                UndoEntry::Neuron { id, before } => self.write_neuron(*id, *before),
                UndoEntry::Coin { token, before } => match before {
                    None => {
                        self.bbg.coins.remove(token);
                    }
                    Some(v) => {
                        self.bbg.coins.insert(*token, CoinRecord { total_supply: *v });
                    }
                },
                UndoEntry::Particle { id, before } => match before {
                    None => {
                        self.bbg.particles.remove(id);
                    }
                    Some(img) => {
                        self.bbg.particles.insert(*id, particle_from(*img));
                    }
                },
                UndoEntry::AxonsOut { p, before } => match before {
                    None => {
                        self.bbg.axons_out.remove(p);
                    }
                    Some(n) => {
                        if let Some(v) = self.bbg.axons_out.get_mut(p) {
                            v.truncate(*n as usize);
                        }
                    }
                },
                UndoEntry::AxonsIn { q, before } => match before {
                    None => {
                        self.bbg.axons_in.remove(q);
                    }
                    Some(n) => {
                        if let Some(v) = self.bbg.axons_in.get_mut(q) {
                            v.truncate(*n as usize);
                        }
                    }
                },
                UndoEntry::AxonEdge { axon, existed } => {
                    if !existed {
                        self.bbg.axon_edges.remove(axon);
                    }
                }
                UndoEntry::TokenSum { token, before } => match before {
                    None => {
                        self.sums.remove(token);
                    }
                    Some(v) => {
                        self.sums.insert(*token, *v);
                    }
                },
                UndoEntry::Height { before } => self.bbg.height = *before,
            }
        }
        // bbg caches its root until a mutation through its own API; the
        // reverts above bypass that, so force the next root() to recompute
        self.bbg.refresh_root();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn journal_reverts_every_dimension_exactly() {
        let mut s = BbgBacked::new();
        let (a, b, t) = ([1u8; 32], [2u8; 32], [9u8; 32]);
        s.credit(&a, &t, 100).unwrap();
        s.mint(&t, 100).unwrap();
        s.ensure_neuron(&a);
        s.link(&a, &[5; 32], &[6; 32], &t, 0, 1, 0).unwrap();
        let root0 = s.root();
        let count0 = (s.bbg.particles.len(), s.bbg.axons_out.len(), s.bbg.balances.len(), s.bbg.neurons.len());

        s.begin();
        s.set_height(1);
        s.debit(&a, &t, 30).unwrap();
        s.credit(&b, &t, 30).unwrap();
        s.bump_nonce(&a).unwrap();
        s.bump_nonce(&b).unwrap();
        s.mint(&t, 5).unwrap();
        s.mint(&[8; 32], 5).unwrap();
        s.link(&b, &[5; 32], &[7; 32], &t, 0, -1, 1).unwrap();
        s.link(&b, &[5; 32], &[6; 32], &t, 0, 0, 1).unwrap();
        let undo = s.commit();
        assert_eq!(s.balance(&a, &t), 70);
        assert_eq!(s.nonce(&a), 1);
        assert_eq!(s.nonce(&b), 1);
        assert_eq!(s.token_sum(&t), 100);
        assert_eq!(s.supply(&t), 105);
        assert_eq!(s.height(), 1);
        assert_ne!(s.root(), root0);

        s.revert(&undo);
        assert_eq!(s.root(), root0);
        assert_eq!((s.bbg.particles.len(), s.bbg.axons_out.len(), s.bbg.balances.len(), s.bbg.neurons.len()), count0);
        assert_eq!(s.balance(&a, &t), 100);
        assert_eq!(s.balance(&b, &t), 0);
        assert_eq!(s.nonce(&a), 0);
        assert!(!s.has_neuron(&b));
        assert_eq!(s.tokens(), vec![t]);
        assert_eq!(s.height(), 0);
        assert_eq!(Undo::decode(&undo.encode()).unwrap(), undo);
    }

    #[test]
    fn debit_never_goes_negative_and_sums_track() {
        let mut s = BbgBacked::new();
        let (a, t) = ([1u8; 32], [9u8; 32]);
        assert_eq!(s.debit(&a, &t, 1), Err(StateError::Insufficient { have: 0, need: 1 }));
        s.credit(&a, &t, u64::MAX).unwrap();
        assert_eq!(s.credit(&a, &t, 1), Err(StateError::Overflow));
        assert_eq!(s.token_sum(&t), u64::MAX);
        s.debit(&a, &t, u64::MAX).unwrap();
        assert_eq!(s.token_sum(&t), 0);
    }
}
