//! Consensus helpers the miner needs before the chain kernel lands —
//! TO BE REMOVED when `feat/chain-kernel` merges: `header.rs` (bits codec,
//! block id), `pow.rs` (`valid_pow`) and `daa.rs` (ASERT) own these; the
//! miner then depends on them. Every function here is pinned to the vectors
//! of `specs/chain-conformance.md` §2–§5 so the two copies cannot drift
//! silently (tests below). Integer only: no float anywhere (chain.md §1.5).

/// A 256-bit unsigned integer as four little-endian 64-bit words (word 3
/// most significant) — the order `valid_pow` compares in (chain.md §1.2).
pub type U256 = [u64; 4];

/// chain.md §0.
pub const HEADER_BYTES: usize = 168;
pub const NONCE_OFFSET: usize = 24;
pub const BITS_OFFSET: usize = 20;
pub const HEIGHT_OFFSET: usize = 4;
pub const PREV_OFFSET: usize = 40;
pub const BLOCK_TIME: i64 = 30;
pub const HALF_LIFE: i64 = 600;
pub const POW_LIMIT_BITS: u32 = 0x1e10_0000;
pub const GENESIS_BITS: u32 = 0x1e01_0000;

/// `2^n` as a [`U256`] (`n < 256`).
pub const fn pow2(n: u32) -> U256 {
    let mut t = [0u64; 4];
    t[(n / 64) as usize] = 1u64 << (n % 64);
    t
}

/// `POW_LIMIT` = 2^236.
pub const POW_LIMIT: U256 = pow2(236);

/// `a < b`.
pub fn below(a: &U256, b: &U256) -> bool {
    for i in (0..4).rev() {
        if a[i] != b[i] {
            return a[i] < b[i];
        }
    }
    false
}

/// The 32 little-endian bytes of a [`U256`] (wire order of a block id).
pub fn to_bytes(t: &U256) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, w) in t.iter().enumerate() {
        out[8 * i..8 * i + 8].copy_from_slice(&w.to_le_bytes());
    }
    out
}

/// A [`U256`] from 32 little-endian bytes.
pub fn from_bytes(b: &[u8; 32]) -> U256 {
    core::array::from_fn(|i| u64::from_le_bytes(b[8 * i..8 * i + 8].try_into().expect("8 bytes")))
}

/// Decode compact `bits` (chain.md §1.3): the target, or `None` when the
/// sign bit is set, the encoding is not canonical, the target is 0 or
/// above `POW_LIMIT`.
pub fn decode_bits(bits: u32) -> Option<U256> {
    if bits & 0x0080_0000 != 0 {
        return None;
    }
    let exp = bits >> 24;
    let mant = (bits & 0x007f_ffff) as u128;
    let target: Wide = if exp >= 3 {
        shl(&from_u128(mant), 8 * (exp - 3))
    } else {
        from_u128(mant >> (8 * (3 - exp)))
    };
    let t = narrow(&target)?;
    if t == [0; 4] || below(&POW_LIMIT, &t) {
        return None;
    }
    (encode_bits(&t) == bits).then_some(t)
}

/// Canonical compact encoding of a target (chain.md §1.3).
pub fn encode_bits(target: &U256) -> u32 {
    let bytes = to_bytes(target);
    let mut size = 32;
    while size > 0 && bytes[size - 1] == 0 {
        size -= 1;
    }
    let mut mant: u32 = if size <= 3 {
        let mut v = 0u32;
        for (i, b) in bytes.iter().enumerate().take(size) {
            v |= (*b as u32) << (8 * i);
        }
        v << (8 * (3 - size))
    } else {
        (bytes[size - 3] as u32)
            | ((bytes[size - 2] as u32) << 8)
            | ((bytes[size - 1] as u32) << 16)
    };
    if mant & 0x0080_0000 != 0 {
        mant >>= 8;
        size += 1;
    }
    ((size as u32) << 24) | mant
}

/// The block id: the flat hemera sponge over the header bytes (§1.1).
pub fn block_id(header: &[u8; HEADER_BYTES]) -> [u8; 32] {
    *hemera::hash(header).as_bytes()
}

/// `u256_le(block_id) < target` (§1.2).
pub fn valid_pow(header: &[u8; HEADER_BYTES], target: &U256) -> bool {
    below(&from_bytes(&block_id(header)), target)
}

// ── ASERT (§1.5), integer only ──

/// The intermediate values of one ASERT step (chain-conformance §4 quotes
/// them).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Asert {
    pub e: i64,
    pub shifts: i64,
    pub frac: u64,
    pub factor: u64,
    pub target: U256,
    pub bits: u32,
}

