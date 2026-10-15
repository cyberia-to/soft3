//! The CPU backend: hemera's interleaved batch permutation
//! (`permute_batch`, 16 states a kernel call on aarch64, one shared
//! inversion per partial round) over contiguous nonce ranges, one range a
//! thread.
//!
//! `search(job, base, count)` returns the least passing index in the
//! dispatch, like the GPU backends: each thread owns a contiguous slice of
//! `base..base+count`, scans it in batches of [`BATCH`], and stops early
//! once a lower hit than its own position is known.

use super::job::{Job, absorb, passes, seed};
use hemera::field::Goldilocks;
use hemera::permutation::permute_batch;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

/// States a batch (a multiple of hemera's 16-lane kernel width).
pub const BATCH: usize = 256;

/// The CPU backend with a thread count.
#[derive(Clone, Copy, Debug)]
pub struct Cpu {
    pub threads: usize,
}

impl Cpu {
    /// `threads` workers (0: every core).
    pub fn new(threads: usize) -> Self {
        let t = if threads == 0 {
            std::thread::available_parallelism().map_or(1, |n| n.get())
        } else {
            threads
        };
        Cpu { threads: t }
    }
}

/// The digests of nonces `base + i` for `i < n` scanned in one batch:
/// the least passing `i`, if any.
fn scan(job: &Job, base: u128, n: usize) -> Option<usize> {
    let mut st: Vec<[Goldilocks; 16]> = (0..n as u128).map(|i| seed(job, base + i)).collect();
    permute_batch(&mut st);
    for b in 1..4 {
        for s in st.iter_mut() {
            absorb(job, s, b);
        }
        permute_batch(&mut st);
    }
    st.iter().position(|s| passes(job, s))
}

impl super::Backend for Cpu {
    fn name(&self) -> String {
        format!("cpu ({} threads)", self.threads)
    }

    fn search(&self, job: &Job, base: u128, count: u32) -> Result<Option<u32>, String> {
        let count = count as usize;
        if count == 0 {
            return Ok(None);
        }
        let best = AtomicU32::new(u32::MAX);
        let per = count.div_ceil(self.threads).max(BATCH);
        std::thread::scope(|s| {
            for t in 0..self.threads {
                let from = t * per;
                if from >= count {
                    break;
                }
                let to = (from + per).min(count);
                let best = &best;
                s.spawn(move || {
                    let mut at = from;
                    while at < to && (at as u32) < best.load(Relaxed) {
                        let n = BATCH.min(to - at);
                        if let Some(i) = scan(job, base + at as u128, n) {
                            best.fetch_min((at + i) as u32, Relaxed);
                            return;
                        }
                        at += n;
                    }
                });
            }
        });
        let b = best.load(Relaxed);
        Ok((b != u32::MAX).then_some(b))
    }

    /// Nonces a dispatch: a few batches a thread (~50 ms at 3 MH/s on 16
    /// threads), so hashrate and tip checks stay responsive.
    fn dispatch(&self) -> u32 {
        (self.threads * BATCH * 64) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::mine::Backend;
    use crate::chain::mine::local::{pow2, valid_pow};

    #[test]
    fn cpu_finds_the_least_passing_nonce_across_threads() {
        let header = [0x5au8; 168];
        // 2^244: one in 4096
        let job = Job::new(&header, pow2(244));
        let cpu = Cpu::new(4);
        let base = (7u128 << 64) + 12_345;
        let i = cpu.search(&job, base, 1 << 15).expect("ok").expect("a nonce");
        let one = Cpu::new(1);
        assert_eq!(one.search(&job, base, 1 << 15).expect("ok"), Some(i));
        assert!(valid_pow(&job.header_with(base + i as u128), &job.target));
        for k in 0..i {
            assert!(!valid_pow(&job.header_with(base + k as u128), &job.target), "{k}");
        }
    }

    #[test]
    fn cpu_reports_none() {
        let job = Job::new(&[0x11u8; 168], [1, 0, 0, 0]);
        assert_eq!(Cpu::new(2).search(&job, 0, 4096).expect("ok"), None);
    }
}
