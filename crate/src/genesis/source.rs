//! Parsers for the burial snapshot tables. Plain CSV (no quoting is used by
//! the extractors: bech32, CIDs, digits, hex, timestamps) and JSON lines.
//! Amounts are parsed as `u128` decimal integers; anything else is an error.

use std::collections::BTreeMap;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

use serde_json::Value;

use super::Chain;
use super::fetch::{SourceFile, SourceManifest, read_manifest};
use super::time::parse_timestamp;

#[derive(Debug, Clone)]
pub struct BalanceRow {
    pub address: String,
    pub denom: String,
    pub amount: u128,
}

#[derive(Debug, Clone)]
pub struct PubkeyRow {
    pub address: String,
    pub kind: String,
    pub key: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct DelegationRow {
    pub delegator: String,
    pub validator: String,
    pub amount: u128,
}

#[derive(Debug, Clone)]
pub struct LinkSource {
    pub from: String,
    pub to: String,
    pub neuron: String,
    pub height: u64,
    pub time: u64,
    pub tx: String,
}

#[derive(Debug, Clone)]
pub struct PassportSource {
    /// cw721 token id (the extractor's top-level `nickname` field).
    pub token_id: String,
    /// The passport nickname (`extension.nickname`, falling back to the token id).
    pub nickname: String,
    pub owner: String,
    pub avatar: Option<String>,
}

/// One liquidity pool: its coin, reserves at halt and (pussy only) the
/// reserve account; bostrom's extractor did not record the account.
#[derive(Debug, Clone)]
pub struct Pool {
    pub id: String,
    pub coin: String,
    pub reserves: BTreeMap<String, u128>,
    pub reserve_account: Option<String>,
}

/// The snapshot, parsed.
pub struct Snapshot {
    pub manifest: SourceManifest,
    pub files: Vec<SourceFile>,
    pub supply: BTreeMap<String, u128>,
    pub pools: Vec<Pool>,
    pub pubkeys: Vec<PubkeyRow>,
    pub balances: Vec<BalanceRow>,
    pub delegations: Vec<DelegationRow>,
    /// Per-address unbonding amount of the native denom (integer, from
    /// `holdings.jsonl`), zero rows omitted.
    pub unbonding: Vec<(String, u128)>,
    pub passports: Vec<PassportSource>,
    pub links: Vec<LinkSource>,
}

impl Snapshot {
    pub fn read(chain: &Chain, manifest_path: &Path, files: &[SourceFile]) -> io::Result<Self> {
        let path = |name: &str| -> io::Result<&Path> {
            files
                .iter()
                .find(|f| f.name == name)
                .map(|f| f.path.as_path())
                .ok_or_else(|| invalid(format!("snapshot file {name} not materialized")))
        };
        Ok(Self {
            manifest: read_manifest(manifest_path)?,
            files: files.to_vec(),
            supply: read_supply(path("supply.json")?)?,
            pools: read_pools(path("pools.json")?)?,
            pubkeys: read_pubkeys(path("pubkeys.csv")?)?,
            balances: read_balances(path("balances.csv")?)?,
            delegations: read_delegations(path("delegations.csv")?)?,
            unbonding: read_unbonding(path("holdings.jsonl")?, chain.native)?,
            passports: read_passports(path("passports.jsonl")?)?,
            links: read_links(path("cyberlinks_indexed.csv.gz")?)?,
        })
    }
}

fn invalid(msg: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.into())
}

fn parse_amount(s: &str, what: &str) -> io::Result<u128> {
    let s = s.trim();
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid(format!("{what}: non-integer amount {s:?}")));
    }
    s.parse::<u128>()
        .map_err(|e| invalid(format!("{what}: amount {s:?}: {e}")))
}

fn split_csv(line: &str) -> Vec<&str> {
    line.trim_end_matches(['\r', '\n']).split(',').collect()
}

fn csv_header(reader: &mut impl BufRead, expect: &[&str], what: &str) -> io::Result<()> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let got = split_csv(&line);
    if got != expect {
        return Err(invalid(format!("{what}: header {got:?} != {expect:?}")));
    }
    Ok(())
}

