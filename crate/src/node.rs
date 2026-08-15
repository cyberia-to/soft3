//! soft3-node — real spacepussy-test surface over cybergraph + bbg.
//!
//! Not a status stub. The node holds a [[cybergraph]] processor: links land
//! in authenticated bbg state, chains advance, height moves on finalize.
//!
//! HTTP (product RPC):
//!   GET  /status   — chain_id, moniker, height, root, stats
//!   GET  /health   — ok
//!   GET  /root     — BBG root hex
//!   GET  /stats    — graph statistics
//!   GET  /log      — the signal-frame log (native replication wire)
//!   POST /v1/frame — submit native signal frames (foculus encoding)
//!   POST /v1/link  — submit a cyberlink (JSON bridge)
//!
//! Persistence: `$home/log` (signal frames). Canon: one signal, one block —
//! every applied signal finalizes, so live state and log replay agree.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use cybergraph::{
    Cybergraph, CyberlinkRecord, NeuronId, Particle, Signal, SELF_NETWORK,
};
use foculus::{decode_events, encode_signal_frame, CyberFrame};

use crate::network::Network;

/// Runtime state for one spacepussy-test node.
pub struct Node {
    home: PathBuf,
    moniker: String,
    network: Network,
    genesis_unix: u64,
    graph: Cybergraph,
}

impl Node {
    pub fn open(home: PathBuf, moniker: String) -> std::io::Result<Self> {
        std::fs::create_dir_all(&home)?;
        let genesis_unix = load_or_init_genesis(&home)?;
        let graph = open_store(&home);
        Ok(Self {
            home,
            moniker,
            network: Network::SpacePussyTest,
            genesis_unix,
            graph,
        })
    }

    pub fn height(&self) -> u64 {
        self.graph.bbg.state.height
    }

    pub fn root_hex(&self) -> String {
        hex(&self.graph.bbg.state.root())
    }

    pub fn signal_count(&self) -> u64 {
        self.graph.bbg.state.signals.len() as u64
    }

    pub fn neuron_count(&self) -> u64 {
        self.graph.bbg.state.neurons.len() as u64
    }

    pub fn particle_count(&self) -> u64 {
        self.graph.bbg.state.particles.len() as u64
    }

    pub fn axon_count(&self) -> u64 {
        self.graph.bbg.state.axons_out.len() as u64
    }

    /// Submit one cyberlink. Labels (non-hex) are hashed via hemera.
    pub fn link(
        &mut self,
        neuron: &str,
        from: &str,
        to: &str,
        token: &str,
        amount: u64,
        valence: i8,
    ) -> Result<(), String> {
        let n = key32(neuron)?;
        let f = key32(from)?;
        let t = key32(to)?;
        let tok = key32(token)?;
        let (step, prev) = next_pos(&self.graph, &n);
        let signal = Signal {
            neuron: n,
            network: SELF_NETWORK,
            links: vec![CyberlinkRecord {
                neuron: n,
                from: f,
                to: t,
                token: tok,
                amount,
                valence,
                height: 0,
            }],
            delta_pi: vec![],
            box_moves: vec![],
            prev,
            step,
            height: 0,
            proof: None,
        };
        self.graph
            .link(signal.clone())
            .map_err(|e| format!("link rejected: {e:?}"))?;
        append_frame(&self.home, &encode_signal_frame(&signal));
        Ok(())
    }

    pub fn finalize(&mut self) -> (u64, String) {
        self.graph.bbg.finalize_block();
        (self.height(), self.root_hex())
    }

    /// The node's status as a cybermark particle — frontmatter plus one body
    /// line. This is the native wire: the same page format the graph speaks,
    /// readable by curl, a browser, and a forty-line parser alike.
    pub fn status_cybermark(&self) -> String {
        let stats = self.graph.bbg.statistics();
        format!(
            "---\n\
             particle: status\n\
             chain: {chain}\n\
             protocol: soft3/spacepussy-test/v2\n\
             engine: cybergraph+bbg\n\
             version: {version}\n\
             moniker: {moniker}\n\
             height: {height}\n\
             genesis: {genesis}\n\
             bbg-root: {root}\n\
             signals: {signals}\n\
             neurons: {neurons}\n\
             particles: {particles}\n\
             axons: {axons}\n\
             node-count: {node_count}\n\
             max-degree: {max_degree}\n\
             role: {role}\n\
             denom: {denom}\n\
             prefix: {prefix}\n\
             rpc: {rpc}\n\
             catching-up: false\n\
             ---\n\
             the state of [[spacepussy-test]] at height {height}\n",
            chain = self.network.chain_id(),
            moniker = frontmatter_safe(&self.moniker),
            version = env!("CARGO_PKG_VERSION"),
            height = self.height(),
            genesis = self.genesis_unix,
            root = self.root_hex(),
            role = self.network.role(),
            denom = self.network.denom(),
            prefix = self.network.bech32_prefix(),
            rpc = self.network.rpc(),
            signals = self.signal_count(),
            neurons = self.neuron_count(),
            particles = self.particle_count(),
            axons = self.axon_count(),
            node_count = stats.node_count,
            max_degree = stats.max_degree,
        )
    }
}

