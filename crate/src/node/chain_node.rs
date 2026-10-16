//! One chain node: the chain behind [`ChainView`], its mempool, the wire and
//! an optional CPU miner, served over the HTTP adapter (chain_routes.rs).
//! `soft3 node --chain <name> --home DIR --bind HOST:PORT --radio HOST:UDP
//! --peers FILE --moniker NAME` (deploy/chain/soft3-chain@.service).

use std::io;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::chain::net::chain_view::ChainView;
use crate::chain::net::codec::{hex, Header, Id};
use crate::chain::net::devchain::{self, DevChain};
use crate::chain::net::mempool::Mempool;
use crate::chain::net::{discovery, Config, Net};

/// Chains this binary launches; their genesis files are not embedded yet
/// (the pipeline of chain.md §4.2 is the kernel's), so `--genesis` is required.
pub const LAUNCH_CHAINS: &[&str] = &["pussy", "bostrom", "pussy-rc"];

pub struct ChainOpts {
    pub chain: String,
    pub home: PathBuf,
    pub genesis: Option<PathBuf>,
    pub radio: Option<SocketAddr>,
    pub peers: Option<PathBuf>,
    pub moniker: String,
    pub mine: bool,
    pub solo: bool,
    pub miner: Option<Id>,
    pub mine_interval: Duration,
    pub relay: bool,
    pub log_net: bool,
}

pub struct ChainNode {
    pub chain: Arc<dyn ChainView>,
    pub mempool: Arc<Mutex<Mempool>>,
    pub net: Arc<Net>,
    pub runtime: tokio::runtime::Runtime,
    pub moniker: String,
    pub chain_name: String,
    pub home: PathBuf,
    pub mining: AtomicBool,
    pub solo: AtomicBool,
    pub miner: Id,
    pub mine_interval: Duration,
    pub mined: AtomicU64,
    pub started: u64,
    /// Templates handed out by `/template`, keyed by their zero-nonce id, valid for one tip.
    pub templates: Mutex<Vec<(Id, Header, Vec<u8>)>>,
    pub dev: bool,
}

/// Default home: `~/.soft3/<chain>`.
pub fn default_home(chain: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".soft3").join(chain)
}

/// Load (or on first start, create and print) the node key and return its endpoint id.
pub fn endpoint_id(home: &Path) -> io::Result<String> {
    let key = crate::chain::net::transport::load_or_create_key(&home.join("radio.key"))?;
    Ok(hex(key.public().as_bytes()))
}

impl ChainNode {
    pub fn open(opts: ChainOpts) -> io::Result<Arc<ChainNode>> {
        std::fs::create_dir_all(&opts.home)?;
        let genesis_path = opts.genesis.clone().unwrap_or_else(|| opts.home.join("genesis.tade"));
        let genesis_bytes = match std::fs::read(&genesis_path) {
            Ok(bytes) => bytes,
            Err(_) if LAUNCH_CHAINS.contains(&opts.chain.as_str()) => {
                return Err(io::Error::other(format!(
                    "the {} genesis is not embedded in this build yet; pass --genesis <file> (chain.md §4.2 pipeline)", opts.chain
                )));
            }
            Err(e) => return Err(io::Error::other(format!("genesis {}: {e}", genesis_path.display()))),
        };
        if opts.genesis.is_some() && !opts.home.join("genesis.tade").exists() {
            std::fs::write(opts.home.join("genesis.tade"), &genesis_bytes)?;
        }
        let chain = DevChain::open(&genesis_bytes, Some(&opts.home)).map_err(io::Error::other)?;
        let name = chain.genesis().name;
        if name != opts.chain {
            return Err(io::Error::other(format!("--chain {} but the genesis file names {name:?}", opts.chain)));
        }
        let dev = !LAUNCH_CHAINS.contains(&name.as_str());
        let chain: Arc<dyn ChainView> = Arc::new(chain);
        let mempool = Arc::new(Mutex::new(Mempool::new()));
        let bootstrap = match &opts.peers {
            Some(path) => {
                let text = std::fs::read_to_string(path)?;
                let (peers, skipped) = discovery::parse_peers(&text, &name).map_err(io::Error::other)?;
                for s in skipped {
                    eprintln!("  peers: {s:?} has no endpoint id — skipped (QUIC dials need the id; `soft3 node id`)");
                }
                peers
            }
            None => Vec::new(),
        };
        let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().thread_name("chain-net").build()?;
        let key = crate::chain::net::transport::load_or_create_key(&opts.home.join("radio.key"))?;
        let miner = opts.miner.unwrap_or_else(|| crate::chain::net::codec::hash(key.public().as_bytes()));
        let cfg = Config { chain_dir: opts.home.clone(), radio_bind: opts.radio, relay: opts.relay, bootstrap, neuron: miner };
        let net = runtime.block_on(Net::start(cfg, chain.clone(), mempool.clone(), opts.log_net)).map_err(io::Error::other)?;
        let node = Arc::new(ChainNode {
            chain, mempool, net: Arc::new(net), runtime, moniker: opts.moniker, chain_name: name, home: opts.home,
            mining: AtomicBool::new(opts.mine), solo: AtomicBool::new(opts.solo), miner, mine_interval: opts.mine_interval,
            mined: AtomicU64::new(0), started: crate::chain::net::now_unix(), templates: Mutex::new(Vec::new()), dev,
        });
        let miner_node = node.clone();
        std::thread::Builder::new().name("chain-miner".into()).spawn(move || miner_node.mine_loop())?;
        Ok(node)
    }

