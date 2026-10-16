//! The mempool — chain-wire.md §4: admission order, per-neuron nonce
//! ordering, capacity with fee-per-byte eviction, 24 h expiry,
//! re-validation after a new tip, and block templates by fee per byte with
//! nonce order preserved. The consensus questions of admission — signature,
//! nonce window, fee floor, balances and coinbase maturity on the tip — are
//! the kernel's (`ChainView::validate_tx` → `ChainApi::validate_tx`).

use std::collections::{BTreeMap, HashMap};

use super::chain_view::{ChainView, TxRefusal};
use super::codec::{txid, Id, Tx, MAX_BLOCK_BYTES, MAX_BLOCK_TXS, MAX_TX_BYTES, NONCE_WINDOW, HEADER_BYTES};

pub const CAPACITY_TXS: usize = 8192;
pub const CAPACITY_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_AGE_SECS: u64 = 24 * 3600;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdmitError {
    Decode(String),
    Known,
    Signature,
    Nonce { current: u64, got: u64 },
    /// inside the window but a lower nonce is missing from the pool
    NonceGap { expected: u64, got: u64 },
    NonceOccupied,
    Fee { floor: u64, got: u64 },
    Balance(String),
    Kind,
    /// valid encoding, signed, but against a transaction rule (self-link, …)
    Rule(String),
}

impl From<TxRefusal> for AdmitError {
    fn from(r: TxRefusal) -> Self {
        match r {
            TxRefusal::Decode(e) => AdmitError::Decode(e),
            TxRefusal::Kind => AdmitError::Kind,
            TxRefusal::Signature => AdmitError::Signature,
            TxRefusal::Nonce { current, got } => AdmitError::Nonce { current, got },
            TxRefusal::Fee { floor, got } => AdmitError::Fee { floor, got },
            TxRefusal::Balance(why) => AdmitError::Balance(why),
            TxRefusal::Rule(why) => AdmitError::Rule(why),
        }
    }
}

impl std::fmt::Display for AdmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdmitError::Decode(e) => write!(f, "decode: {e}"),
            AdmitError::Known => write!(f, "already in mempool"),
            AdmitError::Signature => write!(f, "invalid signature"),
            AdmitError::Nonce { current, got } => write!(f, "nonce {got} outside [{current}, {current} + {NONCE_WINDOW})"),
            AdmitError::NonceGap { expected, got } => write!(f, "nonce {got}: nonce {expected} is not pending"),
            AdmitError::NonceOccupied => write!(f, "nonce already pending"),
            AdmitError::Fee { floor, got } => write!(f, "fee {got} below floor {floor}"),
            AdmitError::Balance(why) => write!(f, "{why}"),
            AdmitError::Kind => write!(f, "kind not admissible"),
            AdmitError::Rule(why) => write!(f, "{why}"),
        }
    }
}

