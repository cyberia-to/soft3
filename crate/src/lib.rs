//! soft3 — the type-2 civilization software stack: the facade.
//!
//! since 0.11 the node is the crate `cybernode` (rung 7 of the ladder,
//! `soft3/proposals/cybernode.md`): the chain kernel, the genesis pipeline, the
//! node, the wallet and the network live there. these re-exports keep
//! `soft3::chain`, `soft3::genesis`, `soft3::network`, `soft3::node` and
//! `soft3::wallet` compiling for one release; new code depends on `cybernode`.
//!
//! Default network: **spacepussy-test** at `https://cyb.ai/spacepussy-test`.

pub use cybernode::{chain, genesis, network, node, wallet};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn identity() -> &'static str {
    "soft3"
}

pub fn default_network() -> network::Network {
    network::Network::DEFAULT
}

pub fn manifesto() -> &'static [&'static str] {
    cybernode::manifesto()
}

#[cfg(feature = "stack")]
pub use cyb;

#[cfg(test)]
mod tests {
    #[test]
    fn default_network_spacepussy_test() {
        assert_eq!(super::default_network().chain_id(), "spacepussy-test");
    }

    #[test]
    fn the_node_is_reachable_through_the_facade() {
        assert_eq!(cybernode::identity(), "cybernode");
        assert_eq!(super::identity(), "soft3");
    }
}
