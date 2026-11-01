//! Genesis pipeline of the phase-1 root chain: burial snapshot → genesis file
//! (one tade stream, `specs/chain.md` §4) → block-0 state root.
//!
//! Contract: `specs/chain.md` §4 (file, derivation, block 0) with the
//! decisions and the debug export in `specs/genesis.md`. Inputs are the
//! burial snapshots pinned by CID + sha256 in `space-pussy/snapshot/` and
//! `bostrom/snapshot/`. The genesis file's hemera particle is the chain id;
//! [`state::load`] applies the file to a fresh `bbg::BbgState` and one
//! `root()` is `state_root` of block 0.
//!
//! Integers only: micro-unit amounts are `u128` in derivation and must fit
//! `u64` in the file. No float touches a balance anywhere in this module.

pub mod derive;
pub mod fetch;
pub mod file;
pub mod source;
pub mod state;
pub mod time;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

pub use file::{GenesisFile, Params};
pub use crate::chain::pool::{self, Pool};

pub type Id = [u8; 32];

/// A bootloader chain whose burial snapshot seeds a phase-1 root chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chain {
    /// Genesis chain name (`pussy` | `bostrom` | `pussy-rc`), the `p` frame's `name`.
    pub name: &'static str,
    /// bech32 prefix of the legacy accounts.
    pub hrp: &'static str,
    /// Native denom; `token(native)` is the chain's native coin.
    pub native: &'static str,
    /// The cosmos chain-id of the buried bootloader.
    pub bootloader: &'static str,
    /// Genesis time of chain.md §4.3.
    pub genesis_time: u64,
}

pub const PUSSY: Chain = Chain {
    name: "pussy",
    hrp: "pussy",
    native: "pussy",
    bootloader: "space-pussy",
    genesis_time: 1_793_885_820, // 2026-11-05T13:37:00Z
};

pub const BOSTROM: Chain = Chain {
    name: "bostrom",
    hrp: "bostrom",
    native: "boot",
    bootloader: "bostrom",
    genesis_time: 1_793_884_962, // 2026-11-05T13:22:42Z
};

/// Rehearsal chain: pussy's burial state (same snapshot, same derivation, same
/// `pussy` hrp and native denom, so the same keys work) under the genesis name
/// `pussy-rc`. The name is in the `p` frame, so the chain id differs from
/// pussy's and the two can never be confused. It has no blessed genesis time:
/// `build` refuses without an operator-given `--time`.
pub const PUSSY_RC: Chain = Chain {
    name: "pussy-rc",
    hrp: "pussy",
    native: "pussy",
    bootloader: "space-pussy",
    genesis_time: 0,
};

impl Chain {
    /// The snapshot (cache and fetch layout) this chain is built from:
    /// `pussy` for `pussy-rc`, else the chain's own name.
    pub fn snapshot_name(&self) -> &'static str {
        if self.name == PUSSY_RC.name { PUSSY.name } else { self.name }
    }

    pub fn parse(name: &str) -> Option<Chain> {
        match name {
            "pussy" | "space-pussy" => Some(PUSSY),
            "bostrom" => Some(BOSTROM),
            "pussy-rc" => Some(PUSSY_RC),
            _ => None,
        }
    }
}

pub fn hash32(bytes: &[u8]) -> Id {
    *hemera::hash(bytes).as_bytes()
}

/// `token(denom) = hemera(denom_bytes)` (chain.md §3.1).
pub fn token_of(denom: &str) -> Id {
    hash32(denom.as_bytes())
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

pub fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

pub fn unhex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect()
}

/// Options for one build.
#[derive(Debug, Clone)]
pub struct Options {
    /// Where the fetched snapshot files live (verified by sha256 on every run).
    pub cache: PathBuf,
    /// Apply the file to bbg and compute the block-0 root.
    pub root: bool,
    /// Also write the debug export (TSV side tables) into this directory.
    pub export: Option<PathBuf>,
    /// Override the genesis time (rehearsal nets only).
    pub time: Option<u64>,
}

impl Options {
    pub fn default_cache(chain: &Chain) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        Path::new(&home).join(".cache").join("cyber").join("snapshots").join(chain.snapshot_name())
    }
}

