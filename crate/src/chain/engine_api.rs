//! `impl ChainApi for Chain`: the reads, the accept entry points and the
//! checkpoint/mempool/template delegations, over the engine's internals.

use super::api::{Block, BlockVerdict, ChainApi, ChainError, Event, HeaderVerdict, Status, Template, Tip, TxInfo};
use super::apply::{self, TxError};
use super::daa::next_bits;
use super::engine::Chain;
use super::genesis::Genesis;
use super::header::{BlockId, Header};
use super::params::RULE_SET;
use super::select::{self, Checkpoint};
use super::state::{Id, StateBackend};
use super::store;
use super::tx::Tx;
use super::u256::U256;

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
        if self.vetoed.contains(&id) {
            return Err(self.vetoed_error(header.height));
        }
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
            // 6a: a body that is not the one the header commits to is dropped;
            // the header stays as it was and the honest body may still come
            let raws = apply::body_matches(&block.header, &block.body).map_err(ChainError::Block)?;
            // 6b: the header commits to this body, so a failure condemns the header
            let txs = match apply::decode_matched(&self.rules, &block.header, raws) {
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
        if new {
            if let Some(ours) = self.store.main_at(cp.height).filter(|id| *id != cp.block_id) {
                self.reject_main_from(cp.height, ours)?;
            }
        }
        Ok(new)
    }

    fn checkpoints(&self) -> Vec<(u64, BlockId)> {
        self.checkpoints.held().map(|(h, id)| (*h, *id)).collect()
    }

    fn validate_tx(&mut self, tx_bytes: &[u8], pending: &[Tx]) -> Result<TxInfo, TxError> {
        super::engine_tx::validate_tx(self, tx_bytes, pending)
    }

    fn header_template(&mut self, to: Id, time: u64, mempool: &[Vec<u8>]) -> Result<Template, ChainError> {
        super::engine_tx::header_template(self, to, time, mempool)
    }

    fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }
}

