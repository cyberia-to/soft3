//! Header-first sync and initial block download — chain-wire.md §3.1–3.2.
//! Headers come by locator from the peer with the most work; bodies are
//! fetched by id along the best header chain in ranges of 128, up to four
//! in flight to different peers, applied strictly in height order.

use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use radio::EndpointId;
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use super::chain_view::{BlockError, BlockVerdict, HeaderVerdict};
use super::codec::{Header, Id, U256};
use super::frames::{Frame, MAX_HEADERS_PER_RESPONSE, MAX_RANGE};
use super::transport::{request, WireError};
use super::{now_unix, short, Inner};

#[derive(Clone, Copy, Debug)]
pub enum Hint {
    Peer(EndpointId),
    Tick,
}

pub const RANGES_IN_FLIGHT: usize = 4;
pub const BUFFER_BLOCKS: usize = 512;
pub const TICK: Duration = Duration::from_secs(5);

pub(crate) async fn run(inner: Arc<Inner>, mut rx: mpsc::Receiver<Hint>) {
    let mut ticks = 0u64;
    loop {
        let hint = match tokio::time::timeout(TICK, rx.recv()).await {
            Ok(Some(h)) => h,
            Ok(None) => return,
            Err(_) => Hint::Tick,
        };
        // drain a burst of hints into one pass
        let mut preferred = match hint {
            Hint::Peer(p) => Some(p),
            Hint::Tick => None,
        };
        while let Ok(h) = rx.try_recv() {
            if let (Hint::Peer(p), None) = (h, preferred) {
                preferred = Some(p);
            }
        }
        if inner.isolated.load(Ordering::Relaxed) {
            continue;
        }
        ticks += 1;
        pass(&inner, preferred, ticks % 3 == 0).await;
    }
}

/// One sync pass: held headers, status-refresh, header sync from the best peer, then bodies.
async fn pass(inner: &Arc<Inner>, preferred: Option<EndpointId>, refresh: bool) {
    // headers that were ahead of our clock may be acceptable now: relay them
    for header in inner.chain.retry_held(now_unix()) {
        inner.note_relayed(header.id());
        let _ = inner.headers_tx.broadcast_neighbors(bytes::Bytes::from(Frame::HeaderGossip(header).encode())).await;
    }
    let ours = inner.chain.tip();
    // peers that claim more work, or a higher tip since their last status; every third tick, everyone
    let mut candidates: Vec<(U256, EndpointId)> = inner.peers.lock().expect("peers").iter()
        .map(|p| (U256::from_le_bytes(&p.chainwork), p.best_height, p.id))
        .filter(|(w, h, _)| refresh || *w > ours.chainwork || *h > ours.height)
        .map(|(w, _, id)| (w, id)).collect();
    candidates.sort_by(|a, b| b.0.cmp(&a.0));
    if let Some(p) = preferred {
        candidates.retain(|(_, id)| *id != p);
        candidates.insert(0, (U256::ZERO, p));
    }
    let limit = if refresh { 8 } else { 2 };
    for (_, peer) in candidates.into_iter().take(limit) {
        if let Some(conn) = inner.conn(&peer) {
            if let Err(e) = header_sync(inner, peer, &conn).await {
                inner.say(&format!("header sync with {}: {e}", short(&peer)));
                inner.charge(&peer, e.score(), "header sync");
            }
        }
        if inner.chain.best_header().chainwork > inner.chain.tip().chainwork {
            break;
        }
    }
    body_sync(inner).await;
    let tip = inner.chain.tip();
    let best = inner.chain.best_header();
    inner.catching_up.store(best.height > tip.height + 2, Ordering::Relaxed);
}

/// §3.1 steps 1–2: status, then `g` by locator until a short response.
pub(crate) async fn header_sync(inner: &Arc<Inner>, peer: EndpointId, conn: &radio::endpoint::Connection) -> Result<usize, WireError> {
    let frames = request(conn, inner.status_frame()).await?;
    let Some(Frame::Status(s)) = frames.first() else { return Err(WireError::Protocol("no status")) };
    if s.chain != inner.chain.chain_id() || s.genesis != inner.chain.genesis_id() {
        inner.disconnect(&peer, "wrong chain");
        return Err(WireError::Protocol("peer on another chain"));
    }
    let ask_checkpoints = match inner.peers.lock().expect("peers").get_mut(&peer) {
        Some(p) => {
            p.best_height = s.height;
            p.best_tip = s.tip;
            p.chainwork = s.chainwork;
            !std::mem::replace(&mut p.checkpoints_asked, true)
        }
        None => false,
    };
    // §3.1 step 1: a new connection also brings the peer's checkpoints above
    // ours. Without it a late joiner — or any node after a restart, holding
    // none — waits up to an hour for the next one to be signed, and meanwhile
    // follows a heavier rewrite deeper than MAX_REORG (simulation
    // audit/network-2026-10.md, `reorg-140-checkpoints-unfetched`).
    if ask_checkpoints && inner.chain.checkpoint_key().is_some() {
        let from_height = inner.checkpoints.lock().expect("checkpoints").last().map(|c| c.height + 1).unwrap_or(0);
        if let Ok(frames) = request(conn, Frame::GetCheckpoints { from_height }).await {
            for f in frames {
                if let Frame::Checkpoint(c) = f {
                    super::gossip::hold_checkpoint(inner, &peer, &c);
                }
            }
        }
    }
    let theirs = U256::from_le_bytes(&s.chainwork);
    if theirs <= inner.chain.best_header().chainwork && inner.chain.header(&s.tip).is_some() {
        return Ok(0);
    }
    let mut total = 0;
    for _ in 0..64 {
        let locator = best_locator(inner);
        let frames = request(conn, Frame::GetHeaders { locator, stop: [0; 32] }).await?;
        let Some(Frame::Headers(headers)) = frames.first() else { return Err(WireError::Protocol("no headers")) };
        if headers.is_empty() {
            if total == 0 {
                // no common point: another genesis
                inner.disconnect(&peer, "no common ancestor");
                return Err(WireError::Protocol("no common ancestor"));
            }
            break;
        }
        let now = now_unix();
        for h in headers {
            match inner.chain.accept_header(h, now) {
                Ok(HeaderVerdict::Accepted) => total += 1,
                Ok(HeaderVerdict::Known) => {}
                Ok(HeaderVerdict::Orphan) => return Err(WireError::Protocol("headers out of order")),
                Ok(HeaderVerdict::Held) => return Ok(total),
                Err(e) => {
                    inner.charge(&peer, e.score(), &format!("header at {}: {e}", h.height));
                    return Ok(total);
                }
            }
        }
        if headers.len() < MAX_HEADERS_PER_RESPONSE {
            break;
        }
    }
    if total > 0 {
        inner.say(&format!("{total} headers from {}", short(&peer)));
    }
    Ok(total)
}

