//! Fixed-width unsigned integers for targets and work. Integer only; no
//! floats anywhere on this path (§1.5 design rule).
//!
//! `U256` is four little-endian u64 limbs. `U512` is eight and exists for
//! the one place a product exceeds 256 bits: `genesis_target · factor` in
//! ASERT (§1.5).

use std::cmp::Ordering;

#[derive(Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct U256(pub [u64; 4]);

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct U512(pub [u64; 8]);

impl U256 {
    pub const ZERO: U256 = U256([0; 4]);
    pub const ONE: U256 = U256([1, 0, 0, 0]);
    pub const MAX: U256 = U256([u64::MAX; 4]);

    pub fn from_u64(v: u64) -> Self {
        U256([v, 0, 0, 0])
    }

    /// Little-endian 32 bytes, the wire form of a block id and of chainwork.
    pub fn from_le_bytes(b: &[u8; 32]) -> Self {
        let mut limbs = [0u64; 4];
        for (i, limb) in limbs.iter_mut().enumerate() {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(&b[i * 8..i * 8 + 8]);
            *limb = u64::from_le_bytes(buf);
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

    pub fn is_zero(self) -> bool {
        self.0 == [0; 4]
    }

    /// Number of significant bits (0 for zero).
    pub fn bits(self) -> u32 {
        for i in (0..4).rev() {
            if self.0[i] != 0 {
                return 64 * i as u32 + (64 - self.0[i].leading_zeros());
            }
        }
        0
    }

    /// Number of significant bytes (0 for zero).
    pub fn byte_length(self) -> u32 {
        self.bits().div_ceil(8)
    }

    pub fn low_u64(self) -> u64 {
        self.0[0]
    }

    pub fn checked_add(self, o: U256) -> Option<U256> {
        let mut out = [0u64; 4];
        let mut carry = 0u64;
        for i in 0..4 {
            let (s1, c1) = self.0[i].overflowing_add(o.0[i]);
            let (s2, c2) = s1.overflowing_add(carry);
            out[i] = s2;
            carry = (c1 as u64) + (c2 as u64);
        }
        if carry != 0 { None } else { Some(U256(out)) }
    }

    /// Saturating sum: chainwork never overflows in practice (2^256 hashes),
    /// and a saturated value still orders correctly.
    pub fn saturating_add(self, o: U256) -> U256 {
        self.checked_add(o).unwrap_or(U256::MAX)
    }

    pub fn wrapping_sub(self, o: U256) -> U256 {
        let mut out = [0u64; 4];
        let mut borrow = 0u64;
        for i in 0..4 {
            let (d1, b1) = self.0[i].overflowing_sub(o.0[i]);
            let (d2, b2) = d1.overflowing_sub(borrow);
            out[i] = d2;
            borrow = (b1 as u64) + (b2 as u64);
        }
        U256(out)
    }

    pub fn shl(self, n: u32) -> U256 {
        if n >= 256 {
            return U256::ZERO;
        }
        let limb_shift = (n / 64) as usize;
        let bit_shift = n % 64;
        let mut out = [0u64; 4];
        for i in (limb_shift..4).rev() {
            let src = i - limb_shift;
            out[i] = self.0[src] << bit_shift;
            if bit_shift != 0 && src > 0 {
                out[i] |= self.0[src - 1] >> (64 - bit_shift);
            }
        }
        U256(out)
    }

    pub fn shr(self, n: u32) -> U256 {
        if n >= 256 {
            return U256::ZERO;
        }
        let limb_shift = (n / 64) as usize;
        let bit_shift = n % 64;
        let mut out = [0u64; 4];
        for i in 0..(4 - limb_shift) {
            let src = i + limb_shift;
            out[i] = self.0[src] >> bit_shift;
            if bit_shift != 0 && src + 1 < 4 {
                out[i] |= self.0[src + 1] << (64 - bit_shift);
            }
        }
        U256(out)
    }

    /// `floor(2^256 / (self + 1))`: the expected hashes to meet a target
    /// (§5.1). Computed as `(2^256 − d) / d + 1` for `d = self + 1`, which
    /// is `(U256::MAX − self) / d + 1` and never leaves 256 bits.
    pub fn work(self) -> U256 {
        if self == U256::MAX {
            return U256::ONE; // 2^256 / 2^256
        }
        let d = self.checked_add(U256::ONE).expect("self < MAX");
        let (q, _) = U256::MAX.wrapping_sub(self).div_rem(d);
        // target 0 would be 2^256 exactly; it is rejected by the bits codec
        // before it reaches here, and saturates if it ever does.
        q.saturating_add(U256::ONE)
    }

    /// Long division; `d` must be non-zero.
    pub fn div_rem(self, d: U256) -> (U256, U256) {
        assert!(!d.is_zero(), "division by zero");
        let mut q = U256::ZERO;
        let mut r = U256::ZERO;
        for i in (0..256).rev() {
            r = r.shl(1);
            if (self.0[i / 64] >> (i % 64)) & 1 == 1 {
                r.0[0] |= 1;
            }
            if r >= d {
                r = r.wrapping_sub(d);
                q.0[i / 64] |= 1 << (i % 64);
            }
        }
        (q, r)
    }

    pub fn to_u512(self) -> U512 {
        let mut limbs = [0u64; 8];
        limbs[..4].copy_from_slice(&self.0);
        U512(limbs)
    }
}

impl std::ops::Shl<u32> for U256 {
    type Output = U256;
    fn shl(self, n: u32) -> U256 {
        U256::shl(self, n)
    }
}

impl std::ops::Shr<u32> for U256 {
    type Output = U256;
    fn shr(self, n: u32) -> U256 {
        U256::shr(self, n)
    }
}

impl Ord for U256 {
    fn cmp(&self, o: &Self) -> Ordering {
        for i in (0..4).rev() {
            match self.0[i].cmp(&o.0[i]) {
                Ordering::Equal => continue,
                other => return other,
            }
        }
        Ordering::Equal
    }
}

impl PartialOrd for U256 {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl std::fmt::Debug for U256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x")?;
        let mut started = false;
        for limb in self.0.iter().rev() {
            if started {
                write!(f, "{limb:016x}")?;
            } else if *limb != 0 {
                write!(f, "{limb:x}")?;
                started = true;
            }
        }
        if !started {
            write!(f, "0")?;
        }
        Ok(())
    }
}

impl U512 {
    pub fn is_zero(self) -> bool {
        self.0 == [0; 8]
    }

