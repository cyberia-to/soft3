//! Fixed-size codecs the wire needs before the kernel lands: the 168-byte
//! header, the four transaction kinds, block ids, txids, the compact target,
//! work and the transaction root — chain.md §1–2, pinned by the vectors of
//! chain-conformance.md. Every value has one byte form; a decoder that
//! accepts two forms for one value is a consensus bug.

use std::cmp::Ordering;

pub const HEADER_BYTES: usize = 168;
pub const MAX_BLOCK_BYTES: usize = 262_144;
pub const MAX_BLOCK_TXS: usize = 1024;
pub const MAX_TX_BYTES: usize = 512;
pub const MIN_FEE_PER_BYTE: u64 = 1;
pub const NONCE_WINDOW: u64 = 16;
pub const COINBASE_MATURITY: u64 = 100;
pub const MAX_FUTURE_DRIFT: u64 = 120;
pub const MTP_WINDOW: usize = 11;
pub const FINALITY_DEPTH: u64 = 120;
pub const MAX_REORG: u64 = FINALITY_DEPTH - 1;
pub const POW_LIMIT_BITS: u32 = 0x1e10_0000;
pub const GENESIS_BITS: u32 = 0x1e01_0000;
pub const RULE_SET: u16 = 1;
pub const SIG_BYTES: usize = 102;

pub type Id = [u8; 32];

pub fn hash(bytes: &[u8]) -> Id {
    *hemera::hash(bytes).as_bytes()
}

/// `hemera(tag ‖ parts…)` — every domain-separated hash of the chain.
pub fn hash_tagged(tag: &[u8], parts: &[&[u8]]) -> Id {
    let mut input = tag.to_vec();
    for part in parts {
        input.extend_from_slice(part);
    }
    hash(&input)
}

pub fn token(denom: &str) -> Id {
    hash(denom.as_bytes())
}

pub fn topic_id(chain: &Id, name: &str) -> Id {
    hash_tagged(b"soft3/chain/topic/v1\0", &[chain, name.as_bytes()])
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn unhex32(s: &str) -> Option<Id> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(out)
}

// ── u256 ────────────────────────────────────────────────────────────────────

/// 256-bit unsigned integer, little-endian limbs; integer only, no floats.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct U256(pub [u64; 4]);

impl U256 {
    pub const ZERO: U256 = U256([0; 4]);
    pub const ONE: U256 = U256([1, 0, 0, 0]);
    pub const MAX: U256 = U256([u64::MAX; 4]);

    pub fn from_le_bytes(bytes: &[u8; 32]) -> Self {
        let mut limbs = [0u64; 4];
        for (i, limb) in limbs.iter_mut().enumerate() {
            let mut b = [0u8; 8];
            b.copy_from_slice(&bytes[i * 8..i * 8 + 8]);
            *limb = u64::from_le_bytes(b);
        }
        U256(limbs)
    }

    pub fn to_le_bytes(self) -> [u8; 32] {
        let mut out = [0u8; 32];
        for (i, limb) in self.0.iter().enumerate() {
            out[i * 8..i * 8 + 8].copy_from_slice(&limb.to_le_bytes());
        }
        out
    }

    pub fn from_u64(v: u64) -> Self {
        U256([v, 0, 0, 0])
    }

    pub fn pow2(exp: u32) -> Self {
        let mut out = [0u64; 4];
        if exp < 256 {
            out[(exp / 64) as usize] = 1u64 << (exp % 64);
        }
        U256(out)
    }

    pub fn is_zero(&self) -> bool {
        self.0 == [0; 4]
    }

    pub fn bits(&self) -> u32 {
        for i in (0..4).rev() {
            if self.0[i] != 0 {
                return i as u32 * 64 + (64 - self.0[i].leading_zeros());
            }
        }
        0
    }

    pub fn saturating_add(self, other: U256) -> U256 {
        let mut out = [0u64; 4];
        let mut carry = 0u64;
        for i in 0..4 {
            let (s, c1) = self.0[i].overflowing_add(other.0[i]);
            let (s, c2) = s.overflowing_add(carry);
            out[i] = s;
            carry = (c1 as u64) + (c2 as u64);
        }
        if carry != 0 { U256::MAX } else { U256(out) }
    }

