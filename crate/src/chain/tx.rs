//! Transactions: the envelope and the four kinds of rule set 1 (§2.1–§2.4),
//! fixed-size, one valid byte form per value.

use super::params::MAX_TX_BYTES;

pub const TX_VERSION: u8 = 1;
/// NSIG1 envelope: `"NSIG1" ‖ pubkey[33] ‖ ecdsa[64]` (§2.7).
pub const SIG_BYTES: usize = 102;

pub const KIND_COINBASE: u8 = 0;
pub const KIND_TRANSFER: u8 = 1;
pub const KIND_CYBERLINK: u8 = 2;
pub const KIND_CLAIM: u8 = 3;

/// Total encoded size per kind, including envelope and signature (§2.1).
pub const COINBASE_BYTES: usize = 82;
pub const TRANSFER_BYTES: usize = 224;
pub const CYBERLINK_BYTES: usize = 257;
pub const CLAIM_BYTES: usize = 140;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Coinbase {
    pub height: u64,
    pub to: [u8; 32],
    pub amount: u64,
    pub extra: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transfer {
    pub from: [u8; 32],
    pub nonce: u64,
    pub token: [u8; 32],
    pub amount: u64,
    pub to: [u8; 32],
    pub fee: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CyberlinkTx {
    pub from: [u8; 32],
    pub nonce: u64,
    pub token: [u8; 32],
    pub amount: u64,
    pub p: [u8; 32],
    pub q: [u8; 32],
    pub valence: i8,
    pub fee: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Claim {
    pub account_id: [u8; 20],
    pub nonce: u64,
    pub fee: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TxBody {
    Coinbase(Coinbase),
    Transfer(Transfer),
    Cyberlink(CyberlinkTx),
    Claim(Claim),
}

/// A decoded transaction: the body plus the signature bytes (empty for a
/// coinbase). Kinds 1–3 carry exactly `SIG_BYTES`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tx {
    pub body: TxBody,
    pub sig: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    Empty,
    Version(u8),
    Kind(u8),
    Length { kind: u8, expected: usize, actual: usize },
    Valence(u8),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::Empty => write!(f, "empty transaction"),
            DecodeError::Version(v) => write!(f, "transaction version {v}"),
            DecodeError::Kind(k) => write!(f, "transaction kind {k} is not in rule set 1"),
            DecodeError::Length { kind, expected, actual } => {
                write!(f, "kind {kind} is {expected} bytes, got {actual}")
            }
            DecodeError::Valence(v) => write!(f, "valence byte {v:#04x} is not -1, 0 or +1"),
        }
    }
}

/// Encoded size of a kind, or `None` outside rule set 1.
pub fn size_of_kind(kind: u8) -> Option<usize> {
    match kind {
        KIND_COINBASE => Some(COINBASE_BYTES),
        KIND_TRANSFER => Some(TRANSFER_BYTES),
        KIND_CYBERLINK => Some(CYBERLINK_BYTES),
        KIND_CLAIM => Some(CLAIM_BYTES),
        _ => None,
    }
}

/// The length a decoder expects from the first two bytes, without decoding
/// the rest: how a body is split into self-delimiting transactions (§1.6).
pub fn peek_len(bytes: &[u8]) -> Result<usize, DecodeError> {
    if bytes.len() < 2 {
        return Err(DecodeError::Empty);
    }
    if bytes[0] != TX_VERSION {
        return Err(DecodeError::Version(bytes[0]));
    }
    size_of_kind(bytes[1]).ok_or(DecodeError::Kind(bytes[1]))
}

impl Tx {
    pub fn kind(&self) -> u8 {
        match self.body {
            TxBody::Coinbase(_) => KIND_COINBASE,
            TxBody::Transfer(_) => KIND_TRANSFER,
            TxBody::Cyberlink(_) => KIND_CYBERLINK,
            TxBody::Claim(_) => KIND_CLAIM,
        }
    }

    pub fn is_coinbase(&self) -> bool {
        matches!(self.body, TxBody::Coinbase(_))
    }

    /// `version ‖ kind ‖ body` — the bytes under the signature statement.
    pub fn encode_unsigned(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(MAX_TX_BYTES);
        out.push(TX_VERSION);
        out.push(self.kind());
        match &self.body {
            TxBody::Coinbase(c) => {
                out.extend_from_slice(&c.height.to_le_bytes());
                out.extend_from_slice(&c.to);
                out.extend_from_slice(&c.amount.to_le_bytes());
                out.extend_from_slice(&c.extra);
            }
            TxBody::Transfer(t) => {
                out.extend_from_slice(&t.from);
                out.extend_from_slice(&t.nonce.to_le_bytes());
                out.extend_from_slice(&t.token);
                out.extend_from_slice(&t.amount.to_le_bytes());
                out.extend_from_slice(&t.to);
                out.extend_from_slice(&t.fee.to_le_bytes());
            }
            TxBody::Cyberlink(l) => {
                out.extend_from_slice(&l.from);
                out.extend_from_slice(&l.nonce.to_le_bytes());
                out.extend_from_slice(&l.token);
                out.extend_from_slice(&l.amount.to_le_bytes());
                out.extend_from_slice(&l.p);
                out.extend_from_slice(&l.q);
                out.push(l.valence as u8);
                out.extend_from_slice(&l.fee.to_le_bytes());
            }
            TxBody::Claim(c) => {
                out.extend_from_slice(&c.account_id);
                out.extend_from_slice(&c.nonce.to_le_bytes());
                out.extend_from_slice(&c.fee.to_le_bytes());
            }
        }
        out
    }

    /// Full wire bytes: unsigned part then the signature.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = self.encode_unsigned();
        out.extend_from_slice(&self.sig);
        out
    }

    /// Strict decode: exact length per kind, valence in {−1, 0, +1}.
    pub fn decode(bytes: &[u8]) -> Result<Tx, DecodeError> {
        let expected = peek_len(bytes)?;
        let kind = bytes[1];
        if bytes.len() != expected {
            return Err(DecodeError::Length { kind, expected, actual: bytes.len() });
        }
        let b = &bytes[2..];
        let u64_at = |o: usize| u64::from_le_bytes(b[o..o + 8].try_into().expect("8 bytes"));
        let arr32 = |o: usize| -> [u8; 32] { b[o..o + 32].try_into().expect("32 bytes") };
        let (body, sig_at) = match kind {
            KIND_COINBASE => (
                TxBody::Coinbase(Coinbase { height: u64_at(0), to: arr32(8), amount: u64_at(40), extra: arr32(48) }),
                None,
            ),
            KIND_TRANSFER => (
                TxBody::Transfer(Transfer {
                    from: arr32(0),
                    nonce: u64_at(32),
                    token: arr32(40),
                    amount: u64_at(72),
                    to: arr32(80),
                    fee: u64_at(112),
                }),
                Some(120),
            ),
            KIND_CYBERLINK => {
                let valence = match b[144] {
                    0x00 => 0i8,
                    0x01 => 1,
                    0xff => -1,
                    other => return Err(DecodeError::Valence(other)),
                };
                (
                    TxBody::Cyberlink(CyberlinkTx {
                        from: arr32(0),
                        nonce: u64_at(32),
                        token: arr32(40),
                        amount: u64_at(72),
                        p: arr32(80),
                        q: arr32(112),
                        valence,
                        fee: u64_at(145),
                    }),
                    Some(153),
                )
            }
            KIND_CLAIM => (
                TxBody::Claim(Claim {
                    account_id: b[0..20].try_into().expect("20 bytes"),
                    nonce: u64_at(20),
                    fee: u64_at(28),
                }),
                Some(36),
            ),
            _ => unreachable!("peek_len admits only rule-set-1 kinds"),
        };
        let sig = sig_at.map(|o| b[o..].to_vec()).unwrap_or_default();
        Ok(Tx { body, sig })
    }

    /// The signing neuron of kinds 1 and 2; `None` for a coinbase or a claim
    /// (whose subject is derived from the signature's key).
    pub fn from(&self) -> Option<[u8; 32]> {
        match &self.body {
            TxBody::Transfer(t) => Some(t.from),
            TxBody::Cyberlink(l) => Some(l.from),
            _ => None,
        }
    }

    pub fn nonce(&self) -> Option<u64> {
        match &self.body {
            TxBody::Transfer(t) => Some(t.nonce),
            TxBody::Cyberlink(l) => Some(l.nonce),
            TxBody::Claim(c) => Some(c.nonce),
            TxBody::Coinbase(_) => None,
        }
    }

    pub fn fee(&self) -> u64 {
        match &self.body {
            TxBody::Transfer(t) => t.fee,
            TxBody::Cyberlink(l) => l.fee,
            TxBody::Claim(c) => c.fee,
            TxBody::Coinbase(_) => 0,
        }
    }
}

