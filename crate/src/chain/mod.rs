//! The phase-1 root chain (specs/chain.md): pure PoW on the soft3 harness.
//!
//! [`mine`] is the reference miner (`soft3 mine`). The kernel — header,
//! pow, daa, tx, state, store, select, mempool, wire, api — lands from
//! `feat/chain-kernel`; until then `mine::local` carries the few consensus
//! helpers the miner needs, pinned to chain-conformance.md.

pub mod mine;
