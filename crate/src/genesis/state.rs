//! Load a genesis manifest into a [`bbg::BbgState`] and commit once.
//!
//! This is the bulk-build path: every table lands in the dimension tables
//! first, the root is computed a single time at the end. A node that loads
//! the same manifest through this function reproduces the same root.

use std::collections::BTreeMap;

use bbg::state::balance_key;
use bbg::types::{CardRecord, CoinRecord, FileRecord, ParticleRecord};
use bbg::{BbgState, Cyberlink, NeuronRecord, Signal, SignalRecord};

use super::tables::{Genesis, network_of, token_of};

/// Build the genesis state. Infallible by construction: amounts already fit
/// `u64`, every id is 32 bytes, and bbg's only insert error (double spend)
/// cannot arise with no box moves.
pub fn load(g: &Genesis) -> BbgState {
    let mut s = BbgState::new();

    // neurons: one record per account, focus/karma/stake zero
    for a in &g.accounts {
        s.neurons.insert(a.owner, NeuronRecord { focus: 0, karma: 0, stake: 0 });
    }

    // coins + balances
    for (denom, supply) in &g.supply {
        let total_supply = u64::try_from(*supply).expect("genesis supply fits u64 (checked in derive)");
        s.coins.insert(token_of(denom), CoinRecord { total_supply });
    }
    for b in &g.balances {
        let key = balance_key(&b.owner, &token_of(&b.denom));
        let mut slot = s.balances.entry(key).or_insert(0);
        *slot = slot.checked_add(b.amount).expect("balance sum fits u64 (bounded by supply)");
    }

    // particles + files (black holes: no bytes at genesis)
    for p in &g.particles {
        s.particles.insert(p.particle, ParticleRecord::zero());
        s.files.insert(p.particle, FileRecord { available: false, chunk_count: 0 });
    }

    // cyberlinks: one signal per (neuron, height, tx) run of consecutive rows
    let network = network_of(g.chain.name);
    let mut position = 0u64;
    let mut i = 0;
    while i < g.links.len() {
        let head = &g.links[i];
        let mut j = i;
        let mut links = Vec::new();
        while j < g.links.len() {
            let l = &g.links[j];
            if l.address != head.address || l.height != head.height || l.tx != head.tx {
                break;
            }
            links.push(Cyberlink { from: l.from, to: l.to, token: [0u8; 32], amount: 0, valence: 1 });
            j += 1;
        }
        let signal = Signal { neuron: head.neuron, links, box_moves: Vec::new(), height: head.height };
        s.insert(&signal).expect("no box moves, no double spend");
        s.apply_signal_record(
            position,
            SignalRecord {
                neuron: head.neuron,
                network,
                link_count: u32::try_from(signal.links.len()).expect("one tx holds fewer than 2^32 links"),
                block_height: head.height,
                proof_hash: [0u8; 32],
            },
        );
        position += 1;
        i = j;
    }

    // passports → cards
    for p in &g.passports {
        s.cards.insert(p.card, CardRecord { owner: p.owner, particle: p.particle });
    }

    s.height = 0;
    s
}

/// Field elements each committed dimension serializes to (bbg `dim_serialize`:
/// 3 header fields, then 8 key limbs + the record's values per entry). A
/// dimension commits as a multilinear polynomial padded to the next power of
/// two; lens Brakedown refuses more than [`BRAKEDOWN_MAX_VARIABLES`] variables.
pub fn fields_per_dimension(s: &BbgState) -> BTreeMap<String, u64> {
    const HEADER: u64 = 3;
    const KEY: u64 = 8;
    let entries = |n: usize, values: u64| HEADER + n as u64 * (KEY + values);
    let list = |m: &BTreeMap<[u8; 32], Vec<[u8; 32]>>| {
        HEADER + m.values().map(|v| KEY + 1 + 4 * v.len() as u64).sum::<u64>()
    };
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

/// `lens::brakedown::MAX_VARIABLES`: the largest multilinear polynomial one
/// dimension commitment can hold is `2^20` field elements.
pub const BRAKEDOWN_MAX_VARIABLES: u32 = 20;

/// Dimensions whose serialized size exceeds the Brakedown limit, with the
/// variables they would need. Empty means `root()` can run.
pub fn over_limit(s: &BbgState) -> BTreeMap<String, String> {
    if s.root_version() == bbg::RootVersion::V2 {
        // the keyed trie has no size cap (bbg specs/state.md § root versions)
        return BTreeMap::new();
    }
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

/// Dimension sizes of a loaded state, for the manifest and the bijection check.
pub fn counts(s: &BbgState) -> BTreeMap<String, u64> {
    let mut c = BTreeMap::new();
    c.insert("particles".into(), s.particles.len() as u64);
    c.insert("axons_out".into(), s.axons_out.len() as u64);
    c.insert("axons_in".into(), s.axons_in.len() as u64);
    c.insert("axons".into(), s.axon_edges.len() as u64);
    c.insert("neurons".into(), s.neurons.len() as u64);
    c.insert("locations".into(), s.locations.len() as u64);
    c.insert("coins".into(), s.coins.len() as u64);
    c.insert("cards".into(), s.cards.len() as u64);
    c.insert("files".into(), s.files.len() as u64);
    c.insert("time".into(), s.time.len() as u64);
    c.insert("signals".into(), s.signals.len() as u64);
    c.insert("balances".into(), s.balances.len() as u64);
    c.insert("intents".into(), s.intents.len() as u64);
    c.insert("height".into(), s.height);
    c
}
