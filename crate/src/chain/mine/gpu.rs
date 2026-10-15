//! The GPU backends: lens's `hemera_mine` kernels (lens `release/0.2`,
//! `gpu/src/mine.rs`) on Metal (Apple Silicon, unified memory) and WebGPU
//! (wgpu: Vulkan, DX12, Metal, GL). Both answer [`super::Backend::search`]
//! with the least passing index of one dispatch, the same answer as the
//! CPU; the kernels are differential-tested in lens against its CPU
//! reference and `hemera::hash`.

use super::job::Job;

fn lens_job(job: &Job) -> cyber_lens_gpu::mine::Job {
    cyber_lens_gpu::mine::Job {
        init: job.init,
        blocks: job.blocks,
        tail_lane: job.tail_lane,
        tail: job.tail,
        target: job.target,
    }
}

/// Nonces a GPU dispatch (2^22 ≈ 0.3 s on an M4 Max at 13 MH/s).
const DISPATCH: u32 = 1 << 22;

#[cfg(all(feature = "metal", target_os = "macos"))]
pub struct Metal {
    inner: cyber_lens_gpu::metal::Metal,
}

#[cfg(all(feature = "metal", target_os = "macos"))]
impl Metal {
    /// The default Metal device, or why not.
    pub fn new() -> Result<Self, String> {
        cyber_lens_gpu::metal::Metal::new().map(|inner| Metal { inner })
    }
}

#[cfg(all(feature = "metal", target_os = "macos"))]
impl super::Backend for Metal {
    fn name(&self) -> String {
        format!("metal ({})", self.inner.device())
    }

    fn search(&self, job: &Job, base: u128, count: u32) -> Result<Option<u32>, String> {
        self.inner.mine(&lens_job(job), base, count)
    }

    fn dispatch(&self) -> u32 {
        DISPATCH
    }
}

#[cfg(feature = "webgpu")]
pub struct WebGpu {
    inner: cyber_lens_gpu::webgpu::WebGpu,
}

#[cfg(feature = "webgpu")]
impl WebGpu {
    /// The first high-performance adapter with compute shaders, or why not.
    pub fn new() -> Result<Self, String> {
        cyber_lens_gpu::webgpu::WebGpu::new().map(|inner| WebGpu { inner })
    }
}

#[cfg(feature = "webgpu")]
impl super::Backend for WebGpu {
    fn name(&self) -> String {
        format!("webgpu ({})", self.inner.adapter())
    }

    fn search(&self, job: &Job, base: u128, count: u32) -> Result<Option<u32>, String> {
        self.inner.mine(&lens_job(job), base, count)
    }

    fn dispatch(&self) -> u32 {
        DISPATCH
    }
}