/// The target of the block after a parent at `parent_height` with time
/// `parent_time`, anchored at `genesis_target` and `genesis_time`.
pub fn asert(
    genesis_target: &U256,
    genesis_time: i64,
    parent_height: u64,
    parent_time: i64,
) -> Asert {
    let dt = parent_time - genesis_time;
    let ideal = parent_height as i64 * BLOCK_TIME;
    let e = ((dt - ideal) * 65536).div_euclid(HALF_LIFE);
    let shifts = e >> 16;
    let frac = (e & 0xffff) as u64;
    let f = frac as u128;
    let factor = 65536
        + ((195_766_423_245_049 * f + 971_821_376 * f * f + 5127 * f * f * f + (1u128 << 47)) >> 48)
            as u64;
    let mut t = mul_u64(&widen(genesis_target), factor);
    t = if shifts < 0 {
        shr(&t, (-shifts).min(1024) as u32)
    } else {
        shl(&t, shifts.min(1024) as u32)
    };
    t = shr(&t, 16);
    let target = match narrow(&t) {
        Some(t) if t == [0; 4] => [1, 0, 0, 0],
        Some(t) if below(&POW_LIMIT, &t) => POW_LIMIT,
        Some(t) => t,
        None => POW_LIMIT,
    };
    Asert {
        e,
        shifts,
        frac,
        factor,
        target,
        bits: encode_bits(&target),
    }
}

// ── a small wide integer: 1024 bits, little-endian words ──

type Wide = [u64; 16];

fn widen(t: &U256) -> Wide {
    let mut w = [0u64; 16];
    w[..4].copy_from_slice(t);
    w
}

fn from_u128(v: u128) -> Wide {
    let mut w = [0u64; 16];
    w[0] = v as u64;
    w[1] = (v >> 64) as u64;
    w
}

/// The low 256 bits, or `None` if anything above is set.
fn narrow(w: &Wide) -> Option<U256> {
    w[4..]
        .iter()
        .all(|&x| x == 0)
        .then(|| [w[0], w[1], w[2], w[3]])
}

fn mul_u64(w: &Wide, m: u64) -> Wide {
    let mut out = [0u64; 16];
    let mut carry = 0u128;
    for i in 0..16 {
        let v = w[i] as u128 * m as u128 + carry;
        out[i] = v as u64;
        carry = v >> 64;
    }
    assert_eq!(carry, 0, "wide overflow");
    out
}

/// Left shift; bits shifted past 1024 are lost only when the value already
/// exceeds every clamp (callers clamp to `POW_LIMIT`), so saturate the top
/// word instead of losing them.
fn shl(w: &Wide, n: u32) -> Wide {
    if n >= 1024 {
        return if w.iter().all(|&x| x == 0) {
            [0; 16]
        } else {
            [u64::MAX; 16]
        };
    }
    let (q, r) = ((n / 64) as usize, n % 64);
    let mut out = [0u64; 16];
    let mut lost = false;
    for i in (0..16).rev() {
        let mut v = 0u64;
        if i >= q {
            v = w[i - q] << r;
            if r > 0 && i > q {
                v |= w[i - q - 1] >> (64 - r);
            }
        }
        out[i] = v;
    }
    for x in &w[16 - q..] {
        lost |= *x != 0;
    }
    if r > 0 && w[15 - q] >> (64 - r) != 0 {
        lost = true;
    }
    if lost {
        [u64::MAX; 16]
    } else {
        out
    }
}

