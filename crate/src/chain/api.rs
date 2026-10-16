//! The chain API: the surface the network layer (`chain/net/`), the miner
//! (`chain/mine/`) and the HTTP routes code against. Stable; additions are
//! fine, changes to existing signatures are a coordinated change.
//!
//! Clock discipline: nothing here reads the wall clock. Every entry point
//! that applies the future-drift rule takes `now` (unix seconds) so tests
//! and the net layer's hold-and-retry are deterministic.

use super::apply::{BlockError, TxError};
use super::genesis::Genesis;
use super::header::{BlockId, Header, HeaderError};
use super::select::{Checkpoint, CheckpointError};
use super::state::Id;
use super::tx::Tx;
use super::u256::U256;

/// The canonical tip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tip {
    pub height: u64,
    pub id: BlockId,
    pub chainwork: U256,
    pub header: Header,
}

/// A header with its body (`count ‖ tx…`, §1.6). Block 0 has an empty body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub header: Header,
    pub body: Vec<u8>,
}

impl Block {
    pub fn id(&self) -> BlockId {
        self.header.id()
    }

    /// `header[168] ‖ body` — the `B` frame payload of chain-wire §3.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = self.header.encode().to_vec();
        out.extend_from_slice(&self.body);
        out
    }

    pub fn decode(b: &[u8]) -> Result<Block, HeaderError> {
        let header = Header::decode_raw(b.get(..168).ok_or(HeaderError::Length(b.len()))?)?;
        Ok(Block { header, body: b[168..].to_vec() })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HeaderVerdict {
    /// passed steps 1–5, stored; relay it
    Accepted,
    /// already in the store
    Known,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockVerdict {
    /// the block extended the tip
    Extended,
    /// the tip moved to this block's chain, undoing `depth` blocks from `old_tip`
    Reorganized { old_tip: BlockId, depth: u64 },
    /// valid as far as checked (steps 1–6) and stored; not heavier than the
    /// tip, so steps 7–8 run if it ever becomes the heaviest chain
    Stored,
    /// heavier than the tip, but these ancestors have no body yet; fetch
    /// them (chain-wire §3.1) and resubmit any one of the chain
    NeedBodies(Vec<BlockId>),
    /// already applied
    Known,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainError {
    /// step 1
    Header(HeaderError),
    /// step 2: `prev` not in the store (the net layer's orphan pool takes it)
    UnknownParent(BlockId),
    /// step 2: the parent is known and invalid
    InvalidParent(BlockId),
    /// step 2
    Height { expected: u64, actual: u64 },
    /// step 3: `time <= MTP(prev)`
    Time { mtp: u64, actual: u64 },
    /// step 3: ahead of the clock; hold and re-evaluate, never score
    FutureTime { time: u64, now: u64 },
    /// step 4
    Bits { expected: u32, actual: u32 },
    /// step 5
    Pow,
    /// §7.1: the activation table ends before this height
    RuleSetUnknown(u64),
    /// §5.2: the fork point is at or below `tip − FINALITY_DEPTH`
    DeepFork { fork_height: u64, tip_height: u64 },
    /// §5.3: the chain excludes a held checkpoint
    Checkpoint { height: u64, expected: BlockId },
    /// §5.3
    CheckpointRejected(CheckpointError),
    /// steps 6–8; the header is now marked invalid with every descendant
    Block(BlockError),
    /// an id in the rejected set (§5.4)
    Rejected(BlockId),
    /// genesis block mismatch on a block claiming height 0
    Genesis,
    /// the store
    Store(String),
}

impl std::fmt::Display for ChainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainError::Header(e) => write!(f, "header: {e}"),
            ChainError::UnknownParent(_) => write!(f, "unknown parent"),
            ChainError::InvalidParent(_) => write!(f, "parent is invalid"),
            ChainError::Height { expected, actual } => write!(f, "height {actual}, expected {expected}"),
            ChainError::Time { mtp, actual } => write!(f, "time {actual} is not above the median time past {mtp}"),
            ChainError::FutureTime { time, now } => write!(f, "time {time} is more than the drift ahead of {now}"),
            ChainError::Bits { expected, actual } => write!(f, "bits {actual:#010x}, ASERT expects {expected:#010x}"),
            ChainError::Pow => write!(f, "block id does not meet the target"),
            ChainError::RuleSetUnknown(h) => write!(f, "no rule set is known for height {h}; this release stops here"),
            ChainError::DeepFork { fork_height, tip_height } => write!(f, "fork at {fork_height} is beyond MAX_REORG from the tip {tip_height}"),
            ChainError::Checkpoint { height, .. } => write!(f, "chain excludes the checkpoint at {height}"),
            ChainError::CheckpointRejected(e) => write!(f, "checkpoint: {e}"),
            ChainError::Block(e) => write!(f, "block: {e}"),
            ChainError::Rejected(_) => write!(f, "previously rejected block"),
            ChainError::Genesis => write!(f, "genesis block mismatch"),
            ChainError::Store(e) => write!(f, "store: {e}"),
        }
    }
}

impl std::error::Error for ChainError {}

