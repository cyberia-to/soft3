//! The phase-1 root chain (specs/chain.md): a public proof-of-work chain on
//! the soft3 harness — hemera as work, bbg as state, mudra NSIG1 as
//! signature, tade as framing.
//!
//! Module map follows chain.md §9.1. Consensus modules are pure functions
//! over bytes; [`api`] is the surface the network, miner and HTTP layers
//! code against; [`mine`] is the reference miner (`soft3 mine`); [`net`] is
//! the wire (specs/chain-wire.md), running on the kernel through
//! [`net::kernel::KernelChain`].

pub mod api;
pub mod apply;
pub mod bbg_state;
pub mod conformance;
pub mod corpus;
pub mod daa;
pub mod economy;
pub mod economy_api;
pub mod engine;
pub mod engine_api;
pub mod engine_tx;
pub mod genesis;
pub mod header;
pub mod mine;
pub mod net;
pub mod params;
pub mod pool;
pub mod pow;
pub mod select;
pub mod sigs;
pub mod state;
pub mod store;
pub mod tree;
pub mod tx;
pub mod u256;

#[cfg(test)]
mod tests;

pub use api::{Block, BlockVerdict, ChainApi, ChainError, Event, HeaderVerdict, Template, Tip, TxInfo};
pub use engine::Chain;
pub use header::{BlockId, Header, HeaderError};
pub use tx::{Body, Tx};
pub use u256::U256;
