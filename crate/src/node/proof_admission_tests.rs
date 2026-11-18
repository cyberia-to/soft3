//! End to end over a real socket: `/v2/frame` reaches Cybergraph admission,
//! which verifies a carried proof (foculus pay proof, zheng public
//! certificate v3) before anything is applied.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use cybergraph::{CyberlinkRecord, Proof, Signal, SELF_NETWORK};
use foculus::signal_codec::{encode_signal, LEGACY_VERSION, MAGIC};

use super::{http, Node};

struct Served {
    address: std::net::SocketAddr,
    home: PathBuf,
}

impl Drop for Served {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.home);
    }
}

/// A node in a scratch home, served on an ephemeral localhost port.
fn serve() -> Served {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let home = std::env::temp_dir().join(format!(
        "soft3-proof-admission-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let node = Arc::new(Mutex::new(
        Node::open(home.clone(), "proof".into()).unwrap(),
    ));
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let _ = http::handle_client(stream, &node);
        }
    });
    Served { address, home }
}

/// One request; returns (status line, body).
fn request(at: &Served, method: &str, path: &str, body: &[u8]) -> (String, String) {
    let mut stream = TcpStream::connect(at.address).unwrap();
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n",
        body.len()
    )
    .unwrap();
    stream.write_all(body).unwrap();
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).unwrap();
    let raw = String::from_utf8(raw).unwrap();
    let (head, body) = raw.split_once("\r\n\r\n").unwrap();
    let status = head.lines().next().unwrap().to_owned();
    eprintln!("{method} {path} -> {status}\n{body}");
    (status, body.to_owned())
}

fn pay_signal(amounts: &[u64]) -> Signal {
    let neuron = [0x51; 32];
    Signal {
        neuron,
        network: SELF_NETWORK,
        links: amounts
            .iter()
            .enumerate()
            .map(|(i, &amount)| CyberlinkRecord {
                neuron,
                from: [0x61; 32],
                to: [0x62 + i as u8; 32],
                token: [0; 32],
                amount,
                valence: 1,
                height: 0,
            })
            .collect(),
        delta_pi: vec![],
        box_moves: vec![],
        prev: [0; 32],
        step: 0,
        height: 0,
        proof: None,
    }
}

fn proved(amounts: &[u64]) -> Signal {
    let mut signal = pay_signal(amounts);
    signal.proof = Some(foculus::prove_pay(&signal).unwrap());
    signal
}

#[test]
fn v2_frame_admits_a_valid_proved_signal() {
    let node = serve();
    let (status, body) = request(
        &node,
        "POST",
        "/v2/frame",
        &encode_signal(&proved(&[100, 50])).unwrap(),
    );
    assert_eq!(status, "HTTP/1.1 200 OK");
    for line in [
        "particle: receipt",
        "applied: 1",
        "rejected: 0",
        "height: 1",
        "signals: 1",
    ] {
        assert!(body.contains(line), "missing {line:?} in {body}");
    }
    let (_, block) = request(&node, "GET", "/block/1", b"");
    assert!(
        block.contains(&format!(
            "link {} -> {} amount 100",
            super::hex(&[0x61; 32]),
            super::hex(&[0x62; 32])
        )),
        "{block}"
    );
}

#[test]
fn v2_frame_rejects_forged_and_transplanted_proofs() {
    let node = serve();
    let (_, root) = request(&node, "GET", "/root", b"");
    let mut forged = proved(&[100, 50]);
    if let Some(Proof::Public(body)) = forged.proof.as_mut() {
        body.certificate.free[0] = (body.certificate.free[0] + 1) % GOLDILOCKS_P;
    }
    let mut transplanted = pay_signal(&[100, 50]);
    transplanted.proof = proved(&[100, 51]).proof;
    for (signal, reason) in [(forged, "Rejected"), (transplanted, "WrongStatement")] {
        let (status, body) = request(&node, "POST", "/v2/frame", &encode_signal(&signal).unwrap());
        assert_eq!(status, "HTTP/1.1 400 Bad Request");
        let value: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(value["code"], "operation_rejected");
        assert_eq!(
            value["error"],
            format!("native operation rejected: signal proof: pay proof rejected: {reason}")
        );
    }
    assert_eq!(
        request(&node, "GET", "/root", b"").1,
        root,
        "state unchanged"
    );
    // the neuron's step 0 is still free: the honest signal is admitted after
    let (status, _) = request(
        &node,
        "POST",
        "/v2/frame",
        &encode_signal(&proved(&[100, 50])).unwrap(),
    );
    assert_eq!(status, "HTTP/1.1 200 OK");
}

#[test]
fn v2_frame_rejects_a_legacy_proof_and_admits_the_bare_v1_signal() {
    let node = serve();
    let signal = pay_signal(&[7]);
    let mut v1 = encode_signal(&signal).unwrap();
    v1[MAGIC.len()] = LEGACY_VERSION;
    let mut legacy = v1.clone();
    let flag = legacy.len() - 1;
    legacy[flag] = 1;
    legacy.extend_from_slice(&[0u8; 64]);
    let (status, body) = request(&node, "POST", "/v2/frame", &legacy);
    assert_eq!(status, "HTTP/1.1 400 Bad Request");
    assert!(
        body.contains("LegacyProof") && body.contains("invalid_request"),
        "{body}"
    );
    let (status, body) = request(&node, "POST", "/v2/frame", &v1);
    assert_eq!(status, "HTTP/1.1 200 OK");
    assert!(body.contains("applied: 1"), "{body}");
}

/// The Goldilocks modulus, so a forged value stays a canonical residue.
const GOLDILOCKS_P: u64 = 0xFFFF_FFFF_0000_0001;