// ── identity helpers ────────────────────────────────────────────────────────

/// Hex (optional 0x, left-pad) or hemera-hash of the label.
fn key32(s: &str) -> Result<Particle, String> {
    if let Some(p) = parse_hex32(s) {
        return Ok(p);
    }
    // label → particle via hemera (stable product identity)
    let h = hemera::hash(s.as_bytes());
    let b = h.as_bytes();
    let mut out = [0u8; 32];
    out[..b.len().min(32)].copy_from_slice(&b[..b.len().min(32)]);
    Ok(out)
}

fn parse_hex32(s: &str) -> Option<Particle> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.is_empty() || s.len() > 64 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let padded = format!("{s:0>64}");
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = u8::from_str_radix(&padded[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(out)
}

fn next_pos(cg: &Cybergraph, neuron: &NeuronId) -> (u64, Particle) {
    match cg.chains.get(neuron) {
        Some(chain) if !chain.entries.is_empty() => {
            let step = chain.entries.len() as u64;
            let prev = chain.entries[&(step - 1)].hash();
            (step, prev)
        }
        _ => (0, [0u8; 32]),
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Frontmatter values live on one line; fold anything that would break the
/// block (newlines) into spaces.
fn frontmatter_safe(s: &str) -> String {
    s.replace(['\n', '\r'], " ")
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

// ── persistence (cybergraph-cli compatible) ─────────────────────────────────

/// Canon replay: one signal, one block. Every applied signal is followed by
/// a finalize, so the live order and any replay of the log produce the same
/// state and the same root — a peer can verify by recomputation.
fn open_store(home: &Path) -> Cybergraph {
    let mut cg = Cybergraph::new();
    if let Ok(bytes) = std::fs::read(home.join("log")) {
        for frame in decode_events(&bytes) {
            match frame {
                CyberFrame::Signal(s) => {
                    if cg.link(s).is_ok() {
                        cg.bbg.finalize_block();
                    }
                }
                CyberFrame::Intent(i) => {
                    cg.bbg.apply_intent(&i);
                }
            }
        }
    }
    cg
}

fn append_frame(home: &Path, frame: &[u8]) {
    let _ = std::fs::create_dir_all(home);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(home.join("log"))
    {
        let _ = f.write_all(frame);
    }
}

fn load_or_init_genesis(home: &Path) -> std::io::Result<u64> {
    let path = home.join("genesis.json");
    if path.exists() {
        let raw = std::fs::read_to_string(&path)?;
        if let Some(t) = parse_genesis_unix(&raw) {
            return Ok(t);
        }
    }
    let t = now_unix();
    let body = format!(
        "{{\n  \"chain_id\": \"spacepussy-test\",\n  \"genesis_time\": {t},\n  \"engine\": \"cybergraph+bbg\",\n  \"protocol\": \"soft3/spacepussy-test/v1\"\n}}\n"
    );
    std::fs::write(path, body)?;
    Ok(t)
}

fn parse_genesis_unix(raw: &str) -> Option<u64> {
    let key = "\"genesis_time\"";
    let i = raw.find(key)?;
    let rest = &raw[i + key.len()..];
    let colon = rest.find(':')?;
    let num: String = rest[colon + 1..]
        .chars()
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    num.parse().ok()
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ── HTTP server ─────────────────────────────────────────────────────────────

/// Default home for spacepussy-test.
pub fn default_home() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".spacepussy-test")
}

/// Run until process exit.
pub fn run(home: PathBuf, bind: &str, moniker: &str) -> std::io::Result<()> {
    let node = Arc::new(Mutex::new(Node::open(home.clone(), moniker.to_string())?));
    {
        let n = node.lock().unwrap();
        eprintln!(
            "soft3 node · {} · moniker={} · home={}",
            n.network.chain_id(),
            n.moniker,
            home.display()
        );
        eprintln!("  engine     cybergraph + bbg");
        eprintln!("  listening  {bind}");
        eprintln!(
            "  height {}  signals {}  particles {}",
            n.height(),
            n.signal_count(),
            n.particle_count()
        );
        eprintln!("  GET  /status /health /root /stats /log");
        eprintln!("  POST /v1/frame  /v1/link  /v1/finalize");
    }

    let listener = TcpListener::bind(bind)?;
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let n = Arc::clone(&node);
                thread::spawn(move || {
                    if let Err(e) = handle_client(s, &n) {
                        eprintln!("  request error: {e}");
                    }
                });
            }
            Err(e) => eprintln!("  accept error: {e}"),
        }
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream, node: &Mutex<Node>) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;

    // Byte-safe request read: headers first, then exactly content-length of
    // body — frames on /v1/frame are binary and must survive untouched.
    let mut raw: Vec<u8> = Vec::with_capacity(8 * 1024);
    let mut tmp = [0u8; 8 * 1024];
    let header_end = loop {
        let n = stream.read(&mut tmp)?;
        if n == 0 {
            return Ok(());
        }
        raw.extend_from_slice(&tmp[..n]);
        if let Some(pos) = find_bytes(&raw, b"\r\n\r\n") {
            break pos + 4;
        }
        if raw.len() > 64 * 1024 {
            return Ok(());
        }
    };
    let head = String::from_utf8_lossy(&raw[..header_end]).into_owned();
    let request_line = head.lines().next().unwrap_or("").to_string();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("GET").to_string();
    let full_path = parts.next().unwrap_or("/").to_string();
    let (path, query) = full_path
        .split_once('?')
        .unwrap_or((full_path.as_str(), ""));

    let content_length: usize = head
        .lines()
        .find_map(|l| {
            let (k, v) = l.split_once(':')?;
            k.trim()
                .eq_ignore_ascii_case("content-length")
                .then(|| v.trim().parse().ok())?
        })
        .unwrap_or(0);
    while raw.len() < header_end + content_length {
        let n = stream.read(&mut tmp)?;
        if n == 0 {
            break;
        }
        raw.extend_from_slice(&tmp[..n]);
    }
    let body = &raw[header_end..(header_end + content_length).min(raw.len())];
    let method = method.as_str();

    // ── native wire ─────────────────────────────────────────────────────
    // The durable log IS the replication protocol: a peer pulls the frame
    // bytes, replays them, and recomputes the root itself — verification by
    // recomputation, no trust in the served numbers.
    if method == "GET" && (path == "/log" || path == "/log/") {
        let bytes = {
            let n = node.lock().unwrap();
            std::fs::read(n.home.join("log")).unwrap_or_default()
        };
        let from: usize = query
            .split('&')
            .find_map(|kv| kv.strip_prefix("from="))
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let slice: &[u8] = if from <= bytes.len() {
            &bytes[from..]
        } else {
            &[]
        };
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
            slice.len()
        );
        stream.write_all(head.as_bytes())?;
        stream.write_all(slice)?;
        return Ok(());
    }
    if method == "POST" && (path == "/v1/frame" || path == "/v1/frame/") {
        let (status, body_out) = handle_frames(node, body);
        let resp = format!(
            "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n{body_out}",
            body_out.len()
        );
        stream.write_all(resp.as_bytes())?;
        return Ok(());
    }

    let (status, ctype, body_out) = match (method, path) {
        ("GET", "/status") | ("GET", "/status/") => {
            let n = node.lock().unwrap();
            ("200 OK", "text/plain; charset=utf-8", n.status_cybermark())
        }
        ("GET", "/health") | ("GET", "/health/") => {
            ("200 OK", "text/plain", "ok\n".into())
        }
        ("GET", "/root") | ("GET", "/root/") => {
            let n = node.lock().unwrap();
            ("200 OK", "text/plain", format!("{}\n", n.root_hex()))
        }
        ("GET", "/stats") | ("GET", "/stats/") => {
            let n = node.lock().unwrap();
            let s = n.graph.bbg.statistics();
            (
                "200 OK",
                "text/plain; charset=utf-8",
                format!(
                    "---\nparticle: stats\nheight: {}\nbbg-root: {}\nsignals: {}\nneurons: {}\nparticles: {}\naxons: {}\nnode-count: {}\nmax-degree: {}\n---\n",
                    n.height(),
                    n.root_hex(),
                    n.signal_count(),
                    n.neuron_count(),
                    n.particle_count(),
                    n.axon_count(),
                    s.node_count,
                    s.max_degree
                ),
            )
        }
        ("GET", "/") => {
            let n = node.lock().unwrap();
            (
                "200 OK",
                "text/plain",
                format!(
                    "soft3 · spacepussy-test · cybergraph+bbg\nmoniker {}\nheight {}\nsignals {}\nGET /status /stats /root\nPOST /v1/link /v1/finalize\n",
                    n.moniker,
                    n.height(),
                    n.signal_count()
                ),
            )
        }
        ("POST", "/v1/link") | ("POST", "/v1/link/") => match handle_link(node, body) {
            Ok(msg) => ("200 OK", "application/json", msg),
            Err(e) => (
                "400 Bad Request",
                "application/json",
                format!("{{\"error\":{}}}\n", json_str(&e)),
            ),
        },
        ("POST", "/v1/finalize") | ("POST", "/v1/finalize/") => {
            // Standalone empty blocks broke replay determinism: a finalize
            // between signals is invisible to the log, so live state and
            // replay diverged. Canon is one signal, one block.
            (
                "410 Gone",
                "text/plain; charset=utf-8",
                "---\nparticle: receipt\nerror: finalize is part of every signal now — one signal, one block\n---\n"
                    .into(),
            )
        }
        _ => ("404 Not Found", "text/plain", "not found\n".into()),
    };

    let resp = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n{body_out}",
        body_out.len()
    );
    stream.write_all(resp.as_bytes())?;
    Ok(())
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|w| w == needle)
}

