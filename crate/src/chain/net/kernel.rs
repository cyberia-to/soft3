//! The kernel behind the wire's port: [`KernelChain`] implements
//! [`ChainView`] over the consensus kernel's [`ChainApi`] (`chain::engine::Chain`).
//!
//! What the adapter adds to the kernel, and nothing more:
//! - interior mutability (the wire is many tasks; the kernel is `&mut self`);
//! - the most-work *header* (bodies may lag during IBD) and the bodies it
//!   still lacks, read off the kernel's store;
//! - headers ahead of the clock held for retry (`ChainError::FutureTime`
//!   is never the sender's fault);
//! - the tip-moved events the mempool and the address book follow, with the
//!   bodies a reorganization took off the main chain;
//! - a txid index of the main chain for `GET /tx/{txid}`.
//!
//! Headers and integers cross the boundary as their wire bytes (the 168-byte
//! header, the 32-byte little-endian u256), so the two codecs can never
//! silently disagree: a test pins header ids and txids on both sides.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use tokio::sync::broadcast;

use crate::chain::api::{Block, BlockVerdict as KBlock, ChainApi, ChainError, HeaderVerdict as KHeaderVerdict, Template};
use crate::chain::apply::TxError;
use crate::chain::engine::{Chain, OpenOptions};
use crate::chain::header::{self as kheader, Header as KHeader, HeaderError as KHeaderError};
use crate::chain::params::ORPHAN_POOL_HEADERS;
use crate::chain::select::Checkpoint;
use crate::chain::state::StateBackend;
use crate::chain::store::Status;
use crate::chain::tx as ktx;
use crate::chain::u256::U256 as KU256;

use super::chain_view::{Admitted, BlockError, BlockVerdict, ChainEvent, ChainView, HeaderError, HeaderVerdict, Tip, TxRefusal};
use super::codec::{Header, Id, U256};

fn to_kernel(h: &Header) -> KHeader {
    KHeader::decode_raw(&h.encode()).expect("a header is 168 bytes")
}

fn from_kernel(h: &KHeader) -> Header {
    Header::decode(&h.encode()).expect("a header is 168 bytes")
}

fn wide(w: KU256) -> U256 {
    U256::from_le_bytes(&w.to_le_bytes())
}

/// The kernel's header errors in the wire's scoring terms (chain-wire §5).
fn header_error(e: ChainError) -> HeaderError {
    match e {
        ChainError::Header(KHeaderError::WrongChain) => HeaderError::WrongChain,
        ChainError::Header(KHeaderError::RuleSet(_)) | ChainError::RuleSetUnknown(_) => HeaderError::UnknownRuleSet,
        ChainError::Header(KHeaderError::Bits(_)) => HeaderError::BadBits,
        ChainError::Header(e @ KHeaderError::Length(_)) => HeaderError::Syntax(e.to_string()),
        ChainError::Height { .. } => HeaderError::BadParent,
        ChainError::InvalidParent(_) | ChainError::Rejected(_) => HeaderError::InvalidParent,
        ChainError::Time { .. } => HeaderError::BadTime,
        ChainError::Bits { .. } => HeaderError::BadTarget,
        ChainError::Pow => HeaderError::BadPow,
        ChainError::Genesis => HeaderError::Syntax("a second block 0".into()),
        e @ (ChainError::DeepFork { .. } | ChainError::Checkpoint { .. } | ChainError::CheckpointRejected(_)) => HeaderError::Refused(e.to_string()),
        ChainError::Store(s) => HeaderError::Store(s),
        // the callers route these before mapping
        e @ (ChainError::UnknownParent(_) | ChainError::FutureTime { .. } | ChainError::Block(_)) => HeaderError::Syntax(e.to_string()),
    }
}

