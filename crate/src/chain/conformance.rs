//! The blessed vectors of chain-conformance.md (§6 signed transactions, §8
//! state, §10 genesis) as `conformance/chain.snap` entries, generated from
//! the reference implementation. `soft3 conformance bless chain` writes
//! the file; `soft3 conformance check chain` regenerates and compares.
//!
//! Entries are `hemera` fingerprints of canonical bytes where the stack's
//! snapshot discipline wants fingerprints, and the 32-byte value itself
//! where the spec names an id or a root (a root is already a hash). Each
//! line: `<key> :: "<scenario>"  tier=gamma  h<hex32>`, keys padded to the
//! longest, sorted by key — the `.snap` shape of conformance/specs/README.md.

use std::collections::BTreeMap;

use crate::genesis::file::{Balance, GenesisFile};

use super::api::ChainApi;
use super::bbg_state::BbgBacked;
use super::engine::Chain;
use super::genesis::{Genesis, Params};
use super::header::hash32;
use super::params::GENESIS_BITS;
use super::sigs::{self, neuron_of, token};
use super::state::StateBackend;
use super::tx::{Body, Tx};

/// One snapshot line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub key: String,
    pub scenario: String,
    pub value: [u8; 32],
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn unhex32(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(out)
}

/// The known-answer key of chain-conformance §6 (the all-`abandon` BIP-39
/// phrase at `m/44'/118'/0'/0/0`), as a raw secret so the `bridge` feature
/// is not needed: the secret the phrase derives to.
pub const KNOWN_ANSWER_SECRET: [u8; 32] = [
    0xc4, 0xa4, 0x8e, 0x2f, 0xce, 0x14, 0x81, 0xcd, 0x32, 0x94, 0xb4, 0x49, 0x0f, 0x66, 0x78, 0x09, 0x0e, 0xa9, 0x8d, 0x3d, 0x0e, 0x5c, 0xd9,
    0x84, 0x55, 0x8a, 0xb0, 0x96, 0x87, 0x41, 0xb1, 0x04,
];

/// The §6 key, from the pinned secret.
pub fn known_answer_key() -> mudra::SigningKey {
    mudra::SigningKey::from_bytes((&KNOWN_ANSWER_SECRET).into()).expect("pinned secret is a valid scalar")
}

/// The conformance net of §8: one `b` frame (`0x66 × 32`, `token("pussy")`,
/// 1,000,000), no links, genesis time 0, `GENESIS_BITS`, no checkpoints.
pub fn conformance_genesis() -> Genesis {
    Genesis::from_file(&GenesisFile {
        params: Params {
            version: 1,
            name: "conformance".into(),
            hrp: "pussy".into(),
            denom: "pussy".into(),
            time: 0,
            bits: GENESIS_BITS,
            subsidy: 0,
            checkpoint_key: [0; 33],
            checkpoint_sunset: 0,
        },
        balances: vec![Balance { owner: [0x66; 32], token: token("pussy"), amount: 1_000_000 }],
        links: vec![],
    })
    .expect("the conformance genesis is canonical")
}

/// §6 blessed signed transactions: the transfer and the cyberlink signed
/// by the known-answer key under `chain = 0x44 × 32`. ECDSA here is
/// RFC 6979 deterministic, so the bytes are stable.
pub fn signed_vectors() -> Vec<(String, Vec<u8>)> {
    let chain = [0x44u8; 32];
    let key = known_answer_key();
    let from = neuron_of(&mudra::cosmos::compressed(key.verifying_key()));
    let mut transfer = Tx { body: Body::Transfer { from, nonce: 0, token: token("pussy"), amount: 5, to: [0x77; 32], fee: 224 }, sig: vec![] };
    sigs::sign(&chain, &mut transfer, &key).expect("own subject");
    let mut link = Tx {
        body: Body::Cyberlink { from, nonce: 0, token: token("pussy"), amount: 0, p: token("boot"), q: token("pussy"), valence: 1, fee: 257 },
        sig: vec![],
    };
    sigs::sign(&chain, &mut link, &key).expect("own subject");
    vec![("signed transfer".into(), transfer.encode()), ("signed cyberlink".into(), link.encode())]
}

