//! The mempool — chain-wire.md §4: admission order, per-neuron nonce
//! ordering, fee floor, capacity with fee-per-byte eviction, 24 h expiry,
//! re-validation after a new tip, and block templates by fee per byte with
//! nonce order preserved.

use std::collections::{BTreeMap, HashMap};

use super::chain_view::ChainView;
use super::codec::{txid, Id, Tx, MAX_BLOCK_BYTES, MAX_BLOCK_TXS, MAX_TX_BYTES, MIN_FEE_PER_BYTE, NONCE_WINDOW, HEADER_BYTES};

pub const CAPACITY_TXS: usize = 8192;
pub const CAPACITY_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_AGE_SECS: u64 = 24 * 3600;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdmitError {
    Decode(String),
    Known,
    Signature,
    Nonce { current: u64, got: u64 },
    NonceOccupied,
    Fee { floor: u64, got: u64 },
    Balance,
    Kind,
}

impl std::fmt::Display for AdmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdmitError::Decode(e) => write!(f, "decode: {e}"),
            AdmitError::Known => write!(f, "already in mempool"),
            AdmitError::Signature => write!(f, "invalid signature"),
            AdmitError::Nonce { current, got } => write!(f, "nonce {got} outside [{current}, {current} + {NONCE_WINDOW})"),
            AdmitError::NonceOccupied => write!(f, "nonce already pending"),
            AdmitError::Fee { floor, got } => write!(f, "fee {got} below floor {floor}"),
            AdmitError::Balance => write!(f, "insufficient spendable balance"),
            AdmitError::Kind => write!(f, "kind not admissible"),
        }
    }
}