fn tx_refusal(e: TxError) -> TxRefusal {
    match e {
        TxError::Decode(e) => TxRefusal::Decode(e.to_string()),
        e @ TxError::TooLarge(_) => TxRefusal::Decode(e.to_string()),
        TxError::CoinbaseInBody => TxRefusal::Kind,
        TxError::Signature | TxError::ClaimKey => TxRefusal::Signature,
        TxError::Nonce { expected, actual } => TxRefusal::Nonce { current: expected, got: actual },
        TxError::Fee { min, actual } => TxRefusal::Fee { floor: min, got: actual },
        e @ (TxError::Insufficient { .. } | TxError::Immature { .. }) => TxRefusal::Balance(e.to_string()),
        e @ (TxError::SelfLink
        | TxError::StakeToken
        | TxError::Valence { .. }
        | TxError::NoCard
        | TxError::Locked { .. }
        | TxError::UnlockAmount { .. }
        | TxError::Overflow) => TxRefusal::Rule(e.to_string()),
    }
}

struct Inner {
    chain: Chain,
    /// the most-work non-invalid header, bodies or not; `None` = rescan
    best: Option<(u64, Id, KU256)>,
    /// headers ahead of the clock, re-evaluated by [`ChainView::retry_held`]
    held: Vec<KHeader>,
    /// txid → (height, block, index) on the main chain
    txindex: HashMap<Id, (u64, Id, u32)>,
}

impl Inner {
    fn best(&mut self) -> (u64, Id, KU256) {
        if let Some(b) = self.best {
            return b;
        }
        let tip = self.chain.tip();
        let mut best = (tip.height, tip.id, tip.chainwork);
        for (id, e) in self.chain.store().iter() {
            if e.status != Status::Invalid && e.chainwork > best.2 {
                best = (e.header.height, *id, e.chainwork);
            }
        }
        self.best = Some(best);
        best
    }

    fn consider(&mut self, id: &Id) {
        let Some(e) = self.chain.store().get(id) else { return };
        let candidate = (e.header.height, *id, e.chainwork);
        if let Some(best) = self.best {
            if candidate.2 > best.2 {
                self.best = Some(candidate);
            }
        }
    }

    fn index(&mut self, height: u64) {
        let Some(id) = self.chain.store().main_at(height) else { return };
        let Some(body) = self.chain.store().body(&id) else { return };
        let chain_id = self.chain.rules().chain;
        for (i, raw) in ktx::split_body(&body).unwrap_or_default().iter().enumerate() {
            self.txindex.insert(crate::chain::sigs::txid(&chain_id, raw), (height, id, i as u32));
        }
    }

    fn unindex(&mut self, body: &[u8]) {
        let chain_id = self.chain.rules().chain;
        for raw in ktx::split_body(body).unwrap_or_default() {
            self.txindex.remove(&crate::chain::sigs::txid(&chain_id, &raw));
        }
    }

    fn hold(&mut self, header: KHeader) {
        if !self.held.iter().any(|h| h.id() == header.id()) {
            if self.held.len() >= ORPHAN_POOL_HEADERS {
                self.held.remove(0);
            }
            self.held.push(header);
        }
    }

    /// After a write: if the tip moved, the bodies that left the main chain
    /// (oldest first), with the txid index kept in step.
    fn moved(&mut self, before: &crate::chain::api::Tip) -> Option<ChainEvent> {
        let after = self.chain.tip();
        // finalized blocks are the shadow indexers' (chain.md §7.3); the wire has no use for them
        let _ = self.chain.take_events();
        if after.id == before.id {
            return None;
        }
        let store = self.chain.store();
        let mut fork = before.height.min(after.height);
        while fork > 0 && store.ancestor_at(&before.id, fork) != store.main_at(fork) {
            fork -= 1;
        }
        let reorged_out: Vec<Vec<u8>> = (fork + 1..=before.height)
            .filter_map(|h| store.ancestor_at(&before.id, h))
            .filter_map(|id| store.body(&id))
            .collect();
        for body in &reorged_out {
            self.unindex(body);
        }
        for h in fork + 1..=after.height {
            self.index(h);
        }
        Some(ChainEvent::NewTip { height: after.height, id: after.id, reorged_out })
    }
}

/// The consensus kernel as the wire sees it.
pub struct KernelChain {
    inner: Mutex<Inner>,
    events: broadcast::Sender<ChainEvent>,
    chain_id: Id,
    genesis_id: Id,
    name: String,
    hrp: String,
    native: Id,
    subsidy: u64,
    genesis_time: u64,
    checkpoint_key: Option<[u8; 33]>,
}

