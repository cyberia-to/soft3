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
    /// The pi ledger: testpussy per neuron, derived deterministically
    /// from the signal log (replayed at open, applied per signal live).
    /// Economics per tru/specs/rewards.md §8, chaosnet phase: the subsidy
    /// envelope runs pro-rata over proven work checkpoints — a link
    /// zheng -> pussy carries its weight in verified tickets and mints
    /// weight x SUBSIDY_PER_PROOF to the casting neuron. Transfers ride
    /// delta_pi: (recipient, amount), debited from the signal's neuron,
    /// skipped deterministically when funds are short.
    balances: std::collections::HashMap<NeuronId, u64>,
    /// The proof weight of the signal about to finalize, if it carries a
    /// zheng -> pussy checkpoint link — set by `link`/`pay`, consumed and
    /// zeroed by the very next `finalize()`. Both calls happen back to
    /// back under the same lock in every HTTP handler, so there is never
    /// a second signal in between to see a stale value.
    pending_weight: u64,
}

/// testpussy minted per verified ticket in a work checkpoint. The
/// bootstrap rate: one proof, one testpussy — matching the declared
/// rate the body page shows.
const SUBSIDY_PER_PROOF: u64 = 1;

/// The subsidy anchor particles (labels, hemera-hashed like key32 does).
fn subsidy_edge() -> (Particle, Particle) {
    let label = |s: &str| -> Particle {
        let h = hemera::hash(s.as_bytes());
        let b = h.as_bytes();
        let mut out = [0u8; 32];
        out[..b.len().min(32)].copy_from_slice(&b[..b.len().min(32)]);
        out
    };
    (label("zheng"), label("pussy"))
}

/// The proof weight a signal carries, if it is a zheng -> pussy checkpoint
/// link — the same amount [`apply_economics`] mints from. Zero for every
/// other kind of signal (a plain cyberlink, a pay).
fn checkpoint_weight(sig: &Signal) -> u64 {
    let (zheng_p, pussy_p) = subsidy_edge();
    sig.links
        .iter()
        .find(|l| l.from == zheng_p && l.to == pussy_p)
        .map(|l| l.amount)
        .unwrap_or(0)
}

/// Apply one signal's economics to the ledger. Called in log replay and
/// on every live signal — one rule, one order, replay-deterministic.
fn apply_economics(balances: &mut std::collections::HashMap<NeuronId, u64>, sig: &Signal) {
    let (zheng_p, pussy_p) = subsidy_edge();
    for l in &sig.links {
        if l.from == zheng_p && l.to == pussy_p {
            *balances.entry(sig.neuron).or_insert(0) +=
                l.amount.saturating_mul(SUBSIDY_PER_PROOF);
        }
    }
    for (to, amount) in &sig.delta_pi {
        let from_bal = balances.get(&sig.neuron).copied().unwrap_or(0);
        if from_bal >= *amount {
            *balances.entry(sig.neuron).or_insert(0) -= amount;
            *balances.entry(*to).or_insert(0) += amount;
        }
        // Short funds: the entry is skipped, deterministically, on every
        // replay alike. The signal still stands as a record.
    }
}

