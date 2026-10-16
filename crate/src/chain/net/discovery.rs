//! Discovery — chain-wire.md §6: the bootstrap file (deploy/chain
//! peers.toml, format v1), the anchors of the last run, and the graph as
//! address book: kind-2 transactions `antenna → endpoint id` and
//! `socket → packed ip:port` exactly as cyb's `wire.rs` packs them. The
//! dialer keeps 8 outbound peers, diverse by /16, with a bootstrap peer
//! until four others are up.

use std::collections::{HashMap, HashSet};
use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use radio::{EndpointAddr, EndpointId, TransportAddr};
use serde::Deserialize;

use super::codec::{hash, hex, unhex32, Id, Tx};
use super::peers::Direction;
use super::{short, Bootstrap, Inner};

pub const DIAL_INTERVAL: Duration = Duration::from_secs(3);

#[derive(Deserialize)]
struct PeersFile {
    version: u32,
    chain: String,
    #[serde(default)]
    peer: Vec<PeerEntry>,
}

#[derive(Deserialize)]
struct PeerEntry {
    #[serde(default)]
    name: String,
    id: String,
    #[serde(default)]
    addrs: Vec<String>,
    #[serde(default)]
    relay: String,
}

/// Parse peers.toml v1; entries with an empty id are skipped (returned as the
/// second value) — QUIC needs the endpoint id to dial.
pub fn parse_peers(text: &str, chain: &str) -> Result<(Vec<Bootstrap>, Vec<String>), String> {
    let file: PeersFile = toml::from_str(text).map_err(|e| format!("peers file: {e}"))?;
    if file.version != 1 {
        return Err(format!("peers file version {} is not 1", file.version));
    }
    if file.chain != chain {
        return Err(format!("peers file is for chain {:?}, this node runs {chain:?}", file.chain));
    }
    let mut out = Vec::new();
    let mut skipped = Vec::new();
    for p in file.peer {
        let _ = &p.relay;
        if p.id.is_empty() {
            skipped.push(p.name);
            continue;
        }
        let bytes = unhex32(&p.id).ok_or_else(|| format!("peer {:?}: id is not 64 hex chars", p.name))?;
        let id = EndpointId::from_bytes(&bytes).map_err(|e| format!("peer {:?}: {e}", p.name))?;
        let addrs: Vec<SocketAddr> = p.addrs.iter().filter_map(|a| a.parse().ok()).collect();
        if addrs.is_empty() {
            return Err(format!("peer {:?}: no parsable address", p.name));
        }
        out.push(Bootstrap { name: p.name, id, addrs });
    }
    Ok((out, skipped))
}

// ── the graph as address book ───────────────────────────────────────────────

pub fn wkp(word: &str) -> Id {
    hash(word.as_bytes())
}

/// `'4' ‖ ipv4[4] ‖ port BE[2] ‖ zeros` — cyb wire.rs.
pub fn pack_sock(s: &SocketAddr) -> Option<Id> {
    let std::net::IpAddr::V4(ip) = s.ip() else { return None };
    let mut p = [0u8; 32];
    p[0] = b'4';
    p[1..5].copy_from_slice(&ip.octets());
    p[5..7].copy_from_slice(&s.port().to_be_bytes());
    Some(p)
}

pub fn unpack_sock(p: &Id) -> Option<SocketAddr> {
    if p[0] != b'4' || p[7..].iter().any(|b| *b != 0) {
        return None;
    }
    Some(SocketAddr::from((Ipv4Addr::new(p[1], p[2], p[3], p[4]), u16::from_be_bytes([p[5], p[6]]))))
}

#[derive(Default)]
pub struct AddressBook {
    antenna: HashMap<Id, EndpointId>,
    sockets: HashMap<Id, Vec<SocketAddr>>,
    follows: HashMap<Id, HashSet<Id>>,
}

impl AddressBook {
    pub fn len(&self) -> usize {
        self.antenna.len()
    }

    pub fn is_empty(&self) -> bool {
        self.antenna.is_empty()
    }

    /// Index the kind-2 transactions of an applied body; newest wins.
    pub fn index_body(&mut self, body: &[u8]) {
        let antenna = wkp("antenna");
        let socket = wkp("socket");
        let follow = wkp("follow");
        for bytes in super::codec::decode_body(body).unwrap_or_default() {
            if let Ok(Tx::Cyberlink { from, p, q, .. }) = Tx::decode(&bytes) {
                if p == antenna {
                    if let Ok(id) = EndpointId::from_bytes(&q) {
                        self.antenna.insert(from, id);
                        self.sockets.remove(&from);
                    }
                } else if p == socket {
                    if let Some(addr) = unpack_sock(&q) {
                        let list = self.sockets.entry(from).or_default();
                        list.retain(|a| *a != addr);
                        list.push(addr);
                        list.truncate(8);
                    }
                } else if p == follow {
                    self.follows.entry(from).or_default().insert(q);
                }
            }
        }
    }

