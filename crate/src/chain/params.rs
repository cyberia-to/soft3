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

// ── the focus economy (chain-economy §0) ───────────────────────────────

/// `EPOCH_LENGTH` (L): blocks per epoch; the boundary of epoch E is height
/// `E · L` (chain-economy §2.1–§2.2).
pub const EPOCH_LENGTH: u64 = 240;
/// `SETTLE_DELAY` (D) = `FINALITY_DEPTH`: settlement of epoch E is a step of
/// the block at `boundary(E) + D` (chain-economy §2.5).
pub const SETTLE_DELAY: u64 = FINALITY_DEPTH;
/// `MIN_LOCK`: blocks a position's stake stays locked after its last
/// increase (chain-economy §3.4). Must be at least `L + D`.
pub const MIN_LOCK: u64 = 20_160;
/// `FOCUS_MAX_BPS`: the focus pool's cap on the subsidy, basis points (§6.1).
pub const FOCUS_MAX_BPS: u64 = 5_000;
/// `POOL_RELEASE`: an epoch's budget is `pool / POOL_RELEASE` (§5.5).
pub const POOL_RELEASE: u64 = 4;
/// `R_MAX_PPM`: a position earns at most this many ppm of its added stake
/// per settlement (§5.5).
pub const R_MAX_PPM: u64 = 10_000;
/// `HWM_KEEP` = 65,410 / 65,536 per epoch: the high-water mark decay
/// (half-life 360 epochs, §5.3).
pub const HWM_KEEP_NUM: u64 = 65_410;
pub const HWM_KEEP_DEN: u64 = 65_536;
/// `PHI_STEPS`: tri-kernel steps per cold solve (§4.4); tru's `EpochParams`
/// fixes the value at the phase1.toml pin.
pub const PHI_STEPS: usize = 48;
/// `ATTR_NODES`: Gauss–Legendre nodes of the attribution path gradient
/// (§5.1, amended 2026-10-17 from the one-node midpoint; chosen against
/// exact Shapley in tru audit/epoch-settlement-2026-10.md §6). The ground
/// weight of §4.4 (`GROUND` = in-weight / 4) is tru's `EpochParams` default.
pub const ATTR_NODES: usize = 4;
/// `SETTLE_BUDGET`: seconds of reference settlement compute (§7.2).
pub const SETTLE_BUDGET_SECS: u64 = 900;
/// `RHO_LAUNCH`, `KARMA_LAUNCH`: the surprise weight and karma of every
/// active position and neuron at genesis (§5.4, §3.7). Both are 1, so they
/// multiply nothing; named here so the reports rule set has one place to
/// replace them.
pub const RHO_LAUNCH: u64 = 1;
pub const KARMA_LAUNCH: u64 = 1;

/// Economy domain tags (chain-economy §3.1), each ending in NUL.
pub const DOMAIN_CARD: &[u8] = b"soft3/chain/card/v1\0";
pub const DOMAIN_STAKE: &[u8] = b"soft3/chain/stake/v1\0";
pub const DOMAIN_FOCUS: &[u8] = b"soft3/chain/focus/v1\0";
pub const DOMAIN_PHI: &[u8] = b"soft3/chain/phi/v1\0";
pub const DOMAIN_EPOCH: &[u8] = b"soft3/chain/epoch/v1\0";
pub const DOMAIN_PAYOUT_LEAF: &[u8] = b"soft3/chain/payout-leaf/v1\0";
pub const DOMAIN_PAYOUT_NODE: &[u8] = b"soft3/chain/payout-node/v1\0";

/// The epoch clock of one chain. The launch chains run [`EpochClock::SPEC`]
/// and nothing else; a private test or dev net may shorten the clock (the
/// way a test genesis relaxes the PoW limit) so a settlement happens inside
/// a test run. The rules are the same functions of `(L, D, MIN_LOCK)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochClock {
    pub length: u64,
    pub delay: u64,
    pub min_lock: u64,
}

