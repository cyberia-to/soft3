//! §0 constants, the rule-set activation table and the embedded chain ids.
//!
//! Every number in `specs/chain.md` §0 lives here and nowhere else.

use super::u256::U256;

/// Header size in bytes (§1.1).
pub const HEADER_BYTES: usize = 168;
/// Target block interval `T`, seconds (§1.5).
pub const BLOCK_TIME: i64 = 30;
/// ASERT half-life `H`, seconds (§1.5).
pub const HALF_LIFE: i64 = 600;
/// Median-time-past window, headers (§1.4).
pub const MTP_WINDOW: usize = 11;
/// Maximum future drift against the validating node's clock, seconds (§1.4).
pub const MAX_FUTURE_DRIFT: u64 = 120;
/// Compact form of `POW_LIMIT` = 2^236 (§1.3).
pub const POW_LIMIT_BITS: u32 = 0x1e10_0000;
/// Compact form of the genesis target 2^232 (§1.3, §4).
pub const GENESIS_BITS: u32 = 0x1e01_0000;
/// Header + body budget, bytes (§1.6).
pub const MAX_BLOCK_BYTES: usize = 262_144;
/// Transactions per block including the coinbase (§1.6).
pub const MAX_BLOCK_TXS: usize = 1_024;
/// Largest transaction, bytes (§2).
pub const MAX_TX_BYTES: usize = 512;
/// Fee floor, native units per transaction byte (§2.6).
pub const MIN_FEE_PER_BYTE: u64 = 1;
/// Blocks before a coinbase output is spendable (§2.2).
pub const COINBASE_MATURITY: u64 = 100;
/// Blocks after which a block is final for every node that saw it (§5.2).
pub const FINALITY_DEPTH: u64 = 120;
/// Checkpoint cadence, blocks (§5.3).
pub const CHECKPOINT_INTERVAL: u64 = 120;
/// Last height a checkpoint is honored at (§5.3).
pub const CHECKPOINT_SUNSET: u64 = 525_600;
/// Blocks per year at `BLOCK_TIME` (§3.5).
pub const BLOCKS_PER_YEAR: u64 = 1_051_200;
/// `SUBSIDY = floor(S0 / SUBSIDY_DIVISOR)`: 1 % of genesis supply per year (§3.5).
pub const SUBSIDY_DIVISOR: u64 = 105_120_000;
/// Orphan pool size, headers (§5.4).
pub const ORPHAN_POOL_HEADERS: usize = 1_024;
/// Orphan pool age, seconds (§5.4).
pub const ORPHAN_POOL_SECONDS: u64 = 600;
/// Nonces ahead of the current one a mempool admits (§2.5).
pub const NONCE_WINDOW: u64 = 16;
/// Deepest reorganization a node performs (§5.2).
pub const MAX_REORG: u64 = FINALITY_DEPTH - 1;

/// The rule set this implementation speaks (§7.1).
pub const RULE_SET: u16 = 1;

/// Activation table: `(rule_set, height)` in ascending height (§7.1).
/// Rule set 1 activates at height 0; a later release appends rule set 2.
pub const ACTIVATIONS: &[(u16, u64)] = &[(RULE_SET, 0)];

/// The rule set in force at `height`, or `None` when the table ends before
/// it — a node then stops accepting blocks at that height (§7.1).
pub fn rule_set_at(height: u64) -> Option<u16> {
    ACTIVATIONS
        .iter()
        .filter(|(_, h)| *h <= height)
        .map(|(r, _)| *r)
        .max()
}

/// Dialect URN of a genesis file (§4.1).
pub const GENESIS_DIALECT: &str = "urn:cyberia:chain-genesis:1";