/// What a build produced; also written beside the file as `<out>.json`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Report {
    pub format: &'static str,
    pub chain: String,
    pub bootloader: String,
    /// `hemera(file bytes)`: the chain id.
    pub particle: String,
    pub file_bytes: u64,
    pub file_sha256: String,
    pub time: u64,
    pub s0: String,
    pub subsidy: u64,
    /// The frozen pool formats the chain id commits to (chain.md §11.10).
    pub pool: PoolReport,
    pub state_root: Option<String>,
    pub block0_id: Option<String>,
    pub root_seconds: Option<u64>,
    pub root_blocked: Option<BTreeMap<String, String>>,
    pub state_counts: Option<BTreeMap<String, u64>>,
    pub state_fields: Option<BTreeMap<String, u64>>,
    pub counts: BTreeMap<String, u64>,
    /// denom → (token id hex, Σ amount) over the `b` frames.
    pub tokens: BTreeMap<String, (String, String)>,
    pub gaps: BTreeMap<String, String>,
    pub source: BTreeMap<String, BTreeMap<String, String>>,
    pub bbg: &'static str,
}

/// The pool record of a genesis file, with the empty-tree anchor it implies.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PoolReport {
    pub format: u32,
    pub depth: u8,
    pub anchor_window: u32,
    /// bytes of a wide digest
    pub digest: u8,
    pub tags: String,
    /// `E_depth`: the anchor of block 0 (no commitments yet).
    pub empty_root: String,
}

impl PoolReport {
    pub fn of(p: &Pool) -> Self {
        Self {
            format: p.format,
            depth: p.depth,
            anchor_window: p.anchor_window,
            digest: p.digest,
            tags: hex(&p.tags),
            empty_root: hex(&pool::empty_root()),
        }
    }
}

/// Build the genesis file for `chain` from the snapshot directory holding
/// `manifest.json`; write it to `out` and the report to `<out>.json`.
pub fn build(chain: Chain, snapshot: &Path, out: &Path, options: &Options) -> io::Result<Report> {
    if chain == PUSSY_RC && options.time.is_none() {
        return Err(io::Error::other("pussy-rc has no blessed genesis time; pass --time"));
    }
    let manifest_path = snapshot.join("manifest.json");
    let sources = fetch::materialize(&chain, &manifest_path, &options.cache)?;
    let snap = source::Snapshot::read(&chain, &manifest_path, &sources)?;
    let time = options.time.unwrap_or(chain.genesis_time);
    let derived = derive::derive(&chain, &snap, time)?;
    let bytes = derived.file.encode()?;
    let particle = hash32(&bytes);
    if let Some(parent) = out.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    write_atomic(out, &bytes)?;

    let mut report = Report {
        format: file::DIALECT,
        chain: chain.name.into(),
        bootloader: chain.bootloader.into(),
        particle: hex(&particle),
        file_bytes: bytes.len() as u64,
        file_sha256: sha256_hex(&bytes),
        time,
        s0: derived.file.native_supply().to_string(),
        subsidy: derived.file.params.subsidy,
        pool: PoolReport::of(&derived.file.params.pool),
        state_root: None,
        block0_id: None,
        root_seconds: None,
        root_blocked: None,
        state_counts: None,
        state_fields: None,
        counts: derived.counts.clone(),
        tokens: derived
            .token_sums
            .iter()
            .map(|(t, d, s)| (d.clone(), (hex(t), s.to_string())))
            .collect(),
        gaps: derived.gaps.clone(),
        source: source_pins(&snap),
        bbg: "bbg dimension version 4 (keyed-trie root, NeuronRecord.nonce, CardRecord positions)",
    };
    if options.root {
        let started = std::time::Instant::now();
        let bbg = state::load(&derived.file);
        report.state_counts = Some(state::counts(&bbg));
        report.state_fields = Some(state::fields_per_dimension(&bbg));
        let blocked = state::over_limit(&bbg);
        if blocked.is_empty() {
            let root = bbg.root();
            report.root_seconds = Some(started.elapsed().as_secs());
            let header = file::block0_header(time, &root, &particle);
            report.state_root = Some(hex(&root));
            report.block0_id = Some(hex(&file::block_id(&header)));
        } else {
            report.root_blocked = Some(blocked);
        }
    }
    if let Some(dir) = &options.export {
        export(&derived, dir)?;
    }
    let mut json = serde_json::to_vec_pretty(&report)?;
    json.push(b'\n');
    write_atomic(&report_path(out), &json)?;
    Ok(report)
}

fn report_path(out: &Path) -> PathBuf {
    let mut p = out.as_os_str().to_owned();
    p.push(".json");
    PathBuf::from(p)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".part");
    let tmp = PathBuf::from(tmp);
    {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)
}

