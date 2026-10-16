//! The pool (rule set 2) — formats frozen at genesis, `specs/chain.md` §11.
//!
//! The pool is off at launch: rule set 1 rejects every pool transaction
//! (kinds 4 and 5 are unknown to its decoder). What genesis freezes is the
//! material and the formats — the record, its commitment and nullifier, the
//! commitment tree, the anchor window, the turnstile, the `SPEND`
//! public-input encoding, the block-proof statement and the domain tags —
//! so that switching the pool on adds verifying keys and nothing else. The
//! genesis `p` frame carries a [`Pool`] record whose tags digest is
//! computed here; the chain id (`hemera(genesis file)`) therefore commits
//! to every constant below.
//!
//! Integers only, little-endian, one byte form per value. `hemera(x)` is
//! the flat sponge `hemera::hash` (`hemera::hash_secret`, the same digest in
//! constant time, wherever `sk` is an input); tree nodes are
//! `hemera::tree::hash_node` (the node jet of zheng#66, `hemera::merkle`'s
//! convention: `is_root` on the last level only).

use std::io;

use crate::genesis::{Id, hash32};

mod record;
mod spend;
mod vectors;

pub use record::{Pool, Record};
pub use spend::{Link, SpendPublic, Withdrawal, block_statement, block_statement_id};
pub use vectors::{sample, sample_withdrawal, vectors};

/// Pool format version written into the genesis `p` frame.
pub const FORMAT: u32 = 1;
/// Commitment-tree depth: 2^40 positions.
pub const DEPTH: u8 = 40;
/// A spend in block `h` may name the tree root after any block `b` with
/// `h − ANCHOR_WINDOW ≤ b ≤ h − 1`.
pub const ANCHOR_WINDOW: u32 = 1_000;
/// Inputs (nullifiers) per spend: at least one, at most four.
pub const MAX_INPUTS: usize = 4;
/// Outputs (commitments) per spend.
pub const MAX_OUTPUTS: usize = 4;
/// Cyberlink outputs per spend.
pub const MAX_LINKS: usize = 4;
/// Bytes of one opaque note.
pub const MAX_NOTE_BYTES: usize = 2_048;
/// `token ‖ value ‖ owner ‖ rho ‖ r ‖ data ‖ predicate`.
pub const RECORD_BYTES: usize = 200;
/// Encoded [`Pool`] record at the end of the genesis `p` frame.
pub const POOL_BYTES: usize = 41;
/// Transaction kinds reserved for the pool (unknown to rule set 1).
pub const KIND_DEPOSIT: u8 = 4;
pub const KIND_SPEND: u8 = 5;
/// Deposit body: `from ‖ nonce ‖ record ‖ fee`.
pub const DEPOSIT_BODY_BYTES: usize = 32 + 8 + RECORD_BYTES + 8;
/// Deposit total: envelope 2 + body + NSIG1 102.
pub const DEPOSIT_BYTES: usize = 2 + DEPOSIT_BODY_BYTES + 102;
/// The longest canonical `SPEND` public-input encoding.
pub const MAX_SPEND_BYTES: usize =
    32 + 32 + 1 + 32 * MAX_INPUTS + 1 + 32 * MAX_OUTPUTS + 1 + 104 * MAX_LINKS + 1 + 72 + 32 + 8 + 32 * (MAX_INPUTS + MAX_OUTPUTS) + 1 + MAX_OUTPUTS * (2 + MAX_NOTE_BYTES);
/// Spend total: envelope 2 + `len: u16` + public inputs (no signature).
pub const MAX_SPEND_TX_BYTES: usize = 2 + 2 + MAX_SPEND_BYTES;

// Domain tags — chosen once, documented in chain.md §11.8, never reused.
pub const D_OWNER: &[u8] = b"soft3/pool/owner/v1\0";
pub const D_CM: &[u8] = b"soft3/pool/cm/v1\0";
pub const D_NULL: &[u8] = b"soft3/pool/nullifier/v1\0";
pub const D_EMPTY: &[u8] = b"soft3/pool/empty/v1\0";
pub const D_SPEND: &[u8] = b"soft3/pool/spend/v1\0";
pub const D_BLOCK: &[u8] = b"soft3/pool/block/v1\0";
pub const D_TURNSTILE: &[u8] = b"soft3/pool/turnstile/v1\0";
pub const D_VK: &[u8] = b"soft3/pool/vk/v1\0";
pub const D_TAGS: &[u8] = b"soft3/pool/tags/v1\0";
/// The tags in digest order.
pub const TAGS: [&[u8]; 8] = [D_OWNER, D_CM, D_NULL, D_EMPTY, D_SPEND, D_BLOCK, D_TURNSTILE, D_VK];