impl KernelChain {
    /// The economy's read surface (chain-economy §11), as JSON.
    pub fn economy(&self, f: impl FnOnce(&Chain) -> Option<serde_json::Value>) -> Option<serde_json::Value> {
        f(&self.lock().chain)
    }

    /// Open (or create) the chain under `home` (`home/blocks` is the store);
    /// a restart replays the main chain on the genesis state (§3.6).
    pub fn open(genesis_bytes: &[u8], home: &Path) -> Result<KernelChain, String> {
        let chain = Chain::open(home, genesis_bytes, true).map_err(|e| e.to_string())?;
        Ok(KernelChain::over(chain))
    }

    /// As [`KernelChain::open`] with explicit [`OpenOptions`] (epoch clock,
    /// focus engine, inline settlement): the simulator's and the tests' entry.
    pub fn open_with(genesis_bytes: &[u8], home: &Path, opts: OpenOptions) -> Result<KernelChain, String> {
        let chain = Chain::open_with(home, genesis_bytes, opts).map_err(|e| e.to_string())?;
        Ok(KernelChain::over(chain))
    }

    fn over(chain: Chain) -> KernelChain {
        let params = chain.genesis().params().clone();
        let key = params.checkpoint_key;
        let mut inner = Inner { best: None, held: Vec::new(), txindex: HashMap::new(), chain };
        for h in 1..=inner.chain.tip().height {
            inner.index(h);
        }
        KernelChain {
            chain_id: inner.chain.chain_id(),
            genesis_id: inner.chain.store().main_at(0).expect("block 0 is stored"),
            native: inner.chain.genesis().native_token(),
            subsidy: inner.chain.genesis().subsidy(),
            genesis_time: params.time,
            checkpoint_key: (key != [0; 33]).then_some(key),
            name: params.name,
            hrp: params.hrp,
            inner: Mutex::new(inner),
            events: broadcast::channel(1024).0,
        }
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().expect("chain")
    }

    fn emit(&self, event: Option<ChainEvent>) {
        if let Some(event) = event {
            let _ = self.events.send(event);
        }
    }

    /// The `next-bits` of `/status`: ASERT on the tip (§1.5).
    pub fn next_bits(&self) -> u32 {
        self.lock().chain.status().next_bits
    }

    /// The `GET /tip` answer of chain-wire §7, rendered by the kernel.
    pub fn tip_json(&self) -> String {
        self.lock().chain.tip().json()
    }

    pub fn finalized_height(&self) -> u64 {
        self.lock().chain.finalized_height()
    }
}

impl ChainView for KernelChain {
    fn chain_id(&self) -> Id {
        self.chain_id
    }
    fn genesis_id(&self) -> Id {
        self.genesis_id
    }
    fn chain_name(&self) -> String {
        self.name.clone()
    }
    fn hrp(&self) -> String {
        self.hrp.clone()
    }
    fn native_token(&self) -> Id {
        self.native
    }
    fn subsidy(&self) -> u64 {
        self.subsidy
    }
    fn genesis_time(&self) -> u64 {
        self.genesis_time
    }
    fn checkpoint_key(&self) -> Option<[u8; 33]> {
        self.checkpoint_key
    }

    fn tip(&self) -> Tip {
        let t = self.lock().chain.tip();
        Tip { height: t.height, id: t.id, chainwork: wide(t.chainwork) }
    }
    fn best_header(&self) -> Tip {
        let (height, id, work) = self.lock().best();
        Tip { height, id, chainwork: wide(work) }
    }
    fn header(&self, id: &Id) -> Option<Header> {
        self.lock().chain.header(id).map(|h| from_kernel(&h))
    }
    fn header_at(&self, height: u64) -> Option<Header> {
        self.lock().chain.header_at(height).map(|h| from_kernel(&h))
    }
    fn chainwork_of(&self, id: &Id) -> Option<U256> {
        self.lock().chain.chainwork(id).map(wide)
    }
    fn has_body(&self, id: &Id) -> bool {
        let inner = self.lock();
        inner.chain.store().has_body(id) || inner.chain.store().main_at(0) == Some(*id)
    }
    fn block(&self, id: &Id) -> Option<(Header, Vec<u8>)> {
        self.lock().chain.block(id).map(|b| (from_kernel(&b.header), b.body))
    }
    fn block_at(&self, height: u64) -> Option<(Header, Vec<u8>)> {
        let inner = self.lock();
        let id = inner.chain.store().main_at(height)?;
        inner.chain.block(&id).map(|b| (from_kernel(&b.header), b.body))
    }
    fn height_of(&self, id: &Id) -> Option<u64> {
        let inner = self.lock();
        let h = inner.chain.header(id)?.height;
        (inner.chain.store().main_at(h) == Some(*id)).then_some(h)
    }

