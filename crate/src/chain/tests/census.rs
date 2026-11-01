//! P8 census (director plan row P8): what a block-validity relation for
//! rule set 1 would have to prove, counted on the native node. Mines a
//! deterministic range on the test harness and validates every block on a
//! second, fresh chain instance with a cold signature cache, so the counts
//! are a validator's, not the miner's (`accept_block`, specs/chain.md §5.5
//! steps 1–8). One line `P8 <json>` per block, `P8-JOB` per settlement
//! engine call, `P8-SUMMARY <json>` at the end. Feature `opcount` only.

use std::collections::BTreeMap;
use std::time::Instant;

use serde_json::{json, Value};

use super::*;
use crate::chain::api::{BlockVerdict, ChainApi};
use crate::chain::opcount::{self, Counters};
use crate::chain::params::{ATTR_NODES, EPOCH_LENGTH, MAX_BLOCK_TXS, PHI_STEPS, SETTLE_DELAY};
use crate::chain::state::StateBackend;

/// splitmix64, as in `simulate.rs`.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

const RICH: u64 = 1_000_000_000_000_000;
const ACCOUNTS: u8 = 24;
const HEIGHTS: u64 = 400;
const MAX_AT: u64 = 150;

fn particle(i: u64) -> Id {
    [i as u8; 32]
}

/// Transactions valid on `chain`'s tip: a mix of transfers, bare links,
/// staked links and payouts of positions that exist.
struct Gen {
    rng: Rng,
    /// valence of each position, fixed per `(author, p, q)`
    valence: BTreeMap<(usize, u64, u64), i8>,
    /// positions staked in earlier blocks: `(author, p, q)`
    staked: Vec<(usize, u64, u64)>,
    pending: Vec<(usize, u64, u64)>,
}

impl Gen {
    fn txs(&mut self, net: &Net, chain: &Chain, n: usize, simple: bool) -> Vec<Vec<u8>> {
        self.staked.append(&mut self.pending);
        let mut nonces: BTreeMap<usize, u64> = BTreeMap::new();
        let mut out = Vec::new();
        for _ in 0..n {
            let from = self.rng.below(net.neurons.len() as u64) as usize;
            let nonce = *nonces.entry(from).or_insert_with(|| chain.nonce(&net.neurons[from]));
            let p = self.rng.below(24);
            let q = (p + 1 + self.rng.below(23)) % 24;
            let roll = self.rng.below(10);
            let roll = if simple && roll >= 7 { 0 } else { roll };
            let tx = match roll {
                0..=4 => {
                    let to = net.neurons[self.rng.below(net.neurons.len() as u64) as usize];
                    net.transfer(from, nonce, to, 1 + self.rng.below(1_000_000), 224 + self.rng.below(50))
                }
                5 | 6 => net.link(from, nonce, particle(p), particle(q), 0, (self.rng.below(3) as i8) - 1, 257 + self.rng.below(50)),
                7 | 8 => {
                    let v = *self.valence.entry((from, p, q)).or_insert_with(|| (self.rng.below(3) as i8) - 1);
                    self.pending.push((from, p, q));
                    net.link(from, nonce, particle(p), particle(q), 1_000_000 + self.rng.below(9_000_000), v, 257 + self.rng.below(50))
                }
                _ => match self.staked.iter().filter(|c| c.0 == from).copied().next() {
                    Some((_, p, q)) => {
                        let to = net.neurons[self.rng.below(net.neurons.len() as u64) as usize];
                        net.payout(from, nonce, particle(p), particle(q), to, 248 + self.rng.below(50))
                    }
                    None => {
                        let to = net.neurons[self.rng.below(net.neurons.len() as u64) as usize];
                        net.transfer(from, nonce, to, 1 + self.rng.below(1_000_000), 224 + self.rng.below(50))
                    }
                },
            };
            nonces.insert(from, nonce + 1);
            out.push(tx);
        }
        out
    }
}

fn kind_name(k: u8) -> &'static str {
    match k {
        0 => "coinbase",
        1 => "transfer",
        2 => "cyberlink",
        3 => "claim",
        6 => "unlock",
        7 => "payout",
        opcount::BOUNDARY => "boundary-snapshot",
        opcount::SETTLE => "settlement-step",
        opcount::BLOCK => "block-level",
        _ => "unknown",
    }
}

