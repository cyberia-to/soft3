//! `soft3 mine bench`: hashes per second of every backend that opens, and
//! the conformance vector found by each (chain-conformance §5: the mined
//! header at `GENESIS_BITS`, nonce 40,165,933).

use super::run::{fmt_rate, rate};
use super::target::{target_of, verifies};
use super::{Backend, Job};
use crate::chain::params::GENESIS_BITS;
use std::time::{Duration, Instant};

/// One backend's measurement.
#[derive(Clone, Debug)]
pub struct Row {
    pub backend: String,
    pub hashes: u64,
    pub seconds: f64,
    pub vector: bool,
}

/// The conformance §5 header with a zero nonce.
pub fn vector_header() -> [u8; 168] {
    let mut h = [0u8; 168];
    h[0..4].copy_from_slice(&1u32.to_le_bytes());
    h[4..12].copy_from_slice(&1u64.to_le_bytes());
    h[12..20].copy_from_slice(&1_793_885_850u64.to_le_bytes());
    h[20..24].copy_from_slice(&GENESIS_BITS.to_le_bytes());
    h[40..72].fill(0x11);
    h[72..104].fill(0x22);
    h[104..136].fill(0x33);
    h[136..168].fill(0x44);
    h
}

/// Measure `backend` for about `seconds` on a target nothing passes, then
/// check it finds the vector nonce.
pub fn measure(backend: &dyn Backend, seconds: f64) -> Result<Row, String> {
    let target = target_of(GENESIS_BITS)?;
    let header = vector_header();
    let dark = Job::new(&header, [1, 0, 0, 0]);
    // warm up (shader compile, buffers)
    backend.search(&dark, 1 << 70, backend.dispatch())?;
    let t = Instant::now();
    let mut hashes = 0u64;
    let mut low = 0u64;
    while t.elapsed().as_secs_f64() < seconds {
        let n = backend.dispatch();
        backend.search(&dark, (1u128 << 70) | low as u128, n)?;
        hashes += n as u64;
        low += n as u64;
    }
    let seconds = t.elapsed().as_secs_f64();
    let job = Job::new(&header, target);
    let found = backend.search(&job, 40_165_000, 2_000)?;
    let vector = found == Some(933) && verifies(&job.header_with(40_165_933), &target);
    Ok(Row {
        backend: backend.name(),
        hashes,
        seconds,
        vector,
    })
}

/// The markdown table of rows.
pub fn table(rows: &[Row]) -> String {
    let mut s = String::from(
        "| backend | hashes | seconds | H/s | conformance §5 nonce |\n|---|---|---|---|---|\n",
    );
    for r in rows {
        s.push_str(&format!(
            "| {} | {} | {:.1} | {} | {} |\n",
            r.backend,
            r.hashes,
            r.seconds,
            fmt_rate(rate(r.hashes, Duration::from_secs_f64(r.seconds))),
            if r.vector { "found" } else { "MISSED" }
        ));
    }
    s
}
