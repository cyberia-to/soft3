//! The chain engine: [`Chain`] implements [`ChainApi`] over the store, the
//! bbg-backed state, the checkpoints and the validation order of §5.5.
//! Headers are validated in steps 1–5 and stored; bodies run steps 6–8 on
//! the parent state; fork choice (§5.1–§5.3) moves the tip with the undo
//! journals. Mempool and template entry points live in [`super::engine_tx`].

use std::collections::{HashSet, VecDeque};
use std::path::Path;

use super::api::{Block, BlockVerdict, ChainApi, ChainError, Event, HeaderVerdict, Status, Tip};
use super::apply::{self, Decoded, Locked, Rules};
use super::bbg_state::BbgBacked;
use super::daa::{median_time_past, next_bits};
use super::genesis::Genesis;
use super::header::{BlockId, Header};
use super::params::{COINBASE_MATURITY, FINALITY_DEPTH, MAX_FUTURE_DRIFT, MTP_WINDOW, RULE_SET, rule_set_at};
use super::pow::{chainwork, valid_pow};
use super::select::{self, Checkpoint, Checkpoints};
use super::state::{Id, StateBackend};
use super::store::{self, Store};
use super::tx::Body;
use super::u256::U256;

const REJECTED_CAPACITY: usize = 4_096;

pub struct Chain {
    genesis: Genesis,
    rules: Rules,
    store: Store,
    state: BbgBacked,
    checkpoints: Checkpoints,
    /// main-chain coinbases `(height, to, amount)` still inside maturity
    coinbases: VecDeque<(u64, Id, u64)>,
    rejected: HashSet<BlockId>,
    rejected_order: VecDeque<BlockId>,
    /// the most work any non-invalid header carries; when the tip has it,
    /// there is nothing to consider
    heaviest: U256,
    finalized: u64,
    events: Vec<Event>,
}

impl Chain {
    /// Open or create the chain under `home` (`home/blocks` is the store).
    /// A store whose block 0 differs from the genesis file refuses to open.
    pub fn open(home: &Path, genesis_bytes: &[u8], checkpoints_enabled: bool) -> Result<Chain, ChainError> {
        let genesis = Genesis::decode(genesis_bytes).map_err(|e| ChainError::Store(e.to_string()))?;
        let rules = Rules {
            chain: genesis.chain_id,
            native: genesis.native_token(),
            hrp: genesis.params().hrp.clone(),
            subsidy: genesis.subsidy(),
            s0: genesis.s0(),
        };
        std::fs::create_dir_all(home).map_err(|e| ChainError::Store(e.to_string()))?;
        let mut store = Store::open(&home.join("blocks")).map_err(ChainError::Store)?;
        let mut state = genesis.state();
        let block0 = genesis.block0(state.root());
        let checkpoints = Checkpoints::new(genesis.params().checkpoint_key, checkpoints_enabled);
        if store.is_empty() {
            store.put_header(block0, U256::ZERO, store::Status::Valid).map_err(ChainError::Store)?;
            store.push_main(block0.id()).map_err(ChainError::Store)?;
            store.persist().map_err(ChainError::Store)?;
        } else if store.main_at(0) != Some(block0.id()) {
            return Err(ChainError::Genesis);
        }
        let mut chain = Chain {
            genesis,
            rules,
            store,
            state,
            checkpoints,
            coinbases: VecDeque::new(),
            rejected: HashSet::new(),
            rejected_order: VecDeque::new(),
            heaviest: U256::ZERO,
            finalized: 0,
            events: Vec::new(),
        };
        chain.replay()?;
        chain.heaviest = chain.store.iter().filter(|(_, e)| e.status != store::Status::Invalid).map(|(_, e)| e.chainwork).max().unwrap_or(U256::ZERO);
        chain.finalized = chain.store.tip_height().saturating_sub(FINALITY_DEPTH);
        Ok(chain)
    }

