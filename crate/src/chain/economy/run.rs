//! Settlement off the block path (chain-economy §2.5, §7.2).
//!
//! At every boundary block the node freezes the boundary state
//! ([`Snapshot`]) and starts the epoch's three solves on a worker thread;
//! the D blocks until `settle(E)` are produced and validated meanwhile. The
//! block at `settle(E)` takes the finished result — joining the worker if
//! it is still running, which is the liveness bound of §2.5: a node that
//! has not finished waits, it never guesses.
//!
//! Everything is keyed by block ids: a snapshot by its boundary block, a
//! result by the pair of boundary blocks it reads. A reorganization that
//! replaces a boundary simply asks for another key; a reorganization above
//! it finds the same result. Results are also kept on disk under
//! `$home/epochs/` (the cache, keyed by the engine id too), so a restart
//! replays the chain without recomputing every epoch.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Instant;

use super::super::params::EpochClock;
use super::super::state::{Id, StateBackend};
use super::focus::{FocusEngine, FocusGraph};
use super::settle::{Earner, FocusOut, Outcome, SettleResult, Snapshot, compute_with};
use super::weights::GenesisWeights;
use super::wide::W;
use super::Mark;

type Key = (Id, Id);

/// Timing of one computed settlement (informational, never consensus).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Timing {
    pub epoch: u64,
    pub millis: u64,
    pub particles: u32,
    pub axons: u32,
}

pub struct Economy {
    pub weights: Arc<GenesisWeights>,
    pub engine: Arc<dyn FocusEngine>,
    pub clock: EpochClock,
    /// compute on worker threads (the node); `false` computes inline
    pub background: bool,
    /// gradient nodes on the epoch's path (1 = the midpoint, chain-economy
    /// §5.1); part of every cached result's key
    pub quadrature: usize,
    snapshots: HashMap<Id, Arc<Snapshot>>,
    jobs: HashMap<Key, JoinHandle<(SettleResult, u64)>>,
    done: HashMap<Key, Arc<SettleResult>>,
    /// `J(φ*)` per boundary block id, for the next epoch's Δ (§5.6)
    syntropy: Arc<Mutex<HashMap<Id, i128>>>,
    /// the settlement outcomes applied, by the settlement block's id
    outcomes: HashMap<Id, Arc<Outcome>>,
    timings: Arc<Mutex<Vec<Timing>>>,
    dir: Option<PathBuf>,
}

impl Economy {
    pub fn new(weights: GenesisWeights, engine: Arc<dyn FocusEngine>, clock: EpochClock, dir: Option<PathBuf>) -> Economy {
        if let Some(d) = &dir {
            let _ = std::fs::create_dir_all(d);
        }
        Economy {
            weights: Arc::new(weights),
            engine,
            clock,
            background: true,
            quadrature: super::super::params::ATTR_NODES,
            snapshots: HashMap::new(),
            jobs: HashMap::new(),
            done: HashMap::new(),
            syntropy: Arc::new(Mutex::new(HashMap::new())),
            outcomes: HashMap::new(),
            timings: Arc::new(Mutex::new(Vec::new())),
            dir,
        }
    }

    pub fn snapshot(&self, block: &Id) -> Option<Arc<Snapshot>> {
        self.snapshots.get(block).cloned()
    }

    /// Freeze the state after the boundary block `block` of `epoch` and,
    /// when the previous boundary's snapshot is held, start the epoch's
    /// settlement. `prev` is the previous boundary's block id (none at
    /// genesis).
    pub fn on_boundary(&mut self, state: &impl StateBackend, epoch: u64, block: Id, root: Id, supply: u64, prev: Option<Id>) {
        if !self.snapshots.contains_key(&block) {
            let snap = Snapshot::capture(state, epoch, block, root, supply);
            self.snapshots.insert(block, Arc::new(snap));
        }
        if let Some(prev) = prev {
            self.start(prev, block);
        }
    }

    fn cache_path(&self, key: &Key) -> Option<PathBuf> {
        let dir = self.dir.as_ref()?;
        let mut buf = self.engine.id().as_bytes().to_vec();
        buf.extend_from_slice(&(self.quadrature as u32).to_le_bytes());
        buf.extend_from_slice(&key.0);
        buf.extend_from_slice(&key.1);
        Some(dir.join(format!("{}.settle", hex(hemera::hash(&buf).as_bytes()))))
    }

    fn load(&mut self, key: &Key) -> Option<Arc<SettleResult>> {
        let path = self.cache_path(key)?;
        let bytes = std::fs::read(path).ok()?;
        let r = Arc::new(decode(&bytes)?);
        self.syntropy.lock().expect("syntropy map").insert(key.1, r.syntropy);
        self.done.insert(*key, r.clone());
        Some(r)
    }