impl AdmitError {
    /// chain-wire §5: +10 for a bad signature, a nonce beyond the window or a
    /// non-canonical encoding; +1 for a duplicate; nothing for balance races.
    pub fn score(&self) -> u32 {
        match self {
            AdmitError::Signature | AdmitError::Decode(_) | AdmitError::Kind | AdmitError::Rule(_) => 10,
            AdmitError::Nonce { .. } => 10,
            AdmitError::Known => 1,
            AdmitError::NonceGap { .. } | AdmitError::NonceOccupied | AdmitError::Fee { .. } | AdmitError::Balance(_) => 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub bytes: Vec<u8>,
    pub tx: Tx,
    pub from: Id,
    pub nonce: u64,
    pub added: u64,
}

impl Entry {
    pub fn fee_per_byte(&self) -> u64 {
        self.tx.fee() / self.bytes.len() as u64
    }
}

#[derive(Default)]
pub struct Mempool {
    entries: HashMap<Id, Entry>,
    by_neuron: HashMap<Id, BTreeMap<u64, Id>>,
    bytes: usize,
}

impl Mempool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn bytes(&self) -> usize {
        self.bytes
    }

    pub fn contains(&self, id: &Id) -> bool {
        self.entries.contains_key(id)
    }

    pub fn get(&self, id: &Id) -> Option<&Entry> {
        self.entries.get(id)
    }

    pub fn ids(&self) -> Vec<Id> {
        self.entries.keys().copied().collect()
    }

    /// The next nonce a wallet should use: the chain's, advanced past pending txs.
    pub fn next_nonce(&self, chain: &dyn ChainView, neuron: &Id) -> u64 {
        let mut next = chain.nonce(neuron);
        if let Some(pending) = self.by_neuron.get(neuron) {
            while pending.contains_key(&next) {
                next += 1;
            }
        }
        next
    }

    /// Admission in the order of chain-wire §4: size and syntax, duplicate,
    /// then the kernel on the tip state with this neuron's lower pending
    /// nonces applied first, then the nonce slot.
    pub fn admit(&mut self, bytes: &[u8], chain: &dyn ChainView, now: u64) -> Result<Id, AdmitError> {
        if bytes.len() > MAX_TX_BYTES {
            return Err(AdmitError::Decode(format!("{} bytes", bytes.len())));
        }
        let tx = Tx::decode(bytes).map_err(|e| AdmitError::Decode(e.to_string()))?;
        if matches!(tx, Tx::Coinbase { .. }) {
            return Err(AdmitError::Kind);
        }
        let id = txid(&chain.chain_id(), bytes);
        if self.entries.contains_key(&id) {
            return Err(AdmitError::Known);
        }
        let from = tx.subject().ok_or(AdmitError::Signature)?;
        let nonce = tx.nonce().ok_or(AdmitError::Kind)?;
        let pending: Vec<Vec<u8>> = self.lower_nonces(&from, nonce).map(|e| e.bytes.clone()).collect();
        let admitted = chain.validate_tx(bytes, &pending).map_err(|r| match r {
            // the kernel names a gap and a nonce beyond the window alike; only the latter is scored (§5)
            TxRefusal::Nonce { current, got } if got > current && got < chain.nonce(&from).saturating_add(NONCE_WINDOW) => {
                AdmitError::NonceGap { expected: current, got }
            }
            r => r.into(),
        })?;
        debug_assert_eq!(admitted.txid, id, "the wire and the kernel agree on txids");
        if self.by_neuron.get(&admitted.account).is_some_and(|m| m.contains_key(&admitted.nonce)) {
            return Err(AdmitError::NonceOccupied);
        }
        self.insert(id, Entry { bytes: bytes.to_vec(), tx, from: admitted.account, nonce: admitted.nonce, added: now });
        self.evict(now);
        Ok(id)
    }

    fn lower_nonces<'a>(&'a self, neuron: &Id, below: u64) -> impl Iterator<Item = &'a Entry> + 'a {
        self.by_neuron.get(neuron).into_iter().flat_map(move |m| m.range(..below)).filter_map(|(_, id)| self.entries.get(id))
    }

    fn insert(&mut self, id: Id, entry: Entry) {
        self.bytes += entry.bytes.len();
        self.by_neuron.entry(entry.from).or_default().insert(entry.nonce, id);
        self.entries.insert(id, entry);
    }

    pub fn remove(&mut self, id: &Id) -> Option<Entry> {
        let entry = self.entries.remove(id)?;
        self.bytes -= entry.bytes.len();
        if let Some(m) = self.by_neuron.get_mut(&entry.from) {
            m.remove(&entry.nonce);
            if m.is_empty() {
                self.by_neuron.remove(&entry.from);
            }
        }
        Some(entry)
    }

