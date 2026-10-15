//! Difficulty adjustment: integer ASERT (§1.5) and median time past (§1.4).
//!
//! The target of a block at height `h ≥ 1` is a pure function of the genesis
//! anchor and the parent; the candidate's own time never enters. A
//! floating-point `pow` anywhere here is a consensus bug.

use super::header::{Header, clamp_bits};
use super::params::{BLOCK_TIME, HALF_LIFE, MTP_WINDOW};
use super::u256::U256;

/// What the formula needs from the genesis block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Anchor {
    pub target: U256,
    pub time: u64,
}

/// The intermediate values conformance §4 pins, so a test can compare each.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Asert {
    pub e: i64,
    pub shifts: i64,
    pub frac: u64,
    pub factor: u64,
    pub target: U256,
    pub bits: u32,
}

/// Floor division toward −∞ for i64 (Rust's `/` truncates toward zero).
pub fn floor_div(a: i64, b: i64) -> i64 {
    let q = a / b;
    if (a % b != 0) && ((a < 0) != (b < 0)) { q - 1 } else { q }
}

fn floor_div_i128(a: i128, b: i128) -> i128 {
    let q = a / b;
    if (a % b != 0) && ((a < 0) != (b < 0)) { q - 1 } else { q }
}

/// The cubic approximation of `2^(frac/65536)` in 16.16 fixed point
/// (aserti3-2d, Bitcoin Cash, MIT), error below 0.01 %.
pub fn factor(frac: u64) -> u64 {
    debug_assert!(frac < 65_536);
    let f = frac as u128;
    let poly = 195_766_423_245_049u128 * f + 971_821_376u128 * f * f + 5_127u128 * f * f * f + (1u128 << 47);
    65_536 + (poly >> 48) as u64
}

/// ASERT for the block after `parent` (§1.5), with every intermediate.
pub fn asert(anchor: Anchor, parent_height: u64, parent_time: u64) -> Asert {
    // i128 so no realistic or hostile height/time can overflow; `e` itself
    // is saturated into i64, far beyond where the clamp already binds.
    let dt = parent_time as i128 - anchor.time as i128;
    let ideal = parent_height as i128 * BLOCK_TIME as i128;
    let e = floor_div_i128((dt - ideal) * 65_536, HALF_LIFE as i128);
    let e = e.clamp(i64::MIN as i128, i64::MAX as i128) as i64;
    let shifts = e >> 16;
    let frac = (e & 0xffff) as u64;
    let factor = factor(frac);
    // scaled < 2^(256+17); a left shift of 240 or more lands above 2^256
    // after the >> 16 and is clamped to POW_LIMIT anyway, so saturate there
    // and never shift past 512 bits ("shifts beyond 256 saturate at the clamp").
    let scaled = anchor.target.to_u512().mul_u64(factor);
    let shifted = if shifts < 0 {
        scaled.shr((-shifts).min(512) as u32)
    } else if shifts >= 240 {
        super::u256::U512([u64::MAX; 8])
    } else {
        scaled.shl(shifts as u32)
    };
    let target = shifted.shr(16).saturating_u256();
    let bits = clamp_bits(target);
    Asert { e, shifts, frac, factor, target: super::header::decode_bits(bits).expect("clamped"), bits }
}

/// Expected `bits` of the block after `parent` (§5.5 step 4).
pub fn next_bits(anchor: Anchor, parent: &Header) -> u32 {
    asert(anchor, parent.height, parent.time).bits
}