    pub fn checked_sub(self, other: U256) -> Option<U256> {
        if self < other {
            return None;
        }
        let mut out = [0u64; 4];
        let mut borrow = 0u64;
        for i in 0..4 {
            let (d, b1) = self.0[i].overflowing_sub(other.0[i]);
            let (d, b2) = d.overflowing_sub(borrow);
            out[i] = d;
            borrow = (b1 as u64) + (b2 as u64);
        }
        Some(U256(out))
    }

    pub fn shl(self, n: u32) -> U256 {
        if n >= 256 {
            return U256::ZERO;
        }
        let limb = (n / 64) as usize;
        let bit = n % 64;
        let mut out = [0u64; 4];
        for i in (limb..4).rev() {
            let mut v = self.0[i - limb] << bit;
            if bit > 0 && i > limb {
                v |= self.0[i - limb - 1] >> (64 - bit);
            }
            out[i] = v;
        }
        U256(out)
    }

    pub fn shr(self, n: u32) -> U256 {
        if n >= 256 {
            return U256::ZERO;
        }
        let limb = (n / 64) as usize;
        let bit = n % 64;
        let mut out = [0u64; 4];
        for i in 0..(4 - limb) {
            let mut v = self.0[i + limb] >> bit;
            if bit > 0 && i + limb + 1 < 4 {
                v |= self.0[i + limb + 1] << (64 - bit);
            }
            out[i] = v;
        }
        U256(out)
    }

    fn bit(&self, i: u32) -> bool {
        (self.0[(i / 64) as usize] >> (i % 64)) & 1 == 1
    }

    /// floor(self / d), binary long division; `d` must be non-zero.
    pub fn div(self, d: U256) -> U256 {
        let mut q = U256::ZERO;
        let mut r = U256::ZERO;
        for i in (0..256).rev() {
            r = r.shl(1);
            if self.bit(i) {
                r.0[0] |= 1;
            }
            if r >= d {
                r = r.checked_sub(d).expect("r >= d");
                q.0[(i / 64) as usize] |= 1u64 << (i % 64);
            }
        }
        q
    }

    pub fn to_decimal(self) -> String {
        if self.is_zero() {
            return "0".into();
        }
        let mut digits = Vec::new();
        let mut v = self;
        let ten = U256::from_u64(10);
        while !v.is_zero() {
            let q = v.div(ten);
            let rem = v.checked_sub(q.mul_small(10)).expect("rem");
            digits.push(b'0' + rem.0[0] as u8);
            v = q;
        }
        digits.reverse();
        String::from_utf8(digits).expect("ascii")
    }

    fn mul_small(self, m: u64) -> U256 {
        let mut out = [0u64; 4];
        let mut carry = 0u128;
        for i in 0..4 {
            let v = self.0[i] as u128 * m as u128 + carry;
            out[i] = v as u64;
            carry = v >> 64;
        }
        U256(out)
    }
}

impl PartialOrd for U256 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for U256 {
    fn cmp(&self, other: &Self) -> Ordering {
        for i in (0..4).rev() {
            match self.0[i].cmp(&other.0[i]) {
                Ordering::Equal => continue,
                o => return o,
            }
        }
        Ordering::Equal
    }
}

// ── compact target ──────────────────────────────────────────────────────────

/// Decode `bits` (chain.md §1.3); `None` for a sign bit, a non-canonical
/// encoding or a zero target. The caller checks the POW limit.
pub fn decode_bits(bits: u32) -> Option<U256> {
    if bits & 0x0080_0000 != 0 {
        return None;
    }
    let exp = bits >> 24;
    let mant = U256::from_u64((bits & 0x007f_ffff) as u64);
    let target = if exp >= 3 {
        if exp > 35 {
            return None;
        }
        mant.shl(8 * (exp - 3))
    } else {
        mant.shr(8 * (3 - exp))
    };
    if target.is_zero() || encode_bits(target) != bits {
        return None;
    }
    Some(target)
}

pub fn encode_bits(target: U256) -> u32 {
    let mut size = target.bits().div_ceil(8);
    let mut mant = if size <= 3 {
        target.shl(8 * (3 - size)).0[0]
    } else {
        target.shr(8 * (size - 3)).0[0]
    };
    if mant & 0x0080_0000 != 0 {
        mant >>= 8;
        size += 1;
    }
    (size << 24) | (mant as u32 & 0x007f_ffff)
}

