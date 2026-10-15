//! Genesis pipeline tests. Pure tests always run; the snapshot-backed tests
//! run when the space-pussy snapshot manifest and its fetched cache exist
//! (`~/cyber/space-pussy/snapshot/manifest.json`, `~/cyber/.cache/snapshots/pussy`)
//! and skip otherwise — no network inside `cargo test`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use super::tables::{
    AccountKind, card_of, claimable_id, hex, module_address, particle_of_cid, token_of, unhex32,
};
use super::{Chain, Options, PUSSY, build, state, tables, verify};

fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into())).join("cyber")
}

fn pussy_snapshot() -> Option<(PathBuf, PathBuf)> {
    let snapshot = home().join("space-pussy").join("snapshot");
    let cache = Options::default_cache(&PUSSY);
    let ready = snapshot.join("manifest.json").is_file()
        && super::fetch::NEEDED.iter().all(|f| cache.join(f).is_file());
    ready.then_some((snapshot, cache))
}

fn tmp(label: &str) -> PathBuf {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("soft3-genesis-{label}-{n}"))
}

// ── pure ─────────────────────────────────────────────────────────────────

#[test]
fn chain_names() {
    assert_eq!(Chain::parse("pussy"), Some(PUSSY));
    assert_eq!(Chain::parse("space-pussy"), Some(PUSSY));
    assert_eq!(Chain::parse("bostrom").map(|c| c.native), Some("boot"));
    assert_eq!(Chain::parse("cosmos"), None);
}

#[test]
fn module_addresses_match_the_cosmos_sdk() {
    // bech32(hrp, sha256(name)[..20]) — checked against the live bank rows of
    // both burials (the pools hold the delegated supply).
    assert_eq!(
        module_address("pussy", "bonded_tokens_pool").unwrap(),
        "pussy1fl48vsnmsdzcv85q5d2q4z5ajdha8yu3uyr3ye"
    );
    assert_eq!(
        module_address("bostrom", "not_bonded_tokens_pool").unwrap(),
        "bostrom1tygms3xhhs3yv487phx3dw4a95jn7t7lzgpd3y"
    );
}

#[test]
fn identities_are_domain_separated_and_stable() {
    let a = claimable_id("pussy1qqnlsrpk5uhf6ktdq08g52tjeqmlmrsyvjj6pc");
    assert_ne!(a, particle_of_cid("pussy1qqnlsrpk5uhf6ktdq08g52tjeqmlmrsyvjj6pc"));
    assert_eq!(a, claimable_id("pussy1qqnlsrpk5uhf6ktdq08g52tjeqmlmrsyvjj6pc"));
    assert_ne!(card_of("pussy", "1"), card_of("bostrom", "1"));
    assert_ne!(token_of("pussy"), token_of("boot"));
    // the black-hole particle is file::readdress's no-bytes branch: hemera(cid)
    assert_eq!(
        particle_of_cid("QmMissing"),
        *hemera::hash(b"QmMissing").as_bytes()
    );
    assert_eq!(unhex32(&hex(&a)), Some(a));
}

#[test]
fn neuron_of_a_known_pubkey_matches_mudra() {
    // mudra's known-answer vector: this pubkey derives this cosmos address.
    let pk = tables::unhex("02950e1cdfcb133d6024109fd489f734eeb4502418e538c28481f22bce276f248c").unwrap();
    assert_eq!(
        mudra::cosmos::address(&pk, "cosmos").unwrap(),
        "cosmos10s4mg25tu6termrk8egltfyme4q7sg3her239u"
    );
    assert_eq!(mudra::claim::neuron_of(&pk), *hemera::hash(&pk).as_bytes());
}

#[test]
fn manifest_round_trips_through_canonical_json() {
    let m = tables::Manifest {
        format: tables::FORMAT.into(),
        chain: "pussy".into(),
        hrp: "pussy".into(),
        native: "pussy".into(),
        bootloader: "space-pussy".into(),
        source: tables::Source {
            manifest_sha256: "00".into(),
            halt_height: 1,
            halt_time: "t".into(),
            files: BTreeMap::new(),
        },
        tables: BTreeMap::new(),
        counts: BTreeMap::new(),
        supply: BTreeMap::new(),
        decisions: BTreeMap::new(),
        gaps: BTreeMap::new(),
        state_root: None,
        state_counts: None,
        state_fields: None,
        root_blocked: None,
    };
    let a = tables::canonical(&m).unwrap();
    let back: tables::Manifest = serde_json::from_slice(&a).unwrap();
    assert_eq!(back, m);
    assert_eq!(tables::canonical(&back).unwrap(), a);
    assert!(!a.contains(&b'\n'));
}

// ── snapshot-backed (space-pussy) ────────────────────────────────────────

