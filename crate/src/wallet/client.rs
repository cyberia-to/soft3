//! The HTTP surface of chain-wire.md §7, as a wallet reads it.
//!
//! Every answer is bounded and read once; `/status` is cybermark (one
//! `key: value` per line between `---` fences), the newer routes are JSON.
//! The parsers accept the spec's shapes and nothing cleverer: a node that
//! answers something else is reported, not guessed at.

use std::io::Read;
use std::time::{Duration, Instant};

use serde_json::Value;

use super::{hex, parse32, Error, SignedTx};

const MAX_ANSWER: usize = 8 * 1024 * 1024;

/// What `GET /status` says about a chain node (the fields a wallet uses).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Status {
    /// `chain:` — the chain's name (`pussy`, `bostrom`, `mockchain`).
    pub chain: String,
    /// `genesis:` — the chain id when the node publishes it as 64 hex digits.
    pub chain_id: Option<[u8; 32]>,
    pub height: u64,
    /// `tip:` — the block id of the tip, hex, when published.
    pub tip: String,
    /// `bbg-root:` — the state root of the tip.
    pub root: String,
    pub catching_up: bool,
    /// Every field as the node sent it.
    pub fields: Vec<(String, String)>,
}

impl Status {
    pub fn field(&self, key: &str) -> Option<&str> {
        self.fields.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    /// Parse a cybermark status document.
    pub fn parse(body: &str) -> Result<Self, Error> {
        let mut fields = Vec::new();
        for line in body.lines() {
            let line = line.trim();
            if line == "---" || line.is_empty() {
                continue;
            }
            let Some((k, v)) = line.split_once(':') else { continue };
            fields.push((k.trim().to_string(), v.trim().to_string()));
        }
        let get = |key: &str| fields.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone());
        let height = get("height")
            .and_then(|h| h.parse().ok())
            .ok_or_else(|| Error::Http("status has no height".into()))?;
        let chain_id = get("genesis").and_then(|g| parse32(&g).ok());
        Ok(Self {
            chain: get("chain").unwrap_or_default(),
            chain_id,
            height,
            tip: get("tip").unwrap_or_default(),
            root: get("bbg-root").unwrap_or_default(),
            catching_up: get("catching-up").is_some_and(|v| v == "true"),
            fields,
        })
    }
}

/// Where a submitted transaction stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TxStatus {
    /// Admitted, not yet in a block.
    Mempool,
    /// In the chain at `height`; `block` is the block id when the node names it.
    Included { height: u64, block: Option<String>, tx: Value },
    /// The node does not know the txid (dropped, never seen, or pruned).
    Unknown,
}

/// A chain node over HTTP.
pub struct Client {
    base: String,
    agent: ureq::Agent,
}

impl Client {
    /// `base` is the node's root URL (`https://cyb.ai/pussy`, `http://127.0.0.1:7781`).
    pub fn new(base: &str) -> Result<Self, Error> {
        if !(base.starts_with("http://") || base.starts_with("https://")) || base.contains(['?', '#', '@']) {
            return Err(Error::Http("node url must be http(s) without query, fragment or credentials".into()));
        }
        Ok(Self {
            base: base.trim_end_matches('/').to_string(),
            agent: ureq::AgentBuilder::new()
                .timeout_connect(Duration::from_secs(5))
                .timeout(Duration::from_secs(30))
                .redirects(0)
                .build(),
        })
    }

    pub fn base(&self) -> &str {
        &self.base
    }

    fn read(response: ureq::Response) -> Result<(u16, Vec<u8>), Error> {
        let status = response.status();
        let mut bytes = Vec::new();
        response
            .into_reader()
            .take(MAX_ANSWER as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| Error::Http(e.to_string()))?;
        if bytes.len() > MAX_ANSWER {
            return Err(Error::Http("answer exceeds 8 MiB".into()));
        }
        Ok((status, bytes))
    }

    fn get(&self, path: &str) -> Result<(u16, Vec<u8>), Error> {
        match self.agent.get(&format!("{}{path}", self.base)).call() {
            Ok(r) => Self::read(r),
            Err(ureq::Error::Status(_, r)) => Self::read(r),
            Err(e) => Err(Error::Http(e.to_string())),
        }
    }

    fn post(&self, path: &str, content_type: &str, body: &[u8]) -> Result<(u16, Vec<u8>), Error> {
        let request = self
            .agent
            .post(&format!("{}{path}", self.base))
            .set("Content-Type", content_type);
        match request.send_bytes(body) {
            Ok(r) => Self::read(r),
            Err(ureq::Error::Status(_, r)) => Self::read(r),
            Err(e) => Err(Error::Http(e.to_string())),
        }
    }

    fn ok_text(&self, path: &str) -> Result<String, Error> {
        let (status, bytes) = self.get(path)?;
        let text = String::from_utf8_lossy(&bytes).into_owned();
        if status != 200 {
            return Err(Error::Http(format!("GET {path}: HTTP {status}: {}", reason(&text))));
        }
        Ok(text)
    }

