//! Genesis pipeline tests. Pure tests always run; the snapshot-backed test
//! runs when the space-pussy snapshot manifest and its fetched cache exist
//! (`~/cyber/space-pussy/snapshot/manifest.json`, `~/cyber/.cache/snapshots/pussy`)
//! and skips otherwise — no network inside `cargo test`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use super::derive::{OwnerKind, legacy_owner, module_address, particle_of_cid};
use super::file::{self, Balance, GenesisFile, Link, Params};
use super::{Chain, Options, PUSSY, build, hash32, hex, state, token_of, unhex, verify};

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
    let n = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("soft3-genesis-{label}-{n}"))
}

fn id32(hex_str: &str) -> [u8; 32] {
    unhex(hex_str).unwrap().try_into().unwrap()
}

// ── chain-conformance anchors ────────────────────────────────────────────

#[test]
fn hemera_anchors_of_chain_conformance_section_1() {
    assert_eq!(hex(&hash32(b"")), "a67a71b221e6bdd6442a20432bf5d74c885d89e5dfbeec3ec4e334cb806d563c");
    assert_eq!(hex(&token_of("boot")), "10275cf6703891a515f64ada465137cd36f2a5d3f0b99a8dafc62e6b95ad1269");
    assert_eq!(hex(&token_of("pussy")), "8cffbea0ac348d13febcee5b2848f916f501237d3ed8966167a31bafddf18ffa");
    assert_eq!(hex(&hash32(&[0u8; 168])), "ba560d1fd2ed380e09ef8e526e8817504cce6e71363c12081537751e4f761754");
}

#[test]
fn header_vector_of_chain_conformance_section_5() {
    let h = file::header_bytes(1, 1, 1_762_349_000, 0x1d10_0000, [0u8; 16], &[0x11; 32], &[0x22; 32], &[0x33; 32], &[0x44; 32]);
    assert_eq!(&h[..24], &unhex(concat!("01000000", "0100000000000000", "c84f0b6900000000", "0000101d")).unwrap()[..]);
    assert_eq!(hex(&file::block_id(&h)), "bdfbbff94e6e17eec9169932276825a0f4957f6f3a6af9030439979eacaeb58d");
    let mut nonce = [0u8; 16];
    nonce[..8].copy_from_slice(&40_165_933u64.to_le_bytes());
    let h = file::header_bytes(1, 1, 1_793_885_850, 0x1e01_0000, nonce, &[0x11; 32], &[0x22; 32], &[0x33; 32], &[0x44; 32]);
    assert_eq!(hex(&file::block_id(&h)), "d1eec40d1db983bb3df0946890538bec4ba9712fb26b82316efc6b162f000000");
}

#[test]
fn legacy_owner_vectors_of_chain_conformance_section_6() {
    assert_eq!(
        hex(&legacy_owner("pussy", &[0x99; 20])),
        "d5e54c499456ee690e5c82a3d2119a8180a2e13f832df5254757a41f5e6d509d"
    );
    let account_id = unhex("28ff5c6d57d8cfd492b6fb42614536ed648e01fd").unwrap();
    assert_eq!(
        hex(&legacy_owner("pussy", &account_id)),
        "cca6f468fe834711fc1e672c21ba95ee7028ed9af6915fbc7b95b449ef0cdbc0"
    );
    let (hrp, data) = bech32::decode("pussy19rl4cm2hmr8afy4kldpxz3fka4jguq0a4l8rzm").unwrap();
    assert_eq!(hrp.as_str(), "pussy");
    assert_eq!(data, account_id);
    let pk = unhex("024f4e2ad99c34d60b9ba6283c9431a8418af8673212961f97a77b6377fcd05b62").unwrap();
    assert_eq!(
        hex(&mudra::claim::neuron_of(&pk)),
        "31284576301bacb9c2cad6379244f856dc32c751c38c3b0af4c5c886fd0ed741"
    );
    assert_eq!(mudra::cosmos::account_id(&pk).to_vec(), account_id);
}

// ── file codec ───────────────────────────────────────────────────────────

fn sample() -> GenesisFile {
    let params = Params {
        version: 1,
        name: "pussy".into(),
        hrp: "pussy".into(),
        denom: "pussy".into(),
        time: 1_793_885_820,
        bits: file::GENESIS_BITS,
        subsidy: file::subsidy_of(1_000_000).unwrap(),
        checkpoint_key: [0u8; 33],
        checkpoint_sunset: file::CHECKPOINT_SUNSET,
    };
    GenesisFile {
        params,
        balances: vec![Balance { owner: [0x66; 32], token: token_of("pussy"), amount: 1_000_000 }],
        links: vec![],
    }
}