#[test]
fn pussy_genesis_is_deterministic_conserving_and_bijective() {
    let Some((snapshot, cache)) = pussy_snapshot() else {
        eprintln!("skipped: space-pussy snapshot not cached");
        return;
    };
    let options = Options { cache, root: true, genesis_time: super::LAUNCH_UNIX };
    let a = tmp("a");
    let b = tmp("b");
    let ra = build(PUSSY, &snapshot, &a, &options).unwrap();
    let rb = build(PUSSY, &snapshot, &b, &options).unwrap();

    // determinism: two builds, same hash, same root, same bytes
    assert_eq!(ra.genesis_hash, rb.genesis_hash);
    assert_eq!(ra.root, rb.root);
    for f in ["manifest.json", "accounts.tsv", "balances.tsv", "cyberlinks.tsv", "particles.tsv", "passports.tsv"] {
        assert_eq!(std::fs::read(a.join(f)).unwrap(), std::fs::read(b.join(f)).unwrap(), "{f}");
    }

    // verify re-derives the same hash and root from the directory alone
    let (hash, root) = verify(&a, true).unwrap();
    assert_eq!(hash, ra.genesis_hash);
    assert_eq!(root, ra.root);

    let (m, g) = tables::read(&a).unwrap();
    let count = |k: &str| *m.counts.get(k).unwrap_or_else(|| panic!("count {k}"));

    // conservation: genesis pussy == bank supply at halt, exact integers
    let sum: u128 = g.balances.iter().filter(|b| b.denom == "pussy").map(|b| u128::from(b.amount)).sum();
    assert_eq!(sum.to_string(), m.supply["pussy"]);
    assert_eq!(m.supply["pussy"], "1099389810279114036");

    // bijection with the snapshot counts
    assert_eq!(count("cyberlinks"), 29_112, "restored links per manifest");
    assert_eq!(count("cyberlinks"), count("source_cyberlinks_rows"));
    assert_eq!(g.links.len() as u64, count("cyberlinks"));
    assert_eq!(count("accounts"), count("source_pubkeys_rows"), "every auth account, once");
    assert_eq!(g.accounts.len() as u64, count("accounts"));
    assert_eq!(count("accounts_neuron") + count("accounts_claimable"), count("accounts"));
    let particles: BTreeSet<[u8; 32]> = g.links.iter().flat_map(|l| [l.from, l.to]).collect();
    assert_eq!(particles.len() as u64, count("particles"));
    assert_eq!(g.particles.len() as u64, count("particles"));
    assert_eq!(count("passports"), count("source_passports_rows"));
    let addresses: BTreeSet<&str> = g.accounts.iter().map(|a| a.address.as_str()).collect();
    assert_eq!(addresses.len(), g.accounts.len(), "addresses are unique");
    let owners: BTreeSet<[u8; 32]> = g.accounts.iter().map(|a| a.owner).collect();
    assert_eq!(owners.len(), g.accounts.len(), "owner ids are unique");

    // the loaded state mirrors the tables
    let s = state::load(&g);
    let c = state::counts(&s);
    assert_eq!(c["neurons"], count("accounts"));
    assert_eq!(c["files"], count("particles"));
    assert_eq!(c["signals"], count("signals"));
    assert_eq!(c["cards"], count("passports"));
    assert_eq!(c["coins"], 1);
    assert_eq!(hex(&s.root()), ra.root.clone().unwrap());

    // spot checks: known addresses
    let by_addr: BTreeMap<&str, &tables::Account> = g.accounts.iter().map(|a| (a.address.as_str(), a)).collect();
    let passport_contract = by_addr["pussy1qyl0j7a24amk8k8gcmvv07y2zjx7nkcwpk73js24euh64hkja6esdnn3k6"];
    assert_eq!(passport_contract.kind, AccountKind::Claimable, "a contract never signed");
    let bonded = module_address("pussy", "bonded_tokens_pool").unwrap();
    let dust: u64 = g.balances.iter().filter(|b| b.address == bonded).map(|b| b.amount).sum();
    assert_eq!(dust, 367, "pool dust after delegations became liquid");
    assert_eq!(count("delegated"), 87_936_873_104_381_212);
    let first = &g.links[0];
    assert_eq!(first.address, "pussy1p0r7uxstcw8ehrwuj4kn8qzzs0yypsjwvyncka");
    assert_eq!(first.height, 368);
    assert_eq!(by_addr[first.address.as_str()].kind, AccountKind::Neuron);
    assert_eq!(first.neuron, by_addr[first.address.as_str()].owner);
    for a in g.accounts.iter().filter(|a| a.kind == AccountKind::Neuron) {
        let pk = a.pubkey.as_ref().unwrap();
        assert_eq!(a.owner, mudra::claim::neuron_of(pk));
        assert_eq!(mudra::cosmos::address(pk, "pussy").unwrap(), a.address);
    }
    for a in g.accounts.iter().filter(|a| a.kind == AccountKind::Claimable) {
        assert_eq!(a.owner, claimable_id(&a.address));
    }
    assert!(m.gaps.contains_key("cyberlinks_unrecoverable"));
    let _ = std::fs::remove_dir_all(&a);
    let _ = std::fs::remove_dir_all(&b);
}
