//! Block transition (§3.4) and the per-transaction rules (§2), steps 6–8 of
//! the validation order (§5.5). Pure over a [`StateBackend`]: the caller
//! supplies the parent state, the immature-coinbase map and the clock-free
//! context; this module mutates under a journal and returns it, or reverts
//! and returns the first error.

use std::collections::{BTreeMap, BTreeSet};

use super::header::Header;
use super::params::{COINBASE_MATURITY, HEADER_BYTES, MAX_BLOCK_BYTES, MAX_BLOCK_TXS, MAX_TX_BYTES, MIN_FEE_PER_BYTE};
use super::sigs::{self, legacy_owner, neuron_of, sig_pubkey, txid};
use super::state::{Id, StateBackend, StateError, Undo};
use super::tree::tx_root;
use super::tx::{self, DecodeError, Tx, TxBody};

/// The chain-level facts a transition needs (§0, §4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    pub chain: Id,
    pub native: Id,
    pub hrp: String,
    pub subsidy: u64,
    pub s0: u64,
}

/// Native units per owner still inside `COINBASE_MATURITY` at the height
/// being validated (§2.2): spendable = balance − locked.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Locked(pub BTreeMap<Id, u64>);

impl Locked {
    /// From the coinbases `(height, to, amount)` of recent blocks, the map
    /// for a block at `at`: a coinbase at `c` is locked while `at < c + 100`.
    pub fn at(coinbases: &[(u64, Id, u64)], at: u64) -> Locked {
        let mut m = BTreeMap::new();
        for (c, to, amount) in coinbases {
            if at < c.saturating_add(COINBASE_MATURITY) {
                let e = m.entry(*to).or_insert(0u64);
                *e = e.saturating_add(*amount);
            }
        }
        Locked(m)
    }

    pub fn of(&self, owner: &Id) -> u64 {
        self.0.get(owner).copied().unwrap_or(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxError {
    Decode(DecodeError),
    TooLarge(usize),
    CoinbaseInBody,
    Signature,
    Nonce { expected: u64, actual: u64 },
    Fee { min: u64, actual: u64 },
    Insufficient { have: u64, need: u64 },
    Immature { spendable: u64, need: u64 },
    SelfLink,
    Box(u64),
    ClaimKey,
    Overflow,
}

impl std::fmt::Display for TxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TxError::Decode(e) => write!(f, "{e}"),
            TxError::TooLarge(n) => write!(f, "{n} bytes exceeds MAX_TX_BYTES"),
            TxError::CoinbaseInBody => write!(f, "coinbase outside position 0"),
            TxError::Signature => write!(f, "signature does not verify"),
            TxError::Nonce { expected, actual } => write!(f, "nonce {actual}, expected {expected}"),
            TxError::Fee { min, actual } => write!(f, "fee {actual} below floor {min}"),
            TxError::Insufficient { have, need } => write!(f, "insufficient balance: have {have}, need {need}"),
            TxError::Immature { spendable, need } => write!(f, "coinbase immature: spendable {spendable}, need {need}"),
            TxError::SelfLink => write!(f, "cyberlink p == q"),
            TxError::Box(a) => write!(f, "cyberlink amount {a} > 0 is not in rule set 1"),
            TxError::ClaimKey => write!(f, "claim key does not derive the account id"),
            TxError::Overflow => write!(f, "arithmetic overflow"),
        }
    }
}

impl From<StateError> for TxError {
    fn from(e: StateError) -> Self {
        match e {
            StateError::Insufficient { have, need } => TxError::Insufficient { have, need },
            StateError::Overflow => TxError::Overflow,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockError {
    Body(DecodeError),
    Empty,
    TooManyTxs(usize),
    TooLarge(usize),
    TxRoot,
    CoinbasePosition,
    CoinbaseHeight { expected: u64, actual: u64 },
    DuplicateTx(usize),
    CoinbaseAmount { expected: u64, actual: u64 },
    Tx { index: usize, error: TxError },
    StateRoot { expected: Id, actual: Id },
    Conservation { token: Id, sum: u64, supply: u64 },
    Overflow,
}

impl std::fmt::Display for BlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockError::Body(e) => write!(f, "body: {e}"),
            BlockError::Empty => write!(f, "body has no coinbase"),
            BlockError::TooManyTxs(n) => write!(f, "{n} transactions exceed MAX_BLOCK_TXS"),
            BlockError::TooLarge(n) => write!(f, "{n} bytes exceed MAX_BLOCK_BYTES"),
            BlockError::TxRoot => write!(f, "tx_root mismatch"),
            BlockError::CoinbasePosition => write!(f, "tx[0] is not a coinbase"),
            BlockError::CoinbaseHeight { expected, actual } => write!(f, "coinbase height {actual}, block height {expected}"),
            BlockError::DuplicateTx(i) => write!(f, "tx {i} duplicates an earlier txid"),
            BlockError::CoinbaseAmount { expected, actual } => write!(f, "coinbase amount {actual}, expected {expected}"),
            BlockError::Tx { index, error } => write!(f, "tx {index}: {error}"),
            BlockError::StateRoot { .. } => write!(f, "state_root mismatch"),
            BlockError::Conservation { sum, supply, .. } => write!(f, "conservation: balances sum {sum}, supply {supply}"),
            BlockError::Overflow => write!(f, "arithmetic overflow"),
        }
    }
}

