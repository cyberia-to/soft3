//! Materialize the burial snapshot: every file the source manifest pins by
//! CID + sha256, fetched from IPFS into a local cache and verified on every
//! run. The cache is not part of any repository.

use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::Value;

use super::Chain;
use super::sha256_hex;

/// One pinned snapshot file, verified against the source manifest.
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub name: String,
    pub cid: String,
    pub sha256: String,
    pub bytes: u64,
    pub path: PathBuf,
}

/// The parsed source manifest (`<chain>/snapshot/manifest.json`).
#[derive(Debug, Clone)]
pub struct SourceManifest {
    pub sha256: String,
    pub halt_height: u64,
    pub halt_time: String,
    pub value: Value,
}

/// Files the pipeline needs, in the order they are read.
pub const NEEDED: &[&str] = &[
    "supply.json",
    "pools.json",
    "pubkeys.csv",
    "balances.csv",
    "delegations.csv",
    "holdings.jsonl",
    "passports.jsonl",
    "cyberlinks_indexed.csv.gz",
];

const GATEWAYS: &[&str] = &["https://gateway.pinata.cloud/ipfs"];

pub fn read_manifest(path: &Path) -> io::Result<SourceManifest> {
    let bytes = std::fs::read(path)?;
    let value: Value = serde_json::from_slice(&bytes)?;
    let halt_height = value
        .get("halt_height")
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid("manifest: halt_height"))?;
    let halt_time = value
        .get("halt_time")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid("manifest: halt_time"))?
        .to_string();
    Ok(SourceManifest {
        sha256: sha256_hex(&bytes),
        halt_height,
        halt_time,
        value,
    })
}

/// URL candidates for one file: the chain's pinned directory layouts first,
/// the burial site last. Every candidate is verified by sha256 and size.
fn candidates(chain: &Chain, cid: &str, name: &str) -> Vec<String> {
    let mut out = Vec::new();
    for gw in GATEWAYS {
        match chain.snapshot_name() {
            "pussy" => out.push(format!("{gw}/{cid}/pussy-snapshot/{name}")),
            _ => out.push(format!("{gw}/{cid}/{name}")),
        }
    }
    for gw in GATEWAYS {
        out.push(format!("{gw}/{cid}/{name}"));
        out.push(format!("{gw}/{cid}"));
    }
    let site = match chain.snapshot_name() {
        "pussy" => "https://pussy.bostrom.network",
        _ => "https://snapshot.bostrom.network",
    };
    out.push(format!("{site}/{name}"));
    out
}

fn download(url: &str, bytes: u64) -> io::Result<Vec<u8>> {
    let response = ureq::get(url)
        .timeout(Duration::from_secs(1800))
        .set("User-Agent", "soft3-genesis/0.1")
        .call()
        .map_err(|e| io::Error::other(format!("{url}: {e}")))?;
    let mut data = Vec::with_capacity(bytes as usize);
    response
        .into_reader()
        .take(bytes + 1)
        .read_to_end(&mut data)?;
    Ok(data)
}

/// Fetch (or reuse from cache) every needed file, verifying sha256 + size.
pub fn materialize(chain: &Chain, manifest_path: &Path, cache: &Path) -> io::Result<Vec<SourceFile>> {
    let manifest = read_manifest(manifest_path)?;
    let files = manifest
        .value
        .get("files")
        .and_then(Value::as_object)
        .ok_or_else(|| invalid("manifest: files"))?;
    std::fs::create_dir_all(cache)?;
    let mut out = Vec::new();
    for name in NEEDED {
        let entry = files
            .get(*name)
            .ok_or_else(|| invalid(format!("manifest: no file {name}")))?;
        let cid = entry.get("cid").and_then(Value::as_str).ok_or_else(|| invalid("cid"))?;
        let sha = entry.get("sha256").and_then(Value::as_str).ok_or_else(|| invalid("sha256"))?;
        let bytes = entry.get("bytes").and_then(Value::as_u64).ok_or_else(|| invalid("bytes"))?;
        let path = cache.join(name);
        let cached_ok = match std::fs::read(&path) {
            Ok(data) => data.len() as u64 == bytes && sha256_hex(&data) == sha,
            Err(_) => false,
        };
        if !cached_ok {
            let mut got = None;
            let mut tried = Vec::new();
            for url in candidates(chain, cid, name) {
                match download(&url, bytes) {
                    Ok(data) if data.len() as u64 == bytes && sha256_hex(&data) == sha => {
                        got = Some(data);
                        break;
                    }
                    Ok(data) => tried.push(format!("{url}: {} bytes, sha mismatch", data.len())),
                    Err(e) => tried.push(e.to_string()),
                }
            }
            let data = got.ok_or_else(|| {
                io::Error::other(format!("{name}: no candidate verified\n  {}", tried.join("\n  ")))
            })?;
            let tmp = cache.join(format!("{name}.part"));
            std::fs::write(&tmp, &data)?;
            std::fs::rename(&tmp, &path)?;
        }
        out.push(SourceFile {
            name: name.to_string(),
            cid: cid.to_string(),
            sha256: sha.to_string(),
            bytes,
            path,
        });
    }
    Ok(out)
}

fn invalid(msg: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.into())
}
