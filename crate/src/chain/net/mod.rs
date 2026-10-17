//! The wire of the phase-1 root chain — specs/chain-wire.md.
//!
//! - [`codec`] — header/tx codecs and hashes the wire needs (vectors of chain-conformance.md)
//! - [`frames`] — the tade dialect `urn:cyberia:chain:1`
//! - [`chain_view`] — the operations the wire asks of the consensus kernel
//! - [`kernel`] — that port implemented over the kernel's `ChainApi`
//! - [`mempool`] — admission, nonce order, eviction (§4)
//! - [`peers`] — limits, scoring, eclipse basics (§5)
//! - [`transport`] — radio endpoint, bounded frame IO (§1)
//! - [`serve`] — answering request streams (§3)
//! - [`gossip`] — the three topics (§2)
//! - [`sync`] — header-first sync and IBD (§3.1–3.2)
//! - [`discovery`] — bootstrap file, anchors, the graph as address book (§6)
//!
//! [`Net`] starts them all on one endpoint.

pub mod chain_view;
pub mod codec;
pub mod discovery;
pub mod frames;
pub mod gossip;
pub mod kernel;
pub mod mempool;
pub mod peers;
pub mod serve;
pub mod sync;
pub mod transport;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use iroh_gossip::api::GossipSender;
use iroh_gossip::net::{Gossip, GOSSIP_ALPN};
use iroh_gossip::proto::TopicId;
use radio::endpoint::Connection;
use radio::protocol::Router;
use radio::{Endpoint, EndpointAddr, EndpointId, TransportAddr};
use tokio::sync::mpsc;

use chain_view::ChainView;
use codec::{hex, Header, Id};
use frames::{Checkpoint, Frame, MAX_GOSSIP_BYTES};
use mempool::Mempool;
use peers::{Direction, Peer, PeerTable};

/// A bootstrap entry (peers.toml v1 or the compiled set).
#[derive(Clone, Debug)]
pub struct Bootstrap {
    pub name: String,
    pub id: EndpointId,
    pub addrs: Vec<SocketAddr>,
}

impl Bootstrap {
    pub fn addr(&self) -> EndpointAddr {
        EndpointAddr::from_parts(self.id, self.addrs.iter().map(|a| TransportAddr::Ip(*a)))
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    /// `$home/<chain>`: radio.key, anchors.
    pub chain_dir: PathBuf,
    pub radio_bind: Option<SocketAddr>,
    pub relay: bool,
    pub bootstrap: Vec<Bootstrap>,
    /// This node's neuron (for `follow`-preferred dialing).
    pub neuron: Id,
}

/// What `/peers` shows.
#[derive(Clone, Debug)]
pub struct PeerView {
    pub id: EndpointId,
    pub direction: Direction,
    pub best_height: u64,
    pub best_tip: Id,
    pub score: u32,
    pub addrs: Vec<SocketAddr>,
}

pub(crate) struct Inner {
    pub endpoint: Endpoint,
    pub chain: Arc<dyn ChainView>,
    pub mempool: Arc<Mutex<Mempool>>,
    pub peers: Mutex<PeerTable>,
    pub conns: Mutex<HashMap<EndpointId, Connection>>,
    pub headers_tx: GossipSender,
    pub txs_tx: GossipSender,
    pub checkpoints_tx: GossipSender,
    pub sync_hint: mpsc::Sender<sync::Hint>,
    pub isolated: AtomicBool,
    pub catching_up: AtomicBool,
    pub bootstrap: Vec<Bootstrap>,
    pub checkpoints: Mutex<Vec<Checkpoint>>,
    pub book: Mutex<discovery::AddressBook>,
    pub dialing: Mutex<std::collections::HashSet<EndpointId>>,
    /// Headers and txs we relayed recently: their echoes are not duplicates to charge.
    pub recent: Mutex<HashMap<Id, Instant>>,
    pub chain_dir: PathBuf,
    pub neuron: Id,
    pub log: AtomicBool,
}

impl Inner {
    pub fn charge(&self, id: &EndpointId, points: u32, why: &str) {
        if points == 0 {
            return;
        }
        let verdict = self.peers.lock().expect("peers").charge(id, points);
        self.say(&format!("peer {} +{points} ({why}) → {verdict:?}", short(id)));
        if verdict == peers::Charge::Banned {
            self.disconnect(id, "banned");
        }
    }

    pub fn disconnect(&self, id: &EndpointId, why: &str) {
        if let Some(conn) = self.conns.lock().expect("conns").remove(id) {
            conn.close(1u32.into(), why.as_bytes());
        }
        self.peers.lock().expect("peers").remove(id);
    }

    pub fn conn(&self, id: &EndpointId) -> Option<Connection> {
        self.conns.lock().expect("conns").get(id).cloned()
    }

