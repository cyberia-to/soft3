//! Block transition (§3.4) and the per-transaction rules (§2), steps 6–8 of
//! the validation order (§5.5). Pure over a [`StateBackend`]: the caller
//! supplies the parent state, the immature-coinbase map and the clock-free
//! context; this module mutates under a journal and returns it, or reverts
//! and returns the first error.

use std::collections::{BTreeMap, BTreeSet};

use super::economy::settle::{self as econ_settle, Outcome, SettleResult};
use super::economy::{self, Card, card_id, focus_key, stake_key};
use super::header::Header;
use super::params::{COINBASE_MATURITY, EpochClock, HEADER_BYTES, MAX_BLOCK_BYTES, MAX_BLOCK_TXS, MAX_TX_BYTES, MIN_FEE_PER_BYTE};
use super::sigs::{self, legacy_owner, neuron_of, sig_pubkey, txid};
use super::state::{Id, StateBackend, StateError, Undo};
use super::tree::tx_root;
use super::tx::{self, Body, DecodeError, Tx};

/// The chain-level facts a transition needs (§0, §4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    pub chain: Id,
    pub native: Id,
    pub hrp: String,
    pub subsidy: u64,
    pub s0: u64,
    /// the epoch clock: `EpochClock::SPEC` on the launch chains
    pub clock: EpochClock,
}

