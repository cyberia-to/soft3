//! What the wire asks of the consensus kernel — the operations of
//! chain.md §5 and chain-wire.md §3–4, as a trait: the port `net/` codes
//! against. [`super::kernel::KernelChain`] implements it over the kernel's
//! `ChainApi` (chain/api.rs); nothing else in `net/` names the kernel.

use super::codec::{Header, Id, U256};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tip {
    pub height: u64,
    pub id: Id,
    pub chainwork: U256,
}

/// Steps 1–5 of chain.md §5.5 passed, or why not.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HeaderVerdict {
    /// New, valid, stored; relay it.
    Accepted,
    /// Already known (duplicate).
    Known,
    /// `prev` unknown — pooled; ask the sender for headers.
    Orphan,
    /// Fails only the future-drift rule; held for re-evaluation.
    Held,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum HeaderError {
    Syntax(String),
    WrongChain,
    BadBits,
    BadParent,
    BadTime,
    BadTarget,
    BadPow,
    InvalidParent,
    UnknownRuleSet,
    /// A fork beyond `MAX_REORG` or one that excludes a held checkpoint
    /// (chain.md §5.2–5.3): logged, never scored — an honest peer on a
    /// stale branch sends these.
    Refused(String),
    /// The local store failed: our problem, not the peer's.
    Store(String),
}

impl HeaderError {
    /// chain-wire §5 score for the failure.
    pub fn score(&self) -> u32 {
        match self {
            HeaderError::BadPow | HeaderError::WrongChain => 100,
            HeaderError::Syntax(_) | HeaderError::BadBits | HeaderError::BadTime | HeaderError::BadTarget => 10,
            HeaderError::BadParent | HeaderError::InvalidParent | HeaderError::UnknownRuleSet => 10,
            HeaderError::Refused(_) | HeaderError::Store(_) => 0,
        }
    }
}

impl std::fmt::Display for HeaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeaderError::Syntax(s) => write!(f, "syntax: {s}"),
            HeaderError::Refused(s) => write!(f, "refused: {s}"),
            HeaderError::Store(s) => write!(f, "store: {s}"),
            other => write!(f, "{other:?}"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BlockVerdict {
    /// Applied; the tip moved (extended, or reorganized onto this chain).
    NewTip,
    /// Stored and valid as far as checked, but not heavier than the tip.
    Stored,
    /// Body already held.
    Known,
    /// Heavier than the tip, but these ancestors have no body yet: fetch
    /// them (chain-wire §3.1) and resubmit any block of the chain.
    NeedBodies(Vec<Id>),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BlockError {
    /// The parent header is unknown — sync headers from the sender first.
    UnknownParent,
    /// The header fails steps 1–5 (PoW, bits, time, parent).
    Header(HeaderError),
    /// The header is ahead of our clock: hold it and retry, never score.
    Held,
    /// A fork beyond `MAX_REORG` or one that excludes a checkpoint: logged.
    Refused(String),
    /// Steps 6–8 failed: the block (and every descendant) is invalid; the
    /// peer that served it is scored.
    Invalid(String),
    /// The local store failed.
    Store(String),
}

impl BlockError {
    /// chain-wire §5 score for the peer that served the block.
    pub fn score(&self) -> u32 {
        match self {
            BlockError::Header(e) => e.score(),
            BlockError::Invalid(_) => 10,
            BlockError::UnknownParent | BlockError::Held | BlockError::Refused(_) | BlockError::Store(_) => 0,
        }
    }
}

impl std::fmt::Display for BlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockError::UnknownParent => write!(f, "unknown parent"),
            BlockError::Header(e) => write!(f, "header: {e}"),
            BlockError::Held => write!(f, "ahead of the clock; held"),
            BlockError::Refused(s) => write!(f, "refused: {s}"),
            BlockError::Invalid(s) => write!(f, "invalid: {s}"),
            BlockError::Store(s) => write!(f, "store: {s}"),
        }
    }
}

/// What mempool admission learns from the kernel (chain-wire §4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Admitted {
    pub txid: Id,
    /// the account whose nonce the tx consumes (a claim's key-derived neuron)
    pub account: Id,
    pub nonce: u64,
    pub fee: u64,
}

/// Why the kernel refuses a transaction, in the mempool's terms.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TxRefusal {
    Decode(String),
    Kind,
    Signature,
    Nonce { current: u64, got: u64 },
    Fee { floor: u64, got: u64 },
    Balance(String),
    Rule(String),
}

