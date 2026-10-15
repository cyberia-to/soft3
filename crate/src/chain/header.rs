//! The 168-byte header (§1.1), its id, and the canonical compact target (§1.3).

use super::params::{HEADER_BYTES, POW_LIMIT_BITS, RULE_SET, pow_limit};
use super::u256::U256;

pub type BlockId = [u8; 32];

/// One header, exactly as on the wire: little-endian, no varints (§1.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub version: u32,
    pub height: u64,
    pub time: u64,
    pub bits: u32,
    pub nonce: [u8; 16],
    pub prev: BlockId,
    pub tx_root: [u8; 32],
    pub state_root: [u8; 32],
    pub chain: [u8; 32],
}

/// Why a header fails syntax (§5.5 step 1). `WrongChain` is distinct from
/// malformed so the wire can score it differently (chain-wire §5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderError {
    Length(usize),
    RuleSet(u16),
    WrongChain,
    Bits(BitsError),
}

impl std::fmt::Display for HeaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeaderError::Length(n) => write!(f, "header is {n} bytes, expected {HEADER_BYTES}"),
            HeaderError::RuleSet(r) => write!(f, "header claims rule set {r}"),
            HeaderError::WrongChain => write!(f, "header belongs to another chain"),
            HeaderError::Bits(e) => write!(f, "bits: {e}"),
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

    /// Decode without any rule check (the store reads its own bytes this way).
    pub fn decode_raw(b: &[u8]) -> Result<Header, HeaderError> {
        if b.len() != HEADER_BYTES {
            return Err(HeaderError::Length(b.len()));
        }
        let u32_at = |o: usize| u32::from_le_bytes(b[o..o + 4].try_into().expect("4 bytes"));
        let u64_at = |o: usize| u64::from_le_bytes(b[o..o + 8].try_into().expect("8 bytes"));
        let arr32 = |o: usize| -> [u8; 32] { b[o..o + 32].try_into().expect("32 bytes") };
        Ok(Header {
            version: u32_at(0),
            height: u64_at(4),
            time: u64_at(12),
            bits: u32_at(20),
            nonce: b[24..40].try_into().expect("16 bytes"),
            prev: arr32(40),
            tx_root: arr32(72),
            state_root: arr32(104),
            chain: arr32(136),
        })
    }

    /// Decode and run §5.5 step 1: length, rule set, chain, canonical bits.
    pub fn decode(b: &[u8], chain: &[u8; 32]) -> Result<Header, HeaderError> {
        let h = Header::decode_raw(b)?;
        h.check_syntax(chain)?;
        Ok(h)
    }

    /// §5.5 step 1 on a decoded header.
    pub fn check_syntax(&self, chain: &[u8; 32]) -> Result<(), HeaderError> {
        if self.rule_set() != RULE_SET {
            return Err(HeaderError::RuleSet(self.rule_set()));
        }
        if &self.chain != chain {
            return Err(HeaderError::WrongChain);
        }
        decode_bits(self.bits).map_err(HeaderError::Bits)?;
        Ok(())
    }

    /// `version & 0xffff` (§7.1).
    pub fn rule_set(&self) -> u16 {
        (self.version & 0xffff) as u16
    }

    /// `version >> 16`: miner signals, informational (§7.1).
    pub fn signals(&self) -> u16 {
        (self.version >> 16) as u16
    }

    /// `block_id = hemera(header_bytes)`, flat sponge, no domain prefix (§1.1).
    pub fn id(&self) -> BlockId {
        hash32(&self.encode())
    }

    /// The target this header claims; valid only after `check_syntax`.
    pub fn target(&self) -> U256 {
        decode_bits(self.bits).expect("checked bits")
    }
}

/// hemera over bytes, 32 bytes out.
pub fn hash32(bytes: &[u8]) -> [u8; 32] {
    let digest = hemera::hash(bytes);
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest.as_bytes()[..32]);
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitsError {
    Sign,
    Zero,
    NonCanonical { canonical: u32 },
    AboveLimit,
}

impl std::fmt::Display for BitsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitsError::Sign => write!(f, "sign bit set"),
            BitsError::Zero => write!(f, "target 0"),
            BitsError::NonCanonical { canonical } => write!(f, "non-canonical (canon {canonical:#010x})"),
            BitsError::AboveLimit => write!(f, "target above POW_LIMIT"),
        }
    }
}

/// Compact → target with no checks beyond the sign bit (§1.3).
pub fn decode_bits_raw(bits: u32) -> Result<U256, BitsError> {
    if bits & 0x0080_0000 != 0 {
        return Err(BitsError::Sign);
    }
    let exp = bits >> 24;
    let mant = U256::from_u64((bits & 0x007f_ffff) as u64);
    Ok(if exp >= 3 { mant << (8 * (exp - 3)) } else { mant >> (8 * (3 - exp)) })
}