#[test]
fn genesis_file_round_trips_and_is_strict() {
    let g = sample();
    let bytes = g.encode().unwrap();
    assert_eq!(&bytes[..3], &[0x1f, b'*', b'k']);
    assert_eq!(GenesisFile::decode(&bytes).unwrap(), g);
    // trailing byte, unknown frame, unsorted / zero balances, wrong dialect
    let mut t = bytes.clone();
    t.push(0);
    assert!(GenesisFile::decode(&t).is_err());
    let mut u = bytes.clone();
    u.extend(tade::Chunk::new(b'#', b'z', tade::bytes::Bytes::new()).encode());
    assert!(GenesisFile::decode(&u).is_err());
    let mut dup = sample();
    dup.balances.push(dup.balances[0]);
    assert!(dup.encode().is_err());
    let mut zero = sample();
    zero.balances[0].amount = 0;
    assert!(zero.encode().is_err());
    let mut wrong = bytes.clone();
    wrong[4 + 1] = b'x'; // inside the dialect string
    assert!(GenesisFile::decode(&wrong).is_err());
}

#[test]
fn conformance_net_genesis_of_section_8() {
    // one `b` frame (owner 0x66 × 32, token("pussy"), 1,000,000), no links
    let g = sample();
    let bytes = g.encode().unwrap();
    let s = state::load(&g);
    assert_eq!(s.neurons.len(), 1);
    assert_eq!(s.coins[&token_of("pussy")].total_supply, 1_000_000);
    assert_eq!(s.balances[&bbg::balance_key(&[0x66; 32], &token_of("pussy"))], 1_000_000);
    let root = s.root();
    let particle = hash32(&bytes);
    let header = file::block0_header(g.params.time, &root, &particle);
    assert_eq!(&header[72..104], &particle);
    assert_eq!(&header[136..168], &particle);
    assert_eq!(&header[104..136], &root);
    let _ = file::block_id(&header);
    // links apply with amount 0 at chain height 0 and create neuron records
    let mut g2 = sample();
    g2.links.push(Link { neuron: [0x77; 32], p: [0x01; 32], q: [0x02; 32], height: 5 });
    let s2 = state::load(&g2);
    assert_eq!(s2.neurons.len(), 2);
    assert_eq!(s2.axon_edges.len(), 1);
    assert_ne!(s2.root(), root);
}

#[test]
fn module_addresses_match_the_cosmos_sdk() {
    assert_eq!(module_address("pussy", "bonded_tokens_pool").unwrap(), "pussy1fl48vsnmsdzcv85q5d2q4z5ajdha8yu3uyr3ye");
    assert_eq!(module_address("bostrom", "not_bonded_tokens_pool").unwrap(), "bostrom1tygms3xhhs3yv487phx3dw4a95jn7t7lzgpd3y");
    assert_eq!(Chain::parse("space-pussy"), Some(PUSSY));
    assert_eq!(particle_of_cid("QmMissing"), *hemera::hash(b"QmMissing").as_bytes());
}

// ── snapshot-backed (space-pussy) ────────────────────────────────────────

