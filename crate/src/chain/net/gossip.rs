//! The gossip topics — chain-wire.md §2: headers, txs, checkpoints; relay
//! only after local validation; bodies never travel here (a pushed body is
//! +100); every message decoded strictly under 4,096 bytes and rate-limited
//! per peer before anything else.

use std::sync::Arc;

use iroh_gossip::api::{Event, GossipReceiver};
use n0_future::StreamExt;
use radio::EndpointId;

use super::chain_view::{BlockError, BlockVerdict, HeaderVerdict};
use super::codec::{hash_tagged, txid, Header};
use super::frames::{Checkpoint, Frame, FrameError, MAX_GOSSIP_BYTES};
use super::peers::{Direction, Peer};
use super::transport::request;
use super::{now_unix, short, Inner};

/// A gossip neighbor we never dialed is still a peer to score and ban.
fn ensure_peer(inner: &Inner, id: EndpointId) -> bool {
    let mut peers = inner.peers.lock().expect("peers");
    if peers.is_banned(&id) {
        return false;
    }
    if !peers.contains(&id) {
        let _ = peers.admit(Peer::new(id, Direction::Inbound, vec![], false));
    }
    true
}

fn decode(inner: &Inner, from: &EndpointId, content: &[u8]) -> Option<Frame> {
    match Frame::decode(content, MAX_GOSSIP_BYTES) {
        Ok(frame) => Some(frame),
        Err(FrameError::Oversize(_)) => {
            inner.charge(from, 100, "gossip frame over size");
            None
        }
        Err(e) => {
            inner.charge(from, 10, &format!("gossip frame: {e}"));
            None
        }
    }
}

pub(crate) async fn run_headers(inner: Arc<Inner>, mut rx: GossipReceiver) {
    // a lagged receiver (a burst beyond the channel) skips, it does not end the topic
    while let Some(event) = rx.next().await {
        let Ok(Event::Received(msg)) = event else { continue };
        let from = msg.delivered_from;
        if !ensure_peer(&inner, from) {
            continue;
        }
        let limited = inner.peers.lock().expect("peers").get_mut(&from).map(|p| !p.headers.take(1) || !p.bytes.take(msg.content.len() as u64)).unwrap_or(true);
        if limited {
            inner.charge(&from, 1, "header rate");
            continue;
        }
        match decode(&inner, &from, &msg.content) {
            Some(Frame::HeaderGossip(header)) => on_header(inner.clone(), from, header).await,
            Some(Frame::Block { .. }) => inner.charge(&from, 100, "body on gossip"),
            Some(_) => inner.charge(&from, 10, "wrong frame on headers topic"),
            None => {}
        }
    }
}

async fn on_header(inner: Arc<Inner>, from: EndpointId, header: Header) {
    let id = header.id();
    match inner.chain.accept_header(&header, now_unix()) {
        Ok(HeaderVerdict::Accepted) => {
            inner.say(&format!("header {} at {} from {}", super::codec::hex(&id[..4]), header.height, short(&from)));
            inner.note_relayed(id);
            let _ = inner.headers_tx.broadcast_neighbors(bytes::Bytes::from(Frame::HeaderGossip(header).encode())).await;
            if let Some(p) = inner.peers.lock().expect("peers").get_mut(&from) {
                if header.height > p.best_height {
                    p.best_height = header.height;
                    p.best_tip = id;
                }
            }
            // the body: from the sender when its parent is applied, else through sync
            if inner.chain.has_body(&header.prev) {
                tokio::spawn(fetch_body(inner.clone(), from, header));
            } else {
                let _ = inner.sync_hint.try_send(super::sync::Hint::Peer(from));
            }
        }
        Ok(HeaderVerdict::Known) => {
            if !inner.recently_relayed(&id) {
                inner.charge(&from, 1, "duplicate header");
            }
        }
        Ok(HeaderVerdict::Orphan) => {
            let _ = inner.sync_hint.try_send(super::sync::Hint::Peer(from));
        }
        Ok(HeaderVerdict::Held) => {}
        Err(e) => inner.charge(&from, e.score(), &format!("header: {e}")),
    }
}