    pub fn say(&self, line: &str) {
        if self.log.load(Ordering::Relaxed) {
            eprintln!("  net · {line}");
        }
    }

    /// Mark a header/tx as just relayed, so its echo is not a scored duplicate.
    pub fn note_relayed(&self, id: Id) {
        let mut recent = self.recent.lock().expect("recent");
        let now = Instant::now();
        if recent.len() > 4096 {
            recent.retain(|_, t| now.duration_since(*t).as_secs() < 120);
        }
        recent.insert(id, now);
    }

    pub fn recently_relayed(&self, id: &Id) -> bool {
        self.recent.lock().expect("recent").get(id).is_some_and(|t| t.elapsed().as_secs() < 120)
    }

    /// Register a connection as a peer; `Err` means the connection must close.
    pub fn register(&self, conn: &Connection, direction: Direction, addrs: Vec<SocketAddr>, bootstrap: bool) -> Result<(), &'static str> {
        let id = conn.remote_id();
        if id == self.endpoint.id() {
            return Err("self");
        }
        if self.isolated.load(Ordering::Relaxed) {
            return Err("isolated");
        }
        let mut peers = self.peers.lock().expect("peers");
        let mut conns = self.conns.lock().expect("conns");
        if conns.contains_key(&id) {
            return Err("already connected");
        }
        // a gossip-only entry (scored, never dialed) is replaced; its score carries over
        let carried = peers.remove(&id).map(|p| p.score_now()).unwrap_or(0);
        let mut peer = Peer::new(id, direction, addrs, bootstrap);
        peer.score = carried;
        peers.admit(peer)?;
        conns.insert(id, conn.clone());
        Ok(())
    }

    pub async fn join_gossip(&self, id: EndpointId) {
        for sender in [&self.headers_tx, &self.txs_tx, &self.checkpoints_tx] {
            let _ = sender.join_peers(vec![id]).await;
        }
    }

    pub fn status_frame(&self) -> Frame {
        let tip = self.chain.tip();
        Frame::Status(frames::Status {
            chain: self.chain.chain_id(), rule_set: self.chain.rule_set(), height: tip.height, tip: tip.id,
            chainwork: tip.chainwork.to_le_bytes(), genesis: self.chain.genesis_id(),
        })
    }
}

pub fn short(id: &EndpointId) -> String {
    hex(&id.as_bytes()[..4])
}

pub fn now_unix() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// The running wire of one chain node.
pub struct Net {
    inner: Arc<Inner>,
    router: Router,
}

impl Net {
    pub async fn start(cfg: Config, chain: Arc<dyn ChainView>, mempool: Arc<Mutex<Mempool>>, log: bool) -> Result<Net, String> {
        let key = transport::load_or_create_key(&cfg.chain_dir.join("radio.key")).map_err(|e| e.to_string())?;
        let endpoint = transport::bind(key, cfg.radio_bind, cfg.relay, vec![transport::ALPN.to_vec(), GOSSIP_ALPN.to_vec()]).await?;
        let gossip = Gossip::builder().max_message_size(MAX_GOSSIP_BYTES).spawn(endpoint.clone());
        let chain_id = chain.chain_id();
        let mut topics = Vec::new();
        for name in ["headers", "txs", "checkpoints"] {
            let topic = gossip.subscribe(TopicId::from_bytes(codec::topic_id(&chain_id, name)), vec![]).await.map_err(|e| e.to_string())?;
            topics.push(topic.split());
        }
        let mut topics = topics.into_iter();
        let (headers_tx, headers_rx) = topics.next().expect("headers");
        let (txs_tx, txs_rx) = topics.next().expect("txs");
        let (checkpoints_tx, checkpoints_rx) = topics.next().expect("checkpoints");
        let (sync_hint, sync_rx) = mpsc::channel(256);
        let inner = Arc::new(Inner {
            endpoint: endpoint.clone(), chain, mempool, peers: Mutex::new(PeerTable::new()), conns: Mutex::new(HashMap::new()),
            headers_tx, txs_tx, checkpoints_tx, sync_hint, isolated: AtomicBool::new(false), catching_up: AtomicBool::new(false),
            bootstrap: cfg.bootstrap.clone(), checkpoints: Mutex::new(Vec::new()), book: Mutex::new(discovery::AddressBook::default()),
            dialing: Mutex::new(Default::default()), recent: Mutex::new(HashMap::new()), chain_dir: cfg.chain_dir.clone(), neuron: cfg.neuron,
            log: AtomicBool::new(log),
        });
        inner.peers.lock().expect("peers").anchors = discovery::load_anchors(&cfg.chain_dir);
        let router = Router::builder(endpoint).accept(transport::ALPN, serve::ChainProto(inner.clone())).accept(GOSSIP_ALPN, gossip.clone()).spawn();
        tokio::spawn(gossip::run_headers(inner.clone(), headers_rx));
        tokio::spawn(gossip::run_txs(inner.clone(), txs_rx));
        tokio::spawn(gossip::run_checkpoints(inner.clone(), checkpoints_rx));
        tokio::spawn(sync::run(inner.clone(), sync_rx));
        tokio::spawn(discovery::run(inner.clone()));
        tokio::spawn(events(inner.clone()));
        Ok(Net { inner, router })
    }

