//! Genesis on the consensus side (§4): the file is `soft3::genesis`'s tade
//! stream (`GenesisFile`, `specs/genesis.md`), applied through
//! `soft3::genesis::state::load`; this module adds what the chain needs —
//! the chain id (= file particle), `S0`, the ASERT anchor, the state
//! backend with its conservation sums, and block 0 (§4.3).

use crate::genesis::file::GenesisFile;
use crate::genesis::state::load;

use super::bbg_state::BbgBacked;
use super::daa::Anchor;
use super::header::{Header, decode_bits, hash32};
use super::params::subsidy;
use super::sigs::token;
use super::state::Id;

pub use crate::genesis::file::{Balance, Link, Params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Genesis {
    pub file: GenesisFile,
    /// the file bytes; `hemera(bytes)` is the chain id
    pub bytes: Vec<u8>,
    pub chain_id: Id,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenesisError {
    /// the tade stream is not a canonical genesis (`soft3::genesis::file`)
    File(String),
    /// `params.bits` is not a canonical target within limits
    Bits,
    /// the native supply does not fit u64
    Supply,
    /// `params.subsidy != floor(S0 / 105,120,000)`
    Subsidy { written: u64, derived: u64 },
}

impl std::fmt::Display for GenesisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenesisError::File(e) => write!(f, "genesis file: {e}"),
            GenesisError::Bits => write!(f, "genesis bits are not a canonical target"),
            GenesisError::Supply => write!(f, "native supply exceeds u64"),
            GenesisError::Subsidy { written, derived } => write!(f, "subsidy {written} written, {derived} derived from S0"),
        }
    }
}

impl Genesis {
    /// Decode and check a genesis file; the chain id is its particle.
    pub fn decode(bytes: &[u8]) -> Result<Genesis, GenesisError> {
        let file = GenesisFile::decode(bytes).map_err(|e| GenesisError::File(e.to_string()))?;
        decode_bits(file.params.bits).map_err(|_| GenesisError::Bits)?;
        let s0 = u64::try_from(file.native_supply()).map_err(|_| GenesisError::Supply)?;
        let derived = subsidy(s0);
        if file.params.subsidy != derived {
            return Err(GenesisError::Subsidy { written: file.params.subsidy, derived });
        }
        Ok(Genesis { file, bytes: bytes.to_vec(), chain_id: hash32(bytes) })
    }

    /// Canonical bytes of a file, then decode (test nets, rehearsals).
    pub fn from_file(file: &GenesisFile) -> Result<Genesis, GenesisError> {
        let bytes = file.encode().map_err(|e| GenesisError::File(e.to_string()))?;
        Genesis::decode(&bytes)
    }

    pub fn params(&self) -> &Params {
        &self.file.params
    }

    pub fn native_token(&self) -> Id {
        token(&self.file.params.denom)
    }

    /// `S0`: the genesis supply of the native coin.
    pub fn s0(&self) -> u64 {
        u64::try_from(self.file.native_supply()).expect("checked on decode")
    }

    pub fn subsidy(&self) -> u64 {
        self.file.params.subsidy
    }

    /// The ASERT anchor (§1.5).
    pub fn anchor(&self) -> Anchor {
        Anchor { target: decode_bits(self.file.params.bits).expect("checked on decode"), time: self.file.params.time }
    }

    /// The state after genesis (§4.1), with the per-token sums the
    /// conservation check needs, through the pipeline's own loader.
    pub fn state(&self) -> BbgBacked {
        let bbg = load(&self.file);
        let sums = self.file.balances.iter().map(|b| (b.token, b.amount));
        BbgBacked::from_bbg(bbg, sums)
    }