    /// Recovery: replay every main-chain body on the genesis state and
    /// verify the stored roots before readiness (§3.6).
    fn replay(&mut self) -> Result<(), ChainError> {
        let tip = self.store.tip_height();
        for h in 1..=tip {
            let id = self.store.main_at(h).ok_or_else(|| ChainError::Store(format!("main chain gap at {h}")))?;
            let header = self.store.get(&id).map(|e| e.header).ok_or_else(|| ChainError::Store("missing header".into()))?;
            let body = self.store.body(&id).ok_or_else(|| ChainError::Store(format!("missing body at {h}")))?;
            let txs = apply::decode_body(&self.rules, &header, &body).map_err(ChainError::Block)?;
            let locked = Locked::at(self.coinbases.make_contiguous(), h);
            apply::apply_block(&self.rules, &header, &txs, &mut self.state, &locked).map_err(ChainError::Block)?;
            self.push_coinbase(h, &txs);
        }
        Ok(())
    }

    pub fn rules(&self) -> &Rules {
        &self.rules
    }

    pub fn store(&self) -> &Store {
        &self.store
    }

    pub fn state(&self) -> &BbgBacked {
        &self.state
    }

    fn entry(&self, id: &BlockId) -> Option<&store::Entry> {
        self.store.get(id)
    }

    fn tip_id(&self) -> BlockId {
        self.store.tip().expect("genesis is always present").1
    }

    fn push_coinbase(&mut self, height: u64, txs: &[Decoded]) {
        if let Some(Body::Coinbase { to, amount, .. }) = txs.first().map(|d| &d.tx.body) {
            self.coinbases.push_back((height, *to, *amount));
        }
        while self.coinbases.front().is_some_and(|(h, _, _)| h.saturating_add(COINBASE_MATURITY) <= height.saturating_add(1)) {
            self.coinbases.pop_front();
        }
    }

    pub(super) fn locked_at(&mut self, height: u64) -> Locked {
        Locked::at(self.coinbases.make_contiguous(), height)
    }

    fn remember_rejected(&mut self, id: BlockId) {
        if self.rejected.insert(id) {
            self.rejected_order.push_back(id);
            if self.rejected_order.len() > REJECTED_CAPACITY {
                if let Some(old) = self.rejected_order.pop_front() {
                    self.rejected.remove(&old);
                }
            }
        }
    }

    fn invalidate(&mut self, id: BlockId) {
        let _ = self.store.set_status(&id, store::Status::Invalid);
        self.remember_rejected(id);
    }

    /// Non-invalid headers heavier than the tip, heaviest first, ties in
    /// first-seen order (§5.1).
    fn candidates(&self) -> Vec<BlockId> {
        let tip_work = self.entry(&self.tip_id()).map(|e| e.chainwork).unwrap_or(U256::ZERO);
        let mut v: Vec<(U256, u64, BlockId)> = self
            .store
            .iter()
            .filter(|(_, e)| e.status != store::Status::Invalid && e.chainwork > tip_work)
            .map(|(id, e)| (e.chainwork, e.seq, *id))
            .collect();
        v.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        v.into_iter().map(|(_, _, id)| id).collect()
    }

    /// Median time past of the parent and its ancestors (§1.4).
    fn mtp(&self, parent: &BlockId) -> u64 {
        let mut times = Vec::with_capacity(MTP_WINDOW);
        let mut cur = *parent;
        while times.len() < MTP_WINDOW {
            let Some(e) = self.entry(&cur) else { break };
            times.push(e.header.time);
            if e.header.height == 0 {
                break;
            }
            cur = e.header.prev;
        }
        median_time_past(&times)
    }