    pub fn id(&self) -> EndpointId {
        self.inner.endpoint.id()
    }

    pub fn bound(&self) -> Vec<SocketAddr> {
        self.inner.endpoint.bound_sockets()
    }

    /// Announce a block mined here: broadcast its header to the neighbors.
    pub async fn publish_header(&self, header: &Header) {
        self.inner.note_relayed(header.id());
        let _ = self.inner.headers_tx.broadcast_neighbors(bytes::Bytes::from(Frame::HeaderGossip(*header).encode())).await;
    }

    /// Hold a signed checkpoint and announce it on the `checkpoints` topic
    /// (chain.md §5.3: the signer publishes one per `CHECKPOINT_INTERVAL`).
    /// `false` when it is not new or its signature does not verify.
    pub async fn publish_checkpoint(&self, cp: Checkpoint) -> bool {
        let own = self.inner.endpoint.id();
        if !gossip::hold_checkpoint(&self.inner, &own, &cp) {
            return false;
        }
        let _ = self.inner.checkpoints_tx.broadcast_neighbors(bytes::Bytes::from(Frame::Checkpoint(cp).encode())).await;
        true
    }

    /// Announce a transaction admitted here.
    pub async fn publish_tx(&self, bytes: &[u8]) {
        self.inner.note_relayed(codec::txid(&self.inner.chain.chain_id(), bytes));
        let _ = self.inner.txs_tx.broadcast_neighbors(bytes::Bytes::from(Frame::Tx(bytes.to_vec()).encode())).await;
    }

    pub fn peers(&self) -> Vec<PeerView> {
        self.inner.peers.lock().expect("peers").iter()
            .map(|p| PeerView { id: p.id, direction: p.direction, best_height: p.best_height, best_tip: p.best_tip, score: p.score_now(), addrs: p.addrs.clone() })
            .collect()
    }

    pub fn outbound(&self) -> usize {
        self.inner.peers.lock().expect("peers").outbound()
    }

    pub fn catching_up(&self) -> bool {
        self.inner.catching_up.load(Ordering::Relaxed)
    }

    /// parano1d's sync-quorum gate: an outbound peer whose tip is within 2 blocks of ours.
    pub fn sync_quorum(&self, height: u64) -> bool {
        self.inner.peers.lock().expect("peers").synced_outbound(height, 2) >= 1
    }

    pub fn checkpoints(&self) -> Vec<Checkpoint> {
        self.inner.checkpoints.lock().expect("checkpoints").clone()
    }

    pub fn book_len(&self) -> usize {
        self.inner.book.lock().expect("book").len()
    }

    /// Dev switch: drop every peer and stop dialing (the fork rehearsal).
    pub fn set_isolated(&self, on: bool) {
        self.inner.isolated.store(on, Ordering::Relaxed);
        if on {
            let ids: Vec<EndpointId> = self.inner.conns.lock().expect("conns").keys().copied().collect();
            for id in ids {
                self.inner.disconnect(&id, "isolated");
            }
        }
    }

    pub fn is_isolated(&self) -> bool {
        self.inner.isolated.load(Ordering::Relaxed)
    }

    pub async fn dial(&self, addr: EndpointAddr) -> Result<(), String> {
        discovery::dial(self.inner.clone(), addr, false).await
    }

    pub async fn shutdown(&self) {
        let _ = self.router.shutdown().await;
    }
}

/// Keep the mempool in step with the tip and index the graph's address records.
async fn events(inner: Arc<Inner>) {
    let mut rx = inner.chain.events();
    loop {
        match rx.recv().await {
            Ok(chain_view::ChainEvent::NewTip { height, id, reorged_out }) => {
                if let Some((_, body)) = inner.chain.block(&id) {
                    let mut pool = inner.mempool.lock().expect("mempool");
                    pool.remove_mined(inner.chain.as_ref(), &body);
                    pool.revalidate(inner.chain.as_ref(), &reorged_out, now_unix());
                    inner.book.lock().expect("book").index_body(&body);
                }
                let best = inner.chain.best_header();
                inner.catching_up.store(best.height > height + 2, Ordering::Relaxed);
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            Err(_) => return,
        }
    }
}