    fn ok_json(&self, path: &str) -> Result<Value, Error> {
        let text = self.ok_text(path)?;
        serde_json::from_str(&text).map_err(|e| Error::Http(format!("GET {path}: not JSON ({e})")))
    }

    /// `GET /status`.
    pub fn status(&self) -> Result<Status, Error> {
        Status::parse(&self.ok_text("/status")?)
    }

    /// The chain id a wallet must sign for: `--chain-id` when pinned,
    /// otherwise the `genesis` field of `/status`.
    pub fn chain_id(&self, pinned: Option<[u8; 32]>) -> Result<[u8; 32], Error> {
        if let Some(id) = pinned {
            return Ok(id);
        }
        self.status()?.chain_id.ok_or_else(|| {
            Error::Http("node publishes no chain id in /status (genesis); pass --chain-id".into())
        })
    }

    /// `GET /nonce/{neuron}` — the next nonce, mempool-aware. Accepts a bare
    /// integer, cybermark `nonce: N` or JSON `{"nonce": N}`.
    pub fn nonce(&self, neuron: &[u8; 32]) -> Result<u64, Error> {
        let text = self.ok_text(&format!("/nonce/{}", hex(neuron)))?;
        let t = text.trim();
        if let Ok(n) = t.parse::<u64>() {
            return Ok(n);
        }
        if let Ok(v) = serde_json::from_str::<Value>(t) {
            if let Some(n) = v.get("nonce").and_then(|n| n.as_u64().or_else(|| n.as_str()?.parse().ok())) {
                return Ok(n);
            }
        }
        cybermark_field(t, "nonce")
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| Error::Http(format!("unreadable nonce answer: {t:?}")))
    }

    /// `GET /balance/{neuron}[?token=hex]` — the balance, native by default.
    pub fn balance(&self, owner: &[u8; 32], token: Option<&[u8; 32]>) -> Result<u64, Error> {
        let query = token.map(|t| format!("?token={}", hex(t))).unwrap_or_default();
        let text = self.ok_text(&format!("/balance/{}{query}", hex(owner)))?;
        let t = text.trim();
        if let Ok(n) = t.parse::<u64>() {
            return Ok(n);
        }
        if let Ok(v) = serde_json::from_str::<Value>(t) {
            if let Some(n) = v.get("balance").and_then(|n| n.as_u64().or_else(|| n.as_str()?.parse().ok())) {
                return Ok(n);
            }
        }
        cybermark_field(t, "balance")
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| Error::Http(format!("unreadable balance answer: {t:?}")))
    }

    /// `GET /balances/{owner}` — every token of an owner, as the node sends it.
    pub fn balances(&self, owner: &[u8; 32]) -> Result<String, Error> {
        self.ok_text(&format!("/balances/{}", hex(owner)))
    }

    /// `GET /tip` — `{height, id, chainwork}`.
    pub fn tip(&self) -> Result<Value, Error> {
        self.ok_json("/tip")
    }

    /// `GET /block/{height or id}`.
    pub fn block(&self, which: &str) -> Result<Value, Error> {
        self.ok_json(&format!("/block/{which}"))
    }

    /// `GET /v2/history?after=POSITION&limit=N`.
    pub fn history(&self, after: Option<u64>, limit: u64) -> Result<Value, Error> {
        let after = after.map(|a| format!("after={a}&")).unwrap_or_default();
        self.ok_json(&format!("/v2/history?{after}limit={limit}"))
    }

    /// `POST /tx` with the wire bytes. Returns the txid the node answers
    /// with, which must equal the wallet's own; `400` carries the mempool's
    /// reason and becomes [`Error::Rejected`].
    pub fn submit(&self, tx: &SignedTx) -> Result<[u8; 32], Error> {
        let (status, bytes) = self.post("/tx", "application/octet-stream", &tx.bytes)?;
        let text = String::from_utf8_lossy(&bytes).into_owned();
        if status == 410 {
            return Err(Error::Rejected(format!("node answers 410 for /tx: {}", reason(&text))));
        }
        if status != 200 && status != 201 && status != 202 {
            return Err(Error::Rejected(format!("HTTP {status}: {}", reason(&text))));
        }
        let answered = txid_in(&text);
        match answered {
            Some(id) if id != tx.txid => Err(Error::Http(format!(
                "node computed txid {} for a tx the wallet ids {} — chain id or encoding disagree",
                hex(&id),
                tx.txid_hex()
            ))),
            _ => Ok(tx.txid),
        }
    }

    /// `GET /tx/{txid}`.
    pub fn tx(&self, txid: &[u8; 32]) -> Result<TxStatus, Error> {
        let path = format!("/tx/{}", hex(txid));
        let (status, bytes) = self.get(&path)?;
        if status == 404 {
            return Ok(TxStatus::Unknown);
        }
        let text = String::from_utf8_lossy(&bytes).into_owned();
        if status != 200 {
            return Err(Error::Http(format!("GET {path}: HTTP {status}: {}", reason(&text))));
        }
        let v: Value = serde_json::from_str(&text)
            .map_err(|e| Error::Http(format!("GET {path}: not JSON ({e})")))?;
        let height = v.get("height");
        if height.is_none() || height.is_some_and(|h| h.is_null() || h.as_str() == Some("mempool")) {
            if v.get("mempool").and_then(Value::as_bool) == Some(true) || height.is_some() || v.get("status").and_then(Value::as_str) == Some("mempool") {
                return Ok(TxStatus::Mempool);
            }
            return Err(Error::Http(format!("GET {path}: neither height nor mempool in {text}")));
        }
        let height = height
            .and_then(|h| h.as_u64().or_else(|| h.as_str()?.parse().ok()))
            .ok_or_else(|| Error::Http(format!("GET {path}: unreadable height in {text}")))?;
        let block = v.get("block").and_then(Value::as_str).map(str::to_string);
        Ok(TxStatus::Included { height, block, tx: v })
    }

    /// Poll `GET /tx/{txid}` until the transaction is in a block, or `budget`
    /// runs out. A tx the node forgets while we wait is reported as such.
    pub fn wait(&self, txid: &[u8; 32], budget: Duration) -> Result<TxStatus, Error> {
        let started = Instant::now();
        let mut seen = false;
        loop {
            match self.tx(txid)? {
                TxStatus::Included { height, block, tx } => {
                    return Ok(TxStatus::Included { height, block, tx })
                }
                TxStatus::Mempool => seen = true,
                TxStatus::Unknown if seen => {
                    return Err(Error::Rejected("transaction left the mempool without a block".into()))
                }
                TxStatus::Unknown => {}
            }
            if started.elapsed() >= budget {
                return Err(Error::Timeout(format!(
                    "{} not included after {}s",
                    hex(txid),
                    budget.as_secs()
                )));
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    }
}

fn cybermark_field(text: &str, key: &str) -> Option<String> {
    text.lines()
        .filter_map(|l| l.trim().split_once(':'))
        .find(|(k, _)| k.trim() == key)
        .map(|(_, v)| v.trim().to_string())
}

/// The error message out of a `{"error": ...}` body, else the body itself.
fn reason(text: &str) -> String {
    serde_json::from_str::<Value>(text)
        .ok()
        .and_then(|v| v.get("error").or(v.get("reason")).and_then(Value::as_str).map(str::to_string))
        .unwrap_or_else(|| text.trim().chars().take(200).collect())
}

/// The txid in a `POST /tx` answer: JSON `{"txid": hex}` or bare hex text.
fn txid_in(text: &str) -> Option<[u8; 32]> {
    let t = text.trim();
    if let Ok(id) = parse32(t) {
        return Some(id);
    }
    serde_json::from_str::<Value>(t)
        .ok()
        .and_then(|v| v.get("txid").and_then(Value::as_str).and_then(|s| parse32(s).ok()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_parses_the_chain_wire_fields() {
        let body = format!(
            "---\nparticle: status\nchain: pussy\nprotocol: soft3/chain/1\nheight: 42\ntip: {}\ngenesis: {}\nbbg-root: {}\ncatching-up: false\n---\n",
            "ab".repeat(32),
            "44".repeat(32),
            "cd".repeat(32)
        );
        let s = Status::parse(&body).unwrap();
        assert_eq!(s.chain, "pussy");
        assert_eq!(s.height, 42);
        assert_eq!(s.chain_id, Some([0x44; 32]));
        assert_eq!(s.tip, "ab".repeat(32));
        assert_eq!(s.root, "cd".repeat(32));
        assert!(!s.catching_up);
        assert_eq!(s.field("protocol"), Some("soft3/chain/1"));
    }

    #[test]
    fn status_without_height_is_an_error() {
        assert!(Status::parse("---\nchain: pussy\n---\n").is_err());
    }

    #[test]
    fn chaosnet_status_has_no_chain_id() {
        let s = Status::parse("---\nchain: spacepussy-test\nheight: 3\ngenesis: 1757000000\n---\n").unwrap();
        assert_eq!(s.chain_id, None);
    }

    #[test]
    fn txid_answers_in_both_shapes() {
        let id = "ab".repeat(32);
        assert_eq!(txid_in(&format!("{id}\n")), Some([0xab; 32]));
        assert_eq!(txid_in(&format!("{{\"txid\":\"{id}\"}}")), Some([0xab; 32]));
        assert_eq!(txid_in("{\"ok\":true}"), None);
    }

    #[test]
    fn reason_prefers_the_error_field() {
        assert_eq!(reason("{\"error\":\"bad nonce\",\"code\":\"x\"}"), "bad nonce");
        assert_eq!(reason("plain text"), "plain text");
    }

    #[test]
    fn client_rejects_non_http_urls() {
        assert!(Client::new("ftp://x").is_err());
        assert!(Client::new("http://x/?q").is_err());
        assert_eq!(Client::new("http://x/").unwrap().base(), "http://x");
    }
}
