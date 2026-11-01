//! Transaction encodings of the phase-1 root chain, client side.
//!
//! One valid byte form per value (chain.md §2): every kind is a fixed-size
//! record, integers little-endian, no varints, no optional fields. The
//! kernel's `chain::tx` carries the same rules on the node; this module is
//! the wallet's mirror of them and is reconciled against the kernel's
//! encodings when they land (`feat/chain-kernel`). Until then the spec is
//! the contract and chain-conformance §6 the vectors (`tests.rs`).

use super::Error;

/// `tx.version` under rule set 1.
pub const VERSION: u8 = 1;
/// Fee floor: native units per transaction byte (chain.md §0).
pub const MIN_FEE_PER_BYTE: u64 = 1;
/// Largest transaction the chain admits (chain.md §0).
pub const MAX_TX_BYTES: usize = 512;
/// The NSIG1 envelope: `"NSIG1" ‖ pubkey[33] ‖ ecdsa[64]`.
pub const SIG_BYTES: usize = 102;

const SIGN_TAG: &[u8] = b"soft3/chain/sign/v1\0";
const TXID_TAG: &[u8] = b"soft3/chain/txid/v1\0";
const LEAF_TAG: &[u8] = b"soft3/chain/leaf/v1\0";
const NODE_TAG: &[u8] = b"soft3/chain/node/v1\0";
const LEGACY_TAG: &[u8] = b"soft3/chain/legacy/v1\0";
const TOPIC_TAG: &[u8] = b"soft3/chain/topic/v1\0";
const CARD_TAG: &[u8] = b"soft3/chain/card/v1\0";
const STAKE_TAG: &[u8] = b"soft3/chain/stake/v1\0";
const FOCUS_TAG: &[u8] = b"soft3/chain/focus/v1\0";

/// Transaction kinds of rule set 1 (chain.md §2.1; kinds 6 and 7 are the
/// focus economy's, chain-economy §3.4–§3.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    Coinbase = 0,
    Transfer = 1,
    Cyberlink = 2,
    Claim = 3,
    Unlock = 6,
    Payout = 7,
}

impl Kind {
    pub fn from_byte(b: u8) -> Result<Self, Error> {
        match b {
            0 => Ok(Self::Coinbase),
            1 => Ok(Self::Transfer),
            2 => Ok(Self::Cyberlink),
            3 => Ok(Self::Claim),
            6 => Ok(Self::Unlock),
            7 => Ok(Self::Payout),
            other => Err(Error::Encoding(format!("unknown tx kind {other}"))),
        }
    }

    /// Body bytes between the two-byte envelope head and the signature.
    pub const fn body_bytes(self) -> usize {
        match self {
            Self::Coinbase => 80,
            Self::Transfer => 120,
            Self::Cyberlink => 153,
            Self::Claim => 36,
            Self::Unlock => 120,
            Self::Payout => 144,
        }
    }

    pub const fn signed(self) -> bool {
        !matches!(self, Self::Coinbase)
    }

    /// Total wire size: head ‖ body ‖ sig.
    pub const fn total_bytes(self) -> usize {
        2 + self.body_bytes() + if self.signed() { SIG_BYTES } else { 0 }
    }

    /// The fee floor for a tx of this kind (chain.md §2.6).
    pub const fn min_fee(self) -> u64 {
        self.total_bytes() as u64 * MIN_FEE_PER_BYTE
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Coinbase => "coinbase",
            Self::Transfer => "transfer",
            Self::Cyberlink => "cyberlink",
            Self::Claim => "claim",
            Self::Unlock => "unlock",
            Self::Payout => "payout",
        }
    }
}