    fn locator(&self) -> Vec<Id> {
        self.lock().chain.locator()
    }
    fn headers_from_locator(&self, locator: &[Id], stop: &Id, max: usize) -> Vec<Header> {
        let inner = self.lock();
        let store = inner.chain.store();
        let Some(start) = locator.iter().find(|id| store.on_main(id)).and_then(|id| store.get(id)).map(|e| e.header.height + 1) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for h in inner.chain.headers(start, max) {
            let done = h.id() == *stop;
            out.push(from_kernel(&h));
            if done {
                break;
            }
        }
        out
    }

    fn accept_header(&self, header: &Header, now: u64) -> Result<HeaderVerdict, HeaderError> {
        let kh = to_kernel(header);
        let mut inner = self.lock();
        match inner.chain.accept_header(&kh, now) {
            Ok(KHeaderVerdict::Accepted) => {
                inner.consider(&kh.id());
                Ok(HeaderVerdict::Accepted)
            }
            Ok(KHeaderVerdict::Known) => Ok(HeaderVerdict::Known),
            Err(ChainError::UnknownParent(_)) => Ok(HeaderVerdict::Orphan),
            Err(ChainError::FutureTime { .. }) => {
                inner.hold(kh);
                Ok(HeaderVerdict::Held)
            }
            Err(e) => Err(header_error(e)),
        }
    }

    fn accept_block(&self, header: &Header, body: &[u8], now: u64) -> Result<BlockVerdict, BlockError> {
        let block = Block { header: to_kernel(header), body: body.to_vec() };
        let id = block.id();
        let mut inner = self.lock();
        let before = inner.chain.tip();
        let result = inner.chain.accept_block(&block, now);
        let event = inner.moved(&before);
        let verdict = match result {
            Ok(v) => {
                inner.consider(&id);
                Ok(match v {
                    KBlock::Extended | KBlock::Reorganized { .. } => BlockVerdict::NewTip,
                    KBlock::Stored => BlockVerdict::Stored,
                    KBlock::Known => BlockVerdict::Known,
                    KBlock::NeedBodies(ids) => BlockVerdict::NeedBodies(ids),
                })
            }
            Err(ChainError::UnknownParent(_)) => Err(BlockError::UnknownParent),
            Err(ChainError::FutureTime { .. }) => {
                inner.hold(block.header);
                Err(BlockError::Held)
            }
            Err(e @ (ChainError::DeepFork { .. } | ChainError::Checkpoint { .. } | ChainError::CheckpointRejected(_))) => Err(BlockError::Refused(e.to_string())),
            Err(ChainError::Block(e)) => {
                // statuses changed (the block and its descendants are invalid)
                inner.best = None;
                Err(BlockError::Invalid(e.to_string()))
            }
            Err(ChainError::Store(s)) => Err(BlockError::Store(s)),
            Err(e) => Err(BlockError::Header(header_error(e))),
        };
        drop(inner);
        self.emit(event);
        verdict
    }

    fn retry_held(&self, now: u64) -> Vec<Header> {
        let mut inner = self.lock();
        let held = std::mem::take(&mut inner.held);
        let mut accepted = Vec::new();
        for kh in held {
            match inner.chain.accept_header(&kh, now) {
                Ok(KHeaderVerdict::Accepted) => {
                    inner.consider(&kh.id());
                    accepted.push(from_kernel(&kh));
                }
                Err(ChainError::FutureTime { .. }) => inner.held.push(kh),
                _ => {}
            }
        }
        accepted
    }