    /// Steps 1–5 without storing.
    fn check_header(&self, header: &Header, now: u64) -> Result<U256, ChainError> {
        header.check_syntax(&self.rules.chain).map_err(ChainError::Header)?;
        match rule_set_at(header.height) {
            Some(rs) if rs == RULE_SET => {}
            Some(_) | None => return Err(ChainError::RuleSetUnknown(header.height)),
        }
        if header.height == 0 {
            return Err(ChainError::Genesis);
        }
        let parent = self.entry(&header.prev).ok_or(ChainError::UnknownParent(header.prev))?;
        if parent.status == store::Status::Invalid {
            return Err(ChainError::InvalidParent(header.prev));
        }
        if header.height != parent.header.height + 1 {
            return Err(ChainError::Height { expected: parent.header.height + 1, actual: header.height });
        }
        let mtp = self.mtp(&header.prev);
        if header.time <= mtp {
            return Err(ChainError::Time { mtp, actual: header.time });
        }
        if header.time > now.saturating_add(MAX_FUTURE_DRIFT) {
            return Err(ChainError::FutureTime { time: header.time, now });
        }
        let expected = next_bits(self.genesis.anchor(), &parent.header);
        if header.bits != expected {
            return Err(ChainError::Bits { expected, actual: header.bits });
        }
        if !valid_pow(header) {
            return Err(ChainError::Pow);
        }
        Ok(chainwork(parent.chainwork, header))
    }

    /// Apply a block whose parent is the tip (steps 7–8), then extend.
    fn extend(&mut self, header: &Header, txs: &[Decoded]) -> Result<(), ChainError> {
        let id = header.id();
        let locked = self.locked_at(header.height);
        let undo = match apply::apply_block(&self.rules, header, txs, &mut self.state, &locked) {
            Ok(undo) => undo,
            Err(e) => {
                self.invalidate(id);
                return Err(ChainError::Block(e));
            }
        };
        self.store.put_undo(&id, &undo).map_err(ChainError::Store)?;
        self.store.set_status(&id, store::Status::Valid).map_err(ChainError::Store)?;
        self.store.push_main(id).map_err(ChainError::Store)?;
        self.push_coinbase(header.height, txs);
        Ok(())
    }

    /// Undo the tip block (its journal is in the store).
    fn retract(&mut self) -> Result<BlockId, ChainError> {
        let (height, id) = self.store.tip().ok_or(ChainError::Store("empty chain".into()))?;
        if height == 0 {
            return Err(ChainError::Store("cannot retract genesis".into()));
        }
        let undo = self.store.undo(&id).ok_or(ChainError::Store("missing undo".into()))?;
        self.state.revert(&undo);
        self.store.pop_main().map_err(ChainError::Store)?;
        self.rebuild_coinbases();
        Ok(id)
    }

    fn rebuild_coinbases(&mut self) {
        self.coinbases.clear();
        let tip = self.store.tip_height();
        let from = tip.saturating_sub(COINBASE_MATURITY).max(1);
        for h in from..=tip {
            let Some(id) = self.store.main_at(h) else { continue };
            let Some(body) = self.store.body(&id) else { continue };
            if let Ok(raws) = super::tx::split_body(&body) {
                if let Some(Ok(tx)) = raws.first().map(|r| super::tx::Tx::decode(r)) {
                    if let Body::Coinbase { to, amount, .. } = tx.body {
                        self.coinbases.push_back((h, to, amount));
                    }
                }
            }
        }
    }

    fn decoded(&self, id: &BlockId) -> Result<(Header, Vec<Decoded>), ChainError> {
        let header = self.entry(id).map(|e| e.header).ok_or(ChainError::Store("missing header".into()))?;
        let body = self.store.body(id).ok_or(ChainError::Store("missing body".into()))?;
        let txs = apply::decode_body(&self.rules, &header, &body).map_err(ChainError::Block)?;
        Ok((header, txs))
    }