/// Accept native signal frames (foculus encoding), apply, finalize once.
/// Answers a cybermark receipt.
fn handle_frames(node: &Mutex<Node>, body: &[u8]) -> (&'static str, String) {
    let frames = decode_events(body);
    if frames.is_empty() {
        return (
            "400 Bad Request",
            "---\nparticle: receipt\nerror: no frames decoded\n---\n".into(),
        );
    }
    let mut n = node.lock().unwrap();
    let mut applied = 0u64;
    let mut rejected = 0u64;
    for f in frames {
        match f {
            CyberFrame::Signal(s) => {
                if n.graph.link(s.clone()).is_ok() {
                    append_frame(&n.home, &encode_signal_frame(&s));
                    n.graph.bbg.finalize_block();
                    applied += 1;
                } else {
                    rejected += 1;
                }
            }
            CyberFrame::Intent(i) => {
                n.graph.bbg.apply_intent(&i);
                applied += 1;
            }
        }
    }
    let (h, root) = (n.height(), n.root_hex());
    (
        "200 OK",
        format!(
            "---\nparticle: receipt\napplied: {applied}\nrejected: {rejected}\nheight: {h}\nbbg-root: {root}\n---\n"
        ),
    )
}

fn handle_link(node: &Mutex<Node>, body: &[u8]) -> Result<String, String> {
    let v: serde_json::Value =
        serde_json::from_slice(body).map_err(|e| format!("json: {e}"))?;
    let neuron = v
        .get("neuron")
        .and_then(|x| x.as_str())
        .ok_or_else(|| "missing neuron".to_string())?;
    let from = v
        .get("from")
        .and_then(|x| x.as_str())
        .ok_or_else(|| "missing from".to_string())?;
    let to = v
        .get("to")
        .and_then(|x| x.as_str())
        .ok_or_else(|| "missing to".to_string())?;
    let token = v.get("token").and_then(|x| x.as_str()).unwrap_or("0");
    let amount = v
        .get("amount")
        .and_then(|x| x.as_u64())
        .unwrap_or(1);
    let valence = v
        .get("valence")
        .and_then(|x| x.as_i64())
        .unwrap_or(0) as i8;
    let mut n = node.lock().unwrap();
    n.link(neuron, from, to, token, amount, valence)?;
    // canon: one signal, one block — the finalize is part of the signal
    let (h, root) = n.finalize();
    Ok(format!(
        "{{\n  \"ok\": true,\n  \"height\": {h},\n  \"root\": \"{root}\",\n  \"signals\": {}\n}}\n",
        n.signal_count()
    ))
}

fn json_str(s: &str) -> String {
    format!("\"{}\"", escape_json(s))
}
