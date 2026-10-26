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

use std::time::{Duration, Instant};

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
    /// Metal where built, else WebGPU if its adapter answers a bounded
    /// calibration probe correctly and faster than the CPU, else CPU.
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
    #[cfg(all(feature = "metal", target_os = "macos", target_arch = "aarch64"))]
    {
        gpu::Metal::new().map(|m| Box::new(m) as Box<dyn Backend>)
    }
    #[cfg(not(all(feature = "metal", target_os = "macos", target_arch = "aarch64")))]
    {
        Err("metal: not built (feature `metal`, Apple Silicon macOS)".into())
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
        Choice::Auto => {
            if let Ok(m) = metal() {
                return Ok(m);
            }
            let cpu = cpu::Cpu::new(threads);
            if let Ok(gpu) = webgpu() {
                let name = gpu.name();
                match probe(gpu, &cpu, PROBE_DEADLINE) {
                    Ok(gpu) => return Ok(gpu),
                    Err(why) => eprintln!("auto: webgpu ({name}) {why}; mining on cpu"),
                }
            }
            Ok(Box::new(cpu))
        }
    }
}

/// Nonces the CPU searches in the calibration probe. The candidate is not
/// probed at this size but at its own `dispatch()`: that is the size the
/// miner will then use, and a stall that depends on dispatch size (the Intel
/// paravirtual adapter answers 2^14 nonces and hangs on 2^22) would pass a
/// small probe.
const PROBE: u32 = 1 << 14;
/// How long `Auto` waits for the probe before giving up on the adapter.
const PROBE_DEADLINE: Duration = Duration::from_secs(5);

/// Hashes per second, a zero duration counting as 1 ns.
fn rate(count: u32, took: Duration) -> f64 {
    count as f64 / took.as_secs_f64().max(1e-9)
}

/// The rule: the candidate (searching `gpu_count >= cpu.1` nonces from the
/// same base) is kept only if it answered in time, answered without error,
/// agreed with the CPU, and hashed strictly faster per second. Both return
/// the least passing index, so a CPU `Some(i)` needs the same `Some(i)`, and
/// a CPU `None` needs `None` or an index beyond the CPU's range.
/// `Err` is the reason, phrased to follow the device name.
fn prefer_gpu(
    gpu_count: u32,
    gpu: Option<(Result<Option<u32>, String>, Duration)>,
    cpu: (Option<u32>, u32, Duration),
) -> Result<(), String> {
    let Some((answer, took)) = gpu else {
        return Err(format!(
            "did not answer a {gpu_count}-nonce probe within {} s",
            PROBE_DEADLINE.as_secs()
        ));
    };
    let found = answer.map_err(|e| format!("failed the probe ({e})"))?;
    let agrees = match cpu.0 {
        Some(i) => found == Some(i),
        None => found.is_none_or(|j| j >= cpu.1),
    };
    if !agrees {
        return Err("disagreed with cpu".into());
    }
    let (g, c) = (rate(gpu_count, took), rate(cpu.1, cpu.2));
    if g <= c {
        return Err(format!("slower than cpu ({g:.0} vs {c:.0} H/s)"));
    }
    Ok(())
}

