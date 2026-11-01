//! P8 census hooks of the bbg-backed state (feature `opcount`).
//!
//! bbg cannot be instrumented from here, so what soft3 does to it is
//! counted exactly (`state.*`: every committed-record write, read, and the
//! `root()` calls) and what bbg then hashes is derived from the touched
//! keys and bbg's own source at rev 2d7873c7 (`derived.*`):
//!
//! - a committed record is re-hashed once per root read, if touched:
//!   `leaf_digest` = 2 permutations for the key (8 limbs, chunks of 4) plus
//!   `ceil(vals / 4)` for the value limbs (`rs/src/trie.rs`, `leaf_digest`);
//! - a trie branch is re-hashed once per root read if a touched key sits
//!   below it: `branch_digest` = 2 permutations (`trie.rs`, `branch_digest`;
//!   `Trie::hash`). The dirty branches are the distinct `(bit, key prefix)`
//!   pairs on the root paths of the touched keys, read off bbg's own
//!   `prove` after the root is computed;
//! - an adjacency list is a two-level trie: `AxonIndex::add` hashes the
//!   member (`leaf_digest(axon, [])`, 2 permutations) at every call, the
//!   particle's set trie re-hashes its dirty branches, and the outer leaf
//!   `[degree, set root]` is 2 + 2 permutations (`keyed.rs`, `AxonIndex`);
//! - the root is the 14-leaf fold, 14 permutations (`root.rs`,
//!   `root_from_leaves`), plus the stats leaf, a hemera hash of 14 u64
//!   = 112 bytes (`stats.rs`, `GraphStats::commit`).

use std::collections::{BTreeMap, BTreeSet};

use bbg::keyed::TrieKey;
use bbg::trie::Digest;

use super::BbgBacked;
use crate::chain::opcount::{self, bump, derived, perms};
use crate::chain::state::Id;

#[derive(Default)]
pub struct Touched {
    pub balances: BTreeSet<Id>,
    pub neurons: BTreeSet<Id>,
    pub cards: BTreeSet<Id>,
    pub time: BTreeSet<u64>,
    pub coins: BTreeSet<Id>,
    pub out: BTreeSet<Id>,
    pub inn: BTreeSet<Id>,
    /// `(particle, axon)` of every `AxonIndex::add` call
    pub adds_out: Vec<(Id, Id)>,
    pub adds_in: Vec<(Id, Id)>,
}

pub fn op(name: &str) {
    bump(name, 1);
}

/// What `link` does through bbg: two adjacency adds, an axon id inside
/// bbg's `insert`, and the keys soft3 derives around the call.
pub fn link(s: &mut BbgBacked, p: &Id, q: &Id, token: &Id) {
    let axon = bbg::state::axon_id(p, q);
    // soft3's own call to bbg's `axon_id`, and the two `balance_key`s it
    // derives only to journal before-images
    opcount::hash(opcount::Purpose::Key, &[p, q]);
    opcount::hash(opcount::Purpose::Journal, &[q, token]);
    opcount::hash(opcount::Purpose::Journal, &[p, token]);
    op("state.axons_out.add");
    op("state.axons_in.add");
    s.touched.out.insert(*p);
    s.touched.inn.insert(*q);
    s.touched.adds_out.push((*p, axon));
    s.touched.adds_in.push((*q, axon));
    // inside bbg: `axon_id` again (64 bytes), and two member digests
    derived("bbg_axon_id", perms(64));
    derived("bbg_member_digest", 2 * 2);
}

fn prefix(key: &[u8; 32], bit: u16) -> [u8; 32] {
    let mut out = [0u8; 32];
    let whole = (bit / 8) as usize;
    out[..whole].copy_from_slice(&key[..whole]);
    let rem = bit % 8;
    if rem > 0 {
        out[whole] = key[whole] & (0xffu8 << (8 - rem));
    }
    out
}

/// The branches on the paths of `keys` in one trie, as `(bit, prefix)`.
fn branches(steps: &[(u16, Digest)], key: &[u8; 32], into: &mut BTreeSet<(u16, [u8; 32])>) {
    for (bit, _) in steps {
        into.insert((*bit, prefix(key, *bit)));
    }
}