fn source_pins(snap: &source::Snapshot) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    let mut m = BTreeMap::new();
    m.insert("sha256".into(), snap.manifest.sha256.clone());
    m.insert("halt_height".into(), snap.manifest.halt_height.to_string());
    m.insert("halt_time".into(), snap.manifest.halt_time.clone());
    out.insert("manifest.json".into(), m);
    for f in &snap.files {
        let mut m = BTreeMap::new();
        m.insert("cid".into(), f.cid.clone());
        m.insert("sha256".into(), f.sha256.clone());
        m.insert("bytes".into(), f.bytes.to_string());
        out.insert(f.name.clone(), m);
    }
    out
}

/// Read a genesis file, recompute its particle and (optionally) the block-0
/// root and id.
pub fn verify(path: &Path, root: bool) -> io::Result<Verified> {
    let bytes = std::fs::read(path)?;
    let file = GenesisFile::decode(&bytes)?;
    let particle = hash32(&bytes);
    let mut v = Verified {
        particle: hex(&particle),
        s0: file.native_supply().to_string(),
        subsidy: file.params.subsidy,
        b_frames: file.balances.len() as u64,
        l_frames: file.links.len() as u64,
        pool: PoolReport::of(&file.params.pool),
        state_root: None,
        block0_id: None,
        root_blocked: None,
    };
    if file::subsidy_of(file.native_supply())? != file.params.subsidy {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "subsidy does not match S0"));
    }
    if root {
        let bbg = state::load(&file);
        let blocked = state::over_limit(&bbg);
        if blocked.is_empty() {
            let r = bbg.root();
            let header = file::block0_header(file.params.time, &r, &particle);
            v.state_root = Some(hex(&r));
            v.block0_id = Some(hex(&file::block_id(&header)));
        } else {
            v.root_blocked = Some(blocked);
        }
    }
    Ok(v)
}

#[derive(Debug, Clone)]
pub struct Verified {
    pub particle: String,
    pub s0: String,
    pub subsidy: u64,
    pub b_frames: u64,
    pub l_frames: u64,
    pub pool: PoolReport,
    pub state_root: Option<String>,
    pub block0_id: Option<String>,
    pub root_blocked: Option<BTreeMap<String, String>>,
}

/// The debug export: TSV side tables that explain the file (addresses,
/// denoms, CIDs, old heights and times). Not consensus; never hashed.
fn export(d: &derive::Derived, dir: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let w = |name: &str, header: &str, rows: &mut dyn Iterator<Item = String>| -> io::Result<()> {
        let mut out = io::BufWriter::with_capacity(1 << 20, std::fs::File::create(dir.join(name))?);
        writeln!(out, "{header}")?;
        for r in rows {
            writeln!(out, "{r}")?;
        }
        out.flush()
    };
    w(
        "accounts.tsv",
        "address\towner\tkind\tpubkey_type\tpubkey",
        &mut d.accounts.iter().map(|a| {
            format!(
                "{}\t{}\t{}\t{}\t{}",
                a.address,
                hex(&a.owner),
                match a.kind {
                    derive::OwnerKind::Neuron => "neuron",
                    derive::OwnerKind::Legacy => "legacy",
                },
                a.pubkey_type,
                a.pubkey.as_ref().map_or("-".to_string(), |k| hex(k))
            )
        }),
    )?;
    w(
        "balances.tsv",
        "owner\ttoken\tamount\taddress\tdenom",
        &mut d.file.balances.iter().zip(&d.balance_rows).map(|(b, (address, denom, _))| {
            format!("{}\t{}\t{}\t{}\t{}", hex(&b.owner), hex(&b.token), b.amount, address, denom)
        }),
    )?;
    w(
        "particles.tsv",
        "particle\tcid",
        &mut d.particles.iter().map(|(cid, p)| format!("{}\t{cid}", hex(p))),
    )?;
    let passports = d.file.links.len() - d.link_rows.len();
    w(
        "links.tsv",
        "neuron\tp\tq\theight\taddress\tfrom\tto\ttime\ttx",
        &mut d.file.links.iter().enumerate().map(|(i, l)| {
            let tail = if i < passports {
                "-\t-\t-\t-\t-".to_string()
            } else {
                let (address, from, to, _, time, tx) = &d.link_rows[i - passports];
                format!("{address}\t{from}\t{to}\t{time}\t{}", if tx.is_empty() { "-" } else { tx })
            };
            format!("{}\t{}\t{}\t{}\t{tail}", hex(&l.neuron), hex(&l.p), hex(&l.q), l.height)
        }),
    )?;
    w(
        "tokens.tsv",
        "token\tdenom\tsum",
        &mut d.token_sums.iter().map(|(t, denom, s)| format!("{}\t{denom}\t{s}", hex(t))),
    )?;
    Ok(())
}