/// `floor(2^256 / (target + 1))`.
pub fn work(target: U256) -> U256 {
    let d = target.saturating_add(U256::ONE);
    if d == U256::MAX && target == U256::MAX {
        return U256::ONE;
    }
    let q = U256::MAX.div(d);
    // 2^256 = (2^256 - 1) + 1: add one when d divides 2^256 exactly (d a power of two).
    let ones: u32 = d.0.iter().map(|limb| limb.count_ones()).sum();
    if ones == 1 { q.saturating_add(U256::ONE) } else { q }
}

// ── header ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Header {
    pub version: u32,
    pub height: u64,
    pub time: u64,
    pub bits: u32,
    pub nonce: [u8; 16],
    pub prev: Id,
    pub tx_root: Id,
    pub state_root: Id,
    pub chain: Id,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    Length { expected: usize, got: usize },
    Version(u32),
    Kind(u8),
    Valence(i8),
    Count,
    Budget,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::Length { expected, got } => write!(f, "length {got}, expected {expected}"),
            DecodeError::Version(v) => write!(f, "unknown version {v}"),
            DecodeError::Kind(k) => write!(f, "unknown kind {k}"),
            DecodeError::Valence(v) => write!(f, "invalid valence {v}"),
            DecodeError::Count => write!(f, "count mismatch"),
            DecodeError::Budget => write!(f, "budget exceeded"),
        }
    }
}

impl Header {
    pub fn encode(&self) -> [u8; HEADER_BYTES] {
        let mut out = [0u8; HEADER_BYTES];
        out[0..4].copy_from_slice(&self.version.to_le_bytes());
        out[4..12].copy_from_slice(&self.height.to_le_bytes());
        out[12..20].copy_from_slice(&self.time.to_le_bytes());
        out[20..24].copy_from_slice(&self.bits.to_le_bytes());
        out[24..40].copy_from_slice(&self.nonce);
        out[40..72].copy_from_slice(&self.prev);
        out[72..104].copy_from_slice(&self.tx_root);
        out[104..136].copy_from_slice(&self.state_root);
        out[136..168].copy_from_slice(&self.chain);
        out
    }

    /// Syntax only (168 bytes); the rule set and chain are checked by the caller.
    pub fn decode(bytes: &[u8]) -> Result<Header, DecodeError> {
        if bytes.len() != HEADER_BYTES {
            return Err(DecodeError::Length { expected: HEADER_BYTES, got: bytes.len() });
        }
        Ok(Header {
            version: u32::from_le_bytes(bytes[0..4].try_into().expect("4")),
            height: u64::from_le_bytes(bytes[4..12].try_into().expect("8")),
            time: u64::from_le_bytes(bytes[12..20].try_into().expect("8")),
            bits: u32::from_le_bytes(bytes[20..24].try_into().expect("4")),
            nonce: bytes[24..40].try_into().expect("16"),
            prev: bytes[40..72].try_into().expect("32"),
            tx_root: bytes[72..104].try_into().expect("32"),
            state_root: bytes[104..136].try_into().expect("32"),
            chain: bytes[136..168].try_into().expect("32"),
        })
    }

    pub fn id(&self) -> Id {
        hash(&self.encode())
    }

    pub fn rule_set(&self) -> u16 {
        (self.version & 0xffff) as u16
    }

    pub fn target(&self) -> Option<U256> {
        decode_bits(self.bits)
    }

    /// `u256_le(block_id) < target`; genesis (height 0) is exempt.
    pub fn valid_pow(&self) -> bool {
        if self.height == 0 {
            return true;
        }
        match self.target() {
            Some(target) => U256::from_le_bytes(&self.id()) < target,
            None => false,
        }
    }

    pub fn work(&self) -> U256 {
        self.target().map(work).unwrap_or(U256::ZERO)
    }
}

// ── transactions ────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Tx {
    Coinbase { height: u64, to: Id, amount: u64, extra: Id },
    Transfer { from: Id, nonce: u64, token: Id, amount: u64, to: Id, fee: u64, sig: Vec<u8> },
    Cyberlink { from: Id, nonce: u64, token: Id, amount: u64, p: Id, q: Id, valence: i8, fee: u64, sig: Vec<u8> },
    Claim { account_id: [u8; 20], nonce: u64, fee: u64, sig: Vec<u8> },
    /// kind 6 (chain-economy §3.4)
    Unlock { from: Id, nonce: u64, p: Id, q: Id, amount: u64, fee: u64, sig: Vec<u8> },
    /// kind 7 (chain-economy §3.5)
    Payout { from: Id, nonce: u64, p: Id, q: Id, payout: Id, fee: u64, sig: Vec<u8> },
}