impl Rules {
    /// `S0 + h · SUBSIDY`: the native supply after block `h` (§3.5).
    pub fn supply_at(&self, h: u64) -> u64 {
        self.s0 + h * self.subsidy
    }
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
    /// a staked cyberlink names a token other than the native coin (chain-economy §3.3)
    StakeToken,
    /// the card exists with the other valence: unlock to zero first (§3.3)
    Valence { card: i8, tx: i8 },
    /// kinds 6 and 7 need `card(from, p, q)` to exist (§3.4–§3.5)
    NoCard,
    /// an unlock before `since + MIN_LOCK` (§3.4)
    Locked { until: u64 },
    /// an unlock of 0 or of more than the position's stake (§3.4)
    UnlockAmount { stake: u64, amount: u64 },
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
            TxError::StakeToken => write!(f, "stake must be the native coin"),
            TxError::Valence { card, tx } => write!(f, "the position holds valence {card}; unlock it to zero before locking {tx}"),
            TxError::NoCard => write!(f, "no position card(from, p, q)"),
            TxError::Locked { until } => write!(f, "stake locked until height {until}"),
            TxError::UnlockAmount { stake, amount } => write!(f, "unlock {amount} of a position holding {stake}"),
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
    /// the settlement of `epoch` is due at this height and the node has no
    /// result for it (chain-economy §2.5: it waits, never guesses)
    Settlement { epoch: u64 },
    /// Σ stake keys differs from the running stake total (§6.3)
    StakeTotal { cards: u128, total: u64 },
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
            BlockError::Settlement { epoch } => write!(f, "settlement of epoch {epoch} is not computed"),
            BlockError::StakeTotal { cards, total } => write!(f, "stake keys sum {cards}, running total {total}"),
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
            (Body::Coinbase { height, .. }, 0) => {
                if *height != header.height {
                    return Err(BlockError::CoinbaseHeight { expected: header.height, actual: *height });
                }
            }
            (Body::Coinbase { .. }, _) => return Err(BlockError::Tx { index: i, error: TxError::CoinbaseInBody }),
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
        Body::Coinbase { .. } => Err(TxError::CoinbaseInBody),
        Body::Transfer { from, nonce, token, amount, to, .. } => {
            if !sigs::verify(&rules.chain, tx) {
                return Err(TxError::Signature);
            }
            let expected = state.nonce(from);
            if *nonce != expected {
                return Err(TxError::Nonce { expected, actual: *nonce });
            }
            if *token == rules.native {
                let need = amount.checked_add(fee).ok_or(TxError::Overflow)?;
                spendable(state, locked, from, &rules.native, need)?;
            } else {
                spendable(state, locked, from, &rules.native, fee)?;
                let have = state.balance(from, token);
                if have < *amount {
                    return Err(TxError::Insufficient { have, need: *amount });
                }
            }
            state.debit(from, &rules.native, fee)?;
            state.debit(from, token, *amount)?;
            state.credit(to, token, *amount)?;
            state.bump_nonce(from)?;
            Ok(fee)
        }
        Body::Cyberlink { from, nonce, token, amount, p, q, valence, .. } => {
            if p == q {
                return Err(TxError::SelfLink);
            }
            if *amount > 0 && *token != rules.native {
                return Err(TxError::StakeToken);
            }
            if !sigs::verify(&rules.chain, tx) {
                return Err(TxError::Signature);
            }
            let expected = state.nonce(from);
            if *nonce != expected {
                return Err(TxError::Nonce { expected, actual: *nonce });
            }
            let card = (*amount > 0).then(|| card_id(from, p, q));
            let existing = card.and_then(|c| state.card(&c));
            if let Some(c) = &existing {
                if c.valence != *valence {
                    return Err(TxError::Valence { card: c.valence, tx: *valence });
                }
            }
            let need = amount.checked_add(fee).ok_or(TxError::Overflow)?;
            spendable(state, locked, from, &rules.native, need)?;
            state.debit(from, &rules.native, need)?;
            if let Some(c) = card {
                // chain-economy §3.3: the stake moves to the unreachable key
                // of the position; any increase restarts the lock
                state.credit_raw(&stake_key(&c), &rules.native, *amount)?;
                let next = match existing {
                    Some(old) => Card { since: height, ..old },
                    None => Card { owner: *from, particle: economy::axon_of(p, q), author: *from, valence: *valence, since: height },
                };
                state.set_card(&c, Some(next));
                let mut sc = state.scalars();
                sc.total_stake = sc.total_stake.checked_add(*amount).ok_or(TxError::Overflow)?;
                if *valence != 0 {
                    sc.active_stake = sc.active_stake.checked_add(*amount).ok_or(TxError::Overflow)?;
                }
                state.set_scalars(sc);
            }
            state.link(from, p, q, token, 0, *valence, height)?;
            state.bump_nonce(from)?;
            Ok(fee)
        }
        Body::Unlock { from, nonce, p, q, amount, .. } => {
            if !sigs::verify(&rules.chain, tx) {
                return Err(TxError::Signature);
            }
            let expected = state.nonce(from);
            if *nonce != expected {
                return Err(TxError::Nonce { expected, actual: *nonce });
            }
            let c = card_id(from, p, q);
            let card = state.card(&c).ok_or(TxError::NoCard)?;
            let key = stake_key(&c);
            let stake = state.raw_balance(&key);
            if *amount == 0 || *amount > stake {
                return Err(TxError::UnlockAmount { stake, amount: *amount });
            }
            let until = card.since.saturating_add(rules.clock.min_lock);
            if height < until {
                return Err(TxError::Locked { until });
            }
            // the fee may come out of what the unlock returns (§3.4)
            let have = state.balance(from, &rules.native).checked_add(*amount).ok_or(TxError::Overflow)?;
            let spendable_after = have - locked.of(from).min(have);
            if spendable_after < fee {
                return Err(TxError::Immature { spendable: spendable_after, need: fee });
            }
            state.debit_raw(&key, &rules.native, *amount)?;
            state.credit(from, &rules.native, *amount)?;
            state.debit(from, &rules.native, fee)?;
            if stake == *amount {
                state.set_card(&c, None);
            }
            let mut sc = state.scalars();
            sc.total_stake -= *amount;
            if card.valence != 0 {
                sc.active_stake -= *amount;
            }
            state.set_scalars(sc);
            state.bump_nonce(from)?;
            Ok(fee)
        }
        Body::Payout { from, nonce, p, q, payout, .. } => {
            if !sigs::verify(&rules.chain, tx) {
                return Err(TxError::Signature);
            }
            let expected = state.nonce(from);
            if *nonce != expected {
                return Err(TxError::Nonce { expected, actual: *nonce });
            }
            let c = card_id(from, p, q);
            let card = state.card(&c).ok_or(TxError::NoCard)?;
            spendable(state, locked, from, &rules.native, fee)?;
            state.debit(from, &rules.native, fee)?;
            state.set_card(&c, Some(Card { owner: *payout, ..card }));
            state.bump_nonce(from)?;
            Ok(fee)
        }
        Body::Claim { account_id, nonce, .. } => {
            let Some(pubkey) = sig_pubkey(&tx.sig) else {
                return Err(TxError::Signature);
            };
            if mudra::cosmos::account_id(&pubkey) != *account_id {
                return Err(TxError::ClaimKey);
            }
            if !sigs::verify(&rules.chain, tx) {
                return Err(TxError::Signature);
            }
            let subject = neuron_of(&pubkey);
            let expected = state.nonce(&subject);
            if *nonce != expected {
                return Err(TxError::Nonce { expected, actual: *nonce });
            }
            let dormant = legacy_owner(&rules.hrp, account_id);
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

/// What a successful transition produced.
#[derive(Clone, Debug)]
pub struct Applied {
    pub root: Id,
    pub undo: Undo,
    /// the settlement this block carried (chain-economy §5.6), if any
    pub settled: Option<Outcome>,
}

/// Steps 7–8 on the parent state: coinbase exactness, the settlement step
/// when due, every transaction in order, height, `state_root`,
/// conservation. On success the state is the block's and the journal is
/// returned; on failure the state is the parent's again.
pub fn apply_block<S: StateBackend>(
    rules: &Rules,
    header: &Header,
    txs: &[Decoded],
    state: &mut S,
    locked: &Locked,
    settle: Option<&SettleResult>,
) -> Result<Applied, BlockError> {
    let applied = transition(rules, header.height, txs, state, locked, settle)?;
    if applied.root != header.state_root {
        state.revert(&applied.undo);
        return Err(BlockError::StateRoot { expected: header.state_root, actual: applied.root });
    }
    Ok(applied)
}

/// `(miner share, pool cut)` of one block's subsidy under the split in
/// force on `state` (chain-economy §6.1–§6.2).
pub fn subsidy_split(rules: &Rules, state: &impl StateBackend) -> (u64, u64) {
    let cut = economy::cut(rules.subsidy, state.scalars().focus_bps);
    (rules.subsidy - cut, cut)
}

/// The transition without the root comparison: what a template builder
/// runs to learn the root its header must carry. Same journal discipline.
/// `settle` is the precomputed settlement of the epoch due at `height`
/// (chain-economy §2.5); a settlement height without it is an error.
pub fn transition<S: StateBackend>(
    rules: &Rules,
    height: u64,
    txs: &[Decoded],
    state: &mut S,
    locked: &Locked,
    settle: Option<&SettleResult>,
) -> Result<Applied, BlockError> {
    let Body::Coinbase { to: cb_to, amount: cb_amount, .. } = &txs.first().ok_or(BlockError::Empty)?.tx.body else {
        return Err(BlockError::CoinbasePosition);
    };
    let due = rules.clock.settles_at(height);
    if let Some(epoch) = due {
        if settle.map(|r| r.epoch) != Some(epoch) {
            return Err(BlockError::Settlement { epoch });
        }
    }
    let mut fees = 0u64;
    for d in &txs[1..] {
        fees = fees.checked_add(d.tx.fee()).ok_or(BlockError::Overflow)?;
    }
    let (miner, cut) = subsidy_split(rules, state);
    let expected = miner.checked_add(fees).ok_or(BlockError::Overflow)?;
    if *cb_amount != expected {
        return Err(BlockError::CoinbaseAmount { expected, actual: *cb_amount });
    }
    state.begin();
    let mut settled = None;
    let result = (|| {
        // 1. coinbase: the miner's share and the pool's cut (§5.7, §6.2)
        #[cfg(feature = "opcount")]
        let _kind = super::opcount::kind(0);
        state.credit(cb_to, &rules.native, *cb_amount).map_err(|_| BlockError::Overflow)?;
        state.credit_raw(&focus_key(&rules.native), &rules.native, cut).map_err(|_| BlockError::Overflow)?;
        state.mint(&rules.native, rules.subsidy).map_err(|_| BlockError::Overflow)?;
        #[cfg(feature = "opcount")]
        drop(_kind);
        // 2. the settlement of the epoch due at this height
        #[cfg(feature = "opcount")]
        let _kind = super::opcount::kind(super::opcount::SETTLE);
        if let (Some(_), Some(r)) = (due, settle) {
            settled = Some(econ_settle::apply(state, &rules.native, height, r).map_err(|_| BlockError::Overflow)?);
        }
        #[cfg(feature = "opcount")]
        drop(_kind);
        // 3. the transactions in body order
        for (i, d) in txs.iter().enumerate().skip(1) {
            #[cfg(feature = "opcount")]
            let _kind = super::opcount::kind(d.raw[1]);
            execute_tx(rules, &d.tx, d.raw.len(), state, locked, height).map_err(|error| BlockError::Tx { index: i, error })?;
        }
        state.set_height(height);
        // a boundary fixes the split for the next epoch's blocks (§6.1)
        if rules.clock.is_boundary(height) {
            let mut sc = state.scalars();
            sc.focus_bps = economy::focus_bps(sc.active_stake, rules.supply_at(height));
            state.set_scalars(sc);
        }
        // 4. conservation and the root
        #[cfg(feature = "opcount")]
        let _kind = super::opcount::kind(super::opcount::BLOCK);
        check_conservation(rules, state, height)?;
        if due.is_some() {
            #[cfg(feature = "opcount")]
            let _kind = super::opcount::kind(super::opcount::SETTLE);
            check_stake_total(state)?;
        }
        Ok(state.root())
    })();
    let undo = state.commit();
    match result {
        Ok(root) => Ok(Applied { root, undo, settled }),
        Err(e) => {
            state.revert(&undo);
            Err(e)
        }
    }
}

/// §6.3: Σ stake keys over the cards equals the running stake total.
pub fn check_stake_total(state: &impl StateBackend) -> Result<(), BlockError> {
    let cards = econ_settle::stake_total(state);
    let total = state.scalars().total_stake;
    if cards != total as u128 {
        return Err(BlockError::StakeTotal { cards, total });
    }
    Ok(())
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