/// One validated block: the body's shape and the validator's counters.
struct Rec {
    scenario: &'static str,
    height: u64,
    txs: usize,
    bytes: usize,
    kinds: BTreeMap<u8, (u64, u64)>,
    boundary: bool,
    settle: bool,
    max: bool,
    millis: u64,
    c: Counters,
    metrics: BTreeMap<String, u64>,
    sizes: Value,
}

fn g(c: &Counters, name: &str) -> u64 {
    c.totals.get(name).copied().unwrap_or(0)
}

const PURPOSES: [&str; 9] = ["header", "txid", "tx_root", "sig_statement", "sig_aux", "key", "journal", "economy", "other"];

/// The per-block numbers the summary takes min / median / max of: every
/// counter, plus the permutation groups.
fn metrics(c: &Counters, txs: usize, bytes: usize) -> BTreeMap<String, u64> {
    let mut m: BTreeMap<String, u64> = c.totals.clone();
    m.insert("block.txs".into(), txs as u64);
    m.insert("block.bytes".into(), bytes as u64);
    let sum_prefix = |prefix: &str| -> u64 { c.totals.iter().filter(|(k, _)| k.starts_with(prefix)).map(|(_, v)| *v).sum() };
    let leaf = sum_prefix("derived.trie_leaf.");
    let branch = sum_prefix("derived.trie_branch.");
    let fold = g(c, "derived.root_fold") + g(c, "derived.stats_commit");
    let bbg_misc = sum_prefix("derived.bbg_");
    let state = leaf + branch + fold + bbg_misc;
    m.insert("perms.state_derived".into(), state);
    m.insert("perms.state_derived.leaf".into(), leaf);
    m.insert("perms.state_derived.branch".into(), branch);
    m.insert("perms.state_derived.root_fold_and_stats".into(), fold);
    m.insert("perms.state_derived.bbg_key_and_member_hashes".into(), bbg_misc);
    let mut exec = state;
    let mut uniq = state;
    for p in PURPOSES {
        let e = g(c, &format!("hash.{p}.perms"));
        let u = g(c, &format!("hash.{p}.uniq_perms"));
        m.insert(format!("perms.{p}"), e);
        m.insert(format!("perms_uniq.{p}"), u);
        exec += e;
        if p != "journal" {
            uniq += u;
        }
    }
    m.insert("perms.total_executed".into(), exec);
    m.insert("perms.total_unique_no_journal".into(), uniq);
    m
}

fn stat(v: &mut Vec<u64>) -> Value {
    v.sort_unstable();
    let n = v.len();
    if n == 0 {
        return Value::Null;
    }
    let sum: u128 = v.iter().map(|x| *x as u128).sum();
    json!({"min": v[0], "median": v[n / 2], "max": v[n - 1], "mean": sum as f64 / n as f64})
}

fn summarize(recs: &[&Rec]) -> Value {
    let mut names: Vec<String> = recs.iter().flat_map(|r| r.metrics.keys().cloned()).collect();
    names.sort();
    names.dedup();
    let mut out = serde_json::Map::new();
    for name in names {
        let mut v: Vec<u64> = recs.iter().map(|r| r.metrics.get(&name).copied().unwrap_or(0)).collect();
        out.insert(name, stat(&mut v));
    }
    Value::Object(out)
}

/// Per kind over `recs`: transactions, bytes, and every counter attributed.
fn per_kind(recs: &[&Rec]) -> Value {
    let mut counters: BTreeMap<u8, BTreeMap<String, u64>> = BTreeMap::new();
    let mut shape: BTreeMap<u8, (u64, u64)> = BTreeMap::new();
    for r in recs {
        for (k, (n, b)) in &r.kinds {
            let e = shape.entry(*k).or_default();
            e.0 += n;
            e.1 += b;
        }
        for (k, m) in &r.c.by_kind {
            let e = counters.entry(*k).or_default();
            for (name, v) in m {
                *e.entry(name.clone()).or_insert(0) += v;
            }
        }
    }
    let mut keys: Vec<u8> = shape.keys().chain(counters.keys()).copied().collect();
    keys.sort_unstable();
    keys.dedup();
    let mut out = serde_json::Map::new();
    for k in keys {
        let (n, b) = shape.get(&k).copied().unwrap_or((0, 0));
        let c = counters.get(&k).cloned().unwrap_or_default();
        out.insert(format!("{k}:{}", kind_name(k)), json!({"txs": n, "bytes": b, "counters": c}));
    }
    Value::Object(out)
}