    /// Capacity by lowest fee per byte then oldest; expiry at 24 h.
    fn evict(&mut self, now: u64) {
        let expired: Vec<Id> = self.entries.iter().filter(|(_, e)| now.saturating_sub(e.added) > MAX_AGE_SECS).map(|(id, _)| *id).collect();
        for id in expired {
            self.remove(&id);
        }
        while self.entries.len() > CAPACITY_TXS || self.bytes > CAPACITY_BYTES {
            let victim = self.entries.iter().min_by_key(|(_, e)| (e.fee_per_byte(), u64::MAX - e.added)).map(|(id, _)| *id);
            match victim {
                Some(id) => {
                    // evicting a nonce leaves its successors unspendable; drop them too
                    let (from, nonce) = { let e = &self.entries[&id]; (e.from, e.nonce) };
                    let later: Vec<Id> = self.by_neuron.get(&from).map(|m| m.range(nonce..).map(|(_, i)| *i).collect()).unwrap_or_default();
                    for id in later {
                        self.remove(&id);
                    }
                }
                None => break,
            }
        }
    }

    /// After a new tip: drop txs now invalid (nonces and balances moved) and
    /// re-admit the transactions of reorged-out blocks.
    pub fn revalidate(&mut self, chain: &dyn ChainView, reorged_out: &[Vec<u8>], now: u64) {
        let mut all: Vec<Entry> = self.entries.values().cloned().collect();
        all.sort_by_key(|e| (e.from, e.nonce));
        self.entries.clear();
        self.by_neuron.clear();
        self.bytes = 0;
        for body in reorged_out {
            for bytes in chain.txs_of(body) {
                if bytes.get(1) != Some(&0) {
                    let _ = self.admit(&bytes, chain, now);
                }
            }
        }
        for entry in all {
            let _ = self.admit(&entry.bytes, chain, entry.added);
        }
    }

    /// Transactions for a block: fee per byte descending, each neuron's
    /// nonces in order, within the budgets of chain.md §1.6.
    pub fn template_txs(&self) -> Vec<Vec<u8>> {
        let mut heads: Vec<(u64, Id)> = self.by_neuron.iter().map(|(n, m)| (*n, *m.values().next().expect("non-empty"))).map(|(n, id)| (self.entries[&id].fee_per_byte(), n)).collect();
        let mut cursor: HashMap<Id, u64> = HashMap::new();
        let mut out = Vec::new();
        let mut bytes = HEADER_BYTES + 4 + 82;
        while out.len() + 1 < MAX_BLOCK_TXS {
            heads.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
            let Some((_, neuron)) = heads.first().copied() else { break };
            let m = &self.by_neuron[&neuron];
            let next_nonce = cursor.get(&neuron).copied().unwrap_or_else(|| *m.keys().next().expect("non-empty"));
            let Some(id) = m.get(&next_nonce) else {
                heads.retain(|(_, n)| *n != neuron);
                continue;
            };
            let entry = &self.entries[id];
            if bytes + entry.bytes.len() > MAX_BLOCK_BYTES {
                heads.retain(|(_, n)| *n != neuron);
                continue;
            }
            bytes += entry.bytes.len();
            out.push(entry.bytes.clone());
            cursor.insert(neuron, next_nonce + 1);
            match m.get(&(next_nonce + 1)) {
                Some(next) => {
                    let fpb = self.entries[next].fee_per_byte();
                    if let Some(h) = heads.iter_mut().find(|(_, n)| *n == neuron) {
                        h.0 = fpb;
                    }
                }
                None => heads.retain(|(_, n)| *n != neuron),
            }
        }
        out
    }

