//! Derivation of the genesis file from the burial snapshot
//! (`specs/chain.md` §4.2; decisions in `specs/genesis.md`).
//!
//! Integers only: amounts are `u128` here and must fit `u64` in the file.

use std::collections::{BTreeMap, BTreeSet};
use std::io;

use sha2::{Digest, Sha256};

use super::file::{self, Balance, GenesisFile, Link, Params};
use super::source::Snapshot;
use super::{Chain, Id, hash32, token_of};

pub const SECP256K1: &str = "/cosmos.crypto.secp256k1.PubKey";
const LEGACY_DOMAIN: &[u8] = b"soft3/chain/legacy/v1\0";

/// `legacy_owner(hrp, account_id) = hemera("soft3/chain/legacy/v1\0" ‖ hrp ‖ account_id)`
/// (chain.md §2.8). `account_id` is the bech32 payload: 20 bytes for a key
/// account, 32 for a contract.
pub fn legacy_owner(hrp: &str, account_id: &[u8]) -> Id {
    let mut h = hemera::Hasher::new();
    h.update(LEGACY_DOMAIN);
    h.update(hrp.as_bytes());
    h.update(account_id);
    *h.finalize().as_bytes()
}

/// The genesis particle of a legacy CID: `hemera(cid bytes)`, the no-bytes
/// branch of `file::readdress` (cyber/launch row 20).
pub fn particle_of_cid(cid: &str) -> Id {
    hash32(cid.as_bytes())
}

/// `hemera("passport:" ‖ nickname)` (chain.md §4.2).
pub fn passport_particle(nickname: &str) -> Id {
    let mut v = b"passport:".to_vec();
    v.extend_from_slice(nickname.as_bytes());
    hash32(&v)
}

/// Cosmos module account address: `bech32(hrp, sha256(name)[..20])`.
pub fn module_address(hrp: &str, name: &str) -> io::Result<String> {
    let digest = Sha256::digest(name.as_bytes());
    let hrp = bech32::Hrp::parse(hrp).map_err(|e| invalid(e.to_string()))?;
    bech32::encode::<bech32::Bech32>(hrp, &digest[..20]).map_err(|e| invalid(e.to_string()))
}

fn invalid(msg: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.into())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnerKind {
    Neuron,
    Legacy,
}

/// One snapshot account and the id it owns at genesis.
#[derive(Clone, Debug)]
pub struct Account {
    pub address: String,
    pub owner: Id,
    pub kind: OwnerKind,
    pub pubkey_type: String,
    pub pubkey: Option<Vec<u8>>,
}

/// Everything derived: the file plus the debug-export side tables.
pub struct Derived {
    pub file: GenesisFile,
    pub accounts: Vec<Account>,
    /// (address, denom, amount) behind every `b` frame, same order as the frames.
    pub balance_rows: Vec<(String, String, u128)>,
    /// CID → particle for every CID a link names, sorted by CID.
    pub particles: Vec<(String, Id)>,
    /// (address, from cid, to cid, old height, time, tx) behind every source link.
    pub link_rows: Vec<(String, String, String, u64, u64, String)>,
    /// (token id, denom, Σ amount) per carried token.
    pub token_sums: Vec<(Id, String, u128)>,
    pub counts: BTreeMap<String, u64>,
    pub gaps: BTreeMap<String, String>,
}

fn decode_address(chain: &Chain, address: &str) -> io::Result<Vec<u8>> {
    let (hrp, data) = bech32::decode(address).map_err(|e| invalid(format!("{address}: {e}")))?;
    if hrp.as_str() != chain.hrp {
        return Err(invalid(format!("{address}: prefix {} is not {}", hrp.as_str(), chain.hrp)));
    }
    if data.len() != 20 && data.len() != 32 {
        return Err(invalid(format!("{address}: {} byte account id", data.len())));
    }
    Ok(data)
}

fn add(map: &mut BTreeMap<(String, String), u128>, address: &str, denom: &str, amount: u128, what: &str) -> io::Result<()> {
    if amount == 0 {
        return Ok(());
    }
    let slot = map.entry((address.to_string(), denom.to_string())).or_insert(0);
    *slot = slot.checked_add(amount).ok_or_else(|| invalid(format!("{what}: overflow")))?;
    Ok(())
}