/// Fetch one announced body from the announcer and apply it.
pub(crate) async fn fetch_body(inner: Arc<Inner>, from: EndpointId, header: Header) {
    let id = header.id();
    // the announcer when we hold a connection to it, else any connected peer
    let conn = inner.conn(&from).or_else(|| inner.conns.lock().expect("conns").values().next().cloned());
    let Some(conn) = conn else {
        let _ = inner.sync_hint.try_send(super::sync::Hint::Tick);
        return;
    };
    let announcer = conn.remote_id() == from;
    let from = conn.remote_id();
    match request(&conn, Frame::GetBlocks(vec![id])).await {
        Ok(frames) => match frames.first() {
            Some(Frame::Block { header: got, body }) if got.id() == id => {
                apply_fetched(&inner, &conn, got, body, header.height).await;
            }
            Some(Frame::Block { .. }) => inner.charge(&from, 100, "body for a header we did not request"),
            // header-first (chain-wire §2): the announcer relayed the header
            // before its own body fetch finished — a race, not an offence
            // (§5 charges nothing for it). Charged +1 it banned honest relays
            // after ~2 h at two blocks a minute against a decay of one point
            // a minute (simulation audit/network-2026-10.md, partition-long).
            _ if announcer => {
                let _ = inner.sync_hint.try_send(super::sync::Hint::Peer(from));
            }
            _ => {
                let _ = inner.sync_hint.try_send(super::sync::Hint::Tick);
            }
        },
        Err(e) => inner.charge(&from, e.score(), &format!("body fetch: {e}")),
    }
}

/// Hand a fetched block to the kernel. When the kernel answers
/// `NeedBodies` (a heavier chain whose ancestors' bodies we lack), fetch
/// those bodies from the same peer in height order and resubmit them.
pub(crate) async fn apply_fetched(inner: &Arc<Inner>, conn: &radio::endpoint::Connection, header: &Header, body: &[u8], height: u64) {
    let from = conn.remote_id();
    let mut pending = vec![(*header, body.to_vec())];
    let mut rounds = 0;
    while let Some((header, body)) = pending.pop() {
        match inner.chain.accept_block(&header, &body, now_unix()) {
            Ok(BlockVerdict::NewTip) | Ok(BlockVerdict::Stored) => {
                inner.peers.lock().expect("peers").note_served(&from);
                let _ = inner.sync_hint.try_send(super::sync::Hint::Tick);
            }
            Ok(BlockVerdict::Known) => {}
            Ok(BlockVerdict::NeedBodies(ids)) => {
                rounds += 1;
                if rounds > 4 {
                    // a long gap is the sync loop's job (ranges, many peers)
                    let _ = inner.sync_hint.try_send(super::sync::Hint::Peer(from));
                    return;
                }
                inner.say(&format!("block {height} needs {} ancestor bodies from {}", ids.len(), short(&from)));
                let want: Vec<_> = ids.iter().copied().take(super::frames::MAX_RANGE as usize).collect();
                match request(conn, Frame::GetBlocks(want.clone())).await {
                    Ok(frames) => {
                        // resubmit the announced block last, after its ancestors in height order
                        pending.push((header, body));
                        let mut got = Vec::new();
                        for (id, frame) in want.iter().zip(frames) {
                            match frame {
                                Frame::Block { header: h, body: b } if h.id() == *id => got.push((h, b)),
                                Frame::Block { .. } => {
                                    inner.charge(&from, 10, "block differs from the validated header");
                                    return;
                                }
                                _ => break,
                            }
                        }
                        if got.is_empty() {
                            let _ = inner.sync_hint.try_send(super::sync::Hint::Peer(from));
                            return;
                        }
                        got.sort_by_key(|(h, _)| std::cmp::Reverse(h.height));
                        pending.extend(got);
                    }
                    Err(e) => {
                        inner.charge(&from, e.score(), &format!("ancestor bodies: {e}"));
                        return;
                    }
                }
            }
            Err(BlockError::UnknownParent) => {
                let _ = inner.sync_hint.try_send(super::sync::Hint::Peer(from));
                return;
            }
            Err(e @ (BlockError::Held | BlockError::Refused(_) | BlockError::Store(_))) => {
                inner.say(&format!("block {} from {}: {e}", header.height, short(&from)));
                return;
            }
            Err(e) => {
                inner.charge(&from, e.score(), &format!("block {}: {e}", header.height));
                return;
            }
        }
    }
}

