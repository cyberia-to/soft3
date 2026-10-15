//! Proof of work and chainwork (§1.2, §5.1). Integer only.

use super::header::{BlockId, Header, decode_bits};
use super::u256::U256;

/// `u256_le(block_id) < target(bits)`; equality fails (§1.2). Genesis is
/// exempt: height 0 is pinned by the genesis file, not mined.
pub fn valid_pow(header: &Header) -> bool {
    if header.height == 0 {
        return true;
    }
    let Ok(target) = decode_bits(header.bits) else {
        return false;
    };
    id_meets_target(&header.id(), target)
}

/// The comparison alone, for a miner that already has the id.
pub fn id_meets_target(id: &BlockId, target: U256) -> bool {
    U256::from_le_bytes(id) < target
}

/// `work(header) = floor(2^256 / (target + 1))` (§5.1).
pub fn work(header: &Header) -> U256 {
    header.target().work()
}

/// `chainwork(B) = chainwork(prev) + work(B)` (§5.1).
pub fn chainwork(prev: U256, header: &Header) -> U256 {
    prev.saturating_add(work(header))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::params::GENESIS_BITS;

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    fn base() -> Header {
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
    fn test_header_fails_its_own_target() {
        let h = base();
        assert_eq!(h.id()[31], 0x8d);
        assert!(!valid_pow(&h));
        let mut genesis = h;
        genesis.height = 0;
        assert!(valid_pow(&genesis), "height 0 is exempt");
    }

    #[test]
    fn conformance_section_5_mined_header() {
        let mut h = base();
        h.bits = GENESIS_BITS;
        h.time = 1_793_885_850;
        h.nonce = 40_165_933u128.to_le_bytes();
        let bytes = h.encode();
        assert_eq!(
            hex(&bytes[..40]),
            "010000000100000000000000" // version, height
                .to_string()
                + "9a86ec6a00000000"
                + "0000011e"
                + "2de26402000000000000000000000000"
        );
        assert_eq!(hex(&h.id()), "d1eec40d1db983bb3df0946890538bec4ba9712fb26b82316efc6b162f000000");
        assert!(valid_pow(&h));
        let mut harder = h;
        harder.bits = 0x1d10_0000;
        assert!(!valid_pow(&harder), "2^228 needs byte 28 below 0x10; it is 0x2f");
        let mut previous_nonce = h;
        previous_nonce.nonce = 40_165_932u128.to_le_bytes();
        assert!(!valid_pow(&previous_nonce));
    }

    #[test]
    fn work_and_chainwork() {
        let mut h = base();
        h.bits = GENESIS_BITS;
        assert_eq!(work(&h), U256::from_u64(16_777_215));
        let mut acc = U256::ZERO;
        for _ in 0..3 {
            acc = chainwork(acc, &h);
        }
        assert_eq!(acc, U256::from_u64(50_331_645));
        assert_eq!(chainwork(U256::MAX, &h), U256::MAX);
    }
}