    /// The path from the fork point (exclusive) to `id` (inclusive), oldest
    /// first, and the fork height.
    fn path_from_main(&self, id: &BlockId) -> Result<(u64, Vec<BlockId>), ChainError> {
        let mut path = Vec::new();
        let mut cur = *id;
        loop {
            if self.store.on_main(&cur) {
                let h = self.entry(&cur).map(|e| e.header.height).unwrap_or(0);
                path.reverse();
                return Ok((h, path));
            }
            let e = self.entry(&cur).ok_or(ChainError::Store("broken chain".into()))?;
            path.push(cur);
            cur = e.header.prev;
        }
    }

    /// Fork choice after a header or body arrives: move to the heaviest
    /// chain whose bodies are all present (§5.1–§5.3), or say what is
    /// missing. A heavier chain the reorg limit or a checkpoint forbids is
    /// reported and skipped in favour of the next candidate.
    pub fn try_advance(&mut self) -> Result<BlockVerdict, ChainError> {
        let tip = self.tip_id();
        let tip_work = self.entry(&tip).map(|e| e.chainwork).unwrap_or(U256::ZERO);
        if !select::heavier(self.heaviest, tip_work) {
            return Ok(BlockVerdict::Known);
        }
        let mut first_error = None;
        for best in self.candidates() {
            let (fork_height, path) = self.path_from_main(&best)?;
            let tip_height = self.store.tip_height();
            if !select::reorg_allowed(tip_height, fork_height) {
                first_error.get_or_insert(ChainError::DeepFork { fork_height, tip_height });
                continue;
            }
            if let Some(floor) = self.checkpoints.floor() {
                if fork_height < floor {
                    let expected = self.checkpoints.get(floor).unwrap_or([0; 32]);
                    first_error.get_or_insert(ChainError::Checkpoint { height: floor, expected });
                    continue;
                }
            }
            if let Some((height, expected)) = self.checkpoints.violation(|h| self.store.ancestor_at(&best, h)) {
                // the chain excludes a held checkpoint: not a candidate, nor its descendants
                self.invalidate(best);
                first_error.get_or_insert(ChainError::Checkpoint { height, expected });
                continue;
            }
            let missing: Vec<BlockId> = path.iter().filter(|id| !self.store.has_body(id)).copied().collect();
            if !missing.is_empty() {
                return Ok(BlockVerdict::NeedBodies(missing));
            }
            return self.reorg(fork_height, &path);
        }
        match first_error {
            Some(e) => Err(e),
            None => Ok(BlockVerdict::Stored),
        }
    }

    /// Does the reorg limit or a checkpoint floor forbid switching to `id`?
    fn own_chain_blocked(&self, id: &BlockId) -> bool {
        let Ok((fork_height, _)) = self.path_from_main(id) else { return false };
        let deep = !select::reorg_allowed(self.store.tip_height(), fork_height);
        let below_floor = self.checkpoints.floor().is_some_and(|f| fork_height < f);
        let vetoed = self.entry(id).is_some_and(|e| e.status == store::Status::Invalid);
        deep || below_floor || vetoed
    }

    fn reorg(&mut self, fork_height: u64, path: &[BlockId]) -> Result<BlockVerdict, ChainError> {
        let old_tip = self.tip_id();
        let depth = self.store.tip_height() - fork_height;
        let mut retracted = Vec::new();
        while self.store.tip_height() > fork_height {
            retracted.push(self.retract()?);
        }
        let mut applied = 0usize;
        let mut failure = None;
        for id in path {
            let (header, txs) = match self.decoded(id) {
                Ok(d) => d,
                Err(e) => {
                    self.invalidate(*id);
                    failure = Some(e);
                    break;
                }
            };
            if let Err(e) = self.extend(&header, &txs) {
                failure = Some(e);
                break;
            }
            applied += 1;
        }
        if let Some(e) = failure {
            // every descendant of the failed block is invalid too
            for id in &path[applied + 1..] {
                self.invalidate(*id);
            }
            for _ in 0..applied {
                self.retract()?;
            }
            for id in retracted.iter().rev() {
                let (header, txs) = self.decoded(id)?;
                self.extend(&header, &txs)?;
            }
            self.store.persist().map_err(ChainError::Store)?;
            return Err(e);
        }
        self.after_tip_move()?;
        Ok(BlockVerdict::Reorganized { old_tip, depth })
    }