pub fn read_supply(path: &Path) -> io::Result<BTreeMap<String, u128>> {
    let value: Value = serde_json::from_slice(&std::fs::read(path)?)?;
    let arr = value.as_array().ok_or_else(|| invalid("supply.json: not an array"))?;
    let mut out = BTreeMap::new();
    for item in arr {
        let denom = item.get("denom").and_then(Value::as_str).ok_or_else(|| invalid("supply: denom"))?;
        let amount = item.get("amount").and_then(Value::as_str).ok_or_else(|| invalid("supply: amount"))?;
        out.insert(denom.to_string(), parse_amount(amount, "supply")?);
    }
    Ok(out)
}

pub fn read_pools(path: &Path) -> io::Result<Vec<Pool>> {
    let value: Value = serde_json::from_slice(&std::fs::read(path)?)?;
    let arr = value.as_array().ok_or_else(|| invalid("pools.json: not an array"))?;
    let mut out = Vec::new();
    for item in arr {
        let id = match item.get("id") {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            _ => return Err(invalid("pools.json: id")),
        };
        let coin = item.get("pool_coin_denom").and_then(Value::as_str).ok_or_else(|| invalid("pools.json: pool_coin_denom"))?;
        let mut reserves = BTreeMap::new();
        for (d, a) in item.get("reserves").and_then(Value::as_object).ok_or_else(|| invalid("pools.json: reserves"))? {
            let amount = match a {
                Value::String(s) => parse_amount(s, "pools.json")?,
                Value::Number(n) => n.as_u64().map(u128::from).ok_or_else(|| invalid("pools.json: reserve not an integer"))?,
                _ => return Err(invalid("pools.json: reserve")),
            };
            reserves.insert(d.clone(), amount);
        }
        let reserve_account = item.get("reserve_account_address").and_then(Value::as_str).map(str::to_string);
        out.push(Pool { id, coin: coin.to_string(), reserves, reserve_account });
    }
    out.sort_by(|a, b| a.coin.cmp(&b.coin));
    Ok(out)
}

pub fn read_pubkeys(path: &Path) -> io::Result<Vec<PubkeyRow>> {
    use base64::Engine;
    let mut r = BufReader::new(std::fs::File::open(path)?);
    csv_header(&mut r, &["address", "pubkey_type", "pubkey_base64"], "pubkeys.csv")?;
    let mut out = Vec::new();
    for line in r.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let f = split_csv(&line);
        if f.len() != 3 {
            return Err(invalid(format!("pubkeys.csv: {line:?}")));
        }
        let key = if f[2].is_empty() {
            Vec::new()
        } else {
            base64::engine::general_purpose::STANDARD
                .decode(f[2])
                .map_err(|e| invalid(format!("pubkeys.csv: {}: {e}", f[0])))?
        };
        out.push(PubkeyRow { address: f[0].to_string(), kind: f[1].to_string(), key });
    }
    Ok(out)
}

pub fn read_balances(path: &Path) -> io::Result<Vec<BalanceRow>> {
    let mut r = BufReader::new(std::fs::File::open(path)?);
    csv_header(&mut r, &["address", "denom", "amount"], "balances.csv")?;
    let mut out = Vec::new();
    for line in r.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let f = split_csv(&line);
        if f.len() != 3 {
            return Err(invalid(format!("balances.csv: {line:?}")));
        }
        out.push(BalanceRow {
            address: f[0].to_string(),
            denom: f[1].to_string(),
            amount: parse_amount(f[2], "balances.csv")?,
        });
    }
    Ok(out)
}

/// The fourth column carries the integer delegated amount; the extractors
/// named it after the denom (`pussy`, `boot`), so only its position is fixed.
pub fn read_delegations(path: &Path) -> io::Result<Vec<DelegationRow>> {
    let mut r = BufReader::new(std::fs::File::open(path)?);
    let mut header = String::new();
    r.read_line(&mut header)?;
    let got = split_csv(&header);
    if got.len() != 4 || got[..3] != ["delegator", "validator", "shares"] {
        return Err(invalid(format!("delegations.csv: header {got:?}")));
    }
    let mut out = Vec::new();
    for line in r.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let f = split_csv(&line);
        if f.len() != 4 {
            return Err(invalid(format!("delegations.csv: {line:?}")));
        }
        out.push(DelegationRow {
            delegator: f[0].to_string(),
            validator: f[1].to_string(),
            amount: parse_amount(f[3], "delegations.csv")?,
        });
    }
    Ok(out)
}