    pub fn lookup(&self, neuron: &Id) -> Option<EndpointAddr> {
        let id = *self.antenna.get(neuron)?;
        let addrs = self.sockets.get(neuron)?;
        Some(EndpointAddr::from_parts(id, addrs.iter().map(|a| TransportAddr::Ip(*a))))
    }

    /// Dial candidates: neurons `me` follows first, then everyone else in a seeded order.
    pub fn candidates(&self, me: &Id, seed: u64) -> Vec<EndpointAddr> {
        let followed = self.follows.get(me).cloned().unwrap_or_default();
        let mut first: Vec<EndpointAddr> = followed.iter().filter_map(|n| self.lookup(n)).collect();
        let mut rest: Vec<(u64, EndpointAddr)> = self.antenna.keys().filter(|n| !followed.contains(*n))
            .filter_map(|n| self.lookup(n).map(|a| (xorshift(seed ^ u64::from_le_bytes(n[..8].try_into().expect("8"))), a))).collect();
        rest.sort_by_key(|(k, _)| *k);
        first.extend(rest.into_iter().map(|(_, a)| a));
        first
    }
}

fn xorshift(mut x: u64) -> u64 {
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

/// The two address transactions of this node (chain-wire §6), unsigned.
pub fn address_links(neuron: Id, nonce: u64, token: Id, endpoint: EndpointId, sockets: &[SocketAddr]) -> Vec<Tx> {
    let mut out = vec![Tx::Cyberlink { from: neuron, nonce, token, amount: 0, p: wkp("antenna"), q: *endpoint.as_bytes(), valence: 0, fee: 257, sig: vec![] }];
    let mut n = nonce + 1;
    for s in sockets.iter().filter_map(pack_sock).take(4) {
        out.push(Tx::Cyberlink { from: neuron, nonce: n, token, amount: 0, p: wkp("socket"), q: s, valence: 0, fee: 257, sig: vec![] });
        n += 1;
    }
    out
}

// ── anchors ─────────────────────────────────────────────────────────────────

pub fn load_anchors(dir: &Path) -> Vec<EndpointId> {
    std::fs::read_to_string(dir.join("anchors")).unwrap_or_default().lines()
        .filter_map(|l| unhex32(l.split_whitespace().next()?)).filter_map(|b| EndpointId::from_bytes(&b).ok()).take(2).collect()
}

pub fn save_anchors(dir: &Path, anchors: &[(EndpointId, Vec<SocketAddr>)]) {
    let text: String = anchors.iter().map(|(id, addrs)| {
        let a: Vec<String> = addrs.iter().map(|s| s.to_string()).collect();
        format!("{} {}\n", hex(id.as_bytes()), a.join(","))
    }).collect();
    let _ = std::fs::write(dir.join("anchors"), text);
}

fn load_anchor_addrs(dir: &Path) -> Vec<(EndpointId, Vec<SocketAddr>)> {
    std::fs::read_to_string(dir.join("anchors")).unwrap_or_default().lines().filter_map(|l| {
        let mut it = l.split_whitespace();
        let id = EndpointId::from_bytes(&unhex32(it.next()?)?).ok()?;
        let addrs = it.next().unwrap_or("").split(',').filter_map(|a| a.parse().ok()).collect();
        Some((id, addrs))
    }).collect()
}

// ── the dialer ──────────────────────────────────────────────────────────────

/// Connect, register as outbound, exchange status, serve the connection.
pub(crate) async fn dial(inner: Arc<Inner>, addr: EndpointAddr, bootstrap: bool) -> Result<(), String> {
    let id = addr.id;
    if id == inner.endpoint.id() {
        return Err("self".into());
    }
    if !inner.dialing.lock().expect("dialing").insert(id) {
        return Err("dial in flight".into());
    }
    let result = dial_inner(&inner, addr, bootstrap).await;
    inner.dialing.lock().expect("dialing").remove(&id);
    result
}

async fn dial_inner(inner: &Arc<Inner>, addr: EndpointAddr, bootstrap: bool) -> Result<(), String> {
    let id = addr.id;
    if inner.conns.lock().expect("conns").contains_key(&id) {
        return Err("already connected".into());
    }
    if inner.peers.lock().expect("peers").is_banned(&id) {
        return Err("banned".into());
    }
    let sockets: Vec<SocketAddr> = addr.addrs.iter().filter_map(|a| match a { TransportAddr::Ip(s) => Some(*s), _ => None }).collect();
    let conn = tokio::time::timeout(Duration::from_secs(10), inner.endpoint.connect(addr, super::transport::ALPN)).await
        .map_err(|_| "connect timeout".to_string())?.map_err(|e| e.to_string())?;
    inner.register(&conn, Direction::Outbound, sockets, bootstrap)?;
    inner.say(&format!("dialed {}{}", short(&id), if bootstrap { " (bootstrap)" } else { "" }));
    inner.join_gossip(id).await;
    let serve = { let (inner, conn) = (inner.clone(), conn.clone()); tokio::spawn(super::serve::serve_connection(inner, conn)) };
    if let Err(e) = super::sync::header_sync(inner, id, &conn).await {
        inner.say(&format!("status with {}: {e}", short(&id)));
        inner.charge(&id, e.score(), "status");
    }
    let _ = inner.sync_hint.try_send(super::sync::Hint::Peer(id));
    let _ = serve;
    Ok(())
}

pub(crate) async fn run(inner: Arc<Inner>) {
    let mut seed = u64::from_le_bytes(inner.endpoint.id().as_bytes()[..8].try_into().expect("8"));
    let anchors = load_anchor_addrs(&inner.chain_dir);
    loop {
        if !inner.isolated.load(Ordering::Relaxed) {
            seed = xorshift(seed);
            let known: HashSet<EndpointId> = inner.conns.lock().expect("conns").keys().copied().collect();
            let (outbound, need_boot, anchor_ids): (usize, bool, Vec<EndpointId>) = {
                let peers = inner.peers.lock().expect("peers");
                (peers.outbound(), peers.needs_bootstrap(), peers.anchors.clone())
            };
            if outbound < super::peers::OUTBOUND_TARGET {
                let mut candidates: Vec<(EndpointAddr, bool)> = Vec::new();
                for (id, addrs) in &anchors {
                    candidates.push((EndpointAddr::from_parts(*id, addrs.iter().map(|a| TransportAddr::Ip(*a))), inner.bootstrap.iter().any(|b| b.id == *id)));
                }
                let mut boots: Vec<(EndpointAddr, bool)> = inner.bootstrap.iter().map(|b| (b.addr(), true)).collect();
                if need_boot {
                    candidates.append(&mut boots);
                } else {
                    let book = inner.book.lock().expect("book").candidates(&inner.neuron, seed);
                    candidates.extend(book.into_iter().map(|a| (a, false)));
                    candidates.append(&mut boots);
                }
                let me = inner.endpoint.id();
                let mut seen = HashSet::new();
                for (addr, bootstrap) in candidates {
                    if addr.id == me || known.contains(&addr.id) || !seen.insert(addr.id) {
                        continue;
                    }
                    let inner = inner.clone();
                    tokio::spawn(async move {
                        if let Err(e) = dial(inner.clone(), addr.clone(), bootstrap).await {
                            inner.say(&format!("dial {} failed: {e}", short(&addr.id)));
                        }
                    });
                    if seen.len() >= 4 {
                        break;
                    }
                }
            }
            // persist anchors with their addresses
            let anchors_now: Vec<(EndpointId, Vec<SocketAddr>)> = {
                let peers = inner.peers.lock().expect("peers");
                anchor_ids.iter().filter_map(|id| peers.get(id).map(|p| (*id, p.addrs.clone()))).collect()
            };
            if !anchors_now.is_empty() {
                save_anchors(&inner.chain_dir, &anchors_now);
            }
        }
        tokio::time::sleep(DIAL_INTERVAL).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peers_file_v1() {
        let id = hex(radio::SecretKey::from_bytes(&[3; 32]).public().as_bytes());
        let text = format!("version = 1\nchain = \"dev\"\n\n[[peer]]\nname = \"a\"\nid = \"{id}\"\naddrs = [\"127.0.0.1:7791\"]\nrelay = \"\"\n\n[[peer]]\nname = \"b\"\nid = \"\"\naddrs = [\"127.0.0.1:7792\"]\n");
        let (peers, skipped) = parse_peers(&text, "dev").unwrap();
        assert_eq!(peers.len(), 1);
        assert_eq!(skipped, vec!["b"]);
        assert!(parse_peers(&text, "pussy").is_err());
        assert!(parse_peers("version = 2\nchain = \"dev\"\n", "dev").is_err());
        assert!(parse_peers("nonsense", "dev").is_err());
    }

    #[test]
    fn socket_packing_matches_wire_rs() {
        let s: SocketAddr = "10.1.2.3:7791".parse().unwrap();
        let p = pack_sock(&s).unwrap();
        assert_eq!(&p[..7], &[b'4', 10, 1, 2, 3, 0x1e, 0x6f]);
        assert_eq!(unpack_sock(&p), Some(s));
        assert_eq!(unpack_sock(&[0u8; 32]), None);
    }

    #[test]
    fn book_indexes_address_links() {
        let id = radio::SecretKey::from_bytes(&[5; 32]).public();
        let s: SocketAddr = "127.0.0.1:7791".parse().unwrap();
        let txs: Vec<Vec<u8>> = std::iter::once(Tx::Coinbase { height: 1, to: [0; 32], amount: 1, extra: [0; 32] }.encode())
            .chain(address_links([1; 32], 0, [2; 32], id, &[s]).iter().map(|t| { let mut t = t.clone(); if let Tx::Cyberlink { sig, .. } = &mut t { *sig = vec![0; 102] }; t.encode() }))
            .collect();
        let body = super::super::codec::encode_body(&txs);
        let mut book = AddressBook::default();
        book.index_body(&body);
        let addr = book.lookup(&[1; 32]).unwrap();
        assert_eq!(addr.id, id);
        assert_eq!(book.candidates(&[9; 32], 1).len(), 1);
    }
}
