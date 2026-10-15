//! `soft3 mine` — the reference miner of the phase-1 root chain
//! (specs/chain.md §1.1–1.3, §8; chain-wire.md §7).
//!
//! The miner takes a header template from a node, grinds the 16-byte
//! nonce over `hemera(header) < target` on one backend, and submits the
//! solved header. Three backends answer one question identically
//! ([`Backend::search`]): the CPU through hemera's batch permutation, Metal
//! and WebGPU through lens's `hemera_mine` kernels — a nonce found by any
//! of them verifies natively ([`target::verifies`]), which the tests pin
//! per backend. `bench` measures them.
//!
//! Consensus questions go to the kernel (`chain::{header, pow, daa}`)
//! through [`target`]; [`asert_sim`] is the 100× step audit.

pub mod api;
mod asert_sim;
pub mod bench;
pub mod cpu;
#[cfg(any(feature = "metal", feature = "webgpu"))]
pub mod gpu;
pub mod job;
pub mod run;
pub mod target;

pub use job::Job;

/// A device that searches nonce ranges.
pub trait Backend: Send + Sync {
    /// The backend and its device.
    fn name(&self) -> String;
    /// The least `i < count` whose nonce `base + i` hashes below the job's
    /// target, or `None`. `base + count` never crosses `2^64` (the caller
    /// splits ranges), so kernels add `i` to the low word only.
    fn search(&self, job: &Job, base: u128, count: u32) -> Result<Option<u32>, String>;
    /// Nonces a dispatch the backend is comfortable with.
    fn dispatch(&self) -> u32;
}

/// `--backend`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    /// Metal where available, else WebGPU if an adapter exists, else CPU.
    Auto,
    Cpu,
    Metal,
    WebGpu,
}

impl core::str::FromStr for Choice {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "auto" => Ok(Choice::Auto),
            "cpu" => Ok(Choice::Cpu),
            "metal" => Ok(Choice::Metal),
            "webgpu" => Ok(Choice::WebGpu),
            _ => Err(format!("unknown backend `{s}` (auto, cpu, metal, webgpu)")),
        }
    }
}

/// The Metal backend, or why not.
pub fn metal() -> Result<Box<dyn Backend>, String> {
    #[cfg(all(feature = "metal", target_os = "macos"))]
    {
        gpu::Metal::new().map(|m| Box::new(m) as Box<dyn Backend>)
    }
    #[cfg(not(all(feature = "metal", target_os = "macos")))]
    {
        Err("metal: not built (feature `metal`, macOS)".into())
    }
}

/// The WebGPU backend, or why not.
pub fn webgpu() -> Result<Box<dyn Backend>, String> {
    #[cfg(feature = "webgpu")]
    {
        gpu::WebGpu::new().map(|w| Box::new(w) as Box<dyn Backend>)
    }
    #[cfg(not(feature = "webgpu"))]
    {
        Err("webgpu: not built (feature `webgpu`)".into())
    }
}

/// The backend of a choice; `threads` is the CPU's.
pub fn select(choice: Choice, threads: usize) -> Result<Box<dyn Backend>, String> {
    match choice {
        Choice::Cpu => Ok(Box::new(cpu::Cpu::new(threads))),
        Choice::Metal => metal(),
        Choice::WebGpu => webgpu(),
        Choice::Auto => Ok(metal()
            .or_else(|_| webgpu())
            .unwrap_or_else(|_| Box::new(cpu::Cpu::new(threads)))),
    }
}

/// Every backend that opens, for `bench` and the cross-backend tests.
pub fn all(threads: usize) -> Vec<Box<dyn Backend>> {
    let mut v: Vec<Box<dyn Backend>> = vec![Box::new(cpu::Cpu::new(threads))];
    for b in [metal(), webgpu()] {
        match b {
            Ok(b) => v.push(b),
            Err(e) => eprintln!("{e}"),
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use target::{pow2, verifies};

    /// Every built backend returns the same index for the same dispatch,
    /// and its nonce verifies with `hemera::hash` — chain-conformance §5's
    /// mined header is one of the jobs (the known nonce 40,165,933).
    #[test]
    fn every_backend_agrees_and_verifies() {
        let backends = all(4);
        // the conformance vector at GENESIS_BITS: base just below the known nonce
        let mut vector = [0u8; 168];
        vector[0..4].copy_from_slice(&1u32.to_le_bytes());
        vector[4..12].copy_from_slice(&1u64.to_le_bytes());
        vector[12..20].copy_from_slice(&1_793_885_850u64.to_le_bytes());
        vector[20..24].copy_from_slice(&0x1e01_0000u32.to_le_bytes());
        vector[40..72].fill(0x11);
        vector[72..104].fill(0x22);
        vector[104..136].fill(0x33);
        vector[136..168].fill(0x44);
        let jobs = [
            (
                Job::new(&vector, pow2(232)),
                40_165_000u128,
                2_000u32,
                Some(933u32),
            ),
            (
                Job::new(&[0x3cu8; 168], pow2(246)),
                (5u128 << 64) + 100,
                1 << 14,
                None,
            ),
            // just below a 2^64 boundary without crossing it (the caller's contract)
            (
                Job::new(&[0x3cu8; 168], pow2(246)),
                ((9u128 << 64) | (u64::MAX as u128)) - 20_000,
                1 << 14,
                None,
            ),
            (
                Job::new(&[0x99u8; 168], [1, 0, 0, 0]),
                0,
                4096,
                Some(u32::MAX),
            ),
        ];
        for (job, base, count, known) in jobs {
            let want = backends[0].search(&job, base, count).expect("cpu");
            if let Some(k) = known {
                assert_eq!(want, (k != u32::MAX).then_some(k), "cpu vs known");
            } else {
                assert!(want.is_some(), "a nonce in the range");
            }
            if let Some(i) = want {
                assert!(verifies(&job.header_with(base + i as u128), &job.target));
            }
            for b in &backends[1..] {
                assert_eq!(
                    b.search(&job, base, count).expect("gpu"),
                    want,
                    "{} base {base:#x}",
                    b.name()
                );
            }
        }
    }
}