/// Least-squares line `y = a + b·x`.
fn fit(points: &[(f64, f64)]) -> Value {
    let n = points.len() as f64;
    if points.len() < 2 {
        return Value::Null;
    }
    let (sx, sy) = points.iter().fold((0.0, 0.0), |(a, b), (x, y)| (a + x, b + y));
    let (mx, my) = (sx / n, sy / n);
    let sxx: f64 = points.iter().map(|(x, _)| (x - mx) * (x - mx)).sum();
    let sxy: f64 = points.iter().map(|(x, y)| (x - mx) * (y - my)).sum();
    if sxx == 0.0 {
        return Value::Null;
    }
    let b = sxy / sxx;
    json!({"intercept": my - b * mx, "slope_per_tx": b, "points": points.len()})
}

fn sizes(c: &Chain) -> Value {
    let b = &c.state().bbg;
    json!({
        "balances": b.balances.len(), "neurons": b.neurons.len(), "cards": b.cards.len(),
        "axons_out": b.axons_out.len(), "axons_in": b.axons_in.len(), "time": b.time.len(), "coins": b.coins.len(),
        "particles": b.particles.len(),
    })
}

/// Validate `block` on `v` with a cold signature cache and return what that cost.
fn validate(scenario: &'static str, v: &mut Chain, block: &Block, max: bool) -> Rec {
    let height = block.header.height;
    let raws = crate::chain::tx::split_body(&block.body).unwrap();
    let mut kinds: BTreeMap<u8, (u64, u64)> = BTreeMap::new();
    for raw in &raws {
        let e = kinds.entry(raw[1]).or_default();
        e.0 += 1;
        e.1 += raw.len() as u64;
    }
    let clock = v.rules().clock;
    sigs::clear_verified_cache();
    opcount::reset();
    let t0 = Instant::now();
    let verdict = v.accept_block(block, NOW).unwrap();
    let millis = t0.elapsed().as_millis() as u64;
    let c = opcount::take();
    assert!(matches!(verdict, BlockVerdict::Extended), "{verdict:?}");
    // every header hash is the 168-byte header: four permutations each
    assert_eq!(g(&c, "hash.header.perms"), 4 * g(&c, "hash.header.calls"), "header hash length at {height}");
    let metrics = metrics(&c, raws.len(), block.body.len());
    Rec {
        scenario,
        height,
        txs: raws.len(),
        bytes: block.body.len(),
        kinds,
        boundary: clock.is_boundary(height),
        settle: clock.settles_at(height).is_some(),
        max,
        millis,
        c,
        metrics,
        sizes: sizes(v),
    }
}

fn emit(r: &Rec) {
    let detail = r.boundary || r.settle || r.max || r.height <= 3;
    let kinds: BTreeMap<String, Value> = r.kinds.iter().map(|(k, (n, b))| (format!("{k}:{}", kind_name(*k)), json!({"txs": n, "bytes": b}))).collect();
    let mut line = json!({
        "scenario": r.scenario, "height": r.height, "txs": r.txs, "body_bytes": r.bytes, "kinds": kinds,
        "boundary": r.boundary, "settle": r.settle, "max": r.max, "millis": r.millis,
        "sizes": r.sizes, "counters": r.c.totals, "metrics": {
            "perms_total_executed": r.metrics["perms.total_executed"],
            "perms_total_unique_no_journal": r.metrics["perms.total_unique_no_journal"],
            "perms_state_derived": r.metrics["perms.state_derived"],
        },
    });
    if detail {
        let by_kind: BTreeMap<String, &BTreeMap<String, u64>> = r.c.by_kind.iter().map(|(k, m)| (format!("{k}:{}", kind_name(*k)), m)).collect();
        line["by_kind"] = json!(by_kind);
    }
    println!("P8 {line}");
}