pub fn tx_len(kind: u8) -> Option<usize> {
    match kind {
        0 => Some(82),
        1 => Some(224),
        2 => Some(257),
        3 => Some(140),
        6 => Some(224),
        7 => Some(248),
        _ => None,
    }
}

fn u64_at(b: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(b[at..at + 8].try_into().expect("8"))
}

fn id_at(b: &[u8], at: usize) -> Id {
    b[at..at + 32].try_into().expect("32")
}

impl Tx {
    pub fn decode(bytes: &[u8]) -> Result<Tx, DecodeError> {
        if bytes.len() < 2 {
            return Err(DecodeError::Length { expected: 2, got: bytes.len() });
        }
        if bytes[0] != 1 {
            return Err(DecodeError::Version(bytes[0] as u32));
        }
        let kind = bytes[1];
        let expected = tx_len(kind).ok_or(DecodeError::Kind(kind))?;
        if bytes.len() != expected {
            return Err(DecodeError::Length { expected, got: bytes.len() });
        }
        let b = &bytes[2..];
        Ok(match kind {
            0 => Tx::Coinbase { height: u64_at(b, 0), to: id_at(b, 8), amount: u64_at(b, 40), extra: id_at(b, 48) },
            1 => Tx::Transfer {
                from: id_at(b, 0), nonce: u64_at(b, 32), token: id_at(b, 40), amount: u64_at(b, 72),
                to: id_at(b, 80), fee: u64_at(b, 112), sig: b[120..].to_vec(),
            },
            2 => {
                let valence = b[144] as i8;
                if !(-1..=1).contains(&valence) {
                    return Err(DecodeError::Valence(valence));
                }
                Tx::Cyberlink {
                    from: id_at(b, 0), nonce: u64_at(b, 32), token: id_at(b, 40), amount: u64_at(b, 72),
                    p: id_at(b, 80), q: id_at(b, 112), valence, fee: u64_at(b, 145), sig: b[153..].to_vec(),
                }
            }
            6 => Tx::Unlock {
                from: id_at(b, 0), nonce: u64_at(b, 32), p: id_at(b, 40), q: id_at(b, 72),
                amount: u64_at(b, 104), fee: u64_at(b, 112), sig: b[120..].to_vec(),
            },
            7 => Tx::Payout {
                from: id_at(b, 0), nonce: u64_at(b, 32), p: id_at(b, 40), q: id_at(b, 72),
                payout: id_at(b, 104), fee: u64_at(b, 136), sig: b[144..].to_vec(),
            },
            _ => Tx::Claim {
                account_id: b[0..20].try_into().expect("20"), nonce: u64_at(b, 20), fee: u64_at(b, 28), sig: b[36..].to_vec(),
            },
        })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = self.encode_unsigned();
        match self {
            Tx::Coinbase { .. } => {}
            Tx::Transfer { sig, .. }
            | Tx::Cyberlink { sig, .. }
            | Tx::Claim { sig, .. }
            | Tx::Unlock { sig, .. }
            | Tx::Payout { sig, .. } => out.extend_from_slice(sig),
        }
        out
    }