/// Time `gpu` at its own dispatch size and `cpu` on [`PROBE`] nonces of one
/// fixed job and keep `gpu` only if [`prefer_gpu`] accepts it. The candidate
/// runs on its own thread so a driver that never returns costs `deadline`,
/// not the process; on timeout that thread is left detached, possibly stuck
/// in the driver for good, and owns the backend until it ends.
fn probe(
    gpu: Box<dyn Backend>,
    cpu: &dyn Backend,
    deadline: Duration,
) -> Result<Box<dyn Backend>, String> {
    let job = Job::new(&[0x5au8; 168], target::pow2(246));
    let count = gpu.dispatch();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let t = Instant::now();
        let r = gpu.search(&job, 0, count);
        // the receiver is gone after a timeout; nothing to tell
        let _ = tx.send((gpu, r, t.elapsed()));
    });
    let t = Instant::now();
    let want = cpu.search(&job, 0, PROBE)?;
    let cpu_row = (want, PROBE, t.elapsed());
    match rx.recv_timeout(deadline).ok() {
        Some((gpu, r, took)) => prefer_gpu(count, Some((r, took)), cpu_row).map(|()| gpu),
        None => Err(prefer_gpu(count, None, cpu_row).unwrap_err()),
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

    /// A candidate that answers `answer` after sleeping `nap`.
    struct Fake {
        nap: Duration,
        answer: Result<Option<u32>, String>,
    }

    impl Backend for Fake {
        fn name(&self) -> String {
            "fake".into()
        }
        fn search(&self, _: &Job, _: u128, _: u32) -> Result<Option<u32>, String> {
            std::thread::sleep(self.nap);
            self.answer.clone()
        }
        fn dispatch(&self) -> u32 {
            PROBE
        }
    }

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    /// The probe job's true answer, from the CPU.
    fn truth(cpu: &cpu::Cpu) -> Option<u32> {
        let job = Job::new(&[0x5au8; 168], pow2(246));
        cpu.search(&job, 0, PROBE).expect("cpu")
    }

    #[test]
    fn probe_gives_up_on_a_device_that_never_answers() {
        let cpu = cpu::Cpu::new(2);
        let slow = Fake {
            nap: Duration::from_secs(2),
            answer: Ok(truth(&cpu)),
        };
        let t = Instant::now();
        assert!(probe(Box::new(slow), &cpu, ms(200)).is_err());
        assert!(t.elapsed() < Duration::from_secs(2), "returned at the deadline");
    }

    #[test]
    fn probe_rejects_a_wrong_index() {
        let cpu = cpu::Cpu::new(2);
        let wrong = Fake {
            nap: Duration::ZERO,
            answer: Ok(truth(&cpu).map_or(Some(0), |i| Some(i + 1))),
        };
        assert!(probe(Box::new(wrong), &cpu, Duration::from_secs(5)).is_err());
    }

    #[test]
    fn probe_rejects_an_error() {
        let cpu = cpu::Cpu::new(2);
        let bad = Fake {
            nap: Duration::ZERO,
            answer: Err("device lost".into()),
        };
        assert!(probe(Box::new(bad), &cpu, Duration::from_secs(5)).is_err());
    }

    #[test]
    fn prefer_gpu_decides_all_outcomes() {
        let cpu = (Some(7), PROBE, ms(100));
        let none = (None, PROBE, ms(100));
        let g = |a: Result<Option<u32>, String>, n: u32, t: u64| {
            prefer_gpu(n, Some((a, ms(t))), cpu)
        };
        assert!(prefer_gpu(PROBE, None, cpu).unwrap_err().contains("-nonce probe within"));
        assert!(g(Err("x".into()), PROBE, 1).unwrap_err().contains("failed"));
        assert!(g(Ok(Some(8)), PROBE, 1).unwrap_err().contains("disagreed"));
        assert!(g(Ok(None), 4 * PROBE, 1).unwrap_err().contains("disagreed"));
        // equal rate is not faster
        assert!(g(Ok(Some(7)), PROBE, 100).unwrap_err().contains("slower"));
        assert_eq!(g(Ok(Some(7)), PROBE, 99), Ok(()));
        // a larger dispatch in a longer time can still be faster by rate
        assert_eq!(g(Ok(Some(7)), 4 * PROBE, 200), Ok(()));
        // and a smaller rate loses even with a larger count
        assert!(g(Ok(Some(7)), 2 * PROBE, 400).unwrap_err().contains("H/s"));
        // CPU found nothing in its range: GPU may find nothing or go beyond it
        let g = |a: Result<Option<u32>, String>| prefer_gpu(4 * PROBE, Some((a, ms(10))), none);
        assert_eq!(g(Ok(None)), Ok(()));
        assert_eq!(g(Ok(Some(PROBE))), Ok(()));
        assert!(g(Ok(Some(PROBE - 1))).unwrap_err().contains("disagreed"));
    }

    /// Every built backend returns the same index for the same dispatch,
    /// and its nonce verifies with `hemera::hash` — chain-conformance §5's
    /// mined header is one of the jobs (the known nonce 934,636).
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
                934_000u128,
                2_000u32,
                Some(636u32),
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
