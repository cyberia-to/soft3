//! Product networks for the soft3 stack.
//!
//! Default after install: **spacepussy-test** — the soft3 chaosnet,
//! hosted on cybernode (`rpc.spacepussy-test.soft3.org`).
//!
//! This is not the cosmos-sdk chain named `space-pussy` on cybernode
//! (rpc.space-pussy.cybernode.ai). That bootloader chain is a migration
//! source; product tools never point there by default.

use std::fmt;

/// soft3 product networks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Network {
    /// soft3 chaosnet — product default after install.
    SpacePussyTest,
}

impl Network {
    /// Product default after `cargo install soft3` / `true-cyber`.
    pub const DEFAULT: Network = Network::SpacePussyTest;

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "spacepussy-test" | "space-pussy-test" | "spacepussy_test" | "sptest"
            | "soft3-test" | "soft3" | "test" | "default" => Some(Self::SpacePussyTest),
            "space-pussy" | "spacepussy" | "pussy" | "sp" | "bostrom" | "boot" => None,
            _ => None,
        }
    }

    /// True when `s` names a cosmos bootloader chain (not a soft3 network).
    pub fn is_bootloader_name(s: &str) -> bool {
        matches!(
            s.trim().to_ascii_lowercase().as_str(),
            "space-pussy" | "spacepussy" | "pussy" | "sp" | "bostrom" | "boot"
        )
    }

    pub fn chain_id(self) -> &'static str {
        match self {
            Self::SpacePussyTest => "spacepussy-test",
        }
    }

    pub fn bech32_prefix(self) -> &'static str {
        match self {
            Self::SpacePussyTest => "pussy",
        }
    }

    pub fn denom(self) -> &'static str {
        match self {
            Self::SpacePussyTest => "testpussy",
        }
    }

    /// Public product RPC — spacepussy-test on cybernode (cyberproxy).
    /// Live edge: HTTPS under cyb.ai until rpc.spacepussy-test.soft3.org DNS is live.
    pub fn rpc(self) -> &'static str {
        match self {
            Self::SpacePussyTest => "https://cyb.ai/spacepussy-test",
        }
    }

    pub fn lcd(self) -> &'static str {
        match self {
            Self::SpacePussyTest => "https://cyb.ai/spacepussy-test",
        }
    }

    pub fn index(self) -> &'static str {
        match self {
            Self::SpacePussyTest => "https://cyb.ai/spacepussy-test",
        }
    }

    pub fn websocket(self) -> &'static str {
        match self {
            Self::SpacePussyTest => "wss://cyb.ai/spacepussy-test/ws",
        }
    }

    /// Local bind for `soft3 node` on the host (behind nginx on cyberproxy).
    pub fn local_bind(self) -> &'static str {
        match self {
            Self::SpacePussyTest => "127.0.0.1:7780",
        }
    }

    pub fn role(self) -> &'static str {
        match self {
            Self::SpacePussyTest => "soft3 chaosnet (product default)",
        }
    }
}

impl Default for Network {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.chain_id())
    }
}

/// Live status from a soft3 /status probe.
#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub network: Network,
    pub reachable: bool,
    pub http_status: Option<u16>,
    pub chain_id: String,
    pub moniker: String,
    pub latest_height: u64,
    pub earliest_height: u64,
    pub catching_up: bool,
    pub bbg_root: String,
    pub engine: String,
    pub signals: u64,
    pub particles: u64,
    pub axons: u64,
    pub body_preview: String,
}

/// Probe the network RPC. Prefers soft3 JSON `/status`; falls back to any 2xx.
pub fn probe(network: Network) -> Result<SyncStatus, String> {
    let base = network.rpc().trim_end_matches('/');
    for path in ["/status", "/health", "/"] {
        let full = format!("{base}{path}");
        match http_get(&full) {
            Ok((code, body)) => {
                if code >= 500 {
                    continue;
                }
                let mut st = SyncStatus {
                    network,
                    reachable: true,
                    http_status: Some(code),
                    chain_id: network.chain_id().into(),
                    moniker: String::new(),
                    latest_height: 0,
                    earliest_height: 0,
                    catching_up: false,
                    bbg_root: String::new(),
                    engine: String::new(),
                    signals: 0,
                    particles: 0,
                    axons: 0,
                    body_preview: body.chars().take(160).collect(),
                };
                if path.contains("status") || body.trim_start().starts_with('{') {
                    enrich_from_status_json(&mut st, &body);
                }
                return Ok(st);
            }
            Err(_) => continue,
        }
    }
    Err(format!(
        "no soft3 node at {} — spacepussy-test offline or unreachable",
        network.rpc()
    ))
}

fn enrich_from_status_json(st: &mut SyncStatus, body: &str) {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
        let result = v.get("result").unwrap_or(&v);
        if let Some(node) = result.get("node_info") {
            if let Some(s) = node.get("network").and_then(|x| x.as_str()) {
                st.chain_id = s.to_string();
            }
            if let Some(s) = node.get("moniker").and_then(|x| x.as_str()) {
                st.moniker = s.to_string();
            }
            if let Some(s) = node.get("engine").and_then(|x| x.as_str()) {
                st.engine = s.to_string();
            }
            if let Some(s) = node.get("protocol").and_then(|x| x.as_str()) {
                if st.engine.is_empty() {
                    st.engine = s.to_string();
                }
            }
        }
        if let Some(sync) = result.get("sync_info") {
            st.latest_height = json_u64(sync.get("latest_block_height")).unwrap_or(0);
            st.earliest_height = json_u64(sync.get("earliest_block_height")).unwrap_or(0);
            st.catching_up = sync
                .get("catching_up")
                .and_then(|x| x.as_bool())
                .unwrap_or(false);
            if let Some(s) = sync.get("bbg_root").and_then(|x| x.as_str()) {
                st.bbg_root = s.to_string();
            }
        }
        if let Some(soft) = result.get("soft3") {
            st.signals = json_u64(soft.get("signals")).unwrap_or(0);
            st.particles = json_u64(soft.get("particles")).unwrap_or(0);
            st.axons = json_u64(soft.get("axons")).unwrap_or(0);
        }
    }
}

