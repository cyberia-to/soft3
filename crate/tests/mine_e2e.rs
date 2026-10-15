//! `soft3 mine` against a mock chain node: `GET /template`, `GET /tip`,
//! `POST /submit` (chain-wire §7 shapes). The mock serves a template at
//! `POW_LIMIT` (2^236, about a million hashes), verifies every submitted
//! header natively with `hemera::hash`, advances its tip on acceptance,
//! answers `409` to a header of the old tip, and drops the first template
//! request so the client's retry is exercised.

use soft3::chain::mine::api::{Accepted, Submit, Template, Tip, hex, unhex};
use soft3::chain::mine::local::{POW_LIMIT_BITS, decode_bits, encode_bits, to_bytes, valid_pow};
use soft3::chain::mine::run::{Outcome, Settings, mine_template};
use soft3::chain::mine::{Choice, api::Client, select};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct Node {
    height: u64,
    prev: [u8; 32],
    chain: [u8; 32],
    accepted: Vec<[u8; 168]>,
}

impl Node {
    fn template(&self) -> Template {
        let mut h = [0u8; 168];
        h[0..4].copy_from_slice(&1u32.to_le_bytes());
        h[4..12].copy_from_slice(&(self.height + 1).to_le_bytes());
        h[12..20].copy_from_slice(&1_793_885_850u64.to_le_bytes());
        h[20..24].copy_from_slice(&POW_LIMIT_BITS.to_le_bytes());
        h[40..72].copy_from_slice(&self.prev);
        h[72..104].fill(0x22);
        h[104..136].fill(0x33);
        h[136..168].copy_from_slice(&self.chain);
        Template {
            chain: "pussy".into(),
            height: self.height + 1,
            prev: hex(&self.prev),
            header: hex(&h),
            bits: format!("{POW_LIMIT_BITS:08x}"),
            target: hex(&to_bytes(&decode_bits(POW_LIMIT_BITS).unwrap())),
            body: "01000000".into(),
            expires: 0,
        }
    }
}

fn respond(stream: &mut TcpStream, status: &str, body: &str) {
    let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
}

fn serve(listener: TcpListener, node: Arc<Mutex<Node>>, drops: Arc<AtomicU32>) {
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { break };
        let mut buf = Vec::new();
        let mut tmp = [0u8; 4096];
        loop {
            let n = stream.read(&mut tmp).unwrap_or(0);
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&tmp[..n]);
            if let Some(end) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                let head = String::from_utf8_lossy(&buf[..end]).to_string();
                let len = head
                    .lines()
                    .find_map(|l| l.strip_prefix("Content-Length: "))
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(0);
                if buf.len() >= end + 4 + len {
                    break;
                }
            }
        }
        let text = String::from_utf8_lossy(&buf).to_string();
        let line = text.lines().next().unwrap_or("").to_string();
        let body = text.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
        let mut n = node.lock().unwrap();
        if line.starts_with("GET /template") {
            if drops.load(Relaxed) > 0 {
                drops.fetch_sub(1, Relaxed);
                drop(stream); // connection reset: the client retries
                continue;
            }
            assert!(line.contains("?to=") && line.contains(&"ab".repeat(32)), "{line}");
            respond(&mut stream, "200 OK", &serde_json::to_string(&n.template()).unwrap());
        } else if line.starts_with("GET /tip") {
            let t = Tip {
                height: n.height,
                id: hex(&n.prev),
                chainwork: "0".into(),
            };
            respond(&mut stream, "200 OK", &serde_json::to_string(&t).unwrap());
        } else if line.starts_with("POST /submit") {
            let s: Submit = serde_json::from_str(&body).expect("submit json");
            let h: [u8; 168] = unhex(&s.header).unwrap().try_into().unwrap();
            let bits = u32::from_le_bytes(h[20..24].try_into().unwrap());
            if h[40..72] != n.prev {
                respond(&mut stream, "409 Conflict", "{\"error\":\"stale template\"}");
            } else if bits != POW_LIMIT_BITS || !valid_pow(&h, &decode_bits(bits).unwrap()) {
                respond(&mut stream, "422 Unprocessable Entity", "{\"error\":\"pow\"}");
            } else {
                let id = hemera::hash(&h);
                n.height += 1;
                n.prev = *id.as_bytes();
                n.accepted.push(h);
                let a = Accepted {
                    id: hex(id.as_bytes()),
                    height: n.height,
                };
                respond(&mut stream, "200 OK", &serde_json::to_string(&a).unwrap());
            }
        } else {
            respond(&mut stream, "404 Not Found", "{}");
        }
    }
}

#[test]
fn mines_two_blocks_against_a_mock_node_with_retry_and_stale_submit() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let node = Arc::new(Mutex::new(Node {
        height: 0,
        prev: [0x11; 32],
        chain: [0x44; 32],
        accepted: Vec::new(),
    }));
    let drops = Arc::new(AtomicU32::new(1));
    {
        let (node, drops) = (node.clone(), drops.clone());
        std::thread::spawn(move || serve(listener, node, drops));
    }
    let settings = Settings {
        chain: "pussy".into(),
        node: url.clone(),
        address: "ab".repeat(32),
        tip_poll: Duration::from_millis(200),
        refresh: Duration::from_secs(60),
        report: Duration::from_millis(500),
    };
    let client = Client::new(&url, &settings.address);
    let backend = select(Choice::Auto, 0).unwrap();
    let log = |m: &str| eprintln!("{m}");
    let stop = AtomicBool::new(false);

    // first template: the first request is dropped, the retry gets it
    let t1 = Client::retry(|| client.template(), 0, Duration::from_secs(1), log).unwrap();
    assert_eq!(drops.load(Relaxed), 0);
    assert_eq!(t1.height, 1);
    assert_eq!(t1.bits_u32().unwrap(), POW_LIMIT_BITS);
    assert_eq!(encode_bits(&t1.target_u256().unwrap()), POW_LIMIT_BITS);
    let o = mine_template(&client, backend.as_ref(), &t1, 0xf00d, &settings, &stop, &log).unwrap();
    let Outcome::Accepted { height, nonce, .. } = o else { panic!("{o:?}") };
    assert_eq!(height, 1);
    assert_eq!(nonce >> 64, 0xf00d);
    {
        let n = node.lock().unwrap();
        assert_eq!(n.accepted.len(), 1);
        assert!(valid_pow(&n.accepted[0], &decode_bits(POW_LIMIT_BITS).unwrap()));
        assert_eq!(&n.accepted[0][24..40], &nonce.to_le_bytes());
    }

    // mining the old template again: the tip moved, so the miner refreshes
    // (tip poll) — or, if it finds first, the node says 409 and it refreshes
    let o = mine_template(&client, backend.as_ref(), &t1, 0xf00e, &settings, &stop, &log).unwrap();
    assert_eq!(o, Outcome::Refresh);

    // the fresh template mines block 2 on top of block 1
    let t2 = client.template().unwrap();
    assert_eq!(t2.height, 2);
    assert_ne!(t2.prev, t1.prev);
    let o = mine_template(&client, backend.as_ref(), &t2, 0xf00f, &settings, &stop, &log).unwrap();
    assert!(matches!(o, Outcome::Accepted { height: 2, .. }), "{o:?}");
    assert_eq!(node.lock().unwrap().accepted.len(), 2);
}
