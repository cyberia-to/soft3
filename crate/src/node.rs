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
//!   POST /v1/link  — submit a cyberlink (JSON)
//!   POST /v1/finalize — close a block (advance height)
//!
//! Persistence: `$home/log` (signal frames) + `$home/blocks` (finalize count),
//! same layout as cybergraph-cli.

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
        bump_blocks(&self.home);
        (self.height(), self.root_hex())
    }

    pub fn status_json(&self) -> String {
        let stats = self.graph.bbg.statistics();
        format!(
            r#"{{
  "jsonrpc": "2.0",
  "id": -1,
  "result": {{
    "node_info": {{
      "network": "{chain}",
      "moniker": "{moniker}",
      "version": "{version}",
      "protocol": "soft3/spacepussy-test/v1",
      "engine": "cybergraph+bbg"
    }},
    "sync_info": {{
      "latest_block_height": "{height}",
      "earliest_block_height": "0",
      "catching_up": false,
      "genesis_time": {genesis},
      "bbg_root": "{root}"
    }},
    "soft3": {{
      "role": "{role}",
      "denom": "{denom}",
      "prefix": "{prefix}",
      "rpc": "{rpc}",
      "signals": {signals},
      "neurons": {neurons},
      "particles": {particles},
      "axons": {axons},
      "node_count": {node_count},
      "max_degree": {max_degree}
    }}
  }}
}}
"#,
            chain = self.network.chain_id(),
            moniker = escape_json(&self.moniker),
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

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

// ── persistence (cybergraph-cli compatible) ─────────────────────────────────

fn open_store(home: &Path) -> Cybergraph {
    let mut cg = Cybergraph::new();
    if let Ok(bytes) = std::fs::read(home.join("log")) {
        for frame in decode_events(&bytes) {
            match frame {
                CyberFrame::Signal(s) => {
                    let _ = cg.link(s);
                }
                CyberFrame::Intent(i) => {
                    cg.bbg.apply_intent(&i);
                }
            }
        }
    }
    if let Ok(text) = std::fs::read_to_string(home.join("blocks")) {
        if let Ok(n) = text.trim().parse::<u64>() {
            for _ in 0..n {
                cg.bbg.finalize_block();
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

fn bump_blocks(home: &Path) {
    let path = home.join("blocks");
    let n = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| t.trim().parse::<u64>().ok())
        .unwrap_or(0);
    let _ = std::fs::write(path, (n + 1).to_string());
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
        eprintln!("  GET  /status /health /root /stats");
        eprintln!("  POST /v1/link  /v1/finalize");
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

    let mut buf = vec![0u8; 64 * 1024];
    let n = stream.read(&mut buf)?;
    if n == 0 {
        return Ok(());
    }
    let req = String::from_utf8_lossy(&buf[..n]);
    let mut lines = req.lines();
    let request_line = lines.next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let path = parts.next().unwrap_or("/").split('?').next().unwrap_or("/");

    // body after blank line
    let body = req
        .split("\r\n\r\n")
        .nth(1)
        .or_else(|| req.split("\n\n").nth(1))
        .unwrap_or("")
        .as_bytes();

    let (status, ctype, body_out) = match (method, path) {
        ("GET", "/status") | ("GET", "/status/") => {
            let n = node.lock().unwrap();
            ("200 OK", "application/json", n.status_json())
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
                "application/json",
                format!(
                    "{{\n  \"height\": {},\n  \"root\": \"{}\",\n  \"signals\": {},\n  \"neurons\": {},\n  \"particles\": {},\n  \"axons\": {},\n  \"node_count\": {},\n  \"max_degree\": {}\n}}\n",
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
            let mut n = node.lock().unwrap();
            let (h, root) = n.finalize();
            (
                "200 OK",
                "application/json",
                format!("{{\n  \"height\": {h},\n  \"root\": \"{root}\"\n}}\n"),
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
    let auto_finalize = v
        .get("finalize")
        .and_then(|x| x.as_bool())
        .unwrap_or(true);

    let mut n = node.lock().unwrap();
    n.link(neuron, from, to, token, amount, valence)?;
    if auto_finalize {
        let (h, root) = n.finalize();
        Ok(format!(
            "{{\n  \"ok\": true,\n  \"height\": {h},\n  \"root\": \"{root}\",\n  \"signals\": {}\n}}\n",
            n.signal_count()
        ))
    } else {
        Ok(format!(
            "{{\n  \"ok\": true,\n  \"height\": {},\n  \"root\": \"{}\",\n  \"signals\": {}\n}}\n",
            n.height(),
            n.root_hex(),
            n.signal_count()
        ))
    }
}

fn json_str(s: &str) -> String {
    format!("\"{}\"", escape_json(s))
}
