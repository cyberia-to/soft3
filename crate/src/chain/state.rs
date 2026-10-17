//! The state backend the kernel codes against (§3), and the undo journal
//! that makes every block transition reversible for reorgs (§5.2).
//!
//! The kernel never touches bbg directly: [`StateBackend`] is the whole
//! contract, implemented over `bbg::BbgState` in [`super::bbg_state`]. A
//! second implementation (a test double, or a future incremental bbg) plugs
//! in without a kernel change.

pub type Id = [u8; 32];

pub use super::economy::{Card, Mark, Scalars};

/// Before-image of one touched entry. Reverting a journal restores these in
/// reverse order, so recording every touch (not only the first) is correct.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UndoEntry {
    /// `balances[key]` was `before` (`None` = absent).
    Balance { key: Id, before: Option<u64> },
    /// the neuron record of `id` was `before` as `[focus, karma, stake, nonce]`.
    Neuron { id: Id, before: Option<[u64; 4]> },
    /// `coins[token].total_supply` was `before`.
    Coin { token: Id, before: Option<u64> },
    /// the particle record of `id` was `before` as
    /// `[energy, pi_star, weight, s_yes, s_no, meta_score]`.
    Particle { id: Id, before: Option<[u64; 6]> },
    /// `axons_out[p]` had `before` entries (`None` = no list).
    AxonsOut { p: Id, before: Option<u64> },
    /// `axons_in[q]` had `before` entries.
    AxonsIn { q: Id, before: Option<u64> },
    /// `axon_edges[axon]` existed before.
    AxonEdge { axon: Id, existed: bool },
    /// the running `Σ balances[·, token]` was `before`.
    TokenSum { token: Id, before: Option<u64> },
    /// the state height was `before`.
    Height { before: u64 },
    /// the card (position) `id` was `before` (chain-economy §3.2).
    Card { id: Id, before: Option<Card> },
    /// the high-water marks of `axon` were `before` (chain-economy §5.3).
    Mark { axon: Id, before: Option<Mark> },
    /// the economy scalars were `before`.
    Scalars { before: Scalars },
    /// `time[height]` was `before` (chain-economy §5.6).
    Time { height: u64, before: Option<Id> },
}

/// The journal of one block, oldest entry first.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Undo(pub Vec<UndoEntry>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    /// a debit exceeds the balance (after coinbase maturity is applied by the caller)
    Insufficient { have: u64, need: u64 },
    /// a credit, mint or sum would exceed u64
    Overflow,
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateError::Insufficient { have, need } => write!(f, "insufficient balance: have {have}, need {need}"),
            StateError::Overflow => write!(f, "arithmetic overflow"),
        }
    }
}

/// Everything the transition (§3.4), genesis (§4.1) and the mempool read or
/// write. Every mutation is journaled between [`begin`](Self::begin) and
/// [`commit`](Self::commit); [`revert`](Self::revert) undoes a committed
/// journal exactly.
pub trait StateBackend {
    // ── reads ───────────────────────────────────────────────────────
    fn height(&self) -> u64;
    fn balance(&self, owner: &Id, token: &Id) -> u64;
    /// `neurons[id].nonce`, 0 when the record is absent.
    fn nonce(&self, id: &Id) -> u64;
    fn has_neuron(&self, id: &Id) -> bool;
    /// `coins[token].total_supply`, 0 when absent.
    fn supply(&self, token: &Id) -> u64;
    /// every token with a coin record, ascending
    fn tokens(&self) -> Vec<Id>;
    /// running `Σ balances[·, token]`, maintained through this trait
    fn token_sum(&self, token: &Id) -> u64;
    fn neuron_count(&self) -> u64;
    fn particle_count(&self) -> u64;
    fn axon_count(&self) -> u64;
    /// the bbg root after the mutations so far (recomputed, §3.3)
    fn root(&mut self) -> Id;

    // ── writes (journaled) ──────────────────────────────────────────
    fn set_height(&mut self, height: u64);
    fn credit(&mut self, owner: &Id, token: &Id, amount: u64) -> Result<(), StateError>;
    fn debit(&mut self, owner: &Id, token: &Id, amount: u64) -> Result<(), StateError>;
    /// create the record with nonce 0 if absent
    fn ensure_neuron(&mut self, id: &Id);
    /// `neurons[id].nonce += 1`, creating the record if absent
    fn bump_nonce(&mut self, id: &Id) -> Result<(), StateError>;
    /// `coins[token].total_supply += amount`, creating the record if absent
    fn mint(&mut self, token: &Id, amount: u64) -> Result<(), StateError>;
    /// bbg `insert` of one link `(p, q, token, amount, valence)` by `from`
    /// at `height`; `amount` is 0 under rule set 1 (§2.4, §10 item 2)
    fn link(&mut self, from: &Id, p: &Id, q: &Id, token: &Id, amount: u64, valence: i8, height: u64) -> Result<(), StateError>;