fn drain_jobs() {
    for line in opcount::take_jobs() {
        println!("P8-JOB {line}");
    }
}

/// The epoch outcome the validator recorded at `height`, if any.
fn print_outcome(label: &str, v: &Chain, height: u64) {
    let Some(h) = v.header_at(height) else { return };
    if let Some(o) = v.economy.outcome(&h.id()) {
        let r = &o.record;
        let scalars = v.state().scalars();
        println!(
            "P8-OUTCOME {}",
            json!({
                "scenario": label, "height": height, "epoch": r.epoch, "boundary": r.boundary, "particles": r.particles, "axons": r.axons,
                "focus_bps": r.focus_bps, "budget": r.budget, "paid": r.paid, "pool_after": r.pool_after, "rewarded": r.rewarded,
                "payouts_listed": o.payouts.len(), "phi_len": o.focus.phi.len(),
                "total_stake": scalars.total_stake, "active_stake": scalars.active_stake, "last_settled": scalars.last_settled,
            })
        );
    }
}

/// The main range: 400 blocks on the spec's epoch clock, one block of the
/// maximum transaction count, through the first boundary (240) and the
/// first settlement (360).
fn main_range() -> Vec<Rec> {
    let net = Net::with_funds(ACCOUNTS, RICH);
    let mut miner = net.open("miner");
    let mut validator = net.open("validator");
    let mut gen = Gen { rng: Rng(8), valence: BTreeMap::new(), staked: Vec::new(), pending: Vec::new() };
    let mut recs = Vec::new();
    for h in 1..=HEIGHTS {
        let max = h == MAX_AT;
        let n = if max { MAX_BLOCK_TXS - 1 } else { gen.rng.below(17) as usize };
        let mut txs = gen.txs(&net, &miner, n, max);
        if h == 5 {
            // the dormant legacy owner claims (kind 3)
            txs.push(net.claim(net.dormant_key as u8, 0, 140 + 10));
        }
        let to = net.neurons[gen.rng.below(ACCOUNTS as u64) as usize];
        let block = mine(&mut miner, to, 0, &txs);
        if max {
            let count = crate::chain::tx::split_body(&block.body).unwrap().len();
            println!("P8-MAX {}", json!({"height": h, "txs_in_body": count, "body_bytes": block.body.len(), "max_block_txs": MAX_BLOCK_TXS}));
            assert_eq!(count, MAX_BLOCK_TXS, "the maximum-count block");
        }
        let rec = validate("main", &mut validator, &block, max);
        emit(&rec);
        if rec.boundary || rec.settle {
            print_outcome("main", &validator, h);
        }
        if rec.settle {
            drain_jobs();
        }
        recs.push(rec);
    }
    assert_eq!(validator.tip(), miner.tip());
    recs
}

/// A short clock (L = 10, D = 5, MIN_LOCK = 15): kinds 3, 6 and 7 and
/// several boundaries and settlements, for the kinds the long range cannot
/// reach (an unlock needs MIN_LOCK = 20,160 blocks on the spec's clock).
fn short_clock() -> Vec<Rec> {
    let mut net = Net::with_funds(3, RICH);
    net.clock = Some(EpochClock { length: 10, delay: 5, min_lock: 15 });
    let mut miner = net.open("miner-short");
    let mut validator = net.open("validator-short");
    let (x, y) = (particle(10), particle(11));
    let mut recs = Vec::new();
    for h in 1..=45u64 {
        let nonce = |m: &Chain, i: usize| m.nonce(&net.neurons[i]);
        let mut txs: Vec<Vec<u8>> = Vec::new();
        match h {
            1 => {
                txs.push(net.link(0, nonce(&miner, 0), x, y, RICH / 1_000, 1, 257));
                txs.push(net.link(1, nonce(&miner, 1), x, y, RICH / 2_000, -1, 257));
                txs.push(net.transfer(2, nonce(&miner, 2), net.neurons[0], 1_000, 224));
            }
            2 => {
                txs.push(net.link(2, nonce(&miner, 2), particle(12), particle(13), 0, 0, 257));
                txs.push(net.claim(net.dormant_key as u8, 0, 140));
            }
            3 => txs.push(net.payout(0, nonce(&miner, 0), x, y, net.neurons[2], 248)),
            12 => {
                txs.push(net.link(0, nonce(&miner, 0), x, y, RICH / 1_000, 1, 257));
                txs.push(net.link(1, nonce(&miner, 1), x, y, RICH / 2_000, -1, 257));
            }
            27 => {
                txs.push(net.unlock(0, nonce(&miner, 0), x, y, RICH / 4_000, 224));
                txs.push(net.unlock(1, nonce(&miner, 1), x, y, RICH / 2_000, 224));
            }
            _ => {}
        }
        let block = mine(&mut miner, net.neurons[2], 0, &txs);
        let rec = validate("short-clock", &mut validator, &block, false);
        emit(&rec);
        if rec.boundary || rec.settle {
            print_outcome("short-clock", &validator, h);
        }
        recs.push(rec);
    }
    drain_jobs();
    recs
}

