//! Transactions on the node side (§2): the one codec is the wallet's
//! (`crate::wallet::tx`, pinned to chain-conformance §6); the kernel adds
//! the signed envelope [`Tx`] and the body framing of §1.6.

pub use crate::wallet::tx::{Kind, MAX_TX_BYTES, MIN_FEE_PER_BYTE, SIG_BYTES, Tx as Body, VERSION as TX_VERSION};

/// Total encoded size per kind, including envelope and signature (§2.1).
pub const COINBASE_BYTES: usize = Kind::Coinbase.total_bytes();
pub const TRANSFER_BYTES: usize = Kind::Transfer.total_bytes();
pub const CYBERLINK_BYTES: usize = Kind::Cyberlink.total_bytes();
pub const CLAIM_BYTES: usize = Kind::Claim.total_bytes();

/// Why bytes are not a rule-set-1 transaction (the wallet codec's reason).
pub type DecodeError = String;

/// A decoded transaction: the body plus the signature bytes (empty for a
/// coinbase, exactly `SIG_BYTES` for kinds 1–3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tx {
    pub body: Body,
    pub sig: Vec<u8>,
}

/// Encoded size of a kind, or `None` outside rule set 1. Every later kind
/// — the pool's deposit and the cores' kinds ≥ 4 (chain.md §7.2, §11) —
/// is rejected here until the activation table (`params::ACTIVATIONS`)
/// names the rule set that admits it.
pub fn size_of_kind(kind: u8) -> Option<usize> {
    Kind::from_byte(kind).ok().map(Kind::total_bytes)
}

/// The length a decoder expects from the first two bytes, without decoding
/// the rest: how a body is split into self-delimiting transactions (§1.6).
pub fn peek_len(bytes: &[u8]) -> Result<usize, DecodeError> {
    if bytes.len() < 2 {
        return Err("tx shorter than its envelope head".into());
    }
    if bytes[0] != TX_VERSION {
        return Err(format!("tx version {} is not {TX_VERSION}", bytes[0]));
    }
    size_of_kind(bytes[1]).ok_or_else(|| format!("unknown tx kind {}", bytes[1]))
}

impl Tx {
    pub fn kind(&self) -> Kind {
        self.body.kind()
    }

    pub fn is_coinbase(&self) -> bool {
        matches!(self.body, Body::Coinbase { .. })
    }

    /// `version ‖ kind ‖ body` — the bytes under the signature statement.
    pub fn encode_unsigned(&self) -> Vec<u8> {
        self.body.encode_unsigned()
    }

    /// Full wire bytes: unsigned part then the signature.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = self.encode_unsigned();
        out.extend_from_slice(&self.sig);
        out
    }

    /// Strict decode: exact length per kind, valence in {−1, 0, +1}.
    pub fn decode(bytes: &[u8]) -> Result<Tx, DecodeError> {
        let (body, sig) = Body::decode(bytes).map_err(|e| e.to_string())?;
        Ok(Tx { body, sig: sig.map(|s| s.to_vec()).unwrap_or_default() })
    }

    /// The signing neuron of kinds 1 and 2; `None` for a coinbase or a claim
    /// (whose subject is derived from the signature's key).
    pub fn from(&self) -> Option<[u8; 32]> {
        match &self.body {
            Body::Transfer { from, .. } | Body::Cyberlink { from, .. } => Some(*from),
            _ => None,
        }
    }

    pub fn nonce(&self) -> Option<u64> {
        self.body.nonce()
    }

    /// The fee paid; 0 for a coinbase.
    pub fn fee(&self) -> u64 {
        self.body.fee().unwrap_or(0)
    }
}