/// One decoded body transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decoded {
    pub raw: Vec<u8>,
    pub tx: Tx,
    pub txid: Id,
}

/// Step 6: budgets, `tx_root`, coinbase position and height, unique txids.
/// Stateless; a block passing this may still fail steps 7–8.
pub fn decode_body(rules: &Rules, header: &Header, body: &[u8]) -> Result<Vec<Decoded>, BlockError> {
    if HEADER_BYTES + body.len() > MAX_BLOCK_BYTES {
        return Err(BlockError::TooLarge(HEADER_BYTES + body.len()));
    }
    let raws = tx::split_body(body).map_err(BlockError::Body)?;
    if raws.is_empty() {
        return Err(BlockError::Empty);
    }
    if raws.len() > MAX_BLOCK_TXS {
        return Err(BlockError::TooManyTxs(raws.len()));
    }
    if tx_root(&raws) != header.tx_root {
        return Err(BlockError::TxRoot);
    }
    let mut seen = BTreeSet::new();
    let mut out = Vec::with_capacity(raws.len());
    for (i, raw) in raws.into_iter().enumerate() {
        if raw.len() > MAX_TX_BYTES {
            return Err(BlockError::Tx { index: i, error: TxError::TooLarge(raw.len()) });
        }
        let tx = Tx::decode(&raw).map_err(|e| BlockError::Tx { index: i, error: TxError::Decode(e) })?;
        match (&tx.body, i) {
            (TxBody::Coinbase(c), 0) => {
                if c.height != header.height {
                    return Err(BlockError::CoinbaseHeight { expected: header.height, actual: c.height });
                }
            }
            (TxBody::Coinbase(_), _) => return Err(BlockError::Tx { index: i, error: TxError::CoinbaseInBody }),
            (_, 0) => return Err(BlockError::CoinbasePosition),
            _ => {}
        }
        let id = txid(&rules.chain, &raw);
        if !seen.insert(id) {
            return Err(BlockError::DuplicateTx(i));
        }
        out.push(Decoded { raw, tx, txid: id });
    }
    Ok(out)
}

fn spendable(state: &impl StateBackend, locked: &Locked, owner: &Id, native: &Id, need: u64) -> Result<(), TxError> {
    let have = state.balance(owner, native);
    let lock = locked.of(owner);
    if have < need {
        return Err(TxError::Insufficient { have, need });
    }
    if have - lock < need {
        return Err(TxError::Immature { spendable: have - lock, need });
    }
    Ok(())
}