pub(crate) fn invalid(msg: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.into())
}

fn cat(tag: &[u8], parts: &[&[u8]]) -> Vec<u8> {
    let mut b = Vec::with_capacity(tag.len() + parts.iter().map(|p| p.len()).sum::<usize>());
    b.extend_from_slice(tag);
    for p in parts {
        b.extend_from_slice(p);
    }
    b
}

pub(crate) fn tagged(tag: &[u8], parts: &[&[u8]]) -> Id {
    hash32(&cat(tag, parts))
}

/// Constant-time variant for inputs carrying `sk` (same digest).
fn tagged_secret(tag: &[u8], parts: &[&[u8]]) -> Id {
    *hemera::hash_secret(&cat(tag, parts)).as_bytes()
}

/// `hemera(D_TAGS ‖ D_OWNER ‖ D_CM ‖ D_NULL ‖ D_EMPTY ‖ D_SPEND ‖ D_BLOCK ‖ D_TURNSTILE ‖ D_VK)`,
/// each tag with its NUL terminator.
pub fn tags_digest() -> Id {
    tagged(D_TAGS, &TAGS)
}

/// The leaf of every unfilled tree position: `hemera(D_EMPTY)`.
pub fn empty_leaf() -> Id {
    hash32(D_EMPTY)
}

/// `hemera::tree::hash_node(left, right, is_root)`.
pub fn node(left: &Id, right: &Id, is_root: bool) -> Id {
    let l = hemera::Hash::from_bytes(*left);
    let r = hemera::Hash::from_bytes(*right);
    *hemera::tree::hash_node(&l, &r, is_root).as_bytes()
}

/// `E_0 = EMPTY`, `E_{i+1} = node(E_i, E_i, false)` for `i + 1 < DEPTH`,
/// `E_DEPTH = node(E_{DEPTH−1}, E_{DEPTH−1}, true)`: the roots of the empty
/// subtrees of every height, `E_DEPTH` the root of the empty tree.
pub fn empty_roots() -> Vec<Id> {
    let mut e = Vec::with_capacity(DEPTH as usize + 1);
    e.push(empty_leaf());
    for i in 1..=DEPTH as usize {
        let below = e[i - 1];
        e.push(node(&below, &below, i == DEPTH as usize));
    }
    e
}

/// The anchor of a chain with no commitments (block 0): `E_DEPTH`.
pub fn empty_root() -> Id {
    empty_roots()[DEPTH as usize]
}

/// Root of the depth-`DEPTH` tree holding `leaves` at positions
/// `0..leaves.len()` (insertion order) and `EMPTY` everywhere else.
pub fn root_of(leaves: &[Id]) -> Id {
    assert!((leaves.len() as u128) <= 1u128 << DEPTH, "more leaves than positions");
    let empty = empty_roots();
    if leaves.is_empty() {
        return empty[DEPTH as usize];
    }
    let mut level: Vec<Id> = leaves.to_vec();
    for i in 1..=DEPTH as usize {
        let is_root = i == DEPTH as usize;
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for pair in level.chunks(2) {
            let right = if pair.len() == 2 { pair[1] } else { empty[i - 1] };
            next.push(node(&pair[0], &right, is_root));
        }
        level = next;
    }
    level[0]
}

/// `owner = hemera(D_OWNER ‖ sk)`.
pub fn owner_of(sk: &Id) -> Id {
    tagged_secret(D_OWNER, &[sk])
}

/// `nullifier = hemera(D_NULL ‖ sk ‖ rho)`.
pub fn nullifier(sk: &Id, rho: &Id) -> Id {
    tagged_secret(D_NULL, &[sk, rho])
}

/// The turnstile of `token`: `T(token) = hemera(D_TURNSTILE ‖ token)`, a
/// raw key of bbg's balances dimension. Its input is 56 bytes; every
/// transparent balance key `hemera(owner ‖ token)` hashes 64, so no
/// transfer, coinbase, claim or genesis frame can reach it. Absent (zero)
/// at genesis.
pub fn turnstile_key(token: &Id) -> Id {
    tagged(D_TURNSTILE, &[token])
}

/// `hemera(D_VK ‖ (len: u32 ‖ vk) …)` over a rule set's verifying keys in
/// the order its activation entry lists them.
pub fn vk_set_digest(vks: &[&[u8]]) -> Id {
    let mut b = D_VK.to_vec();
    for vk in vks {
        b.extend_from_slice(&(vk.len() as u32).to_le_bytes());
        b.extend_from_slice(vk);
    }
    hash32(&b)
}