fn sub(map: &mut BTreeMap<(String, String), u128>, address: &str, denom: &str, amount: u128, what: &str) -> io::Result<()> {
    let key = (address.to_string(), denom.to_string());
    let have = map.get(&key).copied().unwrap_or(0);
    let left = have
        .checked_sub(amount)
        .ok_or_else(|| invalid(format!("{what}: {address} holds {have} {denom}, {amount} needed")))?;
    if left == 0 {
        map.remove(&key);
    } else {
        map.insert(key, left);
    }
    Ok(())
}

pub fn derive(chain: &Chain, snap: &Snapshot, time: u64) -> io::Result<Derived> {
    // ── owners ────────────────────────────────────────────────────────
    let mut keys: BTreeMap<&str, (&str, &[u8])> = BTreeMap::new();
    for pk in &snap.pubkeys {
        keys.insert(&pk.address, (&pk.kind, &pk.key));
    }
    let mut addresses: BTreeSet<&str> = keys.keys().copied().collect();
    addresses.extend(snap.balances.iter().map(|b| b.address.as_str()));
    addresses.extend(snap.delegations.iter().map(|d| d.delegator.as_str()));
    addresses.extend(snap.unbonding.iter().map(|u| u.0.as_str()));
    addresses.extend(snap.links.iter().map(|l| l.neuron.as_str()));
    addresses.extend(snap.passports.iter().map(|p| p.owner.as_str()));
    let mut accounts = Vec::with_capacity(addresses.len());
    let mut owner_of: BTreeMap<&str, Id> = BTreeMap::new();
    let mut by_type: BTreeMap<String, u64> = BTreeMap::new();
    for address in addresses {
        let account_id = decode_address(chain, address)?;
        let (kind, owner, pubkey, pubkey_type) = match keys.get(address) {
            Some((t, key)) if *t == SECP256K1 && key.len() == 33 => {
                let derived = mudra::cosmos::address(key, chain.hrp).map_err(|e| invalid(e.to_string()))?;
                if derived != address {
                    return Err(invalid(format!("pubkey of {address} derives {derived}")));
                }
                (OwnerKind::Neuron, mudra::claim::neuron_of(key), Some(key.to_vec()), t.to_string())
            }
            Some((t, _)) => (
                OwnerKind::Legacy,
                legacy_owner(chain.hrp, &account_id),
                None,
                if t.is_empty() { "none".to_string() } else { t.to_string() },
            ),
            None => (OwnerKind::Legacy, legacy_owner(chain.hrp, &account_id), None, "absent".to_string()),
        };
        *by_type.entry(pubkey_type.clone()).or_insert(0) += 1;
        owner_of.insert(address, owner);
        accounts.push(Account { address: address.to_string(), owner, kind, pubkey_type, pubkey });
    }
    let owners: BTreeSet<Id> = accounts.iter().map(|a| a.owner).collect();
    if owners.len() != accounts.len() {
        return Err(invalid("two addresses share one owner id"));
    }

    // ── balances: bank, then stake returns, then LP dissolution ──────
    let mut bank: BTreeMap<(String, String), u128> = BTreeMap::new();
    for b in &snap.balances {
        add(&mut bank, &b.address, &b.denom, b.amount, "bank")?;
    }
    let native = chain.native;
    let bonded = module_address(chain.hrp, "bonded_tokens_pool")?;
    let not_bonded = module_address(chain.hrp, "not_bonded_tokens_pool")?;
    let pool_total = bank.get(&(bonded.clone(), native.into())).copied().unwrap_or(0)
        + bank.get(&(not_bonded.clone(), native.into())).copied().unwrap_or(0);
    let mut moved: u128 = 0;
    for d in &snap.delegations {
        add(&mut bank, &d.delegator, native, d.amount, "delegation")?;
        moved = moved.checked_add(d.amount).ok_or_else(|| invalid("delegation overflow"))?;
    }
    for (address, amount) in &snap.unbonding {
        add(&mut bank, address, native, *amount, "unbonding")?;
        moved = moved.checked_add(*amount).ok_or_else(|| invalid("unbonding overflow"))?;
    }
    let dust = pool_total
        .checked_sub(moved)
        .ok_or_else(|| invalid(format!("staking pools hold {pool_total}, {moved} delegated or unbonding")))?;
    bank.remove(&(bonded.clone(), native.into()));
    bank.remove(&(not_bonded.clone(), native.into()));
    if dust > 0 {
        bank.insert((bonded.clone(), native.into()), dust);
    }

    // LP positions dissolve into the pool reserves pro rata (integer floor);
    // the reserve account keeps the rounding dust; pool coins are not carried.
    let mut dissolved_positions = 0u64;
    let mut pool_coins: BTreeSet<String> = BTreeSet::new();
    for pool in &snap.pools {
        pool_coins.insert(pool.coin.clone());
        let supply = *snap.supply.get(&pool.coin).unwrap_or(&0);
        let live: BTreeMap<&String, &u128> = pool.reserves.iter().filter(|(_, r)| **r > 0).collect();
        // a drained pool (no reserve left) dissolves to nothing: its coins vanish
        let reserve_account = if live.is_empty() {
            None
        } else {
            Some(match &pool.reserve_account {
                Some(a) => a.clone(),
                None => find_reserve_account(&bank, pool)?,
            })
        };
        let holders: Vec<(String, u128)> = bank
            .iter()
            .filter(|((_, d), _)| *d == pool.coin)
            .map(|((a, _), amt)| (a.clone(), *amt))
            .collect();
        for (holder, lp) in holders {
            sub(&mut bank, &holder, &pool.coin, lp, "pool coin")?;
            if supply == 0 {
                continue;
            }
            let Some(reserve_account) = &reserve_account else { continue };
            for (denom, reserve) in &live {
                let share = lp
                    .checked_mul(**reserve)
                    .ok_or_else(|| invalid("pool share overflow"))?
                    / supply;
                if share > 0 {
                    sub(&mut bank, reserve_account, denom, share, "pool reserve")?;
                    add(&mut bank, &holder, denom, share, "pool share")?;
                }
            }
            dissolved_positions += 1;
        }
        // whatever pool coin supply was not in any account (burned, escrowed)
        // leaves the matching reserve share with the reserve account
    }
    for ((address, denom), _) in bank.iter() {
        if denom.starts_with("pool") || pool_coins.contains(denom) {
            return Err(invalid(format!("{address}: pool coin {denom} survived dissolution")));
        }
    }

    // ── conservation: Σ per carried denom == supply.json ──────────────
    let mut per_denom: BTreeMap<String, u128> = BTreeMap::new();
    for ((_, denom), amount) in &bank {
        *per_denom.entry(denom.clone()).or_insert(0) += amount;
    }
    for (denom, sum) in &per_denom {
        let supply = snap
            .supply
            .get(denom)
            .ok_or_else(|| invalid(format!("supply.json has no {denom}")))?;
        if sum != supply {
            return Err(invalid(format!("{denom}: genesis sum {sum} != supply {supply}")));
        }
    }
    let s0 = *per_denom.get(native).ok_or_else(|| invalid(format!("no {native} in genesis")))?;

    // a `b` frame and a `coins` total carry a u64; a denom with any balance
    // or a total above 2^64 − 1 (18-decimal IBC vouchers) cannot be carried
    // and is excluded whole, listed in the report. Never the native coin.
    let excluded: BTreeSet<String> = bank
        .iter()
        .filter(|((_, denom), amount)| {
            u64::try_from(**amount).is_err() || u64::try_from(per_denom[denom.as_str()]).is_err()
        })
        .map(|((_, denom), _)| denom.clone())
        .collect();
    if excluded.contains(native) {
        return Err(invalid(format!("{native}: a balance exceeds u64")));
    }
    let excluded_rows = bank.iter().filter(|((_, d), _)| excluded.contains(d)).count() as u64;
    bank.retain(|(_, d), _| !excluded.contains(d));
    for d in &excluded {
        per_denom.remove(d);
    }

    // ── b frames ─────────────────────────────────────────────────────
    let mut balances = Vec::with_capacity(bank.len());
    let mut balance_rows = Vec::with_capacity(bank.len());
    for ((address, denom), amount) in &bank {
        let owner = *owner_of.get(address.as_str()).ok_or_else(|| invalid(format!("no account for {address}")))?;
        let amount64 = u64::try_from(*amount).map_err(|_| invalid(format!("{address} {denom}: {amount} exceeds u64")))?;
        balances.push(Balance { owner, token: token_of(denom), amount: amount64 });
        balance_rows.push((address.clone(), denom.clone(), *amount));
    }
    let mut order: Vec<usize> = (0..balances.len()).collect();
    order.sort_by(|&i, &j| (balances[i].owner, balances[i].token).cmp(&(balances[j].owner, balances[j].token)));
    let balances: Vec<Balance> = order.iter().map(|&i| balances[i]).collect();
    let balance_rows: Vec<_> = order.iter().map(|&i| balance_rows[i].clone()).collect();
    for w in balances.windows(2) {
        if (w[0].owner, w[0].token) == (w[1].owner, w[1].token) {
            return Err(invalid("duplicate (owner, token)"));
        }
    }
    let mut token_sums: Vec<(Id, String, u128)> = per_denom
        .iter()
        .map(|(d, s)| (token_of(d), d.clone(), *s))
        .collect();
    token_sums.sort();

    // ── l frames: passports (height 0) then the source links ─────────
    let mut links = Vec::with_capacity(snap.passports.len() + snap.links.len());
    for p in &snap.passports {
        let owner = *owner_of.get(p.owner.as_str()).ok_or_else(|| invalid(format!("no account for {}", p.owner)))?;
        links.push(Link { neuron: owner, p: owner, q: passport_particle(&p.nickname), height: 0 });
    }
    let mut cids: BTreeSet<&str> = BTreeSet::new();
    for l in &snap.links {
        cids.insert(&l.from);
        cids.insert(&l.to);
    }
    let particles: Vec<(String, Id)> = cids.iter().map(|c| (c.to_string(), particle_of_cid(c))).collect();
    let particle_of: BTreeMap<&str, Id> = particles.iter().map(|(c, p)| (c.as_str(), *p)).collect();
    let mut by_legacy = 0u64;
    let mut link_rows = Vec::with_capacity(snap.links.len());
    for l in &snap.links {
        let neuron = *owner_of.get(l.neuron.as_str()).ok_or_else(|| invalid(format!("no account for {}", l.neuron)))?;
        if keys.get(l.neuron.as_str()).is_none_or(|(t, k)| !(*t == SECP256K1 && k.len() == 33)) {
            by_legacy += 1;
        }
        links.push(Link { neuron, p: particle_of[l.from.as_str()], q: particle_of[l.to.as_str()], height: l.height });
        link_rows.push((l.neuron.clone(), l.from.clone(), l.to.clone(), l.height, l.time, l.tx.clone()));
    }

    let params = Params {
        version: file::VERSION,
        name: chain.name.into(),
        hrp: chain.hrp.into(),
        denom: native.into(),
        time,
        bits: file::GENESIS_BITS,
        subsidy: file::subsidy_of(s0)?,
        checkpoint_key: [0u8; 33],
        checkpoint_sunset: file::CHECKPOINT_SUNSET,
    };
    let genesis = GenesisFile { params, balances, links };
    genesis.check()?;

    let mut counts = BTreeMap::new();
    counts.insert("accounts".into(), accounts.len() as u64);
    counts.insert("accounts_neuron".into(), accounts.iter().filter(|a| a.kind == OwnerKind::Neuron).count() as u64);
    counts.insert("accounts_legacy".into(), accounts.iter().filter(|a| a.kind == OwnerKind::Legacy).count() as u64);
    for (t, n) in &by_type {
        counts.insert(format!("accounts_by_key:{t}"), *n);
    }
    counts.insert("b_frames".into(), genesis.balances.len() as u64);
    counts.insert("l_frames".into(), genesis.links.len() as u64);
    counts.insert("l_frames_passports".into(), snap.passports.len() as u64);
    counts.insert("l_frames_source".into(), snap.links.len() as u64);
    counts.insert("l_frames_by_legacy_author".into(), by_legacy);
    counts.insert("tokens".into(), per_denom.len() as u64);
    counts.insert("tokens_excluded_u64".into(), excluded.len() as u64);
    counts.insert("balances_excluded_u64".into(), excluded_rows);
    counts.insert("particles".into(), particles.len() as u64);
    counts.insert("lp_positions_dissolved".into(), dissolved_positions);
    counts.insert("pools".into(), snap.pools.len() as u64);
    counts.insert("source_balances_rows".into(), snap.balances.len() as u64);
    counts.insert("source_pubkeys_rows".into(), snap.pubkeys.len() as u64);
    counts.insert("source_delegations_rows".into(), snap.delegations.len() as u64);
    counts.insert("source_unbonding_rows".into(), snap.unbonding.len() as u64);
    counts.insert("source_cyberlinks_rows".into(), snap.links.len() as u64);
    counts.insert("source_passports_rows".into(), snap.passports.len() as u64);
    counts.insert("staking_pool_dust".into(), u64::try_from(dust).map_err(|_| invalid("dust"))?);
    counts.insert("delegated".into(), u64::try_from(moved).map_err(|_| invalid("moved"))?);

    let mut gaps = BTreeMap::new();
    if !excluded.is_empty() {
        gaps.insert(
            "tokens_excluded_u64".into(),
            format!("{} denoms with a balance or total above 2^64-1 are not carried (u64 b frames and coin totals): {}", excluded.len(), excluded.iter().cloned().collect::<Vec<_>>().join(" ")),
        );
    }
    let m = &snap.manifest.value;
    if let Some(n) = m.pointer("/graph_coverage/missing_early_links").and_then(serde_json::Value::as_u64) {
        gaps.insert("cyberlinks_unrecoverable".into(), format!("{n} early links never indexed and pruned on every node"));
    }
    if let Some(n) = m.pointer("/particles_availability/missing").and_then(serde_json::Value::as_u64) {
        gaps.insert("particles_missing_bytes".into(), format!("{n} CIDs have no bytes on the cybernode; every genesis particle is the CID black hole"));
    }
    for ptr in ["/graph/onchain_cyberlinks", "/graph_coverage/restored_cyberlinks"] {
        if let Some(n) = m.pointer(ptr).and_then(serde_json::Value::as_u64) {
            if n != snap.links.len() as u64 {
                return Err(invalid(format!("manifest {ptr} = {n}, csv has {}", snap.links.len())));
            }
        }
    }

    Ok(Derived { file: genesis, accounts, balance_rows, particles, link_rows, token_sums, counts, gaps })
}

/// bostrom's `pools.json` carries no reserve account; find the one bank
/// holder whose balance of every reserve denom equals the pool's reserve.
fn find_reserve_account(bank: &BTreeMap<(String, String), u128>, pool: &super::source::Pool) -> io::Result<String> {
    let live: Vec<(&String, &u128)> = pool.reserves.iter().filter(|(_, r)| **r > 0).collect();
    let (first_denom, first_amount) = *live.first().ok_or_else(|| invalid("pool without reserves"))?;
    let candidates: Vec<&String> = bank
        .iter()
        .filter(|((_, d), amt)| d == first_denom && *amt == first_amount)
        .map(|((a, _), _)| a)
        .filter(|a| live.iter().all(|(d, r)| bank.get(&((*a).clone(), (*d).clone())).copied().unwrap_or(0) == **r))
        .collect();
    match candidates.as_slice() {
        [one] => Ok((*one).clone()),
        [] => Err(invalid(format!("pool {}: no bank account holds its reserves", pool.id))),
        many => Err(invalid(format!("pool {}: {} accounts hold its reserves", pool.id, many.len()))),
    }
}
