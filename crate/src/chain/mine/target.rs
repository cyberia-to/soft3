//! The miner's view of the kernel: targets as the four little-endian words
//! the kernels compare ([`Words`]), and the two consensus questions a miner
//! asks — the target of a `bits` field and whether a header verifies —
//! answered by [`super::super::header`] and [`super::super::pow`], never a
//! second implementation.

use crate::chain::header::{decode_bits, hash32};
use crate::chain::params::HEADER_BYTES;
use crate::chain::pow::id_meets_target;
use crate::chain::u256::U256;

/// A target as four little-endian 64-bit words (word 3 most significant),
/// the form the CPU, Metal and WebGPU kernels compare digests against.
pub type Words = [u64; 4];

/// Header offsets a miner touches (chain.md §1.1).
pub const NONCE_OFFSET: usize = 24;
pub const BITS_OFFSET: usize = 20;

/// The target of a compact `bits` field, or why it is invalid (§1.3).
pub fn target_of(bits: u32) -> Result<Words, String> {
    decode_bits(bits)
        .map(|t| t.0)
        .map_err(|e| format!("bits {bits:#010x}: {e:?}"))
}

/// `u256_le(hemera(header)) < target` (§1.2), on raw header bytes.
pub fn verifies(header: &[u8; HEADER_BYTES], target: &Words) -> bool {
    id_meets_target(&hash32(header), U256(*target))
}

/// `a < b` as 256-bit integers.
pub fn below(a: &Words, b: &Words) -> bool {
    U256(*a) < U256(*b)
}

/// `2^n` (`n < 256`).
pub const fn pow2(n: u32) -> Words {
    let mut t = [0u64; 4];
    t[(n / 64) as usize] = 1u64 << (n % 64);
    t
}

/// The 32 little-endian bytes of a target (the `target` field of a template).
pub fn to_bytes(t: &Words) -> [u8; 32] {
    U256(*t).to_le_bytes()
}

/// A target from its 32 little-endian bytes.
pub fn from_bytes(b: &[u8; 32]) -> Words {
    U256::from_le_bytes(b).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::params::{GENESIS_BITS, POW_LIMIT_BITS};

    #[test]
    fn targets_come_from_the_kernel() {
        assert_eq!(target_of(GENESIS_BITS).unwrap(), pow2(232));
        assert_eq!(target_of(POW_LIMIT_BITS).unwrap(), pow2(236));
        assert!(target_of(0x1e00_1000).is_err());
        assert_eq!(from_bytes(&to_bytes(&pow2(232))), pow2(232));
        assert!(below(&[u64::MAX, u64::MAX, u64::MAX, 0], &[0, 0, 0, 1]));
    }

    /// chain-conformance §5: the mined header verifies at 2^232, not 2^228;
    /// the nonce before it fails.
    #[test]
    fn conformance_mined_header_verifies() {
        let h = |nonce: u128| {
            let mut h = [0u8; 168];
            h[0..4].copy_from_slice(&1u32.to_le_bytes());
            h[4..12].copy_from_slice(&1u64.to_le_bytes());
            h[12..20].copy_from_slice(&1_793_885_850u64.to_le_bytes());
            h[20..24].copy_from_slice(&GENESIS_BITS.to_le_bytes());
            h[24..40].copy_from_slice(&nonce.to_le_bytes());
            h[40..72].fill(0x11);
            h[72..104].fill(0x22);
            h[104..136].fill(0x33);
            h[136..168].fill(0x44);
            h
        };
        assert!(verifies(&h(934_636), &pow2(232)));
        assert!(!verifies(&h(934_636), &pow2(228)));
        assert!(!verifies(&h(934_635), &pow2(232)));
    }
}
