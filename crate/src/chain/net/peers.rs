//! Per-peer limits, scoring, bans and the eclipse basics — chain-wire.md §5.
//! Token buckets with burst = 4× rate; a score that decays 1/min and bans
//! at 100 for an hour; the outbound budget with /16 diversity and the
//! bootstrap floor; anchors remembered for the next start.

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

use radio::EndpointId;

use super::codec::Id;

pub const OUTBOUND_TARGET: usize = 8;
pub const INBOUND_MAX: usize = 64;
pub const SAME_NET_MAX: usize = 2;
pub const BAN_SCORE: u32 = 100;
pub const BAN_SECS: u64 = 3600;
pub const STREAMS_PER_DIRECTION: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Outbound,
    Inbound,
}

/// A token bucket: `rate` per second, burst `4 × rate`.
#[derive(Clone, Debug)]
pub struct Bucket {
    rate: f64,
    tokens: f64,
    last: Instant,
}

impl Bucket {
    /// Starts with one second of tokens; accrues up to the 4× burst while idle.
    pub fn new(rate: u64) -> Self {
        Bucket { rate: rate as f64, tokens: rate as f64, last: Instant::now() }
    }

    pub fn take(&mut self, n: u64) -> bool {
        self.take_at(n, Instant::now())
    }

    pub fn take_at(&mut self, n: u64, now: Instant) -> bool {
        let elapsed = now.saturating_duration_since(self.last).as_secs_f64();
        self.last = now;
        self.tokens = (self.tokens + elapsed * self.rate).min(4.0 * self.rate);
        if self.tokens >= n as f64 {
            self.tokens -= n as f64;
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
pub struct Peer {
    pub id: EndpointId,
    pub direction: Direction,
    pub addrs: Vec<SocketAddr>,
    pub bootstrap: bool,
    pub score: u32,
    scored_at: Instant,
    pub best_height: u64,
    pub best_tip: Id,
    pub chainwork: [u8; 32],
    pub served_blocks: u64,
    pub headers: Bucket,
    pub txs: Bucket,
    pub bytes: Bucket,
    pub streams: Bucket,
    pub open_streams: usize,
    pub connected_at: Instant,
}

impl Peer {
    pub fn new(id: EndpointId, direction: Direction, addrs: Vec<SocketAddr>, bootstrap: bool) -> Self {
        let now = Instant::now();
        Peer {
            id, direction, addrs, bootstrap, score: 0, scored_at: now, best_height: 0, best_tip: [0; 32], chainwork: [0; 32],
            served_blocks: 0, headers: Bucket::new(64), txs: Bucket::new(256), bytes: Bucket::new(4 * 1024 * 1024),
            streams: Bucket::new(16), open_streams: 0, connected_at: now,
        }
    }

    /// Score after decay (1 per minute).
    pub fn score_now(&self) -> u32 {
        let minutes = self.scored_at.elapsed().as_secs() / 60;
        self.score.saturating_sub(minutes as u32)
    }

    fn charge(&mut self, points: u32) -> u32 {
        self.score = self.score_now().saturating_add(points);
        self.scored_at = Instant::now();
        self.score
    }
}

/// `/16` for IPv4, `/32` for IPv6 — the diversity key.
pub fn net_group(ip: &IpAddr) -> Vec<u8> {
    match ip {
        IpAddr::V4(v4) => v4.octets()[..2].to_vec(),
        IpAddr::V6(v6) => v6.octets()[..4].to_vec(),
    }
}

#[derive(Default)]
pub struct PeerTable {
    peers: HashMap<EndpointId, Peer>,
    bans: HashMap<EndpointId, Instant>,
    pub anchors: Vec<EndpointId>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Charge {
    Scored(u32),
    Banned,
}

impl PeerTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.peers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }

    pub fn get(&self, id: &EndpointId) -> Option<&Peer> {
        self.peers.get(id)
    }

    pub fn get_mut(&mut self, id: &EndpointId) -> Option<&mut Peer> {
        self.peers.get_mut(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Peer> {
        self.peers.values()
    }

    pub fn contains(&self, id: &EndpointId) -> bool {
        self.peers.contains_key(id)
    }

    pub fn outbound(&self) -> usize {
        self.peers.values().filter(|p| p.direction == Direction::Outbound).count()
    }

    pub fn inbound(&self) -> usize {
        self.peers.values().filter(|p| p.direction == Direction::Inbound).count()
    }

    pub fn is_banned(&mut self, id: &EndpointId) -> bool {
        match self.bans.get(id) {
            Some(until) if *until > Instant::now() => true,
            Some(_) => {
                self.bans.remove(id);
                false
            }
            None => false,
        }
    }

    /// Admit a connection, or say why not.
    pub fn admit(&mut self, peer: Peer) -> Result<(), &'static str> {
        if self.is_banned(&peer.id) {
            return Err("banned");
        }
        if self.peers.contains_key(&peer.id) {
            return Err("already listed");
        }
        match peer.direction {
            Direction::Inbound if self.inbound() >= INBOUND_MAX => return Err("inbound full"),
            Direction::Outbound if self.outbound() >= OUTBOUND_TARGET => return Err("outbound full"),
            Direction::Outbound if !self.diverse(&peer.addrs) => return Err("same /16 as two peers"),
            _ => {}
        }
        self.peers.insert(peer.id, peer);
        Ok(())
    }

    /// At most `SAME_NET_MAX` outbound peers per /16 (v4) or /32 (v6);
    /// loopback peers (a fleet on one machine) are not counted.
    pub fn diverse(&self, addrs: &[SocketAddr]) -> bool {
        addrs.iter().filter(|a| !a.ip().is_loopback()).all(|a| {
            let group = net_group(&a.ip());
            self.peers.values().filter(|p| p.direction == Direction::Outbound)
                .filter(|p| p.addrs.iter().any(|pa| net_group(&pa.ip()) == group)).count() < SAME_NET_MAX
        })
    }

    /// chain-wire §5: a bootstrap peer is required until 4 other outbound peers are up.
    pub fn needs_bootstrap(&self) -> bool {
        let others = self.peers.values().filter(|p| p.direction == Direction::Outbound && !p.bootstrap).count();
        others < 4 && !self.peers.values().any(|p| p.direction == Direction::Outbound && p.bootstrap)
    }

    pub fn remove(&mut self, id: &EndpointId) -> Option<Peer> {
        self.peers.remove(id)
    }

    /// Charge points; `Banned` means the caller must disconnect.
    pub fn charge(&mut self, id: &EndpointId, points: u32) -> Charge {
        let Some(peer) = self.peers.get_mut(id) else { return Charge::Scored(0) };
        let score = peer.charge(points);
        if score >= BAN_SCORE {
            self.bans.insert(*id, Instant::now() + Duration::from_secs(BAN_SECS));
            self.peers.remove(id);
            Charge::Banned
        } else {
            Charge::Scored(score)
        }
    }

    pub fn ban(&mut self, id: &EndpointId) {
        self.bans.insert(*id, Instant::now() + Duration::from_secs(BAN_SECS));
        self.peers.remove(id);
    }

    /// Remember the last two outbound peers that served valid blocks.
    pub fn note_served(&mut self, id: &EndpointId) {
        if let Some(p) = self.peers.get_mut(id) {
            p.served_blocks += 1;
            if p.direction == Direction::Outbound {
                self.anchors.retain(|a| a != id);
                self.anchors.insert(0, *id);
                self.anchors.truncate(2);
            }
        }
    }

    /// Outbound peers whose advertised tip is within `within` blocks of `height`.
    pub fn synced_outbound(&self, height: u64, within: u64) -> usize {
        self.peers.values().filter(|p| p.direction == Direction::Outbound && p.best_height + within >= height).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u8) -> EndpointId {
        radio::SecretKey::from_bytes(&[n; 32]).public()
    }

    #[test]
    fn bucket_burst_and_refill() {
        let t0 = Instant::now();
        let mut b = Bucket::new(64);
        for _ in 0..64 {
            assert!(b.take_at(1, t0));
        }
        assert!(!b.take_at(1, t0), "the 65th in one second is dropped (chain-conformance §11)");
        assert!(b.take_at(64, t0 + Duration::from_secs(1)));
        assert!(!b.take_at(1, t0 + Duration::from_secs(1)));
        assert!(b.take_at(256, t0 + Duration::from_secs(60)), "an idle peer may burst 4× the rate");
        assert!(!b.take_at(1, t0 + Duration::from_secs(60)));
    }

    #[test]
    fn scoring_bans_at_100_and_decays() {
        let mut t = PeerTable::new();
        t.admit(Peer::new(id(1), Direction::Inbound, vec![], false)).unwrap();
        assert_eq!(t.charge(&id(1), 10), Charge::Scored(10));
        assert_eq!(t.charge(&id(1), 90), Charge::Banned);
        assert!(t.is_banned(&id(1)));
        assert_eq!(t.admit(Peer::new(id(1), Direction::Inbound, vec![], false)), Err("banned"));
        let mut p = Peer::new(id(2), Direction::Inbound, vec![], false);
        p.charge(5);
        p.scored_at = Instant::now() - Duration::from_secs(180);
        assert_eq!(p.score_now(), 2);
    }

    #[test]
    fn outbound_diversity_and_bootstrap_floor() {
        let mut t = PeerTable::new();
        let a: SocketAddr = "10.1.0.1:1".parse().unwrap();
        let b: SocketAddr = "10.1.9.9:1".parse().unwrap();
        let c: SocketAddr = "10.1.2.3:1".parse().unwrap();
        t.admit(Peer::new(id(1), Direction::Outbound, vec![a], true)).unwrap();
        t.admit(Peer::new(id(2), Direction::Outbound, vec![b], false)).unwrap();
        assert_eq!(t.admit(Peer::new(id(3), Direction::Outbound, vec![c], false)), Err("same /16 as two peers"));
        assert!(!t.needs_bootstrap());
        t.remove(&id(1));
        assert!(t.needs_bootstrap());
        for n in 10..13 {
            t.admit(Peer::new(id(n), Direction::Outbound, vec![format!("10.{n}.0.1:1").parse().unwrap()], false)).unwrap();
        }
        assert!(!t.needs_bootstrap(), "four non-bootstrap outbound peers lift the floor");
        assert_eq!(t.outbound(), 4);
        t.note_served(&id(10));
        t.note_served(&id(11));
        t.note_served(&id(12));
        assert_eq!(t.anchors, vec![id(12), id(11)]);
    }
}