    /// Everything before `sig` — the bytes under the statement hash.
    pub fn encode_unsigned(&self) -> Vec<u8> {
        let mut out = vec![1u8, self.kind()];
        match self {
            Tx::Coinbase { height, to, amount, extra } => {
                out.extend_from_slice(&height.to_le_bytes());
                out.extend_from_slice(to);
                out.extend_from_slice(&amount.to_le_bytes());
                out.extend_from_slice(extra);
            }
            Tx::Transfer { from, nonce, token, amount, to, fee, .. } => {
                out.extend_from_slice(from);
                out.extend_from_slice(&nonce.to_le_bytes());
                out.extend_from_slice(token);
                out.extend_from_slice(&amount.to_le_bytes());
                out.extend_from_slice(to);
                out.extend_from_slice(&fee.to_le_bytes());
            }
            Tx::Cyberlink { from, nonce, token, amount, p, q, valence, fee, .. } => {
                out.extend_from_slice(from);
                out.extend_from_slice(&nonce.to_le_bytes());
                out.extend_from_slice(token);
                out.extend_from_slice(&amount.to_le_bytes());
                out.extend_from_slice(p);
                out.extend_from_slice(q);
                out.push(*valence as u8);
                out.extend_from_slice(&fee.to_le_bytes());
            }
            Tx::Claim { account_id, nonce, fee, .. } => {
                out.extend_from_slice(account_id);
                out.extend_from_slice(&nonce.to_le_bytes());
                out.extend_from_slice(&fee.to_le_bytes());
            }
            Tx::Unlock { from, nonce, p, q, amount, fee, .. } => {
                out.extend_from_slice(from);
                out.extend_from_slice(&nonce.to_le_bytes());
                out.extend_from_slice(p);
                out.extend_from_slice(q);
                out.extend_from_slice(&amount.to_le_bytes());
                out.extend_from_slice(&fee.to_le_bytes());
            }
            Tx::Payout { from, nonce, p, q, payout, fee, .. } => {
                out.extend_from_slice(from);
                out.extend_from_slice(&nonce.to_le_bytes());
                out.extend_from_slice(p);
                out.extend_from_slice(q);
                out.extend_from_slice(payout);
                out.extend_from_slice(&fee.to_le_bytes());
            }
        }
        out
    }

    pub fn kind(&self) -> u8 {
        match self {
            Tx::Coinbase { .. } => 0,
            Tx::Transfer { .. } => 1,
            Tx::Cyberlink { .. } => 2,
            Tx::Claim { .. } => 3,
            Tx::Unlock { .. } => 6,
            Tx::Payout { .. } => 7,
        }
    }

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

    pub fn fee(&self) -> u64 {
        match self {
            Tx::Coinbase { .. } => 0,
            Tx::Transfer { fee, .. }
            | Tx::Cyberlink { fee, .. }
            | Tx::Claim { fee, .. }
            | Tx::Unlock { fee, .. }
            | Tx::Payout { fee, .. } => *fee,
        }
    }

    pub fn sig(&self) -> &[u8] {
        match self {
            Tx::Coinbase { .. } => &[],
            Tx::Transfer { sig, .. }
            | Tx::Cyberlink { sig, .. }
            | Tx::Claim { sig, .. }
            | Tx::Unlock { sig, .. }
            | Tx::Payout { sig, .. } => sig,
        }
    }

    /// The signing subject: `from` for kinds 1, 2, 6, 7, `neuron_of(pubkey)` for a claim.
    pub fn subject(&self) -> Option<Id> {
        match self {
            Tx::Coinbase { .. } => None,
            Tx::Transfer { from, .. } | Tx::Cyberlink { from, .. } | Tx::Unlock { from, .. } | Tx::Payout { from, .. } => Some(*from),
            Tx::Claim { sig, .. } => {
                if sig.len() != SIG_BYTES || &sig[..5] != b"NSIG1" {
                    return None;
                }
                Some(mudra::claim::neuron_of(&sig[5..38]))
            }
        }
    }

    /// `verify_statement(subject, statement, sig)` with the chain bound in.
    pub fn verify(&self, chain: &Id) -> bool {
        let Some(subject) = self.subject() else { return false };
        let statement = statement(chain, &self.encode_unsigned());
        mudra::neuron::verify_statement(subject, statement, self.sig())
    }
}

pub fn txid(chain: &Id, tx_bytes: &[u8]) -> Id {
    hash_tagged(b"soft3/chain/txid/v1\0", &[chain, tx_bytes])
}

pub fn statement(chain: &Id, unsigned: &[u8]) -> Id {
    hash_tagged(b"soft3/chain/sign/v1\0", &[chain, unsigned])
}