impl AdmitError {
    /// chain-wire §5: +10 for a bad signature, a nonce beyond the window or a
    /// non-canonical encoding; +1 for a duplicate; nothing for balance races.
    pub fn score(&self) -> u32 {
        match self {
            AdmitError::Signature | AdmitError::Decode(_) | AdmitError::Kind => 10,
            AdmitError::Nonce { .. } => 10,
            AdmitError::Known => 1,
            AdmitError::NonceOccupied | AdmitError::Fee { .. } | AdmitError::Balance => 0,
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

/// Native debit of a tx: fee plus amount when the token is native.
fn native_debit(tx: &Tx, native: &Id) -> u64 {
    match tx {
        Tx::Transfer { token, amount, fee, .. } | Tx::Cyberlink { token, amount, fee, .. } => {
            if token == native { fee.saturating_add(*amount) } else { *fee }
        }
        Tx::Claim { fee, .. } => *fee,
        Tx::Coinbase { .. } => 0,
    }
}

fn token_debit(tx: &Tx, native: &Id) -> Option<(Id, u64)> {
    match tx {
        Tx::Transfer { token, amount, .. } | Tx::Cyberlink { token, amount, .. } if token != native && *amount > 0 => Some((*token, *amount)),
        _ => None,
    }
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

    /// Admission in the order of chain-wire §4.
    pub fn admit(&mut self, bytes: &[u8], chain: &dyn ChainView, now: u64) -> Result<Id, AdmitError> {
        if bytes.len() > MAX_TX_BYTES {
            return Err(AdmitError::Decode(format!("{} bytes", bytes.len())));
        }
        let tx = Tx::decode(bytes).map_err(|e| AdmitError::Decode(e.to_string()))?;
        if matches!(tx, Tx::Coinbase { .. }) {
            return Err(AdmitError::Kind);
        }
        let chain_id = chain.chain_id();
        let id = txid(&chain_id, bytes);
        if self.entries.contains_key(&id) {
            return Err(AdmitError::Known);
        }
        if !tx.verify(&chain_id) {
            return Err(AdmitError::Signature);
        }
        let from = tx.subject().ok_or(AdmitError::Signature)?;
        if let Tx::Claim { account_id, .. } = &tx {
            let pubkey: [u8; 33] = tx.sig()[5..38].try_into().expect("33");
            if mudra::cosmos::account_id(&pubkey) != *account_id {
                return Err(AdmitError::Signature);
            }
        }
        let nonce = tx.nonce().ok_or(AdmitError::Kind)?;
        let current = chain.nonce(&from);
        if nonce < current || nonce >= current.saturating_add(NONCE_WINDOW) {
            return Err(AdmitError::Nonce { current, got: nonce });
        }
        if self.by_neuron.get(&from).is_some_and(|m| m.contains_key(&nonce)) {
            return Err(AdmitError::NonceOccupied);
        }
        let floor = bytes.len() as u64 * MIN_FEE_PER_BYTE;
        if tx.fee() < floor {
            return Err(AdmitError::Fee { floor, got: tx.fee() });
        }
        let native = chain.native_token();
        let payer = match &tx {
            Tx::Claim { account_id, .. } => super::devchain::legacy_owner(&chain.hrp(), account_id),
            _ => from,
        };
        let pending_native: u64 = self.lower_nonces(&from, nonce).map(|e| native_debit(&e.tx, &native)).sum();
        if chain.spendable(&payer, &native) < pending_native.saturating_add(native_debit(&tx, &native)) {
            return Err(AdmitError::Balance);
        }
        if let Some((token, amount)) = token_debit(&tx, &native) {
            let pending: u64 = self.lower_nonces(&from, nonce).filter_map(|e| token_debit(&e.tx, &native)).filter(|(t, _)| *t == token).map(|(_, a)| a).sum();
            if chain.spendable(&from, &token) < pending.saturating_add(amount) {
                return Err(AdmitError::Balance);
            }
        }
        self.insert(id, Entry { bytes: bytes.to_vec(), tx, from, nonce, added: now });
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
    use crate::chain::net::codec::{sign_tx, Tx};
    use crate::chain::net::devchain::{dev_genesis, DevChain};

    fn key(seed: u8) -> (mudra::SigningKey, Id) {
        let key = mudra::SigningKey::from_bytes((&[seed; 32]).into()).unwrap();
        let id = mudra::claim::neuron_of(&mudra::cosmos::compressed(key.verifying_key()));
        (key, id)
    }

    fn transfer(chain: &DevChain, key: &mudra::SigningKey, from: Id, nonce: u64, amount: u64, fee: u64) -> Vec<u8> {
        let tx = Tx::Transfer { from, nonce, token: chain.native_token(), amount, to: [7; 32], fee, sig: vec![] };
        sign_tx(&chain.chain_id(), &tx, key).unwrap().encode()
    }

    #[test]
    fn admission_order_and_nonce_window() {
        let (k, alice) = key(1);
        let chain = DevChain::open(&dev_genesis("dev", 1, &[(alice, 10_000)]).encode(), None).unwrap();
        let mut pool = Mempool::new();
        assert!(matches!(pool.admit(&[1, 9], &chain, 0), Err(AdmitError::Decode(_))));
        let tx0 = transfer(&chain, &k, alice, 0, 100, 224);
        let id = pool.admit(&tx0, &chain, 0).unwrap();
        assert_eq!(pool.admit(&tx0, &chain, 0), Err(AdmitError::Known));
        assert_eq!(pool.next_nonce(&chain, &alice), 1);
        let mut forged = tx0.clone();
        forged[50] ^= 1;
        assert_eq!(pool.admit(&forged, &chain, 0), Err(AdmitError::Signature));
        assert!(matches!(pool.admit(&transfer(&chain, &k, alice, 16, 1, 224), &chain, 0), Err(AdmitError::Nonce { .. })));
        assert_eq!(pool.admit(&transfer(&chain, &k, alice, 0, 1, 224), &chain, 0), Err(AdmitError::NonceOccupied));
        assert!(matches!(pool.admit(&transfer(&chain, &k, alice, 1, 1, 10), &chain, 0), Err(AdmitError::Fee { .. })));
        // balance assumes lower nonces apply first: 10_000 − 324 leaves 9_676
        assert_eq!(pool.admit(&transfer(&chain, &k, alice, 2, 9_500, 224), &chain, 0), Err(AdmitError::Balance));
        assert!(pool.admit(&transfer(&chain, &k, alice, 2, 9_000, 224), &chain, 0).is_ok());
        assert!(pool.admit(&transfer(&chain, &k, alice, 1, 100, 300), &chain, 0).is_ok());
        let tpl = pool.template_txs();
        assert_eq!(tpl.len(), 3);
        assert_eq!(tpl[0], tx0, "nonce order beats fee order inside one neuron");
        assert!(pool.remove(&id).is_some());
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn eviction_and_revalidation() {
        let (k, alice) = key(1);
        let (k2, bob) = key(2);
        let chain = DevChain::open(&dev_genesis("dev", 1, &[(alice, 1_000_000), (bob, 1_000_000)]).encode(), None).unwrap();
        let mut pool = Mempool::new();
        for n in 0..8 {
            pool.admit(&transfer(&chain, &k, alice, n, 1, 224 + n), &chain, 100).unwrap();
            pool.admit(&transfer(&chain, &k2, bob, n, 1, 500), &chain, 100).unwrap();
        }
        assert_eq!(pool.len(), 16);
        let tpl = pool.template_txs();
        assert_eq!(tpl.len(), 16);
        assert_eq!(Tx::decode(&tpl[0]).unwrap().subject().unwrap(), bob, "higher fee per byte first");
        pool.evict(100 + MAX_AGE_SECS + 1);
        assert!(pool.is_empty());
        pool.admit(&transfer(&chain, &k, alice, 0, 1, 224), &chain, 0).unwrap();
        pool.admit(&transfer(&chain, &k, alice, 1, 1, 224), &chain, 0).unwrap();
        // mine nonce 0 → nonce 1 stays, a stale copy of nonce 0 would be dropped
        let (h, b) = chain.template(&[9; 32], &pool.template_txs()[..1], 100).unwrap();
        let h = crate::chain::net::devchain::mine(h, 1 << 20).unwrap();
        chain.accept_header(&h, 1000).unwrap();
        chain.accept_block(&h, &b).unwrap();
        pool.remove_mined(&chain, &b);
        pool.revalidate(&chain, &[], 0);
        assert_eq!(pool.len(), 1);
        assert_eq!(pool.next_nonce(&chain, &alice), 2);
        pool.revalidate(&chain, &[b.clone()], 0);
        assert_eq!(pool.len(), 1, "re-admitting a mined tx fails on nonce");
    }
}