    fn mine_loop(self: Arc<Self>) {
        loop {
            if !self.mining.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(200));
                continue;
            }
            let tip = self.chain.tip();
            if !self.solo.load(Ordering::Relaxed) && !self.net.sync_quorum(tip.height) {
                std::thread::sleep(Duration::from_secs(1));
                continue;
            }
            let txs = self.mempool.lock().expect("mempool").template_txs();
            let now = crate::chain::net::now_unix();
            let Ok((header, body)) = self.chain.template(&self.miner, &txs, now) else {
                std::thread::sleep(Duration::from_millis(200));
                continue;
            };
            let mut candidate = header;
            let mut solved = None;
            let mut base = 0u64;
            while solved.is_none() {
                let mut h = candidate;
                h.nonce = (base as u128).to_le_bytes();
                solved = devchain::mine_from(h, base, 4096);
                base += 4096;
                if self.chain.tip().id != tip.id || !self.mining.load(Ordering::Relaxed) {
                    break;
                }
                candidate = header;
            }
            let Some(solved) = solved else { continue };
            if self.chain.accept_header(&solved, now + 1).is_ok() && matches!(self.chain.accept_block(&solved, &body), Ok(_)) {
                self.mined.fetch_add(1, Ordering::Relaxed);
                self.runtime.block_on(self.net.publish_header(&solved));
            }
            std::thread::sleep(self.mine_interval);
        }
    }

    /// `POST /submit`: a solved header for a template handed out at this tip.
    pub fn submit(&self, header: Header) -> Result<Id, &'static str> {
        let mut zero = header;
        zero.nonce = [0; 16];
        let key = zero.id();
        let body = {
            let templates = self.templates.lock().expect("templates");
            templates.iter().find(|(k, _, _)| *k == key).map(|(_, _, b)| b.clone())
        };
        let Some(body) = body else { return Err("stale template") };
        if header.prev != self.chain.tip().id {
            return Err("stale template");
        }
        let now = crate::chain::net::now_unix();
        self.chain.accept_header(&header, now).map_err(|_| "header rejected")?;
        self.chain.accept_block(&header, &body).map_err(|_| "block rejected")?;
        self.runtime.block_on(self.net.publish_header(&header));
        Ok(header.id())
    }

    pub fn status_cybermark(&self) -> String {
        let tip = self.chain.tip();
        let best = self.chain.best_header();
        let native = self.chain.native_token();
        let (neurons, particles, axons) = self.chain.counts();
        let pool = self.mempool.lock().expect("mempool");
        let peers = self.net.peers();
        let checkpoint = self.net.checkpoints().last().map(|c| c.height.to_string()).unwrap_or_else(|| "none".into());
        let bits = self.chain.header_at(tip.height).map(|h| h.bits).unwrap_or(0);
        format!(
            "---\nparticle: status\nchain: {}\nprotocol: soft3/chain/1\nrule-set: {}\nactivations: 1@0\nengine: chain+cybergraph+bbg\nversion: {}\nmoniker: {}\nheight: {}\ntip: {}\nchainwork: {}\nbits: {:#010x}\nnext-bits: {:#010x}\ngenesis: {}\ngenesis-time: {}\nbbg-root: {}\nsupply: {}\nconserved: {}\nsubsidy: {}\nneurons: {}\nparticles: {}\naxons: {}\nmempool: {}\npeers: {}\noutbound: {}\nbest-header: {}\ncheckpoint: {}\ncatching-up: {}\nmining: {}\nstorage: bbg-fjall\nendpoint: {}\n---\nthe state of [[{}]] at height {}\n",
            self.chain_name, self.chain.rule_set(), env!("CARGO_PKG_VERSION"), self.moniker.replace(['\n', '\r'], " "),
            tip.height, hex(&tip.id), tip.chainwork.to_decimal(), bits, bits, hex(&self.chain.genesis_id()), self.chain.genesis_time(),
            hex(&self.chain.state_root()), self.chain.supply(&native), self.chain.conserved(), self.chain.subsidy(), neurons, particles, axons,
            pool.len(), peers.len(), peers.iter().filter(|p| p.direction == crate::chain::net::peers::Direction::Outbound).count(),
            best.height, checkpoint, self.net.catching_up(), self.mining.load(Ordering::Relaxed), hex(self.net.id().as_bytes()),
            self.chain_name, tip.height,
        )
    }
}
