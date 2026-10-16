//! The pool (rule set 2) — formats frozen at genesis, `specs/chain.md` §11.
//!
//! The pool is off at launch: rule set 1 rejects every pool transaction
//! (kinds 4 and 5 are unknown to its decoder). What genesis freezes is the
//! material and the formats — the record, its commitment, rho and
//! nullifier, the commitment tree, the anchor window, the turnstile, the
//! `SPEND` public-input encoding, the block-proof statement and the domain
//! tags — so that switching the pool on adds verifying keys and nothing
//! else. The genesis `p` frame carries a [`Pool`] record whose tags digest
//! is computed here; the chain id therefore commits to every constant below.
//!
//! Every consensus-critical digest of the pool is wide: 64 bytes, two
//! squeezes of hemera's own sponge ([`wide`], `Hasher::finalize_xof`); a
//! tree node is the wide digest of its tagged children ([`node`]). Only the
//! hemera API is used — no permutation constant, flag or capacity slot is
//! pinned here. The 32-byte particle stays a locator (token, data, link
//! endpoints, chain id).
//! Integers only, little-endian; every 32- and 64-byte field is a sequence
//! of 8-byte limbs, each `< p`, checked by every decoder.

use std::io;

use hemera::field::P;

use crate::genesis::{Id, hash32};

mod record;
mod spend;
mod vectors;

pub use record::{Pool, Record};
pub use spend::{Link, SpendPublic, Withdrawal, block_statement, block_statement_id};
pub use vectors::{SAMPLE_CHAIN, SAMPLE_DEPOSIT_TXID, sample, sample_withdrawal, vectors};

#[cfg(test)]
mod tests;

/// A wide digest: eight canonical Goldilocks limbs, little-endian.
pub type Wide = [u8; WIDE_BYTES];

/// Pool format version written into the genesis `p` frame.
pub const FORMAT: u32 = 1;
/// Bytes of a wide digest (chain.md §11.1).
pub const WIDE_BYTES: usize = 64;
/// Limbs of a wide digest.
pub const WIDE_LIMBS: usize = 8;
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
/// `token 32 ‖ value 8 ‖ owner 64 ‖ rho 64 ‖ r 32 ‖ data 32 ‖ predicate 64`.
pub const RECORD_BYTES: usize = 296;
/// Encoded [`Pool`] record at the end of the genesis `p` frame.
pub const POOL_BYTES: usize = 74;
/// Transaction kinds reserved for the pool (unknown to rule set 1).
pub const KIND_DEPOSIT: u8 = 4;
pub const KIND_SPEND: u8 = 5;
/// Deposit body: `from 32 ‖ nonce 8 ‖ token 32 ‖ value 8 ‖ owner 64 ‖ r 32 ‖ data 32 ‖ predicate 64 ‖ fee 8`
/// (the record without `rho`, which the chain derives).
pub const DEPOSIT_BODY_BYTES: usize = 32 + 8 + RECORD_BYTES - WIDE_BYTES + 8;
/// Deposit total: envelope 2 + body + NSIG1 102.
pub const DEPOSIT_BYTES: usize = 2 + DEPOSIT_BODY_BYTES + 102;
/// Bytes of one cyberlink output: `p 32 ‖ q 32 ‖ weight 8 ‖ payout 64`.
pub const LINK_BYTES: usize = 136;
/// Bytes of a withdrawal: `to 32 ‖ token 32 ‖ amount 8`.
pub const WITHDRAWAL_BYTES: usize = 72;
/// The longest canonical `SPEND` public-input encoding.
pub const MAX_SPEND_BYTES: usize = 32
    + WIDE_BYTES
    + 1
    + WIDE_BYTES * MAX_INPUTS
    + 1
    + WIDE_BYTES * MAX_OUTPUTS
    + 1
    + LINK_BYTES * MAX_LINKS
    + 1
    + WITHDRAWAL_BYTES
    + 32
    + 8
    + WIDE_BYTES * (MAX_INPUTS + MAX_OUTPUTS)
    + 1
    + MAX_OUTPUTS * (2 + MAX_NOTE_BYTES);
/// Spend total: envelope 2 + `len: u16` + public inputs (no signature).
pub const MAX_SPEND_TX_BYTES: usize = 2 + 2 + MAX_SPEND_BYTES;