/// Step 7 for one non-coinbase transaction at its exact nonce: every check,
/// then the mutation (§2.3–§2.8). Returns the fee paid. Nothing is
/// mutated if any check fails.
pub fn execute_tx<S: StateBackend>(rules: &Rules, tx: &Tx, raw_len: usize, state: &mut S, locked: &Locked, height: u64) -> Result<u64, TxError> {
    let min_fee = (raw_len as u64).saturating_mul(MIN_FEE_PER_BYTE);
    let fee = tx.fee();
    if fee < min_fee {
        return Err(TxError::Fee { min: min_fee, actual: fee });
    }
    match &tx.body {
        TxBody::Coinbase(_) => Err(TxError::CoinbaseInBody),
        TxBody::Transfer(t) => {
            if !sigs::verify(&rules.chain, tx) {
                return Err(TxError::Signature);
            }
            let expected = state.nonce(&t.from);
            if t.nonce != expected {
                return Err(TxError::Nonce { expected, actual: t.nonce });
            }
            if t.token == rules.native {
                let need = t.amount.checked_add(fee).ok_or(TxError::Overflow)?;
                spendable(state, locked, &t.from, &rules.native, need)?;
            } else {
                spendable(state, locked, &t.from, &rules.native, fee)?;
                let have = state.balance(&t.from, &t.token);
                if have < t.amount {
                    return Err(TxError::Insufficient { have, need: t.amount });
                }
            }
            state.debit(&t.from, &rules.native, fee)?;
            state.debit(&t.from, &t.token, t.amount)?;
            state.credit(&t.to, &t.token, t.amount)?;
            state.bump_nonce(&t.from)?;
            Ok(fee)
        }
        TxBody::Cyberlink(l) => {
            if l.p == l.q {
                return Err(TxError::SelfLink);
            }
            if l.amount != 0 {
                return Err(TxError::Box(l.amount));
            }
            if !sigs::verify(&rules.chain, tx) {
                return Err(TxError::Signature);
            }
            let expected = state.nonce(&l.from);
            if l.nonce != expected {
                return Err(TxError::Nonce { expected, actual: l.nonce });
            }
            spendable(state, locked, &l.from, &rules.native, fee)?;
            state.debit(&l.from, &rules.native, fee)?;
            state.link(&l.from, &l.p, &l.q, &l.token, 0, l.valence, height)?;
            state.bump_nonce(&l.from)?;
            Ok(fee)
        }
        TxBody::Claim(c) => {
            let Some(pubkey) = sig_pubkey(&tx.sig) else {
                return Err(TxError::Signature);
            };
            if mudra::cosmos::account_id(&pubkey) != c.account_id {
                return Err(TxError::ClaimKey);
            }
            if !sigs::verify(&rules.chain, tx) {
                return Err(TxError::Signature);
            }
            let subject = neuron_of(&pubkey);
            let expected = state.nonce(&subject);
            if c.nonce != expected {
                return Err(TxError::Nonce { expected, actual: c.nonce });
            }
            let dormant = legacy_owner(&rules.hrp, &c.account_id);
            spendable(state, locked, &dormant, &rules.native, fee)?;
            state.debit(&dormant, &rules.native, fee)?;
            for token in state.tokens() {
                let bal = state.balance(&dormant, &token);
                if bal > 0 {
                    state.debit(&dormant, &token, bal)?;
                    state.credit(&subject, &token, bal)?;
                }
            }
            state.ensure_neuron(&subject);
            state.bump_nonce(&subject)?;
            Ok(fee)
        }
    }
}

/// Steps 7–8 on the parent state: coinbase exactness, every transaction in
/// order, height, `state_root`, conservation. On success the state is the
/// block's and the journal is returned; on failure the state is the
/// parent's again.
pub fn apply_block<S: StateBackend>(rules: &Rules, header: &Header, txs: &[Decoded], state: &mut S, locked: &Locked) -> Result<Undo, BlockError> {
    let TxBody::Coinbase(cb) = &txs.first().ok_or(BlockError::Empty)?.tx.body else {
        return Err(BlockError::CoinbasePosition);
    };
    let mut fees = 0u64;
    for d in &txs[1..] {
        fees = fees.checked_add(d.tx.fee()).ok_or(BlockError::Overflow)?;
    }
    let expected = rules.subsidy.checked_add(fees).ok_or(BlockError::Overflow)?;
    if cb.amount != expected {
        return Err(BlockError::CoinbaseAmount { expected, actual: cb.amount });
    }
    state.begin();
    let result = (|| {
        state.credit(&cb.to, &rules.native, cb.amount).map_err(|_| BlockError::Overflow)?;
        state.mint(&rules.native, rules.subsidy).map_err(|_| BlockError::Overflow)?;
        for (i, d) in txs.iter().enumerate().skip(1) {
            execute_tx(rules, &d.tx, d.raw.len(), state, locked, header.height).map_err(|error| BlockError::Tx { index: i, error })?;
        }
        state.set_height(header.height);
        let root = state.root();
        if root != header.state_root {
            return Err(BlockError::StateRoot { expected: header.state_root, actual: root });
        }
        check_conservation(rules, state, header.height)
    })();
    let undo = state.commit();
    match result {
        Ok(()) => Ok(undo),
        Err(e) => {
            state.revert(&undo);
            Err(e)
        }
    }
}

/// §3.4 invariant: every token's balances sum to its supply, and the native
/// supply is `S0 + h · SUBSIDY`.
pub fn check_conservation(rules: &Rules, state: &impl StateBackend, height: u64) -> Result<(), BlockError> {
    for token in state.tokens() {
        let (sum, supply) = (state.token_sum(&token), state.supply(&token));
        if sum != supply {
            return Err(BlockError::Conservation { token, sum, supply });
        }
    }
    let expected = height.checked_mul(rules.subsidy).and_then(|m| m.checked_add(rules.s0)).ok_or(BlockError::Overflow)?;
    let supply = state.supply(&rules.native);
    if supply != expected {
        return Err(BlockError::Conservation { token: rules.native, sum: expected, supply });
    }
    Ok(())
}