    /// Block 0 (§4.3): `bits = GENESIS_BITS` (the params field, which the
    /// pipeline writes as `GENESIS_BITS`; a test net may anchor elsewhere).
    pub fn block0(&self, state_root: Id) -> Header {
        Header {
            version: 1,
            height: 0,
            time: self.file.params.time,
            bits: self.file.params.bits,
            nonce: [0; 16],
            prev: [0; 32],
            tx_root: self.chain_id,
            state_root,
            chain: self.chain_id,
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use super::super::params::GENESIS_BITS;
    use super::super::state::StateBackend;

    /// The conformance net of chain-conformance §8: one balance, no links.
    pub(crate) fn conformance_params() -> Params {
        Params {
            version: 1,
            name: "conformance".into(),
            hrp: "pussy".into(),
            denom: "pussy".into(),
            time: 0,
            bits: GENESIS_BITS,
            subsidy: 0,
            checkpoint_key: [0; 33],
            checkpoint_sunset: 0,
            pool: crate::chain::pool::Pool::genesis(),
        }
    }

    pub(crate) fn conformance_genesis() -> Genesis {
        Genesis::from_file(&GenesisFile {
            params: conformance_params(),
            balances: vec![Balance { owner: [0x66; 32], token: token("pussy"), amount: 1_000_000 }],
            links: vec![],
        })
        .unwrap()
    }

    #[test]
    fn conformance_net_applies_through_the_pipeline_loader() {
        let g = conformance_genesis();
        assert_eq!(g.s0(), 1_000_000);
        assert_eq!(g.subsidy(), 0, "1,000,000 / 105,120,000 floors to 0");
        assert_eq!(g.chain_id, hash32(&g.bytes));
        let mut s = g.state();
        assert_eq!(s.balance(&[0x66; 32], &token("pussy")), 1_000_000);
        assert_eq!(s.supply(&token("pussy")), 1_000_000);
        assert_eq!(s.token_sum(&token("pussy")), 1_000_000);
        assert_eq!(s.nonce(&[0x66; 32]), 0);
        assert!(s.has_neuron(&[0x66; 32]));
        assert_eq!(s.height(), 0);
        let b0 = g.block0(s.root());
        assert_eq!(b0.tx_root, g.chain_id);
        assert_eq!(b0.encode(), crate::genesis::file::block0_header(0, &b0.state_root, &g.chain_id));
        assert_eq!(Genesis::decode(&g.bytes).unwrap(), g);
    }

    #[test]
    fn subsidy_and_bits_are_checked() {
        let mut p = conformance_params();
        p.subsidy = 1;
        let file = GenesisFile { params: p.clone(), balances: vec![], links: vec![] };
        assert_eq!(Genesis::from_file(&file), Err(GenesisError::Subsidy { written: 1, derived: 0 }));
        p.subsidy = 0;
        p.bits = 0x1e00_1000;
        let file = GenesisFile { params: p, balances: vec![], links: vec![] };
        assert_eq!(Genesis::from_file(&file), Err(GenesisError::Bits));
        assert!(matches!(Genesis::decode(b"not a genesis"), Err(GenesisError::File(_))));
    }

    #[test]
    fn links_apply_with_zero_amount_and_authors_get_records() {
        let links = vec![
            Link { neuron: [1; 32], p: [2; 32], q: [3; 32], height: 5 },
            Link { neuron: [1; 32], p: [2; 32], q: [3; 32], height: 6 },
        ];
        let g = Genesis::from_file(&GenesisFile {
            params: conformance_params(),
            balances: vec![Balance { owner: [0x66; 32], token: token("pussy"), amount: 7 }],
            links: links.clone(),
        })
        .unwrap();
        assert_eq!(g.file.links, links, "duplicates kept");
        let s = g.state();
        assert!(s.has_neuron(&[1; 32]));
        assert_eq!(s.axon_count(), 1);
        assert_eq!(s.token_sum(&token("pussy")), 7);
    }

    /// chain-conformance §10 against the real pussy file when it is cached
    /// (`soft3 genesis build` writes ~/.cache/cyber/genesis/pussy.tade);
    /// the release gate `stack-chain-genesis` runs the same comparison.
    #[test]
    fn pussy_genesis_matches_the_blessed_vectors_when_cached() {
        let home = std::env::var("HOME").unwrap_or_default();
        let path = std::path::Path::new(&home).join(".cache/cyber/genesis/pussy.tade");
        let Ok(bytes) = std::fs::read(&path) else {
            eprintln!("skipped: {} not cached", path.display());
            return;
        };
        let g = Genesis::decode(&bytes).unwrap();
        let hex = |b: &[u8]| b.iter().map(|x| format!("{x:02x}")).collect::<String>();
        assert_eq!(hex(&g.chain_id), "bae85f40d9f508664b5214107551be95d3f188d9585a899178c23e80886720e5");
        assert_eq!(g.s0(), 1_099_389_810_279_114_036);
        assert_eq!(g.subsidy(), 10_458_426_657);
        assert_eq!(g.params().time, 1_793_885_820);
        let mut s = g.state();
        let root = s.root();
        assert_eq!(hex(&root), "d48fee478584d248123c63e2ee6c56224f159c7dadbc625f4261f6af49130371");
        assert_eq!(hex(&g.block0(root).id()), "73208ee2d43fbb0420119ceff23eab79e9d96921050e2a2137d0705c461852d4");
        for t in s.tokens() {
            assert_eq!(s.token_sum(&t), s.supply(&t));
        }
    }
}
