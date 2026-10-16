//! Answering request streams — chain-wire.md §3: one request per stream,
//! dialect first, up to 8 streams per direction, 16 opened per second;
//! `x` codes 1 unknown, 2 pruned, 3 limit, 4 wrong chain.

use std::sync::atomic::Ordering;
use std::sync::Arc;

use radio::endpoint::Connection;
use radio::protocol::{AcceptError, ProtocolHandler};

use super::frames::{Frame, ERR_LIMIT, ERR_PRUNED, ERR_UNKNOWN, ERR_WRONG_CHAIN, MAX_BLOCKS_BY_ID, MAX_HEADERS_PER_RESPONSE, MAX_RANGE};
use super::peers::{Direction, STREAMS_PER_DIRECTION};
use super::transport::{read_request, write_frame, WireError};
use super::{codec::U256, short, Inner};

#[derive(Clone, derive_debug::Debug)]
pub struct ChainProto(pub(crate) Arc<Inner>);

mod derive_debug {
    pub use std::fmt::Debug;
}

impl std::fmt::Debug for Inner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "chain net {}", short(&self.endpoint.id()))
    }
}

impl ProtocolHandler for ChainProto {
    async fn accept(&self, conn: Connection) -> Result<(), AcceptError> {
        let inner = self.0.clone();
        let id = conn.remote_id();
        let addrs = vec![conn_addr(&conn)].into_iter().flatten().collect();
        if let Err(why) = inner.register(&conn, Direction::Inbound, addrs, false) {
            inner.say(&format!("refused inbound {}: {why}", short(&id)));
            conn.close(2u32.into(), why.as_bytes());
            return Ok(());
        }
        inner.say(&format!("inbound {} connected", short(&id)));
        inner.join_gossip(id).await;
        serve_connection(inner, conn).await;
        Ok(())
    }
}

fn conn_addr(conn: &Connection) -> Option<std::net::SocketAddr> {
    use radio::Watcher;
    conn.paths().get().iter().find_map(|p| match p.remote_addr() {
        radio::TransportAddr::Ip(addr) => Some(*addr),
        _ => None,
    })
}

/// Accept request streams until the connection ends; both sides of every
/// connection run this, since either may ask.
pub(crate) async fn serve_connection(inner: Arc<Inner>, conn: Connection) {
    let id = conn.remote_id();
    loop {
        let (send, recv) = tokio::select! {
            stream = conn.accept_bi() => match stream {
                Ok(s) => s,
                Err(_) => break,
            },
            _ = conn.closed() => break,
        };
        let allowed = {
            let mut peers = inner.peers.lock().expect("peers");
            match peers.get_mut(&id) {
                Some(p) => {
                    let ok = p.open_streams < STREAMS_PER_DIRECTION && p.streams.take(1);
                    if ok {
                        p.open_streams += 1;
                    }
                    ok
                }
                None => break,
            }
        };
        if !allowed {
            inner.charge(&id, 1, "stream limit");
            continue;
        }
        let inner = inner.clone();
        tokio::spawn(async move {
            let result = handle_stream(&inner, id, send, recv).await;
            if let Some(p) = inner.peers.lock().expect("peers").get_mut(&id) {
                p.open_streams = p.open_streams.saturating_sub(1);
            }
            if let Err(e) = result {
                inner.say(&format!("stream from {}: {e}", short(&id)));
                inner.charge(&id, e.score(), "bad stream");
            }
        });
    }
    inner.say(&format!("{} left ({})", short(&id), conn.close_reason().map(|r| r.to_string()).unwrap_or_default()));
    inner.conns.lock().expect("conns").remove(&id);
    inner.peers.lock().expect("peers").remove(&id);
}