/// Canonical encode of a target (§1.3).
pub fn encode_bits(target: U256) -> u32 {
    let mut size = target.byte_length();
    let mut mant = if size <= 3 {
        (target << (8 * (3 - size))).low_u64() as u32
    } else {
        (target >> (8 * (size - 3))).low_u64() as u32
    };
    if mant & 0x0080_0000 != 0 {
        mant >>= 8;
        size += 1;
    }
    (size << 24) | mant
}

/// Compact → target, rejecting sign, zero, non-canonical and above-limit
/// encodings (§1.3): the only decoder consensus code may use.
pub fn decode_bits(bits: u32) -> Result<U256, BitsError> {
    let target = decode_bits_raw(bits)?;
    if target.is_zero() {
        return Err(BitsError::Zero);
    }
    let canonical = encode_bits(target);
    if canonical != bits {
        return Err(BitsError::NonCanonical { canonical });
    }
    if target > pow_limit() {
        return Err(BitsError::AboveLimit);
    }
    Ok(target)
}

/// Target → canonical bits after the clamp to `[1, POW_LIMIT]` (§1.5).
pub fn clamp_bits(target: U256) -> u32 {
    let limit = pow_limit();
    let t = if target.is_zero() { U256::ONE } else if target > limit { limit } else { target };
    let bits = encode_bits(t);
    debug_assert!(bits <= POW_LIMIT_BITS);
    bits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    pub fn test_header() -> Header {
        Header {
            version: 1,
            height: 1,
            time: 1_762_349_000,
            bits: 0x1d10_0000,
            nonce: [0; 16],
            prev: [0x11; 32],
            tx_root: [0x22; 32],
            state_root: [0x33; 32],
            chain: [0x44; 32],
        }
    }

    #[test]
    fn conformance_section_5_header_bytes_and_id() {
        let h = test_header();
        let bytes = h.encode();
        assert_eq!(&hex(&bytes[..24]), "010000000100000000000000c84f0b69000000000000101d");
        assert_eq!(&bytes[24..40], &[0u8; 16]);
        assert_eq!(&bytes[40..72], &[0x11u8; 32]);
        assert_eq!(&bytes[136..], &[0x44u8; 32]);
        assert_eq!(hex(&h.id()), "bdfbbff94e6e17eec9169932276825a0f4957f6f3a6af9030439979eacaeb58d");
        assert_eq!(Header::decode(&bytes, &[0x44; 32]).unwrap(), h);
        assert_eq!(Header::decode(&bytes[..167], &[0x44; 32]), Err(HeaderError::Length(167)));
        let mut long = bytes.to_vec();
        long.push(0);
        assert_eq!(Header::decode(&long, &[0x44; 32]), Err(HeaderError::Length(169)));
        assert_eq!(Header::decode(&bytes, &[0x45; 32]), Err(HeaderError::WrongChain));
        let mut v2 = h;
        v2.version = 2;
        assert_eq!(Header::decode(&v2.encode(), &[0x44; 32]), Err(HeaderError::RuleSet(2)));
        let mut signalled = h;
        signalled.version = 1 | (0xabcd << 16);
        assert!(Header::decode(&signalled.encode(), &[0x44; 32]).is_ok());
        assert_eq!(signalled.signals(), 0xabcd);
    }

    #[test]
    fn conformance_section_2_compact_target() {
        let canonical = [
            (0x1e01_0000u32, U256::ONE << 232),
            (0x1e10_0000, U256::ONE << 236),
            (0x1d10_0000, U256::ONE << 228),
            (0x1d20_0000, U256::ONE << 229),
            (0x1d08_0000, U256::ONE << 227),
            (0x1d08_4860, U256::from_u64(0x08_4860) << (8 * 26)),
            (0x0312_3456, U256::from_u64(0x12_3456)),
        ];
        for (bits, target) in canonical {
            assert_eq!(decode_bits(bits), Ok(target), "{bits:#x}");
            assert_eq!(encode_bits(target), bits, "{bits:#x}");
        }
        assert_eq!(decode_bits(0x1e00_1000), Err(BitsError::NonCanonical { canonical: 0x1d10_0000 }));
        assert_eq!(decode_bits(0x0400_1234), Err(BitsError::NonCanonical { canonical: 0x0312_3400 }));
        assert_eq!(decode_bits(0x0100_3456), Err(BitsError::Zero));
        assert_eq!(decode_bits(0x1d80_0000), Err(BitsError::Sign));
        assert_eq!(decode_bits(0x0000_0000), Err(BitsError::Zero));
        assert_eq!(decode_bits(0x1f01_0000), Err(BitsError::AboveLimit));
        assert_eq!(clamp_bits(U256::ZERO), 0x0101_0000);
        assert_eq!(clamp_bits(U256::ONE << 240), POW_LIMIT_BITS);
    }
}
