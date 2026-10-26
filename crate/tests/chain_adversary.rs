//! Adversarial wire tests against a real `soft3 node --chain dev` process on
//! the consensus kernel, mined by a real `soft3 mine --backend cpu` process
//! (chain-wire.md §5, chain-conformance.md §11): oversized and malformed
//! frames, a stream without the dialect, a wrong-chain status, a body
//! pushed on gossip, a PoW-invalid header (never followed by a body
//! request), a flood of transactions beyond the per-peer rate, and a peer
//! that serves a bad block. The node must stay up and score every one.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use iroh_gossip::net::{Gossip, GOSSIP_ALPN};
use iroh_gossip::proto::TopicId;
use radio::endpoint::Connection;
use radio::protocol::{AcceptError, ProtocolHandler, Router};
use radio::{Endpoint, EndpointAddr, EndpointId, SecretKey, TransportAddr};
use soft3::chain::net::codec::{self, hex, topic_id, unhex32, Header};
use soft3::chain::params::POW_LIMIT_BITS;
use soft3::chain::net::frames::{Frame, Status, DIALECT, ERR_LIMIT, ERR_WRONG_CHAIN};
use soft3::chain::net::transport::{self, read_frame, request, write_frame, ALPN};

struct NodeProc {
    child: Child,
    miner: Child,
    http: u16,
    radio: u16,
    home: PathBuf,
    id: String,
}

impl Drop for NodeProc {
    fn drop(&mut self) {
        let _ = self.miner.kill();
        let _ = self.miner.wait();
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.home);
    }
}

fn free_tcp_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

fn free_udp_port() -> u16 {
    std::net::UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

fn get(http: u16, path: &str) -> String {
    ureq::get(&format!("http://127.0.0.1:{http}{path}")).timeout(Duration::from_secs(5)).call()
        .map(|r| r.into_string().unwrap_or_default()).unwrap_or_default()
}

/// How long a healthy node may go without a new block before the test calls
/// it stalled. A block at the dev chain's floor target (bits `0x1e100000`)
/// takes 2^20 expected hashes, and the wait is exponential, so a window of
/// `T` seconds at `r` hashes/s misses a block with probability
/// `exp(-T·r / 2^20)`. The slowest rate measured for the two-thread debug
/// miner on the build service is 23.9 kH/s (diagnostic 40371902376, both
/// adversary tests running): 600 s gives `exp(-13.7)`, about 1e-6. The old
/// 15 s window was red in 13 of 30 runs of that diagnostic. A
/// passing run returns as soon as the tip moves; only a real stall waits.
const MINING_LIVENESS_SECS: u64 = 600;

fn wait_until(secs: u64, mut f: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < deadline {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(300));
    }
    false
}

/// A key's neuron, through the CLI the harness uses.
const MINER: &str = "6666666666666666666666666666666666666666666666666666666666666666";