/// Sign a transaction with a mudra key; `subject` must be the key's neuron.
pub fn sign_tx(chain: &Id, tx: &Tx, key: &mudra::SigningKey) -> Result<Tx, String> {
    let subject = match tx {
        Tx::Claim { .. } => mudra::claim::neuron_of(&mudra::cosmos::compressed(key.verifying_key())),
        _ => tx.subject().ok_or("coinbase is not signed")?,
    };
    let statement = statement(chain, &tx.encode_unsigned());
    let sig = mudra::neuron::sign(key, subject, statement).map_err(|e| e.to_string())?;
    let mut signed = tx.clone();
    match &mut signed {
        Tx::Coinbase { .. } => return Err("coinbase is not signed".into()),
        Tx::Transfer { sig: s, .. }
        | Tx::Cyberlink { sig: s, .. }
        | Tx::Claim { sig: s, .. }
        | Tx::Unlock { sig: s, .. }
        | Tx::Payout { sig: s, .. } => *s = sig,
    }
    Ok(signed)
}

// ── body and transaction root ───────────────────────────────────────────────

pub fn encode_body(txs: &[Vec<u8>]) -> Vec<u8> {
    let mut out = (txs.len() as u32).to_le_bytes().to_vec();
    for tx in txs {
        out.extend_from_slice(tx);
    }
    out
}

/// Split a body into self-delimiting transactions under the budgets of §1.6.
pub fn decode_body(body: &[u8]) -> Result<Vec<Vec<u8>>, DecodeError> {
    if body.len() < 4 {
        return Err(DecodeError::Length { expected: 4, got: body.len() });
    }
    if HEADER_BYTES + body.len() > MAX_BLOCK_BYTES {
        return Err(DecodeError::Budget);
    }
    let count = u32::from_le_bytes(body[0..4].try_into().expect("4")) as usize;
    if count == 0 || count > MAX_BLOCK_TXS {
        return Err(DecodeError::Count);
    }
    let mut txs = Vec::with_capacity(count);
    let mut at = 4;
    for _ in 0..count {
        if at + 2 > body.len() {
            return Err(DecodeError::Count);
        }
        if body[at] != 1 {
            return Err(DecodeError::Version(body[at] as u32));
        }
        let len = tx_len(body[at + 1]).ok_or(DecodeError::Kind(body[at + 1]))?;
        if at + len > body.len() {
            return Err(DecodeError::Count);
        }
        txs.push(body[at..at + len].to_vec());
        at += len;
    }
    if at != body.len() {
        return Err(DecodeError::Count);
    }
    Ok(txs)
}

pub fn tx_leaf(tx_bytes: &[u8]) -> Id {
    hash_tagged(b"soft3/chain/leaf/v1\0", &[tx_bytes])
}

pub fn tx_node(l: &Id, r: &Id) -> Id {
    hash_tagged(b"soft3/chain/node/v1\0", &[l, r])
}

