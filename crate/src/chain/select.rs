//! Fork choice (§5.1), the reorg limit (§5.2), foundation checkpoints with
//! a sunset (§5.3) and the sync locator (chain-wire §3). Pure functions and
//! one small table; the engine owns the store.

use std::collections::BTreeMap;

use super::header::BlockId;
use super::params::{CHECKPOINT_INTERVAL, CHECKPOINT_SUNSET, FINALITY_DEPTH};
use super::sigs::{checkpoint_statement, neuron_of};
use super::tx::SIG_BYTES;
use super::u256::U256;

/// Heaviest chain: strictly more work wins; ties keep the first seen.
pub fn heavier(candidate: U256, tip: U256) -> bool {
    candidate > tip
}

/// A reorg from `tip_height` to a fork point at `fork_height` is allowed
/// unless the fork point is at or below `tip − FINALITY_DEPTH` (§5.2).
pub fn reorg_allowed(tip_height: u64, fork_height: u64) -> bool {
    fork_height.saturating_add(FINALITY_DEPTH) > tip_height
}

/// Heights of a get-headers locator for a main chain of `tip_height`: the
/// last 10, then doubling spacing back to genesis (chain-wire §3).
pub fn locator_heights(tip_height: u64) -> Vec<u64> {
    let mut out = Vec::new();
    let mut h = tip_height;
    for _ in 0..10 {
        out.push(h);
        if h == 0 {
            return out;
        }
        h -= 1;
    }
    let mut step = 2u64;
    loop {
        if h < step {
            break;
        }
        h -= step;
        out.push(h);
        if h == 0 {
            return out;
        }
        step = step.saturating_mul(2);
    }
    out.push(0);
    out
}

/// `height: u64 ‖ block_id: [32] ‖ sig: NSIG1[102]` (§5.3, chain-wire §2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checkpoint {
    pub height: u64,
    pub block_id: BlockId,
    pub sig: Vec<u8>,
}

pub const CHECKPOINT_BYTES: usize = 8 + 32 + SIG_BYTES;

impl Checkpoint {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(CHECKPOINT_BYTES);
        out.extend_from_slice(&self.height.to_le_bytes());
        out.extend_from_slice(&self.block_id);
        out.extend_from_slice(&self.sig);
        out
    }

    pub fn decode(b: &[u8]) -> Option<Checkpoint> {
        if b.len() != CHECKPOINT_BYTES {
            return None;
        }
        Some(Checkpoint {
            height: u64::from_le_bytes(b[..8].try_into().ok()?),
            block_id: b[8..40].try_into().ok()?,
            sig: b[40..].to_vec(),
        })
    }

    /// Sign with the foundation key (the signer box only).
    pub fn sign(chain: &[u8; 32], height: u64, block_id: BlockId, key: &mudra::SigningKey) -> Result<Checkpoint, mudra::Error> {
        let pubkey = mudra::cosmos::compressed(key.verifying_key());
        let sig = mudra::neuron::sign(key, neuron_of(&pubkey), checkpoint_statement(chain, height, &block_id))?;
        Ok(Checkpoint { height, block_id, sig })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckpointError {
    Disabled,
    Signature,
    Interval(u64),
    Sunset(u64),
    Conflict { height: u64, held: BlockId },
}

impl std::fmt::Display for CheckpointError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckpointError::Disabled => write!(f, "checkpoints are disabled"),
            CheckpointError::Signature => write!(f, "checkpoint signature does not verify under the foundation key"),
            CheckpointError::Interval(h) => write!(f, "height {h} is not a multiple of {CHECKPOINT_INTERVAL}"),
            CheckpointError::Sunset(h) => write!(f, "height {h} is past the sunset {CHECKPOINT_SUNSET}"),
            CheckpointError::Conflict { height, held } => write!(f, "a different checkpoint is held at {height}: {}", hex(held)),
        }
    }
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// The checkpoints a node holds: veto-only, bounded in power and time.
#[derive(Clone, Debug, Default)]
pub struct Checkpoints {
    key: Option<[u8; 33]>,
    subject: [u8; 32],
    held: BTreeMap<u64, BlockId>,
}

impl Checkpoints {
    /// `key` all-zero (§4.1) or `enabled = false` (`--no-checkpoints`)
    /// disables them.
    pub fn new(key: [u8; 33], enabled: bool) -> Self {
        let on = enabled && key != [0; 33];
        Checkpoints { key: on.then_some(key), subject: if on { neuron_of(&key) } else { [0; 32] }, held: BTreeMap::new() }
    }

    pub fn enabled(&self) -> bool {
        self.key.is_some()
    }

    pub fn held(&self) -> impl Iterator<Item = (&u64, &BlockId)> {
        self.held.iter()
    }

    pub fn get(&self, height: u64) -> Option<BlockId> {
        self.held.get(&height).copied()
    }

    /// The highest checkpointed height, below which no reorg goes.
    pub fn floor(&self) -> Option<u64> {
        self.held.keys().next_back().copied()
    }

