//! Minimal spacepussy-test node — product chaosnet surface.
//!
//! Serves HTTP on the product RPC port (default 7780):
//!   GET /status  — chain identity + height (JSON)
//!   GET /health  — ok
//!   GET /        — short human blurb
//!
//! Height advances every BLOCK_SECS from a genesis wall-clock so a live
//! node always reports a moving tip. Full soft3-node (cybergraph + radio +
//! foculus) will replace this surface; the /status contract stays.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::network::Network;

/// Seconds per height tick (soft block time for the chaosnet scaffold).
const BLOCK_SECS: u64 = 5;

/// On-disk state for a long-lived node.
#[derive(Clone)]
struct NodeState {
    #[allow(dead_code)]
    home: PathBuf,
    network: Network,
    moniker: String,
    genesis_unix: u64,
    /// Manual height offset (link events later); tip = tick_height + offset.
    height_offset: Arc<AtomicU64>,
}

impl NodeState {
    fn load_or_init(home: &Path, moniker: &str) -> std::io::Result<Self> {
        std::fs::create_dir_all(home)?;
        let genesis_path = home.join("genesis.json");
        let genesis_unix = if genesis_path.exists() {
            let raw = std::fs::read_to_string(&genesis_path)?;
            parse_genesis_unix(&raw).unwrap_or_else(now_unix)
        } else {
            let t = now_unix();
            let body = format!(
                "{{\n  \"chain_id\": \"{}\",\n  \"genesis_time\": {},\n  \"network\": \"spacepussy-test\"\n}}\n",
                Network::SpacePussyTest.chain_id(),
                t
            );
            std::fs::write(&genesis_path, body)?;
            t
        };
        let offset_path = home.join("height_offset");
        let height_offset = if offset_path.exists() {
            std::fs::read_to_string(&offset_path)
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0)
        } else {
            0
        };
        Ok(Self {
            home: home.to_path_buf(),
            network: Network::SpacePussyTest,
            moniker: moniker.to_string(),
            genesis_unix,
            height_offset: Arc::new(AtomicU64::new(height_offset)),
        })
    }

    fn tip_height(&self) -> u64 {
        let elapsed = now_unix().saturating_sub(self.genesis_unix);
        let tick = elapsed / BLOCK_SECS;
        tick + self.height_offset.load(Ordering::Relaxed)
    }

    fn status_json(&self) -> String {
        let height = self.tip_height();
        let n = self.network;
        format!(
            r#"{{
  "jsonrpc": "2.0",
  "id": -1,
  "result": {{
    "node_info": {{
      "network": "{chain}",
      "moniker": "{moniker}",
      "version": "{version}",
      "protocol": "soft3/spacepussy-test/v0"
    }},
    "sync_info": {{
      "latest_block_height": "{height}",
      "earliest_block_height": "1",
      "catching_up": false,
      "genesis_time": {genesis}
    }},
    "soft3": {{
      "role": "{role}",
      "denom": "{denom}",
      "prefix": "{prefix}",
      "rpc": "{rpc}",
      "block_secs": {block_secs}
    }}
  }}
}}
"#,
            chain = n.chain_id(),
            moniker = escape_json(&self.moniker),
            version = env!("CARGO_PKG_VERSION"),
            height = height,
            genesis = self.genesis_unix,
            role = n.role(),
            denom = n.denom(),
            prefix = n.bech32_prefix(),
            rpc = n.rpc(),
            block_secs = BLOCK_SECS,
        )
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn parse_genesis_unix(raw: &str) -> Option<u64> {
    // tiny extract: "genesis_time": 123
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

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Run the node until the process is killed.
pub fn run(home: PathBuf, bind: &str, moniker: &str) -> std::io::Result<()> {
    let state = Arc::new(NodeState::load_or_init(&home, moniker)?);
    let listener = TcpListener::bind(bind)?;
    eprintln!(
        "soft3 node · {} · moniker={} · home={}",
        state.network.chain_id(),
        state.moniker,
        home.display()
    );
    eprintln!("  listening  {bind}");
    eprintln!("  tip height {}", state.tip_height());
    eprintln!("  GET /status  /health  /");

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let st = Arc::clone(&state);
                thread::spawn(move || {
                    if let Err(e) = handle_client(s, &st) {
                        eprintln!("  request error: {e}");
                    }
                });
            }
            Err(e) => eprintln!("  accept error: {e}"),
        }
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream, state: &NodeState) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf)?;
    if n == 0 {
        return Ok(());
    }
    let req = String::from_utf8_lossy(&buf[..n]);
    let path = req
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    let (status, content_type, body) = match path.split('?').next().unwrap_or(path) {
        "/status" | "/status/" => ("200 OK", "application/json", state.status_json()),
        "/health" | "/health/" => ("200 OK", "text/plain", "ok\n".into()),
        "/" => (
            "200 OK",
            "text/plain",
            format!(
                "soft3 · spacepussy-test\nmoniker {}\nheight {}\nrpc /status\n",
                state.moniker,
                state.tip_height()
            ),
        ),
        _ => ("404 Not Found", "text/plain", "not found\n".into()),
    };

    let resp = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(resp.as_bytes())?;
    Ok(())
}

/// Default home directory for the product chaosnet.
pub fn default_home() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".spacepussy-test")
}