pub fn tx_root(txs: &[Vec<u8>]) -> Id {
    let mut level: Vec<Id> = txs.iter().map(|t| tx_leaf(t)).collect();
    if level.is_empty() {
        return [0u8; 32];
    }
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for pair in level.chunks(2) {
            if pair.len() == 2 {
                next.push(tx_node(&pair[0], &pair[1]));
            } else {
                next.push(pair[0]);
            }
        }
        level = next;
    }
    level[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h(s: &str) -> Id {
        unhex32(s).unwrap()
    }

    #[test]
    fn hemera_anchors() {
        assert_eq!(hex(&hash(b"")), "9e89f2487be432e48c27e24e6945adffe6918c8b5d6a62061e70e0bebd657326");
        assert_eq!(hex(&token("boot")), "1d443e1103abbd0aebe4abaf3fadc203160a471e9accb9b6e0a73804ea785b3b");
        assert_eq!(hex(&token("pussy")), "87a70ff9ed0638fc9fafb1a30f57c5ad2ebe5155c48702c50bdbb2ece87f6a71");
        assert_eq!(hex(&hash(&[0u8; 168])), "355fd03315a5ceb056cda53d11ff7b9755167182d59ff9e31134d69d33bdf0f6");
    }

    #[test]
    fn compact_target_vectors() {
        assert_eq!(decode_bits(0x1e010000), Some(U256::pow2(232)));
        assert_eq!(decode_bits(0x1e100000), Some(U256::pow2(236)));
        assert_eq!(decode_bits(0x1d100000), Some(U256::pow2(228)));
        assert_eq!(decode_bits(0x1d200000), Some(U256::pow2(229)));
        assert_eq!(decode_bits(0x1d080000), Some(U256::pow2(227)));
        assert_eq!(decode_bits(0x1d084860), Some(U256::from_u64(0x084860).shl(8 * 26)));
        assert_eq!(decode_bits(0x03123456), Some(U256::from_u64(0x123456)));
        assert_eq!(decode_bits(0x1e001000), None);
        assert_eq!(decode_bits(0x04001234), None);
        assert_eq!(decode_bits(0x01003456), None);
        assert_eq!(decode_bits(0x1d800000), None);
        assert_eq!(decode_bits(0), None);
        assert_eq!(decode_bits(0x1f010000), Some(U256::pow2(240)));
        assert_eq!(encode_bits(U256::pow2(232)), 0x1e010000);
        assert_eq!(encode_bits(U256::from_u64(0x123400)), 0x03123400);
    }

    #[test]
    fn work_vectors() {
        assert_eq!(work(U256::pow2(228)), U256::from_u64(268_435_455));
        assert_eq!(work(U256::pow2(232)), U256::from_u64(16_777_215));
        assert_eq!(work(U256::pow2(236)), U256::from_u64(1_048_575));
        assert_eq!(work(U256::ONE), U256::pow2(255));
        let three = work(U256::pow2(232)).saturating_add(work(U256::pow2(232))).saturating_add(work(U256::pow2(232)));
        assert_eq!(three, U256::from_u64(50_331_645));
        assert_eq!(U256::from_u64(50_331_645).to_decimal(), "50331645");
    }

    fn test_header() -> Header {
        Header {
            version: 1, height: 1, time: 1762349000, bits: 0x1d100000, nonce: [0; 16],
            prev: [0x11; 32], tx_root: [0x22; 32], state_root: [0x33; 32], chain: [0x44; 32],
        }
    }

    #[test]
    fn header_vector() {
        let header = test_header();
        let bytes = header.encode();
        assert_eq!(&bytes[..4], &[1, 0, 0, 0]);
        assert_eq!(&bytes[12..20], &[0xc8, 0x4f, 0x0b, 0x69, 0, 0, 0, 0]);
        assert_eq!(&bytes[20..24], &[0, 0, 0x10, 0x1d]);
        assert_eq!(hex(&header.id()), "bdd7f2ef3a5a1c573b8d84a7a2aeedc95c6f8aa2139eea5c83946b33c4180c31");
        assert!(!header.valid_pow());
        assert_eq!(Header::decode(&bytes).unwrap(), header);
        assert!(Header::decode(&bytes[..167]).is_err());
        let mut long = bytes.to_vec();
        long.push(0);
        assert!(Header::decode(&long).is_err());
    }

    #[test]
    fn mined_header_vector() {
        let mut header = test_header();
        header.bits = 0x1e010000;
        header.time = 1793885850;
        header.nonce = 934636u128.to_le_bytes();
        assert_eq!(hex(&header.id()), "a3dc0cdb7c69e1d4986c388effe1612613b1b71f2c2ca49084bc0730da000000");
        assert!(header.valid_pow());
        let mut harder = header;
        harder.bits = 0x1d100000;
        assert!(!harder.valid_pow());
        let mut before = header;
        before.nonce = 934635u128.to_le_bytes();
        assert!(!before.valid_pow());
    }

    #[test]
    fn coinbase_vector() {
        let tx = Tx::Coinbase { height: 1, to: [0x55; 32], amount: 10_000, extra: [0; 32] };
        let bytes = tx.encode();
        assert_eq!(bytes.len(), 82);
        assert_eq!(hex(&txid(&[0x44; 32], &bytes)), "6cf0e4d280e6d5cbb5c7fef566d28913b17249cb4a6fdecf9c59ad3f0995c3f9");
        let leaf = tx_leaf(&bytes);
        assert_eq!(hex(&leaf), "514d5d57388046e2cd534c69a789bf09e329678cedbabcff4cc9a439e8c0fe9c");
        assert_eq!(tx_root(&[bytes.clone()]), leaf);
        assert_eq!(hex(&tx_node(&leaf, &leaf)), "9d68ac9b03293ba25059ceb5d601e3c6ddae700e05343c190fdd4c8eea6c595b");
        assert_eq!(Tx::decode(&bytes).unwrap(), tx);
    }

    #[test]
    fn transfer_statement_vector() {
        let tx = Tx::Transfer {
            from: [0x66; 32], nonce: 0, token: token("pussy"), amount: 5, to: [0x77; 32], fee: 224, sig: vec![0; 102],
        };
        let unsigned = tx.encode_unsigned();
        assert_eq!(unsigned.len(), 122);
        assert_eq!(hex(&statement(&[0x44; 32], &unsigned)), "41e149559c98af20b62db3e02ac2dfd4c62568f695d92986f9c274a7d62d12a7");
        assert_eq!(tx.encode().len(), 224);
    }

    #[test]
    fn sign_and_verify_binds_chain() {
        let key = mudra::SigningKey::from_bytes((&[7u8; 32]).into()).unwrap();
        let from = mudra::claim::neuron_of(&mudra::cosmos::compressed(key.verifying_key()));
        let tx = Tx::Transfer { from, nonce: 0, token: token("pussy"), amount: 5, to: [0x77; 32], fee: 224, sig: vec![] };
        let signed = sign_tx(&[0x44; 32], &tx, &key).unwrap();
        assert!(signed.verify(&[0x44; 32]));
        assert!(!signed.verify(&[0x45; 32]));
        let mut bytes = signed.encode();
        bytes[200] ^= 1;
        assert!(!Tx::decode(&bytes).unwrap().verify(&[0x44; 32]));
    }

    #[test]
    fn cyberlink_rules() {
        let link = Tx::Cyberlink {
            from: [1; 32], nonce: 0, token: token("boot"), amount: 0, p: token("boot"), q: token("pussy"), valence: 1, fee: 257, sig: vec![0; 102],
        };
        let mut bytes = link.encode();
        assert_eq!(bytes.len(), 257);
        assert_eq!(Tx::decode(&bytes).unwrap(), link);
        bytes[2 + 144] = 2;
        assert_eq!(Tx::decode(&bytes), Err(DecodeError::Valence(2)));
    }

    #[test]
    fn encoding_rejections() {
        let tx = Tx::Coinbase { height: 1, to: [0x55; 32], amount: 1, extra: [0; 32] };
        let mut bytes = tx.encode();
        bytes.push(0);
        assert!(Tx::decode(&bytes).is_err());
        bytes.truncate(81);
        assert!(Tx::decode(&bytes).is_err());
        bytes = tx.encode();
        bytes[0] = 2;
        assert_eq!(Tx::decode(&bytes), Err(DecodeError::Version(2)));
        bytes[0] = 1;
        bytes[1] = 4;
        assert_eq!(Tx::decode(&bytes), Err(DecodeError::Kind(4)));
    }

    #[test]
    fn topic_vector() {
        assert_eq!(hex(&topic_id(&[0x44; 32], "headers")), "598b569691c3dc0cc6ee22e1cc7a595ee747e33c1b49defc69bc26c831bbd7a8");
    }

    #[test]
    fn body_round_trip_and_budgets() {
        let cb = Tx::Coinbase { height: 1, to: [0x55; 32], amount: 1, extra: [0; 32] }.encode();
        let body = encode_body(&[cb.clone(), cb.clone()]);
        assert_eq!(decode_body(&body).unwrap().len(), 2);
        let mut bad = body.clone();
        bad.push(0);
        assert!(decode_body(&bad).is_err());
        assert!(decode_body(&body[..body.len() - 1]).is_err());
        assert!(decode_body(&0u32.to_le_bytes()).is_err());
    }

    /// Decoders never panic on arbitrary input (xorshift, no external fuzzer).
    #[test]
    fn decoders_survive_random_bytes() {
        let mut x = 0x9e3779b97f4a7c15u64;
        let mut next = || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        for _ in 0..20_000 {
            let len = (next() % 600) as usize;
            let mut bytes: Vec<u8> = (0..len).map(|_| next() as u8).collect();
            let _ = Header::decode(&bytes);
            let _ = Tx::decode(&bytes);
            let _ = decode_body(&bytes);
            let _ = decode_bits(next() as u32);
            if !bytes.is_empty() {
                bytes[0] = 1;
                bytes.truncate([82, 224, 257, 140][(next() % 4) as usize].min(bytes.len()));
                let _ = Tx::decode(&bytes);
            }
        }
        assert_eq!(h("00".repeat(32).as_str()), [0u8; 32]);
        assert_eq!(unhex32("00"), None);
    }
}