/// A decoded transaction body (chain.md §2.2–2.4, §2.8).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tx {
    Coinbase { height: u64, to: [u8; 32], amount: u64, extra: [u8; 32] },
    Transfer { from: [u8; 32], nonce: u64, token: [u8; 32], amount: u64, to: [u8; 32], fee: u64 },
    Cyberlink {
        from: [u8; 32],
        nonce: u64,
        token: [u8; 32],
        amount: u64,
        p: [u8; 32],
        q: [u8; 32],
        valence: i8,
        fee: u64,
    },
    Claim { account_id: [u8; 20], nonce: u64, fee: u64 },
    /// kind 6: return `amount` of the stake of `card(from, p, q)` to `from`
    /// once `MIN_LOCK` has passed since its last increase (chain-economy §3.4)
    Unlock { from: [u8; 32], nonce: u64, p: [u8; 32], q: [u8; 32], amount: u64, fee: u64 },
    /// kind 7: point the rewards of `card(from, p, q)` at `payout`
    /// (chain-economy §3.5)
    Payout { from: [u8; 32], nonce: u64, p: [u8; 32], q: [u8; 32], payout: [u8; 32], fee: u64 },
}

impl Tx {
    pub fn kind(&self) -> Kind {
        match self {
            Tx::Coinbase { .. } => Kind::Coinbase,
            Tx::Transfer { .. } => Kind::Transfer,
            Tx::Cyberlink { .. } => Kind::Cyberlink,
            Tx::Claim { .. } => Kind::Claim,
            Tx::Unlock { .. } => Kind::Unlock,
            Tx::Payout { .. } => Kind::Payout,
        }
    }

    /// The nonce a signed kind carries; none for the coinbase.
    pub fn nonce(&self) -> Option<u64> {
        match self {
            Tx::Coinbase { .. } => None,
            Tx::Transfer { nonce, .. }
            | Tx::Cyberlink { nonce, .. }
            | Tx::Claim { nonce, .. }
            | Tx::Unlock { nonce, .. }
            | Tx::Payout { nonce, .. } => Some(*nonce),
        }
    }

    /// The fee a signed kind pays; none for the coinbase.
    pub fn fee(&self) -> Option<u64> {
        match self {
            Tx::Coinbase { .. } => None,
            Tx::Transfer { fee, .. }
            | Tx::Cyberlink { fee, .. }
            | Tx::Claim { fee, .. }
            | Tx::Unlock { fee, .. }
            | Tx::Payout { fee, .. } => Some(*fee),
        }
    }

    /// The neuron that signs: `from` of kinds 1, 2, 6 and 7; none for a
    /// coinbase or a claim (whose subject is the signing key's neuron).
    pub fn from(&self) -> Option<[u8; 32]> {
        match self {
            Tx::Transfer { from, .. } | Tx::Cyberlink { from, .. } | Tx::Unlock { from, .. } | Tx::Payout { from, .. } => {
                Some(*from)
            }
            Tx::Coinbase { .. } | Tx::Claim { .. } => None,
        }
    }

    /// Kind rules a wallet can check before spending a fee on a rejection.
    pub fn check_rules(&self) -> Result<(), Error> {
        let kind = self.kind();
        if let Some(fee) = self.fee() {
            if fee < kind.min_fee() {
                return Err(Error::Rule(format!(
                    "fee {fee} below the floor {} for a {}",
                    kind.min_fee(),
                    kind.name()
                )));
            }
        }
        if let Tx::Cyberlink { p, q, valence, .. } = self {
            if p == q {
                return Err(Error::Rule("cyberlink requires p != q".into()));
            }
            if !(-1..=1).contains(valence) {
                return Err(Error::Rule(format!("valence {valence} outside -1..=1")));
            }
        }
        if let Tx::Unlock { amount: 0, .. } = self {
            return Err(Error::Rule("unlock of 0 does nothing".into()));
        }
        Ok(())
    }