#[test]
fn pussy_genesis_is_deterministic_conserving_and_bijective() {
    let Some((snapshot, cache)) = pussy_snapshot() else {
        eprintln!("skipped: space-pussy snapshot not cached");
        return;
    };
    let dir = tmp("pussy");
    std::fs::create_dir_all(&dir).unwrap();
    let a = dir.join("a.tade");
    let b = dir.join("b.tade");
    let options = Options { cache, root: true, export: Some(dir.join("export")), time: None };
    let ra = build(PUSSY, &snapshot, &a, &options).unwrap();
    let rb = build(PUSSY, &snapshot, &b, &Options { export: None, ..options.clone() }).unwrap();

    // determinism: byte-identical files, same particle, root and block id
    assert_eq!(std::fs::read(&a).unwrap(), std::fs::read(&b).unwrap());
    assert_eq!(ra.particle, rb.particle);
    assert_eq!(ra.state_root, rb.state_root);
    assert_eq!(ra.block0_id, rb.block0_id);
    assert!(ra.state_root.is_some(), "pussy root must commit: {:?}", ra.root_blocked);

    // verify re-derives particle, root and block id from the file alone
    let v = verify(&a, true).unwrap();
    assert_eq!(v.particle, ra.particle);
    assert_eq!(v.state_root, ra.state_root);
    assert_eq!(v.block0_id, ra.block0_id);

    let bytes = std::fs::read(&a).unwrap();
    let g = GenesisFile::decode(&bytes).unwrap();
    assert_eq!(g.params.time, 1_793_885_820);
    assert_eq!(g.params.name, "pussy");
    assert_eq!(g.params.bits, 0x1e01_0000);
    let count = |k: &str| *ra.counts.get(k).unwrap_or_else(|| panic!("count {k}"));

    // conservation: Σ per token == supply.json, S0 == bank supply at halt
    assert_eq!(ra.s0, "1099389810279114036");
    assert_eq!(g.native_supply().to_string(), ra.s0);
    assert_eq!(g.params.subsidy, 1_099_389_810_279_114_036u64 / 105_120_000);
    assert_eq!(ra.tokens["pussy"].1, ra.s0);
    assert_eq!(ra.tokens["pussy"].0, "8cffbea0ac348d13febcee5b2848f916f501237d3ed8966167a31bafddf18ffa");
    let mut sums: BTreeMap<[u8; 32], u128> = BTreeMap::new();
    for bal in &g.balances {
        *sums.entry(bal.token).or_insert(0) += u128::from(bal.amount);
    }
    for (denom, (token_hex, sum)) in &ra.tokens {
        assert_eq!(sums[&id32(token_hex)].to_string(), *sum, "{denom}");
    }
    assert!(!ra.tokens.keys().any(|d| d.starts_with("pool")), "pool coins dissolved");

    // bijection with the snapshot counts
    assert_eq!(count("l_frames_source"), 29_112);
    assert_eq!(count("l_frames_passports"), 1);
    assert_eq!(g.links.len() as u64, 29_113);
    assert_eq!(count("accounts"), count("source_pubkeys_rows"));
    assert_eq!(count("accounts"), 616);
    let authors: BTreeSet<[u8; 32]> = g.links.iter().map(|l| l.neuron).collect();
    let owners: BTreeSet<[u8; 32]> = g.balances.iter().map(|b| b.owner).collect();
    assert!(authors.iter().all(|n| owners.contains(n)), "every link author is a snapshot account with a balance");
    assert_eq!(count("particles"), 16_271);
    let s = state::load(&g);
    assert_eq!(s.coins.len() as u64, count("tokens"));
    assert_eq!(hex(&s.root()), ra.state_root.clone().unwrap());

    // spot checks via the debug export
    let accounts = std::fs::read_to_string(dir.join("export").join("accounts.tsv")).unwrap();
    let row = |addr: &str| accounts.lines().find(|l| l.starts_with(addr)).map(|l| l.split('\t').map(str::to_string).collect::<Vec<_>>());
    let contract = row("pussy1qyl0j7a24amk8k8gcmvv07y2zjx7nkcwpk73js24euh64hkja6esdnn3k6").unwrap();
    assert_eq!(contract[2], "legacy", "the passport contract is dormant");
    let first = row("pussy1p0r7uxstcw8ehrwuj4kn8qzzs0yypsjwvyncka").unwrap();
    assert_eq!(first[2], "neuron");
    assert_eq!(hex(&g.links[1].neuron), first[1], "first source link author");
    assert_eq!(g.links[1].height, 368);
    assert_eq!(g.links[0].height, 0, "passport link first");
    // the known-answer key never holds a balance (chain-conformance §10)
    assert!(row("pussy19rl4cm2hmr8afy4kldpxz3fka4jguq0a4l8rzm").is_none());
    for l in accounts.lines().skip(1) {
        let f: Vec<&str> = l.split('\t').collect();
        let (hrp, data) = bech32::decode(f[0]).unwrap();
        assert_eq!(hrp.as_str(), "pussy");
        match f[2] {
            "neuron" => {
                let pk = unhex(f[4]).unwrap();
                assert_eq!(id32(f[1]), mudra::claim::neuron_of(&pk));
                assert_eq!(mudra::cosmos::address(&pk, "pussy").unwrap(), f[0]);
            }
            _ => assert_eq!(id32(f[1]), legacy_owner("pussy", &data)),
        }
    }
    assert_eq!(count("staking_pool_dust"), 367);
    assert!(ra.gaps.contains_key("cyberlinks_unrecoverable"));
    let _ = OwnerKind::Neuron;
    let _ = std::fs::remove_dir_all(&dir);
}