pub(crate) async fn run_txs(inner: Arc<Inner>, mut rx: GossipReceiver) {
    // a lagged receiver (a burst beyond the channel) skips, it does not end the topic
    while let Some(event) = rx.next().await {
        let Ok(Event::Received(msg)) = event else { continue };
        let from = msg.delivered_from;
        if !ensure_peer(&inner, from) {
            continue;
        }
        let limited = inner.peers.lock().expect("peers").get_mut(&from).map(|p| !p.txs.take(1) || !p.bytes.take(msg.content.len() as u64)).unwrap_or(true);
        if limited {
            inner.charge(&from, 1, "tx rate");
            continue;
        }
        match decode(&inner, &from, &msg.content) {
            Some(Frame::Tx(bytes)) => {
                let id = txid(&inner.chain.chain_id(), &bytes);
                let admitted = inner.mempool.lock().expect("mempool").admit(&bytes, inner.chain.as_ref(), now_unix());
                match admitted {
                    Ok(_) => {
                        inner.note_relayed(id);
                        let _ = inner.txs_tx.broadcast_neighbors(bytes::Bytes::from(Frame::Tx(bytes).encode())).await;
                    }
                    Err(super::mempool::AdmitError::Known) if inner.recently_relayed(&id) => {}
                    Err(e) => {
                        let peer_height = inner.peers.lock().expect("peers").get(&from).map(|p| p.best_height).unwrap_or(0);
                        let points = super::mempool::relay_charge(&e, peer_height, inner.chain.tip().height);
                        inner.charge(&from, points, &format!("tx: {e}"));
                    }
                }
            }
            Some(Frame::Block { .. }) => inner.charge(&from, 100, "body on gossip"),
            Some(_) => inner.charge(&from, 10, "wrong frame on txs topic"),
            None => {}
        }
    }
}

/// `statement = hemera("soft3/chain/checkpoint/v1\0" ‖ chain ‖ height ‖ block_id)`.
pub(crate) fn checkpoint_valid(inner: &Inner, c: &Checkpoint) -> bool {
    let Some(key) = inner.chain.checkpoint_key() else { return false };
    let statement = hash_tagged(b"soft3/chain/checkpoint/v1\0", &[&inner.chain.chain_id(), &c.height.to_le_bytes(), &c.block_id]);
    mudra::neuron::verify_statement(mudra::claim::neuron_of(&key), statement, &c.sig)
}

pub(crate) async fn run_checkpoints(inner: Arc<Inner>, mut rx: GossipReceiver) {
    // a lagged receiver (a burst beyond the channel) skips, it does not end the topic
    while let Some(event) = rx.next().await {
        let Ok(Event::Received(msg)) = event else { continue };
        let from = msg.delivered_from;
        if !ensure_peer(&inner, from) {
            continue;
        }
        let limited = inner.peers.lock().expect("peers").get_mut(&from).map(|p| !p.headers.take(1)).unwrap_or(true);
        if limited {
            inner.charge(&from, 1, "checkpoint rate");
            continue;
        }
        match decode(&inner, &from, &msg.content) {
            Some(Frame::Checkpoint(c)) => {
                if inner.chain.checkpoint_key().is_none() {
                    continue;
                }
                if !checkpoint_valid(&inner, &c) {
                    inner.charge(&from, 10, "checkpoint signature");
                    continue;
                }
                let fresh = {
                    let mut held = inner.checkpoints.lock().expect("checkpoints");
                    if held.iter().any(|h| h.height == c.height) {
                        false
                    } else {
                        held.push(c.clone());
                        held.sort_by_key(|c| c.height);
                        true
                    }
                };
                if fresh {
                    // the kernel holds it too: fork choice never crosses a held checkpoint (§5.3)
                    if let Err(e) = inner.chain.accept_checkpoint(c.height, &c.block_id, &c.sig) {
                        inner.say(&format!("checkpoint at {}: {e}", c.height));
                    }
                    let _ = inner.checkpoints_tx.broadcast_neighbors(bytes::Bytes::from(Frame::Checkpoint(c).encode())).await;
                }
            }
            Some(Frame::Block { .. }) => inner.charge(&from, 100, "body on gossip"),
            Some(_) => inner.charge(&from, 10, "wrong frame on checkpoints topic"),
            None => {}
        }
    }
}
