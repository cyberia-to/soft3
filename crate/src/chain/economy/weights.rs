//! Genesis weights (chain-economy §4.1) and the axon weight `A_pq` (§4.2).
//!
//! ```text
//! B(n)  = native balance of n in the genesis file (0 if none)
//! L(n)  = number of l frames authored by n
//! g(f)  = max(1, floor(B(author(f)) / L(author(f))))      per l frame f
//! G_pq  = Σ g(f) over the l frames f with (p, q)
//! A_pq  = max(0, G_pq + S⁺_pq − S⁻_pq)
//! ```
//!
//! Genesis weights are a pure function of the genesis file, which the chain
//! id commits to; they are computed at load and stored nowhere.

use std::collections::{BTreeMap, HashMap};

use crate::genesis::file::GenesisFile;

use super::super::sigs::token;
use super::super::state::Id;

/// One axon with a weight, `(p, q, w)`.
pub type Axon<W> = (Id, Id, W);

/// The genesis weights of a chain, ascending by `p ‖ q`, one entry per axon.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GenesisWeights {
    pub axons: Vec<Axon<u64>>,
    /// Σ G over every axon
    pub total: u128,
    /// l frames whose weight is the floor of 1 (authors with too little balance)
    pub at_floor: u64,
}

impl GenesisWeights {
    pub fn from_file(file: &GenesisFile) -> GenesisWeights {
        let native = token(&file.params.denom);
        let mut balance: HashMap<Id, u64> = HashMap::new();
        for b in file.balances.iter().filter(|b| b.token == native) {
            *balance.entry(b.owner).or_insert(0) += b.amount;
        }
        let mut frames: HashMap<Id, u64> = HashMap::new();
        for l in &file.links {
            *frames.entry(l.neuron).or_insert(0) += 1;
        }
        let mut g_of: HashMap<Id, u64> = HashMap::with_capacity(frames.len());
        let mut at_floor = 0u64;
        for (author, count) in &frames {
            let b = balance.get(author).copied().unwrap_or(0);
            let g = (b / count).max(1);
            if b / count == 0 {
                at_floor += count;
            }
            g_of.insert(*author, g);
        }
        let mut sums: BTreeMap<(Id, Id), u128> = BTreeMap::new();
        for l in &file.links {
            *sums.entry((l.p, l.q)).or_insert(0) += g_of[&l.neuron] as u128;
        }
        let mut total = 0u128;
        let axons = sums
            .into_iter()
            .map(|((p, q), g)| {
                total += g;
                (p, q, u64::try_from(g).expect("Σ g ≤ S0 + frames < 2^64"))
            })
            .collect();
        GenesisWeights { axons, total, at_floor }
    }

    /// `hemera` over `(p ‖ q ‖ G_pq: u64)` sorted — the digest conformance
    /// §14 pins for a genesis file.
    pub fn digest(&self) -> Id {
        let mut buf = Vec::with_capacity(self.axons.len() * 72);
        for (p, q, g) in &self.axons {
            buf.extend_from_slice(p);
            buf.extend_from_slice(q);
            buf.extend_from_slice(&g.to_le_bytes());
        }
        *hemera::hash(&buf).as_bytes()
    }
}

/// `A_pq = max(0, G_pq + S⁺_pq − S⁻_pq)` over the union of genesis axons
/// and staked axons, ascending by `p ‖ q`, present axons only (`A > 0`).
/// `sides` maps an axon to its `(S⁺, S⁻)`.
pub fn adjacency(genesis: &[Axon<u64>], sides: &BTreeMap<(Id, Id), (u64, u64)>) -> Vec<Axon<u128>> {
    let mut out = Vec::with_capacity(genesis.len() + sides.len());
    let mut s = sides.iter().peekable();
    let a = |g: u64, (pos, neg): (u64, u64)| -> u128 { (g as i128 + pos as i128 - neg as i128).max(0) as u128 };
    let mut push = |p: Id, q: Id, w: u128| {
        if w > 0 {
            out.push((p, q, w));
        }
    };
    for (p, q, g) in genesis {
        while let Some((&(sp, sq), &v)) = s.peek() {
            if (sp, sq) < (*p, *q) {
                push(sp, sq, a(0, v));
                s.next();
            } else {
                break;
            }
        }
        match s.peek() {
            Some((&(sp, sq), &v)) if (sp, sq) == (*p, *q) => {
                push(*p, *q, a(*g, v));
                s.next();
            }
            _ => push(*p, *q, a(*g, (0, 0))),
        }
    }
    for (&(sp, sq), &v) in s {
        push(sp, sq, a(0, v));
    }
    out
}

/// `A_mid = floor((A_prev + A_cur) / 2)` per axon present in either, kept
/// where positive (§5.1). Both inputs ascending by `p ‖ q`.
pub fn midpoint(prev: &[Axon<u128>], cur: &[Axon<u128>]) -> Vec<Axon<u128>> {
    let mut out = Vec::with_capacity(prev.len().max(cur.len()));
    let (mut i, mut j) = (0, 0);
    while i < prev.len() || j < cur.len() {
        let (p, q, a, b) = match (prev.get(i), cur.get(j)) {
            (Some(x), Some(y)) if (x.0, x.1) == (y.0, y.1) => {
                i += 1;
                j += 1;
                (x.0, x.1, x.2, y.2)
            }
            (Some(x), Some(y)) if (x.0, x.1) < (y.0, y.1) => {
                i += 1;
                (x.0, x.1, x.2, 0)
            }
            (Some(x), None) => {
                i += 1;
                (x.0, x.1, x.2, 0)
            }
            (_, Some(y)) => {
                j += 1;
                (y.0, y.1, 0, y.2)
            }
            (None, None) => unreachable!(),
        };
        let m = (a + b) / 2;
        if m > 0 {
            out.push((p, q, m));
        }
    }
    out
}