fn leaf_perms(value_limbs: u64) -> u64 {
    2 + value_limbs.div_ceil(4)
}

/// One keyed dimension: dirty leaves and dirty branches.
fn dim<K: TrieKey>(name: &str, keys: &BTreeSet<K>, present: impl Fn(&K) -> bool, value_limbs: u64, proof: impl Fn(&K) -> bbg::trie::TrieProof) {
    let mut leaves = 0u64;
    let mut removed = 0u64;
    let mut set = BTreeSet::new();
    for k in keys {
        if present(k) {
            leaves += 1;
        } else {
            removed += 1;
        }
        let proof = proof(k);
        branches(&proof.steps, &k.key_bytes(), &mut set);
    }
    bump(&format!("state.{name}.dirty_leaves"), leaves);
    bump(&format!("state.{name}.dirty_removed"), removed);
    bump(&format!("state.{name}.dirty_branches"), set.len() as u64);
    derived(&format!("trie_leaf.{name}"), leaves * leaf_perms(value_limbs));
    derived(&format!("trie_branch.{name}"), 2 * set.len() as u64);
}

/// `root()`: bbg's refresh, then the census of what it just hashed.
pub fn root(s: &mut BbgBacked) -> Id {
    let t = std::mem::take(&mut s.touched);
    let b = &s.bbg;
    // bbg's own count of records awaiting a commitment read, measured
    let pending = [
        ("balances", b.balances.pending(), t.balances.len()),
        ("neurons", b.neurons.pending(), t.neurons.len()),
        ("cards", b.cards.pending(), t.cards.len()),
        ("time", b.time.pending(), t.time.len()),
        ("coins", b.coins.pending(), t.coins.len()),
        ("axons_out", b.axons_out.pending(), t.out.len()),
        ("axons_in", b.axons_in.pending(), t.inn.len()),
    ];
    for (name, bbg_pending, mine) in pending {
        bump(&format!("state.{name}.pending"), bbg_pending as u64);
        if bbg_pending != mine {
            bump("state.pending_mismatch", 1);
        }
    }
    let root = s.bbg.refresh_root();
    let b = &s.bbg;
    bump("state.root", 1);
    derived("root_fold", 14);
    derived("stats_commit", perms(14 * 8));

    // value limbs: u64 → 2; neuron 4 u64 → 8; card 3·8 + 2 + 2 = 28; time value 32 bytes → 8
    dim("balances", &t.balances, |k| b.balances.contains_key(k), 2, |k| b.balances.prove(k));
    dim("neurons", &t.neurons, |k| b.neurons.contains_key(k), 8, |k| b.neurons.prove(k));
    dim("cards", &t.cards, |k| b.cards.contains_key(k), 28, |k| b.cards.prove(k));
    dim("time", &t.time, |k| b.time.contains_key(k), 8, |k| b.time.prove(k));
    dim("coins", &t.coins, |k| b.coins.contains_key(k), 2, |k| b.coins.prove(k));

    // adjacency: outer leaf [degree (2), set root (4)] = 6 limbs; outer branches;
    // the inner set tries of the particles that received members
    for (name, keys, adds, index) in [("axons_out", &t.out, &t.adds_out, &b.axons_out), ("axons_in", &t.inn, &t.adds_in, &b.axons_in)] {
        dim(name, keys, |k| index.contains_key(k), 6, |k| index.prove(k));
        let mut per_particle: BTreeMap<Id, BTreeSet<(u16, [u8; 32])>> = BTreeMap::new();
        for (particle, axon) in adds {
            if let Some(proof) = index.prove_member(particle, axon) {
                branches(&proof.steps, axon, per_particle.entry(*particle).or_default());
            }
        }
        let inner: u64 = per_particle.values().map(|v| v.len() as u64).sum();
        bump(&format!("state.{name}.inner_dirty_branches"), inner);
        bump(&format!("state.{name}.member_adds"), adds.len() as u64);
        derived(&format!("trie_branch.{name}_inner"), 2 * inner);
    }
    root
}
