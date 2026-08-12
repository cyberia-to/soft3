//! Default networks for soft3 / cyb / cyber stack.
//!
//! **Space Pussy is the default sync target** after install — the living
//! bootloader graph used for product smoke tests and light-client sync.

use std::fmt;

/// Known cyber ecosystems (Cosmos SDK cyber-family chains).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Network {
    /// Space Pussy — default product / test sync network.
    SpacePussy,
    /// Bostrom — main bootloader chain.
    Bostrom,
}

impl Network {
    /// Product default after `cargo install soft3` / `cyb`.
    pub const DEFAULT: Network = Network::SpacePussy;

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "space-pussy" | "spacepussy" | "pussy" | "sp" => Some(Self::SpacePussy),
            "bostrom" | "boot" => Some(Self::Bostrom),
            _ => None,
        }
    }

    pub fn chain_id(self) -> &'static str {
        match self {
            Self::SpacePussy => "space-pussy",
            Self::Bostrom => "bostrom",
        }
    }

    pub fn bech32_prefix(self) -> &'static str {
        match self {
            Self::SpacePussy => "pussy",
            Self::Bostrom => "bostrom",
        }
    }

    pub fn denom(self) -> &'static str {
        match self {
            Self::SpacePussy => "pussy",
            Self::Bostrom => "boot",
        }
    }

    /// Tendermint RPC (HTTPS).
    pub fn rpc(self) -> &'static str {
        match self {
            Self::SpacePussy => "https://rpc.space-pussy.cybernode.ai",
            Self::Bostrom => "https://rpc.bostrom.cybernode.ai",
        }
    }

    /// REST LCD.
    pub fn lcd(self) -> &'static str {
        match self {
            Self::SpacePussy => "https://lcd.space-pussy.cybernode.ai",
            Self::Bostrom => "https://lcd.bostrom.cybernode.ai",
        }
    }

    /// GraphQL index.
    pub fn index(self) -> &'static str {
        match self {
            Self::SpacePussy => "https://index.space-pussy.cybernode.ai/v1/graphql",
            Self::Bostrom => "https://index.bostrom.cybernode.ai/v1/graphql",
        }
    }

    pub fn websocket(self) -> &'static str {
        match self {
            Self::SpacePussy => "wss://rpc.space-pussy.cybernode.ai/websocket",
            Self::Bostrom => "wss://rpc.bostrom.cybernode.ai/websocket",
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

/// Live status pulled from Tendermint `/status`.
#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub network: Network,
    pub chain_id: String,
    pub moniker: String,
    pub latest_height: u64,
    pub catching_up: bool,
    pub earliest_height: u64,
}

/// Probe default (Space Pussy) or the given network RPC.
pub fn probe(network: Network) -> Result<SyncStatus, String> {
    let url = format!("{}/status", network.rpc().trim_end_matches('/'));
    let body = http_get(&url)?;
    parse_status(network, &body)
}

fn http_get(url: &str) -> Result<String, String> {
    // minimal dependency-free GET via `ureq` if present; else std-only fallback with curl-less TCP is hard.
    // We use ureq from soft3 Cargo.toml.
    ureq::get(url)
        .timeout(std::time::Duration::from_secs(12))
        .call()
        .map_err(|e| format!("rpc request failed: {e}"))?
        .into_string()
        .map_err(|e| format!("rpc body: {e}"))
}

fn parse_status(network: Network, body: &str) -> Result<SyncStatus, String> {
    // tiny hand-parse to avoid serde_json weight in edge cases; use serde_json for reliability
    let v: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("status json: {e}"))?;
    let result = v
        .get("result")
        .ok_or_else(|| "missing result".to_string())?;
    let node = result
        .get("node_info")
        .ok_or_else(|| "missing node_info".to_string())?;
    let sync = result
        .get("sync_info")
        .ok_or_else(|| "missing sync_info".to_string())?;

    let chain_id = node
        .get("network")
        .and_then(|x| x.as_str())
        .unwrap_or("?")
        .to_string();
    let moniker = node
        .get("moniker")
        .and_then(|x| x.as_str())
        .unwrap_or("?")
        .to_string();
    let latest_height = sync
        .get("latest_block_height")
        .and_then(|x| x.as_str())
        .and_then(|s| s.parse().ok())
        .or_else(|| sync.get("latest_block_height").and_then(|x| x.as_u64()))
        .unwrap_or(0);
    let earliest_height = sync
        .get("earliest_block_height")
        .and_then(|x| x.as_str())
        .and_then(|s| s.parse().ok())
        .or_else(|| sync.get("earliest_block_height").and_then(|x| x.as_u64()))
        .unwrap_or(0);
    let catching_up = sync
        .get("catching_up")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);

    Ok(SyncStatus {
        network,
        chain_id,
        moniker,
        latest_height,
        catching_up,
        earliest_height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_space_pussy() {
        assert_eq!(Network::DEFAULT, Network::SpacePussy);
        assert_eq!(Network::default().chain_id(), "space-pussy");
    }

    #[test]
    fn parse_aliases() {
        assert_eq!(Network::parse("pussy"), Some(Network::SpacePussy));
        assert_eq!(Network::parse("bostrom"), Some(Network::Bostrom));
    }
}