    /// Start the settlement reading boundaries `prev` and `cur`, unless it
    /// is done, running or cached.
    pub fn start(&mut self, prev: Id, cur: Id) {
        let key = (prev, cur);
        if self.done.contains_key(&key) || self.jobs.contains_key(&key) || self.load(&key).is_some() {
            return;
        }
        let (Some(a), Some(b)) = (self.snapshot(&prev), self.snapshot(&cur)) else { return };
        let (weights, engine, syntropy, timings) = (self.weights.clone(), self.engine.clone(), self.syntropy.clone(), self.timings.clone());
        let path = self.cache_path(&key);
        let epoch = b.epoch;
        let settle_height = self.clock.settle(epoch);
        let quadrature = self.quadrature;
        let job = move || {
            let t0 = Instant::now();
            // The compute runs outside every lock guard, so a panic in the
            // engine poisons nothing; it is re-raised naming the epoch and
            // the height at which the settlement is due.
            let computed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let prev_j = syntropy.lock().expect("syntropy map").get(&a.block_id).copied();
                let prev_j = prev_j.unwrap_or_else(|| {
                    let j = engine.focus(&FocusGraph::build(&a.adjacency(&weights))).syntropy;
                    syntropy.lock().expect("syntropy map").insert(a.block_id, j);
                    j
                });
                compute_with(&a, &b, &weights, engine.as_ref(), Some(prev_j), quadrature)
            }));
            let r = match computed {
                Ok(r) => r,
                Err(payload) => {
                    let original = payload
                        .downcast_ref::<&str>()
                        .map(|s| s.to_string())
                        .or_else(|| payload.downcast_ref::<String>().cloned())
                        .unwrap_or_else(|| "<non-string panic payload>".to_string());
                    let line = format!("settlement of epoch {epoch} at height {settle_height} panicked: {original}");
                    eprintln!("{line}");
                    panic!("{line}");
                }
            };
            let millis = t0.elapsed().as_millis() as u64;
            syntropy.lock().expect("syntropy map").insert(b.block_id, r.syntropy);
            timings.lock().expect("timings").push(Timing { epoch: r.epoch, millis, particles: r.particles, axons: r.axons });
            if let Some(p) = path {
                let tmp = p.with_extension("tmp");
                if std::fs::write(&tmp, encode(&r)).is_ok() {
                    let _ = std::fs::rename(&tmp, &p);
                }
            }
            (r, millis)
        };
        if self.background {
            let handle = std::thread::Builder::new().name(format!("settle-{epoch}")).spawn(job).expect("spawn the settlement worker");
            self.jobs.insert(key, handle);
        } else {
            let (r, _) = job();
            self.done.insert(key, Arc::new(r));
        }
    }

    /// Is the settlement of `(prev, cur)` finished (or cached)?
    pub fn ready(&self, prev: &Id, cur: &Id) -> bool {
        let key = (*prev, *cur);
        self.done.contains_key(&key) || self.jobs.get(&key).is_some_and(|j| j.is_finished())
    }

    /// The settlement reading boundaries `prev` and `cur`: finished, joined
    /// (waiting for the worker), or computed now. `None` only when a
    /// snapshot is missing — the boundary was never applied on this node.
    ///
    /// A panic in the settlement is a halt by design (chain-economy §2.5
    /// liveness; plan P2, decision D14): it is re-raised with the epoch and
    /// height named, nothing is recorded, a replay halts at the same block.
    pub fn result(&mut self, prev: &Id, cur: &Id) -> Option<Arc<SettleResult>> {
        let key = (*prev, *cur);
        if let Some(r) = self.done.get(&key) {
            return Some(r.clone());
        }
        if let Some(job) = self.jobs.remove(&key) {
            let (r, _) = match job.join() {
                Ok(done) => done,
                Err(payload) => std::panic::resume_unwind(payload),
            };
            let r = Arc::new(r);
            self.done.insert(key, r.clone());
            return Some(r);
        }
        if let Some(r) = self.load(&key) {
            return Some(r);
        }
        let background = std::mem::replace(&mut self.background, false);
        self.start(*prev, *cur);
        self.background = background;
        self.done.get(&key).cloned()
    }

    pub fn record_outcome(&mut self, block: Id, outcome: Outcome) {
        self.outcomes.insert(block, Arc::new(outcome));
    }

    pub fn outcome(&self, block: &Id) -> Option<Arc<Outcome>> {
        self.outcomes.get(block).cloned()
    }

    pub fn timings(&self) -> Vec<Timing> {
        self.timings.lock().expect("timings").clone()
    }

    /// Drop snapshots, results and outcomes of boundaries more than three
    /// epochs below `tip` (a reorg never reaches them, §5.2). The genesis
    /// snapshot stays.
    pub fn prune(&mut self, tip: u64) {
        let floor = tip.saturating_sub(3 * self.clock.length + self.clock.delay);
        self.snapshots.retain(|_, s| s.height == 0 || s.height >= floor);
        self.done.retain(|_, r| r.boundary >= floor);
        // records and payout lists stay (served by /epoch); φ* vectors of
        // all but the recent epochs go
        for o in self.outcomes.values_mut() {
            if o.record.boundary < floor && !o.focus.phi.is_empty() {
                let mut slim = (**o).clone();
                slim.focus = Arc::new(FocusOut { particles: Vec::new(), phi: Vec::new(), syntropy: o.focus.syntropy, digest: o.focus.digest });
                *o = Arc::new(slim);
            }
        }
        let keep: std::collections::HashSet<Id> = self.snapshots.keys().copied().collect();
        self.syntropy.lock().expect("syntropy map").retain(|id, _| keep.contains(id));
    }
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

