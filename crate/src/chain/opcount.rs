//! Operation counters for the P8 census (director plan row P8; feature
//! `opcount`, off by default). Nothing here is consensus: the hooks only
//! add to thread-local tallies, and with the feature off none of them
//! exists.
//!
//! What is measured and what is derived is kept apart by name:
//! `hash.*` are hemera permutations computed from the length of every
//! input soft3 hashes (the permutation count of `hemera::hash` is
//! `len / 56 + 1`, see [`perms`] and the test that replays the sponge);
//! `state.*` are trie operations soft3 issues through the state backend
//! (measured); `derived.*` are the permutations bbg performs inside those
//! operations, computed in `bbg_state.rs` from bbg's own source.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

/// Why a hash is computed (the census purposes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purpose {
    /// block id = proof-of-work hash of the 168-byte header
    Header,
    /// `txid` of a transaction
    Txid,
    /// Merkle leaf and node of `tx_root`
    TxRoot,
    /// the NSIG1 statement hash of `sigs.rs`
    SigStatement,
    /// hashing inside the signature check that soft3 cannot instrument
    /// (mudra `neuron_of(pubkey)` in `verify_statement`): derived, one
    /// 33-byte hemera hash per secp256k1 verification actually run
    SigAux,
    /// key derivations soft3 performs: balance key, axon id, card id,
    /// stake key, focus key, legacy owner, neuron id
    Key,
    /// key derivations made only to journal before-images for the undo
    /// (`link`'s two `balance_key`): not part of a validity relation
    Journal,
    /// the economy's own hashes: payout leaf and node, φ digest, epoch record
    Economy,
    /// everything else (generic `hash32`)
    Other,
}

impl Purpose {
    pub fn name(self) -> &'static str {
        match self {
            Purpose::Header => "header",
            Purpose::Txid => "txid",
            Purpose::TxRoot => "tx_root",
            Purpose::SigStatement => "sig_statement",
            Purpose::SigAux => "sig_aux",
            Purpose::Key => "key",
            Purpose::Journal => "journal",
            Purpose::Economy => "economy",
            Purpose::Other => "other",
        }
    }
}

/// hemera's rate in bytes: eight 7-byte limbs (hemera `params.rs`, `RATE_BYTES`).
pub const RATE_BYTES: usize = 56;

/// Permutations of `hemera::hash(input)` for an input of `len` bytes: one
/// per full 56-byte rate block, plus the padded final block (`0x01 ‖ 0…`,
/// length in a capacity lane) which is always absorbed; 32 bytes of output
/// are four elements of the eight-element rate, so no squeeze permutation.
pub fn perms(len: usize) -> u64 {
    (len / RATE_BYTES) as u64 + 1
}

/// The kind label of work done by the block itself outside any transaction
/// (the state root, conservation).
pub const BLOCK: u8 = 255;
/// Label of the settlement step at `settle(E)`.
pub const SETTLE: u8 = 254;
/// Label of the boundary snapshot taken after a boundary block.
pub const BOUNDARY: u8 = 253;

#[derive(Default)]
pub struct Tally {
    totals: BTreeMap<String, u64>,
    by_kind: BTreeMap<u8, BTreeMap<String, u64>>,
    seen: HashSet<(u8, u64)>,
}