/// `holdings.jsonl` is derived data; the only field taken from it is the
/// integer `undelegating` amount of the native denom per address, which the
/// extractor summed from the unbonding-delegation entries of the state export.
pub fn read_unbonding(path: &Path, native: &str) -> io::Result<Vec<(String, u128)>> {
    let r = BufReader::new(std::fs::File::open(path)?);
    let mut out = Vec::new();
    for line in r.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let v: Value = serde_json::from_str(&line)?;
        let address = v.get("address").and_then(Value::as_str).ok_or_else(|| invalid("holdings: address"))?;
        let Some(h) = v.get("holdings").and_then(|h| h.get(native)) else { continue };
        let Some(u) = h.get("undelegating") else { continue };
        let amount = match u {
            Value::Number(n) => n
                .as_u64()
                .map(u128::from)
                .ok_or_else(|| invalid(format!("holdings: {address}: undelegating not an integer: {n}")))?,
            Value::String(s) => parse_amount(s, "holdings")?,
            _ => return Err(invalid(format!("holdings: {address}: undelegating {u}"))),
        };
        if amount > 0 {
            out.push((address.to_string(), amount));
        }
    }
    Ok(out)
}

pub fn read_passports(path: &Path) -> io::Result<Vec<PassportSource>> {
    let r = BufReader::new(std::fs::File::open(path)?);
    let mut out = Vec::new();
    for line in r.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let v: Value = serde_json::from_str(&line)?;
        let token_id = v.get("nickname").and_then(Value::as_str).ok_or_else(|| invalid("passport: nickname"))?;
        let owner = v.get("owner").and_then(Value::as_str).ok_or_else(|| invalid("passport: owner"))?;
        let ext = v.get("extension");
        let nickname = ext
            .and_then(|e| e.get("nickname"))
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .unwrap_or(token_id);
        let avatar = ext
            .and_then(|e| e.get("avatar"))
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        out.push(PassportSource {
            token_id: token_id.to_string(),
            nickname: nickname.to_string(),
            owner: owner.to_string(),
            avatar,
        });
    }
    Ok(out)
}

pub fn read_links(path: &Path) -> io::Result<Vec<LinkSource>> {
    let file = std::fs::File::open(path)?;
    let gz = flate2::read::MultiGzDecoder::new(file);
    read_links_from(BufReader::with_capacity(1 << 20, gz))
}

pub fn read_links_from(mut r: impl BufRead) -> io::Result<Vec<LinkSource>> {
    csv_header(
        &mut r,
        &["particle_from", "particle_to", "neuron", "height", "timestamp", "transaction_hash"],
        "cyberlinks_indexed.csv",
    )?;
    let mut out = Vec::new();
    let mut line = String::new();
    loop {
        line.clear();
        if r.read_line(&mut line)? == 0 {
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        let f = split_csv(&line);
        if f.len() != 6 {
            return Err(invalid(format!("cyberlinks row {}: {line:?}", out.len() + 1)));
        }
        let height = f[3]
            .parse::<u64>()
            .map_err(|e| invalid(format!("cyberlinks row {}: height {e}", out.len() + 1)))?;
        let time = parse_timestamp(f[4])
            .ok_or_else(|| invalid(format!("cyberlinks row {}: timestamp {:?}", out.len() + 1, f[4])))?;
        out.push(LinkSource {
            from: f[0].to_string(),
            to: f[1].to_string(),
            neuron: f[2].to_string(),
            height,
            time,
            tx: f[5].to_string(),
        });
    }
    Ok(out)
}

/// Read a whole file, bounded.
pub fn read_bounded(path: &Path, limit: u64) -> io::Result<Vec<u8>> {
    let mut data = Vec::new();
    std::fs::File::open(path)?.take(limit + 1).read_to_end(&mut data)?;
    if data.len() as u64 > limit {
        return Err(invalid(format!("{} exceeds {limit} bytes", path.display())));
    }
    Ok(data)
}
