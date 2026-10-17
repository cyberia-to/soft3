//! Unsigned 512-bit integers for attribution (chain-economy §5.2–§5.5).
//!
//! `a(c) = Δs · σ · g` is a 256-bit signed quantity (a u64 stake times an
//! i128 gradient); `x(c) = floor(a⁺ · N / R)` and `pay = floor(B · x / X)`
//! need one more u64 factor before the division. Every product here fits
//! 512 bits by construction, and every division floors. Little-endian
//! limbs; no allocation; nothing platform-dependent.

use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct W(pub [u64; 8]);

impl W {
    pub const ZERO: W = W([0; 8]);

    pub fn from_u64(v: u64) -> W {
        let mut w = [0u64; 8];
        w[0] = v;
        W(w)
    }

    pub fn from_u128(v: u128) -> W {
        let mut w = [0u64; 8];
        w[0] = v as u64;
        w[1] = (v >> 64) as u64;
        W(w)
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|l| *l == 0)
    }

    /// Bits needed to write the value (0 for zero).
    pub fn bits(&self) -> u32 {
        for i in (0..8).rev() {
            if self.0[i] != 0 {
                return 64 * i as u32 + 64 - self.0[i].leading_zeros();
            }
        }
        0
    }

    /// The value if it fits a u64.
    pub fn to_u64(&self) -> Option<u64> {
        self.0[1..].iter().all(|l| *l == 0).then_some(self.0[0])
    }

    pub fn checked_add(&self, o: &W) -> Option<W> {
        let mut out = [0u64; 8];
        let mut carry = 0u128;
        for (i, slot) in out.iter_mut().enumerate() {
            let s = self.0[i] as u128 + o.0[i] as u128 + carry;
            *slot = s as u64;
            carry = s >> 64;
        }
        (carry == 0).then_some(W(out))
    }

    fn sub_assign(&mut self, o: &W) {
        let mut borrow = 0i128;
        for i in 0..8 {
            let d = self.0[i] as i128 - o.0[i] as i128 - borrow;
            if d < 0 {
                self.0[i] = (d + (1i128 << 64)) as u64;
                borrow = 1;
            } else {
                self.0[i] = d as u64;
                borrow = 0;
            }
        }
        debug_assert_eq!(borrow, 0, "sub underflow");
    }

    /// Full product; `None` if it exceeds 512 bits.
    pub fn checked_mul(&self, o: &W) -> Option<W> {
        let mut out = [0u64; 16];
        for i in 0..8 {
            if self.0[i] == 0 {
                continue;
            }
            let mut carry = 0u128;
            for j in 0..8 {
                let cur = out[i + j] as u128 + self.0[i] as u128 * o.0[j] as u128 + carry;
                out[i + j] = cur as u64;
                carry = cur >> 64;
            }
            let mut k = i + 8;
            while carry > 0 {
                let cur = out[k] as u128 + carry;
                out[k] = cur as u64;
                carry = cur >> 64;
                k += 1;
            }
        }
        if out[8..].iter().any(|l| *l != 0) {
            return None;
        }
        let mut w = [0u64; 8];
        w.copy_from_slice(&out[..8]);
        Some(W(w))
    }

    fn bit(&self, i: u32) -> bool {
        (self.0[(i / 64) as usize] >> (i % 64)) & 1 == 1
    }

    fn shl1_or(&mut self, b: bool) {
        let mut carry = b as u64;
        for l in self.0.iter_mut() {
            let next = *l >> 63;
            *l = (*l << 1) | carry;
            carry = next;
        }
    }

    /// `floor(self / d)` for a u64 divisor (limb-wise, fast).
    pub fn div_u64(&self, d: u64) -> W {
        assert!(d != 0, "division by zero");
        let mut out = [0u64; 8];
        let mut rem = 0u128;
        for i in (0..8).rev() {
            let cur = (rem << 64) | self.0[i] as u128;
            out[i] = (cur / d as u128) as u64;
            rem = cur % d as u128;
        }
        W(out)
    }

    /// `floor(self / d)`, bit-serial over the numerator's bits.
    pub fn div(&self, d: &W) -> W {
        assert!(!d.is_zero(), "division by zero");
        if let Some(small) = d.to_u64() {
            return self.div_u64(small);
        }
        let mut q = W::ZERO;
        let mut r = W::ZERO;
        for i in (0..self.bits()).rev() {
            r.shl1_or(self.bit(i));
            q.shl1_or(false);
            if r >= *d {
                r.sub_assign(d);
                q.0[0] |= 1;
            }
        }
        q
    }
}

impl Ord for W {
    fn cmp(&self, o: &W) -> Ordering {
        for i in (0..8).rev() {
            match self.0[i].cmp(&o.0[i]) {
                Ordering::Equal => continue,
                other => return other,
            }
        }
        Ordering::Equal
    }
}

impl PartialOrd for W {
    fn partial_cmp(&self, o: &W) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

/// `floor(a · b / c)` with every operand up to 512 bits and the product
/// within 512 bits (true for every caller: ≤ 2^256 · 2^64).
pub fn mul_div(a: &W, b: &W, c: &W) -> W {
    a.checked_mul(b).expect("attribution products fit 512 bits").div(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_agrees_with_u128_where_it_fits() {
        let cases: [(u128, u128, u128); 5] =
            [(0, 5, 3), (7, 9, 4), (u64::MAX as u128, u64::MAX as u128, 3), (1 << 100, 1 << 20, (1 << 64) + 7), ((1 << 120) + 99, 255, 1 << 70)];
        for (a, b, c) in cases {
            let got = mul_div(&W::from_u128(a), &W::from_u128(b), &W::from_u128(c));
            assert_eq!(got, W::from_u128(a * b / c), "{a} * {b} / {c}");
        }
    }

    #[test]
    fn wide_products_and_divisions() {
        // 2^255 · (2^64 − 1) / 2^255 = 2^64 − 1
        let mut big = W::ZERO;
        big.0[3] = 1 << 63;
        let got = mul_div(&big, &W::from_u64(u64::MAX), &big);
        assert_eq!(got.to_u64(), Some(u64::MAX));
        // (2^200 + 1) · 3 / (2^199) = 6 (floor)
        let mut a = W::ZERO;
        a.0[3] = 1 << 8;
        a.0[0] = 1;
        let mut c = W::ZERO;
        c.0[3] = 1 << 7;
        assert_eq!(mul_div(&a, &W::from_u64(3), &c).to_u64(), Some(6));
        assert_eq!(a.bits(), 201);
        assert!(W::from_u64(1).checked_add(&W([u64::MAX; 8])).is_none());
        assert!(W([u64::MAX; 8]).checked_mul(&W::from_u64(2)).is_none());
    }
}