fn shr(w: &Wide, n: u32) -> Wide {
    if n >= 1024 {
        return [0; 16];
    }
    let (q, r) = ((n / 64) as usize, n % 64);
    let mut out = [0u64; 16];
    for i in 0..16 {
        if i + q < 16 {
            let mut v = w[i + q] >> r;
            if r > 0 && i + q + 1 < 16 {
                v |= w[i + q + 1] << (64 - r);
            }
            out[i] = v;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_bits_vectors() {
        // chain-conformance §2
        assert_eq!(decode_bits(0x1e01_0000), Some(pow2(232)));
        assert_eq!(decode_bits(0x1e10_0000), Some(pow2(236)));
        assert_eq!(decode_bits(0x1d10_0000), Some(pow2(228)));
        assert_eq!(decode_bits(0x1d20_0000), Some(pow2(229)));
        assert_eq!(decode_bits(0x1d08_0000), Some(pow2(227)));
        let mut t = [0u64; 4];
        // 0x084860 · 256^26 = 0x084860 << 208
        t[3] = 0x0008_4860_u64 << 16;
        assert_eq!(decode_bits(0x1d08_4860), Some(t));
        assert_eq!(decode_bits(0x0312_3456), Some([0x12_3456, 0, 0, 0]));
        for bad in [
            0x1e00_1000u32,
            0x0400_1234,
            0x0100_3456,
            0x1d80_0000,
            0,
            0x1f01_0000,
        ] {
            assert_eq!(decode_bits(bad), None, "{bad:#x}");
        }
        assert_eq!(encode_bits(&pow2(228)), 0x1d10_0000);
        assert_eq!(encode_bits(&[0x12_3400, 0, 0, 0]), 0x0312_3400);
    }

    /// chain-conformance §5: the test header and the mined one.
    pub(crate) fn vector_header(bits: u32, time: u64, nonce: u128) -> [u8; 168] {
        let mut h = [0u8; 168];
        h[0..4].copy_from_slice(&1u32.to_le_bytes());
        h[4..12].copy_from_slice(&1u64.to_le_bytes());
        h[12..20].copy_from_slice(&time.to_le_bytes());
        h[20..24].copy_from_slice(&bits.to_le_bytes());
        h[24..40].copy_from_slice(&nonce.to_le_bytes());
        h[40..72].fill(0x11);
        h[72..104].fill(0x22);
        h[104..136].fill(0x33);
        h[136..168].fill(0x44);
        h
    }

    #[test]
    fn header_vectors() {
        let h = vector_header(0x1d10_0000, 1_762_349_000, 0);
        let hex: String = block_id(&h).iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            hex,
            "bdfbbff94e6e17eec9169932276825a0f4957f6f3a6af9030439979eacaeb58d"
        );
        assert!(!valid_pow(&h, &pow2(228)));
        let h = vector_header(0x1e01_0000, 1_793_885_850, 40_165_933);
        let hex: String = block_id(&h).iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            hex,
            "d1eec40d1db983bb3df0946890538bec4ba9712fb26b82316efc6b162f000000"
        );
        assert!(valid_pow(&h, &pow2(232)));
        assert!(!valid_pow(&h, &pow2(228)));
        assert!(!valid_pow(
            &vector_header(0x1e01_0000, 1_793_885_850, 40_165_932),
            &pow2(232)
        ));
    }

    /// chain-conformance §4, behavioral: a chain at T-spacing holds bits;
    /// T/2 blocks for 20 blocks halve the target (factor in [1.9, 2.1])
    /// after 600 s of block time; a 100× slowdown recovers the T interval
    /// within 70 minutes of simulated wall time. Hash rate is simulated
    /// deterministically: a block takes `expected_hashes / rate` seconds
    /// (integer seconds, rounded to nearest), where
    /// `expected_hashes = 2^256/(target+1)`; no randomness, no floats.
    fn simulate(genesis_target: &U256, steps: &[(u64, u64)]) -> Vec<(u64, i64, U256)> {
        // steps: (blocks, hash rate in H/s) phases; returns (height, time, target of the next block)
        let mut out = Vec::new();
        let (mut h, mut t) = (0u64, 0i64);
        let mut target = *genesis_target;
        for &(blocks, rate) in steps {
            for _ in 0..blocks {
                // seconds a block at `target` and `rate`: 2^256 / (target+1) / rate, nearest
                let exp = expected_hashes(&target);
                let secs = ((exp + rate as u128 / 2) / rate as u128) as i64;
                t += secs.max(1);
                h += 1;
                let a = asert(genesis_target, 0, h, t);
                target = a.target;
                out.push((h, t, target));
            }
        }
        out
    }

    /// `2^256 / (target + 1)` for targets of the form `2^k` or close: via
    /// the leading 128 bits (test accuracy, not consensus).
    fn expected_hashes(target: &U256) -> u128 {
        // work ≈ 2^256 / target: shift so the target fits 64 bits
        let lz = {
            let mut z = 0;
            for w in target.iter().rev() {
                if *w == 0 {
                    z += 64
                } else {
                    z += w.leading_zeros();
                    break;
                }
            }
            z
        };
        let bits = 256 - lz; // target in [2^(bits-1), 2^bits)
        let top = if bits > 64 {
            let w = widen(target);
            let s = shr(&w, bits - 64);
            s[0]
        } else {
            target[0]
        };
        // 2^256/target = 2^(256-(bits-64)) / top = 2^(320-bits)/top
        let num_shift = 320 - bits;
        if num_shift >= 128 {
            ((1u128 << 127) / top as u128) << (num_shift - 127)
        } else {
            (1u128 << num_shift) / top as u128
        }
    }

    #[test]
    fn asert_behaviour_and_hundredfold_step() {
        let g = pow2(228);
        let base_rate: u64 = (expected_hashes(&g) / 30) as u64; // exactly T-spacing at 2^228
                                                                // 1. T-spacing holds bits
        let steady = simulate(&g, &[(200, base_rate)]);
        for (h, _, target) in &steady {
            assert_eq!(encode_bits(target), 0x1d10_0000, "height {h}");
        }
        // 2. blocks at T/2 spacing: after 600 s of block time (40 blocks at
        //    15 s, the chain 600 s = one half-life ahead) the target has
        //    halved — factor in [1.9, 2.1]
        let a = asert(&g, 0, 40, 600);
        let ratio_x10 = expected_hashes(&a.target) * 10 / expected_hashes(&g);
        assert!(
            (19..=21).contains(&ratio_x10),
            "work ratio ×10 = {ratio_x10}"
        );
        assert_eq!(a.bits, 0x1d08_0000);
        // 3. a 100× step: how long until the interval is back near T, and
        //    after the farm leaves (÷100), how long until blocks flow again.
        //    Lead grows at most 30 s a block, absorbing 100× needs
        //    log2(100) half-lives ≈ 3,986 s of lead, so > 133 blocks by
        //    construction; the collapse recovers in H·log2(100) ≈ 66 min
        //    of wall time whether or not blocks arrive (chain.md §1.5).
        let run = simulate(&g, &[(1_000, 100 * base_rate)]);
        let settled = run
            .windows(2)
            .position(|w| w[1].1 - w[0].1 >= 27)
            .expect("settles");
        let (h_set, t_set, _) = run[settled + 1];
        eprintln!(
            "asert 100x step: interval ≥ 27 s from block {h_set}, after {t_set}s wall ({} min)",
            t_set / 60
        );
        assert!(
            h_set <= 300 && t_set <= 3 * 3600,
            "100x absorbed in {h_set} blocks / {t_set}s"
        );
        let tail = run[999].1 - run[998].1;
        assert!(
            (27..=33).contains(&tail),
            "interval after 1000 blocks at 100x: {tail}s"
        );
        // the collapse: steady at 100× (continue from the settled state), then ÷100
        let both = simulate(&g, &[(1_000, 100 * base_rate), (200, base_rate)]);
        let t_leave = both[999].1;
        let first_slow = both[1000].1 - both[999].1;
        let back = both[1000..]
            .windows(2)
            .find(|w| w[1].1 - w[0].1 <= 60)
            .map(|w| w[1].1 - t_leave)
            .expect("recovers");
        eprintln!("asert 100x collapse: first block {first_slow}s ({} min), blocks ≤ 60 s again after {back}s ({} min)", first_slow / 60, back / 60);
        assert!(back <= 70 * 60, "recovery took {back}s");
        // 4. the same step from the real genesis target 2^232
        let g2 = pow2(232);
        let r2: u64 = (expected_hashes(&g2) / 30) as u64;
        let run2 = simulate(&g2, &[(20, r2), (1_000, 100 * r2)]);
        let first_fast = run2[20].1 - run2[19].1;
        let settled2 = run2[20..]
            .windows(2)
            .position(|w| w[1].1 - w[0].1 >= 27)
            .expect("settles");
        let (h2, t2, _) = run2[20 + settled2 + 1];
        let last = run2.last().unwrap();
        eprintln!(
            "asert 100x step @2^232: first block {first_fast}s, interval ≥ 27 s from block {h2} ({}s wall), bits after 1000 blocks {:#x}",
            t2 - run2[19].1,
            encode_bits(&last.2)
        );
    }

    #[test]
    fn asert_vectors() {
        // chain-conformance §4: anchor 2^228, genesis time 0
        let g = pow2(228);
        let rows: [(u64, i64, i64, i64, u64, u64, u32); 6] = [
            (0, 0, 0, 0, 0, 65536, 0x1d10_0000),
            (19, 570, 0, 0, 0, 65536, 0x1d10_0000),
            (19, 1170, 65536, 1, 0, 65536, 0x1d20_0000),
            (100, 2400, -65536, -1, 0, 65536, 0x1d08_0000),
            (19, 0, -62260, -1, 3276, 67852, 0x1d08_4860),
            (5, 1_000_000, 109_210_282, 1666, 27306, 87473, 0x1e10_0000),
        ];
        for (h, t, e, shifts, frac, factor, bits) in rows {
            let a = asert(&g, 0, h, t);
            assert_eq!(
                (a.e, a.shifts, a.frac, a.factor, a.bits),
                (e, shifts, frac, factor, bits),
                "parent {h} @ {t}"
            );
        }
    }
}