#[derive(Clone, Debug)]
pub enum ChainEvent {
    /// The tip moved; `reorged_out` lists bodies that left the main chain.
    NewTip { height: u64, id: Id, reorged_out: Vec<Vec<u8>> },
}

pub trait ChainView: Send + Sync + 'static {
    fn chain_id(&self) -> Id;
    fn genesis_id(&self) -> Id;
    fn chain_name(&self) -> String;
    /// The bech32 prefix of the legacy accounts (chain.md §2.8).
    fn hrp(&self) -> String;
    fn rule_set(&self) -> u16 {
        super::codec::RULE_SET
    }
    fn native_token(&self) -> Id;
    fn subsidy(&self) -> u64;
    fn genesis_time(&self) -> u64;
    /// The foundation checkpoint key (chain.md §5.3); `None` when all-zero.
    fn checkpoint_key(&self) -> Option<[u8; 33]>;

    fn tip(&self) -> Tip;
    /// The most-work header known, bodies or not.
    fn best_header(&self) -> Tip;
    fn header(&self, id: &Id) -> Option<Header>;
    /// Main-chain header at a height.
    fn header_at(&self, height: u64) -> Option<Header>;
    fn chainwork_of(&self, id: &Id) -> Option<U256>;
    fn has_body(&self, id: &Id) -> bool;
    fn block(&self, id: &Id) -> Option<(Header, Vec<u8>)>;
    fn block_at(&self, height: u64) -> Option<(Header, Vec<u8>)>;
    fn height_of(&self, id: &Id) -> Option<u64>;

    /// Our main-chain locator: the last 10 ids then doubling spacing to genesis.
    fn locator(&self) -> Vec<Id>;
    /// Headers after the first locator entry on our main chain, ascending, ≤ `max`.
    fn headers_from_locator(&self, locator: &[Id], stop: &Id, max: usize) -> Vec<Header>;

    fn accept_header(&self, header: &Header, now: u64) -> Result<HeaderVerdict, HeaderError>;
    /// Steps 1–8 and fork choice; the header is accepted first.
    fn accept_block(&self, header: &Header, body: &[u8], now: u64) -> Result<BlockVerdict, BlockError>;
    /// Re-evaluate the headers held for being ahead of the clock; returns
    /// the ones accepted now (to relay).
    fn retry_held(&self, now: u64) -> Vec<Header>;
    /// A checkpoint whose signature the wire verified (chain.md §5.3).
    fn accept_checkpoint(&self, height: u64, block_id: &Id, sig: &[u8]) -> Result<bool, String>;
    /// Heights of the best header chain above the tip whose bodies we lack, ascending.
    fn missing_bodies(&self, max: usize) -> Vec<(u64, Id)>;

    fn nonce(&self, neuron: &Id) -> u64;
    /// Mempool admission on the tip state, `pending` (this account's lower
    /// nonces, wire bytes) applied first; never mutates the chain.
    fn validate_tx(&self, tx: &[u8], pending: &[Vec<u8>]) -> Result<Admitted, TxRefusal>;
    fn balance(&self, owner: &Id, token: &Id) -> u64;
    /// Balance minus immature coinbase credits.
    fn spendable(&self, owner: &Id, token: &Id) -> u64;
    fn balances(&self, owner: &Id) -> Vec<(Id, u64)>;
    /// `(height, block, index)` of a main-chain transaction.
    fn tx_location(&self, txid: &Id) -> Option<(u64, Id, u32)>;
    fn supply(&self, token: &Id) -> u64;
    fn conserved(&self) -> bool;
    fn counts(&self) -> (u64, u64, u64);
    fn state_root(&self) -> Id;

    /// A header with zero nonce and the body it commits to, from the tip.
    fn template(&self, to: &Id, txs: &[Vec<u8>], now: u64) -> Result<(Header, Vec<u8>), String>;
    /// The answer of `GET /template` (chain-wire §7), rendered by the kernel.
    fn template_json(&self, header: &Header, body: &[u8], expires: u64) -> String;
    fn events(&self) -> tokio::sync::broadcast::Receiver<ChainEvent>;
    /// Transactions of a body as byte strings (empty when undecodable).
    fn txs_of(&self, body: &[u8]) -> Vec<Vec<u8>> {
        super::codec::decode_body(body).unwrap_or_default()
    }
}
