//! The chain engine: [`Chain`] implements [`ChainApi`] over the store, the
//! bbg-backed state, the checkpoints and the validation order of §5.5.
//! Headers are validated in steps 1–5 and stored; bodies run steps 6–8 on
//! the parent state; fork choice (§5.1–§5.3) moves the tip with the undo
//! journals. The [`ChainApi`](super::api::ChainApi) impl is in
//! [`super::engine_api`]; mempool and template entry points in
//! [`super::engine_tx`].

use std::collections::{HashSet, VecDeque};
use std::path::Path;
use std::sync::Arc;

use super::api::{BlockVerdict, ChainError, Event};
use super::apply::{self, Decoded, Locked, Rules};
use super::bbg_state::BbgBacked;
use super::daa::{median_time_past, next_bits};
use super::economy::focus::{FocusEngine, Tru};
use super::economy::run::Economy;
use super::economy::settle::SettleResult;
use super::economy::weights::GenesisWeights;
use super::genesis::Genesis;
use super::header::{BlockId, Header};
use super::params::{COINBASE_MATURITY, EpochClock, FINALITY_DEPTH, MAX_FUTURE_DRIFT, MTP_WINDOW, RULE_SET, rule_set_at};
use super::pow::{chainwork, valid_pow};
use super::select::{self, Checkpoints};
use super::state::{Id, StateBackend};
use super::store::{self, Store};
use super::tx::Body;
use super::u256::U256;

const REJECTED_CAPACITY: usize = 4_096;

pub struct Chain {
    pub(super) genesis: Genesis,
    pub(super) rules: Rules,
    pub(super) store: Store,
    pub(super) state: BbgBacked,
    pub(super) checkpoints: Checkpoints,
    /// main-chain coinbases `(height, to, amount)` still inside maturity
    pub(super) coinbases: VecDeque<(u64, Id, u64)>,
    pub(super) rejected: HashSet<BlockId>,
    pub(super) rejected_order: VecDeque<BlockId>,
    /// the most work any non-invalid header carries; when the tip has it,
    /// there is nothing to consider
    pub(super) heaviest: U256,
    pub(super) finalized: u64,
    pub(super) events: Vec<Event>,
    /// the focus economy's snapshots, settlement workers and outcomes
    pub(super) economy: Economy,
}

/// The launch chains: they run the spec's epoch clock and nothing else.
pub const LAUNCH_CHAINS: &[&str] = &["pussy", "bostrom"];

/// How a chain is opened beyond its genesis file.
#[derive(Clone, Default)]
pub struct OpenOptions {
    pub checkpoints: bool,
    /// a shortened epoch clock for a test or dev net; ignored (the spec's
    /// clock is used) for the launch chains
    pub clock: Option<EpochClock>,
    /// the φ* engine; tru's epoch engine when `None`
    pub engine: Option<Arc<dyn FocusEngine>>,
    /// gradient nodes on the epoch path; `ATTR_NODES` (the protocol) when 0
    pub quadrature: usize,
    /// compute settlements inline instead of on worker threads
    pub inline_settlement: bool,
}

impl Chain {
    /// Open or create the chain under `home` (`home/blocks` is the store).
    /// A store whose block 0 differs from the genesis file refuses to open.
    pub fn open(home: &Path, genesis_bytes: &[u8], checkpoints_enabled: bool) -> Result<Chain, ChainError> {
        let clock = EpochClock::from_env();
        Chain::open_with(home, genesis_bytes, OpenOptions { checkpoints: checkpoints_enabled, clock, ..Default::default() })
    }