// ── the cache file: one settlement result without φ*'s vector ───────────

const MAGIC: &[u8; 8] = b"S3SETL01";

fn encode(r: &SettleResult) -> Vec<u8> {
    let mut out = MAGIC.to_vec();
    for v in [r.epoch, r.boundary, r.focus_bps] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    for id in [&r.prev_block, &r.beacon, &r.boundary_root, &r.phi] {
        out.extend_from_slice(id);
    }
    out.extend_from_slice(&r.particles.to_le_bytes());
    out.extend_from_slice(&r.axons.to_le_bytes());
    out.extend_from_slice(&r.syntropy.to_le_bytes());
    out.extend_from_slice(&r.prev_syntropy.to_le_bytes());
    out.extend_from_slice(&(r.earners.len() as u64).to_le_bytes());
    for e in &r.earners {
        out.extend_from_slice(&e.card);
        out.extend_from_slice(&e.owner);
        for l in e.x.0 {
            out.extend_from_slice(&l.to_le_bytes());
        }
        out.extend_from_slice(&e.cap.to_le_bytes());
    }
    out.extend_from_slice(&(r.marks.len() as u64).to_le_bytes());
    for (a, m) in &r.marks {
        out.extend_from_slice(a);
        out.extend_from_slice(&m.pos.to_le_bytes());
        out.extend_from_slice(&m.neg.to_le_bytes());
    }
    out
}

fn decode(b: &[u8]) -> Option<SettleResult> {
    let mut at = 0usize;
    let mut take = |n: usize| -> Option<&[u8]> {
        let s = b.get(at..at + n)?;
        at += n;
        Some(s)
    };
    if take(8)? != MAGIC {
        return None;
    }
    let u64_ = |s: &[u8]| u64::from_le_bytes(s.try_into().expect("8"));
    let (epoch, boundary, focus_bps) = (u64_(take(8)?), u64_(take(8)?), u64_(take(8)?));
    let id = |s: &[u8]| -> Id { s.try_into().expect("32") };
    let (prev_block, beacon, boundary_root, phi) = (id(take(32)?), id(take(32)?), id(take(32)?), id(take(32)?));
    let particles = u32::from_le_bytes(take(4)?.try_into().ok()?);
    let axons = u32::from_le_bytes(take(4)?.try_into().ok()?);
    let syntropy = i128::from_le_bytes(take(16)?.try_into().ok()?);
    let prev_syntropy = i128::from_le_bytes(take(16)?.try_into().ok()?);
    let n = u64_(take(8)?) as usize;
    let mut earners = Vec::with_capacity(n.min(1 << 20));
    for _ in 0..n {
        let card = id(take(32)?);
        let owner = id(take(32)?);
        let mut x = [0u64; 8];
        for l in x.iter_mut() {
            *l = u64_(take(8)?);
        }
        let cap = u64_(take(8)?);
        earners.push(Earner { card, owner, x: W(x), cap });
    }
    let m = u64_(take(8)?) as usize;
    let mut marks = Vec::with_capacity(m.min(1 << 20));
    for _ in 0..m {
        let a = id(take(32)?);
        marks.push((a, Mark { pos: u64_(take(8)?), neg: u64_(take(8)?) }));
    }
    if at != b.len() {
        return None;
    }
    Some(SettleResult {
        epoch,
        prev_block,
        boundary,
        beacon,
        boundary_root,
        particles,
        axons,
        phi,
        syntropy,
        prev_syntropy,
        focus_bps,
        earners,
        marks,
        focus: Arc::new(FocusOut { particles: Vec::new(), phi: Vec::new(), syntropy, digest: phi }),
    })
}

#[cfg(test)]
pub(crate) fn round_trip(r: &SettleResult) -> Option<SettleResult> {
    decode(&encode(r))
}