/// One dev node in a scratch home, a test genesis at the floor target, and
/// a `soft3 mine` process working it.
fn spawn_node(label: &str) -> (NodeProc, Vec<u8>) {
    let home = std::env::temp_dir().join(format!("chain-adversary-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();
    let bin = env!("CARGO_BIN_EXE_soft3");
    let genesis_path = home.join("genesis.tade");
    let made = Command::new(bin).args(["chain", "genesis-dev", "--name", "dev", "--fund", &format!("{MINER}=1000000000"), "--out"]).arg(&genesis_path).output().unwrap();
    assert!(made.status.success(), "{}", String::from_utf8_lossy(&made.stderr));
    let genesis = std::fs::read(&genesis_path).unwrap();
    let id = String::from_utf8(Command::new(bin).args(["node", "id", "--home"]).arg(&home).output().unwrap().stdout).unwrap().trim().to_owned();
    // A port probed free can be taken before the node binds it (diagnostic
    // 40374758349, whole-14: `Address already in use`); only that failure is retried.
    let (log_path, mine_path) = (std::env::temp_dir().join(format!("chain-adversary-{label}.log")), std::env::temp_dir().join(format!("chain-adversary-{label}-mine.log")));
    for attempt in 1..=5 {
        let (http, radio) = (free_tcp_port(), free_udp_port());
        let log = std::fs::File::create(&log_path).unwrap();
        let mut child = Command::new(bin)
            .args(["node", "--chain", "dev", "--home"]).arg(&home)
            .args(["--bind", &format!("127.0.0.1:{http}"), "--radio", &format!("127.0.0.1:{radio}"), "--genesis"]).arg(&genesis_path)
            .args(["--moniker", label, "--log-net"])
            .stdout(Stdio::null()).stderr(log).spawn().unwrap();
        let mine_log = std::fs::File::create(&mine_path).unwrap();
        let mut miner = Command::new(bin)
            .args(["mine", "--chain", "dev", "--node", &format!("http://127.0.0.1:{http}"), "--address", MINER, "--backend", "cpu", "--threads", "2"])
            .stdout(mine_log).stderr(Stdio::null()).spawn().unwrap();
        let mut healthy = false;
        wait_until(30, || {
            healthy = get(http, "/health") == "ok\n";
            healthy || matches!(child.try_wait(), Ok(Some(_)))
        });
        if healthy {
            return (NodeProc { child, miner, http, radio, home, id }, genesis);
        }
        let exited = matches!(child.try_wait(), Ok(Some(_)));
        let text = std::fs::read_to_string(&log_path).unwrap_or_default();
        let _ = miner.kill();
        let _ = miner.wait();
        let _ = child.kill();
        let _ = child.wait();
        if exited && text.contains("Address already in use") && attempt < 5 {
            eprintln!("spawn_node {label}: attempt {attempt} lost a port (http {http}, radio {radio}); retrying");
            continue;
        }
        let _ = std::fs::remove_dir_all(&home);
        panic!("node did not come up\n{text}");
    }
    unreachable!()
}

/// Block 0's id, as the node publishes it.
fn genesis_block(node: &NodeProc) -> [u8; 32] {
    let v: serde_json::Value = serde_json::from_str(&get(node.http, "/header/0")).unwrap();
    unhex32(v["id"].as_str().unwrap()).unwrap()
}

/// The ASERT bits the node expects next (`next-bits` of /status).
fn next_bits(node: &NodeProc) -> u32 {
    let status = get(node.http, "/status");
    let line = status.lines().find(|l| l.starts_with("next-bits: ")).expect("next-bits");
    u32::from_str_radix(line["next-bits: 0x".len()..].trim(), 16).unwrap()
}

fn pow_ok(h: &Header) -> bool {
    soft3::chain::pow::valid_pow(&soft3::chain::header::Header::decode_raw(&h.encode()).unwrap())
}

/// Solve a header with the reference miner's CPU backend.
fn solve(h: Header) -> Header {
    use soft3::chain::mine::{cpu::Cpu, target::target_of, Backend, Job};
    let job = Job::new(&h.encode(), target_of(h.bits).unwrap());
    let cpu = Cpu::new(0);
    let mut base = 0u128;
    loop {
        if let Some(i) = cpu.search(&job, base, 1 << 16).unwrap() {
            return Header::decode(&job.header_with(base + i as u128)).unwrap();
        }
        base += 1 << 16;
    }
}

fn node_addr(node: &NodeProc) -> EndpointAddr {
    let id = EndpointId::from_bytes(&unhex32(&node.id).unwrap()).unwrap();
    let sock: SocketAddr = format!("127.0.0.1:{}", node.radio).parse().unwrap();
    EndpointAddr::from_parts(id, [TransportAddr::Ip(sock)])
}

fn peer_score(node: &NodeProc, endpoint: &EndpointId) -> Option<u64> {
    let v: serde_json::Value = serde_json::from_str(&get(node.http, "/peers")).ok()?;
    v["peers"].as_array()?.iter().find(|p| p["endpoint"] == hex(endpoint.as_bytes())).and_then(|p| p["score"].as_u64())
}

fn peer_listed(node: &NodeProc, endpoint: &EndpointId) -> bool {
    peer_score(node, endpoint).is_some()
}

async fn endpoint(seed: u8) -> Endpoint {
    transport::bind(SecretKey::from_bytes(&[seed; 32]), None, false, vec![ALPN.to_vec(), GOSSIP_ALPN.to_vec()]).await.unwrap()
}

/// A malicious responder: answers every `b` request with the announced
/// header and a garbage body, and counts the requests.
#[derive(Clone, Debug)]
struct BadServer {
    requests: Arc<AtomicUsize>,
    announced: Arc<std::sync::Mutex<Option<Header>>>,
}

impl BadServer {
    fn new() -> Self {
        BadServer { requests: Arc::new(AtomicUsize::new(0)), announced: Arc::new(std::sync::Mutex::new(None)) }
    }
}

impl ProtocolHandler for BadServer {
    async fn accept(&self, conn: Connection) -> Result<(), AcceptError> {
        while let Ok((mut send, mut recv)) = conn.accept_bi().await {
            let requests = self.requests.clone();
            let announced = self.announced.clone();
            tokio::spawn(async move {
                let Ok(req) = transport::read_request(&mut recv).await else { return };
                let _ = write_frame(&mut send, &Frame::Dialect(DIALECT.into())).await;
                match req {
                    Frame::Status(s) => {
                        let _ = write_frame(&mut send, &Frame::Status(s)).await;
                    }
                    Frame::GetBlocks(ids) => {
                        requests.fetch_add(ids.len(), Ordering::Relaxed);
                        let header = announced.lock().unwrap().unwrap_or(Header { version: 1, height: 1, time: 1, bits: POW_LIMIT_BITS, nonce: [0; 16], prev: [0; 32], tx_root: [0; 32], state_root: [0; 32], chain: [0; 32] });
                        for _ in ids {
                            let _ = write_frame(&mut send, &Frame::Block { header, body: vec![9, 9, 9, 9, 9] }).await;
                        }
                    }
                    _ => {}
                }
                let _ = write_frame(&mut send, &Frame::End).await;
                let _ = send.finish();
                let _ = send.stopped().await;
            });
        }
        Ok(())
    }
}

#[test]
fn streams_frames_and_status_are_policed() {
    let (node, genesis) = spawn_node("streams");
    let chain = codec::hash(&genesis);
    let genesis_id = genesis_block(&node);
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let ep = endpoint(1).await;
        let me = ep.id();
        let conn = ep.connect(node_addr(&node), ALPN).await.expect("connect");

        // a well-formed status: answered with the node's own, no score
        let frames = request(&conn, Frame::Status(Status { chain, rule_set: 1, height: 0, tip: genesis_id, chainwork: [0; 32], genesis: genesis_id })).await.unwrap();
        assert!(matches!(frames.first(), Some(Frame::Status(s)) if s.chain == chain && s.genesis == genesis_id), "{frames:?}");
        assert!(wait_until(5, || peer_score(&node, &me) == Some(0)));

        // get-range over the limit → x code 3
        match request(&conn, Frame::GetRange { from: 0, count: 129 }).await {
            Err(transport::WireError::Remote { code, .. }) => assert_eq!(code, ERR_LIMIT),
            other => panic!("expected x code 3, got {other:?}"),
        }

        // a stream whose first frame is not the dialect is closed and scored +10
        let (mut send, mut recv) = conn.open_bi().await.unwrap();
        write_frame(&mut send, &Frame::End).await.unwrap();
        let _ = send.finish();
        let _ = tokio::time::timeout(Duration::from_secs(5), read_frame(&mut recv, 128)).await;
        assert!(wait_until(5, || peer_score(&node, &me) == Some(10)), "score {:?}", peer_score(&node, &me));

        // a malformed request frame → +10 more
        let (mut send, mut recv) = conn.open_bi().await.unwrap();
        write_frame(&mut send, &Frame::Dialect(DIALECT.into())).await.unwrap();
        send.write_all(&[0x1f, b'#', b'r', 3, 1, 2, 3]).await.unwrap();
        let _ = send.finish();
        let _ = tokio::time::timeout(Duration::from_secs(5), read_frame(&mut recv, 128)).await;
        assert!(wait_until(5, || peer_score(&node, &me) == Some(20)), "score {:?}", peer_score(&node, &me));

        // a frame over size (a 10 MiB request) → +100: banned, connection closed
        let (mut send, _recv) = conn.open_bi().await.unwrap();
        write_frame(&mut send, &Frame::Dialect(DIALECT.into())).await.unwrap();
        let mut huge = vec![0x1f, b'#', b'g'];
        tade::encode_varint(10 * 1024 * 1024, &mut huge);
        huge.extend_from_slice(&[0u8; 64]);
        let _ = send.write_all(&huge).await;
        let _ = send.finish();
        assert!(wait_until(10, || !peer_listed(&node, &me)), "the peer was not banned");
        assert!(wait_until(10, || conn.close_reason().is_some()), "the connection stayed open");

        // banned: a new connection is refused at once
        let again = ep.connect(node_addr(&node), ALPN).await.expect("quic connects");
        let refused = tokio::time::timeout(Duration::from_secs(10), again.closed()).await;
        assert!(refused.is_ok(), "a banned peer kept its connection");

        // a status on another chain, from a fresh identity → x code 4 and disconnect
        let ep2 = endpoint(2).await;
        let conn2 = ep2.connect(node_addr(&node), ALPN).await.unwrap();
        match request(&conn2, Frame::Status(Status { chain: [1; 32], rule_set: 1, height: 0, tip: [0; 32], chainwork: [0; 32], genesis: [0; 32] })).await {
            Err(transport::WireError::Remote { code, .. }) => assert_eq!(code, ERR_WRONG_CHAIN),
            other => panic!("expected x code 4, got {other:?}"),
        }
        assert!(wait_until(10, || !peer_listed(&node, &ep2.id())));
        ep.close().await;
        ep2.close().await;
    });
    // the node is still healthy and mining
    let h1 = get(node.http, "/tip");
    assert!(get(node.http, "/health") == "ok\n");
    assert!(wait_until(MINING_LIVENESS_SECS, || get(node.http, "/tip") != h1), "the node stopped mining");
}