/// The §8 state scenarios on the conformance net, run on the engine.
fn state_entries() -> Vec<Entry> {
    let mut out = Vec::new();
    let push = |out: &mut Vec<Entry>, scenario: &str, value: [u8; 32]| {
        out.push(Entry { key: "chain::state@v1".into(), scenario: scenario.into(), value });
    };
    let mut empty = BbgBacked::new();
    push(&mut out, "empty state root", empty.root());
    let g = conformance_genesis();
    push(&mut out, "conformance genesis file", hash32(&g.bytes));
    let mut s = g.state();
    let root0 = s.root();
    push(&mut out, "conformance genesis state_root", root0);
    push(&mut out, "conformance block0 id", g.block0(root0).id());
    out
}

/// §10 genesis entries from a genesis file (the pussy file), or the
/// committed values when no file is given (`None` keeps them as they are).
pub fn genesis_entries(name: &str, bytes: &[u8]) -> Result<Vec<Entry>, String> {
    let g = Genesis::decode(bytes).map_err(|e| e.to_string())?;
    let mut s = g.state();
    let root = s.root();
    Ok(vec![
        Entry { key: "chain::genesis@v1".into(), scenario: format!("{name} particle"), value: g.chain_id },
        Entry { key: "chain::genesis@v1".into(), scenario: format!("{name} state_root"), value: root },
        Entry { key: "chain::genesis@v1".into(), scenario: format!("{name} block0_id"), value: g.block0(root).id() },
    ])
}

/// Every entry the reference implementation can produce without the
/// pinned genesis file.
pub fn entries() -> Vec<Entry> {
    let mut out = Vec::new();
    for (scenario, bytes) in signed_vectors() {
        out.push(Entry { key: "chain::tx@v1".into(), scenario, value: hash32(&bytes) });
    }
    out.extend(state_entries());
    out
}

/// Parse a committed `chain.snap`.
pub fn parse(src: &str) -> Vec<Entry> {
    let mut out = Vec::new();
    for line in src.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, rest)) = line.split_once(" :: \"") else { continue };
        let Some((scenario, rest)) = rest.split_once('"') else { continue };
        let Some(h) = rest.split_whitespace().find_map(|w| w.strip_prefix('h')) else { continue };
        if let Some(value) = unhex32(h) {
            out.push(Entry { key: key.trim().into(), scenario: scenario.into(), value });
        }
    }
    out
}

/// Render entries in the `.snap` shape: sorted by `(key, scenario)`, keys
/// padded to the longest.
pub fn render(entries: &[Entry], date: &str) -> String {
    let mut sorted: Vec<&Entry> = entries.iter().collect();
    sorted.sort_by(|a, b| (&a.key, &a.scenario).cmp(&(&b.key, &b.scenario)));
    let label = |e: &Entry| format!("{} :: \"{}\"", e.key, e.scenario);
    let width = sorted.iter().map(|e| label(e).len()).max().unwrap_or(0);
    let mut out = String::new();
    out.push_str(&format!("# generated by `soft3 conformance bless chain` ({date}); do not edit by hand\n"));
    out.push_str("# chain.snap — chain-conformance.md blessed vectors: §6 signed txs (fingerprints), §8 state roots and ids, §10 genesis\n");
    out.push_str("# (particle = chain id, block-0 root and id). Roots move with bbg's DIMENSION_VERSION; a moved entry is a consensus change.\n");
    for e in sorted {
        out.push_str(&format!("{:<width$}  tier=gamma  h{}\n", label(e), hex(&e.value), width = width));
    }
    out
}

/// Merge: entries from `current` replace committed ones of the same
/// `(key, scenario)`; committed entries not regenerated (genesis without a
/// file) are kept.
pub fn merged(committed: &[Entry], current: &[Entry]) -> Vec<Entry> {
    let mut map: BTreeMap<(String, String), [u8; 32]> = committed.iter().map(|e| ((e.key.clone(), e.scenario.clone()), e.value)).collect();
    for e in current {
        map.insert((e.key.clone(), e.scenario.clone()), e.value);
    }
    map.into_iter().map(|((key, scenario), value)| Entry { key, scenario, value }).collect()
}

