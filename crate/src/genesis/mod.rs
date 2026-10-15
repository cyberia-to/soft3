//! Genesis pipeline for the phase-1 PoW root chain: burial snapshot → genesis
//! manifest → bbg state root.
//!
//! Contract: `soft3/specs/genesis.md`. Inputs are the burial snapshots pinned
//! by CID + sha256 in `space-pussy/snapshot/manifest.json` and
//! `bostrom/snapshot/manifest.json`. Output is a directory of canonical TSV
//! tables plus `manifest.json`; `sha256(manifest.json)` is the genesis hash.
//! [`state::load`] turns the tables into a [`bbg::BbgState`] with one root
//! computation at the end (the bulk-build path of launch row 43, not the
//! per-batch recommit of the live node).
//!
//! Every amount is an integer: micro-denominations parsed as `u128`, summed as
//! `u128`, and required to fit `u64` before they reach bbg. No float touches a
//! balance anywhere in this module.

pub mod fetch;
pub mod source;
pub mod state;
pub mod tables;
pub mod time;

#[cfg(test)]
mod tests;

use std::io;
use std::path::Path;

pub use tables::{Account, AccountKind, Balance, Genesis, LinkRow, Manifest, PassportRow, ParticleRow};

/// A bootloader chain whose burial snapshot seeds a phase-1 root chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chain {
    /// Genesis chain name (`pussy` | `bostrom`).
    pub name: &'static str,
    /// bech32 prefix of the legacy accounts.
    pub hrp: &'static str,
    /// Native micro-denomination carried into genesis.
    pub native: &'static str,
    /// The cosmos chain-id of the buried bootloader.
    pub bootloader: &'static str,
    /// Column of `delegations.csv` carrying the integer delegated amount.
    pub delegation_column: &'static str,
}

pub const PUSSY: Chain = Chain {
    name: "pussy",
    hrp: "pussy",
    native: "pussy",
    bootloader: "space-pussy",
    delegation_column: "pussy",
};

pub const BOSTROM: Chain = Chain {
    name: "bostrom",
    hrp: "bostrom",
    native: "boot",
    bootloader: "bostrom",
    delegation_column: "balance_boot",
};

/// TOCYB → CYB at bostrom genesis: every TOCYB becomes 666 CYB
/// (`bostrom/snapshot/manifest.json` → `supply_audit.tocyb_to_cyb.rate`).
pub const TOCYB_RATE: u128 = 666;

impl Chain {
    pub fn parse(name: &str) -> Option<Chain> {
        match name {
            "pussy" | "space-pussy" => Some(PUSSY),
            "bostrom" => Some(BOSTROM),
            _ => None,
        }
    }

    /// Denoms loaded into the bbg balances dimension. Everything else in the
    /// bank is carried in `balances-deferred.tsv` untouched.
    pub fn genesis_denoms(&self) -> &'static [&'static str] {
        match self.name {
            "bostrom" => &["boot", "cyb"],
            _ => &["pussy"],
        }
    }
}

/// What `build` produced, for the CLI report and the tests.
#[derive(Debug, Clone)]
pub struct Report {
    pub chain: &'static str,
    pub genesis_hash: String,
    /// `None` when `--no-root` or when a dimension exceeds the commitment
    /// limit (then `manifest.root_blocked` says which).
    pub root: Option<String>,
    pub manifest: Manifest,
    pub root_seconds: Option<u64>,
}

/// Options for one build.
#[derive(Debug, Clone)]
pub struct Options {
    /// Where the fetched snapshot files live (verified by sha256 on every run).
    pub cache: std::path::PathBuf,
    /// Compute the bbg state root (the expensive step).
    pub root: bool,
    /// Genesis block time for the header stub, unix seconds.
    pub genesis_time: u64,
}

impl Options {
    pub fn default_cache(chain: &Chain) -> std::path::PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        Path::new(&home)
            .join("cyber")
            .join(".cache")
            .join("snapshots")
            .join(chain.name)
    }
}

/// 2026-11-05T00:00:00Z, the launch day.
pub const LAUNCH_UNIX: u64 = time::civil_to_unix(2026, 11, 5, 0, 0, 0);

/// Build the genesis directory for `chain` from the snapshot directory that
/// holds `manifest.json` (fetching the pinned files into the cache as needed).
pub fn build(chain: Chain, snapshot: &Path, out: &Path, options: &Options) -> io::Result<Report> {
    let manifest_path = snapshot.join("manifest.json");
    let sources = fetch::materialize(&chain, &manifest_path, &options.cache)?;
    let snap = source::Snapshot::read(&chain, &manifest_path, &sources)?;
    let genesis = tables::derive(&chain, &snap)?;
    std::fs::create_dir_all(out)?;
    let mut manifest = tables::write(&genesis, out)?;

    let mut root = None;
    let mut root_seconds = None;
    if options.root {
        let started = std::time::Instant::now();
        let bbg = state::load(&genesis);
        manifest.state_counts = Some(state::counts(&bbg));
        manifest.state_fields = Some(state::fields_per_dimension(&bbg));
        let blocked = state::over_limit(&bbg);
        if blocked.is_empty() {
            let r = bbg.root();
            root_seconds = Some(started.elapsed().as_secs());
            let r_hex = tables::hex(&r);
            manifest.state_root = Some(r_hex.clone());
            root = Some(r_hex);
        } else {
            manifest.root_blocked = Some(blocked);
        }
    }
    let genesis_hash = tables::write_manifest(&manifest, out)?;
    tables::write_header(&manifest, &genesis_hash, options.genesis_time, out)?;
    Ok(Report {
        chain: chain.name,
        genesis_hash,
        root,
        manifest,
        root_seconds,
    })
}

/// Re-read a genesis directory, recompute every table hash and the manifest
/// hash, and (optionally) the state root. Returns the recomputed hash and root.
pub fn verify(dir: &Path, root: bool) -> io::Result<(String, Option<String>)> {
    let (manifest, genesis) = tables::read(dir)?;
    let recomputed = tables::table_hashes(dir, &manifest)?;
    for (name, want) in &manifest.tables {
        let got = recomputed
            .get(name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, format!("missing table {name}")))?;
        if got != &want.sha256 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{name}: sha256 {got} != manifest {}", want.sha256),
            ));
        }
    }
    let bytes = std::fs::read(dir.join("manifest.json"))?;
    let hash = tables::sha256_hex(&bytes);
    let recorded = std::fs::read_to_string(dir.join("genesis.hash"))?;
    if recorded.trim() != hash {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("genesis.hash {} != sha256(manifest.json) {hash}", recorded.trim()),
        ));
    }
    let root_hex = if root {
        let bbg = state::load(&genesis);
        let r = tables::hex(&bbg.root());
        if let Some(want) = &manifest.state_root {
            if want != &r {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("state root {r} != manifest {want}"),
                ));
            }
        }
        Some(r)
    } else {
        None
    };
    Ok((hash, root_hex))
}
