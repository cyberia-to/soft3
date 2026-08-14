//! soft3 — the type-2 civilization software stack.
//!
//! - [[network]] — spacepussy-test product network (soft3 chaosnet)
//! - [[node]] — real soft3-node over cybergraph + bbg
//! - CLI: `soft3 sync`, `soft3 node`, `soft3 network`
//!
//! Default network: **spacepussy-test** at `https://cyb.ai/spacepussy-test`
//! (cybernode). Not the cosmos-sdk chain `space-pussy`.

pub mod network;
pub mod node;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn identity() -> &'static str {
    "soft3"
}

pub fn default_network() -> network::Network {
    network::Network::DEFAULT
}

pub fn manifesto() -> &'static [&'static str] {
    &[
        "one mind — shared, provable, self-improving",
        "many languages — write, compute, mean",
        "open world — no schema, no gatekeeper",
        "engine: cybergraph + bbg on spacepussy-test",
        "default network: spacepussy-test (soft3 chaosnet)",
    ]
}

#[cfg(feature = "stack")]
pub use cyb;

#[cfg(test)]
mod tests {
    #[test]
    fn default_network_spacepussy_test() {
        assert_eq!(super::default_network().chain_id(), "spacepussy-test");
    }
}
