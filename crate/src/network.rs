//! Product networks for the soft3 stack.
//!
//! Default after install: **spacepussy-test** — the soft3 chaosnet.
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
            "spacepussy-test"
            | "space-pussy-test"
            | "spacepussy_test"
            | "sptest"
            | "soft3-test"
            | "soft3"
            | "test"
            | "default" => Some(Self::SpacePussyTest),
            // explicit rejection of bootloader cosmos names — never silent alias
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
            // soft3-native naming — not cosmos `pussy` prefix
            Self::SpacePussyTest => "pussy",
        }
    }

    pub fn denom(self) -> &'static str {
        match self {
            Self::SpacePussyTest => "testpussy",
        }
    }

    /// Default local soft3 node RPC (product chaosnet).
    pub fn rpc(self) -> &'static str {
        match self {
            Self::SpacePussyTest => "http://127.0.0.1:7780",
        }
    }

    pub fn lcd(self) -> &'static str {
        match self {
            Self::SpacePussyTest => "http://127.0.0.1:7781",
        }
    }

    pub fn index(self) -> &'static str {
        match self {
            Self::SpacePussyTest => "http://127.0.0.1:7782",
        }
    }

    pub fn websocket(self) -> &'static str {
        match self {
            Self::SpacePussyTest => "ws://127.0.0.1:7780/ws",
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

/// Reachability / light status for a soft3 network endpoint.
#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub network: Network,
    pub reachable: bool,
    pub http_status: Option<u16>,
    pub body_preview: String,
}

/// Probe the network RPC. soft3 nodes are not cosmos tendermint — we only
/// check that something answers. A connection error means the local (or
/// configured) spacepussy-test node is not running.
pub fn probe(network: Network) -> Result<SyncStatus, String> {
    let url = network.rpc().trim_end_matches('/').to_string();
    // try /status then bare root — soft3 node surface is still settling
    for path in ["/status", "/", "/health"] {
        let full = format!("{url}{path}");
        match http_get(&full) {
            Ok((code, body)) => {
                let preview: String = body.chars().take(120).collect();
                return Ok(SyncStatus {
                    network,
                    reachable: code < 500,
                    http_status: Some(code),
                    body_preview: preview,
                });
            }
            Err(_) => continue,
        }
    }
    Err(format!(
        "no soft3 node at {} — start spacepussy-test locally or set the rpc",
        network.rpc()
    ))
}

fn http_get(url: &str) -> Result<(u16, String), String> {
    let resp = ureq::get(url)
        .timeout(std::time::Duration::from_secs(3))
        .call()
        .map_err(|e| format!("request failed: {e}"))?;
    let code = resp.status();
    let body = resp
        .into_string()
        .map_err(|e| format!("body: {e}"))?;
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
    fn parse_product_aliases() {
        assert_eq!(Network::parse("spacepussy-test"), Some(Network::SpacePussyTest));
        assert_eq!(Network::parse("test"), Some(Network::SpacePussyTest));
        assert_eq!(Network::parse("soft3"), Some(Network::SpacePussyTest));
    }

    #[test]
    fn bootloader_names_rejected() {
        assert!(Network::parse("space-pussy").is_none());
        assert!(Network::parse("bostrom").is_none());
        assert!(Network::parse("pussy").is_none());
        assert!(Network::is_bootloader_name("space-pussy"));
        assert!(Network::is_bootloader_name("bostrom"));
    }

    #[test]
    fn no_cybernode_in_product_rpc() {
        assert!(!Network::DEFAULT.rpc().contains("cybernode.ai"));
        assert!(!Network::DEFAULT.rpc().contains("space-pussy"));
    }
}