    /// `version ‖ kind ‖ body` — the bytes the statement covers.
    pub fn encode_unsigned(&self) -> Vec<u8> {
        let kind = self.kind();
        let mut out = Vec::with_capacity(kind.total_bytes());
        out.push(VERSION);
        out.push(kind as u8);
        match self {
            Tx::Coinbase { height, to, amount, extra } => {
                out.extend_from_slice(&height.to_le_bytes());
                out.extend_from_slice(to);
                out.extend_from_slice(&amount.to_le_bytes());
                out.extend_from_slice(extra);
            }
            Tx::Transfer { from, nonce, token, amount, to, fee } => {
                out.extend_from_slice(from);
                out.extend_from_slice(&nonce.to_le_bytes());
                out.extend_from_slice(token);
                out.extend_from_slice(&amount.to_le_bytes());
                out.extend_from_slice(to);
                out.extend_from_slice(&fee.to_le_bytes());
            }
            Tx::Cyberlink { from, nonce, token, amount, p, q, valence, fee } => {
                out.extend_from_slice(from);
                out.extend_from_slice(&nonce.to_le_bytes());
                out.extend_from_slice(token);
                out.extend_from_slice(&amount.to_le_bytes());
                out.extend_from_slice(p);
                out.extend_from_slice(q);
                out.push(*valence as u8);
                out.extend_from_slice(&fee.to_le_bytes());
            }
            Tx::Claim { account_id, nonce, fee } => {
                out.extend_from_slice(account_id);
                out.extend_from_slice(&nonce.to_le_bytes());
                out.extend_from_slice(&fee.to_le_bytes());
            }
            Tx::Unlock { from, nonce, p, q, amount, fee } => {
                out.extend_from_slice(from);
                out.extend_from_slice(&nonce.to_le_bytes());
                out.extend_from_slice(p);
                out.extend_from_slice(q);
                out.extend_from_slice(&amount.to_le_bytes());
                out.extend_from_slice(&fee.to_le_bytes());
            }
            Tx::Payout { from, nonce, p, q, payout, fee } => {
                out.extend_from_slice(from);
                out.extend_from_slice(&nonce.to_le_bytes());
                out.extend_from_slice(p);
                out.extend_from_slice(q);
                out.extend_from_slice(payout);
                out.extend_from_slice(&fee.to_le_bytes());
            }
        }
        debug_assert_eq!(out.len(), 2 + kind.body_bytes());
        out
    }

