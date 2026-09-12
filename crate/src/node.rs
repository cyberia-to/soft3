//! Local HTTP adapter over Cybergraph's durable native coordinator.
//!
//! Acceptance, economics and recovery are owned by Cybergraph and BBG.
//! See `soft3/specs/native-node.md` for the HTTP and storage lifecycle contract.

mod genesis;
mod http;
mod requests;
mod routes;

use std::io;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use cybergraph::native::{ImportReport, NativeNode};
use cybergraph::{NeuronId, Particle};

use crate::network::Network;

/// Runtime adapter for one local spacepussy-test node.
pub struct Node {
    moniker: String,
    network: Network,
    genesis_unix: u64,
    native: NativeNode,
    failed: bool,
}

impl Node {
    pub fn open(home: PathBuf, moniker: String) -> io::Result<Self> {
        std::fs::create_dir_all(&home)?;
        let database = home.join("bbg");
        let legacy = home.join("log");
        if legacy.try_exists()? && !database.try_exists()? {
            return Err(io::Error::other(
                "legacy log requires cyber storage import-legacy",
            ));
        }
        let genesis = genesis::load(&home, !database.try_exists()?)?;
        let native = NativeNode::open(&database, &genesis.canonical).map_err(io::Error::other)?;
        if legacy.try_exists()? {
            let log = genesis::read_bounded(&legacy, genesis::MAX_LEGACY_LOG)?;
            if !native
                .legacy_source_matches(&log)
                .map_err(io::Error::other)?
            {
                return Err(io::Error::other("legacy log differs from the imported source; explicit storage recovery is required"));
            }
        }
        Ok(Self {
            moniker,
            network: Network::SpacePussyTest,
            genesis_unix: genesis.time,
            native,
            failed: false,
        })
    }

    pub fn height(&self) -> u64 {
        self.native.graph().bbg.state.height
    }

    pub fn root_hex(&self) -> String {
        hex(&self.native.graph().bbg.state.root())
    }

    pub fn signal_count(&self) -> u64 {
        self.native.graph().bbg.state.signals.len() as u64
    }

    pub fn intent_count(&self) -> u64 {
        self.native.graph().bbg.state.intents.len() as u64
    }

    pub fn neuron_count(&self) -> u64 {
        self.native.graph().bbg.state.neurons.len() as u64
    }

    pub fn particle_count(&self) -> u64 {
        self.native.graph().bbg.state.particles.len() as u64
    }

    pub fn axon_count(&self) -> u64 {
        self.native.graph().bbg.state.axons_out.len() as u64
    }

    pub fn balance(&self, neuron: &NeuronId) -> u64 {
        self.native.balance(neuron)
    }

    pub fn pi_supply(&self) -> u64 {
        self.native.supply()
    }

    pub fn status_cybermark(&self) -> String {
        let stats = self.native.graph().bbg.statistics();
        format!(
            "---\nparticle: status\nchain: {}\nprotocol: soft3/spacepussy-test/v2\nengine: cybergraph+bbg\nversion: {}\nmoniker: {}\nheight: {}\ngenesis: {}\nbbg-root: {}\nsignals: {}\nneurons: {}\nparticles: {}\naxons: {}\nnode-count: {}\nmax-degree: {}\nrole: {}\ndenom: {}\nprefix: {}\nrpc: {}\ncatching-up: false\nstorage: bbg-fjall\nintents: {}\n---\nthe state of [[spacepussy-test]] at height {}\n",
            self.network.chain_id(), env!("CARGO_PKG_VERSION"),
            self.moniker.replace(['\n', '\r'], " "), self.height(),
            self.genesis_unix, self.root_hex(), self.signal_count(),
            self.neuron_count(), self.particle_count(), self.axon_count(),
            stats.node_count, stats.max_degree, self.network.role(),
            self.network.denom(), self.network.bech32_prefix(), self.network.rpc(),
            self.intent_count(), self.height(),
        )
    }
}

/// Import the retained legacy log into a fresh BBG directory.
/// The product caller holds the home lock for this entire operation.
pub fn import_legacy(home: &Path) -> io::Result<ImportReport> {
    let genesis = genesis::load(home, false)?;
    let log = genesis::read_bounded(&home.join("log"), genesis::MAX_LEGACY_LOG)?;
    NativeNode::import_legacy(&home.join("bbg"), &genesis.canonical, &log).map_err(io::Error::other)
}

/// Preserve the existing hex/label identity convention.
fn key32(s: &str) -> Particle {
    let candidate = s.strip_prefix("0x").unwrap_or(s);
    if !candidate.is_empty()
        && candidate.len() <= 64
        && candidate.bytes().all(|b| b.is_ascii_hexdigit())
    {
        let padded = format!("{candidate:0>64}");
        let mut out = [0u8; 32];
        for (i, byte) in out.iter_mut().enumerate() {
            // The ASCII hex check above establishes valid byte pairs.
            *byte = u8::from_str_radix(&padded[i * 2..i * 2 + 2], 16)
                .expect("validated hexadecimal pair");
        }
        out
    } else {
        hash32(s.as_bytes())
    }
}

fn hash32(bytes: &[u8]) -> Particle {
    let digest = hemera::hash(bytes);
    let mut out = [0; 32];
    out.copy_from_slice(&digest.as_bytes()[..32]);
    out
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn default_home() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".spacepussy-test")
}

/// Validate and recover durable state before binding the listener.
pub fn run(home: PathBuf, bind: &str, moniker: &str) -> io::Result<()> {
    let node = Node::open(home.clone(), moniker.to_owned())?;
    let listener = TcpListener::bind(bind)?;
    eprintln!(
        "soft3 node · {} · moniker={} · home={}",
        node.network.chain_id(),
        moniker,
        home.display()
    );
    eprintln!(
        "  listening {bind}; height {}; root {}",
        node.height(),
        node.root_hex()
    );
    let node = Arc::new(Mutex::new(node));
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let node = Arc::clone(&node);
                thread::spawn(move || {
                    if let Err(error) = http::handle_client(stream, &node) {
                        eprintln!("  request error: {error}");
                    }
                });
            }
            Err(error) => eprintln!("  accept error: {error}"),
        }
    }
    Ok(())
}
