//! The phase-1 root chain (specs/chain.md): a public proof-of-work chain on
//! the soft3 harness — hemera as work, bbg as state, mudra NSIG1 as
//! signature, tade as framing.
//!
//! Module map follows chain.md §9.1. Consensus modules are pure functions
//! over bytes; [`api`] is the surface the network, miner and HTTP layers
//! code against; [`mine`] is the reference miner (`soft3 mine`).

pub mod api;
pub mod apply;
pub mod bbg_state;
pub mod daa;
pub mod genesis;
pub mod header;
pub mod mine;
pub mod params;
pub mod pow;
pub mod select;
pub mod sigs;
pub mod state;
pub mod store;
pub mod tree;
pub mod tx;
pub mod u256;

pub use header::{BlockId, Header, HeaderError};
pub use tx::{Tx, TxBody};
pub use u256::U256;
