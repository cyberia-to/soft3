//! One chain node: the consensus kernel behind [`ChainView`]
//! ([`KernelChain`]), its mempool and the wire, served over the HTTP adapter
//! (chain_routes.rs). Mining is a separate process: `soft3 mine --node URL`
//! speaks `/template` and `/submit` (chain-wire §7).
//! `soft3 node --chain <name> --home DIR --bind HOST:PORT --radio HOST:UDP
//! --peers FILE --moniker NAME` (deploy/chain/soft3-chain@.service).

use std::io;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::chain::header::hash32;
use crate::chain::net::chain_view::{BlockVerdict, ChainView};
use crate::chain::net::codec::{hex, Header, Id};
use crate::chain::net::kernel::KernelChain;
use crate::chain::net::mempool::Mempool;
use crate::chain::net::{discovery, Config, Net};

/// Chains this binary launches; their genesis files are not embedded yet
/// (the pipeline of chain.md §4.2 is the kernel's), so `--genesis` is required.
pub const LAUNCH_CHAINS: &[&str] = &["pussy", "bostrom", "pussy-rc"];

#[derive(Debug, PartialEq, Eq)]
pub enum SubmitError {
    /// `prev` is no longer the tip, or the template was never handed out here
    Stale,
    /// pow, bits, time or the body (chain.md §5.5)
    Rejected(String),
}

pub struct ChainOpts {
    pub chain: String,
    pub home: PathBuf,
    pub genesis: Option<PathBuf>,
    pub radio: Option<SocketAddr>,
    pub peers: Option<PathBuf>,
    pub moniker: String,
    /// this node's neuron (follow-preferred dialing); default: hash of the radio key
    pub neuron: Option<Id>,
    pub relay: bool,
    pub log_net: bool,
    /// `--corpus DIR`: record every block given to the kernel (chain/corpus.rs)
    pub corpus: Option<PathBuf>,
}