    fn after_tip_move(&mut self) -> Result<(), ChainError> {
        self.store.persist().map_err(ChainError::Store)?;
        let new_final = self.store.tip_height().saturating_sub(FINALITY_DEPTH);
        while self.finalized < new_final {
            let h = self.finalized + 1;
            if let Some(id) = self.store.main_at(h) {
                if let (Some(e), Some(body)) = (self.entry(&id), self.store.body(&id)) {
                    self.events.push(Event::Block { header: e.header, body, state_root: e.header.state_root });
                }
            }
            self.finalized = h;
        }
        Ok(())
    }
}

impl ChainApi for Chain {
    fn chain_id(&self) -> Id {
        self.rules.chain
    }

    fn genesis(&self) -> &Genesis {
        &self.genesis
    }

    fn tip(&self) -> Tip {
        let (height, id) = self.store.tip().expect("genesis present");
        let e = self.entry(&id).expect("tip stored");
        Tip { height, id, chainwork: e.chainwork, header: e.header }
    }

    fn status(&self) -> Status {
        let tip = self.tip();
        let native = self.rules.native;
        let conserved = apply::check_conservation(&self.rules, &self.state, tip.height).is_ok();
        Status {
            chain: self.rules.chain,
            name: self.genesis.params().name.clone(),
            rule_set: RULE_SET,
            height: tip.height,
            tip: tip.id,
            chainwork: tip.chainwork,
            bits: tip.header.bits,
            next_bits: next_bits(self.genesis.anchor(), &tip.header),
            genesis_block: self.store.main_at(0).unwrap_or([0; 32]),
            genesis_time: self.genesis.params().time,
            state_root: tip.header.state_root,
            supply: self.state.supply(&native),
            conserved,
            subsidy: self.rules.subsidy,
            neurons: self.state.neuron_count(),
            particles: self.state.particle_count(),
            axons: self.state.axon_count(),
            checkpoint: self.checkpoints.floor(),
            finalized: self.finalized,
        }
    }

    fn header(&self, id: &BlockId) -> Option<Header> {
        self.entry(id).map(|e| e.header)
    }

    fn header_at(&self, height: u64) -> Option<Header> {
        self.store.main_at(height).and_then(|id| self.header(&id))
    }

    fn headers(&self, from: u64, count: usize) -> Vec<Header> {
        self.store.main_range(from, count).iter().filter_map(|id| self.header(id)).collect()
    }

    fn block(&self, id: &BlockId) -> Option<Block> {
        let header = self.header(id)?;
        let body = if header.height == 0 { Vec::new() } else { self.store.body(id)? };
        Some(Block { header, body })
    }

    fn blocks(&self, from: u64, count: usize) -> Vec<Block> {
        self.store.main_range(from, count).iter().filter_map(|id| self.block(id)).collect()
    }

    fn chainwork(&self, id: &BlockId) -> Option<U256> {
        self.entry(id).map(|e| e.chainwork)
    }

    fn locator(&self) -> Vec<BlockId> {
        select::locator_heights(self.store.tip_height()).iter().filter_map(|h| self.store.main_at(*h)).collect()
    }

    fn ancestor_at(&self, id: &BlockId, height: u64) -> Option<BlockId> {
        self.store.ancestor_at(id, height)
    }

    fn balance(&self, owner: &Id, token: &Id) -> u64 {
        self.state.balance(owner, token)
    }

    fn nonce(&self, neuron: &Id) -> u64 {
        self.state.nonce(neuron)
    }

    fn supply(&self, token: &Id) -> u64 {
        self.state.supply(token)
    }

