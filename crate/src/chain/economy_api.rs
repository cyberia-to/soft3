//! The economy's read surface on the engine (chain-economy §11): epoch
//! records and payout lists, φ* of the last settled epoch, positions, and
//! the `/status` fields with the per-settlement checks of §6.3. JSON
//! values, so the HTTP layer only routes.

use serde_json::{Value, json};

use super::economy::{focus_key, stake_key};
use super::engine::Chain;
use super::state::{Id, StateBackend};

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

impl Chain {
    /// The record of epoch `e` on the main chain, its digest and payouts.
    pub fn epoch_json(&self, e: u64) -> Option<Value> {
        let h = self.rules.clock.settle(e);
        let id = self.store.main_at(h)?;
        let o = self.economy.outcome(&id)?;
        let r = &o.record;
        Some(json!({
            "epoch": r.epoch, "boundary": r.boundary, "settled_at": h, "beacon": hex(&r.beacon), "boundary_root": hex(&r.boundary_root),
            "particles": r.particles, "axons": r.axons, "phi": hex(&r.phi), "syntropy": r.syntropy, "delta_syntropy": r.delta_syntropy,
            "focus_bps": r.focus_bps, "budget": r.budget, "paid": r.paid, "pool_after": r.pool_after, "rewarded": r.rewarded,
            "payouts_root": hex(&r.payouts), "digest": hex(&r.digest()), "record": hex(&r.encode()),
            "payouts": o.payouts.iter().map(|(c, owner, a)| json!({"card": hex(c), "owner": hex(owner), "amount": a})).collect::<Vec<_>>(),
        }))
    }

    /// Up to `limit` (≤ 500) records after epoch `after`.
    pub fn epochs_json(&self, after: u64, limit: usize) -> Value {
        let last = self.state.scalars().last_settled;
        let records: Vec<Value> = (after + 1..=last).take(limit.min(500)).filter_map(|e| self.epoch_json(e)).collect();
        json!({ "last_settled": last, "epochs": records })
    }

    /// φ* of the last settled epoch this node computed with its vector.
    fn last_focus(&self) -> Option<(u64, std::sync::Arc<super::economy::settle::Outcome>)> {
        let last = self.state.scalars().last_settled;
        (1..=last).rev().take(3).find_map(|e| {
            let id = self.store.main_at(self.rules.clock.settle(e))?;
            self.economy.outcome(&id).filter(|o| !o.focus.phi.is_empty()).map(|o| (e, o))
        })
    }

    /// φ*_E of `particle` at the last settled epoch: raw fixed point, the
    /// scale, and its rank (1 = highest).
    pub fn focus_json(&self, particle: &Id) -> Option<Value> {
        let (e, o) = self.last_focus()?;
        let f = &o.focus;
        let i = f.particles.binary_search(particle).ok()?;
        let v = f.phi[i];
        let rank = 1 + f.phi.iter().filter(|x| **x > v).count();
        Some(json!({"epoch": e, "particle": hex(particle), "phi": v, "scale_bits": 60, "rank": rank, "particles": f.particles.len()}))
    }

    /// The `n` particles of highest focus at the last settled epoch.
    pub fn focus_top_json(&self, n: usize) -> Option<Value> {
        let (e, o) = self.last_focus()?;
        let f = &o.focus;
        let mut idx: Vec<usize> = (0..f.phi.len()).collect();
        idx.sort_by(|a, b| f.phi[*b].cmp(&f.phi[*a]).then(f.particles[*a].cmp(&f.particles[*b])));
        let top: Vec<Value> = idx.into_iter().take(n.min(1_000)).map(|i| json!({"particle": hex(&f.particles[i]), "phi": f.phi[i]})).collect();
        Some(json!({"epoch": e, "scale_bits": 60, "syntropy": f.syntropy.to_string(), "digest": hex(&f.digest), "top": top}))
    }

    fn card_value(&self, id: &Id) -> Option<Value> {
        let c = self.state.card(id)?;
        let (p, q) = self.state.axon_edge(&c.particle).unwrap_or(([0; 32], [0; 32]));
        Some(json!({
            "card": hex(id), "owner": hex(&c.owner), "author": hex(&c.author), "axon": hex(&c.particle), "p": hex(&p), "q": hex(&q),
            "valence": c.valence, "since": c.since, "unlock_at": c.since + self.rules.clock.min_lock, "stake": self.state.raw_balance(&stake_key(id)),
        }))
    }

    pub fn card_json(&self, id: &Id) -> Option<Value> {
        self.card_value(id)
    }

    /// Every position `neuron` authored (a scan; node-side, not consensus).
    pub fn cards_json(&self, neuron: &Id) -> Value {
        let cards: Vec<Value> = self.state.cards().into_iter().filter(|(_, c)| c.author == *neuron).filter_map(|(id, _)| self.card_value(&id)).collect();
        json!({"neuron": hex(neuron), "cards": cards})
    }

    /// `/status` economy fields and the §6.3 checks.
    pub fn economy_status(&self) -> Value {
        let clock = self.rules.clock;
        let tip = self.store.tip_height();
        let s = self.state.scalars();
        let pool = self.state.raw_balance(&focus_key(&self.rules.native));
        let stake_keys_match = super::apply::check_stake_total(&self.state).is_ok();
        let last = (s.last_settled > 0).then(|| self.store.main_at(clock.settle(s.last_settled)).and_then(|id| self.economy.outcome(&id))).flatten();
        let settlement_ok = last.as_ref().is_none_or(|o| o.record.paid <= o.record.budget && o.record.budget <= o.record.paid + o.record.pool_after);
        json!({
            "epoch": clock.epoch(tip), "epoch_length": clock.length, "settle_delay": clock.delay, "min_lock": clock.min_lock,
            "next_boundary": (tip / clock.length + 1) * clock.length,
            "last_settled": s.last_settled, "focus_bps": s.focus_bps, "pool": pool, "active_stake": s.active_stake, "total_stake": s.total_stake,
            "stake_keys_match": stake_keys_match, "settlement_ok": settlement_ok, "engine": self.economy.engine.id(),
            "timings": self.economy.timings().iter().rev().take(4).map(|t| json!({"epoch": t.epoch, "millis": t.millis, "particles": t.particles, "axons": t.axons})).collect::<Vec<_>>(),
        })
    }
}