    pub fn open_with(home: &Path, genesis_bytes: &[u8], opts: OpenOptions) -> Result<Chain, ChainError> {
        let genesis = Genesis::decode(genesis_bytes).map_err(|e| ChainError::Store(e.to_string()))?;
        let launch = LAUNCH_CHAINS.contains(&genesis.params().name.as_str());
        let clock = match opts.clock {
            Some(c) if !launch && c.valid() => c,
            _ => EpochClock::SPEC,
        };
        let rules = Rules {
            chain: genesis.chain_id,
            native: genesis.native_token(),
            hrp: genesis.params().hrp.clone(),
            subsidy: genesis.subsidy(),
            s0: genesis.s0(),
            clock,
        };
        std::fs::create_dir_all(home).map_err(|e| ChainError::Store(e.to_string()))?;
        let mut store = Store::open(&home.join("blocks")).map_err(ChainError::Store)?;
        let mut state = genesis.state();
        let root0 = state.root();
        let block0 = genesis.block0(root0);
        let checkpoints = Checkpoints::new(genesis.params().checkpoint_key, opts.checkpoints);
        if store.is_empty() {
            store.put_header(block0, U256::ZERO, store::Status::Valid).map_err(ChainError::Store)?;
            store.push_main(block0.id()).map_err(ChainError::Store)?;
            store.persist().map_err(ChainError::Store)?;
        } else if store.main_at(0) != Some(block0.id()) {
            return Err(ChainError::Genesis);
        }
        let engine: Arc<dyn FocusEngine> = opts.engine.unwrap_or_else(|| Arc::new(Tru::for_node()));
        let mut economy = Economy::new(GenesisWeights::from_file(&genesis.file), engine, clock, Some(home.join("epochs")));
        economy.background = !opts.inline_settlement;
        economy.quadrature = if opts.quadrature == 0 { super::params::ATTR_NODES } else { opts.quadrature };
        economy.on_boundary(&state, 0, block0.id(), root0, rules.s0, None);
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
            economy,
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
            self.apply_on_tip(&header, &txs, &locked).map_err(ChainError::Block)?;
            self.push_coinbase(h, &txs);
        }
        self.economy.prune(tip);
        Ok(())
    }

    /// The settlement the block at `height` (on the main tip) must carry,
    /// if one is due: the result reading the main chain's boundaries of
    /// epochs E − 1 and E (chain-economy §2.5).
    pub(super) fn settlement_for(&mut self, height: u64) -> Option<Arc<SettleResult>> {
        let e = self.rules.clock.settles_at(height)?;
        let prev = self.store.main_at(self.rules.clock.boundary(e - 1))?;
        let cur = self.store.main_at(self.rules.clock.boundary(e))?;
        self.economy.result(&prev, &cur)
    }

    /// Steps 7–8 for a block whose parent is the main tip, with the
    /// economy's side effects: the settlement result in, the outcome kept,
    /// the boundary snapshot taken and its settlement started.
    fn apply_on_tip(&mut self, header: &Header, txs: &[Decoded], locked: &Locked) -> Result<super::state::Undo, apply::BlockError> {
        let settle = self.settlement_for(header.height);
        let applied = apply::apply_block(&self.rules, header, txs, &mut self.state, locked, settle.as_deref())?;
        let id = header.id();
        if let Some(outcome) = applied.settled {
            self.economy.record_outcome(id, outcome);
        }
        let clock = self.rules.clock;
        if clock.is_boundary(header.height) {
            let epoch = clock.epoch(header.height);
            let prev = self.store.main_at(clock.boundary(epoch - 1));
            let supply = self.rules.supply_at(header.height);
            self.economy.on_boundary(&self.state, epoch, id, header.state_root, supply, prev);
        }
        Ok(applied.undo)
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

    pub(super) fn entry(&self, id: &BlockId) -> Option<&store::Entry> {
        self.store.get(id)
    }

    pub(super) fn tip_id(&self) -> BlockId {
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

    pub(super) fn invalidate(&mut self, id: BlockId) {
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
    pub(super) fn check_header(&self, header: &Header, now: u64) -> Result<U256, ChainError> {
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
        // §5.3: a chain not containing a held checkpoint is rejected — at the
        // checkpointed height itself, whatever path the block arrives by
        if let Some(expected) = self.checkpoints.get(header.height) {
            if header.id() != expected {
                return Err(ChainError::Checkpoint { height: header.height, expected });
            }
        }
        Ok(chainwork(parent.chainwork, header))
    }

    /// Apply a block whose parent is the tip (steps 7–8), then extend.
    pub(super) fn extend(&mut self, header: &Header, txs: &[Decoded]) -> Result<(), ChainError> {
        let id = header.id();
        let locked = self.locked_at(header.height);
        let undo = match self.apply_on_tip(header, txs, &locked) {
            Ok(undo) => undo,
            // a settlement this node cannot compute is its own condition,
            // never the block's fault (chain-economy §2.5: it waits)
            Err(e @ apply::BlockError::Settlement { .. }) => return Err(ChainError::Block(e)),
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
                // reorganizing below the floor is what a checkpoint forbids;
                // a tip still under it moves up through it (a chain that
                // excludes it is caught by `violation` below)
                if fork_height < floor && tip_height >= floor {
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

    /// The heaviest non-invalid header whose chain the node may still move
    /// to — the target of header-first body download (chain-wire §3.2) —
    /// or the tip when there is none. A heavier chain that the reorg limit
    /// or a checkpoint forbids is skipped: its bodies can never be applied,
    /// and following it would starve the bodies of the best chain the node
    /// can take (a released private fork deeper than `MAX_REORG` otherwise
    /// strands every honest node that sits on a stale branch).
    pub fn best_reachable(&self) -> (u64, BlockId, U256) {
        let tip = self.tip_id();
        let (tip_height, tip_work) = self.entry(&tip).map(|e| (e.header.height, e.chainwork)).unwrap_or((0, U256::ZERO));
        if !select::heavier(self.heaviest, tip_work) {
            return (tip_height, tip, tip_work);
        }
        for id in self.candidates() {
            if !self.own_chain_blocked(&id) {
                if let Some(e) = self.entry(&id) {
                    return (e.header.height, id, e.chainwork);
                }
            }
        }
        (tip_height, tip, tip_work)
    }

    /// A held checkpoint contradicts our main chain at `height` (chain.md
    /// §5.3: a chain not containing it is rejected — our own too): retract
    /// below `height`, mark the retracted blocks invalid, and move to the
    /// heaviest chain left (the checkpointed one when its blocks are known;
    /// sync fetches them otherwise). Bounded by the reorg limit like any
    /// reorganization: deeper than `MAX_REORG`, the node logs and keeps its
    /// chain — the operator resyncs (§5.2).
    pub(super) fn reject_main_from(&mut self, height: u64, ours: BlockId) -> Result<(), ChainError> {
        let tip_height = self.store.tip_height();
        let fork = height.saturating_sub(1);
        if !select::reorg_allowed(tip_height, fork) {
            eprintln!("chain: checkpoint at {height} contradicts the main chain {} blocks deep, beyond MAX_REORG: resync from a checkpoint", tip_height - fork);
            return Ok(());
        }
        self.invalidate(ours);
        while self.store.tip_height() > fork {
            let id = self.retract()?;
            self.invalidate(id);
        }
        self.heaviest = self.store.iter().filter(|(_, e)| e.status != store::Status::Invalid).map(|(_, e)| e.chainwork).max().unwrap_or(U256::ZERO);
        self.after_tip_move()?;
        let _ = self.try_advance();
        Ok(())
    }

    /// Does the reorg limit or a checkpoint floor forbid switching to `id`?
    pub(super) fn own_chain_blocked(&self, id: &BlockId) -> bool {
        let Ok((fork_height, _)) = self.path_from_main(id) else { return false };
        let tip_height = self.store.tip_height();
        let deep = !select::reorg_allowed(tip_height, fork_height);
        let below_floor = self.checkpoints.floor().is_some_and(|f| fork_height < f && tip_height >= f);
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

    pub(super) fn after_tip_move(&mut self) -> Result<(), ChainError> {
        self.store.persist().map_err(ChainError::Store)?;
        self.economy.prune(self.store.tip_height());
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

impl Chain {
    pub(super) fn state_mut(&mut self) -> &mut BbgBacked {
        &mut self.state
    }

    pub(super) fn mtp_of_tip(&self) -> u64 {
        self.mtp(&self.tip_id())
    }

    pub(super) fn tip_header(&self) -> Header {
        let id = self.tip_id();
        self.entry(&id).expect("tip stored").header
    }

    pub(super) fn genesis_anchor(&self) -> super::daa::Anchor {
        self.genesis.anchor()
    }
}