    fn finalized_height(&self) -> u64 {
        self.finalized
    }

    fn accept_header(&mut self, header: &Header, now: u64) -> Result<HeaderVerdict, ChainError> {
        let id = header.id();
        if self.rejected.contains(&id) {
            return Err(ChainError::Rejected(id));
        }
        if let Some(e) = self.entry(&id) {
            return if e.status == store::Status::Invalid { Err(ChainError::Rejected(id)) } else { Ok(HeaderVerdict::Known) };
        }
        let work = self.check_header(header, now)?;
        self.store.put_header(*header, work, store::Status::Header).map_err(ChainError::Store)?;
        self.heaviest = self.heaviest.max(work);
        Ok(HeaderVerdict::Accepted)
    }

    fn accept_block(&mut self, block: &Block, now: u64) -> Result<BlockVerdict, ChainError> {
        let id = block.id();
        self.accept_header(&block.header, now)?;
        if self.entry(&id).is_some_and(|e| e.status == store::Status::Valid) {
            return self.try_advance().map(|v| if v == BlockVerdict::Stored { BlockVerdict::Known } else { v });
        }
        let mut stored_new = false;
        if !self.store.has_body(&id) {
            stored_new = true;
            let txs = match apply::decode_body(&self.rules, &block.header, &block.body) {
                Ok(txs) => txs,
                Err(e) => {
                    self.invalidate(id);
                    return Err(ChainError::Block(e));
                }
            };
            self.store.put_body(&id, &block.body).map_err(ChainError::Store)?;
            self.store.set_status(&id, store::Status::Body).map_err(ChainError::Store)?;
            if block.header.prev == self.tip_id() {
                self.extend(&block.header, &txs)?;
                self.after_tip_move()?;
                let _ = self.try_advance();
                return Ok(BlockVerdict::Extended);
            }
        }
        match self.try_advance() {
            Ok(BlockVerdict::Known) if stored_new => Ok(BlockVerdict::Stored),
            Ok(v) => Ok(v),
            // a blocked candidate elsewhere is not this block's fault: only
            // report the error when this block's own chain is the blocked one
            Err(e) if self.own_chain_blocked(&id) => Err(e),
            Err(_) => Ok(BlockVerdict::Stored),
        }
    }

    fn accept_checkpoint(&mut self, cp: &Checkpoint) -> Result<bool, ChainError> {
        let new = self.checkpoints.accept(&self.rules.chain, cp).map_err(ChainError::CheckpointRejected)?;
        if new && self.store.main_at(cp.height).is_some_and(|id| id != cp.block_id) {
            // our main chain contradicts the checkpoint: it is not our chain
            // any more — the net layer resyncs from peers that carry it
            eprintln!("chain: checkpoint at {} contradicts the main chain", cp.height);
        }
        Ok(new)
    }

    fn checkpoints(&self) -> Vec<(u64, BlockId)> {
        self.checkpoints.held().map(|(h, id)| (*h, *id)).collect()
    }

    fn validate_tx(&mut self, tx_bytes: &[u8], pending: &[super::tx::Tx]) -> Result<super::api::TxInfo, super::apply::TxError> {
        super::engine_tx::validate_tx(self, tx_bytes, pending)
    }

    fn header_template(&mut self, to: Id, time: u64, mempool: &[Vec<u8>]) -> Result<super::api::Template, ChainError> {
        super::engine_tx::header_template(self, to, time, mempool)
    }

    fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }
}

impl Chain {
    pub(super) fn state_mut(&mut self) -> &mut BbgBacked {
        &mut self.state
    }

    pub(super) fn mtp_of_tip(&self) -> u64 {
        self.mtp(&self.tip_id())
    }

    pub(super) fn tip_header(&self) -> Header {
        self.tip().header
    }

    pub(super) fn genesis_anchor(&self) -> super::daa::Anchor {
        self.genesis.anchor()
    }
}