async fn handle_stream(inner: &Arc<Inner>, id: radio::EndpointId, mut send: radio::endpoint::SendStream, mut recv: radio::endpoint::RecvStream) -> Result<(), WireError> {
    let request = read_request(&mut recv).await?;
    let frames = answer(inner, id, request).await;
    write_frame(&mut send, &Frame::Dialect(super::frames::DIALECT.into())).await?;
    for frame in &frames {
        write_frame(&mut send, frame).await?;
    }
    let _ = send.finish();
    let _ = send.stopped().await;
    // a request that banned the peer (wrong chain) closes the connection after the answer
    let banned = {
        let mut peers = inner.peers.lock().expect("peers");
        !peers.contains(&id) && peers.is_banned(&id)
    };
    if banned {
        inner.disconnect(&id, "banned");
    }
    Ok(())
}

/// The response frames for one request, ending in `e` or `x`.
pub(crate) async fn answer(inner: &Arc<Inner>, id: radio::EndpointId, request: Frame) -> Vec<Frame> {
    let chain = &inner.chain;
    let mut out = Vec::new();
    match request {
        Frame::Status(s) => {
            if s.chain != chain.chain_id() || s.genesis != chain.genesis_id() {
                // +100: banned; the stream still carries the `x` before the connection closes
                inner.say(&format!("{} is on another chain", short(&id)));
                let _ = inner.peers.lock().expect("peers").charge(&id, 100);
                return vec![Frame::Error { code: ERR_WRONG_CHAIN, reason: "wrong chain".into() }];
            }
            let ours = chain.tip();
            let mut ahead = false;
            if let Some(p) = inner.peers.lock().expect("peers").get_mut(&id) {
                p.best_height = s.height;
                p.best_tip = s.tip;
                p.chainwork = s.chainwork;
                ahead = U256::from_le_bytes(&s.chainwork) > ours.chainwork;
            }
            if ahead {
                let _ = inner.sync_hint.try_send(super::sync::Hint::Peer(id));
            }
            out.push(inner.status_frame());
        }
        Frame::GetHeaders { locator, stop } => {
            out.push(Frame::Headers(chain.headers_from_locator(&locator, &stop, MAX_HEADERS_PER_RESPONSE)));
        }
        Frame::GetRange { from, count } => {
            if count > MAX_RANGE {
                return vec![Frame::Error { code: ERR_LIMIT, reason: format!("count {count} > {MAX_RANGE}") }];
            }
            let tip = chain.tip().height;
            if from > tip {
                return vec![Frame::Error { code: ERR_UNKNOWN, reason: format!("height {from} above tip {tip}") }];
            }
            for h in from..from.saturating_add(count as u64).min(tip + 1) {
                match chain.block_at(h) {
                    Some((header, body)) => out.push(Frame::Block { header, body }),
                    None => return vec![Frame::Error { code: ERR_PRUNED, reason: format!("height {h} pruned") }],
                }
            }
        }
        Frame::GetBlocks(ids) => {
            if ids.len() > MAX_BLOCKS_BY_ID {
                return vec![Frame::Error { code: ERR_LIMIT, reason: "too many ids".into() }];
            }
            for id in ids {
                match chain.block(&id) {
                    Some((header, body)) => out.push(Frame::Block { header, body }),
                    None => {
                        out.push(Frame::Error { code: ERR_UNKNOWN, reason: "unknown block".into() });
                        return out;
                    }
                }
            }
        }
        Frame::GetTxs(ids) => {
            let pool = inner.mempool.lock().expect("mempool");
            for id in ids {
                if let Some(e) = pool.get(&id) {
                    out.push(Frame::Tx(e.bytes.clone()));
                }
            }
        }
        Frame::GetCheckpoints { from_height } => {
            for c in inner.checkpoints.lock().expect("checkpoints").iter().filter(|c| c.height >= from_height) {
                out.push(Frame::Checkpoint(c.clone()));
            }
        }
        _ => return vec![Frame::Error { code: ERR_UNKNOWN, reason: "not a request".into() }],
    }
    let _ = inner.isolated.load(Ordering::Relaxed);
    out.push(Frame::End);
    out
}