    /// Decode a complete wire transaction. Returns the body and, for signed
    /// kinds, the 102-byte NSIG1 envelope. Trailing or missing bytes, an
    /// unknown version or kind, and a non-canonical valence are errors.
    pub fn decode(bytes: &[u8]) -> Result<(Tx, Option<[u8; SIG_BYTES]>), Error> {
        if bytes.len() < 2 {
            return Err(Error::Encoding("tx shorter than its envelope head".into()));
        }
        if bytes[0] != VERSION {
            return Err(Error::Encoding(format!("tx version {} is not {VERSION}", bytes[0])));
        }
        let kind = Kind::from_byte(bytes[1])?;
        if bytes.len() != kind.total_bytes() {
            return Err(Error::Encoding(format!(
                "{} tx is {} bytes, got {}",
                kind.name(),
                kind.total_bytes(),
                bytes.len()
            )));
        }
        let mut r = Reader { bytes, at: 2 };
        let tx = match kind {
            Kind::Coinbase => Tx::Coinbase {
                height: r.u64(),
                to: r.arr32(),
                amount: r.u64(),
                extra: r.arr32(),
            },
            Kind::Transfer => Tx::Transfer {
                from: r.arr32(),
                nonce: r.u64(),
                token: r.arr32(),
                amount: r.u64(),
                to: r.arr32(),
                fee: r.u64(),
            },
            Kind::Cyberlink => {
                let (from, nonce, token, amount, p, q) =
                    (r.arr32(), r.u64(), r.arr32(), r.u64(), r.arr32(), r.arr32());
                let valence = r.u8() as i8;
                if !(-1..=1).contains(&valence) {
                    return Err(Error::Encoding(format!("valence byte {valence} is not -1, 0 or 1")));
                }
                Tx::Cyberlink { from, nonce, token, amount, p, q, valence, fee: r.u64() }
            }
            Kind::Claim => Tx::Claim { account_id: r.arr20(), nonce: r.u64(), fee: r.u64() },
            Kind::Unlock => Tx::Unlock { from: r.arr32(), nonce: r.u64(), p: r.arr32(), q: r.arr32(), amount: r.u64(), fee: r.u64() },
            Kind::Payout => Tx::Payout { from: r.arr32(), nonce: r.u64(), p: r.arr32(), q: r.arr32(), payout: r.arr32(), fee: r.u64() },
        };
        let sig = if kind.signed() {
            let mut sig = [0u8; SIG_BYTES];
            sig.copy_from_slice(&bytes[r.at..r.at + SIG_BYTES]);
            Some(sig)
        } else {
            None
        };
        Ok((tx, sig))
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn u8(&mut self) -> u8 {
        let v = self.bytes[self.at];
        self.at += 1;
        v
    }
    fn u64(&mut self) -> u64 {
        let mut b = [0u8; 8];
        b.copy_from_slice(&self.bytes[self.at..self.at + 8]);
        self.at += 8;
        u64::from_le_bytes(b)
    }
    fn arr32(&mut self) -> [u8; 32] {
        let mut b = [0u8; 32];
        b.copy_from_slice(&self.bytes[self.at..self.at + 32]);
        self.at += 32;
        b
    }
    fn arr20(&mut self) -> [u8; 20] {
        let mut b = [0u8; 20];
        b.copy_from_slice(&self.bytes[self.at..self.at + 20]);
        self.at += 20;
        b
    }
}

fn h(parts: &[&[u8]]) -> [u8; 32] {
    let mut input = Vec::new();
    for p in parts {
        input.extend_from_slice(p);
    }
    *hemera::hash(&input).as_bytes()
}

/// `statement = hemera("soft3/chain/sign/v1\0" ‖ chain ‖ tx_bytes_without_sig)`.
pub fn statement(chain: &[u8; 32], unsigned: &[u8]) -> [u8; 32] {
    #[cfg(feature = "opcount")]
    crate::chain::opcount::hash_k(crate::chain::opcount::Purpose::SigStatement, unsigned.get(1).copied(), &[SIGN_TAG, chain, unsigned]);
    h(&[SIGN_TAG, chain, unsigned])
}

/// `txid = hemera("soft3/chain/txid/v1\0" ‖ chain ‖ tx_bytes)` over the full
/// wire bytes, signature included.
pub fn txid(chain: &[u8; 32], tx_bytes: &[u8]) -> [u8; 32] {
    #[cfg(feature = "opcount")]
    crate::chain::opcount::hash_k(crate::chain::opcount::Purpose::Txid, tx_bytes.get(1).copied(), &[TXID_TAG, chain, tx_bytes]);
    h(&[TXID_TAG, chain, tx_bytes])
}

/// Merkle leaf of a transaction (chain.md §1.7).
pub fn leaf(tx_bytes: &[u8]) -> [u8; 32] {
    #[cfg(feature = "opcount")]
    crate::chain::opcount::hash_k(crate::chain::opcount::Purpose::TxRoot, tx_bytes.get(1).copied(), &[LEAF_TAG, tx_bytes]);
    h(&[LEAF_TAG, tx_bytes])
}

/// Merkle inner node (chain.md §1.7).
pub fn node(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    #[cfg(feature = "opcount")]
    crate::chain::opcount::hash_k(crate::chain::opcount::Purpose::TxRoot, Some(crate::chain::opcount::BLOCK), &[NODE_TAG, left, right]);
    h(&[NODE_TAG, left, right])
}

/// `tx_root` over transactions in body order: odd nodes are promoted, never
/// duplicated. Empty input has no root.
pub fn tx_root(txs: &[Vec<u8>]) -> Option<[u8; 32]> {
    let mut level: Vec<[u8; 32]> = txs.iter().map(|t| leaf(t)).collect();
    if level.is_empty() {
        return None;
    }
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len() / 2 + 1);
        for pair in level.chunks(2) {
            match pair {
                [l, r] => next.push(node(l, r)),
                [last] => next.push(*last),
                _ => unreachable!(),
            }
        }
        level = next;
    }
    Some(level[0])
}