/// Differences between committed and current entries: `(label, committed, current)`.
pub fn drift(committed: &[Entry], current: &[Entry]) -> Vec<(String, Option<[u8; 32]>, [u8; 32])> {
    let map: BTreeMap<(String, String), [u8; 32]> = committed.iter().map(|e| ((e.key.clone(), e.scenario.clone()), e.value)).collect();
    let mut out = Vec::new();
    for e in current {
        let was = map.get(&(e.key.clone(), e.scenario.clone())).copied();
        if was != Some(e.value) {
            out.push((format!("{} :: \"{}\"", e.key, e.scenario), was, e.value));
        }
    }
    out
}

/// Today's UTC date as `YYYY-MM-DD` (civil-from-days, no time crate).
pub fn utc_date() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let z = (secs / 86_400) as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

/// The 200-block simulation of §8 ("conservation") and the §9 checks run
/// as engine tests (`chain::tests`); this smoke check runs the conformance
/// net one block forward and confirms the state root the engine commits
/// equals a fresh transition — the §8 "after block 1" row in miniature.
pub fn block1_root() -> [u8; 32] {
    let g = conformance_genesis();
    let mut dir = std::env::temp_dir();
    dir.push(format!("soft3-chain-conformance-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut chain = Chain::open(&dir, &g.bytes, true).expect("conformance net opens");
    let tpl = chain.header_template([0x55; 32], 1, &[]).expect("template");
    let _ = std::fs::remove_dir_all(&dir);
    tpl.header.state_root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_answer_secret_is_the_abandon_key() {
        let key = known_answer_key();
        let pk = mudra::cosmos::compressed(key.verifying_key());
        assert_eq!(hex(&pk), "024f4e2ad99c34d60b9ba6283c9431a8418af8673212961f97a77b6377fcd05b62");
        let derived = mudra::spell::cosmos_key(super::super::sigs::tests::ABANDON, "").unwrap();
        assert_eq!(derived.to_bytes().as_slice(), &KNOWN_ANSWER_SECRET, "the pinned secret is the phrase's key");
        assert!(utc_date().len() == 10);
    }

    #[test]
    fn signed_vectors_verify_and_are_deterministic() {
        let a = signed_vectors();
        let b = signed_vectors();
        assert_eq!(a, b, "RFC 6979 signatures are deterministic");
        for (_, bytes) in &a {
            let tx = Tx::decode(bytes).unwrap();
            assert!(sigs::verify(&[0x44; 32], &tx));
            assert!(!sigs::verify(&[0x45; 32], &tx));
        }
        assert_eq!(a[0].1.len(), 224);
        assert_eq!(a[1].1.len(), 257);
    }

    #[test]
    fn render_parse_round_trip_and_drift() {
        let e = entries();
        let text = render(&e, "2026-10-15");
        let back = parse(&text);
        let mut sorted = e.clone();
        sorted.sort_by(|a, b| (&a.key, &a.scenario).cmp(&(&b.key, &b.scenario)));
        assert_eq!(back, sorted);
        assert!(drift(&back, &e).is_empty());
        let mut moved = e.clone();
        moved[0].value[0] ^= 1;
        assert_eq!(drift(&back, &moved).len(), 1);
        let committed = parse("chain::genesis@v1 :: \"pussy particle\"    tier=gamma  h2a83e54fe395ffbd924fc74a54a5c805eeec5396f97bf4c73fced8631a9e61b5\n");
        assert_eq!(committed.len(), 1);
        assert_eq!(merged(&committed, &e).len(), e.len() + 1);
    }

    /// The committed snapshot must match this build (the `chain-conformance`
    /// feature's acceptance): every regenerated entry equals its committed
    /// line, and the pussy genesis entries when the file is cached.
    #[test]
    fn committed_chain_snap_matches_this_build() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../conformance/chain.snap");
        let committed = parse(&std::fs::read_to_string(path).expect("conformance/chain.snap"));
        let mut current = entries();
        let home = std::env::var("HOME").unwrap_or_default();
        let pussy = std::path::Path::new(&home).join("cyber/.cache/genesis/pussy.tade");
        if let Ok(bytes) = std::fs::read(&pussy) {
            current.extend(genesis_entries("pussy", &bytes).unwrap());
        }
        let d = drift(&committed, &current);
        assert!(d.is_empty(), "chain.snap drift (re-bless with `soft3 conformance bless chain` and review): {d:?}");
        assert!(committed.iter().any(|e| e.scenario == "pussy particle"));
    }
}

