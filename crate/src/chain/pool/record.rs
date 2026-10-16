//! The pool's record and the genesis pool record (chain.md §11.1, §11.9).

use std::io;

use super::{ANCHOR_WINDOW, D_CM, DEPTH, FORMAT, POOL_BYTES, RECORD_BYTES, WIDE_BYTES, Wide, invalid, require_canonical, tagged, tags_digest};
use crate::genesis::{Id, hex};

/// A pool record (chain.md §11.1): 296 bytes, every 32- and 64-byte field
/// canonical (each 8-byte limb `< p`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    pub token: Id,
    pub value: u64,
    pub owner: Wide,
    pub rho: Wide,
    pub r: Id,
    pub data: Id,
    pub predicate: Wide,
}

impl Record {
    pub fn encode(&self) -> [u8; RECORD_BYTES] {
        let mut b = [0u8; RECORD_BYTES];
        b[0..32].copy_from_slice(&self.token);
        b[32..40].copy_from_slice(&self.value.to_le_bytes());
        b[40..104].copy_from_slice(&self.owner);
        b[104..168].copy_from_slice(&self.rho);
        b[168..200].copy_from_slice(&self.r);
        b[200..232].copy_from_slice(&self.data);
        b[232..296].copy_from_slice(&self.predicate);
        b
    }

    /// Strict: exactly 296 bytes, every identifier limb `< p`.
    pub fn decode(b: &[u8]) -> io::Result<Self> {
        if b.len() != RECORD_BYTES {
            return Err(invalid(format!("record of {} bytes", b.len())));
        }
        let rec = Self {
            token: b[0..32].try_into().unwrap(),
            value: u64::from_le_bytes(b[32..40].try_into().unwrap()),
            owner: b[40..104].try_into().unwrap(),
            rho: b[104..168].try_into().unwrap(),
            r: b[168..200].try_into().unwrap(),
            data: b[200..232].try_into().unwrap(),
            predicate: b[232..296].try_into().unwrap(),
        };
        rec.check()?;
        Ok(rec)
    }

    /// Every identifier field canonical (the value is any u64).
    pub fn check(&self) -> io::Result<()> {
        require_canonical("record token", &self.token)?;
        require_canonical("record owner", &self.owner)?;
        require_canonical("record rho", &self.rho)?;
        require_canonical("record r", &self.r)?;
        require_canonical("record data", &self.data)?;
        require_canonical("record predicate", &self.predicate)
    }

    /// `commitment = W(D_CM ‖ record)`.
    pub fn commitment(&self) -> Wide {
        tagged(D_CM, &[&self.encode()])
    }
}

/// The pool record of the genesis `p` frame (chain.md §4.1, §11.9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pool {
    pub format: u32,
    pub depth: u8,
    pub anchor_window: u32,
    /// bytes of a wide digest
    pub digest: u8,
    /// [`tags_digest`]
    pub tags: Wide,
}

impl Pool {
    /// What every genesis of this spec carries.
    pub fn genesis() -> Self {
        Self { format: FORMAT, depth: DEPTH, anchor_window: ANCHOR_WINDOW, digest: WIDE_BYTES as u8, tags: tags_digest() }
    }

    pub fn encode(&self) -> [u8; POOL_BYTES] {
        let mut b = [0u8; POOL_BYTES];
        b[0..4].copy_from_slice(&self.format.to_le_bytes());
        b[4] = self.depth;
        b[5..9].copy_from_slice(&self.anchor_window.to_le_bytes());
        b[9] = self.digest;
        b[10..74].copy_from_slice(&self.tags);
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
            digest: b[9],
            tags: b[10..74].try_into().unwrap(),
        })
    }

    /// A genesis is valid only with the frozen pool record.
    pub fn check(&self) -> io::Result<()> {
        if *self != Self::genesis() {
            return Err(invalid(format!(
                "pool record (format {}, depth {}, window {}, digest {}) is not the frozen one (format {FORMAT}, depth {DEPTH}, window {ANCHOR_WINDOW}, digest {WIDE_BYTES}, tags {})",
                self.format,
                self.depth,
                self.anchor_window,
                self.digest,
                hex(&tags_digest())
            )));
        }
        Ok(())
    }
}