    /// Verify and hold a checkpoint. `Ok(true)` when it is new, `Ok(false)`
    /// when already held; a checkpoint past the sunset is ignored as an
    /// error the caller may log without scoring (§5.3).
    pub fn accept(&mut self, chain: &[u8; 32], cp: &Checkpoint) -> Result<bool, CheckpointError> {
        if self.key.is_none() {
            return Err(CheckpointError::Disabled);
        }
        if cp.height > CHECKPOINT_SUNSET {
            return Err(CheckpointError::Sunset(cp.height));
        }
        if cp.height == 0 || cp.height % CHECKPOINT_INTERVAL != 0 {
            return Err(CheckpointError::Interval(cp.height));
        }
        let st = checkpoint_statement(chain, cp.height, &cp.block_id);
        if !mudra::neuron::verify_statement(self.subject, st, &cp.sig) {
            return Err(CheckpointError::Signature);
        }
        match self.held.get(&cp.height) {
            Some(held) if *held == cp.block_id => Ok(false),
            Some(held) => Err(CheckpointError::Conflict { height: cp.height, held: *held }),
            None => {
                self.held.insert(cp.height, cp.block_id);
                Ok(true)
            }
        }
    }

    /// The first held checkpoint a chain violates, given its ancestor at
    /// each height (`None` above the chain's tip is not a violation).
    pub fn violation(&self, ancestor_at: impl Fn(u64) -> Option<BlockId>) -> Option<(u64, BlockId)> {
        for (h, id) in &self.held {
            if let Some(actual) = ancestor_at(*h) {
                if actual != *id {
                    return Some((*h, *id));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locator_of_height_1000_is_the_frozen_list() {
        assert_eq!(
            locator_heights(1000),
            vec![1000, 999, 998, 997, 996, 995, 994, 993, 992, 991, 989, 985, 977, 961, 929, 865, 737, 481, 0]
        );
        assert_eq!(locator_heights(0), vec![0]);
        assert_eq!(locator_heights(3), vec![3, 2, 1, 0]);
        assert_eq!(locator_heights(12), vec![12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 1, 0]);
    }

    #[test]
    fn reorg_limit_is_119() {
        assert!(reorg_allowed(1000, 1000 - 119));
        assert!(!reorg_allowed(1000, 1000 - 120));
        assert!(reorg_allowed(5, 0));
        assert!(heavier(U256::from_u64(2), U256::from_u64(1)));
        assert!(!heavier(U256::from_u64(1), U256::from_u64(1)));
    }

    #[test]
    fn checkpoints_verify_hold_and_veto() {
        let chain = [0x44; 32];
        let key = mudra::SigningKey::from_bytes((&[7u8; 32]).into()).unwrap();
        let other = mudra::SigningKey::from_bytes((&[8u8; 32]).into()).unwrap();
        let pubkey = mudra::cosmos::compressed(key.verifying_key());
        let mut cps = Checkpoints::new(pubkey, true);
        assert!(cps.enabled());
        let cp = Checkpoint::sign(&chain, 120, [1; 32], &key).unwrap();
        assert_eq!(cp.encode().len(), CHECKPOINT_BYTES);
        assert_eq!(Checkpoint::decode(&cp.encode()), Some(cp.clone()));
        assert_eq!(cps.accept(&chain, &cp), Ok(true));
        assert_eq!(cps.accept(&chain, &cp), Ok(false));
        let forged = Checkpoint::sign(&chain, 240, [2; 32], &other).unwrap();
        assert_eq!(cps.accept(&chain, &forged), Err(CheckpointError::Signature));
        let off = Checkpoint::sign(&chain, 121, [2; 32], &key).unwrap();
        assert_eq!(cps.accept(&chain, &off), Err(CheckpointError::Interval(121)));
        let late = Checkpoint::sign(&chain, CHECKPOINT_SUNSET + 120, [2; 32], &key).unwrap();
        assert_eq!(cps.accept(&chain, &late), Err(CheckpointError::Sunset(CHECKPOINT_SUNSET + 120)));
        let conflict = Checkpoint::sign(&chain, 120, [9; 32], &key).unwrap();
        assert_eq!(cps.accept(&chain, &conflict), Err(CheckpointError::Conflict { height: 120, held: [1; 32] }));
        assert_eq!(cps.floor(), Some(120));
        assert_eq!(cps.violation(|h| (h == 120).then_some([1; 32])), None);
        assert_eq!(cps.violation(|h| (h == 120).then_some([3; 32])), Some((120, [1; 32])));
        assert_eq!(cps.violation(|_| None), None, "a chain below the checkpoint is not in violation");
        let wrong_chain = Checkpoint::sign(&[0x45; 32], 240, [2; 32], &key).unwrap();
        assert_eq!(cps.accept(&chain, &wrong_chain), Err(CheckpointError::Signature));
        let mut off_switch = Checkpoints::new(pubkey, false);
        assert_eq!(off_switch.accept(&chain, &cp), Err(CheckpointError::Disabled));
        let mut no_key = Checkpoints::new([0; 33], true);
        assert_eq!(no_key.accept(&chain, &cp), Err(CheckpointError::Disabled));
    }
}