/// The locator of our best *header* chain (bodies may lag during IBD).
fn best_locator(inner: &Inner) -> Vec<Id> {
    let best = inner.chain.best_header();
    let mut out = Vec::new();
    let mut id = best.id;
    let mut step = 1u64;
    let mut height = best.height;
    loop {
        out.push(id);
        if height == 0 || out.len() >= 255 {
            break;
        }
        if out.len() >= 10 {
            step *= 2;
        }
        let target = height.saturating_sub(step);
        // walk back `height - target` headers
        let mut cursor = id;
        let mut h = height;
        while h > target {
            match inner.chain.header(&cursor) {
                Some(hd) => {
                    cursor = hd.prev;
                    h -= 1;
                }
                None => return out,
            }
        }
        id = cursor;
        height = target;
    }
    out
}

/// §3.2: bodies of the best header chain above the tip, in ranges, in order.
pub(crate) async fn body_sync(inner: &Arc<Inner>) {
    loop {
        let missing = inner.chain.missing_bodies(BUFFER_BLOCKS);
        if missing.is_empty() {
            return;
        }
        let top = missing.last().map(|m| m.0).unwrap_or(0);
        // only peers we hold a request connection to; those at the range's end first
        let connected: Vec<EndpointId> = inner.conns.lock().expect("conns").keys().copied().collect();
        let mut peers: Vec<EndpointId> = inner.peers.lock().expect("peers").iter()
            .filter(|p| p.best_height >= top && connected.contains(&p.id)).map(|p| p.id).collect();
        if peers.is_empty() {
            peers = connected;
        }
        if peers.is_empty() {
            return;
        }
        let mut set = JoinSet::new();
        for (i, range) in missing.chunks(MAX_RANGE as usize).take(RANGES_IN_FLIGHT).enumerate() {
            let peer = peers[i % peers.len()];
            let Some(conn) = inner.conn(&peer) else { continue };
            let want: Vec<(u64, Id)> = range.to_vec();
            set.spawn(async move {
                let ids: Vec<Id> = want.iter().map(|(_, id)| *id).collect();
                let got = request(&conn, Frame::GetBlocks(ids)).await;
                (peer, want, got)
            });
        }
        let mut buffer: BTreeMap<u64, (Header, Vec<u8>, EndpointId)> = BTreeMap::new();
        while let Some(Ok((peer, want, got))) = set.join_next().await {
            match got {
                Ok(frames) => {
                    for ((height, id), frame) in want.iter().zip(frames) {
                        match frame {
                            Frame::Block { header, body } if header.id() == *id => {
                                buffer.insert(*height, (header, body, peer));
                            }
                            Frame::Block { .. } => {
                                inner.charge(&peer, 10, "block differs from the validated header");
                                break;
                            }
                            _ => break,
                        }
                    }
                }
                Err(e) => inner.charge(&peer, e.score(), &format!("range: {e}")),
            }
            if buffer.len() > BUFFER_BLOCKS {
                break;
            }
        }
        if buffer.is_empty() {
            return;
        }
        let mut applied = 0;
        for (height, (header, body, peer)) in buffer {
            match inner.chain.accept_block(&header, &body, now_unix()) {
                // NeedBodies: stored; the next round's missing list carries the gap
                Ok(BlockVerdict::NewTip) | Ok(BlockVerdict::Stored) | Ok(BlockVerdict::Known) | Ok(BlockVerdict::NeedBodies(_)) => applied += 1,
                Err(e @ (BlockError::Held | BlockError::Refused(_) | BlockError::Store(_) | BlockError::UnknownParent)) => {
                    inner.say(&format!("block {height}: {e}"));
                    return;
                }
                Err(e) => {
                    inner.charge(&peer, e.score(), &format!("block {height}: {e}"));
                    return;
                }
            }
        }
        inner.say(&format!("applied {applied} blocks, tip {}", inner.chain.tip().height));
        if applied == 0 {
            return;
        }
    }
}