#[test]
fn census() {
    println!(
        "P8-META {}",
        json!({
            "hemera_rate_bytes": opcount::RATE_BYTES, "perms_168": opcount::perms(168), "perms_64": opcount::perms(64),
            "epoch_length": EPOCH_LENGTH, "settle_delay": SETTLE_DELAY, "phi_steps": PHI_STEPS, "attr_nodes": ATTR_NODES,
            "max_block_txs": MAX_BLOCK_TXS, "heights": HEIGHTS, "accounts": ACCOUNTS, "max_at": MAX_AT,
            "bbg_rev": "2d7873c739bb799f6c9a48c566c941df39de79d8", "hemera_rev": "8559b52f480ebb6fdd0a25dec16ba27747da9cab",
        })
    );
    assert_eq!(opcount::perms(168), 4);
    let main = main_range();
    let short = short_clock();

    let ordinary: Vec<&Rec> = main.iter().filter(|r| !r.boundary && !r.settle && !r.max).collect();
    let keep = ["perms.total_executed", "perms.total_unique_no_journal", "perms.state_derived", "perms.header", "perms.txid", "perms.tx_root", "perms.sig_statement", "perms.sig_aux", "perms.key", "perms.economy", "perms.other", "sig.verify", "sig.cache_hit", "block.txs", "block.bytes"];
    let mut special: Vec<Value> = Vec::new();
    for r in main.iter().filter(|r| r.boundary || r.settle || r.max) {
        let m: BTreeMap<&str, u64> = keep.iter().map(|k| (*k, r.metrics.get(*k).copied().unwrap_or(0))).collect();
        let by_kind: BTreeMap<String, BTreeMap<String, u64>> = r.c.by_kind.iter().map(|(k, v)| (format!("{k}:{}", kind_name(*k)), v.clone())).collect();
        special.push(json!({"height": r.height, "boundary": r.boundary, "settle": r.settle, "max": r.max, "metrics": m, "counters": r.c.totals, "by_kind": by_kind}));
    }
    let fit_points: Vec<(f64, f64)> = ordinary.iter().map(|r| (r.txs as f64, r.metrics["perms.total_unique_no_journal"] as f64)).collect();
    let mismatch: u64 = main.iter().chain(short.iter()).map(|r| g(&r.c, "state.pending_mismatch")).sum();
    let all_main: Vec<&Rec> = main.iter().collect();
    let all_short: Vec<&Rec> = short.iter().collect();
    let summary = json!({
        "main": {
            "blocks": main.len(), "ordinary_blocks": ordinary.len(),
            "ordinary_per_block": summarize(&ordinary),
            "fit_unique_perms_vs_txs_ordinary": fit(&fit_points),
            "special_blocks": special,
            "per_kind_all_blocks": per_kind(&all_main),
        },
        "short_clock": { "blocks": short.len(), "per_kind_all_blocks": per_kind(&all_short) },
        "pending_mismatch_blocks_total": mismatch,
        "millis_validate_main": main.iter().map(|r| r.millis).sum::<u64>(),
    });
    println!("P8-SUMMARY {summary}");
    drain_jobs();
}