impl EpochClock {
    pub const SPEC: EpochClock = EpochClock { length: EPOCH_LENGTH, delay: SETTLE_DELAY, min_lock: MIN_LOCK };

    /// `L ≥ D + 1` (one settlement in flight) and `MIN_LOCK ≥ L + D` (stake
    /// added in an epoch is still locked at its settlement), §2.2, §3.4.
    pub fn valid(&self) -> bool {
        self.delay >= 1 && self.length > self.delay && self.min_lock >= self.length + self.delay
    }

    /// `epoch(h) = ceil(h / L)`; height 0 is epoch 0 (§2.1).
    pub fn epoch(&self, h: u64) -> u64 {
        h.div_ceil(self.length)
    }

    /// `boundary(E) = E · L`, the last block of epoch E.
    pub fn boundary(&self, e: u64) -> u64 {
        e * self.length
    }

    pub fn is_boundary(&self, h: u64) -> bool {
        h % self.length == 0
    }

    /// `settle(E) = boundary(E) + D`.
    pub fn settle(&self, e: u64) -> u64 {
        self.boundary(e) + self.delay
    }

    /// The epoch E ≥ 1 whose settlement is a step of the block at `h`.
    pub fn settles_at(&self, h: u64) -> Option<u64> {
        (h > self.delay && (h - self.delay) % self.length == 0).then(|| (h - self.delay) / self.length).filter(|e| *e >= 1)
    }

    /// A dev or test clock from `SOFT3_DEV_EPOCH=L:D:MIN_LOCK`; `None` when
    /// unset or invalid. Never consulted for a launch chain.
    pub fn from_env() -> Option<EpochClock> {
        let v = std::env::var("SOFT3_DEV_EPOCH").ok()?;
        let mut it = v.split(':').map(|x| x.trim().parse::<u64>());
        let (Some(Ok(length)), Some(Ok(delay)), Some(Ok(min_lock))) = (it.next(), it.next(), it.next()) else { return None };
        let c = EpochClock { length, delay, min_lock };
        c.valid().then_some(c)
    }
}

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
    #[cfg(feature = "sim")]
    if sim_pow::relaxed() {
        return U256::ONE << 255;
    }
    U256::ONE << 236
}

/// The out-of-tree twin of [`test_pow`], behind the `sim` feature: the
/// network simulator (cyberia-to/simulation) runs many in-process nodes on
/// a private test genesis and needs the same trivial targets on every
/// thread (settlement workers included), so the switch is process-wide.
/// Without the feature the module does not exist and `POW_LIMIT` is 2^236;
/// a node binary never enables it. The launch genesis files keep their
/// 2^232 target either way; the simulator runs them in a process that
/// never calls [`sim_pow::relax`].
#[cfg(feature = "sim")]
pub mod sim_pow {
    use std::sync::atomic::{AtomicBool, Ordering};

    static RELAXED: AtomicBool = AtomicBool::new(false);

    /// Raise the limit for the whole process; targets up to 2^255 become legal.
    pub fn relax() {
        RELAXED.store(true, Ordering::SeqCst);
    }

    pub fn relaxed() -> bool {
        RELAXED.load(Ordering::SeqCst)
    }

    /// Canonical bits of 2^255 (one hash in two), as `test_pow::EASY_BITS`.
    pub const EASY_BITS: u32 = 0x2100_8000;
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
/// id of block 0. Non-zero ids are pinned: the node refuses a genesis file
/// whose particle differs, and a kernel whose block 0 differs. pussy is
/// pinned to the values blessed 2026-10-17 in `conformance/chain-genesis.toml`
/// (and `chain.snap`). Zero ids mean "not yet pinned": bostrom waits until
/// pussy's genesis is covered end to end; until pinned, `--genesis` is
/// mandatory for that chain and any file naming it is accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmbeddedChain {
    pub name: &'static str,
    pub hrp: &'static str,
    pub chain_id: [u8; 32],
    pub genesis_block: [u8; 32],
    pub genesis_time: u64,
}

