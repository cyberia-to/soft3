//! soft3 — stack facade and CLI product crate.
//!
//! Default network after install: **spacepussy-test**
//! (`https://cyb.ai/spacepussy-test`) — the soft3 chaosnet on cybernode.
//!
//! Not the cosmos-sdk chain `space-pussy` on cybernode.

pub mod network;
pub mod node;

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Stack identity.
pub fn identity() -> &'static str {
    "soft3"
}

/// Default network for install / first sync (spacepussy-test).
pub fn default_network() -> network::Network {
    network::Network::DEFAULT
}

/// Manifest lines for the landing story.
pub fn manifesto() -> &'static [&'static str] {
    &[
        "one mind — shared, provable, self-improving",
        "many languages — write, compute, mean",
        "open world — no schema, no gatekeeper",
        "light client validates all of history in roughly ~100 ns",
        "interplanetary consensus — no vote; sync in cyberspace",
        "default network: spacepussy-test (soft3 chaosnet)",
    ]
}

#[cfg(test)]
mod tests {
    #[test]
    fn manifesto_nonempty() {
        assert!(!super::manifesto().is_empty());
    }

    #[test]
    fn default_network_spacepussy_test() {
        assert_eq!(super::default_network().chain_id(), "spacepussy-test");
    }
}

// Re-exports of the published train (product facade).
pub use cyb;
pub use cybergraph;
pub use foculus;
