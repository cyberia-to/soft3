//! The stack's normative surfaces, one module per owning component.
//!
//! Each surface turns fixed inputs into canonical bytes and records the
//! hemera fingerprint of those bytes as an encoding or mechanism entry.
//! Inputs are either literal here, committed under `conformance/fixtures/`,
//! or read from the owning sibling (hemera's published vectors, nox's jet
//! formulas) so that a change to them is itself drift.

use std::path::{Path, PathBuf};

use crate::snap::Snapshot;
use crate::{EncodingSnapshot, Fingerprint, MechanismSnapshot, Tier};

pub mod cybergraph;
pub mod hemera;
pub mod nebu;
pub mod nox;
pub mod tade;
pub mod zheng;

/// Every entry starts enforced. Promotion to delta/epsilon is a later,
/// per-entry ceremony (specs/README.md, tiers).
pub const TIER: Tier = Tier::Gamma;

/// Where fixture-backed surfaces read their committed proof fixtures from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fixtures {
    /// `--check`: verify the committed fixtures as they are on disk
    Committed,
    /// `--bless`: verify the freshly produced fixtures (which bless writes)
    Fresh,
}

/// Inputs of one harness run.
#[derive(Debug, Clone)]
pub struct Context {
    /// directory holding the sibling checkouts (`hemera/`, `nox/`, …, `soft3/`)
    pub stack: PathBuf,
    /// `soft3/conformance`
    pub conformance: PathBuf,
    /// which proof fixtures the verify mechanisms read
    pub fixtures: Fixtures,
}

impl Context {
    /// Default layout: this crate is `<stack>/soft3/conformance/rs`.
    pub fn from_manifest_dir(fixtures: Fixtures) -> Self {
        let rs = Path::new(env!("CARGO_MANIFEST_DIR"));
        let conformance = rs.parent().expect("conformance dir").to_path_buf();
        let stack = conformance.parent().and_then(Path::parent).expect("stack dir").to_path_buf();
        Context { stack, conformance, fixtures }
    }

    /// `conformance/fixtures/<rel>`
    pub fn fixture(&self, rel: &str) -> PathBuf {
        self.conformance.join("fixtures").join(rel)
    }
}

/// Accumulates entries, fixtures to write, and hard failures.
#[derive(Debug, Default)]
pub struct Out {
    /// entries produced so far
    pub snapshot: Snapshot,
    /// fixtures produced on this run: path relative to `conformance/fixtures`, bytes
    pub fixtures: Vec<(String, Vec<u8>)>,
    /// invariant violations that fail the run regardless of snapshots
    pub failures: Vec<String>,
}

impl Out {
    /// Record an encoding entry over canonical bytes.
    pub fn encoding(&mut self, name: &str, bytes: &[u8]) {
        self.snapshot.encodings.push(EncodingSnapshot {
            name: name.to_string(),
            tier: TIER,
            fingerprint: Fingerprint::of(bytes),
        });
    }

    /// Record a mechanism entry over the output bytes of `scenario`.
    pub fn mechanism(&mut self, mechanism: &str, scenario: &str, bytes: &[u8]) {
        debug_assert!(!scenario.contains('"'));
        self.snapshot.mechanisms.push(MechanismSnapshot {
            mechanism: mechanism.to_string(),
            scenario: scenario.to_string(),
            tier: TIER,
            fingerprint: Fingerprint::of(bytes),
        });
    }

    /// Record an invariant violation.
    pub fn fail(&mut self, msg: impl Into<String>) {
        self.failures.push(msg.into());
    }
}

/// Run every surface.
pub fn generate(ctx: &Context) -> Out {
    let mut out = Out::default();
    hemera::run(ctx, &mut out);
    nebu::run(ctx, &mut out);
    nox::run(ctx, &mut out);
    zheng::run(ctx, &mut out);
    tade::run(ctx, &mut out);
    cybergraph::run(ctx, &mut out);
    for k in out.snapshot.duplicate_keys() {
        out.fail(format!("duplicate snapshot key {k}"));
    }
    out
}

/// Little-endian byte writer for canonical outputs.
#[derive(Debug, Default, Clone)]
pub struct Bytes(pub Vec<u8>);

impl Bytes {
    /// empty
    pub fn new() -> Self {
        Self::default()
    }
    /// one byte
    pub fn u8(&mut self, v: u8) -> &mut Self {
        self.0.push(v);
        self
    }
    /// u32 little-endian
    pub fn u32(&mut self, v: u32) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    /// u64 little-endian
    pub fn u64(&mut self, v: u64) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    /// raw bytes, no prefix
    pub fn raw(&mut self, b: &[u8]) -> &mut Self {
        self.0.extend_from_slice(b);
        self
    }
    /// u64 length prefix then bytes
    pub fn lv(&mut self, b: &[u8]) -> &mut Self {
        self.u64(b.len() as u64).raw(b)
    }
}

/// The 11-element Goldilocks edge set every arithmetic surface runs over:
/// zero, one, small values, the 32-bit boundary, 2^63 and the top of the field.
pub const EDGES: [u64; 11] = [
    0,
    1,
    2,
    7,
    0xFFFF_FFFF,
    0x1_0000_0000,
    0x1_0000_0001,
    1 << 63,
    0xFFFF_FFFF_0000_0000 - 1, // p - 2
    0xFFFF_FFFF_0000_0000,     // p - 1
    0x1234_5678_9ABC_DEF0,
];

/// Deterministic integer ramp (splitmix64) reduced below p — no floats, no
/// platform randomness.
pub fn ramp(seed: u64, n: usize) -> Vec<u64> {
    let mut x = seed;
    (0..n)
        .map(|_| {
            x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = x;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^= z >> 31;
            z % ::nebu::field::P
        })
        .collect()
}

/// Sorted file names with `ext` in `dir`.
pub fn list(dir: &Path, ext: &str) -> Result<Vec<String>, String> {
    let rd = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut names: Vec<String> = rd
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(ext))
        .collect();
    names.sort();
    Ok(names)
}