impl EmbeddedChain {
    /// True once the chain id is pinned (non-zero).
    pub fn pinned(&self) -> bool {
        self.chain_id != [0; 32]
    }
}

/// Decode 64 hex digits at compile time; a typo in a pin fails the build.
const fn h32(s: &str) -> [u8; 32] {
    const fn nibble(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            _ => panic!("pin is not lowercase hex"),
        }
    }
    let b = s.as_bytes();
    assert!(b.len() == 64, "pin is not 64 hex digits");
    let mut out = [0u8; 32];
    let mut i = 0;
    while i < 32 {
        out[i] = (nibble(b[2 * i]) << 4) | nibble(b[2 * i + 1]);
        i += 1;
    }
    out
}

/// The two chains one binary serves (§4.4).
pub const CHAINS: &[EmbeddedChain] = &[
    EmbeddedChain {
        name: "pussy",
        hrp: "pussy",
        chain_id: h32("bae85f40d9f508664b5214107551be95d3f188d9585a899178c23e80886720e5"),
        genesis_block: h32("73208ee2d43fbb0420119ceff23eab79e9d96921050e2a2137d0705c461852d4"),
        genesis_time: 1_793_885_820,
    },
    EmbeddedChain {
        // unpinned on purpose: bostrom waits for pussy's genesis coverage
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

    fn unhex(s: &str) -> [u8; 32] {
        let b: Vec<u8> = (0..64).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect();
        b.try_into().unwrap()
    }

    #[test]
    fn pussy_pins_match_the_blessed_files_and_bostrom_is_unpinned() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../conformance");
        let pussy = embedded("pussy").unwrap();
        assert!(pussy.pinned());
        assert!(!embedded("bostrom").unwrap().pinned());

        let toml_text = std::fs::read_to_string(format!("{root}/chain-genesis.toml")).unwrap();
        let table: toml::Table = toml_text.parse().unwrap();
        let sect = table["pussy"].as_table().unwrap();
        assert_eq!(pussy.chain_id, unhex(sect["particle"].as_str().unwrap()));
        assert_eq!(pussy.genesis_block, unhex(sect["block0_id"].as_str().unwrap()));

        let snap = std::fs::read_to_string(format!("{root}/chain.snap")).unwrap();
        let entries = crate::chain::conformance::parse(&snap);
        let find = |scenario: &str| entries.iter().find(|e| e.key == "chain::genesis@v1" && e.scenario == scenario).unwrap().value;
        assert_eq!(pussy.chain_id, find("pussy particle"));
        assert_eq!(pussy.genesis_block, find("pussy block0_id"));
    }

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
    fn epoch_clock_matches_chain_economy_section_10() {
        let c = EpochClock::SPEC;
        assert!(c.valid());
        assert_eq!((c.epoch(0), c.epoch(1), c.epoch(240), c.epoch(241)), (0, 1, 1, 2));
        assert_eq!(c.settle(1), 360);
        assert_eq!(c.settles_at(360), Some(1));
        assert_eq!(c.settles_at(600), Some(2));
        assert_eq!((c.settles_at(120), c.settles_at(359), c.settles_at(240)), (None, None, None));
        assert!(MIN_LOCK >= EPOCH_LENGTH + SETTLE_DELAY);
        assert!(!EpochClock { length: 10, delay: 10, min_lock: 20 }.valid());
        assert!(!EpochClock { length: 10, delay: 5, min_lock: 14 }.valid());
        assert!(EpochClock { length: 10, delay: 5, min_lock: 15 }.valid());
    }

    #[test]
    fn embedded_chains_have_the_launch_times() {
        assert_eq!(embedded("pussy").unwrap().genesis_time, 1_793_885_820);
        assert_eq!(embedded("bostrom").unwrap().genesis_time, 1_793_884_962);
        assert!(embedded("spacepussy-test").is_none());
    }
}