// Domain tags — chosen once, documented in chain.md §11.9, never reused.
pub const D_OWNER: &[u8] = b"soft3/pool/owner/v1\0";
pub const D_CM: &[u8] = b"soft3/pool/cm/v1\0";
pub const D_NULL: &[u8] = b"soft3/pool/nullifier/v1\0";
pub const D_RHO: &[u8] = b"soft3/pool/rho/v1\0";
pub const D_PRED: &[u8] = b"soft3/pool/predicate/v1\0";
pub const D_EMPTY: &[u8] = b"soft3/pool/empty/v1\0";
pub const D_NODE: &[u8] = b"soft3/pool/node/v1\0";
pub const D_SPEND: &[u8] = b"soft3/pool/spend/v1\0";
pub const D_BLOCK: &[u8] = b"soft3/pool/block/v1\0";
pub const D_TURNSTILE: &[u8] = b"soft3/pool/turnstile/v1\0";
pub const D_VK: &[u8] = b"soft3/pool/vk/v1\0";
pub const D_TAGS: &[u8] = b"soft3/pool/tags/v1\0";
/// The tags in digest order.
pub const TAGS: [&[u8]; 11] = [D_OWNER, D_CM, D_NULL, D_RHO, D_PRED, D_EMPTY, D_NODE, D_SPEND, D_BLOCK, D_TURNSTILE, D_VK];

pub(crate) fn invalid(msg: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.into())
}

/// Every 8-byte little-endian limb of `b` is `< p` (a canonical element).
pub fn canonical(b: &[u8]) -> bool {
    b.len() % 8 == 0 && b.chunks(8).all(|c| u64::from_le_bytes(c.try_into().unwrap()) < P)
}

pub(crate) fn require_canonical(what: &str, b: &[u8]) -> io::Result<()> {
    if canonical(b) { Ok(()) } else { Err(invalid(format!("{what}: a limb is not < p"))) }
}

fn cat(tag: &[u8], parts: &[&[u8]]) -> Vec<u8> {
    let mut b = Vec::with_capacity(tag.len() + parts.iter().map(|p| p.len()).sum::<usize>());
    b.extend_from_slice(tag);
    for p in parts {
        b.extend_from_slice(p);
    }
    b
}

/// `W(x)`: the first 64 bytes of hemera's extendable output over `x` — the
/// plain sponge absorbs `x` exactly as `hemera(x)` does, then two squeezes
/// of four elements each (`Hasher::finalize_xof`). `W(x)[..32] = hemera(x)`;
/// separation from every other hemera input is the pool tag `x` begins with.
pub fn wide(x: &[u8]) -> Wide {
    let mut h = hemera::Hasher::new();
    h.update(x);
    let mut out = [0u8; WIDE_BYTES];
    h.finalize_xof().fill(&mut out);
    out
}

/// [`wide`] in constant time, for inputs that carry `sk` (same digest).
pub fn wide_secret(x: &[u8]) -> Wide {
    let mut h = hemera::Hasher::new_secret();
    h.update(x);
    let mut out = [0u8; WIDE_BYTES];
    h.finalize_xof().fill(&mut out);
    out
}

/// `W(tag ‖ parts…)`.
pub(crate) fn tagged(tag: &[u8], parts: &[&[u8]]) -> Wide {
    wide(&cat(tag, parts))
}

fn tagged_secret(tag: &[u8], parts: &[&[u8]]) -> Wide {
    wide_secret(&cat(tag, parts))
}

/// A tree node (chain.md §11.4): `W(D_NODE ‖ left ‖ right)`, 147 bytes in.
/// Cost (hemera 0.4, same sponge as 0.3): three absorbing permutations (56-byte rate blocks,
/// the last carrying the padding) and one more for the second squeeze —
/// four, against one for the narrow node jet over 32-byte children.
pub fn node(left: &Wide, right: &Wide) -> Wide {
    debug_assert!(canonical(left) && canonical(right), "node children are canonical digests");
    tagged(D_NODE, &[left, right])
}

/// `W(D_TAGS ‖ D_OWNER ‖ D_CM ‖ D_NULL ‖ D_RHO ‖ D_PRED ‖ D_EMPTY ‖ D_NODE ‖ D_SPEND ‖ D_BLOCK ‖ D_TURNSTILE ‖ D_VK)`,
/// each tag with its NUL terminator.
pub fn tags_digest() -> Wide {
    tagged(D_TAGS, &TAGS)
}

/// The leaf of every unfilled tree position: `W(D_EMPTY)`.
pub fn empty_leaf() -> Wide {
    wide(D_EMPTY)
}