    /// Multiply by a small factor; the product of a 256-bit value and a u64
    /// always fits 512 bits.
    pub fn mul_u64(self, m: u64) -> U512 {
        let mut out = [0u64; 8];
        let mut carry = 0u128;
        for i in 0..8 {
            let p = self.0[i] as u128 * m as u128 + carry;
            out[i] = p as u64;
            carry = p >> 64;
        }
        debug_assert_eq!(carry, 0, "u512 overflow");
        U512(out)
    }

    pub fn shl(self, n: u32) -> U512 {
        if n >= 512 {
            return U512([0; 8]);
        }
        let limb_shift = (n / 64) as usize;
        let bit_shift = n % 64;
        let mut out = [0u64; 8];
        for i in (limb_shift..8).rev() {
            let src = i - limb_shift;
            out[i] = self.0[src] << bit_shift;
            if bit_shift != 0 && src > 0 {
                out[i] |= self.0[src - 1] >> (64 - bit_shift);
            }
        }
        U512(out)
    }

    pub fn shr(self, n: u32) -> U512 {
        if n >= 512 {
            return U512([0; 8]);
        }
        let limb_shift = (n / 64) as usize;
        let bit_shift = n % 64;
        let mut out = [0u64; 8];
        for i in 0..(8 - limb_shift) {
            let src = i + limb_shift;
            out[i] = self.0[src] >> bit_shift;
            if bit_shift != 0 && src + 1 < 8 {
                out[i] |= self.0[src + 1] << (64 - bit_shift);
            }
        }
        U512(out)
    }

