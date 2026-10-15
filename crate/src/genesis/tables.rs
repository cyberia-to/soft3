//! The genesis manifest: canonical tables derived from the snapshot, their
//! TSV encoding, and the hashed `manifest.json`. See `soft3/specs/genesis.md`.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::source::Snapshot;
use super::{Chain, TOCYB_RATE};

pub type Id = [u8; 32];

pub const FORMAT: &str = "soft3/genesis/v1";
pub const SECP256K1: &str = "/cosmos.crypto.secp256k1.PubKey";

const CLAIMABLE_DOMAIN: &[u8] = b"soft3/genesis/claimable/v1\0";
const PASSPORT_DOMAIN: &[u8] = b"soft3/genesis/passport/v1\0";
const NETWORK_DOMAIN: &[u8] = b"soft3/genesis/network/v1\0";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AccountKind {
    /// A secp256k1 pubkey is known: owner = `mudra::claim::neuron_of(pubkey)`.
    Neuron,
    /// No usable pubkey (never signed, or a non-secp256k1 key): owner =
    /// `hemera("soft3/genesis/claimable/v1\0" ‖ address)`, claimable later via
    /// the mudra legacy bridge.
    Claimable,
}

impl AccountKind {
    fn label(self) -> &'static str {
        match self {
            AccountKind::Neuron => "neuron",
            AccountKind::Claimable => "claimable",
        }
    }
    fn parse(s: &str) -> Option<Self> {
        match s {
            "neuron" => Some(AccountKind::Neuron),
            "claimable" => Some(AccountKind::Claimable),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Account {
    pub address: String,
    pub owner: Id,
    pub kind: AccountKind,
    pub pubkey: Option<Vec<u8>>,
    pub pubkey_type: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Balance {
    pub address: String,
    pub owner: Id,
    pub denom: String,
    pub amount: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeferredBalance {
    pub address: String,
    pub denom: String,
    pub amount: u128,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticleRow {
    pub particle: Id,
    pub cid: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkRow {
    pub from: Id,
    pub to: Id,
    pub neuron: Id,
    pub height: u64,
    pub time: u64,
    pub tx: String,
    pub address: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PassportRow {
    pub card: Id,
    pub owner: Id,
    pub particle: Id,
    pub token_id: String,
    pub nickname: String,
    pub avatar: Option<String>,
    pub address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceEntry {
    pub cid: String,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Source {
    pub manifest_sha256: String,
    pub halt_height: u64,
    pub halt_time: String,
    pub files: BTreeMap<String, SourceEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Table {
    pub sha256: String,
    pub rows: u64,
    pub bytes: u64,
}

/// `manifest.json`. Serialized compact with sorted keys; `sha256` of those
/// bytes is the genesis hash.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: String,
    pub chain: String,
    pub hrp: String,
    pub native: String,
    pub bootloader: String,
    pub source: Source,
    pub tables: BTreeMap<String, Table>,
    pub counts: BTreeMap<String, u64>,
    /// Genesis supply per loaded denom, decimal micro-units.
    pub supply: BTreeMap<String, String>,
    pub decisions: BTreeMap<String, String>,
    pub gaps: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_counts: Option<BTreeMap<String, u64>>,
    /// Field elements per committed dimension of the loaded state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_fields: Option<BTreeMap<String, u64>>,
    /// Present when the root could not be committed: dimension → why.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_blocked: Option<BTreeMap<String, String>>,
}

/// The derived genesis, in memory.
pub struct Genesis {
    pub chain: Chain,
    pub source: Source,
    pub accounts: Vec<Account>,
    pub balances: Vec<Balance>,
    pub deferred: Vec<DeferredBalance>,
    pub particles: Vec<ParticleRow>,
    pub links: Vec<LinkRow>,
    pub passports: Vec<PassportRow>,
    pub supply: BTreeMap<String, u128>,
    pub counts: BTreeMap<String, u64>,
    pub gaps: BTreeMap<String, String>,
}

// ── identities ───────────────────────────────────────────────────────────

pub fn hash32(bytes: &[u8]) -> Id {
    *hemera::hash(bytes).as_bytes()
}

fn domain_hash(domain: &[u8], parts: &[&[u8]]) -> Id {
    let mut h = hemera::Hasher::new();
    h.update(domain);
    for p in parts {
        h.update(&(p.len() as u64).to_le_bytes());
        h.update(p);
    }
    *h.finalize().as_bytes()
}

/// Owner id of an account whose secp256k1 key is unknown.
pub fn claimable_id(address: &str) -> Id {
    domain_hash(CLAIMABLE_DOMAIN, &[address.as_bytes()])
}

/// The genesis particle of a legacy CID: the black-hole derivation of
/// `file::readdress` (`Particle::hash(cid.as_bytes())`), used uniformly at
/// genesis because no content bytes are fetched here.
pub fn particle_of_cid(cid: &str) -> Id {
    hash32(cid.as_bytes())
}

/// Token id of a denom in the bbg balances/coins dimensions.
pub fn token_of(denom: &str) -> Id {
    hash32(denom.as_bytes())
}

/// Card id of a passport.
pub fn card_of(chain: &str, token_id: &str) -> Id {
    domain_hash(PASSPORT_DOMAIN, &[chain.as_bytes(), token_id.as_bytes()])
}

/// The network every genesis signal is recorded under.
pub fn network_of(chain: &str) -> Id {
    domain_hash(NETWORK_DOMAIN, &[chain.as_bytes()])
}

/// Cosmos module account address: `bech32(hrp, sha256(name)[..20])`.
pub fn module_address(hrp: &str, name: &str) -> io::Result<String> {
    let digest = Sha256::digest(name.as_bytes());
    let hrp = bech32::Hrp::parse(hrp).map_err(|e| invalid(e.to_string()))?;
    bech32::encode::<bech32::Bech32>(hrp, &digest[..20]).map_err(|e| invalid(e.to_string()))
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
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

pub fn unhex32(s: &str) -> Option<Id> {
    unhex(s)?.try_into().ok()
}

fn invalid(msg: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.into())
}

fn fits_u64(v: u128, what: &str) -> io::Result<u64> {
    u64::try_from(v).map_err(|_| invalid(format!("{what}: {v} does not fit u64")))
}

// ── derivation ───────────────────────────────────────────────────────────

/// Derive the genesis tables from the parsed snapshot. Every rule here is
/// recorded in `decisions()` and in `soft3/specs/genesis.md`.
pub fn derive(chain: &Chain, snap: &Snapshot) -> io::Result<Genesis> {
    // 1. accounts: every address the snapshot names, exactly once.
    let mut keys: BTreeMap<String, (String, Vec<u8>)> = BTreeMap::new();
    for pk in &snap.pubkeys {
        check_address(chain, &pk.address)?;
        keys.insert(pk.address.clone(), (pk.kind.clone(), pk.key.clone()));
    }
    let mut addresses: BTreeSet<String> = keys.keys().cloned().collect();
    for b in &snap.balances {
        addresses.insert(b.address.clone());
    }
    for d in &snap.delegations {
        addresses.insert(d.delegator.clone());
    }
    for (a, _) in &snap.unbonding {
        addresses.insert(a.clone());
    }
    for l in &snap.links {
        addresses.insert(l.neuron.clone());
    }
    for p in &snap.passports {
        addresses.insert(p.owner.clone());
    }
    let mut accounts = Vec::with_capacity(addresses.len());
    let mut owner_of: BTreeMap<String, Id> = BTreeMap::new();
    let mut by_type: BTreeMap<String, u64> = BTreeMap::new();
    for address in addresses {
        check_address(chain, &address)?;
        let (kind, owner, pubkey, pubkey_type) = match keys.get(&address) {
            Some((t, key)) if t == SECP256K1 && key.len() == 33 => {
                let derived = mudra::cosmos::address(key, chain.hrp).map_err(|e| invalid(e.to_string()))?;
                if derived != address {
                    return Err(invalid(format!("pubkey of {address} derives {derived}")));
                }
                (AccountKind::Neuron, mudra::claim::neuron_of(key), Some(key.clone()), t.clone())
            }
            Some((t, _)) => (
                AccountKind::Claimable,
                claimable_id(&address),
                None,
                if t.is_empty() { "none".to_string() } else { t.clone() },
            ),
            None => (AccountKind::Claimable, claimable_id(&address), None, "absent".to_string()),
        };
        *by_type.entry(pubkey_type.clone()).or_insert(0) += 1;
        owner_of.insert(address.clone(), owner);
        accounts.push(Account { address, owner, kind, pubkey, pubkey_type });
    }

    // 2. balances: native liquid + delegated + unbonding, out of the staking
    //    pools; tocyb × 666 → cyb on bostrom; every other denom deferred raw.
    let native = chain.native;
    let mut liquid: BTreeMap<(String, String), u128> = BTreeMap::new();
    for b in &snap.balances {
        let slot = liquid.entry((b.address.clone(), b.denom.clone())).or_insert(0);
        *slot = slot.checked_add(b.amount).ok_or_else(|| invalid("bank overflow"))?;
    }
    let mut genesis_native: BTreeMap<String, u128> = BTreeMap::new();
    let mut deferred = Vec::new();
    for ((address, denom), amount) in &liquid {
        if denom == native {
            genesis_native.insert(address.clone(), *amount);
        } else if !(chain.name == "bostrom" && denom == "tocyb") {
            deferred.push(DeferredBalance { address: address.clone(), denom: denom.clone(), amount: *amount });
        }
    }
    let bonded = module_address(chain.hrp, "bonded_tokens_pool")?;
    let not_bonded = module_address(chain.hrp, "not_bonded_tokens_pool")?;
    let pool_total = genesis_native.get(&bonded).copied().unwrap_or(0)
        + genesis_native.get(&not_bonded).copied().unwrap_or(0);
    let mut moved: u128 = 0;
    for d in &snap.delegations {
        let slot = genesis_native.entry(d.delegator.clone()).or_insert(0);
        *slot = slot.checked_add(d.amount).ok_or_else(|| invalid("delegation overflow"))?;
        moved = moved.checked_add(d.amount).ok_or_else(|| invalid("delegation overflow"))?;
    }
    for (address, amount) in &snap.unbonding {
        let slot = genesis_native.entry(address.clone()).or_insert(0);
        *slot = slot.checked_add(*amount).ok_or_else(|| invalid("unbonding overflow"))?;
        moved = moved.checked_add(*amount).ok_or_else(|| invalid("unbonding overflow"))?;
    }
    let remainder = pool_total.checked_sub(moved).ok_or_else(|| {
        invalid(format!("staking pools hold {pool_total} but {moved} is delegated or unbonding"))
    })?;
    genesis_native.insert(bonded.clone(), remainder);
    genesis_native.insert(not_bonded.clone(), 0);
    let native_sum: u128 = genesis_native.values().sum();
    let native_supply = *snap
        .supply
        .get(native)
        .ok_or_else(|| invalid(format!("supply.json has no {native}")))?;
    if native_sum != native_supply {
        return Err(invalid(format!("{native}: genesis sum {native_sum} != supply {native_supply}")));
    }
    let mut supply: BTreeMap<String, u128> = BTreeMap::new();
    supply.insert(native.to_string(), native_supply);
    let mut balances = Vec::new();
    for (address, amount) in &genesis_native {
        if *amount == 0 {
            continue;
        }
        let owner = *owner_of.get(address).ok_or_else(|| invalid(format!("no account for {address}")))?;
        balances.push(Balance {
            address: address.clone(),
            owner,
            denom: native.to_string(),
            amount: fits_u64(*amount, address)?,
        });
    }
    if chain.name == "bostrom" {
        let mut cyb_sum: u128 = 0;
        for ((address, denom), amount) in &liquid {
            if denom != "tocyb" || *amount == 0 {
                continue;
            }
            let cyb = amount.checked_mul(TOCYB_RATE).ok_or_else(|| invalid("cyb overflow"))?;
            cyb_sum = cyb_sum.checked_add(cyb).ok_or_else(|| invalid("cyb overflow"))?;
            let owner = *owner_of.get(address).ok_or_else(|| invalid(format!("no account for {address}")))?;
            balances.push(Balance {
                address: address.clone(),
                owner,
                denom: "cyb".to_string(),
                amount: fits_u64(cyb, address)?,
            });
        }
        let tocyb_supply = *snap.supply.get("tocyb").ok_or_else(|| invalid("supply.json has no tocyb"))?;
        let cyb_supply = tocyb_supply.checked_mul(TOCYB_RATE).ok_or_else(|| invalid("cyb overflow"))?;
        if cyb_sum != cyb_supply {
            return Err(invalid(format!("cyb: genesis sum {cyb_sum} != tocyb supply × 666 = {cyb_supply}")));
        }
        if let Some(audited) = snap
            .manifest
            .value
            .pointer("/supply_audit/tocyb_to_cyb/genesis_cyb")
            .and_then(serde_json::Value::as_u64)
        {
            if u128::from(audited) != cyb_supply {
                return Err(invalid(format!("cyb supply {cyb_supply} != audited {audited}")));
            }
        }
        supply.insert("cyb".to_string(), cyb_supply);
    }
    balances.sort_by(|a, b| (&a.address, &a.denom).cmp(&(&b.address, &b.denom)));
    deferred.sort_by(|a, b| (&a.address, &a.denom).cmp(&(&b.address, &b.denom)));

    // 3. particles: every CID a cyberlink names, once, under its genesis particle.
    let mut cids: BTreeSet<&str> = BTreeSet::new();
    for l in &snap.links {
        cids.insert(&l.from);
        cids.insert(&l.to);
    }
    let mut particle_of: BTreeMap<&str, Id> = BTreeMap::new();
    let mut particles = Vec::with_capacity(cids.len());
    for cid in cids {
        let p = particle_of_cid(cid);
        if let Some(prev) = particle_of.insert(cid, p) {
            if prev != p {
                return Err(invalid("particle derivation is not a function"));
            }
        }
        particles.push(ParticleRow { particle: p, cid: cid.to_string() });
    }
    let distinct: BTreeSet<Id> = particles.iter().map(|p| p.particle).collect();
    if distinct.len() != particles.len() {
        return Err(invalid("two CIDs share one genesis particle"));
    }

    // 4. cyberlinks: snapshot order, original neuron, height and time.
    let mut links = Vec::with_capacity(snap.links.len());
    let mut by_claimable = 0u64;
    let mut signals = 0u64;
    let mut prev: Option<(&str, u64, &str)> = None;
    for l in &snap.links {
        let neuron = *owner_of.get(&l.neuron).ok_or_else(|| invalid(format!("no account for {}", l.neuron)))?;
        if keys.get(&l.neuron).is_none_or(|(t, k)| !(t == SECP256K1 && k.len() == 33)) {
            by_claimable += 1;
        }
        let key = (l.neuron.as_str(), l.height, l.tx.as_str());
        if prev != Some(key) {
            signals += 1;
            prev = Some(key);
        }
        links.push(LinkRow {
            from: particle_of[l.from.as_str()],
            to: particle_of[l.to.as_str()],
            neuron,
            height: l.height,
            time: l.time,
            tx: l.tx.clone(),
            address: l.neuron.clone(),
        });
    }

    // 5. passports: one card per token, owner mapped, nickname particle.
    let mut passports = Vec::with_capacity(snap.passports.len());
    let mut cards: BTreeSet<Id> = BTreeSet::new();
    for p in &snap.passports {
        let owner = *owner_of.get(&p.owner).ok_or_else(|| invalid(format!("no account for {}", p.owner)))?;
        let card = card_of(chain.name, &p.token_id);
        if !cards.insert(card) {
            return Err(invalid(format!("duplicate passport token {}", p.token_id)));
        }
        passports.push(PassportRow {
            card,
            owner,
            particle: hash32(p.nickname.as_bytes()),
            token_id: p.token_id.clone(),
            nickname: p.nickname.clone(),
            avatar: p.avatar.clone(),
            address: p.owner.clone(),
        });
    }

    // counts and known gaps
    let mut counts = BTreeMap::new();
    counts.insert("accounts".into(), accounts.len() as u64);
    counts.insert(
        "accounts_neuron".into(),
        accounts.iter().filter(|a| a.kind == AccountKind::Neuron).count() as u64,
    );
    counts.insert(
        "accounts_claimable".into(),
        accounts.iter().filter(|a| a.kind == AccountKind::Claimable).count() as u64,
    );
    for (t, n) in &by_type {
        counts.insert(format!("accounts_by_key:{t}"), *n);
    }
    counts.insert("source_pubkeys_rows".into(), snap.pubkeys.len() as u64);
    counts.insert("source_balances_rows".into(), snap.balances.len() as u64);
    counts.insert("source_delegations_rows".into(), snap.delegations.len() as u64);
    counts.insert("source_unbonding_rows".into(), snap.unbonding.len() as u64);
    counts.insert("source_cyberlinks_rows".into(), snap.links.len() as u64);
    counts.insert("source_passports_rows".into(), snap.passports.len() as u64);
    counts.insert("balances".into(), balances.len() as u64);
    counts.insert("balances_deferred".into(), deferred.len() as u64);
    counts.insert("particles".into(), particles.len() as u64);
    counts.insert("cyberlinks".into(), links.len() as u64);
    counts.insert("cyberlinks_by_claimable".into(), by_claimable);
    counts.insert("signals".into(), signals);
    counts.insert("passports".into(), passports.len() as u64);
    counts.insert("staking_pool_remainder".into(), fits_u64(remainder, "pool remainder")?);
    counts.insert("delegated".into(), fits_u64(snap.delegations.iter().map(|d| d.amount).sum(), "delegated")?);
    counts.insert("unbonding".into(), fits_u64(snap.unbonding.iter().map(|u| u.1).sum(), "unbonding")?);

    let mut gaps = BTreeMap::new();
    let m = &snap.manifest.value;
    if let Some(n) = m.pointer("/graph_coverage/missing_early_links").and_then(serde_json::Value::as_u64) {
        gaps.insert(
            "cyberlinks_unrecoverable".into(),
            format!("{n} early links never indexed and pruned on every node; restored {} of {} on-chain",
                m.pointer("/graph_coverage/restored_cyberlinks").and_then(serde_json::Value::as_u64).unwrap_or(0),
                m.pointer("/graph_coverage/onchain_cyberlinks").and_then(serde_json::Value::as_u64).unwrap_or(0)),
        );
    }
    if let Some(n) = m.pointer("/particles_availability/missing").and_then(serde_json::Value::as_u64) {
        gaps.insert(
            "particles_missing_bytes".into(),
            format!("{n} CIDs have no bytes on the cybernode blockstore; every genesis particle is the CID black hole regardless"),
        );
    }
    if let Some(n) = m.pointer("/graph/onchain_cyberlinks").and_then(serde_json::Value::as_u64) {
        if n != snap.links.len() as u64 {
            return Err(invalid(format!("manifest declares {n} on-chain links, csv has {}", snap.links.len())));
        }
    }
    if let Some(n) = m.pointer("/graph_coverage/restored_cyberlinks").and_then(serde_json::Value::as_u64) {
        if n != snap.links.len() as u64 {
            return Err(invalid(format!("manifest declares {n} restored links, csv has {}", snap.links.len())));
        }
    }

    let mut files = BTreeMap::new();
    for f in &snap.files {
        files.insert(f.name.clone(), SourceEntry { cid: f.cid.clone(), sha256: f.sha256.clone(), bytes: f.bytes });
    }
    Ok(Genesis {
        chain: *chain,
        source: Source {
            manifest_sha256: snap.manifest.sha256.clone(),
            halt_height: snap.manifest.halt_height,
            halt_time: snap.manifest.halt_time.clone(),
            files,
        },
        accounts,
        balances,
        deferred,
        particles,
        links,
        passports,
        supply,
        counts,
        gaps,
    })
}

fn check_address(chain: &Chain, address: &str) -> io::Result<()> {
    let (hrp, data) = bech32::decode(address).map_err(|e| invalid(format!("{address}: {e}")))?;
    if hrp.as_str() != chain.hrp {
        return Err(invalid(format!("{address}: prefix {} is not {}", hrp.as_str(), chain.hrp)));
    }
    if data.len() != 20 && data.len() != 32 {
        return Err(invalid(format!("{address}: {} byte account id", data.len())));
    }
    Ok(())
}

pub fn decisions(chain: &Chain) -> BTreeMap<String, String> {
    let mut d = BTreeMap::new();
    d.insert("accounts".into(), "owner = mudra::claim::neuron_of(secp256k1 pubkey) when the snapshot holds the key; otherwise owner = hemera(\"soft3/genesis/claimable/v1\\0\" ‖ u64le(len) ‖ bech32 address), claimable through the mudra legacy bridge (mudra/specs/bridge.md); multisig and keyless accounts are claimable".into());
    d.insert("delegations".into(), "delegations and unbonding entries become liquid at genesis: each delegator's integer delegated amount (delegations.csv) and unbonding amount (holdings.jsonl, integer) is added to their native balance and taken out of the bonded/not-bonded staking pool module accounts; the pool dust left over stays on bonded_tokens_pool; the validator set is not carried (pure PoW)".into());
    d.insert("module_accounts".into(), "module accounts (distribution, fee collector, pools, ibc escrow) are carried as claimable rows under their bech32 module address so the genesis supply equals the bank supply exactly; their disposition is a launch decision outside this pipeline".into());
    d.insert("denoms".into(), format!("only {} are loaded into the bbg balances dimension; every other bank denom (ibc vouchers, pool coins, hydrogen/ampere/volt, liquid derivatives) is carried raw in balances-deferred.tsv and not converted", chain.genesis_denoms().join("+")));
    if chain.name == "bostrom" {
        d.insert("tocyb".into(), "every TOCYB becomes 666 CYB (denom cyb) at genesis; cyb supply = tocyb supply × 666 and must equal the audited genesis_cyb of the snapshot manifest".into());
    }
    d.insert("particles".into(), "every legacy CID becomes the black-hole particle hemera(cid bytes) — the no-bytes branch of file::readdress — uniformly, because genesis fetches no content; particles.tsv is the CID → particle table; the content particle sparks when bytes surface, and the files dimension marks every genesis particle available=false".into());
    d.insert("cyberlinks".into(), "one genesis link per restored snapshot row in snapshot order with the original neuron, height and block time; amount 0, valence +1, token 0; consecutive rows with the same (neuron, height, tx) form one signal; signals are keyed by their global position".into());
    d.insert("passports".into(), "each passport is one card in the bbg cards dimension: card = hemera(\"soft3/genesis/passport/v1\\0\" ‖ chain ‖ token id), owner = the owner's genesis id, particle = hemera(nickname); the avatar CID and nickname are carried in passports.tsv; no naming cyberlink is minted at genesis".into());
    d.insert("holdings".into(), "holdings.jsonl, pools.json, prices.json and validators.json are derived views and are not genesis records; only the integer unbonding amount is read from holdings.jsonl".into());
    d.insert("root".into(), "the state root is bbg::BbgState::root() after loading every table into an in-memory state and committing once (bulk build), with height 0, empty time and locations dimensions; the node that loads this manifest must reproduce the same root through soft3::genesis::state::load".into());
    d.insert("arithmetic".into(), "integers only: micro-unit amounts are u128 in derivation and must fit u64 in bbg; no float is read or written".into());
    d
}

// ── encoding ─────────────────────────────────────────────────────────────

const TABLES: &[&str] = &[
    "accounts.tsv",
    "balances.tsv",
    "balances-deferred.tsv",
    "particles.tsv",
    "cyberlinks.tsv",
    "passports.tsv",
];

fn write_table(dir: &Path, name: &str, header: &str, rows: &mut dyn Iterator<Item = String>) -> io::Result<Table> {
    let path = dir.join(name);
    let tmp = dir.join(format!("{name}.part"));
    let mut hasher = Sha256::new();
    let mut n = 0u64;
    let mut bytes = 0u64;
    {
        let file = std::fs::File::create(&tmp)?;
        let mut w = io::BufWriter::with_capacity(1 << 20, file);
        let line = format!("{header}\n");
        hasher.update(line.as_bytes());
        bytes += line.len() as u64;
        w.write_all(line.as_bytes())?;
        for row in rows {
            let line = format!("{row}\n");
            hasher.update(line.as_bytes());
            bytes += line.len() as u64;
            w.write_all(line.as_bytes())?;
            n += 1;
        }
        w.flush()?;
    }
    std::fs::rename(&tmp, &path)?;
    Ok(Table { sha256: hex(&hasher.finalize()), rows: n, bytes })
}

fn opt(s: &Option<String>) -> &str {
    s.as_deref().unwrap_or("-")
}

/// Write every table; returns the manifest without `state_root`.
pub fn write(g: &Genesis, dir: &Path) -> io::Result<Manifest> {
    let mut tables = BTreeMap::new();
    tables.insert(
        "accounts.tsv".to_string(),
        write_table(
            dir,
            "accounts.tsv",
            "address\towner\tkind\tpubkey_type\tpubkey",
            &mut g.accounts.iter().map(|a| {
                format!(
                    "{}\t{}\t{}\t{}\t{}",
                    a.address,
                    hex(&a.owner),
                    a.kind.label(),
                    a.pubkey_type,
                    a.pubkey.as_ref().map_or("-".to_string(), |k| hex(k))
                )
            }),
        )?,
    );
    tables.insert(
        "balances.tsv".to_string(),
        write_table(
            dir,
            "balances.tsv",
            "address\towner\tdenom\tamount",
            &mut g.balances.iter().map(|b| format!("{}\t{}\t{}\t{}", b.address, hex(&b.owner), b.denom, b.amount)),
        )?,
    );
    tables.insert(
        "balances-deferred.tsv".to_string(),
        write_table(
            dir,
            "balances-deferred.tsv",
            "address\tdenom\tamount",
            &mut g.deferred.iter().map(|b| format!("{}\t{}\t{}", b.address, b.denom, b.amount)),
        )?,
    );
    tables.insert(
        "particles.tsv".to_string(),
        write_table(
            dir,
            "particles.tsv",
            "particle\tcid",
            &mut g.particles.iter().map(|p| format!("{}\t{}", hex(&p.particle), p.cid)),
        )?,
    );
    tables.insert(
        "cyberlinks.tsv".to_string(),
        write_table(
            dir,
            "cyberlinks.tsv",
            "from\tto\tneuron\theight\ttime\ttx\taddress",
            &mut g.links.iter().map(|l| {
                format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    hex(&l.from),
                    hex(&l.to),
                    hex(&l.neuron),
                    l.height,
                    l.time,
                    if l.tx.is_empty() { "-" } else { l.tx.as_str() },
                    l.address
                )
            }),
        )?,
    );
    tables.insert(
        "passports.tsv".to_string(),
        write_table(
            dir,
            "passports.tsv",
            "card\towner\tparticle\ttoken_id\tnickname\tavatar\taddress",
            &mut g.passports.iter().map(|p| {
                format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    hex(&p.card),
                    hex(&p.owner),
                    hex(&p.particle),
                    p.token_id,
                    p.nickname.replace(['\t', '\n', '\r'], " "),
                    opt(&p.avatar),
                    p.address
                )
            }),
        )?,
    );
    Ok(Manifest {
        format: FORMAT.into(),
        chain: g.chain.name.into(),
        hrp: g.chain.hrp.into(),
        native: g.chain.native.into(),
        bootloader: g.chain.bootloader.into(),
        source: g.source.clone(),
        tables,
        counts: g.counts.clone(),
        supply: g.supply.iter().map(|(k, v)| (k.clone(), v.to_string())).collect(),
        decisions: decisions(&g.chain),
        gaps: g.gaps.clone(),
        state_root: None,
        state_counts: None,
        state_fields: None,
        root_blocked: None,
    })
}

/// Canonical bytes of a manifest: compact JSON with keys in sorted order.
pub fn canonical(m: &Manifest) -> io::Result<Vec<u8>> {
    let value = serde_json::to_value(m)?;
    Ok(serde_json::to_vec(&value)?)
}

/// Write `manifest.json` and `genesis.hash`; returns the genesis hash.
pub fn write_manifest(m: &Manifest, dir: &Path) -> io::Result<String> {
    let bytes = canonical(m)?;
    std::fs::write(dir.join("manifest.json"), &bytes)?;
    let hash = sha256_hex(&bytes);
    std::fs::write(dir.join("genesis.hash"), format!("{hash}\n"))?;
    Ok(hash)
}

/// The genesis block header stub (`header.json`): height 0, no parent.
pub fn write_header(m: &Manifest, genesis_hash: &str, time: u64, dir: &Path) -> io::Result<()> {
    let mut h = BTreeMap::new();
    h.insert("chain", serde_json::Value::String(m.chain.clone()));
    h.insert("protocol", serde_json::Value::String(FORMAT.into()));
    h.insert("height", serde_json::Value::from(0u64));
    h.insert("time", serde_json::Value::from(time));
    h.insert("time_iso", serde_json::Value::String(super::time::format_unix(time)));
    h.insert("prev", serde_json::Value::String(hex(&[0u8; 32])));
    h.insert("genesis_hash", serde_json::Value::String(genesis_hash.into()));
    h.insert(
        "state_root",
        m.state_root.clone().map_or(serde_json::Value::Null, serde_json::Value::String),
    );
    let mut bytes = serde_json::to_vec_pretty(&h)?;
    bytes.push(b'\n');
    std::fs::write(dir.join("header.json"), bytes)?;
    Ok(())
}

/// sha256 of every table file in `dir` named by the manifest.
pub fn table_hashes(dir: &Path, m: &Manifest) -> io::Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    for name in m.tables.keys() {
        let mut hasher = Sha256::new();
        let mut file = std::fs::File::open(dir.join(name))?;
        io::copy(&mut file, &mut hasher)?;
        out.insert(name.clone(), hex(&hasher.finalize()));
    }
    Ok(out)
}

// ── decoding ─────────────────────────────────────────────────────────────

fn rows(dir: &Path, name: &str, header: &str) -> io::Result<impl Iterator<Item = io::Result<Vec<String>>>> {
    let mut r = BufReader::with_capacity(1 << 20, std::fs::File::open(dir.join(name))?);
    let mut first = String::new();
    r.read_line(&mut first)?;
    if first.trim_end_matches('\n') != header {
        return Err(invalid(format!("{name}: header {first:?}")));
    }
    Ok(r.lines().map(|l| l.map(|l| l.split('\t').map(str::to_string).collect())))
}

fn field32(f: &[String], i: usize, what: &str) -> io::Result<Id> {
    f.get(i)
        .and_then(|s| unhex32(s))
        .ok_or_else(|| invalid(format!("{what}: field {i} not a 32-byte hex id")))
}

fn field_u64(f: &[String], i: usize, what: &str) -> io::Result<u64> {
    f.get(i)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| invalid(format!("{what}: field {i} not an integer")))
}

/// Read a genesis directory back into memory.
pub fn read(dir: &Path) -> io::Result<(Manifest, Genesis)> {
    let bytes = std::fs::read(dir.join("manifest.json"))?;
    let m: Manifest = serde_json::from_slice(&bytes)?;
    if m.format != FORMAT {
        return Err(invalid(format!("manifest format {}", m.format)));
    }
    let chain = Chain::parse(&m.chain).ok_or_else(|| invalid(format!("unknown chain {}", m.chain)))?;
    for name in TABLES {
        if !m.tables.contains_key(*name) {
            return Err(invalid(format!("manifest lists no {name}")));
        }
    }
    let mut accounts = Vec::new();
    for f in rows(dir, "accounts.tsv", "address\towner\tkind\tpubkey_type\tpubkey")? {
        let f = f?;
        if f.len() != 5 {
            return Err(invalid("accounts.tsv: field count"));
        }
        accounts.push(Account {
            address: f[0].clone(),
            owner: field32(&f, 1, "accounts.tsv")?,
            kind: AccountKind::parse(&f[2]).ok_or_else(|| invalid("accounts.tsv: kind"))?,
            pubkey_type: f[3].clone(),
            pubkey: if f[4] == "-" { None } else { Some(unhex(&f[4]).ok_or_else(|| invalid("accounts.tsv: pubkey"))?) },
        });
    }
    let mut balances = Vec::new();
    for f in rows(dir, "balances.tsv", "address\towner\tdenom\tamount")? {
        let f = f?;
        if f.len() != 4 {
            return Err(invalid("balances.tsv: field count"));
        }
        balances.push(Balance {
            address: f[0].clone(),
            owner: field32(&f, 1, "balances.tsv")?,
            denom: f[2].clone(),
            amount: field_u64(&f, 3, "balances.tsv")?,
        });
    }
    let mut deferred = Vec::new();
    for f in rows(dir, "balances-deferred.tsv", "address\tdenom\tamount")? {
        let f = f?;
        if f.len() != 3 {
            return Err(invalid("balances-deferred.tsv: field count"));
        }
        deferred.push(DeferredBalance {
            address: f[0].clone(),
            denom: f[1].clone(),
            amount: f[2].parse().map_err(|_| invalid("balances-deferred.tsv: amount"))?,
        });
    }
    let mut particles = Vec::new();
    for f in rows(dir, "particles.tsv", "particle\tcid")? {
        let f = f?;
        if f.len() != 2 {
            return Err(invalid("particles.tsv: field count"));
        }
        particles.push(ParticleRow { particle: field32(&f, 0, "particles.tsv")?, cid: f[1].clone() });
    }
    let mut links = Vec::new();
    for f in rows(dir, "cyberlinks.tsv", "from\tto\tneuron\theight\ttime\ttx\taddress")? {
        let f = f?;
        if f.len() != 7 {
            return Err(invalid("cyberlinks.tsv: field count"));
        }
        links.push(LinkRow {
            from: field32(&f, 0, "cyberlinks.tsv")?,
            to: field32(&f, 1, "cyberlinks.tsv")?,
            neuron: field32(&f, 2, "cyberlinks.tsv")?,
            height: field_u64(&f, 3, "cyberlinks.tsv")?,
            time: field_u64(&f, 4, "cyberlinks.tsv")?,
            tx: if f[5] == "-" { String::new() } else { f[5].clone() },
            address: f[6].clone(),
        });
    }
    let mut passports = Vec::new();
    for f in rows(dir, "passports.tsv", "card\towner\tparticle\ttoken_id\tnickname\tavatar\taddress")? {
        let f = f?;
        if f.len() != 7 {
            return Err(invalid("passports.tsv: field count"));
        }
        passports.push(PassportRow {
            card: field32(&f, 0, "passports.tsv")?,
            owner: field32(&f, 1, "passports.tsv")?,
            particle: field32(&f, 2, "passports.tsv")?,
            token_id: f[3].clone(),
            nickname: f[4].clone(),
            avatar: if f[5] == "-" { None } else { Some(f[5].clone()) },
            address: f[6].clone(),
        });
    }
    let supply = m
        .supply
        .iter()
        .map(|(k, v)| v.parse::<u128>().map(|v| (k.clone(), v)).map_err(|_| invalid("manifest: supply")))
        .collect::<io::Result<BTreeMap<_, _>>>()?;
    let g = Genesis {
        chain,
        source: m.source.clone(),
        accounts,
        balances,
        deferred,
        particles,
        links,
        passports,
        supply,
        counts: m.counts.clone(),
        gaps: m.gaps.clone(),
    };
    Ok((m, g))
}
