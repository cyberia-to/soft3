//! The `SPEND` public inputs and the block-proof statement (chain.md §11.5, §11.7).

use std::io;

use super::{D_BLOCK, D_SPEND, MAX_INPUTS, MAX_LINKS, MAX_NOTE_BYTES, MAX_OUTPUTS, invalid, tagged};
use crate::genesis::Id;

/// A public cyberlink output of a spend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Link {
    pub p: Id,
    pub q: Id,
    pub weight: u64,
    pub payout: Id,
}

/// A transparent withdrawal output of a spend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Withdrawal {
    pub to: Id,
    pub token: Id,
    pub amount: u64,
}

/// The public inputs of `SPEND` (chain.md §11.5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpendPublic {
    pub chain: Id,
    pub anchor: Id,
    pub nullifiers: Vec<Id>,
    pub commitments: Vec<Id>,
    pub links: Vec<Link>,
    pub withdrawal: Option<Withdrawal>,
    pub fee_token: Id,
    pub fee: u64,
    /// one per input, then one per output
    pub predicates: Vec<Id>,
    /// at most one per output
    pub notes: Vec<Vec<u8>>,
}

impl SpendPublic {
    pub fn check(&self) -> io::Result<()> {
        let n = self.nullifiers.len();
        let m = self.commitments.len();
        if n == 0 || n > MAX_INPUTS || m > MAX_OUTPUTS {
            return Err(invalid(format!("{n} inputs, {m} outputs: 1..=4 inputs, 0..=4 outputs")));
        }
        if self.links.len() > MAX_LINKS {
            return Err(invalid("too many links"));
        }
        if self.predicates.len() != n + m {
            return Err(invalid("one predicate per input and per output"));
        }
        if self.notes.len() > m || self.notes.iter().any(|x| x.len() > MAX_NOTE_BYTES) {
            return Err(invalid("notes: at most one per output, each at most MAX_NOTE_BYTES"));
        }
        for (i, a) in self.nullifiers.iter().enumerate() {
            if self.nullifiers[..i].contains(a) {
                return Err(invalid("duplicate nullifier"));
            }
        }
        for (i, a) in self.commitments.iter().enumerate() {
            if self.commitments[..i].contains(a) {
                return Err(invalid("duplicate commitment"));
            }
        }
        if self.links.iter().any(|l| l.p == l.q) {
            return Err(invalid("link with p == q"));
        }
        if self.withdrawal.is_some_and(|w| w.amount == 0) {
            return Err(invalid("a withdrawal of 0 is written as no withdrawal"));
        }
        Ok(())
    }

    /// Canonical bytes (chain.md §11.5).
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        self.check()?;
        let mut b = Vec::with_capacity(1_024);
        b.extend_from_slice(&self.chain);
        b.extend_from_slice(&self.anchor);
        b.push(self.nullifiers.len() as u8);
        for x in &self.nullifiers {
            b.extend_from_slice(x);
        }
        b.push(self.commitments.len() as u8);
        for x in &self.commitments {
            b.extend_from_slice(x);
        }
        b.push(self.links.len() as u8);
        for l in &self.links {
            b.extend_from_slice(&l.p);
            b.extend_from_slice(&l.q);
            b.extend_from_slice(&l.weight.to_le_bytes());
            b.extend_from_slice(&l.payout);
        }
        match &self.withdrawal {
            None => b.push(0),
            Some(w) => {
                b.push(1);
                b.extend_from_slice(&w.to);
                b.extend_from_slice(&w.token);
                b.extend_from_slice(&w.amount.to_le_bytes());
            }
        }
        b.extend_from_slice(&self.fee_token);
        b.extend_from_slice(&self.fee.to_le_bytes());
        for x in &self.predicates {
            b.extend_from_slice(x);
        }
        b.push(self.notes.len() as u8);
        for x in &self.notes {
            b.extend_from_slice(&(x.len() as u16).to_le_bytes());
            b.extend_from_slice(x);
        }
        Ok(b)
    }

    /// Strict decoder: counts within caps, no trailing bytes.
    pub fn decode(b: &[u8]) -> io::Result<Self> {
        let mut r = Cursor(b);
        let chain = r.id()?;
        let anchor = r.id()?;
        let n = r.u8()? as usize;
        let nullifiers = (0..n).map(|_| r.id()).collect::<io::Result<Vec<_>>>()?;
        let m = r.u8()? as usize;
        let commitments = (0..m).map(|_| r.id()).collect::<io::Result<Vec<_>>>()?;
        let k = r.u8()? as usize;
        let mut links = Vec::with_capacity(k);
        for _ in 0..k {
            links.push(Link { p: r.id()?, q: r.id()?, weight: r.u64()?, payout: r.id()? });
        }
        let withdrawal = match r.u8()? {
            0 => None,
            1 => Some(Withdrawal { to: r.id()?, token: r.id()?, amount: r.u64()? }),
            w => return Err(invalid(format!("withdrawal flag {w}"))),
        };
        let fee_token = r.id()?;
        let fee = r.u64()?;
        let predicates = (0..n + m).map(|_| r.id()).collect::<io::Result<Vec<_>>>()?;
        let c = r.u8()? as usize;
        let mut notes = Vec::with_capacity(c);
        for _ in 0..c {
            let len = u16::from_le_bytes(r.take(2)?.try_into().unwrap()) as usize;
            notes.push(r.take(len)?.to_vec());
        }
        if !r.0.is_empty() {
            return Err(invalid(format!("{} trailing bytes in spend", r.0.len())));
        }
        let s = Self { chain, anchor, nullifiers, commitments, links, withdrawal, fee_token, fee, predicates, notes };
        s.check()?;
        Ok(s)
    }

    /// `spend_id = hemera(D_SPEND ‖ encode)`: what the proof binds and
    /// what the block-proof statement lists.
    pub fn id(&self) -> io::Result<Id> {
        Ok(tagged(D_SPEND, &[&self.encode()?]))
    }
}

struct Cursor<'a>(&'a [u8]);

impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> io::Result<&'a [u8]> {
        if self.0.len() < n {
            return Err(invalid("truncated spend"));
        }
        let (a, b) = self.0.split_at(n);
        self.0 = b;
        Ok(a)
    }
    fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn id(&mut self) -> io::Result<Id> {
        Ok(self.take(32)?.try_into().unwrap())
    }
}

/// The block-proof statement (chain.md §11.7):
/// `chain ‖ height ‖ vk_set ‖ count: u32 ‖ spend_id × count`.
pub fn block_statement(chain: &Id, height: u64, vk_set: &Id, spend_ids: &[Id]) -> Vec<u8> {
    let mut b = Vec::with_capacity(76 + 32 * spend_ids.len());
    b.extend_from_slice(chain);
    b.extend_from_slice(&height.to_le_bytes());
    b.extend_from_slice(vk_set);
    b.extend_from_slice(&(spend_ids.len() as u32).to_le_bytes());
    for s in spend_ids {
        b.extend_from_slice(s);
    }
    b
}

/// `hemera(D_BLOCK ‖ block_statement)`: the block proof's public input.
pub fn block_statement_id(statement: &[u8]) -> Id {
    tagged(D_BLOCK, &[statement])
}

