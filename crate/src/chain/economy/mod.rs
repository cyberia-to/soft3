//! The focus economy of the root chain (specs/chain-economy.md): staked
//! cyberlinks with valence, epochs of a fixed number of blocks, φ* of the
//! staked graph at every boundary, settlement of Δφ⁺ to the stake that
//! caused it, and the subsidy split between miners and the focus pool.
//!
//! Module map (chain-economy §13):
//!
//! - this file: positions, identities (§3.1), the emission split (§6.1),
//!   the mark decay (§5.3) — pure integer functions;
//! - [`wide`]: the 512-bit integers attribution needs (§5.2–§5.5);
//! - [`weights`]: genesis weights (§4.1) and the axon weight `A_pq` (§4.2);
//! - [`focus`]: the canonical focus graph (§4.3) and the φ*/gradient engine
//!   seam (§4.4, §5.1) with the integer reference engine;
//! - [`settle`]: snapshots (§2.4), the three solves and attribution
//!   (§5.1–§5.4), and the settlement step of a block (§5.5–§5.6);
//! - [`run`]: the off-block-path settlement worker and its caches.
//!
//! Integers and fixed point only; nothing here reads a float or a clock.

pub mod focus;
pub mod run;
pub mod vectors;
pub mod settle;
pub mod weights;
pub mod wide;

#[cfg(test)]
mod tests;

use super::params::{DOMAIN_PAYOUT_LEAF, DOMAIN_PAYOUT_NODE, DOMAIN_PHI, FOCUS_MAX_BPS, HWM_KEEP_DEN, HWM_KEEP_NUM};
use super::state::Id;

pub use crate::wallet::tx::{card_id, focus_key, stake_key};

/// `axon(p, q) = hemera(p ‖ q)`, bbg's `axon_id`: a card's `particle`.
pub fn axon_of(p: &Id, q: &Id) -> Id {
    crate::wallet::tx::axon(p, q)
}

/// A position (chain-economy §3.2): one author's stake on one axon. The
/// stake itself is the balance at `stake_key(card)`, not a field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Card {
    /// payout owner: the account credited with the position's rewards
    pub owner: Id,
    /// the axon-particle `axon(p, q) = hemera(p ‖ q)`
    pub particle: Id,
    /// the neuron that locked the stake; only it may unlock
    pub author: Id,
    /// `v ∈ {−1, 0, +1}`; fixed while the stake is positive
    pub valence: i8,
    /// height of the last stake increase
    pub since: u64,
}

impl Card {
    /// `σ(c)`: +1 for `v ∈ {0, +1}`, −1 for `v = −1` (§5.2).
    pub fn side(&self) -> i8 {
        side_of(self.valence)
    }

    /// The bbg byte form of the valence, `v + 1 ∈ {0, 1, 2}` (§3.2).
    pub fn valence_byte(&self) -> u8 {
        (self.valence + 1) as u8
    }
}

/// `σ(v)`: the side of the edge a valence pushes.
pub fn side_of(valence: i8) -> i8 {
    if valence < 0 { -1 } else { 1 }
}

/// The high-water marks of one axon, `H⁺` and `H⁻` (§5.3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mark {
    pub pos: u64,
    pub neg: u64,
}

impl Mark {
    pub fn side(&self, sigma: i8) -> u64 {
        if sigma < 0 { self.neg } else { self.pos }
    }
}

/// Running economy scalars, journaled with the state: what the coinbase
/// of every block and `/status` read without walking the cards.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Scalars {
    /// `focus_bps(epoch(h) − 1)`: the split in force for the next block (§6.1)
    pub focus_bps: u64,
    /// Σ stake over every card
    pub total_stake: u64,
    /// Σ stake over cards with `v ≠ 0` — `active` of §6.1
    pub active_stake: u64,
    /// the last epoch settled on this chain (0 before the first settlement)
    pub last_settled: u64,
}

impl Scalars {
    pub fn encode(&self) -> [u64; 4] {
        [self.focus_bps, self.total_stake, self.active_stake, self.last_settled]
    }

    pub fn decode(v: [u64; 4]) -> Scalars {
        Scalars { focus_bps: v[0], total_stake: v[1], active_stake: v[2], last_settled: v[3] }
    }
}