/// Split a body into its transactions (§1.6): `count: u32 LE ‖ tx…`, each
/// self-delimiting. Returns the raw byte slices in order.
pub fn split_body(body: &[u8]) -> Result<Vec<Vec<u8>>, DecodeError> {
    if body.len() < 4 {
        return Err("body shorter than its count".into());
    }
    let count = u32::from_le_bytes(body[0..4].try_into().expect("4 bytes")) as usize;
    let mut txs = Vec::with_capacity(count.min(1024));
    let mut at = 4;
    for i in 0..count {
        let len = peek_len(&body[at..])?;
        if at + len > body.len() {
            return Err(format!("tx {i} truncated: needs {len} bytes, {} left", body.len() - at));
        }
        txs.push(body[at..at + len].to_vec());
        at += len;
    }
    if at != body.len() {
        return Err(format!("{} trailing bytes after {count} transactions", body.len() - at));
    }
    Ok(txs)
}

/// Assemble a body from encoded transactions.
pub fn join_body(txs: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + txs.iter().map(Vec::len).sum::<usize>());
    out.extend_from_slice(&(txs.len() as u32).to_le_bytes());
    for t in txs {
        out.extend_from_slice(t);
    }
    out
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    pub(crate) fn coinbase() -> Tx {
        Tx { body: Body::Coinbase { height: 1, to: [0x55; 32], amount: 10_000, extra: [0; 32] }, sig: vec![] }
    }

    #[test]
    fn conformance_section_6_coinbase_bytes() {
        let bytes = coinbase().encode();
        assert_eq!(bytes.len(), COINBASE_BYTES);
        assert_eq!(hex(&bytes[..10]), "01000100000000000000");
        assert_eq!(hex(&bytes[42..50]), "1027000000000000");
        assert_eq!(Tx::decode(&bytes).unwrap(), coinbase());
    }

    #[test]
    fn sizes_and_rejections_every_kind() {
        let sig = vec![0u8; SIG_BYTES];
        let txs = [
            (coinbase(), COINBASE_BYTES),
            (Tx { body: Body::Transfer { from: [0x66; 32], nonce: 0, token: [1; 32], amount: 5, to: [0x77; 32], fee: 224 }, sig: sig.clone() }, TRANSFER_BYTES),
            (
                Tx {
                    body: Body::Cyberlink { from: [0x66; 32], nonce: 0, token: [1; 32], amount: 0, p: [2; 32], q: [3; 32], valence: -1, fee: 257 },
                    sig: sig.clone(),
                },
                CYBERLINK_BYTES,
            ),
            (Tx { body: Body::Claim { account_id: [9; 20], nonce: 3, fee: 140 }, sig }, CLAIM_BYTES),
        ];
        assert_eq!([COINBASE_BYTES, TRANSFER_BYTES, CYBERLINK_BYTES, CLAIM_BYTES], [82, 224, 257, 140]);
        for (tx, size) in txs {
            let bytes = tx.encode();
            assert_eq!(bytes.len(), size);
            assert_eq!(Tx::decode(&bytes).unwrap(), tx);
            let mut trailing = bytes.clone();
            trailing.push(0);
            assert!(Tx::decode(&trailing).is_err());
            assert!(Tx::decode(&bytes[..bytes.len() - 1]).is_err());
            let mut v2 = bytes.clone();
            v2[0] = 2;
            assert!(Tx::decode(&v2).unwrap_err().contains("version 2"));
            let mut k4 = bytes.clone();
            k4[1] = 4;
            assert!(Tx::decode(&k4).unwrap_err().contains("kind 4"));
        }
    }

    #[test]
    fn valence_two_is_rejected() {
        let mut bytes = Tx {
            body: Body::Cyberlink { from: [0; 32], nonce: 0, token: [0; 32], amount: 0, p: [1; 32], q: [2; 32], valence: 1, fee: 0 },
            sig: vec![0; SIG_BYTES],
        }
        .encode();
        bytes[2 + 144] = 2;
        assert!(Tx::decode(&bytes).unwrap_err().contains("valence"));
    }

    #[test]
    fn body_split_round_trip() {
        let cb = coinbase().encode();
        let body = join_body(&[cb.clone(), cb.clone()]);
        assert_eq!(split_body(&body).unwrap(), vec![cb.clone(), cb.clone()]);
        let mut short = body.clone();
        short.pop();
        assert!(split_body(&short).is_err());
        let mut long = body;
        long.push(0);
        assert!(split_body(&long).is_err());
        assert!(split_body(&[]).is_err());
    }
}