/// What a mempool learns from a validated transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TxInfo {
    pub txid: Id,
    pub tx: Tx,
    pub size: usize,
    pub fee: u64,
    /// the account whose nonce the tx consumes (the claim's key-derived neuron)
    pub account: Id,
    pub nonce: u64,
}

/// A mining template (chain-wire §7 `/template`): the header with zero
/// nonce and the ASERT target, plus the body it commits to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Template {
    pub header: Header,
    pub target: U256,
    pub body: Vec<u8>,
    pub txids: Vec<Id>,
    pub fees: u64,
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

impl Template {
    /// The `GET /template` answer of chain-wire §7, exactly.
    pub fn json(&self, chain_name: &str, expires: u64) -> String {
        format!(
            "{{\"chain\":{},\"height\":{},\"prev\":\"{}\",\"header\":\"{}\",\"bits\":\"{:08x}\",\"target\":\"{}\",\"body\":\"{}\",\"expires\":{}}}",
            serde_json::to_string(chain_name).unwrap_or_else(|_| "\"\"".into()),
            self.header.height,
            hex(&self.header.prev),
            hex(&self.header.encode()),
            self.header.bits,
            hex(&self.target.to_le_bytes()),
            hex(&self.body),
            expires
        )
    }
}

impl Tip {
    /// The `GET /tip` answer of chain-wire §7, exactly.
    pub fn json(&self) -> String {
        format!("{{\"height\":{},\"id\":\"{}\",\"chainwork\":\"{}\"}}", self.height, hex(&self.id), self.chainwork.to_decimal())
    }
}

/// Finalized block stream for the shadow indexers (§7.3): emitted once a
/// block is `FINALITY_DEPTH` deep, never reorged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Block { header: Header, body: Vec<u8>, state_root: Id },
}

/// `/status` fields the chain owns (chain-wire §7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status {
    pub chain: Id,
    pub name: String,
    pub rule_set: u16,
    pub height: u64,
    pub tip: BlockId,
    pub chainwork: U256,
    pub bits: u32,
    pub next_bits: u32,
    pub genesis_block: BlockId,
    pub genesis_time: u64,
    pub state_root: Id,
    pub supply: u64,
    pub conserved: bool,
    pub subsidy: u64,
    pub neurons: u64,
    pub particles: u64,
    pub axons: u64,
    pub checkpoint: Option<u64>,
    pub finalized: u64,
}

/// The chain as the other layers see it.
pub trait ChainApi {
    fn chain_id(&self) -> Id;
    fn genesis(&self) -> &Genesis;
    fn tip(&self) -> Tip;
    fn status(&self) -> Status;

    // ── reads ───────────────────────────────────────────────────────
    fn header(&self, id: &BlockId) -> Option<Header>;
    /// main-chain header at a height
    fn header_at(&self, height: u64) -> Option<Header>;
    /// main-chain headers `[from, from + count)`, ascending, truncated at the tip
    fn headers(&self, from: u64, count: usize) -> Vec<Header>;
    fn block(&self, id: &BlockId) -> Option<Block>;
    /// main-chain blocks `[from, from + count)`, ascending
    fn blocks(&self, from: u64, count: usize) -> Vec<Block>;
    fn chainwork(&self, id: &BlockId) -> Option<U256>;
    /// main-chain ids for a get-headers locator (chain-wire §3)
    fn locator(&self) -> Vec<BlockId>;
    /// the main-chain id at `height` on the branch ending at `id`
    fn ancestor_at(&self, id: &BlockId, height: u64) -> Option<BlockId>;
    fn balance(&self, owner: &Id, token: &Id) -> u64;
    fn nonce(&self, neuron: &Id) -> u64;
    fn supply(&self, token: &Id) -> u64;
    fn finalized_height(&self) -> u64;

    // ── writes ──────────────────────────────────────────────────────
    /// steps 1–5; stores a valid header
    fn accept_header(&mut self, header: &Header, now: u64) -> Result<HeaderVerdict, ChainError>;
    /// steps 1–8 and fork choice; the header is accepted first
    fn accept_block(&mut self, block: &Block, now: u64) -> Result<BlockVerdict, ChainError>;
    /// §5.3; `Ok(true)` when new
    fn accept_checkpoint(&mut self, cp: &Checkpoint) -> Result<bool, ChainError>;
    fn checkpoints(&self) -> Vec<(u64, BlockId)>;
    /// mempool admission (chain-wire §4): decode, signature, nonce within
    /// `NONCE_WINDOW` given `pending` (the mempool's txs of the same
    /// account, applied first in nonce order), fee floor, balances and
    /// maturity on the tip state. Never mutates the chain.
    fn validate_tx(&mut self, tx_bytes: &[u8], pending: &[Tx]) -> Result<TxInfo, TxError>;
    /// a template on the tip for a miner paying `to`, at `time` (raised to
    /// MTP + 1 if needed), from candidate transactions already admitted
    /// (chain-wire §4: fee per byte descending, nonce order kept, budgets)
    fn header_template(&mut self, to: Id, time: u64, mempool: &[Vec<u8>]) -> Result<Template, ChainError>;
    /// drain the finalized block events
    fn take_events(&mut self) -> Vec<Event>;
}
