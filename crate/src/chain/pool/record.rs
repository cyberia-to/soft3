//! The pool's record and the genesis pool record (chain.md §11.1, §11.9).

use std::io;

use super::{D_CM, DEPTH, ANCHOR_WINDOW, FORMAT, POOL_BYTES, RECORD_BYTES, invalid, tagged, tags_digest};
use crate::genesis::{Id, hex};

/// A pool record (chain.md §11.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    pub token: Id,
    pub value: u64,
    pub owner: Id,
    pub rho: Id,
    pub r: Id,
    pub data: Id,
    pub predicate: Id,
}

impl Record {
    pub fn encode(&self) -> [u8; RECORD_BYTES] {
        let mut b = [0u8; RECORD_BYTES];
        b[0..32].copy_from_slice(&self.token);
        b[32..40].copy_from_slice(&self.value.to_le_bytes());
        b[40..72].copy_from_slice(&self.owner);
        b[72..104].copy_from_slice(&self.rho);
        b[104..136].copy_from_slice(&self.r);
        b[136..168].copy_from_slice(&self.data);
        b[168..200].copy_from_slice(&self.predicate);
        b
    }

    pub fn decode(b: &[u8]) -> io::Result<Self> {
        if b.len() != RECORD_BYTES {
            return Err(invalid(format!("record of {} bytes", b.len())));
        }
        let id = |r: std::ops::Range<usize>| -> Id { b[r].try_into().unwrap() };
        Ok(Self {
            token: id(0..32),
            value: u64::from_le_bytes(b[32..40].try_into().unwrap()),
            owner: id(40..72),
            rho: id(72..104),
            r: id(104..136),
            data: id(136..168),
            predicate: id(168..200),
        })
    }

    /// `commitment = hemera(D_CM ‖ record)`.
    pub fn commitment(&self) -> Id {
        tagged(D_CM, &[&self.encode()])
    }
}

/// The pool record of the genesis `p` frame (chain.md §4.1, §11.9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pool {
    pub format: u32,
    pub depth: u8,
    pub anchor_window: u32,
    /// [`tags_digest`]
    pub tags: Id,
}

impl Pool {
    /// What every genesis of this spec carries.
    pub fn genesis() -> Self {
        Self { format: FORMAT, depth: DEPTH, anchor_window: ANCHOR_WINDOW, tags: tags_digest() }
    }

    pub fn encode(&self) -> [u8; POOL_BYTES] {
        let mut b = [0u8; POOL_BYTES];
        b[0..4].copy_from_slice(&self.format.to_le_bytes());
        b[4] = self.depth;
        b[5..9].copy_from_slice(&self.anchor_window.to_le_bytes());
        b[9..41].copy_from_slice(&self.tags);
        b
    }

    pub fn decode(b: &[u8]) -> io::Result<Self> {
        if b.len() != POOL_BYTES {
            return Err(invalid(format!("pool record of {} bytes", b.len())));
        }
        Ok(Self {
            format: u32::from_le_bytes(b[0..4].try_into().unwrap()),
            depth: b[4],
            anchor_window: u32::from_le_bytes(b[5..9].try_into().unwrap()),
            tags: b[9..41].try_into().unwrap(),
        })
    }

    /// A genesis is valid only with the frozen pool record.
    pub fn check(&self) -> io::Result<()> {
        if *self != Self::genesis() {
            return Err(invalid(format!(
                "pool record {:?} is not the frozen one (format {FORMAT}, depth {DEPTH}, window {ANCHOR_WINDOW}, tags {})",
                self,
                hex(&tags_digest())
            )));
        }
        Ok(())
    }
}