    // ── the focus economy (chain-economy §3, §5) ────────────────────
    /// `balances[key]` for a raw key: a stake key or the focus pool (§3.1)
    fn raw_balance(&self, key: &Id) -> u64;
    /// credit a raw key; `token` keeps the conservation sum
    fn credit_raw(&mut self, key: &Id, token: &Id, amount: u64) -> Result<(), StateError>;
    fn debit_raw(&mut self, key: &Id, token: &Id, amount: u64) -> Result<(), StateError>;
    fn card(&self, id: &Id) -> Option<Card>;
    /// write or (with `None`) delete a card
    fn set_card(&mut self, id: &Id, card: Option<Card>);
    /// every card, ascending by id
    fn cards(&self) -> Vec<(Id, Card)>;
    /// `(p, q)` of an axon from `axon_edges`
    fn axon_edge(&self, axon: &Id) -> Option<(Id, Id)>;
    /// every high-water mark, ascending by axon
    fn marks(&self) -> Vec<(Id, Mark)>;
    fn set_mark(&mut self, axon: &Id, mark: Option<Mark>);
    fn scalars(&self) -> Scalars;
    fn set_scalars(&mut self, s: Scalars);
    fn time(&self, height: u64) -> Option<Id>;
    fn set_time(&mut self, height: u64, value: Id);

    // ── journal ─────────────────────────────────────────────────────
    fn begin(&mut self);
    fn commit(&mut self) -> Undo;
    fn revert(&mut self, undo: &Undo);
}

// ── undo wire form, for the store ──────────────────────────────────

fn put_opt_u64s(out: &mut Vec<u8>, v: Option<&[u64]>) {
    match v {
        None => out.push(0),
        Some(vals) => {
            out.push(1);
            for x in vals {
                out.extend_from_slice(&x.to_le_bytes());
            }
        }
    }
}