/// `E_0 = EMPTY`, `E_i = node(E_{i−1}, E_{i−1})`: the roots of
/// the empty subtrees of every height, `E_DEPTH` the root of the empty tree.
pub fn empty_roots() -> Vec<Wide> {
    let mut e = Vec::with_capacity(DEPTH as usize + 1);
    e.push(empty_leaf());
    for i in 1..=DEPTH as usize {
        let below = e[i - 1];
        e.push(node(&below, &below));
    }
    e
}

/// The anchor of a chain with no commitments (block 0): `E_DEPTH`.
pub fn empty_root() -> Wide {
    empty_roots()[DEPTH as usize]
}

/// Root of the depth-`DEPTH` tree holding `leaves` at positions
/// `0..leaves.len()` (append order) and `EMPTY` everywhere else.
pub fn root_of(leaves: &[Wide]) -> Wide {
    assert!((leaves.len() as u128) <= 1u128 << DEPTH, "more leaves than positions");
    let empty = empty_roots();
    if leaves.is_empty() {
        return empty[DEPTH as usize];
    }
    let mut level: Vec<Wide> = leaves.to_vec();
    for i in 1..=DEPTH as usize {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for pair in level.chunks(2) {
            let right = if pair.len() == 2 { pair[1] } else { empty[i - 1] };
            next.push(node(&pair[0], &right));
        }
        level = next;
    }
    level[0]
}

/// Recompute a root from a leaf at `position` and its 40 siblings, leaf to
/// root (bit `i` of `position` set: the sibling at level `i` is on the left).
pub fn root_from_path(leaf: &Wide, position: u64, siblings: &[Wide]) -> Option<Wide> {
    if siblings.len() != DEPTH as usize || position >> DEPTH != 0 {
        return None;
    }
    let mut cur = *leaf;
    for (i, sib) in siblings.iter().enumerate() {
        cur = if (position >> i) & 1 == 1 { node(sib, &cur) } else { node(&cur, sib) };
    }
    Some(cur)
}

/// `owner = W(D_OWNER ‖ sk)`.
pub fn owner_of(sk: &Id) -> Wide {
    tagged_secret(D_OWNER, &[sk])
}

/// `nullifier = W(D_NULL ‖ chain ‖ sk ‖ rho)`.
pub fn nullifier(chain: &Id, sk: &Id, rho: &Wide) -> Wide {
    tagged_secret(D_NULL, &[chain, sk, rho])
}

/// `rho_j = W(D_RHO ‖ nf_0 ‖ j: u8)`: the rho of output `j` of a spend whose
/// first nullifier is `nf_0` (derived inside the proof).
pub fn rho_of_output(nf0: &Wide, j: u8) -> Wide {
    tagged(D_RHO, &[nf0, &[j]])
}

/// `rho = W(D_RHO ‖ txid ‖ 0: u8)`: the rho of a deposited record, derived
/// by every node from the deposit's transaction id (32-byte seed, so it never
/// meets a spend's 64-byte seed).
pub fn rho_of_deposit(txid: &Id) -> Wide {
    tagged(D_RHO, &[txid, &[0]])
}

/// `predicate = W(D_PRED ‖ program)` over a nox program's canonical bytes.
pub fn predicate_of(program: &[u8]) -> Wide {
    tagged(D_PRED, &[program])
}

/// The turnstile of `token`: `T(token) = hemera(D_TURNSTILE ‖ token)`, a
/// 32-byte raw key of bbg's balances dimension (a key of the transparent
/// ledger, sized like every balance key). Its input is 56 bytes; every
/// transparent balance key `hemera(owner ‖ token)` hashes 64, so no
/// transfer, coinbase, claim or genesis frame can reach it. Absent (zero)
/// at genesis.
pub fn turnstile_key(token: &Id) -> Id {
    let mut b = D_TURNSTILE.to_vec();
    b.extend_from_slice(token);
    hash32(&b)
}

/// `W(D_VK ‖ (len: u32 ‖ vk) …)` over a rule set's verifying keys in the
/// order its activation entry lists them.
pub fn vk_set_digest(vks: &[&[u8]]) -> Wide {
    let mut b = D_VK.to_vec();
    for vk in vks {
        b.extend_from_slice(&(vk.len() as u32).to_le_bytes());
        b.extend_from_slice(vk);
    }
    wide(&b)
}