thread_local! {
    static TALLY: RefCell<Tally> = RefCell::new(Tally::default());
    static KIND: Cell<Option<u8>> = const { Cell::new(None) };
    static PURPOSE: Cell<Option<Purpose>> = const { Cell::new(None) };
    static TAG: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Totals and per-kind tallies taken from the current thread.
#[derive(Default, Debug)]
pub struct Counters {
    pub totals: BTreeMap<String, u64>,
    pub by_kind: BTreeMap<u8, BTreeMap<String, u64>>,
}

/// Clear the current thread's tallies.
pub fn reset() {
    TALLY.with(|t| *t.borrow_mut() = Tally::default());
}

/// Take the current thread's tallies, leaving them empty.
pub fn take() -> Counters {
    TALLY.with(|t| {
        let t = std::mem::take(&mut *t.borrow_mut());
        Counters { totals: t.totals, by_kind: t.by_kind }
    })
}

fn add(kind: Option<u8>, name: &str, n: u64) {
    TALLY.with(|t| {
        let mut t = t.borrow_mut();
        *t.totals.entry(name.to_string()).or_insert(0) += n;
        if let Some(k) = kind {
            *t.by_kind.entry(k).or_default().entry(name.to_string()).or_insert(0) += n;
        }
    });
}

/// Add `n` to the counter `name`, attributed to the current kind scope.
pub fn bump(name: &str, n: u64) {
    add(KIND.with(|k| k.get()), name, n);
}

/// Keeps a transaction kind (or [`BLOCK`], [`SETTLE`], [`BOUNDARY`]) in
/// force for the counters bumped while it lives.
pub struct KindGuard(Option<u8>);

pub fn kind(k: u8) -> KindGuard {
    KindGuard(KIND.with(|c| c.replace(Some(k))))
}

impl Drop for KindGuard {
    fn drop(&mut self) {
        KIND.with(|c| c.set(self.0));
    }
}

/// Forces the purpose of every hash computed while it lives (the block id
/// is computed by the generic `hash32`).
pub struct PurposeGuard(Option<Purpose>);

pub fn purpose(p: Purpose) -> PurposeGuard {
    PurposeGuard(PURPOSE.with(|c| c.replace(Some(p))))
}

impl Drop for PurposeGuard {
    fn drop(&mut self) {
        PURPOSE.with(|c| c.set(self.0));
    }
}

fn ident(parts: &[&[u8]]) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for p in parts {
        p.hash(&mut h);
    }
    h.finish()
}

/// A hemera hash of the concatenation of `parts`, attributed to `p` (or to
/// the forced purpose) and to the current kind scope.
pub fn hash(p: Purpose, parts: &[&[u8]]) {
    hash_k(p, None, parts);
}

/// As [`hash`] with an explicit transaction kind byte (the txid, statement
/// and Merkle leaf of a transaction know theirs from `bytes[1]`).
pub fn hash_k(p: Purpose, kind: Option<u8>, parts: &[&[u8]]) {
    let p = PURPOSE.with(|c| c.get()).unwrap_or(p);
    let kind = kind.or_else(|| KIND.with(|k| k.get()));
    let len: usize = parts.iter().map(|x| x.len()).sum();
    let n = perms(len);
    let fresh = TALLY.with(|t| t.borrow_mut().seen.insert((p as u8, ident(parts))));
    let name = p.name();
    add(kind, &format!("hash.{name}.calls"), 1);
    add(kind, &format!("hash.{name}.perms"), n);
    if fresh {
        add(kind, &format!("hash.{name}.uniq_calls"), 1);
        add(kind, &format!("hash.{name}.uniq_perms"), n);
    }
}

/// `n` permutations of a given purpose that happen inside code soft3 does
/// not own (bbg, mudra); `name` says which formula derived them.
pub fn derived(name: &str, perms: u64) {
    bump(&format!("derived.{name}"), perms);
}

// ── the off-block-path settlement worker ────────────────────────────────

static JOBS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Name the chain whose worker this thread is (the store directory), so a
/// census can tell the validator's solves from the miner's.
pub fn set_tag(tag: String) {
    TAG.with(|t| *t.borrow_mut() = tag);
}

pub fn tag() -> String {
    TAG.with(|t| t.borrow().clone())
}

/// Start a settlement job on this thread: the thread's tally is set aside
/// (an inline job runs on the block thread) and a fresh one counts the
/// job's own hashing.
pub fn job_begin(tag_: String) -> Tally {
    set_tag(tag_);
    TALLY.with(|t| std::mem::take(&mut *t.borrow_mut()))
}

/// End the job: its counters become one note, the thread's tally returns.
pub fn job_end(saved: Tally, epoch: u64, millis: u64) {
    let mine = TALLY.with(|t| std::mem::replace(&mut *t.borrow_mut(), saved));
    let totals = mine.totals.iter().map(|(k, v)| format!("{k:?}:{v}")).collect::<Vec<_>>().join(",");
    job_note(format!("{{\"tag\":{:?},\"epoch\":{epoch},\"call\":\"job\",\"millis\":{millis},\"counters\":{{{totals}}}}}", tag()));
}