/// Split a body into its transactions (§1.6): `count: u32 LE ‖ tx…`, each
/// self-delimiting. Returns the raw byte slices in order.
pub fn split_body(body: &[u8]) -> Result<Vec<Vec<u8>>, DecodeError> {
    if body.len() < 4 {
        return Err(DecodeError::Empty);
    }
    let count = u32::from_le_bytes(body[0..4].try_into().expect("4 bytes")) as usize;
    let mut txs = Vec::with_capacity(count.min(1024));
    let mut at = 4;
    for _ in 0..count {
        let len = peek_len(&body[at..])?;
        if at + len > body.len() {
            return Err(DecodeError::Length { kind: body[at + 1], expected: len, actual: body.len() - at });
        }
        txs.push(body[at..at + len].to_vec());
        at += len;
    }
    if at != body.len() {
        return Err(DecodeError::Length { kind: 0xff, expected: at, actual: body.len() });
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
        Tx { body: TxBody::Coinbase(Coinbase { height: 1, to: [0x55; 32], amount: 10_000, extra: [0; 32] }), sig: vec![] }
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
            (
                Tx {
                    body: TxBody::Transfer(Transfer { from: [0x66; 32], nonce: 0, token: [1; 32], amount: 5, to: [0x77; 32], fee: 224 }),
                    sig: sig.clone(),
                },
                TRANSFER_BYTES,
            ),
            (
                Tx {
                    body: TxBody::Cyberlink(CyberlinkTx {
                        from: [0x66; 32],
                        nonce: 0,
                        token: [1; 32],
                        amount: 0,
                        p: [2; 32],
                        q: [3; 32],
                        valence: -1,
                        fee: 257,
                    }),
                    sig: sig.clone(),
                },
                CYBERLINK_BYTES,
            ),
            (Tx { body: TxBody::Claim(Claim { account_id: [9; 20], nonce: 3, fee: 140 }), sig }, CLAIM_BYTES),
        ];
        for (tx, size) in txs {
            let bytes = tx.encode();
            assert_eq!(bytes.len(), size);
            assert_eq!(Tx::decode(&bytes).unwrap(), tx);
            let mut trailing = bytes.clone();
            trailing.push(0);
            assert!(matches!(Tx::decode(&trailing), Err(DecodeError::Length { .. })));
            assert!(matches!(Tx::decode(&bytes[..bytes.len() - 1]), Err(DecodeError::Length { .. })));
            let mut v2 = bytes.clone();
            v2[0] = 2;
            assert_eq!(Tx::decode(&v2), Err(DecodeError::Version(2)));
            let mut k4 = bytes.clone();
            k4[1] = 4;
            assert_eq!(Tx::decode(&k4), Err(DecodeError::Kind(4)));
        }
    }

    #[test]
    fn valence_two_is_rejected() {
        let mut bytes = Tx {
            body: TxBody::Cyberlink(CyberlinkTx {
                from: [0; 32],
                nonce: 0,
                token: [0; 32],
                amount: 0,
                p: [1; 32],
                q: [2; 32],
                valence: 1,
                fee: 0,
            }),
            sig: vec![0; SIG_BYTES],
        }
        .encode();
        bytes[2 + 144] = 2;
        assert_eq!(Tx::decode(&bytes), Err(DecodeError::Valence(2)));
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
