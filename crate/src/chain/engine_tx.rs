//! Mempool admission and block templates on the tip state (chain-wire §4,
//! §7 `/template`). Both run scratch transitions under the state journal
//! and revert, so the chain is never mutated by a query.

use std::collections::BTreeMap;

use super::api::{ChainApi, ChainError, Template, TxInfo};
use super::apply::{self, Decoded, TxError};
use super::engine::Chain;
use super::header::Header;
use super::params::{HEADER_BYTES, MAX_BLOCK_BYTES, MAX_BLOCK_TXS, MAX_TX_BYTES, NONCE_WINDOW};
use super::sigs::{subject, txid};
use super::state::{Id, StateBackend};
use super::tree::tx_root;
use super::tx::{self, Body, Tx};

/// Admission on the tip state assuming the `pending` txs of the same
/// account with lower nonces apply first; nonce must be within
/// `[current, current + NONCE_WINDOW)`; balances and maturity as in a block.
pub fn validate_tx(chain: &mut Chain, tx_bytes: &[u8], pending: &[Tx]) -> Result<TxInfo, TxError> {
    if tx_bytes.len() > MAX_TX_BYTES {
        return Err(TxError::TooLarge(tx_bytes.len()));
    }
    let tx = Tx::decode(tx_bytes).map_err(TxError::Decode)?;
    if tx.is_coinbase() {
        return Err(TxError::CoinbaseInBody);
    }
    let account = subject(&tx).ok_or(TxError::Signature)?;
    // chain-wire §4 admission order: the signature before nonce, fee and
    // balance (a block checks all of them; a mempool scores the peer by
    // which one failed first)
    if !super::sigs::verify(&chain.rules().chain, &tx) {
        return Err(TxError::Signature);
    }
    let nonce = tx.nonce().ok_or(TxError::CoinbaseInBody)?;
    let current = chain.nonce(&account);
    if nonce < current || nonce >= current.saturating_add(NONCE_WINDOW) {
        return Err(TxError::Nonce { expected: current, actual: nonce });
    }
    let height = chain.tip().height + 1;
    let locked = chain.locked_at(height);
    let rules = chain.rules().clone();
    let mut earlier: Vec<&Tx> = pending.iter().filter(|p| subject(p) == Some(account) && p.nonce().is_some_and(|n| n < nonce)).collect();
    earlier.sort_by_key(|p| p.nonce());
    let state = chain.state_mut();
    state.begin();
    let result = (|| {
        for p in earlier {
            apply::execute_tx(&rules, p, p.encode().len(), state, &locked, height)?;
        }
        apply::execute_tx(&rules, &tx, tx_bytes.len(), state, &locked, height)
    })();
    let undo = state.commit();
    state.revert(&undo);
    let fee = result?;
    Ok(TxInfo { txid: txid(&rules.chain, tx_bytes), tx, size: tx_bytes.len(), fee, account, nonce })
}

/// Fee per byte ordering: `a` before `b` when `fee_a / len_a > fee_b / len_b`,
/// compared as cross products (no division, no floats).
fn fee_rate_desc(a: &(Vec<u8>, Tx), b: &(Vec<u8>, Tx)) -> std::cmp::Ordering {
    let (fa, la) = (a.1.fee() as u128, a.0.len() as u128);
    let (fb, lb) = (b.1.fee() as u128, b.0.len() as u128);
    (fb * la).cmp(&(fa * lb))
}

/// Build a template on the tip: transactions by fee per byte descending
/// with per-account nonce order kept and the §1.6 budgets, then the
/// coinbase from `SUBSIDY + Σ fees`, the tx root and the state root.
pub fn header_template(chain: &mut Chain, to: Id, time: u64, mempool: &[Vec<u8>]) -> Result<Template, ChainError> {
    let rules = chain.rules().clone();
    let parent = chain.tip_header();
    let height = parent.height + 1;
    let locked = chain.locked_at(height);
    let mtp = chain.mtp_of_tip();
    let time = time.max(mtp + 1);

    let mut candidates: Vec<(Vec<u8>, Tx)> = mempool
        .iter()
        .filter(|raw| raw.len() <= MAX_TX_BYTES)
        .filter_map(|raw| Tx::decode(raw).ok().filter(|t| !t.is_coinbase()).map(|t| (raw.clone(), t)))
        .collect();
    candidates.sort_by(fee_rate_desc);

    // selection runs on the same state the block will see: coinbase first
    // (its amount is unknown yet, so credit nothing — only the mint), then
    // transactions in the order chosen, each kept only if it executes
    let state = chain.state_mut();
    state.begin();
    let mut chosen: Vec<(Vec<u8>, Tx)> = Vec::new();
    let mut bytes = HEADER_BYTES + 4 + tx::COINBASE_BYTES;
    let mut fees = 0u64;
    let mut next_nonce: BTreeMap<Id, u64> = BTreeMap::new();
    let mut done = vec![false; candidates.len()];
    let mut progress = true;
    while progress && chosen.len() + 1 < MAX_BLOCK_TXS {
        progress = false;
        for (i, (raw, tx)) in candidates.iter().enumerate() {
            if done[i] || chosen.len() + 1 >= MAX_BLOCK_TXS {
                continue;
            }
            let Some(account) = subject(tx) else {
                done[i] = true;
                continue;
            };
            let expected = *next_nonce.entry(account).or_insert_with(|| state.nonce(&account));
            let Some(nonce) = tx.nonce() else {
                done[i] = true;
                continue;
            };
            if nonce < expected {
                done[i] = true;
                continue;
            }
            if nonce > expected {
                continue; // wait for the lower nonce
            }
            if bytes + raw.len() > MAX_BLOCK_BYTES {
                done[i] = true;
                continue;
            }
            match apply::execute_tx(&rules, tx, raw.len(), state, &locked, height) {
                Ok(fee) => {
                    fees = fees.saturating_add(fee);
                    bytes += raw.len();
                    next_nonce.insert(account, expected + 1);
                    chosen.push((raw.clone(), tx.clone()));
                    progress = true;
                }
                Err(_) => {}
            }
            done[i] = true;
        }
    }
    let undo = state.commit();
    state.revert(&undo);

    let coinbase = Tx {
        body: Body::Coinbase { height, to, amount: rules.subsidy.saturating_add(fees), extra: [0; 32] },
        sig: Vec::new(),
    };
    let mut raws: Vec<Vec<u8>> = vec![coinbase.encode()];
    raws.extend(chosen.iter().map(|(raw, _)| raw.clone()));
    let txs: Vec<Decoded> = raws
        .iter()
        .map(|raw| Decoded { raw: raw.clone(), tx: Tx::decode(raw).expect("re-encoded"), txid: txid(&rules.chain, raw) })
        .collect();
    let (state_root, undo) = apply::transition(&rules, height, &txs, chain.state_mut(), &locked).map_err(ChainError::Block)?;
    chain.state_mut().revert(&undo);

    let bits = super::daa::next_bits(chain.genesis_anchor(), &parent);
    let header = Header {
        version: 1,
        height,
        time,
        bits,
        nonce: [0; 16],
        prev: parent.id(),
        tx_root: tx_root(&raws),
        state_root,
        chain: rules.chain,
    };
    Ok(Template { header, target: super::header::decode_bits(bits).expect("ASERT output is canonical"), body: tx::join_body(&raws), txids: txs.iter().map(|d| d.txid).collect(), fees })
}
