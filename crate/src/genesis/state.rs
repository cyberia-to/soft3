//! Apply a genesis file to a fresh [`bbg::BbgState`] (`specs/chain.md` §4.1,
//! "applying genesis") and commit once: the root is `state_root` of block 0.
//!
//! Every `b` becomes a balance entry and adds to its token's `coins` total;
//! every `l` runs bbg `insert` for one link (`token = τ_native`, `amount 0`,
//! `valence 0`) authored by `neuron` at chain height 0; every distinct owner
//! and link author gets a neuron record. Nothing else is written.
//!
//! bbg at the pinned revision has no `NeuronRecord.nonce` yet (chain.md
//! §3.2 asks for it with `DIMENSION_VERSION = 3`); when it lands, the root
//! moves and the blessed vector is re-blessed — a consensus change by
//! definition.

use std::collections::BTreeMap;

use bbg::state::balance_key;
use bbg::types::CoinRecord;
use bbg::{BbgState, Cyberlink, NeuronRecord, Signal};

use super::file::GenesisFile;
use super::token_of;

pub fn load(g: &GenesisFile) -> BbgState {
    let mut s = BbgState::new();
    let native = token_of(&g.params.denom);

    for b in &g.balances {
        s.neurons.entry(b.owner).or_insert(NeuronRecord { focus: 0, karma: 0, stake: 0 });
        let key = balance_key(&b.owner, &b.token);
        let slot = s.balances.entry(key).or_insert(0);
        *slot = slot.checked_add(b.amount).expect("balance fits u64 (unique (owner, token) checked)");
        let coin = s.coins.entry(b.token).or_insert(CoinRecord { total_supply: 0 });
        coin.total_supply = coin.total_supply.checked_add(b.amount).expect("token supply fits u64 (checked in derive)");
    }

    for l in &g.links {
        s.neurons.entry(l.neuron).or_insert(NeuronRecord { focus: 0, karma: 0, stake: 0 });
        let signal = Signal {
            neuron: l.neuron,
            links: vec![Cyberlink { from: l.p, to: l.q, token: native, amount: 0, valence: 0 }],
            box_moves: Vec::new(),
            height: 0,
        };
        s.insert(&signal).expect("no box moves, no double spend");
    }

    s.height = 0;
    s
}

/// Field elements each committed dimension serializes to (bbg `dim_serialize`:
/// 3 header fields, then 8 key limbs + the record's values per entry).
pub fn fields_per_dimension(s: &BbgState) -> BTreeMap<String, u64> {
    const HEADER: u64 = 3;
    const KEY: u64 = 8;
    let entries = |n: usize, values: u64| HEADER + n as u64 * (KEY + values);
    let list = |m: &BTreeMap<[u8; 32], Vec<[u8; 32]>>| HEADER + m.values().map(|v| KEY + 1 + 4 * v.len() as u64).sum::<u64>();
    let mut c = BTreeMap::new();
    c.insert("particles".into(), entries(s.particles.len(), 6));
    c.insert("axons_out".into(), list(&s.axons_out));
    c.insert("axons_in".into(), list(&s.axons_in));
    c.insert("neurons".into(), entries(s.neurons.len(), 3));
    c.insert("locations".into(), entries(s.locations.len(), 2));
    c.insert("coins".into(), entries(s.coins.len(), 1));
    c.insert("cards".into(), entries(s.cards.len(), 8));
    c.insert("files".into(), entries(s.files.len(), 2));
    c.insert("time".into(), entries(s.time.len(), 4));
    c.insert("signals".into(), entries(s.signals.len(), 14));
    c.insert("balances".into(), entries(s.balances.len(), 1));
    c
}

/// `lens::brakedown::MAX_VARIABLES`: one dimension commitment holds at most
/// `2^20` field elements at the pinned lens revision.
pub const BRAKEDOWN_MAX_VARIABLES: u32 = 20;

/// Dimensions whose serialized size exceeds the Brakedown limit. Empty means
/// `root()` can run.
pub fn over_limit(s: &BbgState) -> BTreeMap<String, String> {
    let limit = 1u64 << BRAKEDOWN_MAX_VARIABLES;
    fields_per_dimension(s)
        .into_iter()
        .filter(|(_, n)| *n > limit)
        .map(|(d, n)| {
            let vars = 64 - n.next_power_of_two().leading_zeros() - 1;
            (d, format!("{n} fields need 2^{vars} > 2^{BRAKEDOWN_MAX_VARIABLES} (lens Brakedown MAX_VARIABLES)"))
        })
        .collect()
}

/// Dimension sizes of a loaded state.
pub fn counts(s: &BbgState) -> BTreeMap<String, u64> {
    let mut c = BTreeMap::new();
    c.insert("particles".into(), s.particles.len() as u64);
    c.insert("axons_out".into(), s.axons_out.len() as u64);
    c.insert("axons_in".into(), s.axons_in.len() as u64);
    c.insert("axons".into(), s.axon_edges.len() as u64);
    c.insert("neurons".into(), s.neurons.len() as u64);
    c.insert("coins".into(), s.coins.len() as u64);
    c.insert("balances".into(), s.balances.len() as u64);
    c.insert("signals".into(), s.signals.len() as u64);
    c.insert("height".into(), s.height);
    c
}