impl Undo {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.0.len() * 48 + 4);
        out.extend_from_slice(&(self.0.len() as u32).to_le_bytes());
        for e in &self.0 {
            match e {
                UndoEntry::Balance { key, before } => {
                    out.push(0);
                    out.extend_from_slice(key);
                    put_opt_u64s(&mut out, before.as_ref().map(std::slice::from_ref));
                }
                UndoEntry::Neuron { id, before } => {
                    out.push(1);
                    out.extend_from_slice(id);
                    put_opt_u64s(&mut out, before.as_ref().map(|b| &b[..]));
                }
                UndoEntry::Coin { token, before } => {
                    out.push(2);
                    out.extend_from_slice(token);
                    put_opt_u64s(&mut out, before.as_ref().map(std::slice::from_ref));
                }
                UndoEntry::Particle { id, before } => {
                    out.push(3);
                    out.extend_from_slice(id);
                    put_opt_u64s(&mut out, before.as_ref().map(|b| &b[..]));
                }
                UndoEntry::AxonsOut { p, before } => {
                    out.push(4);
                    out.extend_from_slice(p);
                    put_opt_u64s(&mut out, before.as_ref().map(std::slice::from_ref));
                }
                UndoEntry::AxonsIn { q, before } => {
                    out.push(5);
                    out.extend_from_slice(q);
                    put_opt_u64s(&mut out, before.as_ref().map(std::slice::from_ref));
                }
                UndoEntry::AxonEdge { axon, existed } => {
                    out.push(6);
                    out.extend_from_slice(axon);
                    out.push(*existed as u8);
                }
                UndoEntry::TokenSum { token, before } => {
                    out.push(7);
                    out.extend_from_slice(token);
                    put_opt_u64s(&mut out, before.as_ref().map(std::slice::from_ref));
                }
                UndoEntry::Height { before } => {
                    out.push(8);
                    out.extend_from_slice(&before.to_le_bytes());
                }
                UndoEntry::Card { id, before } => {
                    out.push(9);
                    out.extend_from_slice(id);
                    match before {
                        None => out.push(0),
                        Some(c) => {
                            out.push(1);
                            out.extend_from_slice(&c.owner);
                            out.extend_from_slice(&c.particle);
                            out.extend_from_slice(&c.author);
                            out.push(c.valence_byte());
                            out.extend_from_slice(&c.since.to_le_bytes());
                        }
                    }
                }
                UndoEntry::Mark { axon, before } => {
                    out.push(10);
                    out.extend_from_slice(axon);
                    put_opt_u64s(&mut out, before.map(|m| [m.pos, m.neg]).as_ref().map(|v| &v[..]));
                }
                UndoEntry::Scalars { before } => {
                    out.push(11);
                    for v in before.encode() {
                        out.extend_from_slice(&v.to_le_bytes());
                    }
                }
                UndoEntry::Time { height, before } => {
                    out.push(12);
                    out.extend_from_slice(&height.to_le_bytes());
                    match before {
                        None => out.push(0),
                        Some(v) => {
                            out.push(1);
                            out.extend_from_slice(v);
                        }
                    }
                }
            }
        }
        out
    }

    pub fn decode(b: &[u8]) -> Option<Undo> {
        let mut at = 0usize;
        let take = |at: &mut usize, n: usize| -> Option<&[u8]> {
            let s = b.get(*at..*at + n)?;
            *at += n;
            Some(s)
        };
        let count = u32::from_le_bytes(take(&mut at, 4)?.try_into().ok()?) as usize;
        let mut out = Vec::with_capacity(count.min(1 << 16));
        for _ in 0..count {
            let tag = take(&mut at, 1)?[0];
            if tag == 8 {
                let before = u64::from_le_bytes(take(&mut at, 8)?.try_into().ok()?);
                out.push(UndoEntry::Height { before });
                continue;
            }
            if tag == 11 {
                let mut v = [0u64; 4];
                for x in v.iter_mut() {
                    *x = u64::from_le_bytes(take(&mut at, 8)?.try_into().ok()?);
                }
                out.push(UndoEntry::Scalars { before: Scalars::decode(v) });
                continue;
            }
            if tag == 12 {
                let height = u64::from_le_bytes(take(&mut at, 8)?.try_into().ok()?);
                let before = match take(&mut at, 1)?[0] {
                    0 => None,
                    1 => Some(take(&mut at, 32)?.try_into().ok()?),
                    _ => return None,
                };
                out.push(UndoEntry::Time { height, before });
                continue;
            }
            let id: Id = take(&mut at, 32)?.try_into().ok()?;
            if tag == 6 {
                let existed = take(&mut at, 1)?[0] != 0;
                out.push(UndoEntry::AxonEdge { axon: id, existed });
                continue;
            }
            if tag == 9 {
                let before = match take(&mut at, 1)?[0] {
                    0 => None,
                    1 => {
                        let owner = take(&mut at, 32)?.try_into().ok()?;
                        let particle = take(&mut at, 32)?.try_into().ok()?;
                        let author = take(&mut at, 32)?.try_into().ok()?;
                        let vb = take(&mut at, 1)?[0];
                        if vb > 2 {
                            return None;
                        }
                        let since = u64::from_le_bytes(take(&mut at, 8)?.try_into().ok()?);
                        Some(Card { owner, particle, author, valence: vb as i8 - 1, since })
                    }
                    _ => return None,
                };
                out.push(UndoEntry::Card { id, before });
                continue;
            }
            let n = match tag {
                1 => 4,
                3 => 6,
                10 => 2,
                _ => 1,
            };
            let present = take(&mut at, 1)?[0] != 0;
            let mut vals = [0u64; 6];
            if present {
                for v in vals.iter_mut().take(n) {
                    *v = u64::from_le_bytes(take(&mut at, 8)?.try_into().ok()?);
                }
            }
            let one = present.then_some(vals[0]);
            out.push(match tag {
                0 => UndoEntry::Balance { key: id, before: one },
                1 => UndoEntry::Neuron { id, before: present.then_some([vals[0], vals[1], vals[2], vals[3]]) },
                2 => UndoEntry::Coin { token: id, before: one },
                3 => UndoEntry::Particle { id, before: present.then_some(vals) },
                4 => UndoEntry::AxonsOut { p: id, before: one },
                5 => UndoEntry::AxonsIn { q: id, before: one },
                7 => UndoEntry::TokenSum { token: id, before: one },
                10 => UndoEntry::Mark { axon: id, before: present.then_some(Mark { pos: vals[0], neg: vals[1] }) },
                _ => return None,
            });
        }
        if at != b.len() {
            return None;
        }
        Some(Undo(out))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_round_trips() {
        let u = Undo(vec![
            UndoEntry::Balance { key: [1; 32], before: None },
            UndoEntry::Balance { key: [2; 32], before: Some(7) },
            UndoEntry::Neuron { id: [3; 32], before: Some([0, 0, 0, 9]) },
            UndoEntry::Neuron { id: [4; 32], before: None },
            UndoEntry::Coin { token: [5; 32], before: Some(u64::MAX) },
            UndoEntry::Particle { id: [6; 32], before: Some([1, 2, 3, 4, 5, 6]) },
            UndoEntry::AxonsOut { p: [7; 32], before: Some(2) },
            UndoEntry::AxonsIn { q: [8; 32], before: None },
            UndoEntry::AxonEdge { axon: [9; 32], existed: true },
            UndoEntry::TokenSum { token: [10; 32], before: Some(3) },
            UndoEntry::Height { before: 41 },
            UndoEntry::Card { id: [11; 32], before: None },
            UndoEntry::Card { id: [12; 32], before: Some(Card { owner: [1; 32], particle: [2; 32], author: [3; 32], valence: -1, since: 99 }) },
            UndoEntry::Mark { axon: [13; 32], before: Some(Mark { pos: 5, neg: 6 }) },
            UndoEntry::Mark { axon: [14; 32], before: None },
            UndoEntry::Scalars { before: Scalars { focus_bps: 999, total_stake: 7, active_stake: 3, last_settled: 2 } },
            UndoEntry::Time { height: 360, before: None },
            UndoEntry::Time { height: 600, before: Some([15; 32]) },
        ]);
        let bytes = u.encode();
        assert_eq!(Undo::decode(&bytes), Some(u));
        assert_eq!(Undo::decode(&bytes[..bytes.len() - 1]), None);
        let mut long = bytes;
        long.push(0);
        assert_eq!(Undo::decode(&long), None);
    }
}
