//! What the wire asks of the consensus kernel — the operations of
//! chain.md §5 and chain-wire.md §3–4, as a trait. Until the kernel's
//! `Chain` (chain/api.rs) lands, [`super::devchain`] implements it; the swap
//! is an adapter over this trait, nothing in `net/` names the kernel.

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
}

impl HeaderError {
    /// chain-wire §5 score for the failure.
    pub fn score(&self) -> u32 {
        match self {
            HeaderError::BadPow | HeaderError::WrongChain => 100,
            HeaderError::Syntax(_) | HeaderError::BadBits | HeaderError::BadTime | HeaderError::BadTarget => 10,
            HeaderError::BadParent | HeaderError::InvalidParent | HeaderError::UnknownRuleSet => 10,
        }
    }
}

impl std::fmt::Display for HeaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeaderError::Syntax(s) => write!(f, "syntax: {s}"),
            other => write!(f, "{other:?}"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockVerdict {
    /// Applied; the tip moved to this block.
    NewTip,
    /// Stored and valid, but not heavier than the tip.
    Stored,
    /// Body already held.
    Known,
    /// Valid headers on a fork deeper than `MAX_REORG`: stored, never applied.
    DeepFork,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BlockError {
    /// The header was never accepted (steps 1–5) — the body was unsolicited.
    UnknownHeader,
    /// Parent body missing: apply in height order.
    MissingParent,
    Body(String),
    Tx(String),
    StateRoot,
    InvalidParent,
}

impl std::fmt::Display for BlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
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
    fn accept_block(&self, header: &Header, body: &[u8]) -> Result<BlockVerdict, BlockError>;
    /// Heights of the best header chain above the tip whose bodies we lack, ascending.
    fn missing_bodies(&self, max: usize) -> Vec<(u64, Id)>;

    fn nonce(&self, neuron: &Id) -> u64;
    fn balance(&self, owner: &Id, token: &Id) -> u64;
    /// Balance minus immature coinbase credits.
    fn spendable(&self, owner: &Id, token: &Id) -> u64;
    fn balances(&self, owner: &Id) -> Vec<(Id, u64)>;
    fn tx_location(&self, txid: &Id) -> Option<(u64, Id)>;
    fn supply(&self, token: &Id) -> u64;
    fn conserved(&self) -> bool;
    fn counts(&self) -> (u64, u64, u64);
    fn state_root(&self) -> Id;

    /// A header with zero nonce and the body it commits to, from the tip.
    fn template(&self, to: &Id, txs: &[Vec<u8>], now: u64) -> Result<(Header, Vec<u8>), String>;
    fn events(&self) -> tokio::sync::broadcast::Receiver<ChainEvent>;
    /// Transactions of a body as byte strings (empty when undecodable).
    fn txs_of(&self, body: &[u8]) -> Vec<Vec<u8>> {
        super::codec::decode_body(body).unwrap_or_default()
    }
}