/// Record one engine call of a settlement (`focus` or `gradient`): the
/// graph's particles and edges, and the milliseconds it took.
pub fn solve(kind: &str, epoch: u64, particles: usize, edges: usize, millis: u64) {
    let line = format!(
        "{{\"tag\":{:?},\"epoch\":{epoch},\"call\":{kind:?},\"particles\":{particles},\"edges\":{edges},\"millis\":{millis}}}",
        tag()
    );
    JOBS.lock().expect("jobs").push(line);
}

/// Record a free-form fact about a settlement job (input sizes, hashes done
/// off the block path).
pub fn job_note(line: String) {
    JOBS.lock().expect("jobs").push(line);
}

/// Everything the settlement workers recorded, drained.
pub fn take_jobs() -> Vec<String> {
    std::mem::take(&mut *JOBS.lock().expect("jobs"))
}

/// The φ* engine of a settlement, counting every call it is asked for.
pub struct Counting<'a> {
    inner: &'a dyn super::economy::focus::FocusEngine,
    epoch: u64,
}

impl<'a> Counting<'a> {
    pub fn new(inner: &'a dyn super::economy::focus::FocusEngine, epoch: u64) -> Self {
        Counting { inner, epoch }
    }
}

impl super::economy::focus::FocusEngine for Counting<'_> {
    fn id(&self) -> &'static str {
        self.inner.id()
    }

    fn focus(&self, g: &super::economy::focus::FocusGraph) -> super::economy::focus::Focus {
        let t0 = std::time::Instant::now();
        let out = self.inner.focus(g);
        solve("focus", self.epoch, g.n(), g.edges(), t0.elapsed().as_millis() as u64);
        out
    }

    fn gradient(&self, g: &super::economy::focus::FocusGraph) -> Vec<i128> {
        let t0 = std::time::Instant::now();
        let out = self.inner.gradient(g);
        solve("gradient", self.epoch, g.n(), g.edges(), t0.elapsed().as_millis() as u64);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hemera::field::Goldilocks;

    /// hemera's flat sponge replayed over the public permutation, counting
    /// its calls: 7-byte little-endian limbs, rate 8, zero state, padding
    /// `0x01`, message length in lane 10, four output lanes.
    fn replay(msg: &[u8]) -> ([u8; 32], u64) {
        let mut s = [Goldilocks::new(0); 16];
        let mut n = 0u64;
        let limbs = |block: &[u8; RATE_BYTES]| -> [u64; 8] {
            core::array::from_fn(|i| {
                let mut w = [0u8; 8];
                w[..7].copy_from_slice(&block[i * 7..i * 7 + 7]);
                u64::from_le_bytes(w)
            })
        };
        let full = msg.len() / RATE_BYTES;
        for b in 0..full {
            let block: [u8; RATE_BYTES] = msg[b * RATE_BYTES..(b + 1) * RATE_BYTES].try_into().unwrap();
            for (l, v) in s.iter_mut().zip(limbs(&block)) {
                *l += Goldilocks::new(v);
            }
            hemera::permutation::permute(&mut s);
            n += 1;
        }
        let rem = &msg[full * RATE_BYTES..];
        let mut padded = [0u8; RATE_BYTES];
        padded[..rem.len()].copy_from_slice(rem);
        padded[rem.len()] = 0x01;
        for (l, v) in s.iter_mut().zip(limbs(&padded)) {
            *l += Goldilocks::new(v);
        }
        s[10] = Goldilocks::new(msg.len() as u64);
        hemera::permutation::permute(&mut s);
        n += 1;
        let mut out = [0u8; 32];
        for i in 0..4 {
            out[i * 8..i * 8 + 8].copy_from_slice(&s[i].as_canonical_u64().to_le_bytes());
        }
        (out, n)
    }

    /// The permutation formula is the sponge: the replay (which counts its
    /// permutations as it makes them) reproduces `hemera::hash` for every
    /// length, and the count equals `perms(len)`.
    #[test]
    fn census_perms_formula_is_the_hemera_sponge() {
        for len in 0..=400usize {
            let msg: Vec<u8> = (0..len).map(|i| (i as u8).wrapping_mul(31).wrapping_add(7)).collect();
            let (digest, n) = replay(&msg);
            assert_eq!(&digest, hemera::hash(&msg).as_bytes(), "digest at {len}");
            assert_eq!(n, perms(len), "permutations at {len}");
        }
        // the known fact of mine/job.rs: a 168-byte header is four permutations
        assert_eq!(perms(crate::chain::params::HEADER_BYTES), 4);
    }
}