    fn accept_checkpoint(&self, height: u64, block_id: &Id, sig: &[u8]) -> Result<bool, String> {
        let cp = Checkpoint { height, block_id: *block_id, sig: sig.to_vec() };
        let mut inner = self.lock();
        let before = inner.chain.tip();
        let result = inner.chain.accept_checkpoint(&cp).map_err(|e| e.to_string());
        inner.best = None;
        let event = inner.moved(&before);
        drop(inner);
        self.emit(event);
        result
    }

    fn missing_bodies(&self, max: usize) -> Vec<(u64, Id)> {
        let mut inner = self.lock();
        let (_, best, work) = inner.best();
        if work <= inner.chain.tip().chainwork {
            return Vec::new();
        }
        let store = inner.chain.store();
        let mut out = Vec::new();
        let mut id = best;
        while let Some(e) = store.get(&id) {
            if store.on_main(&id) {
                break;
            }
            if !store.has_body(&id) {
                out.push((e.header.height, id));
            }
            id = e.header.prev;
        }
        out.reverse();
        out.truncate(max);
        out
    }

    fn nonce(&self, neuron: &Id) -> u64 {
        self.lock().chain.nonce(neuron)
    }
    fn validate_tx(&self, tx: &[u8], pending: &[Vec<u8>]) -> Result<Admitted, TxRefusal> {
        let pending: Vec<ktx::Tx> = pending.iter().filter_map(|raw| ktx::Tx::decode(raw).ok()).collect();
        let info = self.lock().chain.validate_tx(tx, &pending).map_err(tx_refusal)?;
        Ok(Admitted { txid: info.txid, account: info.account, nonce: info.nonce, fee: info.fee })
    }
    fn balance(&self, owner: &Id, token: &Id) -> u64 {
        self.lock().chain.balance(owner, token)
    }
    fn spendable(&self, owner: &Id, token: &Id) -> u64 {
        let mut inner = self.lock();
        let balance = inner.chain.balance(owner, token);
        if *token != self.native {
            return balance;
        }
        let next = inner.chain.tip().height + 1;
        balance.saturating_sub(inner.chain.locked_at(next).of(owner))
    }
    fn balances(&self, owner: &Id) -> Vec<(Id, u64)> {
        let inner = self.lock();
        let state = inner.chain.state();
        state.tokens().into_iter().map(|t| (t, state.balance(owner, &t))).filter(|(_, a)| *a > 0).collect()
    }
    fn tx_location(&self, txid: &Id) -> Option<(u64, Id, u32)> {
        self.lock().txindex.get(txid).copied()
    }
    fn supply(&self, token: &Id) -> u64 {
        self.lock().chain.supply(token)
    }
    fn conserved(&self) -> bool {
        self.lock().chain.status().conserved
    }
    fn counts(&self) -> (u64, u64, u64) {
        let inner = self.lock();
        let s = inner.chain.state();
        (s.neuron_count(), s.particle_count(), s.axon_count())
    }
    fn state_root(&self) -> Id {
        self.lock().chain.tip().header.state_root
    }

    fn template(&self, to: &Id, txs: &[Vec<u8>], now: u64) -> Result<(Header, Vec<u8>), String> {
        let t = self.lock().chain.header_template(*to, now, txs).map_err(|e| e.to_string())?;
        Ok((from_kernel(&t.header), t.body))
    }
    fn template_json(&self, header: &Header, body: &[u8], expires: u64) -> String {
        let target = kheader::decode_bits(header.bits).unwrap_or(KU256::ZERO);
        Template { header: to_kernel(header), target, body: body.to_vec(), txids: Vec::new(), fees: 0 }.json(&self.name, expires)
    }
    fn events(&self) -> broadcast::Receiver<ChainEvent> {
        self.events.subscribe()
    }
    fn txs_of(&self, body: &[u8]) -> Vec<Vec<u8>> {
        ktx::split_body(body).unwrap_or_default()
    }
}

#[cfg(test)]
pub(crate) mod tests;