#[test]
fn gossip_is_policed_and_bad_blocks_are_scored() {
    let (node, genesis) = spawn_node("gossip");
    let chain = codec::hash(&genesis);
    let genesis_id = genesis_block(&node);
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        // a peer that dials the node on the chain ALPN (so the node can ask it for bodies),
        // joins the headers/txs topics and misbehaves on them
        let ep = endpoint(3).await;
        let me = ep.id();
        let bad = BadServer::new();
        let gossip = Gossip::builder().max_message_size(4096).spawn(ep.clone());
        let _router = Router::builder(ep.clone()).accept(ALPN, bad.clone()).accept(GOSSIP_ALPN, gossip.clone()).spawn();
        let conn = ep.connect(node_addr(&node), ALPN).await.unwrap();
        tokio::spawn({ let (bad, conn) = (bad.clone(), conn.clone()); async move { let _ = bad.accept(conn).await; } });
        let node_id = node_addr(&node).id;
        let status = |h: u64, tip: [u8; 32]| Frame::Status(Status { chain, rule_set: 1, height: h, tip, chainwork: [0; 32], genesis: genesis_id });
        let frames = request(&conn, status(0, genesis_id)).await.unwrap();
        assert!(matches!(frames.first(), Some(Frame::Status(_))));
        let mut headers = gossip.subscribe(TopicId::from_bytes(topic_id(&chain, "headers")), vec![node_id]).await.unwrap();
        let mut txs = gossip.subscribe(TopicId::from_bytes(topic_id(&chain, "txs")), vec![node_id]).await.unwrap();
        headers.joined().await.unwrap();
        txs.joined().await.unwrap();
        assert!(wait_until(5, || peer_score(&node, &me) == Some(0)));

        // 1. a PoW-invalid header on the headers topic: +100, banned, and no body was ever requested
        let tip: serde_json::Value = serde_json::from_str(&get(node.http, "/tip")).unwrap();
        let tip_id = unhex32(tip["id"].as_str().unwrap()).unwrap();
        let height = tip["height"].as_u64().unwrap();
        let bogus = Header {
            version: 1, height: height + 1, time: soft3::chain::net::now_unix(), bits: next_bits(&node), nonce: [0xff; 16],
            prev: tip_id, tx_root: [3; 32], state_root: [4; 32], chain,
        };
        let bogus = (0u8..=255).map(|n| Header { nonce: [n; 16], ..bogus }).find(|h| !pow_ok(h)).expect("a 2^-20 target fails almost every nonce");
        headers.broadcast_neighbors(bytes::Bytes::from(Frame::HeaderGossip(bogus).encode())).await.unwrap();
        assert!(wait_until(10, || !peer_listed(&node, &me)), "PoW-invalid header did not ban: {:?}", peer_score(&node, &me));
        assert_eq!(bad.requests.load(Ordering::Relaxed), 0, "the node asked for a body before checking PoW");
        drop(headers);
        drop(txs);
        ep.close().await;

        // 2. a fresh identity serves a bad block: valid PoW header announced, garbage body served → +10
        std::thread::sleep(Duration::from_secs(1));
        let ep = endpoint(4).await;
        let me = ep.id();
        let bad = BadServer::new();
        let gossip = Gossip::builder().max_message_size(4096).spawn(ep.clone());
        let _router = Router::builder(ep.clone()).accept(ALPN, bad.clone()).accept(GOSSIP_ALPN, gossip.clone()).spawn();
        let conn = ep.connect(node_addr(&node), ALPN).await.unwrap();
        let _ = request(&conn, status(0, genesis_id)).await.unwrap();
        tokio::spawn({ let (bad, conn) = (bad.clone(), conn.clone()); async move { let _ = bad.accept(conn).await; } });
        let mut headers = gossip.subscribe(TopicId::from_bytes(topic_id(&chain, "headers")), vec![node_id]).await.unwrap();
        headers.joined().await.unwrap();
        let tip: serde_json::Value = serde_json::from_str(&get(node.http, "/tip")).unwrap();
        let tip_id = unhex32(tip["id"].as_str().unwrap()).unwrap();
        let height = tip["height"].as_u64().unwrap();
        let candidate = Header {
            version: 1, height: height + 1, time: soft3::chain::net::now_unix() + 5, bits: next_bits(&node), nonce: [0; 16],
            prev: tip_id, tx_root: [3; 32], state_root: [4; 32], chain,
        };
        let solved = solve(candidate);
        *bad.announced.lock().unwrap() = Some(solved);
        headers.broadcast_neighbors(bytes::Bytes::from(Frame::HeaderGossip(solved).encode())).await.unwrap();
        assert!(wait_until(15, || bad.requests.load(Ordering::Relaxed) > 0), "the node never asked for the body");
        assert!(wait_until(10, || peer_score(&node, &me).is_some_and(|s| s >= 10)), "bad block not scored: {:?}", peer_score(&node, &me));
        assert!(peer_listed(&node, &me), "a single bad block must not ban");

        // 3. transaction flood: 400 junk txs in a burst → the per-peer rate drops the excess and the junk is scored; banned
        let mut txs = gossip.subscribe(TopicId::from_bytes(topic_id(&chain, "txs")), vec![node_id]).await.unwrap();
        txs.joined().await.unwrap();
        for i in 0..400u32 {
            let mut junk = vec![1u8, 1];
            junk.extend_from_slice(&[0u8; 222]);
            junk[2..6].copy_from_slice(&i.to_le_bytes());
            txs.broadcast_neighbors(bytes::Bytes::from(Frame::Tx(junk).encode())).await.unwrap();
        }
        assert!(wait_until(15, || !peer_listed(&node, &me)), "tx flood did not ban: {:?}", peer_score(&node, &me));

        // 4. a body pushed on gossip from yet another identity → +100
        let ep = endpoint(5).await;
        let me = ep.id();
        let gossip = Gossip::builder().max_message_size(4096).spawn(ep.clone());
        let _router = Router::builder(ep.clone()).accept(GOSSIP_ALPN, gossip.clone()).spawn();
        let conn = ep.connect(node_addr(&node), ALPN).await.unwrap();
        let _ = request(&conn, status(0, genesis_id)).await.unwrap();
        let mut headers = gossip.subscribe(TopicId::from_bytes(topic_id(&chain, "headers")), vec![node_id]).await.unwrap();
        headers.joined().await.unwrap();
        headers.broadcast_neighbors(bytes::Bytes::from(Frame::Block { header: solved, body: vec![1, 0, 0, 0] }.encode())).await.unwrap();
        assert!(wait_until(10, || !peer_listed(&node, &me)), "pushed body did not ban");
    });
    assert_eq!(get(node.http, "/health"), "ok\n");
    let status = get(node.http, "/status");
    assert!(status.contains("conserved: true"), "{status}");
}
