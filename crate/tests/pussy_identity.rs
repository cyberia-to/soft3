//! The pussy identity pin: `ChainNode::open --chain pussy` boots only the
//! blessed genesis file (particle and block 0 pinned in `chain::params`,
//! values from `conformance/chain-genesis.toml`).

use soft3::chain::net::chain_view::ChainView;
use soft3::chain::net::codec::hex;
use soft3::chain::params::embedded;
use soft3::genesis::file::{Balance, GenesisFile, Params};
use soft3::node::chain_node::{ChainNode, ChainOpts};
use std::path::{Path, PathBuf};

fn opts(home: &Path, genesis: Option<PathBuf>) -> ChainOpts {
    ChainOpts {
        chain: "pussy".into(),
        home: home.to_path_buf(),
        genesis,
        radio: Some("127.0.0.1:0".parse().unwrap()),
        peers: None,
        moniker: "identity-test".into(),
        neuron: None,
        relay: false,
        log_net: false,
        corpus: None,
    }
}

fn fresh_dir(label: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("soft3-pussy-identity-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn refused(label: &str, genesis_bytes: &[u8]) {
    let dir = fresh_dir(label);
    let file = dir.join("given.tade");
    std::fs::write(&file, genesis_bytes).unwrap();
    let home = dir.join("home");
    let err = match ChainNode::open(opts(&home, Some(file))) {
        Ok(_) => panic!("a non-blessed genesis booted as pussy"),
        Err(e) => e.to_string(),
    };
    let pin = embedded("pussy").unwrap();
    assert!(err.contains(&hex(&pin.chain_id)), "{err}");
    assert!(err.contains("conformance/chain-genesis.toml"), "{err}");
    assert!(!home.join("genesis.tade").exists(), "a refused start left a genesis in the home");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn garbage_genesis_is_refused_by_the_particle_check() {
    refused("garbage", b"not the pussy genesis");
}

#[test]
fn well_formed_genesis_naming_pussy_is_refused_by_the_particle_check() {
    let name = "pussy".to_string();
    let balances = vec![Balance { owner: [7; 32], token: soft3::chain::sigs::token(&name), amount: 1_000_000 }];
    let file = GenesisFile {
        params: Params {
            version: 1, name: name.clone(), hrp: name.clone(), denom: name, time: 1_793_885_820,
            bits: soft3::chain::params::POW_LIMIT_BITS, subsidy: soft3::chain::params::subsidy(1_000_000),
            checkpoint_key: [0; 33], checkpoint_sunset: soft3::chain::params::CHECKPOINT_SUNSET,
            pool: soft3::chain::pool::Pool::genesis(),
        },
        balances,
        links: vec![],
    };
    let genesis = soft3::chain::genesis::Genesis::from_file(&file).unwrap();
    assert_ne!(genesis.chain_id, embedded("pussy").unwrap().chain_id);
    refused("forged", &genesis.bytes);
}

#[test]
fn the_blessed_file_boots_when_cached() {
    let cached = [std::env::var("SOFT3_GENESIS_DIR").ok(), std::env::var("HOME").ok().map(|h| format!("{h}/.cache/cyber/genesis"))]
        .into_iter()
        .flatten()
        .map(|d| Path::new(&d).join("pussy.tade"))
        .find(|p| p.exists());
    let Some(path) = cached else {
        eprintln!("skipped: no pussy.tade in $SOFT3_GENESIS_DIR or ~/.cache/cyber/genesis");
        return;
    };
    let dir = fresh_dir("blessed");
    let node = ChainNode::open(opts(&dir.join("home"), Some(path))).expect("the blessed genesis boots");
    let pin = embedded("pussy").unwrap();
    assert_eq!(node.chain.chain_id(), pin.chain_id);
    assert_eq!(node.chain.genesis_id(), pin.genesis_block);
    drop(node);
    let _ = std::fs::remove_dir_all(&dir);
}