fn json_u64(v: Option<&serde_json::Value>) -> Option<u64> {
    let v = v?;
    v.as_u64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}

fn http_get(url: &str) -> Result<(u16, String), String> {
    let resp = ureq::get(url)
        .timeout(std::time::Duration::from_secs(8))
        .call()
        .map_err(|e| format!("request failed: {e}"))?;
    let code = resp.status();
    let body = resp.into_string().map_err(|e| format!("body: {e}"))?;
    Ok((code, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_spacepussy_test() {
        assert_eq!(Network::DEFAULT, Network::SpacePussyTest);
        assert_eq!(Network::default().chain_id(), "spacepussy-test");
    }

    #[test]
    fn product_rpc_is_public_soft3_edge() {
        let rpc = Network::DEFAULT.rpc();
        assert!(rpc.contains("spacepussy-test"));
        assert!(rpc.starts_with("https://"));
        assert!(!rpc.contains("space-pussy.cybernode"));
        assert!(!rpc.contains("127.0.0.1"));
    }

    #[test]
    fn bootloader_names_rejected() {
        assert!(Network::parse("space-pussy").is_none());
        assert!(Network::parse("bostrom").is_none());
        assert!(Network::is_bootloader_name("space-pussy"));
    }

    fn blank_status() -> SyncStatus {
        SyncStatus {
            network: Network::DEFAULT,
            reachable: true,
            http_status: Some(200),
            chain_id: Network::DEFAULT.chain_id().into(),
            moniker: String::new(),
            latest_height: 0,
            earliest_height: 0,
            catching_up: false,
            bbg_root: String::new(),
            engine: String::new(),
            signals: 0,
            particles: 0,
            axons: 0,
            body_preview: String::new(),
        }
    }

    #[test]
    fn enrich_reads_full_status_json() {
        let mut st = blank_status();
        let body = serde_json::json!({
            "result": {
                "node_info": {"network": "pussy-rc", "moniker": "beacon-1", "engine": "foculus/1"},
                "sync_info": {
                    "latest_block_height": 41000,
                    "earliest_block_height": 1,
                    "catching_up": true,
                    "bbg_root": "abcd1234",
                },
                "soft3": {"signals": 12, "particles": 7, "axons": 3},
            }
        })
        .to_string();
        enrich_from_status_json(&mut st, &body);
        assert_eq!(st.chain_id, "pussy-rc");
        assert_eq!(st.moniker, "beacon-1");
        assert_eq!(st.engine, "foculus/1");
        assert_eq!(st.latest_height, 41000);
        assert_eq!(st.earliest_height, 1);
        assert!(st.catching_up);
        assert_eq!(st.bbg_root, "abcd1234");
        assert_eq!(st.signals, 12);
        assert_eq!(st.particles, 7);
        assert_eq!(st.axons, 3);
    }

    #[test]
    fn enrich_falls_back_to_protocol_when_engine_absent() {
        let mut st = blank_status();
        let body = serde_json::json!({
            "result": {"node_info": {"protocol": "soft3/pussy-rc/v1"}}
        })
        .to_string();
        enrich_from_status_json(&mut st, &body);
        assert_eq!(st.engine, "soft3/pussy-rc/v1");
    }

    #[test]
    fn enrich_prefers_engine_over_protocol_when_both_present() {
        let mut st = blank_status();
        let body = serde_json::json!({
            "result": {"node_info": {"engine": "foculus/1", "protocol": "soft3/pussy-rc/v1"}}
        })
        .to_string();
        enrich_from_status_json(&mut st, &body);
        assert_eq!(st.engine, "foculus/1");
    }

    #[test]
    fn enrich_tolerates_missing_result_wrapper() {
        let mut st = blank_status();
        let body = serde_json::json!({
            "node_info": {"moniker": "flat-1"},
            "sync_info": {"latest_block_height": 5},
        })
        .to_string();
        enrich_from_status_json(&mut st, &body);
        assert_eq!(st.moniker, "flat-1");
        assert_eq!(st.latest_height, 5);
    }

    #[test]
    fn enrich_leaves_defaults_on_empty_object() {
        let mut st = blank_status();
        enrich_from_status_json(&mut st, "{}");
        assert_eq!(st.moniker, "");
        assert_eq!(st.latest_height, 0);
        assert!(!st.catching_up);
    }

    #[test]
    fn enrich_does_not_panic_on_malformed_json() {
        let mut st = blank_status();
        enrich_from_status_json(&mut st, "not json at all {{{");
        assert_eq!(st.chain_id, Network::DEFAULT.chain_id());
    }

    #[test]
    fn json_u64_reads_native_number() {
        let v = serde_json::json!({"n": 42});
        assert_eq!(json_u64(v.get("n")), Some(42));
    }

    #[test]
    fn json_u64_reads_numeric_string() {
        let v = serde_json::json!({"n": "42"});
        assert_eq!(json_u64(v.get("n")), Some(42));
    }

    #[test]
    fn json_u64_none_on_missing_or_non_numeric() {
        let v = serde_json::json!({"n": "not-a-number"});
        assert_eq!(json_u64(v.get("n")), None);
        assert_eq!(json_u64(v.get("missing")), None);
        assert_eq!(json_u64(None), None);
    }
}