    /// Drop the transactions a block carried (after it was applied).
    pub fn remove_mined(&mut self, chain: &dyn ChainView, body: &[u8]) {
        let chain_id = chain.chain_id();
        for bytes in chain.txs_of(body) {
            self.remove(&txid(&chain_id, &bytes));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::net::kernel::tests::{mine, scratch};
    use crate::chain::tests::Net;

    #[test]
    fn admission_order_and_nonce_window() {
        let net = Net::new(2);
        let s = scratch(&net, "pool-admit");
        let chain = &s.chain;
        let alice = net.neurons[0];
        let mut pool = Mempool::new();
        assert!(matches!(pool.admit(&[1, 9], chain, 0), Err(AdmitError::Decode(_))));
        let tx0 = net.transfer(0, 0, [7; 32], 100, 300);
        let id = pool.admit(&tx0, chain, 0).unwrap();
        assert_eq!(pool.admit(&tx0, chain, 0), Err(AdmitError::Known));
        assert_eq!(pool.next_nonce(chain, &alice), 1);
        let mut forged = tx0.clone();
        forged[50] ^= 1;
        assert_eq!(pool.admit(&forged, chain, 0), Err(AdmitError::Signature));
        let beyond = pool.admit(&net.transfer(0, 16, [7; 32], 1, 300), chain, 0).unwrap_err();
        assert!(matches!(beyond, AdmitError::Nonce { .. }) && beyond.score() == 10, "{beyond:?}");
        let gap = pool.admit(&net.transfer(0, 2, [7; 32], 1, 300), chain, 0).unwrap_err();
        assert!(matches!(gap, AdmitError::NonceGap { expected: 1, got: 2 }) && gap.score() == 0, "{gap:?}");
        assert_eq!(pool.admit(&net.transfer(0, 0, [7; 32], 1, 300), chain, 0), Err(AdmitError::NonceOccupied));
        assert!(matches!(pool.admit(&net.transfer(0, 1, [7; 32], 1, 10), chain, 0), Err(AdmitError::Fee { .. })));
        // balances assume the lower nonces apply first
        let funds = chain.balance(&alice, &chain.native_token());
        let over = pool.admit(&net.transfer(0, 1, [7; 32], funds - 400 + 1, 300), chain, 0).unwrap_err();
        assert!(matches!(over, AdmitError::Balance(_)), "{over:?}");
        assert!(pool.admit(&net.transfer(0, 1, [7; 32], 100, 300), chain, 0).is_ok());
        assert!(pool.admit(&net.transfer(0, 2, [7; 32], 100, 400), chain, 0).is_ok());
        let tpl = pool.template_txs();
        assert_eq!(tpl.len(), 3);
        assert_eq!(tpl[0], tx0, "nonce order beats fee order inside one neuron");
        assert!(pool.remove(&id).is_some());
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn eviction_and_revalidation() {
        let net = Net::new(2);
        let s = scratch(&net, "pool-revalidate");
        let chain = &s.chain;
        let (alice, bob) = (net.neurons[0], net.neurons[1]);
        let mut pool = Mempool::new();
        for n in 0..8 {
            pool.admit(&net.transfer(0, n, [7; 32], 1, 300 + n), chain, 100).unwrap();
            pool.admit(&net.transfer(1, n, [7; 32], 1, 500), chain, 100).unwrap();
        }
        assert_eq!(pool.len(), 16);
        let tpl = pool.template_txs();
        assert_eq!(tpl.len(), 16);
        assert_eq!(Tx::decode(&tpl[0]).unwrap().subject().unwrap(), bob, "higher fee per byte first");
        pool.evict(100 + MAX_AGE_SECS + 1);
        assert!(pool.is_empty());
        pool.admit(&net.transfer(0, 0, [7; 32], 1, 300), chain, 0).unwrap();
        pool.admit(&net.transfer(0, 1, [7; 32], 1, 300), chain, 0).unwrap();
        // mine nonce 0 → nonce 1 stays
        let (_, body) = mine(chain, &bob, &pool.template_txs()[..1]);
        pool.remove_mined(chain, &body);
        pool.revalidate(chain, &[], 0);
        assert_eq!(pool.len(), 1);
        assert_eq!(pool.next_nonce(chain, &alice), 2);
        pool.revalidate(chain, &[body.clone()], 0);
        assert_eq!(pool.len(), 1, "re-admitting a mined tx fails on nonce");
    }
}
