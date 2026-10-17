//! chain-wire §3.1 step 1: a new connection brings the peer's checkpoints.
//! Two in-process nodes on a test genesis with a checkpoint key: A holds a
//! signed checkpoint; B starts afterwards, holding none, and dials A. B
//! must hold A's checkpoint without waiting for the next one to be signed
//! (found by the chain simulator: a late joiner without the checkpoints
//! followed a 140-deep rewrite, simulation audit/network-2026-10.md).

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use soft3::chain::genesis::Genesis;
use soft3::chain::net::chain_view::ChainView;
use soft3::chain::net::frames::Checkpoint;
use soft3::chain::net::kernel::KernelChain;
use soft3::chain::net::mempool::Mempool;
use soft3::chain::net::{Bootstrap, Config, Net};
use soft3::chain::sigs::token;
use soft3::genesis::file::{Balance, GenesisFile, Params};

fn temp(label: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("soft3-cp-connect-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_new_connection_brings_the_peers_checkpoints() {
    let signer = mudra::SigningKey::from_bytes((&[7u8; 32]).into()).unwrap();
    let native = token("cptest");
    let balances = vec![Balance { owner: [1; 32], token: native, amount: 105_120_000_000 }];
    let file = GenesisFile {
        params: Params {
            version: 1,
            name: "cptest".into(),
            hrp: "cptest".into(),
            denom: "cptest".into(),
            time: 1_800_000_000,
            bits: soft3::chain::params::POW_LIMIT_BITS,
            subsidy: soft3::chain::params::subsidy(105_120_000_000),
            checkpoint_key: mudra::cosmos::compressed(signer.verifying_key()),
            checkpoint_sunset: soft3::chain::params::CHECKPOINT_SUNSET,
            pool: soft3::chain::pool::Pool::genesis(),
        },
        balances,
        links: vec![],
    };
    let genesis = Genesis::from_file(&file).unwrap();
    let start = |label: &str, bootstrap: Vec<Bootstrap>| {
        let dir = temp(label);
        let chain: Arc<dyn ChainView> = Arc::new(KernelChain::open(&genesis.bytes, &dir).unwrap());
        let cfg = Config { chain_dir: dir, radio_bind: Some("127.0.0.1:0".parse().unwrap()), relay: false, bootstrap, neuron: [0; 32] };
        Net::start(cfg, chain, Arc::new(Mutex::new(Mempool::new())), false)
    };
    let a = start("a", vec![]).await.unwrap();
    let signed = soft3::chain::select::Checkpoint::sign(&genesis.chain_id, 120, [9; 32], &signer).unwrap();
    let cp = Checkpoint { height: 120, block_id: [9; 32], sig: signed.sig };
    assert!(a.publish_checkpoint(cp.clone()).await, "A holds the checkpoint");
    let b = start("b", vec![Bootstrap { name: "a".into(), id: a.id(), addrs: a.bound() }]).await.unwrap();
    let t0 = Instant::now();
    while b.checkpoints().is_empty() && t0.elapsed() < Duration::from_secs(30) {
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    assert_eq!(b.checkpoints(), vec![cp], "B fetched A's checkpoint on connecting");
    b.shutdown().await;
    a.shutdown().await;
}