impl Node {
    pub fn open(home: PathBuf, moniker: String) -> std::io::Result<Self> {
        std::fs::create_dir_all(&home)?;
        let genesis_unix = load_or_init_genesis(&home)?;
        let (graph, balances) = open_store(&home);
        Ok(Self {
            home,
            moniker,
            network: Network::SpacePussyTest,
            genesis_unix,
            graph,
            balances,
            pending_weight: 0,
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
        apply_economics(&mut self.balances, &signal);
        self.pending_weight = checkpoint_weight(&signal);
        append_frame(&self.home, &encode_signal_frame(&signal));
        Ok(())
    }

    /// The balance of one neuron, in the chain's denom.
    pub fn balance(&self, neuron: &NeuronId) -> u64 {
        self.balances.get(neuron).copied().unwrap_or(0)
    }

    /// Total testpussy in circulation (sum of the ledger).
    pub fn pi_supply(&self) -> u64 {
        self.balances.values().sum()
    }

    /// A transfer: `amount` from `from` to `to`, as a delta_pi signal —
    /// one signal, one block, finality the moment it lands. Refused when
    /// the funds are short: the LEDGER gates here at the door, so a
    /// rejected pay never becomes a signal at all.
    pub fn pay(&mut self, from: &str, to: &str, amount: u64) -> Result<(), String> {
        let f = key32(from)?;
        let t = key32(to)?;
        if amount == 0 {
            return Err("zero amount".into());
        }
        if self.balance(&f) < amount {
            return Err(format!("insufficient funds: {} < {amount}", self.balance(&f)));
        }
        let (step, prev) = next_pos(&self.graph, &f);
        let signal = Signal {
            neuron: f,
            network: SELF_NETWORK,
            links: vec![],
            delta_pi: vec![(t, amount)],
            box_moves: vec![],
            prev,
            step,
            height: 0,
            proof: None,
        };
        self.graph
            .link(signal.clone())
            .map_err(|e| format!("pay rejected: {e:?}"))?;
        apply_economics(&mut self.balances, &signal);
        self.pending_weight = 0;
        append_frame(&self.home, &encode_signal_frame(&signal));
        Ok(())
    }

    pub fn finalize(&mut self) -> (u64, String) {
        self.graph.bbg.finalize_block();
        self.record_block();
        (self.height(), self.root_hex())
    }

    /// Append this block's observability line — local bookkeeping only,
    /// never part of the signal/wire format. `oracle` (the explorer page)
    /// is the only reader: height, wall-clock time, running pi supply, and
    /// the proof weight this block carried (0 when it wasn't a zheng ->
    /// pussy checkpoint). Covers the JSON bridge (`link`/`pay`) — the
    /// native frame path (`/v1/frame`) finalizes without going through
    /// economics at all today, so it has nothing to record here either.
    fn record_block(&mut self) {
        let line = format!(
            "{} {} {} {}\n",
            self.height(),
            now_unix(),
            self.pi_supply(),
            self.pending_weight,
        );
        self.pending_weight = 0;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.home.join("block_meta"))
        {
            let _ = f.write_all(line.as_bytes());
        }
    }

    /// The block-observability sidecar, one line per height starting from
    /// whenever this feature first ran — blocks finalized before that
    /// carry no time/supply/weight record, honestly: nothing was ever
    /// measured for them, so nothing is invented now.
    fn block_meta(&self) -> Vec<(u64, u64, u64, u64)> {
        let Ok(text) = std::fs::read_to_string(self.home.join("block_meta")) else {
            return Vec::new();
        };
        text.lines()
            .filter_map(|l| {
                let mut it = l.split_whitespace();
                let height: u64 = it.next()?.parse().ok()?;
                let time: u64 = it.next()?.parse().ok()?;
                let supply: u64 = it.next()?.parse().ok()?;
                let weight: u64 = it.next()?.parse().ok()?;
                Some((height, time, supply, weight))
            })
            .collect()
    }

    /// This block's own signal — the transactions an explorer shows for
    /// it. Replays the log exactly like [`open_store`], counting only the
    /// signals that actually finalized (native-frame Intents never do),
    /// and stops at the requested height. O(height) per call: fine at
    /// this chain's size, and correct is worth more than fast here.
    fn block_signal(&self, height: u64) -> Option<Signal> {
        let bytes = std::fs::read(self.home.join("log")).ok()?;
        let mut cg = Cybergraph::new();
        let mut seen = 0u64;
        for frame in decode_events(&bytes) {
            if let CyberFrame::Signal(s) = frame {
                if cg.link(s.clone()).is_ok() {
                    cg.bbg.finalize_block();
                    seen += 1;
                    if seen == height {
                        return Some(s);
                    }
                }
            }
        }
        None
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
fn open_store(home: &Path) -> (Cybergraph, std::collections::HashMap<NeuronId, u64>) {
    let mut cg = Cybergraph::new();
    let mut balances = std::collections::HashMap::new();
    if let Ok(bytes) = std::fs::read(home.join("log")) {
        for frame in decode_events(&bytes) {
            match frame {
                CyberFrame::Signal(s) => {
                    if cg.link(s.clone()).is_ok() {
                        cg.bbg.finalize_block();
                        apply_economics(&mut balances, &s);
                    }
                }
                CyberFrame::Intent(i) => {
                    cg.bbg.apply_intent(&i);
                }
            }
        }
    }
    (cg, balances)
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
        ("GET", p) if p.starts_with("/balance/") => {
            let arg = p.trim_start_matches("/balance/").trim_end_matches('/');
            match key32(arg) {
                Ok(neuron) => {
                    let n = node.lock().unwrap();
                    (
                        "200 OK",
                        "text/plain; charset=utf-8",
                        format!(
                            "---\nparticle: balance\nneuron: {}\ndenom: {}\nbalance: {}\nheight: {}\nsupply: {}\n---\n",
                            hex(&neuron),
                            n.network.denom(),
                            n.balance(&neuron),
                            n.height(),
                            n.pi_supply(),
                        ),
                    )
                }
                Err(e) => ("400 Bad Request", "text/plain", format!("{e}\n")),
            }
        }
        // ── oracle (block explorer) ─────────────────────────────────────
        // `/blocks` lists the observability sidecar (height/time/supply/
        // proof-weight), newest first; `/block/<height>` decodes that
        // one height's own signal for its transactions. Both are local
        // bookkeeping reads — neither touches consensus state.
        ("GET", p) if p == "/blocks" || p == "/blocks/" => {
            let n = node.lock().unwrap();
            let qp = |key: &str| -> Option<u64> {
                query
                    .split('&')
                    .find_map(|kv| kv.strip_prefix(key))
                    .and_then(|v| v.parse().ok())
            };
            let limit = qp("limit=").unwrap_or(50).clamp(1, 500) as usize;
            let before = qp("before=");
            let mut rows = n.block_meta();
            rows.sort_by(|a, b| b.0.cmp(&a.0));
            if let Some(before) = before {
                rows.retain(|(h, ..)| *h < before);
            }
            let mut out = String::from("---\nparticle: blocks\n---\n");
            for (h, t, supply, weight) in rows.into_iter().take(limit) {
                out.push_str(&format!("{h} {t} {supply} {weight}\n"));
            }
            ("200 OK", "text/plain; charset=utf-8", out)
        }
        ("GET", p) if p.starts_with("/block/") => {
            let arg = p.trim_start_matches("/block/").trim_end_matches('/');
            match arg.parse::<u64>() {
                Ok(height) => {
                    let n = node.lock().unwrap();
                    let meta = n.block_meta().into_iter().find(|(h, ..)| *h == height);
                    match n.block_signal(height) {
                        Some(sig) => {
                            let (_, time, supply, weight) = meta.unwrap_or((height, 0, 0, 0));
                            let mut out = format!(
                                "---\nparticle: block\nheight: {height}\ntime: {time}\nsupply: {supply}\nweight: {weight}\nneuron: {}\n---\n",
                                hex(&sig.neuron),
                            );
                            for l in &sig.links {
                                out.push_str(&format!(
                                    "link {} -> {} amount {} valence {}\n",
                                    hex(&l.from), hex(&l.to), l.amount, l.valence
                                ));
                            }
                            for (to, amount) in &sig.delta_pi {
                                out.push_str(&format!("pay -> {} amount {}\n", hex(to), amount));
                            }
                            ("200 OK", "text/plain; charset=utf-8", out)
                        }
                        None => (
                            "404 Not Found",
                            "text/plain",
                            format!("no block at height {height}\n"),
                        ),
                    }
                }
                Err(_) => ("400 Bad Request", "text/plain", "bad height\n".into()),
            }
        }
        ("POST", "/v1/pay") | ("POST", "/v1/pay/") => match handle_pay(node, body) {
            Ok(msg) => ("200 OK", "application/json", msg),
            Err(e) => (
                "400 Bad Request",
                "application/json",
                format!("{{\"error\":{}}}\n", json_str(&e)),
            ),
        },
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

/// `{"neuron": from, "to": recipient, "amount": N}` — strings are hex32
/// or labels, like everywhere else on this wire. The pay finalizes as its
/// own block; the response carries the finality (height, root) and the
/// sender's remaining balance.
fn handle_pay(node: &Mutex<Node>, body: &[u8]) -> Result<String, String> {
    let v: serde_json::Value =
        serde_json::from_slice(body).map_err(|e| format!("json: {e}"))?;
    let from = v
        .get("neuron")
        .and_then(|x| x.as_str())
        .ok_or_else(|| "missing neuron".to_string())?;
    let to = v
        .get("to")
        .and_then(|x| x.as_str())
        .ok_or_else(|| "missing to".to_string())?;
    let amount = v
        .get("amount")
        .and_then(|x| x.as_u64())
        .ok_or_else(|| "missing amount".to_string())?;
    let mut n = node.lock().unwrap();
    n.pay(from, to, amount)?;
    let (h, root) = n.finalize();
    let from_key = key32(from)?;
    Ok(format!(
        "{{\n  \"ok\": true,\n  \"height\": {h},\n  \"root\": \"{root}\",\n  \"balance\": {}\n}}\n",
        n.balance(&from_key)
    ))
}

fn json_str(s: &str) -> String {
    format!("\"{}\"", escape_json(s))
}

#[cfg(test)]
mod pi_tests {
    use super::*;

    fn tmp_node(tag: &str) -> Node {
        let home = std::env::temp_dir().join(format!("soft3-pi-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        Node::open(home, "test".into()).expect("open")
    }

    /// The whole loop: proven work mints, a pay moves it, short funds are
    /// refused at the door, and a REOPEN replays the same ledger.
    #[test]
    fn subsidy_pay_and_replay() {
        let mut n = tmp_node("loop");
        n.link("miner", "zheng", "pussy", "0", 100, 0).expect("checkpoint");
        n.finalize();
        let miner = key32("miner").unwrap();
        let friend = key32("friend").unwrap();
        assert_eq!(n.balance(&miner), 100 * SUBSIDY_PER_PROOF);

        n.pay("miner", "friend", 40).expect("pay");
        n.finalize();
        assert_eq!(n.balance(&miner), 60);
        assert_eq!(n.balance(&friend), 40);
        assert_eq!(n.pi_supply(), 100);

        assert!(n.pay("miner", "friend", 1000).is_err(), "short funds must refuse");

        // Replay: a fresh node over the same log derives the same ledger.
        let home = n.home.clone();
        drop(n);
        let n2 = Node::open(home, "test".into()).expect("reopen");
        assert_eq!(n2.balance(&miner), 60);
        assert_eq!(n2.balance(&friend), 40);
    }

    /// An ordinary link (not the subsidy edge) mints nothing.
    #[test]
    fn ordinary_links_mint_nothing() {
        let mut n = tmp_node("plain");
        n.link("someone", "body", "networks", "0", 5, 0).expect("link");
        n.finalize();
        assert_eq!(n.pi_supply(), 0);
    }
}