    /// Lower 256 bits, or `None` when the upper half is non-zero.
    pub fn to_u256(self) -> Option<U256> {
        if self.0[4..].iter().any(|l| *l != 0) {
            return None;
        }
        let mut limbs = [0u64; 4];
        limbs.copy_from_slice(&self.0[..4]);
        Some(U256(limbs))
    }

    /// Overflow of 256 bits saturates to `U256::MAX`.
    pub fn saturating_u256(self) -> U256 {
        self.to_u256().unwrap_or(U256::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn add512(a: U512, b: U512) -> U512 {
        let mut out = [0u64; 8];
        let mut carry = 0u128;
        for i in 0..8 {
            let s = a.0[i] as u128 + b.0[i] as u128 + carry;
            out[i] = s as u64;
            carry = s >> 64;
        }
        U512(out)
    }

    #[test]
    fn shifts_and_bits() {
        let one = U256::ONE;
        assert_eq!((one << 232).bits(), 233);
        assert_eq!((one << 232).byte_length(), 30);
        assert_eq!(((one << 232) >> 232), one);
        assert_eq!((one << 255) >> 255, one);
        assert_eq!(one << 256, U256::ZERO);
        assert_eq!(U256::ZERO.bits(), 0);
        assert_eq!(U256::from_u64(0x123456).byte_length(), 3);
    }

    #[test]
    fn work_vectors_of_conformance_section_3() {
        assert_eq!((U256::ONE << 228).work(), U256::from_u64(268_435_455));
        assert_eq!((U256::ONE << 232).work(), U256::from_u64(16_777_215));
        assert_eq!((U256::ONE << 236).work(), U256::from_u64(1_048_575));
        assert_eq!(U256::ONE.work(), U256::ONE << 255);
        assert_eq!(U256::ZERO.work(), U256::MAX);
        assert_eq!(U256::MAX.work(), U256::ONE);
    }

    #[test]
    fn division_round_trips() {
        let a = U256([0x1234_5678_9abc_def0, 0xfedc_ba98_7654_3210, 7, 0x8000_0000_0000_0000]);
        let d = U256([0xdead_beef, 3, 0, 0]);
        let (q, r) = a.div_rem(d);
        assert!(r < d);
        // q·d + r == a, with q·d computed limb by limb through u512
        let mut acc = U512([0; 8]);
        for (i, limb) in d.0.iter().enumerate() {
            acc = add512(acc, q.to_u512().mul_u64(*limb).shl(64 * i as u32));
        }
        let back = add512(acc, r.to_u512()).to_u256().unwrap();
        assert_eq!(back, a);
        let (q2, r2) = U256::from_u64(1000).div_rem(U256::from_u64(7));
        assert_eq!((q2.low_u64(), r2.low_u64()), (142, 6));
    }

    #[test]
    fn u512_mul_and_shift() {
        let g = (U256::ONE << 228).to_u512();
        let v = g.mul_u64(65536).shr(16).to_u256().unwrap();
        assert_eq!(v, U256::ONE << 228);
        assert!(g.mul_u64(u64::MAX).shl(200).to_u256().is_none());
        assert_eq!(g.mul_u64(u64::MAX).shl(200).saturating_u256(), U256::MAX);
    }

    #[test]
    fn le_bytes_round_trip_and_order() {
        let mut b = [0u8; 32];
        b[31] = 0x8d;
        let v = U256::from_le_bytes(&b);
        assert_eq!(v.to_le_bytes(), b);
        assert!(v > U256::ONE << 228);
        assert!(v >= U256::ONE << 255);
    }
}
