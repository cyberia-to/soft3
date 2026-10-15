//! A mining job: the 168-byte header as hemera absorbs it, and the one
//! question every backend answers the same way.
//!
//! `hemera::hash` over 168 bytes is four permutations of a width-16
//! Goldilocks sponge: three 56-byte rate blocks (eight 7-byte limbs each)
//! added lane-wise into the state, then the padded block `0x01 ‖ 0…` with
//! the message length written to capacity lane 10. The nonce occupies bytes
//! 24..40 of the first block and therefore limbs 3, 4 and 5 of it; because
//! the limbs are 7-byte values the nonce enters as three integer additions
//! on disjoint bit ranges (`lens::gpu::mine` documents the table). A
//! [`Job`] has those three ranges zeroed in `blocks[0]`; [`seed`] adds a
//! nonce back.
//!
//! The question (every backend, [`super::Backend::search`]): for a job, a
//! first nonce `base` and a `count`, the least `i < count` such that the
//! digest of nonce `base + i` is below the target as a 256-bit
//! little-endian integer — or none. Any backend's answer verifies with
//! [`super::local::valid_pow`] on the header it came from, which is what
//! the tests pin.

use super::local::{BITS_OFFSET, HEADER_BYTES, NONCE_OFFSET, U256, below};
use hemera::field::Goldilocks;

/// Hemera's rate: eight limbs a block.
pub const RATE: usize = 8;
/// Bytes a limb.
pub const LIMB_BYTES: usize = 7;
/// The capacity lane hemera writes the message length to before the final
/// permutation (`CAPACITY_START + 2`).
pub const LENGTH_LANE: usize = 10;

/// A header as hemera's sponge sees it, nonce lanes zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Job {
    /// The sponge state before block 0 (hemera's plain mode: all zero).
    pub init: [u64; 16],
    /// The four rate blocks as limbs; `blocks[0]` with the nonce bits zero.
    pub blocks: [[u64; RATE]; 4],
    /// Lane written before the final permutation, and its value (168).
    pub tail_lane: usize,
    pub tail: u64,
    /// The target the digest must be strictly below.
    pub target: U256,
    /// The header the job came from (nonce bytes as given), for
    /// [`Job::header_with`].
    pub header: [u8; HEADER_BYTES],
}

fn limb(b: &[u8]) -> u64 {
    let mut w = [0u8; 8];
    w[..b.len()].copy_from_slice(b);
    u64::from_le_bytes(w)
}

impl Job {
    /// The job of a header at a target.
    pub fn new(header: &[u8; HEADER_BYTES], target: U256) -> Self {
        let mut blocks = [[0u64; RATE]; 4];
        for (b, block) in blocks.iter_mut().enumerate().take(3) {
            for (i, l) in block.iter_mut().enumerate() {
                let at = b * RATE * LIMB_BYTES + i * LIMB_BYTES;
                *l = limb(&header[at..at + LIMB_BYTES]);
            }
        }
        blocks[3][0] = 0x01;
        // the nonce's bit ranges: limb 3 bits 24..56, limb 4 whole, limb 5 bits 0..40
        blocks[0][3] &= 0x00ff_ffff;
        blocks[0][4] = 0;
        blocks[0][5] &= !0xff_ffff_ffff;
        Job {
            init: [0; 16],
            blocks,
            tail_lane: LENGTH_LANE,
            tail: HEADER_BYTES as u64,
            target,
            header: *header,
        }
    }

    /// The job's header with nonce `n` written at bytes 24..40.
    pub fn header_with(&self, n: u128) -> [u8; HEADER_BYTES] {
        let mut h = self.header;
        h[NONCE_OFFSET..NONCE_OFFSET + 16].copy_from_slice(&n.to_le_bytes());
        h
    }

    /// The header's compact bits field.
    pub fn bits(&self) -> u32 {
        u32::from_le_bytes(self.header[BITS_OFFSET..BITS_OFFSET + 4].try_into().expect("4 bytes"))
    }
}

/// The three lane additions of nonce `n`: lanes 3, 4, 5.
pub const fn nonce_lanes(n: u128) -> (u64, u64, u64) {
    (
        ((n as u64) & 0xffff_ffff) << 24,
        ((n >> 32) as u64) & 0x00ff_ffff_ffff_ffff,
        ((n >> 88) as u64) & 0xff_ffff_ffff,
    )
}

/// The permutation input of nonce `n`: block 0 and the nonce lanes added to
/// the initial state (field additions; every addend is below `2^56`).
pub fn seed(job: &Job, n: u128) -> [Goldilocks; 16] {
    let (a3, a4, a5) = nonce_lanes(n);
    let mut s: [Goldilocks; 16] = core::array::from_fn(|i| Goldilocks::new(job.init[i]));
    for (l, b) in s.iter_mut().zip(job.blocks[0]) {
        *l = *l + Goldilocks::new(b);
    }
    s[3] = s[3] + Goldilocks::new(a3);
    s[4] = s[4] + Goldilocks::new(a4);
    s[5] = s[5] + Goldilocks::new(a5);
    s
}

/// Block `b` (1..4) added lane-wise; the length lane set for the last.
pub fn absorb(job: &Job, s: &mut [Goldilocks; 16], b: usize) {
    for (l, v) in s.iter_mut().zip(job.blocks[b]) {
        *l = *l + Goldilocks::new(v);
    }
    if b == 3 {
        s[job.tail_lane] = Goldilocks::new(job.tail);
    }
}

/// The digest limbs of a finished state.
pub fn digest(s: &[Goldilocks; 16]) -> U256 {
    core::array::from_fn(|i| s[i].as_canonical_u64())
}

/// Whether the finished state's digest is below the job's target.
pub fn passes(job: &Job, s: &[Goldilocks; 16]) -> bool {
    below(&digest(s), &job.target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::mine::local::{block_id, from_bytes, pow2};

    #[test]
    fn single_state_sponge_is_hemera_hash() {
        let mut header = [0u8; HEADER_BYTES];
        for (i, b) in header.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(29).wrapping_add(3);
        }
        let job = Job::new(&header, pow2(232));
        for n in [0u128, 7, 1 << 40, (1u128 << 64) + 9, u128::MAX] {
            let mut s = seed(&job, n);
            hemera::permutation::permute(&mut s);
            for b in 1..4 {
                absorb(&job, &mut s, b);
                hemera::permutation::permute(&mut s);
            }
            assert_eq!(digest(&s), from_bytes(&block_id(&job.header_with(n))), "nonce {n}");
        }
    }

    #[test]
    fn header_with_writes_only_the_nonce() {
        let header = [0xabu8; HEADER_BYTES];
        let job = Job::new(&header, pow2(232));
        let h = job.header_with(0x0102_0304_0506_0708_090a_0b0c_0d0e_0f10);
        assert_eq!(&h[..24], &header[..24]);
        assert_eq!(&h[40..], &header[40..]);
        assert_eq!(h[24], 0x10);
        assert_eq!(h[39], 0x01);
        assert_eq!(job.bits(), 0xabab_abab);
    }
}