pub struct ChainNode {
    pub chain: Arc<dyn ChainView>,
    pub kernel: Arc<KernelChain>,
    pub mempool: Arc<Mutex<Mempool>>,
    pub net: Arc<Net>,
    pub runtime: tokio::runtime::Runtime,
    pub moniker: String,
    pub chain_name: String,
    pub home: PathBuf,
    pub started: u64,
    /// unix second of the last `/template` handed out (an external miner works this node)
    pub last_template: std::sync::atomic::AtomicU64,
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
        let pin = crate::chain::params::embedded(&opts.chain).filter(|c| c.pinned());
        let genesis_path = opts.genesis.clone().unwrap_or_else(|| opts.home.join("genesis.tade"));
        let genesis_bytes = match std::fs::read(&genesis_path) {
            Ok(bytes) => bytes,
            Err(_) if LAUNCH_CHAINS.contains(&opts.chain.as_str()) => {
                return Err(io::Error::other(match pin {
                    Some(pin) => format!(
                        "the {} genesis file is still passed by hand (--genesis <file>, or {}); it must have particle {}",
                        opts.chain, opts.home.join("genesis.tade").display(), hex(&pin.chain_id)
                    ),
                    None => format!(
                        "the {} genesis is not embedded in this build yet; pass --genesis <file> (chain.md §4.2 pipeline)", opts.chain
                    ),
                }));
            }
            Err(e) => return Err(io::Error::other(format!("genesis {}: {e}", genesis_path.display()))),
        };
        // a pinned chain boots only the blessed file: checked before anything is written
        if let Some(pin) = pin {
            let particle = hash32(&genesis_bytes);
            if particle != pin.chain_id {
                return Err(io::Error::other(format!(
                    "--chain {}: genesis file {} has particle {} but this build pins {} (conformance/chain-genesis.toml)",
                    opts.chain, genesis_path.display(), hex(&particle), hex(&pin.chain_id)
                )));
            }
        }
        std::fs::create_dir_all(&opts.home)?;
        if opts.genesis.is_some() && !opts.home.join("genesis.tade").exists() {
            std::fs::write(opts.home.join("genesis.tade"), &genesis_bytes)?;
        }
        let kernel = Arc::new(KernelChain::open(&genesis_bytes, &opts.home).map_err(io::Error::other)?);
        if let Some(pin) = pin {
            let block0 = kernel.genesis_id();
            if block0 != pin.genesis_block {
                return Err(io::Error::other(format!(
                    "--chain {}: this build computes block 0 {} from the pinned genesis but pins {} (conformance/chain-genesis.toml); the state rules drifted from the blessed root",
                    opts.chain, hex(&block0), hex(&pin.genesis_block)
                )));
            }
        }
        if let Some(dir) = &opts.corpus {
            kernel.record_corpus(dir)?;
        }
        let name = kernel.chain_name();
        if name != opts.chain {
            return Err(io::Error::other(format!("--chain {} but the genesis file names {name:?}", opts.chain)));
        }
        let dev = !LAUNCH_CHAINS.contains(&name.as_str());
        let chain: Arc<dyn ChainView> = kernel.clone();
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
        let neuron = opts.neuron.unwrap_or_else(|| crate::chain::net::codec::hash(key.public().as_bytes()));
        let cfg = Config { chain_dir: opts.home.clone(), radio_bind: opts.radio, relay: opts.relay, bootstrap, neuron };
        let net = runtime.block_on(Net::start(cfg, chain.clone(), mempool.clone(), opts.log_net)).map_err(io::Error::other)?;
        let node = Arc::new(ChainNode {
            chain, kernel, mempool, net: Arc::new(net), runtime, moniker: opts.moniker, chain_name: name, home: opts.home,
            started: crate::chain::net::now_unix(), last_template: Default::default(), templates: Mutex::new(Vec::new()), dev,
        });
        Ok(node)
    }

    /// `POST /submit`: a solved header for a template handed out at this
    /// tip; the kernel validates it with the template's body (§5.5 1–8).
    pub fn submit(&self, header: Header) -> Result<Id, SubmitError> {
        let mut zero = header;
        zero.nonce = [0; 16];
        let key = zero.id();
        let body = {
            let templates = self.templates.lock().expect("templates");
            templates.iter().find(|(k, _, _)| *k == key).map(|(_, _, b)| b.clone())
        };
        let Some(body) = body else { return Err(SubmitError::Stale) };
        if header.prev != self.chain.tip().id {
            return Err(SubmitError::Stale);
        }
        match self.chain.accept_block(&header, &body, crate::chain::net::now_unix()) {
            Ok(BlockVerdict::NewTip) => {
                self.runtime.block_on(self.net.publish_header(&header));
                Ok(header.id())
            }
            Ok(BlockVerdict::Known) => Ok(header.id()),
            Ok(other) => Err(SubmitError::Rejected(format!("not the new tip: {other:?}"))),
            Err(e) => Err(SubmitError::Rejected(e.to_string())),
        }
    }

    /// A miner asked for a template within the last minute.
    pub fn mining(&self) -> bool {
        let last = self.last_template.load(std::sync::atomic::Ordering::Relaxed);
        last > 0 && crate::chain::net::now_unix().saturating_sub(last) < 60
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
        let next_bits = self.kernel.next_bits();
        // chain-economy §11: the economy's /status fields
        let econ = self.kernel.economy(|c| Some(c.economy_status())).unwrap_or_default();
        let field = |k: &str| econ.get(k).map(|v| v.to_string()).unwrap_or_else(|| "none".into());
        let economy = format!(
            "epoch: {}\nnext-boundary: {}\nlast-settled: {}\nfocus-bps: {}\npool: {}\nactive-stake: {}\ntotal-stake: {}\nstake-keys-match: {}\nsettlement-ok: {}\nfocus-engine: {}\n",
            field("epoch"), field("next_boundary"), field("last_settled"), field("focus_bps"), field("pool"), field("active_stake"),
            field("total_stake"), field("stake_keys_match"), field("settlement_ok"), field("engine").trim_matches('"'),
        );
        let status = format!(
            "---\nparticle: status\nchain: {}\nprotocol: soft3/chain/1\nrule-set: {}\nactivations: 1@0\nengine: chain+cybergraph+bbg\nversion: {}\nmoniker: {}\nheight: {}\ntip: {}\nchainwork: {}\nbits: {:#010x}\nnext-bits: {:#010x}\ngenesis: {}\ngenesis-block: {}\ngenesis-time: {}\nbbg-root: {}\nsupply: {}\nconserved: {}\nsubsidy: {}\nneurons: {}\nparticles: {}\naxons: {}\nmempool: {}\npeers: {}\noutbound: {}\nbest-header: {}\ncheckpoint: {}\nfinalized: {}\ncatching-up: {}\nmining: {}\nstorage: bbg-fjall\nendpoint: {}\n---\nthe state of [[{}]] at height {}\n",
            self.chain_name, self.chain.rule_set(), env!("CARGO_PKG_VERSION"), self.moniker.replace(['\n', '\r'], " "),
            tip.height, hex(&tip.id), tip.chainwork.to_decimal(), bits, next_bits, hex(&self.chain.chain_id()), hex(&self.chain.genesis_id()), self.chain.genesis_time(),
            hex(&self.chain.state_root()), self.chain.supply(&native), self.chain.conserved(), self.chain.subsidy(), neurons, particles, axons,
            pool.len(), peers.len(), peers.iter().filter(|p| p.direction == crate::chain::net::peers::Direction::Outbound).count(),
            best.height, checkpoint, self.kernel.finalized_height(), self.net.catching_up(), self.mining(), hex(self.net.id().as_bytes()),
            self.chain_name, tip.height,
        );
        // the economy's fields go before the closing front-matter fence
        match status.split_once("storage: bbg-fjall\n") {
            Some((head, tail)) => format!("{head}{economy}storage: bbg-fjall\n{tail}"),
            None => status,
        }
    }
}