/// Domain tags (§1.7, §2.1, §2.8, §5.3, chain-wire §2). Each ends in NUL.
pub const DOMAIN_LEAF: &[u8] = b"soft3/chain/leaf/v1\0";
pub const DOMAIN_NODE: &[u8] = b"soft3/chain/node/v1\0";
pub const DOMAIN_TXID: &[u8] = b"soft3/chain/txid/v1\0";
pub const DOMAIN_SIGN: &[u8] = b"soft3/chain/sign/v1\0";
pub const DOMAIN_LEGACY: &[u8] = b"soft3/chain/legacy/v1\0";
pub const DOMAIN_CHECKPOINT: &[u8] = b"soft3/chain/checkpoint/v1\0";
pub const DOMAIN_TOPIC: &[u8] = b"soft3/chain/topic/v1\0";

/// `POW_LIMIT` as a target.
///
/// Under `cfg(test)` only, a thread-local override lets the simulated
/// miner of the engine tests work at trivial targets (chain-conformance
/// §9 "a private test genesis"); release binaries have no such hook.
pub fn pow_limit() -> U256 {
    #[cfg(test)]
    if let Some(limit) = test_pow::LIMIT.with(|l| l.get()) {
        return limit;
    }
    U256::ONE << 236
}

#[cfg(test)]
pub mod test_pow {
    use super::U256;
    use std::cell::Cell;

    thread_local! {
        pub static LIMIT: Cell<Option<U256>> = const { Cell::new(None) };
    }

    /// Raise the limit on this thread; targets up to 2^255 become legal.
    pub fn relax() {
        LIMIT.with(|l| l.set(Some(U256::ONE << 255)));
    }

    /// Canonical bits of 2^255: the test genesis target (one hash in two).
    pub const EASY_BITS: u32 = 0x2100_8000;
}

/// The genesis target 2^232.
pub fn genesis_target() -> U256 {
    U256::ONE << 232
}

/// Subsidy per block for a genesis supply (§3.5). No floats, floor division.
pub fn subsidy(s0: u64) -> u64 {
    s0 / SUBSIDY_DIVISOR
}

/// An embedded chain: name, hrp, chain id (= genesis file particle) and the
/// id of block 0. Zero ids mean "not yet pinned": the genesis pipeline
/// (specs/genesis.md, cyber/launch rows 14/20) publishes them and a release
/// carries them here; until then `--genesis` is mandatory for that chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmbeddedChain {
    pub name: &'static str,
    pub hrp: &'static str,
    pub chain_id: [u8; 32],
    pub genesis_block: [u8; 32],
    pub genesis_time: u64,
}

/// The two chains one binary serves (§4.4).
pub const CHAINS: &[EmbeddedChain] = &[
    EmbeddedChain {
        name: "pussy",
        hrp: "pussy",
        chain_id: [0; 32],
        genesis_block: [0; 32],
        genesis_time: 1_793_885_820,
    },
    EmbeddedChain {
        name: "bostrom",
        hrp: "bostrom",
        chain_id: [0; 32],
        genesis_block: [0; 32],
        genesis_time: 1_793_884_962,
    },
];

/// Look up an embedded chain by name.
pub fn embedded(name: &str) -> Option<&'static EmbeddedChain> {
    CHAINS.iter().find(|c| c.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subsidy_is_one_percent_per_year_floored() {
        assert_eq!(subsidy(105_120_000), 1);
        assert_eq!(subsidy(105_119_999), 0);
        assert_eq!(subsidy(1_152_075_800_167_936_256), 10_959_625_191);
        assert_eq!(BLOCKS_PER_YEAR * 100, SUBSIDY_DIVISOR);
    }

    #[test]
    fn rule_set_table_starts_at_genesis() {
        assert_eq!(rule_set_at(0), Some(1));
        assert_eq!(rule_set_at(u64::MAX), Some(1));
        assert_eq!(MAX_REORG, 119);
    }

    #[test]
    fn embedded_chains_have_the_launch_times() {
        assert_eq!(embedded("pussy").unwrap().genesis_time, 1_793_885_820);
        assert_eq!(embedded("bostrom").unwrap().genesis_time, 1_793_884_962);
        assert!(embedded("spacepussy-test").is_none());
    }
}