/// `token(denom) = hemera(denom_bytes)` (chain.md §3.1).
pub fn token(denom: &str) -> [u8; 32] {
    *hemera::hash(denom.as_bytes()).as_bytes()
}

/// `neuron_of(pubkey) = hemera(compressed pubkey)` — mudra's native id.
pub fn neuron_of(pubkey: &[u8; 33]) -> [u8; 32] {
    #[cfg(feature = "opcount")]
    crate::chain::opcount::hash(crate::chain::opcount::Purpose::Key, &[pubkey]);
    mudra::claim::neuron_of(pubkey)
}

/// The dormant owner genesis parks an address-only account under
/// (chain.md §2.8): `hemera("soft3/chain/legacy/v1\0" ‖ hrp ‖ account_id)`.
pub fn legacy_owner(hrp: &str, account_id: &[u8; 20]) -> [u8; 32] {
    #[cfg(feature = "opcount")]
    crate::chain::opcount::hash(crate::chain::opcount::Purpose::Key, &[LEGACY_TAG, hrp.as_bytes(), account_id]);
    h(&[LEGACY_TAG, hrp.as_bytes(), account_id])
}

/// Gossip topic id (chain-wire.md §2).
pub fn topic(chain: &[u8; 32], name: &str) -> [u8; 32] {
    h(&[TOPIC_TAG, chain, name.as_bytes()])
}

/// `axon(p, q) = hemera(p ‖ q)` — the particle a cyberlink's box binds to.
pub fn axon(p: &[u8; 32], q: &[u8; 32]) -> [u8; 32] {
    #[cfg(feature = "opcount")]
    crate::chain::opcount::hash(crate::chain::opcount::Purpose::Key, &[p, q]);
    h(&[p, q])
}

/// `card(author, p, q) = hemera("soft3/chain/card/v1\0" ‖ author ‖ p ‖ q)` —
/// the position id of a staked cyberlink (chain-economy §3.1).
pub fn card_id(author: &[u8; 32], p: &[u8; 32], q: &[u8; 32]) -> [u8; 32] {
    #[cfg(feature = "opcount")]
    crate::chain::opcount::hash(crate::chain::opcount::Purpose::Key, &[CARD_TAG, author, p, q]);
    h(&[CARD_TAG, author, p, q])
}

/// `stake_key(card) = hemera("soft3/chain/stake/v1\0" ‖ card)`: the raw
/// balances key holding a position's stake (chain-economy §3.1).
pub fn stake_key(card: &[u8; 32]) -> [u8; 32] {
    #[cfg(feature = "opcount")]
    crate::chain::opcount::hash(crate::chain::opcount::Purpose::Key, &[STAKE_TAG, card]);
    h(&[STAKE_TAG, card])
}

/// `focus_key(token) = hemera("soft3/chain/focus/v1\0" ‖ token)`: the raw
/// balances key of the focus pool (chain-economy §3.1).
pub fn focus_key(token: &[u8; 32]) -> [u8; 32] {
    #[cfg(feature = "opcount")]
    crate::chain::opcount::hash(crate::chain::opcount::Purpose::Key, &[FOCUS_TAG, token]);
    h(&[FOCUS_TAG, token])
}

/// `balance_key(owner, token) = hemera(owner ‖ token)`.
pub fn balance_key(owner: &[u8; 32], token: &[u8; 32]) -> [u8; 32] {
    #[cfg(feature = "opcount")]
    crate::chain::opcount::hash(crate::chain::opcount::Purpose::Key, &[owner, token]);
    h(&[owner, token])
}