/// Median time past of the last `MTP_WINDOW` headers (§1.4): `times` holds
/// the parent's and its ancestors' times, any order, at most the window;
/// fewer near genesis. The median of an even count is the lower middle.
pub fn median_time_past(times: &[u64]) -> u64 {
    assert!(!times.is_empty() && times.len() <= MTP_WINDOW);
    let mut sorted = times.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() - 1) / 2]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anchor() -> Anchor {
        Anchor { target: U256::ONE << 228, time: 0 }
    }

    #[test]
    fn floor_div_rounds_toward_negative_infinity() {
        assert_eq!(floor_div(-37_355_520, 600), -62_260);
        assert_eq!(floor_div(37_355_520, 600), 62_259);
        assert_eq!(floor_div(-600, 600), -1);
        assert_eq!(floor_div(-1, 600), -1);
        assert_eq!(floor_div(0, 600), 0);
    }

    #[test]
    fn conformance_section_4_rows() {
        let rows: [(u64, u64, i64, i64, u64, u64, u32); 6] = [
            (0, 0, 0, 0, 0, 65_536, 0x1d10_0000),
            (19, 570, 0, 0, 0, 65_536, 0x1d10_0000),
            (19, 1_170, 65_536, 1, 0, 65_536, 0x1d20_0000),
            (100, 2_400, -65_536, -1, 0, 65_536, 0x1d08_0000),
            (19, 0, -62_260, -1, 3_276, 67_852, 0x1d08_4860),
            (5, 1_000_000, 109_210_282, 1_666, 27_306, 87_473, 0x1e10_0000),
        ];
        for (height, time, e, shifts, frac, factor, bits) in rows {
            let a = asert(anchor(), height, time);
            assert_eq!((a.e, a.shifts, a.frac, a.factor, a.bits), (e, shifts, frac, factor, bits), "parent {height}@{time}");
        }
        assert_eq!(asert(anchor(), 19, 0).target, U256::from_u64(0x08_4860) << (8 * 26));
        assert_eq!(asert(anchor(), 5, 1_000_000).target, U256::ONE << 236);
    }

    #[test]
    fn far_below_schedule_clamps_to_one() {
        let a = asert(anchor(), 1_000_000, 0);
        assert_eq!(a.bits, 0x0101_0000);
        assert_eq!(a.target, U256::ONE);
    }

    #[test]
    fn factor_endpoints() {
        assert_eq!(factor(0), 65_536);
        let top = factor(65_535);
        assert!(top > 131_060 && top <= 131_072, "{top}");
    }

    #[test]
    fn mtp_lower_middle() {
        assert_eq!(median_time_past(&[5]), 5);
        assert_eq!(median_time_past(&[1, 2]), 1);
        assert_eq!(median_time_past(&[3, 1, 2]), 2);
        assert_eq!(median_time_past(&[4, 1, 3, 2]), 2);
        assert_eq!(median_time_past(&[11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]), 6);
    }

    #[test]
    fn t_spacing_holds_bits_and_half_spacing_roughly_doubles_difficulty() {
        let a = anchor();
        for h in 1..200u64 {
            assert_eq!(asert(a, h, h * 30).bits, 0x1d10_0000);
        }
        // blocks at T/2 for 600 s of block time (40 blocks): parent 40 at time
        // 600 is 600 s ahead of schedule → the target halves exactly (ratio
        // 2.0, inside [1.9, 2.1])
        let x = asert(a, 40, 600);
        assert_eq!(x.target, U256::ONE << 227);
        // 100× slowdown: the target doubles per 600 s of wall time whether or
        // not blocks arrive; after 4,200 s (70 min) it has grown 2^7 = 128 ≥ 100,
        // so the interval is back at T — and still under POW_LIMIT
        let slow = asert(a, 1, 30 + 4_200);
        assert_eq!(slow.target, U256::ONE << 235);
        assert!(slow.target <= U256::ONE << 236);
        // 2 × step: 90 % absorbed within 50 min — parent 100 blocks at T/2
        // spacing is 1,500 s ahead → factor 2^-2.5; the drift stops growing
        // once the target has fallen by 2 (checked at the 600 s mark above)
        assert!(asert(a, 100, 1_500).target < U256::ONE << 226);
    }

    #[test]
    fn huge_positive_shift_saturates_at_limit() {
        let a = anchor();
        assert_eq!(asert(a, 0, u64::MAX / 4).bits, 0x1e10_0000);
        assert_eq!(asert(Anchor { target: U256::ONE, time: 0 }, 0, 600 * 300).target, U256::ONE << 236);
    }
}