fn tagged(tag: &[u8], parts: &[&[u8]]) -> Id {
    #[cfg(feature = "opcount")]
    {
        let mut all: Vec<&[u8]> = vec![tag];
        all.extend_from_slice(parts);
        super::opcount::hash(super::opcount::Purpose::Economy, &all);
    }
    let mut buf = Vec::with_capacity(tag.len() + parts.iter().map(|p| p.len()).sum::<usize>());
    buf.extend_from_slice(tag);
    for p in parts {
        buf.extend_from_slice(p);
    }
    *hemera::hash(&buf).as_bytes()
}

/// `hemera(D_PAYOUT_LEAF ‖ card ‖ owner ‖ amount: u64)` (§5.6).
pub fn payout_leaf(card: &Id, owner: &Id, amount: u64) -> Id {
    tagged(DOMAIN_PAYOUT_LEAF, &[card, owner, &amount.to_le_bytes()])
}

/// `hemera(D_PAYOUT_NODE ‖ L ‖ R)` (§5.6).
pub fn payout_node(l: &Id, r: &Id) -> Id {
    tagged(DOMAIN_PAYOUT_NODE, &[l, r])
}

/// Merkle root over payout leaves in the given order (cards ascending,
/// amounts > 0), odd node promoted unchanged; 32 zero bytes when empty.
pub fn payouts_root(leaves: &[Id]) -> Id {
    if leaves.is_empty() {
        return [0; 32];
    }
    let mut level = leaves.to_vec();
    while level.len() > 1 {
        level = level
            .chunks(2)
            .map(|pair| match pair {
                [l, r] => payout_node(l, r),
                [one] => *one,
                _ => unreachable!(),
            })
            .collect();
    }
    level[0]
}

/// `hemera(D_PHI ‖ n: u32 ‖ φ[0]: u64 ‖ … ‖ φ[n−1]: u64)` (§5.6), particles
/// ascending, tru's raw fixed point.
pub fn phi_digest(phi: &[u64]) -> Id {
    let mut buf = Vec::with_capacity(DOMAIN_PHI.len() + 4 + 8 * phi.len());
    buf.extend_from_slice(DOMAIN_PHI);
    buf.extend_from_slice(&(phi.len() as u32).to_le_bytes());
    for v in phi {
        buf.extend_from_slice(&v.to_le_bytes());
    }
    #[cfg(feature = "opcount")]
    super::opcount::hash(super::opcount::Purpose::Economy, &[&buf]);
    *hemera::hash(&buf).as_bytes()
}

/// Floor integer square root of a u128.
pub fn isqrt(n: u128) -> u128 {
    if n < 2 {
        return n;
    }
    // Newton from a power of two at or above the root: monotone decreasing
    let mut x = 1u128 << (128 - n.leading_zeros()).div_ceil(2);
    loop {
        let y = (x + n / x) / 2;
        if y >= x {
            return x;
        }
        x = y;
    }
}

/// `focus_bps(E) = min(FOCUS_MAX_BPS, isqrt(floor(active · 10^8 / supply)))`
/// (§6.1), u128 inside. `supply = S0 + boundary(E) · SUBSIDY`.
pub fn focus_bps(active: u64, supply: u64) -> u64 {
    if supply == 0 {
        return 0;
    }
    let theta = (active as u128 * 100_000_000) / supply as u128;
    (isqrt(theta) as u64).min(FOCUS_MAX_BPS)
}

/// `cut(h) = floor(SUBSIDY · focus_bps / 10,000)`: the focus pool's share
/// of one block's subsidy (§6.1); the miner keeps `SUBSIDY − cut`.
pub fn cut(subsidy: u64, bps: u64) -> u64 {
    ((subsidy as u128 * bps as u128) / 10_000) as u64
}

/// One epoch of the high-water mark (§5.3): with `M = max(H, S)`,
/// `H' = S + floor((M − S) · 65,410 / 65,536)`.
pub fn decay_mark(h: u64, s: u64) -> u64 {
    let m = h.max(s);
    s + (((m - s) as u128 * HWM_KEEP_NUM as u128) / HWM_KEEP_DEN as u128) as u64
}
